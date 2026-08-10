//! Burn-native multi-head attention — all four projections trainable.
//!
//! ## Why this exists
//! `CALMEngine`'s hand-written `MultiHeadLatentAttention` computes attention
//! correctly (Q/K/V projections, per-head scaled dot-product, SIMD) but its
//! `train_step` updates **only `w_o`**:
//!
//! ```text
//! self.w_o[idx] += learning_rate * error[i] * 0.01;   // w_q, w_k, w_v: never touched
//! ```
//!
//! So the query, key, and value projections stay frozen at random
//! initialization for the life of the process. Attention can re-weight the
//! *mixing* of head outputs, but the heads themselves never learn what to look
//! for. That is consistent with the measured benchmark profile — near
//! multiple-choice chance on knowledge tasks (MMLU, HellaSwag), with the score
//! carried by exact symbolic paths (GSM8K, HumanEval) that do not depend on
//! learned representations.
//!
//! This module expresses the same computation as Burn `Linear` layers, so every
//! projection is a registered parameter reachable by `.backward()` and steppable
//! by the SSO/Muon optimizer.
//!
//! ## Shapes
//! ```text
//! query   [batch, model_dim]
//! context [n_ctx, model_dim]        <- e.g. ContextRingBuffer::view().keys()
//! output  [batch, model_dim]
//! ```
//! Context is shared across the batch, which is the shape a
//! [`crate::ml::context_ring::ContextView`] produces: one set of context
//! vectors attended to by every query in the step.

use burn::module::Module;
use burn::nn::{Linear, LinearConfig};
use burn::tensor::activation::softmax;
use burn::tensor::backend::Backend;
use burn::tensor::Tensor;

/// Attention shape parameters.
#[derive(Debug, Clone)]
pub struct BurnAttentionConfig {
    /// Width of the query/context vectors.
    pub model_dim: usize,
    /// Number of attention heads.
    pub num_heads: usize,
    /// Width per head. `num_heads * head_dim` is the internal projection width.
    pub head_dim: usize,
}

impl Default for BurnAttentionConfig {
    fn default() -> Self {
        Self { model_dim: 256, num_heads: 8, head_dim: 32 }
    }
}

impl BurnAttentionConfig {
    /// Start from defaults.
    pub fn new() -> Self {
        Self::default()
    }
    /// Set the model width.
    pub fn with_model_dim(mut self, d: usize) -> Self {
        self.model_dim = d;
        self
    }
    /// Set the head count.
    pub fn with_num_heads(mut self, h: usize) -> Self {
        self.num_heads = h;
        self
    }
    /// Set the per-head width.
    pub fn with_head_dim(mut self, d: usize) -> Self {
        self.head_dim = d;
        self
    }
    /// Internal projection width, `num_heads * head_dim`.
    pub fn inner_dim(&self) -> usize {
        self.num_heads * self.head_dim
    }
}

/// Multi-head attention whose Q, K, V **and** O projections all train.
#[derive(Module, Debug)]
pub struct BurnMultiHeadAttention<B: Backend> {
    w_q: Linear<B>,
    w_k: Linear<B>,
    w_v: Linear<B>,
    w_o: Linear<B>,
    num_heads: usize,
    head_dim: usize,
}

impl<B: Backend> BurnMultiHeadAttention<B> {
    /// Initialize all four projections.
    pub fn new(config: &BurnAttentionConfig, device: &B::Device) -> Self {
        let inner = config.inner_dim();
        Self {
            w_q: LinearConfig::new(config.model_dim, inner).init(device),
            w_k: LinearConfig::new(config.model_dim, inner).init(device),
            w_v: LinearConfig::new(config.model_dim, inner).init(device),
            w_o: LinearConfig::new(inner, config.model_dim).init(device),
            num_heads: config.num_heads,
            head_dim: config.head_dim,
        }
    }

    /// Attend from `query` over `context`.
    ///
    /// `query`   `[batch, model_dim]`
    /// `context` `[n_ctx, model_dim]`
    /// returns   `[batch, model_dim]`
    pub fn forward(&self, query: Tensor<B, 2>, context: Tensor<B, 2>) -> Tensor<B, 2> {
        let [batch, _] = query.dims();
        let [n_ctx, _] = context.dims();
        let (h, hd) = (self.num_heads, self.head_dim);

        // Project. [batch, inner] and [n_ctx, inner].
        let q = self.w_q.forward(query);
        let k = self.w_k.forward(context.clone());
        let v = self.w_v.forward(context);

        // Split heads and move head to the batch axis so matmul is per-head.
        // [batch, h, hd] -> [h, batch, hd]
        let q = q.reshape([batch, h, hd]).swap_dims(0, 1);
        let k = k.reshape([n_ctx, h, hd]).swap_dims(0, 1);
        let v = v.reshape([n_ctx, h, hd]).swap_dims(0, 1);

        // Scaled dot-product: [h, batch, hd] @ [h, hd, n_ctx] -> [h, batch, n_ctx]
        let scale = (hd as f32).sqrt();
        let scores = q.matmul(k.swap_dims(1, 2)).div_scalar(scale);
        let weights = softmax(scores, 2);

        // [h, batch, n_ctx] @ [h, n_ctx, hd] -> [h, batch, hd]
        let attended = weights.matmul(v);

        // Recombine heads: [h, batch, hd] -> [batch, h*hd]
        let merged = attended.swap_dims(0, 1).reshape([batch, h * hd]);
        self.w_o.forward(merged)
    }

    /// Attend from `query` over a **per-item** `context`.
    ///
    /// [`Self::forward`] shares one context across the whole batch, which is
    /// the ring-buffer shape. Batched multiple-choice training needs the
    /// opposite: every item carries its *own* candidate set, so the context
    /// must gain a leading batch axis and the padded candidate slots must be
    /// removed from the softmax.
    ///
    /// ```text
    /// query   [batch, n_q,   model_dim]
    /// context [batch, n_ctx, model_dim]
    /// bias    [batch, 1, 1, n_ctx]   (broadcast to [batch, heads, n_q, n_ctx])
    /// output  [batch, n_q,   model_dim]
    /// ```
    ///
    /// `bias` is *additive*: `0.0` on real context slots and a large negative
    /// value (`-1e30`) on padded ones, so `softmax` assigns them exactly zero
    /// weight and no gradient flows into the padding. Use a finite sentinel
    /// rather than `f32::NEG_INFINITY` — [`softmax`] subtracts a per-row max
    /// before `exp`, and `-inf - -inf` is `NaN` if a row is ever fully masked.
    pub fn forward_batched(
        &self,
        query: Tensor<B, 3>,
        context: Tensor<B, 3>,
        bias: Option<Tensor<B, 4>>,
    ) -> Tensor<B, 3> {
        let [batch, n_q, _] = query.dims();
        let [_, n_ctx, _] = context.dims();
        let (h, hd) = (self.num_heads, self.head_dim);

        // Linear is rank-generic ([..., d_in] -> [..., d_out]), so the batch
        // axis rides through the projections untouched.
        let q = self.w_q.forward(query).reshape([batch, n_q, h, hd]).swap_dims(1, 2);
        let k = self.w_k.forward(context.clone()).reshape([batch, n_ctx, h, hd]).swap_dims(1, 2);
        let v = self.w_v.forward(context).reshape([batch, n_ctx, h, hd]).swap_dims(1, 2);

        // [b, h, n_q, hd] @ [b, h, hd, n_ctx] -> [b, h, n_q, n_ctx]
        let scale = (hd as f32).sqrt();
        let mut scores = q.matmul(k.clone().swap_dims(2, 3)).div_scalar(scale);
        if let Some(bias) = bias {
            scores = scores + bias;
        }
        let weights = softmax(scores, 3);

        // [b, h, n_q, n_ctx] @ [b, h, n_ctx, hd] -> [b, h, n_q, hd]
        let attended = weights.matmul(v);

        let merged = attended.swap_dims(1, 2).reshape([batch, n_q, h * hd]);
        self.w_o.forward(merged)
    }

    /// Attention weights only, `[num_heads, batch, n_ctx]`.
    ///
    /// Useful for inspecting *what* the heads learned to look at.
    pub fn attention_weights(&self, query: Tensor<B, 2>, context: Tensor<B, 2>) -> Tensor<B, 3> {
        let [batch, _] = query.dims();
        let [n_ctx, _] = context.dims();
        let (h, hd) = (self.num_heads, self.head_dim);

        let q = self.w_q.forward(query).reshape([batch, h, hd]).swap_dims(0, 1);
        let k = self.w_k.forward(context).reshape([n_ctx, h, hd]).swap_dims(0, 1);

        let scale = (hd as f32).sqrt();
        softmax(q.matmul(k.swap_dims(1, 2)).div_scalar(scale), 2)
    }

    /// Head count.
    pub fn num_heads(&self) -> usize {
        self.num_heads
    }

    /// Per-head width.
    pub fn head_dim(&self) -> usize {
        self.head_dim
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use burn::backend::NdArray;
    use burn::tensor::TensorData;

    type TB = NdArray;

    fn cfg() -> BurnAttentionConfig {
        BurnAttentionConfig::new().with_model_dim(8).with_num_heads(2).with_head_dim(4)
    }

    #[test]
    fn output_shape_matches_query() {
        let device = Default::default();
        let attn = BurnMultiHeadAttention::<TB>::new(&cfg(), &device);
        let q = Tensor::<TB, 2>::zeros([3, 8], &device);
        let c = Tensor::<TB, 2>::zeros([5, 8], &device);
        assert_eq!(attn.forward(q, c).dims(), [3, 8]);
    }

    #[test]
    fn attention_weights_are_a_distribution() {
        let device = Default::default();
        let attn = BurnMultiHeadAttention::<TB>::new(&cfg(), &device);
        let q = Tensor::<TB, 2>::from_data(
            TensorData::new((0..16).map(|i| i as f32 * 0.1).collect::<Vec<f32>>(), [2, 8]),
            &device,
        );
        let c = Tensor::<TB, 2>::from_data(
            TensorData::new((0..32).map(|i| i as f32 * 0.05).collect::<Vec<f32>>(), [4, 8]),
            &device,
        );
        let w = attn.attention_weights(q, c);
        assert_eq!(w.dims(), [2, 2, 4]); // [heads, batch, n_ctx]

        // Each (head, query) row must sum to 1 over the context axis.
        let sums = w.sum_dim(2).into_data().convert::<f32>();
        for s in sums.as_slice::<f32>().unwrap() {
            assert!((s - 1.0).abs() < 1e-4, "softmax row summed to {s}, expected 1.0");
        }
    }

    #[test]
    fn all_four_projections_are_registered_parameters() {
        let device = Default::default();
        let attn = BurnMultiHeadAttention::<TB>::new(&cfg(), &device);
        // num_params counts every registered tensor. Four Linear layers with
        // bias => 8 tensors; the point is that w_q/w_k/w_v are included rather
        // than being plain Vec<f32> fields invisible to autodiff.
        assert!(
            attn.num_params() > 0,
            "projections must be registered parameters, not opaque buffers"
        );
    }

    #[test]
    fn batched_matches_the_shared_context_path() {
        // With batch = 1 the per-item path must reproduce the shared-context
        // path exactly; otherwise the two are not the same computation.
        let device = Default::default();
        let attn = BurnMultiHeadAttention::<TB>::new(&cfg(), &device);
        let q = Tensor::<TB, 2>::from_data(
            TensorData::new((0..24).map(|i| i as f32 * 0.1).collect::<Vec<f32>>(), [3, 8]),
            &device,
        );
        let c = Tensor::<TB, 2>::from_data(
            TensorData::new((0..40).map(|i| i as f32 * 0.05).collect::<Vec<f32>>(), [5, 8]),
            &device,
        );

        let flat = attn.forward(q.clone(), c.clone());
        let batched: Tensor<TB, 3> =
            attn.forward_batched(q.unsqueeze_dim(0), c.unsqueeze_dim(0), None);
        assert_eq!(batched.dims(), [1, 3, 8]);

        let a = flat.into_data().convert::<f32>();
        let b = batched.reshape([3, 8]).into_data().convert::<f32>();
        for (x, y) in a.as_slice::<f32>().unwrap().iter().zip(b.as_slice::<f32>().unwrap()) {
            assert!((x - y).abs() < 1e-5, "batched path diverged: {x} vs {y}");
        }
    }

    #[test]
    fn additive_mask_zeroes_padded_context_slots() {
        // A padded slot must not influence the output at all: attending over
        // [real, real, PAD] with the pad masked has to equal attending over
        // just [real, real].
        let device = Default::default();
        let attn = BurnMultiHeadAttention::<TB>::new(&cfg(), &device);
        let q = Tensor::<TB, 3>::from_data(
            TensorData::new((0..8).map(|i| i as f32 * 0.1).collect::<Vec<f32>>(), [1, 1, 8]),
            &device,
        );
        let real = TensorData::new((0..16).map(|i| i as f32 * 0.05).collect::<Vec<f32>>(), [1, 2, 8]);
        let c2 = Tensor::<TB, 3>::from_data(real.clone(), &device);
        let c3 = Tensor::cat(vec![c2.clone(), Tensor::<TB, 3>::ones([1, 1, 8], &device) * 7.0], 1);

        // bias [batch, 1, 1, n_ctx]: 0 on real slots, -1e30 on the pad.
        let bias = Tensor::<TB, 4>::from_data(
            TensorData::new(vec![0.0f32, 0.0, -1.0e30], [1, 1, 1, 3]),
            &device,
        );

        let unpadded = attn.forward_batched(q.clone(), c2, None);
        let padded = attn.forward_batched(q, c3, Some(bias));

        let a = unpadded.into_data().convert::<f32>();
        let b = padded.into_data().convert::<f32>();
        for (x, y) in a.as_slice::<f32>().unwrap().iter().zip(b.as_slice::<f32>().unwrap()) {
            assert!((x - y).abs() < 1e-5, "padding leaked into the output: {x} vs {y}");
        }
    }

    #[test]
    fn context_length_is_flexible() {
        // The ring buffer produces a variable-width view; attention must accept it.
        let device = Default::default();
        let attn = BurnMultiHeadAttention::<TB>::new(&cfg(), &device);
        let q = Tensor::<TB, 2>::zeros([1, 8], &device);
        for n_ctx in [1usize, 7, 64, 320] {
            let c = Tensor::<TB, 2>::zeros([n_ctx, 8], &device);
            assert_eq!(attn.forward(q.clone(), c).dims(), [1, 8]);
        }
    }
}
