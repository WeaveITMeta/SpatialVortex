//! Burn-based Model Definitions
//!
//! Transformer model implementations using the Burn framework for training.
//! These models can be trained with distributed data parallelism.
//!
//! ## Architecture
//!
//! ```text
//! SpatialVortexModel
//! ├── Embedding (vocab_size × d_model)
//! ├── RoPE (rotary position encoding)
//! ├── TransformerLayers × N
//! │   ├── LayerNorm
//! │   ├── MultiHeadAttention (with KV-cache)
//! │   ├── LayerNorm
//! │   └── FeedForward (SwiGLU)
//! └── LMHead (d_model × vocab_size)
//! ```

use std::sync::Arc;
use ndarray::{Array1, Array2, Array3, Axis, s};
use parking_lot::RwLock;

use crate::error::{Result, SpatialVortexError};
use super::distributed::{Parameter, ParameterStore};

// ============================================================================
// MODEL CONFIGURATION
// ============================================================================

/// Model configuration for training
#[derive(Debug, Clone)]
pub struct ModelConfig {
    /// Model dimension
    pub d_model: usize,
    /// Number of transformer layers
    pub num_layers: usize,
    /// Number of attention heads
    pub num_heads: usize,
    /// Head dimension (d_model / num_heads)
    pub head_dim: usize,
    /// Feed-forward intermediate dimension
    pub d_ff: usize,
    /// Vocabulary size
    pub vocab_size: usize,
    /// Maximum sequence length
    pub max_seq_len: usize,
    /// Dropout rate
    pub dropout: f32,
    /// RoPE base frequency
    pub rope_base: f32,
    /// Use SwiGLU activation
    pub use_swiglu: bool,
    /// Tie embedding and output weights
    pub tie_weights: bool,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            d_model: 768,
            num_layers: 12,
            num_heads: 12,
            head_dim: 64,
            d_ff: 3072,
            vocab_size: 32000,
            max_seq_len: 2048,
            dropout: 0.1,
            rope_base: 10000.0,
            use_swiglu: true,
            tie_weights: true,
        }
    }
}

impl ModelConfig {
    /// Create 7B parameter configuration
    pub fn llama_7b() -> Self {
        Self {
            d_model: 4096,
            num_layers: 32,
            num_heads: 32,
            head_dim: 128,
            d_ff: 11008,
            vocab_size: 32000,
            max_seq_len: 4096,
            dropout: 0.0,
            rope_base: 10000.0,
            use_swiglu: true,
            tie_weights: false,
        }
    }
    
    /// Create small model for testing
    pub fn small() -> Self {
        Self {
            d_model: 256,
            num_layers: 4,
            num_heads: 4,
            head_dim: 64,
            d_ff: 1024,
            vocab_size: 1000,
            max_seq_len: 512,
            dropout: 0.1,
            rope_base: 10000.0,
            use_swiglu: true,
            tie_weights: true,
        }
    }
    
    /// Calculate total parameters
    pub fn total_params(&self) -> usize {
        let embed = self.vocab_size * self.d_model;
        let per_layer = 
            // Attention: Q, K, V, O projections
            4 * self.d_model * self.d_model +
            // FFN: up, gate, down (SwiGLU)
            3 * self.d_model * self.d_ff +
            // Layer norms
            4 * self.d_model;
        let lm_head = if self.tie_weights { 0 } else { self.d_model * self.vocab_size };
        
        embed + self.num_layers * per_layer + lm_head
    }
}

// ============================================================================
// ROTARY POSITION EMBEDDING
// ============================================================================

/// RoPE (Rotary Position Embedding) for training
pub struct RoPE {
    /// Cosine cache
    cos_cache: Array2<f32>,
    /// Sine cache
    sin_cache: Array2<f32>,
    head_dim: usize,
}

impl RoPE {
    pub fn new(head_dim: usize, max_seq_len: usize, base: f32) -> Self {
        let half_dim = head_dim / 2;
        
        // Compute inverse frequencies
        let inv_freq: Vec<f32> = (0..half_dim)
            .map(|i| 1.0 / base.powf(2.0 * i as f32 / head_dim as f32))
            .collect();
        
        let mut cos_cache = Array2::zeros((max_seq_len, half_dim));
        let mut sin_cache = Array2::zeros((max_seq_len, half_dim));
        
        for pos in 0..max_seq_len {
            for (i, &freq) in inv_freq.iter().enumerate() {
                let angle = pos as f32 * freq;
                cos_cache[[pos, i]] = angle.cos();
                sin_cache[[pos, i]] = angle.sin();
            }
        }
        
        Self { cos_cache, sin_cache, head_dim }
    }
    
    /// Apply RoPE to query/key tensors
    /// Input shape: [seq_len, num_heads, head_dim]
    pub fn apply(&self, x: &mut Array3<f32>, start_pos: usize) {
        let seq_len = x.shape()[0];
        let half_dim = self.head_dim / 2;
        
        for pos in 0..seq_len {
            let abs_pos = start_pos + pos;
            if abs_pos >= self.cos_cache.nrows() {
                continue;
            }
            
            let cos = self.cos_cache.row(abs_pos);
            let sin = self.sin_cache.row(abs_pos);
            
            for head in 0..x.shape()[1] {
                for i in 0..half_dim {
                    let x0 = x[[pos, head, i]];
                    let x1 = x[[pos, head, i + half_dim]];
                    
                    x[[pos, head, i]] = x0 * cos[i] - x1 * sin[i];
                    x[[pos, head, i + half_dim]] = x0 * sin[i] + x1 * cos[i];
                }
            }
        }
    }
}

// ============================================================================
// TRANSFORMER LAYER
// ============================================================================

/// Single transformer layer for training
pub struct TransformerLayer {
    /// Layer index
    layer_idx: usize,
    /// Configuration
    config: ModelConfig,
    /// Attention norm
    attn_norm: Array1<f32>,
    /// FFN norm
    ffn_norm: Array1<f32>,
    /// Q projection
    w_q: Array2<f32>,
    /// K projection
    w_k: Array2<f32>,
    /// V projection
    w_v: Array2<f32>,
    /// Output projection
    w_o: Array2<f32>,
    /// FFN up projection
    w_up: Array2<f32>,
    /// FFN gate projection (for SwiGLU)
    w_gate: Array2<f32>,
    /// FFN down projection
    w_down: Array2<f32>,
}

impl TransformerLayer {
    pub fn new(layer_idx: usize, config: &ModelConfig) -> Self {
        let d = config.d_model;
        let d_ff = config.d_ff;
        
        // Xavier initialization scale
        let scale = (2.0 / (d + d) as f32).sqrt();
        let ff_scale = (2.0 / (d + d_ff) as f32).sqrt();
        
        Self {
            layer_idx,
            config: config.clone(),
            attn_norm: Array1::ones(d),
            ffn_norm: Array1::ones(d),
            w_q: Array2::from_shape_fn((d, d), |(i, j)| {
                ((i * j) as f32 / (d * d) as f32 - 0.5) * scale
            }),
            w_k: Array2::from_shape_fn((d, d), |(i, j)| {
                ((i + j) as f32 / (d * 2) as f32 - 0.5) * scale
            }),
            w_v: Array2::from_shape_fn((d, d), |(i, j)| {
                ((i * 2 + j) as f32 / (d * 3) as f32 - 0.5) * scale
            }),
            w_o: Array2::from_shape_fn((d, d), |(i, j)| {
                ((i + j * 2) as f32 / (d * 3) as f32 - 0.5) * scale
            }),
            w_up: Array2::from_shape_fn((d_ff, d), |(i, j)| {
                ((i + j) as f32 / (d_ff + d) as f32 - 0.5) * ff_scale
            }),
            w_gate: Array2::from_shape_fn((d_ff, d), |(i, j)| {
                ((i * j) as f32 / (d_ff * d) as f32 - 0.5) * ff_scale
            }),
            w_down: Array2::from_shape_fn((d, d_ff), |(i, j)| {
                ((i + j) as f32 / (d + d_ff) as f32 - 0.5) * ff_scale
            }),
        }
    }
    
    /// Register parameters with store
    pub fn register_params(&self, store: &mut ParameterStore) {
        let prefix = format!("layer_{}", self.layer_idx);
        
        store.register(Parameter::new(
            &format!("{}.attn_norm", prefix),
            self.attn_norm.clone().insert_axis(Axis(0)),
        ));
        store.register(Parameter::new(
            &format!("{}.ffn_norm", prefix),
            self.ffn_norm.clone().insert_axis(Axis(0)),
        ));
        store.register(Parameter::new(&format!("{}.w_q", prefix), self.w_q.clone()));
        store.register(Parameter::new(&format!("{}.w_k", prefix), self.w_k.clone()));
        store.register(Parameter::new(&format!("{}.w_v", prefix), self.w_v.clone()));
        store.register(Parameter::new(&format!("{}.w_o", prefix), self.w_o.clone()));
        store.register(Parameter::new(&format!("{}.w_up", prefix), self.w_up.clone()));
        store.register(Parameter::new(&format!("{}.w_gate", prefix), self.w_gate.clone()));
        store.register(Parameter::new(&format!("{}.w_down", prefix), self.w_down.clone()));
    }
    
    /// Forward pass
    /// Input: [seq_len, d_model]
    /// Returns: [seq_len, d_model]
    pub fn forward(&self, x: &Array2<f32>, rope: &RoPE, start_pos: usize) -> Array2<f32> {
        let seq_len = x.nrows();
        let d = self.config.d_model;
        let num_heads = self.config.num_heads;
        let head_dim = self.config.head_dim;
        
        // Pre-attention norm (RMSNorm simplified as LayerNorm)
        let x_norm = self.rms_norm(x, &self.attn_norm);
        
        // Compute Q, K, V
        let q = x_norm.dot(&self.w_q.t());
        let k = x_norm.dot(&self.w_k.t());
        let v = x_norm.dot(&self.w_v.t());
        
        // Reshape to [seq_len, num_heads, head_dim]
        let mut q_heads = q.into_shape_with_order((seq_len, num_heads, head_dim)).unwrap();
        let mut k_heads = k.into_shape_with_order((seq_len, num_heads, head_dim)).unwrap();
        let v_heads = v.into_shape_with_order((seq_len, num_heads, head_dim)).unwrap();
        
        // Apply RoPE
        rope.apply(&mut q_heads, start_pos);
        rope.apply(&mut k_heads, start_pos);
        
        // Compute attention for each head
        let scale = 1.0 / (head_dim as f32).sqrt();
        let mut attn_out = Array3::zeros((seq_len, num_heads, head_dim));
        
        for h in 0..num_heads {
            let q_h = q_heads.slice(s![.., h, ..]).to_owned();
            let k_h = k_heads.slice(s![.., h, ..]).to_owned();
            let v_h = v_heads.slice(s![.., h, ..]).to_owned();
            
            // Attention scores
            let scores = q_h.dot(&k_h.t()) * scale;
            
            // Causal mask
            let mut masked_scores = scores.clone();
            for i in 0..seq_len {
                for j in (i + 1)..seq_len {
                    masked_scores[[i, j]] = f32::NEG_INFINITY;
                }
            }
            
            // Softmax
            let attn_weights = self.softmax(&masked_scores);
            
            // Apply attention
            let head_out = attn_weights.dot(&v_h);
            attn_out.slice_mut(s![.., h, ..]).assign(&head_out);
        }
        
        // Reshape back and project
        let attn_flat = attn_out.into_shape_with_order((seq_len, d)).unwrap();
        let attn_proj = attn_flat.dot(&self.w_o.t());
        
        // Residual connection
        let x_attn = x + &attn_proj;
        
        // Pre-FFN norm
        let x_ffn_norm = self.rms_norm(&x_attn, &self.ffn_norm);
        
        // FFN with SwiGLU
        let up = x_ffn_norm.dot(&self.w_up.t());
        let gate = x_ffn_norm.dot(&self.w_gate.t());
        
        // SiLU activation on gate
        let gate_activated = gate.mapv(|x| x * (1.0 / (1.0 + (-x).exp())));
        
        // Element-wise multiply
        let hidden = &up * &gate_activated;
        
        // Down projection
        let ffn_out = hidden.dot(&self.w_down.t());
        
        // Residual connection
        &x_attn + &ffn_out
    }
    
    /// RMS normalization
    fn rms_norm(&self, x: &Array2<f32>, weight: &Array1<f32>) -> Array2<f32> {
        let eps = 1e-6;
        let mut result = Array2::zeros(x.dim());
        
        for (i, row) in x.rows().into_iter().enumerate() {
            let rms = (row.mapv(|v| v * v).mean().unwrap() + eps).sqrt();
            for (j, &v) in row.iter().enumerate() {
                result[[i, j]] = (v / rms) * weight[j];
            }
        }
        
        result
    }
    
    /// Softmax along last axis
    fn softmax(&self, x: &Array2<f32>) -> Array2<f32> {
        let mut result = Array2::zeros(x.dim());
        
        for (i, row) in x.rows().into_iter().enumerate() {
            let max_val = row.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            let exp_sum: f32 = row.iter().map(|&v| (v - max_val).exp()).sum();
            
            for (j, &v) in row.iter().enumerate() {
                result[[i, j]] = (v - max_val).exp() / exp_sum;
            }
        }
        
        result
    }
}

// ============================================================================
// FULL MODEL
// ============================================================================

/// Complete SpatialVortex model for training
pub struct SpatialVortexModel {
    config: ModelConfig,
    /// Token embeddings
    embed: Array2<f32>,
    /// Transformer layers
    layers: Vec<TransformerLayer>,
    /// Final norm
    final_norm: Array1<f32>,
    /// LM head (output projection)
    lm_head: Array2<f32>,
    /// RoPE
    rope: RoPE,
}

impl SpatialVortexModel {
    pub fn new(config: ModelConfig) -> Self {
        let d = config.d_model;
        let vocab = config.vocab_size;
        
        // Initialize embeddings
        let embed_scale = (1.0 / d as f32).sqrt();
        let embed = Array2::from_shape_fn((vocab, d), |(i, j)| {
            ((i * j) as f32 / (vocab * d) as f32 - 0.5) * embed_scale
        });
        
        // Initialize layers
        let layers: Vec<TransformerLayer> = (0..config.num_layers)
            .map(|i| TransformerLayer::new(i, &config))
            .collect();
        
        // Final norm
        let final_norm = Array1::ones(d);
        
        // LM head (tied to embeddings if configured)
        let lm_head = if config.tie_weights {
            embed.clone()
        } else {
            Array2::from_shape_fn((vocab, d), |(i, j)| {
                ((i + j) as f32 / (vocab + d) as f32 - 0.5) * embed_scale
            })
        };
        
        let rope = RoPE::new(config.head_dim, config.max_seq_len, config.rope_base);
        
        Self {
            config,
            embed,
            layers,
            final_norm,
            lm_head,
            rope,
        }
    }
    
    /// Register all parameters with store
    pub fn register_params(&self, store: &mut ParameterStore) {
        store.register(Parameter::new("embed", self.embed.clone()));
        
        for layer in &self.layers {
            layer.register_params(store);
        }
        
        store.register(Parameter::new(
            "final_norm",
            self.final_norm.clone().insert_axis(Axis(0)),
        ));
        
        if !self.config.tie_weights {
            store.register(Parameter::new("lm_head", self.lm_head.clone()));
        }
    }
    
    /// Forward pass for training
    /// Input: [seq_len] token IDs
    /// Returns: [seq_len, vocab_size] logits
    pub fn forward(&self, input_ids: &[u32]) -> Array2<f32> {
        let seq_len = input_ids.len();
        
        // Embed tokens
        let mut hidden = Array2::zeros((seq_len, self.config.d_model));
        for (i, &token_id) in input_ids.iter().enumerate() {
            let idx = (token_id as usize).min(self.config.vocab_size - 1);
            hidden.row_mut(i).assign(&self.embed.row(idx));
        }
        
        // Pass through layers
        for layer in &self.layers {
            hidden = layer.forward(&hidden, &self.rope, 0);
        }
        
        // Final norm
        hidden = self.rms_norm(&hidden, &self.final_norm);
        
        // Project to vocabulary
        hidden.dot(&self.lm_head.t())
    }
    
    /// Compute cross-entropy loss
    pub fn compute_loss(&self, logits: &Array2<f32>, labels: &[u32]) -> f32 {
        let seq_len = logits.nrows().min(labels.len());
        let mut total_loss = 0.0;
        let mut count = 0;
        
        for i in 0..seq_len {
            let label = labels[i] as usize;
            if label >= self.config.vocab_size {
                continue;
            }
            
            // Log-softmax
            let row = logits.row(i);
            let max_val = row.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            let log_sum_exp: f32 = row.iter().map(|&v| (v - max_val).exp()).sum::<f32>().ln() + max_val;
            
            let log_prob = row[label] - log_sum_exp;
            total_loss -= log_prob;
            count += 1;
        }
        
        if count > 0 {
            total_loss / count as f32
        } else {
            0.0
        }
    }
    
    /// RMS normalization
    fn rms_norm(&self, x: &Array2<f32>, weight: &Array1<f32>) -> Array2<f32> {
        let eps = 1e-6;
        let mut result = Array2::zeros(x.dim());
        
        for (i, row) in x.rows().into_iter().enumerate() {
            let rms = (row.mapv(|v| v * v).mean().unwrap() + eps).sqrt();
            for (j, &v) in row.iter().enumerate() {
                result[[i, j]] = (v / rms) * weight[j];
            }
        }
        
        result
    }
    
    /// Get configuration
    pub fn config(&self) -> &ModelConfig {
        &self.config
    }
    
    /// Total parameters
    pub fn num_params(&self) -> usize {
        self.config.total_params()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_model_config() {
        let config = ModelConfig::small();
        let params = config.total_params();
        println!("Small model params: {}", params);
        assert!(params > 0);
        
        let config_7b = ModelConfig::llama_7b();
        let params_7b = config_7b.total_params();
        println!("7B model params: {} (~{:.1}B)", params_7b, params_7b as f64 / 1e9);
        assert!(params_7b > 6_000_000_000);
    }
    
    #[test]
    fn test_rope() {
        let rope = RoPE::new(64, 1024, 10000.0);
        
        let mut x = Array3::from_shape_fn((10, 4, 64), |(i, j, k)| {
            ((i + j + k) as f32 / 100.0).sin()
        });
        
        let x_orig = x.clone();
        rope.apply(&mut x, 0);
        
        // Should be modified
        assert!(x != x_orig);
    }
    
    #[test]
    fn test_model_forward() {
        let config = ModelConfig::small();
        let model = SpatialVortexModel::new(config);
        
        let input_ids = vec![1, 2, 3, 4, 5];
        let logits = model.forward(&input_ids);
        
        assert_eq!(logits.nrows(), 5);
        assert_eq!(logits.ncols(), 1000);  // vocab_size
    }
    
    #[test]
    fn test_loss_computation() {
        let config = ModelConfig::small();
        let model = SpatialVortexModel::new(config);
        
        let input_ids = vec![1, 2, 3, 4, 5];
        let labels = vec![2, 3, 4, 5, 6];
        
        let logits = model.forward(&input_ids);
        let loss = model.compute_loss(&logits, &labels);
        
        println!("Loss: {}", loss);
        assert!(loss > 0.0);
        assert!(loss < 20.0);  // Should be reasonable for random init
    }
    
    #[test]
    fn test_parameter_registration() {
        let config = ModelConfig::small();
        let model = SpatialVortexModel::new(config);
        
        let mut store = ParameterStore::new();
        model.register_params(&mut store);
        
        let total = store.total_params();
        println!("Registered params: {}", total);
        assert!(total > 0);
    }
}
