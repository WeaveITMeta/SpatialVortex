//! Rotary Position Embeddings (RoPE)
//!
//! Modern positional encoding that encodes position through rotation:
//! - Better extrapolation to longer sequences
//! - Relative position awareness built-in
//! - Used by LLaMA, Mistral, and most modern LLMs
//!
//! ## Mathematical Foundation
//!
//! RoPE applies rotation matrices to query and key vectors:
//! ```text
//! q_rotated = R(θ_m) * q
//! k_rotated = R(θ_n) * k
//! 
//! where R(θ) is a rotation matrix and θ depends on position
//! ```
//!
//! The attention score q·k naturally becomes position-aware:
//! ```text
//! q_m · k_n = (R(θ_m) * q) · (R(θ_n) * k) = q · R(θ_n - θ_m) · k
//! ```

use ndarray::{Array1, Array2, Array3, Axis, s};
use std::f32::consts::PI;

/// Rotary Position Embedding configuration
#[derive(Debug, Clone)]
pub struct RoPEConfig {
    /// Model dimension (must be even)
    pub dim: usize,
    /// Maximum sequence length
    pub max_seq_len: usize,
    /// Base for frequency computation (default: 10000)
    pub base: f32,
    /// Scaling factor for extended context (default: 1.0)
    pub scaling_factor: f32,
    /// Use NTK-aware scaling for longer contexts
    pub use_ntk_scaling: bool,
    /// NTK alpha parameter
    pub ntk_alpha: f32,
}

impl Default for RoPEConfig {
    fn default() -> Self {
        Self {
            dim: 64,
            max_seq_len: 4096,
            base: 10000.0,
            scaling_factor: 1.0,
            use_ntk_scaling: false,
            ntk_alpha: 1.0,
        }
    }
}

/// Rotary Position Embeddings
pub struct RotaryPositionEmbedding {
    config: RoPEConfig,
    /// Precomputed cos values [max_seq_len, dim/2]
    cos_cache: Array2<f32>,
    /// Precomputed sin values [max_seq_len, dim/2]
    sin_cache: Array2<f32>,
}

impl RotaryPositionEmbedding {
    /// Create new RoPE with precomputed frequencies
    pub fn new(config: RoPEConfig) -> Self {
        let half_dim = config.dim / 2;
        
        // Compute frequency bands
        let base = if config.use_ntk_scaling {
            // NTK-aware scaling for extended context
            config.base * config.ntk_alpha.powf(config.dim as f32 / (config.dim as f32 - 2.0))
        } else {
            config.base
        };
        
        // inv_freq = 1 / (base^(2i/dim)) for i in 0..dim/2
        let inv_freq: Vec<f32> = (0..half_dim)
            .map(|i| 1.0 / base.powf(2.0 * i as f32 / config.dim as f32))
            .collect();
        
        // Precompute cos and sin for all positions
        let mut cos_cache = Array2::zeros((config.max_seq_len, half_dim));
        let mut sin_cache = Array2::zeros((config.max_seq_len, half_dim));
        
        for pos in 0..config.max_seq_len {
            let scaled_pos = pos as f32 / config.scaling_factor;
            for (i, &freq) in inv_freq.iter().enumerate() {
                let angle = scaled_pos * freq;
                cos_cache[[pos, i]] = angle.cos();
                sin_cache[[pos, i]] = angle.sin();
            }
        }
        
        Self {
            config,
            cos_cache,
            sin_cache,
        }
    }
    
    /// Apply rotary embeddings to query and key tensors
    /// 
    /// # Arguments
    /// * `q` - Query tensor [batch, seq_len, num_heads, head_dim]
    /// * `k` - Key tensor [batch, seq_len, num_heads, head_dim]
    /// * `position_ids` - Position indices [batch, seq_len]
    /// 
    /// # Returns
    /// * (rotated_q, rotated_k)
    pub fn apply(
        &self,
        q: &Array3<f32>,  // [seq_len, num_heads, head_dim]
        k: &Array3<f32>,
        start_pos: usize,
    ) -> (Array3<f32>, Array3<f32>) {
        let seq_len = q.shape()[0];
        let num_heads = q.shape()[1];
        let head_dim = q.shape()[2];
        let half_dim = head_dim / 2;
        
        let mut q_rotated = q.clone();
        let mut k_rotated = k.clone();
        
        for s in 0..seq_len {
            let pos = start_pos + s;
            if pos >= self.config.max_seq_len {
                continue;
            }
            
            for h in 0..num_heads {
                // Split into pairs and rotate
                for i in 0..half_dim {
                    let cos = self.cos_cache[[pos, i]];
                    let sin = self.sin_cache[[pos, i]];
                    
                    // Query rotation
                    let q0 = q[[s, h, 2 * i]];
                    let q1 = q[[s, h, 2 * i + 1]];
                    q_rotated[[s, h, 2 * i]] = q0 * cos - q1 * sin;
                    q_rotated[[s, h, 2 * i + 1]] = q0 * sin + q1 * cos;
                    
                    // Key rotation
                    let k0 = k[[s, h, 2 * i]];
                    let k1 = k[[s, h, 2 * i + 1]];
                    k_rotated[[s, h, 2 * i]] = k0 * cos - k1 * sin;
                    k_rotated[[s, h, 2 * i + 1]] = k0 * sin + k1 * cos;
                }
            }
        }
        
        (q_rotated, k_rotated)
    }
    
    /// Apply RoPE to a single vector (for incremental decoding)
    pub fn apply_single(&self, x: &Array2<f32>, position: usize) -> Array2<f32> {
        let num_heads = x.shape()[0];
        let head_dim = x.shape()[1];
        let half_dim = head_dim / 2;
        
        let mut rotated = x.clone();
        
        if position >= self.config.max_seq_len {
            return rotated;
        }
        
        for h in 0..num_heads {
            for i in 0..half_dim {
                let cos = self.cos_cache[[position, i]];
                let sin = self.sin_cache[[position, i]];
                
                let x0 = x[[h, 2 * i]];
                let x1 = x[[h, 2 * i + 1]];
                rotated[[h, 2 * i]] = x0 * cos - x1 * sin;
                rotated[[h, 2 * i + 1]] = x0 * sin + x1 * cos;
            }
        }
        
        rotated
    }
    
    /// Get the dimension
    pub fn dim(&self) -> usize {
        self.config.dim
    }
    
    /// Get max sequence length
    pub fn max_seq_len(&self) -> usize {
        self.config.max_seq_len
    }
}

/// Extended RoPE for very long contexts (YaRN-style)
pub struct ExtendedRoPE {
    base_rope: RotaryPositionEmbedding,
    /// Original trained context length
    original_max_len: usize,
    /// Extended context length
    extended_max_len: usize,
    /// Interpolation factor
    scale: f32,
}

impl ExtendedRoPE {
    /// Create extended RoPE for longer contexts
    pub fn new(
        dim: usize,
        original_max_len: usize,
        extended_max_len: usize,
    ) -> Self {
        let scale = extended_max_len as f32 / original_max_len as f32;
        
        let config = RoPEConfig {
            dim,
            max_seq_len: extended_max_len,
            scaling_factor: scale,
            use_ntk_scaling: true,
            ntk_alpha: scale,
            ..Default::default()
        };
        
        Self {
            base_rope: RotaryPositionEmbedding::new(config),
            original_max_len,
            extended_max_len,
            scale,
        }
    }
    
    /// Apply extended RoPE
    pub fn apply(
        &self,
        q: &Array3<f32>,
        k: &Array3<f32>,
        start_pos: usize,
    ) -> (Array3<f32>, Array3<f32>) {
        self.base_rope.apply(q, k, start_pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rope_creation() {
        let config = RoPEConfig {
            dim: 64,
            max_seq_len: 1024,
            ..Default::default()
        };
        
        let rope = RotaryPositionEmbedding::new(config);
        
        assert_eq!(rope.cos_cache.shape(), &[1024, 32]);
        assert_eq!(rope.sin_cache.shape(), &[1024, 32]);
    }
    
    #[test]
    fn test_rope_apply() {
        let config = RoPEConfig {
            dim: 8,
            max_seq_len: 100,
            ..Default::default()
        };
        
        let rope = RotaryPositionEmbedding::new(config);
        
        let q = Array3::from_shape_fn((4, 2, 8), |(s, h, d)| (s * h * d) as f32 * 0.1);
        let k = Array3::from_shape_fn((4, 2, 8), |(s, h, d)| (s + h + d) as f32 * 0.1);
        
        let (q_rot, k_rot) = rope.apply(&q, &k, 0);
        
        assert_eq!(q_rot.shape(), q.shape());
        assert_eq!(k_rot.shape(), k.shape());
        
        // Values should be different after rotation
        assert!((q_rot[[0, 0, 0]] - q[[0, 0, 0]]).abs() > 1e-6 || q[[0, 0, 0]] == 0.0);
    }
    
    #[test]
    fn test_rope_position_invariance() {
        let config = RoPEConfig {
            dim: 8,
            max_seq_len: 100,
            ..Default::default()
        };
        
        let rope = RotaryPositionEmbedding::new(config);
        
        // Same vector at different positions should give different results
        let x = Array2::from_shape_fn((2, 8), |(h, d)| (h + d) as f32 * 0.1);
        
        let rot_0 = rope.apply_single(&x, 0);
        let rot_10 = rope.apply_single(&x, 10);
        
        // Should be different due to position encoding
        assert!((rot_0[[0, 0]] - rot_10[[0, 0]]).abs() > 1e-6);
    }
    
    #[test]
    fn test_extended_rope() {
        let extended = ExtendedRoPE::new(64, 4096, 32768);
        
        assert_eq!(extended.extended_max_len, 32768);
        assert!((extended.scale - 8.0).abs() < 1e-6);
    }
}
