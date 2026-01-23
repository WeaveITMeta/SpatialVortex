//! Vortex Math SGD optimizer using sacred sequences

use crate::change_dot::{ChangeDotIter, BackwardChain};

/// Training configuration
#[derive(Debug, Clone)]
pub struct TrainingConfig {
    /// Learning rate
    pub learning_rate: f64,
    /// Momentum coefficient
    pub momentum: f64,
    /// Sacred jump probability (15%)
    pub sacred_jump_prob: f64,
    /// Position 0 dropout probability (10%)
    pub center_dropout_prob: f64,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            learning_rate: 0.01,
            momentum: 0.9,
            sacred_jump_prob: 0.15,
            center_dropout_prob: 0.10,
        }
    }
}

/// Vortex Math SGD optimizer
///
/// Implements stochastic gradient descent using:
/// - Forward chain (1→2→4→8→7→5) for forward pass
/// - Backward chain (1→5→7→8→4→2) for backpropagation
/// - Sacred positions (3,6,9) as checkpoints
///
/// # Examples
///
/// ```
/// use spatial_vortex::training::{VortexSGD, TrainingConfig};
///
/// let config = TrainingConfig::default();
/// let mut sgd = VortexSGD::new(config);
///
/// // Forward pass through positions
/// let positions = sgd.forward_positions();
/// println!("Forward: {:?}", positions);
/// ```
pub struct VortexSGD {
    config: TrainingConfig,
    forward_chain: ChangeDotIter,
    backward_chain: BackwardChain,
    /// Current position in vortex cycle (0-9)
    /// Tracks runtime propagation through sacred geometry
    pub current_position: u8,
    /// Cycle count for exponential runtime tracking
    pub cycle_count: u64,
}

impl VortexSGD {
    /// Creates a new Vortex SGD optimizer
    pub fn new(config: TrainingConfig) -> Self {
        // Create a dummy engine for iterator initialization
        use crate::flux_matrix::FluxMatrixEngine;
        let engine = FluxMatrixEngine::new();
        
        Self {
            config,
            forward_chain: ChangeDotIter::from_value(1, &engine),
            backward_chain: BackwardChain::new(),
            current_position: 1,
            cycle_count: 0,
        }
    }
    
    /// Get current runtime position
    pub fn position(&self) -> u8 {
        self.current_position
    }
    
    /// Advance position through cycle
    pub fn advance_position(&mut self, direction: bool) {
        // direction: true = forward, false = backward
        if direction {
            // Forward: 1→2→4→8→7→5→1
            self.current_position = match self.current_position {
                1 => 2,
                2 => 4,
                4 => 8,
                8 => 7,
                7 => 5,
                5 => { self.cycle_count += 1; 1 },
                _ => 1,
            };
        } else {
            // Backward: 1→5→7→8→4→2→1
            self.current_position = match self.current_position {
                1 => 5,
                5 => 7,
                7 => 8,
                8 => 4,
                4 => 2,
                2 => { self.cycle_count += 1; 1 },
                _ => 1,
            };
        }
    }
    
    /// Get exponential runtime step (2^n based on cycle count)
    pub fn exponential_runtime(&self) -> u64 {
        2_u64.saturating_pow(self.cycle_count.min(63) as u32)
    }
    
    /// Returns forward propagation positions (1→2→4→8→7→5→1)
    pub fn forward_positions(&mut self) -> Vec<u8> {
        let mut positions = Vec::new();
        for _ in 0..6 {
            if let Some(event) = self.forward_chain.next() {
                match event {
                    crate::change_dot::ChangeDotEvent::Step { to, .. } => positions.push(to),
                    _ => {}
                }
            }
        }
        positions
    }
    
    /// Returns backward propagation positions (1→5→7→8→4→2→1)
    pub fn backward_positions(&mut self) -> Vec<u8> {
        let mut positions = Vec::new();
        for _ in 0..6 {
            if let Some(pos) = self.backward_chain.next() {
                positions.push(pos);
            }
        }
        positions
    }
    
    /// Performs forward pass
    ///
    /// Data flows through positions following doubling sequence
    pub fn forward_pass(&mut self, data: &[f64]) -> Vec<f64> {
        let positions = self.forward_positions();
        
        // Apply transformations at each position
        let mut activations = data.to_vec();
        for pos in positions {
            activations = self.apply_position_transform(pos, &activations);
        }
        
        activations
    }
    
    /// Performs backward pass
    ///
    /// Gradients flow backward through halving sequence
    pub fn backward_pass(&mut self, gradients: &[f64]) -> Vec<f64> {
        let positions = self.backward_positions();
        
        // Propagate gradients backward
        let mut grad = gradients.to_vec();
        for pos in positions {
            grad = self.apply_gradient_transform(pos, &grad);
        }
        
        grad
    }
    
    /// Applies transformation at a specific position
    fn apply_position_transform(&self, position: u8, data: &[f64]) -> Vec<f64> {
        // Position-specific transformation
        // In practice, this would apply learned weights
        data.iter()
            .map(|&x| x * self.position_scale(position))
            .collect()
    }
    
    /// Applies gradient transformation at a specific position
    fn apply_gradient_transform(&self, position: u8, gradients: &[f64]) -> Vec<f64> {
        // Gradient transformation with position-specific scaling
        gradients.iter()
            .map(|&g| g * self.position_scale(position))
            .collect()
    }
    
    /// Returns position-specific scale factor
    fn position_scale(&self, position: u8) -> f64 {
        match position {
            1 => 1.0,      // Unity
            2 => 2.0,      // Duality
            4 => 1.5,      // Foundation
            5 => 1.2,      // Change
            7 => 1.3,      // Completion
            8 => 1.4,      // Power
            _ => 1.0,
        }
    }
    
    /// Updates parameters using computed gradients
    pub fn step(&mut self, parameters: &mut [f64], gradients: &[f64]) {
        for (param, grad) in parameters.iter_mut().zip(gradients) {
            *param -= self.config.learning_rate * grad;
        }
    }
    
    /// Returns current learning rate
    pub fn learning_rate(&self) -> f64 {
        self.config.learning_rate
    }
    
    /// Sets learning rate
    pub fn set_learning_rate(&mut self, lr: f64) {
        self.config.learning_rate = lr;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vortex_sgd_creation() {
        let config = TrainingConfig::default();
        let sgd = VortexSGD::new(config);
        
        assert_eq!(sgd.learning_rate(), 0.01);
    }
    
    #[test]
    fn test_forward_positions() {
        let config = TrainingConfig::default();
        let mut sgd = VortexSGD::new(config);
        
        let positions = sgd.forward_positions();
        
        // Should follow doubling sequence: 1→2→4→8→7→5
        assert_eq!(positions.len(), 6);
        assert_eq!(positions[0], 1);
        assert_eq!(positions[1], 2);
        assert_eq!(positions[2], 4);
        assert_eq!(positions[3], 8);
        assert_eq!(positions[4], 7);
        assert_eq!(positions[5], 5);
    }
    
    #[test]
    fn test_backward_positions() {
        let config = TrainingConfig::default();
        let mut sgd = VortexSGD::new(config);
        
        let positions = sgd.backward_positions();
        
        // Should follow halving sequence: 1→5→7→8→4→2
        assert_eq!(positions.len(), 6);
        assert_eq!(positions[0], 1);
        assert_eq!(positions[1], 5);
        assert_eq!(positions[2], 7);
        assert_eq!(positions[3], 8);
        assert_eq!(positions[4], 4);
        assert_eq!(positions[5], 2);
    }
    
    #[test]
    fn test_forward_pass() {
        let config = TrainingConfig::default();
        let mut sgd = VortexSGD::new(config);
        
        let data = vec![1.0, 2.0, 3.0];
        let output = sgd.forward_pass(&data);
        
        assert_eq!(output.len(), data.len());
    }
    
    #[test]
    fn test_backward_pass() {
        let config = TrainingConfig::default();
        let mut sgd = VortexSGD::new(config);
        
        let gradients = vec![0.1, 0.2, 0.3];
        let grad_output = sgd.backward_pass(&gradients);
        
        assert_eq!(grad_output.len(), gradients.len());
    }
    
    #[test]
    fn test_parameter_update() {
        let config = TrainingConfig::default();
        let mut sgd = VortexSGD::new(config);
        
        let mut params = vec![1.0, 2.0, 3.0];
        let gradients = vec![0.1, 0.2, 0.3];
        
        sgd.step(&mut params, &gradients);
        
        // Parameters should decrease by learning_rate * gradient
        assert!((params[0] - 0.999).abs() < 0.001);
        assert!((params[1] - 1.998).abs() < 0.001);
        assert!((params[2] - 2.997).abs() < 0.001);
    }
}
