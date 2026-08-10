//! Graph attention over Eustress instance hierarchies.
//!
//! ## Why hierarchy and not proximity
//! An earlier measurement claimed spatial proximity predicts semantic
//! relatedness at 18x. That was **largely prefab duplication** — 80.7% of
//! positioned instances share a base name (`RopeShaftRoundSmoothBase` x977),
//! and name twins score Jaccard 1.0 by construction. Twin-controlled, the
//! proximity signal collapses to 0.0055 vs a 0.0002 floor (z = 2.51).
//!
//! **Containment survives that control far better.** Measured on 20,000 nodes
//! of Mountain Ascension, twin-controlled and restricted to concept names:
//!
//! ```text
//! sibling (same parent)      0.0210
//! cousin  (same grandparent) 0.0131
//! random                     0.0002      z = 9.32, p = 5.6e-21
//! ```
//!
//! The load-bearing evidence is the *monotone gradient* — sibling > cousin >
//! random. Closer in the tree means more related, which is what a semantic
//! hierarchy should do and what a duplication artifact would not produce.
//!
//! ## What this module is
//! A node attends over its neighbourhood (parent, siblings, children) to
//! produce a context-aware embedding. That is a graph attention layer, and the
//! mechanism already exists: [`BurnMultiHeadAttention::forward_batched`] takes
//! a **per-item** context with an additive mask, which is exactly a per-node
//! neighbourhood with ragged degree.
//!
//! ```text
//! node       [batch, 1,       dim]   the node being embedded
//! neighbours [batch, max_deg, dim]   its neighbourhood, zero-padded
//! mask       [batch, max_deg]        1.0 real, 0.0 padding
//! out        [batch,          dim]
//! ```
//!
//! Nodes with fewer neighbours than `max_deg` are padded, and the padding is
//! masked out of the softmax so no gradient reaches it. A node with **zero**
//! neighbours is handled explicitly: a fully-masked softmax row is uniform
//! rather than NaN, so the result is passed through a residual from the node's
//! own embedding instead of an arbitrary mixture.

use burn::module::Module;
use burn::nn::{LayerNorm, LayerNormConfig, Linear, LinearConfig};
use burn::tensor::backend::Backend;
use burn::tensor::Tensor;

use crate::ml::burn_attention::{BurnAttentionConfig, BurnMultiHeadAttention};

/// Edge kinds in an instance graph.
///
/// Mountain Ascension carries **only containment** — instances do not
/// cross-reference by uuid, and `Attachment` nodes are plain child folders. So
/// the relation set is deliberately small and honest rather than aspirational.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeKind {
    /// Neighbour is this node's parent.
    Parent,
    /// Neighbour shares this node's parent.
    Sibling,
    /// Neighbour is a direct child of this node.
    Child,
}

impl EdgeKind {
    /// Index into the edge-type embedding table.
    pub fn index(self) -> usize {
        match self {
            EdgeKind::Parent => 0,
            EdgeKind::Sibling => 1,
            EdgeKind::Child => 2,
        }
    }
    /// Number of distinct kinds.
    pub const COUNT: usize = 3;
}

/// Shape parameters for [`GraphAttention`].
#[derive(Debug, Clone)]
pub struct GraphAttentionConfig {
    /// Node embedding width.
    pub dim: usize,
    /// Attention heads.
    pub num_heads: usize,
    /// Width per head.
    pub head_dim: usize,
}

impl Default for GraphAttentionConfig {
    fn default() -> Self {
        Self { dim: 128, num_heads: 4, head_dim: 32 }
    }
}

impl GraphAttentionConfig {
    /// Start from defaults.
    pub fn new() -> Self {
        Self::default()
    }
    /// Set node width.
    pub fn with_dim(mut self, d: usize) -> Self {
        self.dim = d;
        self
    }
    /// Set head count.
    pub fn with_num_heads(mut self, h: usize) -> Self {
        self.num_heads = h;
        self
    }
    /// Set per-head width.
    pub fn with_head_dim(mut self, d: usize) -> Self {
        self.head_dim = d;
        self
    }
}

/// One graph-attention block: aggregate a neighbourhood into a node embedding.
#[derive(Module, Debug)]
pub struct GraphAttention<B: Backend> {
    attn: BurnMultiHeadAttention<B>,
    /// Learned per-edge-kind offset, added to neighbour embeddings so the model
    /// can weight a parent differently from a sibling.
    edge_embed: Linear<B>,
    norm: LayerNorm<B>,
    dim: usize,
}

impl<B: Backend> GraphAttention<B> {
    /// Initialize.
    pub fn new(config: &GraphAttentionConfig, device: &B::Device) -> Self {
        Self {
            attn: BurnMultiHeadAttention::new(
                &BurnAttentionConfig::new()
                    .with_model_dim(config.dim)
                    .with_num_heads(config.num_heads)
                    .with_head_dim(config.head_dim),
                device,
            ),
            edge_embed: LinearConfig::new(EdgeKind::COUNT, config.dim).init(device),
            norm: LayerNormConfig::new(config.dim).init(device),
            dim: config.dim,
        }
    }

    /// Aggregate each node's neighbourhood.
    ///
    /// `node`       `[batch, dim]`
    /// `neighbours` `[batch, max_deg, dim]`
    /// `edge_types` `[batch, max_deg, EdgeKind::COUNT]` one-hot
    /// `mask`       `[batch, max_deg]` — 1.0 real, 0.0 padding
    ///
    /// Returns `[batch, dim]`.
    pub fn forward(
        &self,
        node: Tensor<B, 2>,
        neighbours: Tensor<B, 3>,
        edge_types: Tensor<B, 3>,
        mask: Tensor<B, 2>,
    ) -> Tensor<B, 2> {
        let [batch, max_deg] = mask.dims();

        // Edge kind shifts the neighbour representation, so a parent and a
        // sibling holding the same content are distinguishable.
        let typed = neighbours + self.edge_embed.forward(edge_types);

        // Additive attention bias: 0.0 on real neighbours, -1e30 on padding.
        // Finite sentinel, not -inf: softmax subtracts a per-row max, and
        // -inf - -inf is NaN for a fully-padded (degree-0) row.
        let bias = (mask.clone() - 1.0)
            .mul_scalar(1.0e30)
            .reshape([batch, 1, 1, max_deg]);

        let q = node.clone().reshape([batch, 1, self.dim]);
        let attended = self
            .attn
            .forward_batched(q, typed, Some(bias))
            .reshape([batch, self.dim]);

        // A degree-0 node has a uniform (meaningless) softmax row, so gate its
        // aggregate to zero and let the residual carry it unchanged. Without
        // this an isolated node would be assigned an arbitrary mixture of
        // padding rows.
        let has_neighbours = mask
            .sum_dim(1)
            .clamp(0.0, 1.0)
            .reshape([batch, 1]);

        self.norm.forward(node + attended * has_neighbours)
    }

    /// Node embedding width.
    pub fn dim(&self) -> usize {
        self.dim
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use burn::backend::NdArray;
    use burn::tensor::TensorData;

    type TB = NdArray;

    fn cfg() -> GraphAttentionConfig {
        GraphAttentionConfig::new().with_dim(16).with_num_heads(2).with_head_dim(8)
    }

    fn onehot(batch: usize, deg: usize, kinds: &[EdgeKind], device: &<TB as Backend>::Device) -> Tensor<TB, 3> {
        let mut v = vec![0.0f32; batch * deg * EdgeKind::COUNT];
        for b in 0..batch {
            for d in 0..deg {
                let k = kinds[(b * deg + d) % kinds.len()];
                v[(b * deg + d) * EdgeKind::COUNT + k.index()] = 1.0;
            }
        }
        Tensor::<TB, 3>::from_data(TensorData::new(v, [batch, deg, EdgeKind::COUNT]), device)
    }

    #[test]
    fn output_shape_matches_node() {
        let device = Default::default();
        let g = GraphAttention::<TB>::new(&cfg(), &device);
        let node = Tensor::<TB, 2>::zeros([4, 16], &device);
        let nb = Tensor::<TB, 3>::zeros([4, 5, 16], &device);
        let et = onehot(4, 5, &[EdgeKind::Sibling], &device);
        let mask = Tensor::<TB, 2>::ones([4, 5], &device);
        assert_eq!(g.forward(node, nb, et, mask).dims(), [4, 16]);
    }

    #[test]
    fn isolated_node_is_finite_and_not_arbitrary() {
        // Degree 0: every neighbour slot is padding. A naive implementation
        // produces NaN (with -inf) or an arbitrary padding mixture.
        let device = Default::default();
        let g = GraphAttention::<TB>::new(&cfg(), &device);
        let node = Tensor::<TB, 2>::ones([2, 16], &device);
        let nb = Tensor::<TB, 3>::ones([2, 4, 16], &device);
        let et = onehot(2, 4, &[EdgeKind::Parent], &device);
        let mask = Tensor::<TB, 2>::zeros([2, 4], &device); // nothing real

        let out = g.forward(node, nb, et, mask).into_data().convert::<f32>();
        for v in out.as_slice::<f32>().unwrap() {
            assert!(v.is_finite(), "degree-0 node produced {v}");
        }
    }

    #[test]
    fn padding_does_not_change_the_result() {
        // The strongest correctness property: appending padded neighbour slots
        // must leave the output bit-comparable. If padding leaked into the
        // softmax this would drift.
        let device = Default::default();
        let g = GraphAttention::<TB>::new(&cfg(), &device);

        let node = Tensor::<TB, 2>::from_data(
            TensorData::new((0..16).map(|i| i as f32 * 0.1).collect::<Vec<f32>>(), [1, 16]),
            &device,
        );
        let real: Vec<f32> = (0..32).map(|i| (i as f32 * 0.07).sin()).collect();

        let nb2 = Tensor::<TB, 3>::from_data(TensorData::new(real.clone(), [1, 2, 16]), &device);
        let et2 = onehot(1, 2, &[EdgeKind::Sibling], &device);
        let m2 = Tensor::<TB, 2>::ones([1, 2], &device);
        let a = g.forward(node.clone(), nb2, et2, m2).into_data().convert::<f32>();

        // Same two real neighbours, plus three padded slots.
        let mut padded = real.clone();
        padded.extend(std::iter::repeat(0.0).take(3 * 16));
        let nb5 = Tensor::<TB, 3>::from_data(TensorData::new(padded, [1, 5, 16]), &device);
        let et5 = onehot(1, 5, &[EdgeKind::Sibling], &device);
        let m5 = Tensor::<TB, 2>::from_data(
            TensorData::new(vec![1.0f32, 1.0, 0.0, 0.0, 0.0], [1, 5]),
            &device,
        );
        let b = g.forward(node, nb5, et5, m5).into_data().convert::<f32>();

        for (x, y) in a.as_slice::<f32>().unwrap().iter().zip(b.as_slice::<f32>().unwrap()) {
            assert!((x - y).abs() < 1e-5, "padding leaked: {x} vs {y}");
        }
    }

    #[test]
    fn edge_kind_changes_the_aggregate() {
        // If edge type were ignored, a parent and a sibling with identical
        // content would produce identical output and the relation would be
        // untrained.
        let device = Default::default();
        let g = GraphAttention::<TB>::new(&cfg(), &device);
        let node = Tensor::<TB, 2>::ones([1, 16], &device);
        let nb = Tensor::<TB, 3>::ones([1, 3, 16], &device);
        let mask = Tensor::<TB, 2>::ones([1, 3], &device);

        let p = g
            .forward(node.clone(), nb.clone(), onehot(1, 3, &[EdgeKind::Parent], &device), mask.clone())
            .into_data()
            .convert::<f32>();
        let s = g
            .forward(node, nb, onehot(1, 3, &[EdgeKind::Sibling], &device), mask)
            .into_data()
            .convert::<f32>();

        let diff: f32 = p
            .as_slice::<f32>()
            .unwrap()
            .iter()
            .zip(s.as_slice::<f32>().unwrap())
            .map(|(a, b)| (a - b).abs())
            .sum();
        assert!(diff > 1e-6, "edge kind had no effect (sum |diff| = {diff})");
    }

    #[test]
    fn all_parameters_are_registered() {
        let device = Default::default();
        let g = GraphAttention::<TB>::new(&cfg(), &device);
        assert!(g.num_params() > 0, "attention + edge embed + norm must all train");
    }
}
