//! Gradient Checkpointing for Memory-Efficient Training
//!
//! Implements gradient checkpointing (activation checkpointing) to reduce
//! memory usage during training by trading compute for memory:
//!
//! ## How It Works
//!
//! ```text
//! Standard Training:
//!   Forward: Save ALL activations → High memory usage
//!   Backward: Use saved activations → Fast
//!
//! With Checkpointing:
//!   Forward: Save only CHECKPOINT activations → Low memory
//!   Backward: Recompute between checkpoints → Slower but memory efficient
//! ```
//!
//! ## Memory Savings
//!
//! For a model with N layers:
//! - Without checkpointing: O(N) memory for activations
//! - With checkpointing every √N layers: O(√N) memory
//! - Compute overhead: ~33% more forward passes
//!
//! ## Sacred Geometry Integration
//!
//! Checkpoints are placed at sacred positions (3, 6, 9, ...) for optimal
//! alignment with vortex mathematics principles.

use std::collections::HashMap;
use std::sync::Arc;
use ndarray::{Array2, Array3};
use parking_lot::RwLock;

/// Checkpoint strategy for gradient checkpointing
#[derive(Debug, Clone, PartialEq)]
pub enum CheckpointStrategy {
    /// No checkpointing (maximum memory, minimum compute)
    None,
    /// Checkpoint every N layers
    EveryN(usize),
    /// Checkpoint at sacred positions (3, 6, 9, ...)
    SacredPositions,
    /// Checkpoint at sqrt(num_layers) intervals
    SqrtN,
    /// Checkpoint all layers (minimum memory, maximum compute)
    All,
    /// Custom checkpoint positions
    Custom(Vec<usize>),
}

/// Configuration for gradient checkpointing
#[derive(Debug, Clone)]
pub struct CheckpointConfig {
    /// Checkpointing strategy
    pub strategy: CheckpointStrategy,
    /// Number of layers in the model
    pub num_layers: usize,
    /// Enable offloading checkpoints to CPU
    pub offload_to_cpu: bool,
    /// Use mixed precision for checkpoints
    pub mixed_precision: bool,
}

impl Default for CheckpointConfig {
    fn default() -> Self {
        Self {
            strategy: CheckpointStrategy::SacredPositions,
            num_layers: 32,
            offload_to_cpu: false,
            mixed_precision: false,
        }
    }
}

impl CheckpointConfig {
    /// Get checkpoint positions based on strategy
    pub fn get_checkpoint_positions(&self) -> Vec<usize> {
        match &self.strategy {
            CheckpointStrategy::None => vec![],
            CheckpointStrategy::EveryN(n) => {
                (0..self.num_layers).step_by(*n).collect()
            },
            CheckpointStrategy::SacredPositions => {
                // Checkpoint at positions where digital root is 3, 6, or 9
                (0..self.num_layers)
                    .filter(|&i| {
                        let dr = ((i % 9) + 1) as u8;
                        dr == 3 || dr == 6 || dr == 9
                    })
                    .collect()
            },
            CheckpointStrategy::SqrtN => {
                let interval = (self.num_layers as f32).sqrt().ceil() as usize;
                (0..self.num_layers).step_by(interval.max(1)).collect()
            },
            CheckpointStrategy::All => (0..self.num_layers).collect(),
            CheckpointStrategy::Custom(positions) => positions.clone(),
        }
    }
    
    /// Estimate memory savings factor
    pub fn memory_savings_factor(&self) -> f32 {
        let checkpoints = self.get_checkpoint_positions().len();
        if checkpoints == 0 {
            1.0  // No savings
        } else {
            self.num_layers as f32 / checkpoints as f32
        }
    }
    
    /// Estimate compute overhead factor
    pub fn compute_overhead_factor(&self) -> f32 {
        match self.strategy {
            CheckpointStrategy::None => 1.0,
            CheckpointStrategy::All => 2.0,  // Recompute everything
            _ => {
                let checkpoints = self.get_checkpoint_positions().len();
                if checkpoints == 0 {
                    1.0
                } else {
                    1.0 + (self.num_layers as f32 / checkpoints as f32 - 1.0) * 0.33
                }
            }
        }
    }
}

/// Activation checkpoint storage
pub struct ActivationCheckpoint {
    /// Layer index
    pub layer_idx: usize,
    /// Stored activation [seq_len, hidden_size]
    pub activation: Array2<f32>,
    /// Whether this is stored on CPU
    pub on_cpu: bool,
}

/// Gradient Checkpointing Manager
pub struct CheckpointManager {
    config: CheckpointConfig,
    /// Stored checkpoints
    checkpoints: RwLock<HashMap<usize, ActivationCheckpoint>>,
    /// Checkpoint positions
    checkpoint_positions: Vec<usize>,
    /// Statistics
    stats: RwLock<CheckpointStats>,
}

/// Checkpointing statistics
#[derive(Debug, Clone, Default)]
pub struct CheckpointStats {
    pub checkpoints_saved: usize,
    pub checkpoints_loaded: usize,
    pub recompute_count: usize,
    pub memory_saved_bytes: usize,
}

impl CheckpointManager {
    /// Create new checkpoint manager
    pub fn new(config: CheckpointConfig) -> Self {
        let checkpoint_positions = config.get_checkpoint_positions();
        
        Self {
            config,
            checkpoints: RwLock::new(HashMap::new()),
            checkpoint_positions,
            stats: RwLock::new(CheckpointStats::default()),
        }
    }
    
    /// Check if a layer should be checkpointed
    pub fn should_checkpoint(&self, layer_idx: usize) -> bool {
        self.checkpoint_positions.contains(&layer_idx)
    }
    
    /// Save activation checkpoint
    pub fn save_checkpoint(&self, layer_idx: usize, activation: Array2<f32>) {
        if !self.should_checkpoint(layer_idx) {
            return;
        }
        
        let checkpoint = ActivationCheckpoint {
            layer_idx,
            activation,
            on_cpu: self.config.offload_to_cpu,
        };
        
        let mut checkpoints = self.checkpoints.write();
        let mut stats = self.stats.write();
        
        stats.checkpoints_saved += 1;
        stats.memory_saved_bytes += checkpoint.activation.len() * 4;  // f32 = 4 bytes
        
        checkpoints.insert(layer_idx, checkpoint);
    }
    
    /// Load activation checkpoint
    pub fn load_checkpoint(&self, layer_idx: usize) -> Option<Array2<f32>> {
        let checkpoints = self.checkpoints.read();
        
        if let Some(checkpoint) = checkpoints.get(&layer_idx) {
            self.stats.write().checkpoints_loaded += 1;
            Some(checkpoint.activation.clone())
        } else {
            None
        }
    }
    
    /// Get the nearest checkpoint before a given layer
    pub fn get_nearest_checkpoint(&self, layer_idx: usize) -> Option<(usize, Array2<f32>)> {
        let checkpoints = self.checkpoints.read();
        
        // Find the largest checkpoint position <= layer_idx
        let nearest = self.checkpoint_positions.iter()
            .filter(|&&pos| pos <= layer_idx)
            .max()
            .copied();
        
        if let Some(pos) = nearest {
            if let Some(checkpoint) = checkpoints.get(&pos) {
                return Some((pos, checkpoint.activation.clone()));
            }
        }
        
        None
    }
    
    /// Clear all checkpoints
    pub fn clear(&self) {
        self.checkpoints.write().clear();
    }
    
    /// Get statistics
    pub fn get_stats(&self) -> CheckpointStats {
        self.stats.read().clone()
    }
    
    /// Get checkpoint positions
    pub fn get_positions(&self) -> &[usize] {
        &self.checkpoint_positions
    }
    
    /// Mark a recomputation
    pub fn mark_recompute(&self) {
        self.stats.write().recompute_count += 1;
    }
}

/// Mixed Precision Training Support
/// Simulates FP16/BF16 training in pure Rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrecisionMode {
    /// Full precision (FP32)
    FP32,
    /// Mixed precision with FP16 for forward/backward, FP32 for weights
    FP16,
    /// Mixed precision with BF16 (better range than FP16)
    BF16,
}

/// Mixed precision configuration
#[derive(Debug, Clone)]
pub struct MixedPrecisionConfig {
    pub mode: PrecisionMode,
    /// Loss scaling factor for FP16 stability
    pub loss_scale: f32,
    /// Dynamic loss scaling
    pub dynamic_loss_scaling: bool,
    /// Minimum loss scale
    pub min_loss_scale: f32,
    /// Scale update interval
    pub scale_update_interval: usize,
}

impl Default for MixedPrecisionConfig {
    fn default() -> Self {
        Self {
            mode: PrecisionMode::FP32,
            loss_scale: 65536.0,
            dynamic_loss_scaling: true,
            min_loss_scale: 1.0,
            scale_update_interval: 2000,
        }
    }
}

/// Mixed Precision Scaler
pub struct GradScaler {
    config: MixedPrecisionConfig,
    current_scale: f32,
    growth_factor: f32,
    backoff_factor: f32,
    growth_interval: usize,
    steps_since_growth: usize,
    found_inf: bool,
}

impl GradScaler {
    pub fn new(config: MixedPrecisionConfig) -> Self {
        Self {
            current_scale: config.loss_scale,
            growth_factor: 2.0,
            backoff_factor: 0.5,
            growth_interval: config.scale_update_interval,
            steps_since_growth: 0,
            found_inf: false,
            config,
        }
    }
    
    /// Scale loss for backward pass
    pub fn scale_loss(&self, loss: f32) -> f32 {
        if self.config.mode == PrecisionMode::FP32 {
            loss
        } else {
            loss * self.current_scale
        }
    }
    
    /// Unscale gradients after backward pass
    pub fn unscale_gradients(&mut self, gradients: &mut HashMap<String, Array2<f32>>) -> bool {
        if self.config.mode == PrecisionMode::FP32 {
            return true;
        }
        
        let inv_scale = 1.0 / self.current_scale;
        self.found_inf = false;
        
        for grad in gradients.values_mut() {
            for val in grad.iter_mut() {
                *val *= inv_scale;
                
                // Check for inf/nan
                if !val.is_finite() {
                    self.found_inf = true;
                }
            }
        }
        
        !self.found_inf
    }
    
    /// Update scale after optimizer step
    pub fn update(&mut self) {
        if self.config.mode == PrecisionMode::FP32 || !self.config.dynamic_loss_scaling {
            return;
        }
        
        if self.found_inf {
            // Reduce scale on overflow
            self.current_scale = (self.current_scale * self.backoff_factor)
                .max(self.config.min_loss_scale);
            self.steps_since_growth = 0;
        } else {
            self.steps_since_growth += 1;
            
            // Grow scale periodically
            if self.steps_since_growth >= self.growth_interval {
                self.current_scale *= self.growth_factor;
                self.steps_since_growth = 0;
            }
        }
    }
    
    /// Get current scale
    pub fn get_scale(&self) -> f32 {
        self.current_scale
    }
    
    /// Check if last step had overflow
    pub fn had_overflow(&self) -> bool {
        self.found_inf
    }
}

/// Simulated half-precision operations
pub mod half_precision {
    use ndarray::Array2;
    
    /// Simulate FP16 by clamping to representable range
    pub fn to_fp16(x: f32) -> f32 {
        const FP16_MAX: f32 = 65504.0;
        const FP16_MIN: f32 = -65504.0;
        const FP16_EPS: f32 = 0.0009765625;  // 2^-10
        
        if x.abs() < FP16_EPS {
            0.0
        } else {
            x.clamp(FP16_MIN, FP16_MAX)
        }
    }
    
    /// Convert array to simulated FP16
    pub fn array_to_fp16(arr: &Array2<f32>) -> Array2<f32> {
        arr.mapv(to_fp16)
    }
    
    /// Simulate BF16 (truncate mantissa)
    pub fn to_bf16(x: f32) -> f32 {
        let bits = x.to_bits();
        // BF16 keeps sign + exponent + 7 mantissa bits (truncate lower 16 bits)
        let bf16_bits = bits & 0xFFFF0000;
        f32::from_bits(bf16_bits)
    }
    
    /// Convert array to simulated BF16
    pub fn array_to_bf16(arr: &Array2<f32>) -> Array2<f32> {
        arr.mapv(to_bf16)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_checkpoint_config() {
        let config = CheckpointConfig {
            strategy: CheckpointStrategy::SacredPositions,
            num_layers: 32,
            ..Default::default()
        };
        
        let positions = config.get_checkpoint_positions();
        
        // Should include layers at digital root 3, 6, 9
        assert!(positions.contains(&2));   // 2+1=3
        assert!(positions.contains(&5));   // 5+1=6
        assert!(positions.contains(&8));   // 8+1=9
        assert!(positions.contains(&11));  // 11+1=12 -> 1+2=3
    }
    
    #[test]
    fn test_checkpoint_manager() {
        let config = CheckpointConfig {
            strategy: CheckpointStrategy::EveryN(4),
            num_layers: 16,
            ..Default::default()
        };
        
        let manager = CheckpointManager::new(config);
        
        assert!(manager.should_checkpoint(0));
        assert!(manager.should_checkpoint(4));
        assert!(manager.should_checkpoint(8));
        assert!(!manager.should_checkpoint(1));
        
        // Save and load
        let activation = Array2::from_shape_fn((4, 64), |(i, j)| (i * j) as f32);
        manager.save_checkpoint(4, activation.clone());
        
        let loaded = manager.load_checkpoint(4);
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().shape(), activation.shape());
    }
    
    #[test]
    fn test_grad_scaler() {
        let config = MixedPrecisionConfig {
            mode: PrecisionMode::FP16,
            loss_scale: 1024.0,
            dynamic_loss_scaling: true,
            ..Default::default()
        };
        
        let mut scaler = GradScaler::new(config);
        
        // Scale loss
        let scaled = scaler.scale_loss(1.0);
        assert_eq!(scaled, 1024.0);
        
        // Unscale gradients
        let mut grads = HashMap::new();
        grads.insert("test".to_string(), Array2::from_elem((2, 2), 1024.0));
        
        let success = scaler.unscale_gradients(&mut grads);
        assert!(success);
        
        // Check unscaled values
        assert!((grads["test"][[0, 0]] - 1.0).abs() < 1e-6);
    }
    
    #[test]
    fn test_memory_savings() {
        let config = CheckpointConfig {
            strategy: CheckpointStrategy::SqrtN,
            num_layers: 64,
            ..Default::default()
        };
        
        let savings = config.memory_savings_factor();
        let overhead = config.compute_overhead_factor();
        
        // sqrt(64) = 8 checkpoints, so ~8x memory savings
        assert!(savings > 7.0 && savings < 9.0);
        // Compute overhead should be reasonable
        assert!(overhead < 1.5);
    }
    
    #[test]
    fn test_half_precision() {
        use half_precision::*;
        
        // FP16 clamping
        assert_eq!(to_fp16(100000.0), 65504.0);
        assert_eq!(to_fp16(-100000.0), -65504.0);
        assert_eq!(to_fp16(0.0001), 0.0);  // Below epsilon
        
        // BF16 truncation
        let x = 1.234567f32;
        let bf16 = to_bf16(x);
        assert!((bf16 - x).abs() < 0.01);  // Should be close but not exact
    }
}
