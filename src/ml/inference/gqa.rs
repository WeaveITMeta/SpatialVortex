//! Grouped Query Attention (GQA)
//!
//! Memory-efficient attention mechanism that shares key-value heads:
//! - Reduces KV-cache memory by num_heads / num_kv_heads
//! - Maintains quality close to Multi-Head Attention (MHA)
//! - Used by LLaMA 2, Mistral, and modern efficient LLMs
//!
//! ## Architecture Comparison
//!
//! ```text
//! Multi-Head Attention (MHA):
//!   Q: [batch, seq, num_heads, head_dim]
//!   K: [batch, seq, num_heads, head_dim]  <- Full heads
//!   V: [batch, seq, num_heads, head_dim]  <- Full heads
//!
//! Grouped Query Attention (GQA):
//!   Q: [batch, seq, num_heads, head_dim]
//!   K: [batch, seq, num_kv_heads, head_dim]  <- Fewer heads
//!   V: [batch, seq, num_kv_heads, head_dim]  <- Fewer heads
//!
//! Multi-Query Attention (MQA):
//!   Q: [batch, seq, num_heads, head_dim]
//!   K: [batch, seq, 1, head_dim]  <- Single head
//!   V: [batch, seq, 1, head_dim]  <- Single head
//! ```

use ndarray::{Array1, Array2, Array3, Array4, Axis, s};
use std::sync::Arc;

use super::rope::RotaryPositionEmbedding;

/// GQA Configuration
#[derive(Debug, Clone)]
pub struct GQAConfig {
    /// Model dimension
    pub d_model: usize,
    /// Number of query heads
    pub num_heads: usize,
    /// Number of key-value heads (must divide num_heads evenly)
    pub num_kv_heads: usize,
    /// Head dimension (d_model / num_heads)
    pub head_dim: usize,
    /// Maximum sequence length
    pub max_seq_len: usize,
    /// Use RoPE for positional encoding
    pub use_rope: bool,
    /// Attention dropout (training only)
    pub dropout: f32,
    /// Use flash attention optimization
    pub use_flash_attention: bool,
}

impl Default for GQAConfig {
    fn default() -> Self {
        Self {
            d_model: 4096,
            num_heads: 32,
            num_kv_heads: 8,  // 4x reduction in KV cache
            head_dim: 128,
            max_seq_len: 4096,
            use_rope: true,
            dropout: 0.0,
            use_flash_attention: true,
        }
    }
}

impl GQAConfig {
    /// LLaMA 2 7B configuration
    pub fn llama2_7b() -> Self {
        Self {
            d_model: 4096,
            num_heads: 32,
            num_kv_heads: 32,  // LLaMA 2 7B uses MHA
            head_dim: 128,
            max_seq_len: 4096,
            use_rope: true,
            ..Default::default()
        }
    }
    
    /// LLaMA 2 70B configuration (uses GQA)
    pub fn llama2_70b() -> Self {
        Self {
            d_model: 8192,
            num_heads: 64,
            num_kv_heads: 8,  // 8x reduction
            head_dim: 128,
            max_seq_len: 4096,
            use_rope: true,
            ..Default::default()
        }
    }
    
    /// Mistral 7B configuration
    pub fn mistral_7b() -> Self {
        Self {
            d_model: 4096,
            num_heads: 32,
            num_kv_heads: 8,  // 4x reduction
            head_dim: 128,
            max_seq_len: 32768,  // Sliding window
            use_rope: true,
            ..Default::default()
        }
    }
    
    /// Get the group size (queries per KV head)
    pub fn group_size(&self) -> usize {
        self.num_heads / self.num_kv_heads
    }
    
    /// Get KV cache memory reduction factor
    pub fn kv_reduction_factor(&self) -> usize {
        self.num_heads / self.num_kv_heads
    }
}

/// Grouped Query Attention Layer
pub struct GroupedQueryAttention {
    config: GQAConfig,
    /// Query projection [d_model, num_heads * head_dim]
    w_q: Array2<f32>,
    /// Key projection [d_model, num_kv_heads * head_dim]
    w_k: Array2<f32>,
    /// Value projection [d_model, num_kv_heads * head_dim]
    w_v: Array2<f32>,
    /// Output projection [num_heads * head_dim, d_model]
    w_o: Array2<f32>,
    /// RoPE for positional encoding
    rope: Option<RotaryPositionEmbedding>,
}

impl GroupedQueryAttention {
    /// Create new GQA layer
    pub fn new(config: GQAConfig) -> Self {
        let d_model = config.d_model;
        let num_heads = config.num_heads;
        let num_kv_heads = config.num_kv_heads;
        let head_dim = config.head_dim;
        
        // Xavier initialization
        let scale_q = (2.0 / (d_model + num_heads * head_dim) as f32).sqrt();
        let scale_kv = (2.0 / (d_model + num_kv_heads * head_dim) as f32).sqrt();
        let scale_o = (2.0 / (num_heads * head_dim + d_model) as f32).sqrt();
        
        let w_q = Array2::from_shape_fn((d_model, num_heads * head_dim), |_| {
            (rand::random::<f32>() - 0.5) * scale_q
        });
        let w_k = Array2::from_shape_fn((d_model, num_kv_heads * head_dim), |_| {
            (rand::random::<f32>() - 0.5) * scale_kv
        });
        let w_v = Array2::from_shape_fn((d_model, num_kv_heads * head_dim), |_| {
            (rand::random::<f32>() - 0.5) * scale_kv
        });
        let w_o = Array2::from_shape_fn((num_heads * head_dim, d_model), |_| {
            (rand::random::<f32>() - 0.5) * scale_o
        });
        
        let rope = if config.use_rope {
            Some(RotaryPositionEmbedding::new(super::rope::RoPEConfig {
                dim: head_dim,
                max_seq_len: config.max_seq_len,
                ..Default::default()
            }))
        } else {
            None
        };
        
        Self {
            config,
            w_q,
            w_k,
            w_v,
            w_o,
            rope,
        }
    }
    
    /// Forward pass
    /// 
    /// # Arguments
    /// * `hidden` - Input hidden states [seq_len, d_model]
    /// * `kv_cache` - Optional (cached_k, cached_v) for incremental decoding
    /// * `start_pos` - Starting position for RoPE
    /// * `attention_mask` - Optional attention mask
    /// 
    /// # Returns
    /// * (output, new_k, new_v) for caching
    pub fn forward(
        &self,
        hidden: &Array2<f32>,
        kv_cache: Option<(&Array3<f32>, &Array3<f32>)>,
        start_pos: usize,
        attention_mask: Option<&Array2<f32>>,
    ) -> (Array2<f32>, Array3<f32>, Array3<f32>) {
        let seq_len = hidden.nrows();
        let num_heads = self.config.num_heads;
        let num_kv_heads = self.config.num_kv_heads;
        let head_dim = self.config.head_dim;
        let group_size = self.config.group_size();
        
        // Project to Q, K, V
        let q_proj = hidden.dot(&self.w_q);  // [seq_len, num_heads * head_dim]
        let k_proj = hidden.dot(&self.w_k);  // [seq_len, num_kv_heads * head_dim]
        let v_proj = hidden.dot(&self.w_v);  // [seq_len, num_kv_heads * head_dim]
        
        // Reshape to [seq_len, num_heads/kv_heads, head_dim]
        let q = q_proj.into_shape((seq_len, num_heads, head_dim)).unwrap();
        let k = k_proj.into_shape((seq_len, num_kv_heads, head_dim)).unwrap();
        let v = v_proj.into_shape((seq_len, num_kv_heads, head_dim)).unwrap();
        
        // Apply RoPE if enabled
        let (q, k) = if let Some(ref rope) = self.rope {
            // Need to expand K for RoPE application, then contract
            let k_expanded = self.expand_kv(&k, group_size);
            let (q_rot, k_rot) = rope.apply(&q, &k_expanded, start_pos);
            let k_contracted = self.contract_kv(&k_rot, group_size);
            (q_rot, k_contracted)
        } else {
            (q, k)
        };
        
        // Concatenate with cache if present
        let (k_full, v_full) = if let Some((cached_k, cached_v)) = kv_cache {
            let k_cat = ndarray::concatenate(Axis(0), &[cached_k.view(), k.view()]).unwrap();
            let v_cat = ndarray::concatenate(Axis(0), &[cached_v.view(), v.view()]).unwrap();
            (k_cat, v_cat)
        } else {
            (k.clone(), v.clone())
        };
        
        // Compute attention
        let output = self.compute_attention(&q, &k_full, &v_full, attention_mask);
        
        // Project output
        let output_flat = output.into_shape((seq_len, num_heads * head_dim)).unwrap();
        let output_proj = output_flat.dot(&self.w_o);
        
        (output_proj, k, v)
    }
    
    /// Compute grouped attention scores
    fn compute_attention(
        &self,
        q: &Array3<f32>,      // [seq_len, num_heads, head_dim]
        k: &Array3<f32>,      // [cache_len, num_kv_heads, head_dim]
        v: &Array3<f32>,      // [cache_len, num_kv_heads, head_dim]
        mask: Option<&Array2<f32>>,
    ) -> Array3<f32> {
        let seq_len = q.shape()[0];
        let num_heads = q.shape()[1];
        let head_dim = q.shape()[2];
        let cache_len = k.shape()[0];
        let num_kv_heads = k.shape()[1];
        let group_size = num_heads / num_kv_heads;
        
        let scale = 1.0 / (head_dim as f32).sqrt();
        let mut output = Array3::zeros((seq_len, num_heads, head_dim));
        
        // Process each query head
        for h in 0..num_heads {
            let kv_head = h / group_size;  // Which KV head to use
            
            // Get Q slice for this head
            let q_head = q.slice(s![.., h, ..]);  // [seq_len, head_dim]
            
            // Get K, V slices for corresponding KV head
            let k_head = k.slice(s![.., kv_head, ..]);  // [cache_len, head_dim]
            let v_head = v.slice(s![.., kv_head, ..]);  // [cache_len, head_dim]
            
            // Compute attention scores: Q @ K^T
            let scores = q_head.dot(&k_head.t()) * scale;  // [seq_len, cache_len]
            
            // Apply mask if present
            let scores = if let Some(m) = mask {
                let mut masked = scores.clone();
                for i in 0..seq_len {
                    for j in 0..cache_len {
                        if m[[i, j]] < 0.5 {
                            masked[[i, j]] = f32::NEG_INFINITY;
                        }
                    }
                }
                masked
            } else {
                scores
            };
            
            // Softmax
            let attn_weights = self.softmax_2d(&scores);
            
            // Apply attention to values
            let attended = attn_weights.dot(&v_head.to_owned());  // [seq_len, head_dim]
            
            // Store in output
            for s in 0..seq_len {
                for d in 0..head_dim {
                    output[[s, h, d]] = attended[[s, d]];
                }
            }
        }
        
        output
    }
    
    /// Expand KV heads to match query heads (for RoPE)
    fn expand_kv(&self, kv: &Array3<f32>, group_size: usize) -> Array3<f32> {
        let seq_len = kv.shape()[0];
        let num_kv_heads = kv.shape()[1];
        let head_dim = kv.shape()[2];
        let num_heads = num_kv_heads * group_size;
        
        let mut expanded = Array3::zeros((seq_len, num_heads, head_dim));
        
        for s in 0..seq_len {
            for kv_h in 0..num_kv_heads {
                for g in 0..group_size {
                    let h = kv_h * group_size + g;
                    for d in 0..head_dim {
                        expanded[[s, h, d]] = kv[[s, kv_h, d]];
                    }
                }
            }
        }
        
        expanded
    }
    
    /// Contract expanded KV back to original size
    fn contract_kv(&self, kv: &Array3<f32>, group_size: usize) -> Array3<f32> {
        let seq_len = kv.shape()[0];
        let num_heads = kv.shape()[1];
        let head_dim = kv.shape()[2];
        let num_kv_heads = num_heads / group_size;
        
        let mut contracted = Array3::zeros((seq_len, num_kv_heads, head_dim));
        
        for s in 0..seq_len {
            for kv_h in 0..num_kv_heads {
                // Take first head in group (they should be identical after RoPE)
                let h = kv_h * group_size;
                for d in 0..head_dim {
                    contracted[[s, kv_h, d]] = kv[[s, h, d]];
                }
            }
        }
        
        contracted
    }
    
    /// Row-wise softmax
    fn softmax_2d(&self, x: &Array2<f32>) -> Array2<f32> {
        let mut result = x.clone();
        for mut row in result.rows_mut() {
            let max = row.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            row.mapv_inplace(|v| (v - max).exp());
            let sum: f32 = row.sum();
            if sum > 0.0 {
                row.mapv_inplace(|v| v / sum);
            }
        }
        result
    }
    
    /// Get weight references for training
    pub fn get_weights(&self) -> (&Array2<f32>, &Array2<f32>, &Array2<f32>, &Array2<f32>) {
        (&self.w_q, &self.w_k, &self.w_v, &self.w_o)
    }
    
    /// Get mutable weight references for training
    pub fn get_weights_mut(&mut self) -> (&mut Array2<f32>, &mut Array2<f32>, &mut Array2<f32>, &mut Array2<f32>) {
        (&mut self.w_q, &mut self.w_k, &mut self.w_v, &mut self.w_o)
    }
}

/// GQA-based KV Cache with reduced memory
pub struct GQAKVCache {
    /// Cached keys [num_layers, cache_len, num_kv_heads, head_dim]
    keys: Vec<Array3<f32>>,
    /// Cached values
    values: Vec<Array3<f32>>,
    /// Current sequence lengths per layer
    seq_lens: Vec<usize>,
    /// Configuration
    num_layers: usize,
    num_kv_heads: usize,
    head_dim: usize,
    max_len: usize,
}

impl GQAKVCache {
    pub fn new(num_layers: usize, num_kv_heads: usize, head_dim: usize, max_len: usize) -> Self {
        Self {
            keys: vec![Array3::zeros((0, num_kv_heads, head_dim)); num_layers],
            values: vec![Array3::zeros((0, num_kv_heads, head_dim)); num_layers],
            seq_lens: vec![0; num_layers],
            num_layers,
            num_kv_heads,
            head_dim,
            max_len,
        }
    }
    
    /// Append new KV to cache
    pub fn append(&mut self, layer: usize, k: &Array3<f32>, v: &Array3<f32>) {
        if layer >= self.num_layers {
            return;
        }
        
        self.keys[layer] = ndarray::concatenate(
            Axis(0),
            &[self.keys[layer].view(), k.view()]
        ).unwrap_or(k.clone());
        
        self.values[layer] = ndarray::concatenate(
            Axis(0),
            &[self.values[layer].view(), v.view()]
        ).unwrap_or(v.clone());
        
        self.seq_lens[layer] = self.keys[layer].shape()[0];
        
        // Trim if exceeding max
        if self.seq_lens[layer] > self.max_len {
            let trim = self.seq_lens[layer] - self.max_len;
            self.keys[layer] = self.keys[layer].slice(s![trim.., .., ..]).to_owned();
            self.values[layer] = self.values[layer].slice(s![trim.., .., ..]).to_owned();
            self.seq_lens[layer] = self.max_len;
        }
    }
    
    /// Get cached KV for a layer
    pub fn get(&self, layer: usize) -> Option<(&Array3<f32>, &Array3<f32>)> {
        if layer < self.num_layers && self.seq_lens[layer] > 0 {
            Some((&self.keys[layer], &self.values[layer]))
        } else {
            None
        }
    }
    
    /// Clear cache
    pub fn clear(&mut self) {
        for layer in 0..self.num_layers {
            self.keys[layer] = Array3::zeros((0, self.num_kv_heads, self.head_dim));
            self.values[layer] = Array3::zeros((0, self.num_kv_heads, self.head_dim));
            self.seq_lens[layer] = 0;
        }
    }
    
    /// Get memory usage in bytes (approximate)
    pub fn memory_bytes(&self) -> usize {
        let total_elements: usize = self.seq_lens.iter()
            .map(|&len| len * self.num_kv_heads * self.head_dim * 2)  // K + V
            .sum();
        total_elements * 4  // f32 = 4 bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_gqa_config() {
        let config = GQAConfig::default();
        assert_eq!(config.group_size(), 4);  // 32 / 8
        assert_eq!(config.kv_reduction_factor(), 4);
    }
    
    #[test]
    fn test_gqa_forward() {
        let config = GQAConfig {
            d_model: 256,
            num_heads: 8,
            num_kv_heads: 2,
            head_dim: 32,
            max_seq_len: 100,
            use_rope: false,
            ..Default::default()
        };
        
        let gqa = GroupedQueryAttention::new(config);
        
        let hidden = Array2::from_shape_fn((4, 256), |(s, d)| (s * d) as f32 * 0.001);
        
        let (output, k, v) = gqa.forward(&hidden, None, 0, None);
        
        assert_eq!(output.shape(), &[4, 256]);
        assert_eq!(k.shape(), &[4, 2, 32]);  // num_kv_heads = 2
        assert_eq!(v.shape(), &[4, 2, 32]);
    }
    
    #[test]
    fn test_gqa_with_cache() {
        let config = GQAConfig {
            d_model: 128,
            num_heads: 4,
            num_kv_heads: 2,
            head_dim: 32,
            max_seq_len: 100,
            use_rope: false,
            ..Default::default()
        };
        
        let gqa = GroupedQueryAttention::new(config);
        
        // First forward
        let hidden1 = Array2::from_shape_fn((3, 128), |(s, d)| (s + d) as f32 * 0.01);
        let (_, k1, v1) = gqa.forward(&hidden1, None, 0, None);
        
        // Second forward with cache
        let hidden2 = Array2::from_shape_fn((1, 128), |(s, d)| (s * d) as f32 * 0.01);
        let (output, k2, v2) = gqa.forward(&hidden2, Some((&k1, &v1)), 3, None);
        
        assert_eq!(output.shape(), &[1, 128]);
        assert_eq!(k2.shape(), &[1, 2, 32]);
    }
    
    #[test]
    fn test_gqa_kv_cache() {
        let mut cache = GQAKVCache::new(2, 4, 32, 100);
        
        let k = Array3::from_shape_fn((2, 4, 32), |(s, h, d)| (s * h * d) as f32);
        let v = Array3::from_shape_fn((2, 4, 32), |(s, h, d)| (s + h + d) as f32);
        
        cache.append(0, &k, &v);
        
        let (cached_k, cached_v) = cache.get(0).unwrap();
        assert_eq!(cached_k.shape(), &[2, 4, 32]);
        
        // Memory should be reasonable
        let mem = cache.memory_bytes();
        assert!(mem > 0);
    }
}
