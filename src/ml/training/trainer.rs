//! Training Infrastructure with Backpropagation
//!
//! Implements the complete training loop using the **halving sequence** for
//! error correction and weight updates:
//!
//! **Halving Sequence** (Backward Chain Propagation):
//! ```
//! 1 → 5 → 7 → 8 → 4 → 2 → 1 (reverse of doubling)
//! ```
//!
//! ## Training Loop
//!
//! 1. **Forward Pass**: Model generates output from infinite loop
//! 2. **Compute Loss**: Difference from target (cross-entropy, MSE)
//! 3. **Backward Pass**: Compute gradients via chain rule (halving sequence)
//! 4. **Optimizer**: Update weights, biases, and attributes
//! 5. **Repeat**: Billions of times across terabytes of data
//!
//! Goal: Train on trillions of tokens and hundreds of billions of parameters

use ndarray::Array2;
use std::collections::HashMap;

/// Loss Functions for Training
pub enum LossFunction {
    /// Cross-Entropy Loss (classification)
    /// L = -Σ y_true · log(y_pred)
    CrossEntropy,
    
    /// Mean Squared Error (regression)
    /// L = (1/n) · Σ (y_true - y_pred)²
    MeanSquaredError,
    
    /// Binary Cross-Entropy
    /// L = -[y·log(p) + (1-y)·log(1-p)]
    BinaryCrossEntropy,
    
    /// Huber Loss (robust to outliers)
    Huber { delta: f32 },
}

impl LossFunction {
    /// Compute loss
    pub fn compute(&self, y_true: &Array2<f32>, y_pred: &Array2<f32>) -> f32 {
        match self {
            LossFunction::CrossEntropy => {
                self.cross_entropy(y_true, y_pred)
            },
            LossFunction::MeanSquaredError => {
                self.mse(y_true, y_pred)
            },
            LossFunction::BinaryCrossEntropy => {
                self.binary_cross_entropy(y_true, y_pred)
            },
            LossFunction::Huber { delta } => {
                self.huber(y_true, y_pred, *delta)
            },
        }
    }
    
    /// Compute gradient of loss
    pub fn gradient(&self, y_true: &Array2<f32>, y_pred: &Array2<f32>) -> Array2<f32> {
        match self {
            LossFunction::CrossEntropy => {
                self.cross_entropy_gradient(y_true, y_pred)
            },
            LossFunction::MeanSquaredError => {
                self.mse_gradient(y_true, y_pred)
            },
            LossFunction::BinaryCrossEntropy => {
                self.binary_cross_entropy_gradient(y_true, y_pred)
            },
            LossFunction::Huber { delta } => {
                self.huber_gradient(y_true, y_pred, *delta)
            },
        }
    }
    
    fn cross_entropy(&self, y_true: &Array2<f32>, y_pred: &Array2<f32>) -> f32 {
        let eps = 1e-7;  // Numerical stability
        let loss: f32 = y_true
            .iter()
            .zip(y_pred.iter())
            .map(|(t, p)| {
                -t * (p + eps).ln()
            })
            .sum();
        loss / y_true.nrows() as f32
    }
    
    fn cross_entropy_gradient(&self, y_true: &Array2<f32>, y_pred: &Array2<f32>) -> Array2<f32> {
        let eps = 1e-7;
        let n = y_true.nrows() as f32;
        (y_pred - y_true) / (y_pred.mapv(|p| p + eps) * n)
    }
    
    fn mse(&self, y_true: &Array2<f32>, y_pred: &Array2<f32>) -> f32 {
        let diff = y_pred - y_true;
        diff.mapv(|x| x.powi(2)).sum() / (2.0 * y_true.nrows() as f32)
    }
    
    fn mse_gradient(&self, y_true: &Array2<f32>, y_pred: &Array2<f32>) -> Array2<f32> {
        let n = y_true.nrows() as f32;
        (y_pred - y_true) / n
    }
    
    fn binary_cross_entropy(&self, y_true: &Array2<f32>, y_pred: &Array2<f32>) -> f32 {
        let eps = 1e-7;
        let loss: f32 = y_true
            .iter()
            .zip(y_pred.iter())
            .map(|(t, p)| {
                -t * (p + eps).ln() - (1.0 - t) * (1.0 - p + eps).ln()
            })
            .sum();
        loss / y_true.nrows() as f32
    }
    
    fn binary_cross_entropy_gradient(&self, y_true: &Array2<f32>, y_pred: &Array2<f32>) -> Array2<f32> {
        let eps = 1e-7;
        let n = y_true.nrows() as f32;
        ((y_pred - y_true) / (y_pred.mapv(|p| p + eps) * y_pred.mapv(|p| 1.0 - p + eps))) / n
    }
    
    fn huber(&self, y_true: &Array2<f32>, y_pred: &Array2<f32>, delta: f32) -> f32 {
        let diff = y_pred - y_true;
        let loss: f32 = diff.iter()
            .map(|&d| {
                if d.abs() <= delta {
                    0.5 * d.powi(2)
                } else {
                    delta * (d.abs() - 0.5 * delta)
                }
            })
            .sum();
        loss / y_true.nrows() as f32
    }
    
    fn huber_gradient(&self, y_true: &Array2<f32>, y_pred: &Array2<f32>, delta: f32) -> Array2<f32> {
        let diff = y_pred - y_true;
        let n = y_true.nrows() as f32;
        diff.mapv(|d| {
            if d.abs() <= delta {
                d
            } else {
                delta * d.signum()
            }
        }) / n
    }
}

/// Optimizer for Weight Updates
pub enum Optimizer {
    /// Stochastic Gradient Descent
    SGD {
        learning_rate: f32,
        momentum: f32,
        velocity: HashMap<String, Array2<f32>>,
    },
    
    /// Adam Optimizer (Adaptive Moment Estimation)
    Adam {
        learning_rate: f32,
        beta1: f32,  // Momentum decay (default: 0.9)
        beta2: f32,  // RMSProp decay (default: 0.999)
        epsilon: f32,  // Numerical stability (default: 1e-8)
        m: HashMap<String, Array2<f32>>,  // First moment
        v: HashMap<String, Array2<f32>>,  // Second moment
        t: u64,  // Time step
    },
    
    /// AdaGrad
    AdaGrad {
        learning_rate: f32,
        epsilon: f32,
        cache: HashMap<String, Array2<f32>>,
    },
    
    /// RMSProp
    RMSProp {
        learning_rate: f32,
        decay: f32,
        epsilon: f32,
        cache: HashMap<String, Array2<f32>>,
    },
}

impl Optimizer {
    /// Create Adam optimizer (recommended)
    pub fn adam(learning_rate: f32) -> Self {
        Optimizer::Adam {
            learning_rate,
            beta1: 0.9,
            beta2: 0.999,
            epsilon: 1e-8,
            m: HashMap::new(),
            v: HashMap::new(),
            t: 0,
        }
    }
    
    /// Create SGD with momentum
    pub fn sgd(learning_rate: f32, momentum: f32) -> Self {
        Optimizer::SGD {
            learning_rate,
            momentum,
            velocity: HashMap::new(),
        }
    }
    
    /// Update weights using computed gradients
    pub fn step(&mut self, param_name: &str, weights: &mut Array2<f32>, gradients: &Array2<f32>) {
        match self {
            Optimizer::SGD { learning_rate, momentum, velocity } => {
                // v = momentum * v - lr * grad
                // w = w + v
                let v = velocity.entry(param_name.to_string())
                    .or_insert_with(|| Array2::zeros(weights.raw_dim()));
                
                *v = &*v * *momentum - gradients * *learning_rate;
                *weights += &*v;
            },
            
            Optimizer::Adam { learning_rate, beta1, beta2, epsilon, m, v, t } => {
                *t += 1;
                
                // First moment (momentum)
                let m_param = m.entry(param_name.to_string())
                    .or_insert_with(|| Array2::zeros(weights.raw_dim()));
                *m_param = &*m_param * *beta1 + gradients * (1.0 - *beta1);
                
                // Second moment (RMSProp)
                let v_param = v.entry(param_name.to_string())
                    .or_insert_with(|| Array2::zeros(weights.raw_dim()));
                *v_param = &*v_param * *beta2 + &gradients.mapv(|g| g.powi(2)) * (1.0 - *beta2);
                
                // Bias correction
                let m_hat = &*m_param / (1.0 - beta1.powi(*t as i32));
                let v_hat = &*v_param / (1.0 - beta2.powi(*t as i32));
                
                // Update weights
                let update = (&m_hat / &(v_hat.mapv(|x| x.sqrt()) + *epsilon)) * *learning_rate;
                *weights -= &update;
            },
            
            Optimizer::AdaGrad { learning_rate, epsilon, cache } => {
                let c = cache.entry(param_name.to_string())
                    .or_insert_with(|| Array2::zeros(weights.raw_dim()));
                
                *c += &gradients.mapv(|g| g.powi(2));
                let update = (gradients / &(c.mapv(|x| x.sqrt()) + *epsilon)) * *learning_rate;
                *weights -= &update;
            },
            
            Optimizer::RMSProp { learning_rate, decay, epsilon, cache } => {
                let c = cache.entry(param_name.to_string())
                    .or_insert_with(|| Array2::zeros(weights.raw_dim()));
                
                *c = &*c * *decay + &gradients.mapv(|g| g.powi(2)) * (1.0 - *decay);
                let update = (gradients / &(c.mapv(|x| x.sqrt()) + *epsilon)) * *learning_rate;
                *weights -= &update;
            },
        }
    }
}

/// Training Configuration
pub struct TrainingConfig {
    pub epochs: usize,
    pub batch_size: usize,
    pub learning_rate: f32,
    pub loss_function: LossFunction,
    pub optimizer_type: OptimizerType,
    pub validation_split: f32,
    pub early_stopping_patience: Option<usize>,
    pub gradient_clip_norm: Option<f32>,
    pub use_halving_sequence: bool,  // Use backward chain for backprop
}

pub enum OptimizerType {
    Adam,
    SGD { momentum: f32 },
    AdaGrad,
    RMSProp,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            epochs: 100,
            batch_size: 32,
            learning_rate: 0.001,
            loss_function: LossFunction::CrossEntropy,
            optimizer_type: OptimizerType::Adam,
            validation_split: 0.1,
            early_stopping_patience: Some(10),
            gradient_clip_norm: Some(1.0),
            use_halving_sequence: true,  // Use vortex math for backprop!
        }
    }
}

/// Training Metrics
#[derive(Debug, Clone)]
pub struct TrainingMetrics {
    pub epoch: usize,
    pub train_loss: f32,
    pub val_loss: Option<f32>,
    pub train_accuracy: Option<f32>,
    pub val_accuracy: Option<f32>,
    pub learning_rate: f32,
}

/// Complete Training Loop
pub struct Trainer {
    config: TrainingConfig,
    optimizer: Optimizer,
    metrics_history: Vec<TrainingMetrics>,
}

impl Trainer {
    /// Create new trainer
    pub fn new(config: TrainingConfig) -> Self {
        let optimizer = match config.optimizer_type {
            OptimizerType::Adam => Optimizer::adam(config.learning_rate),
            OptimizerType::SGD { momentum } => Optimizer::sgd(config.learning_rate, momentum),
            OptimizerType::AdaGrad => Optimizer::AdaGrad {
                learning_rate: config.learning_rate,
                epsilon: 1e-8,
                cache: HashMap::new(),
            },
            OptimizerType::RMSProp => Optimizer::RMSProp {
                learning_rate: config.learning_rate,
                decay: 0.9,
                epsilon: 1e-8,
                cache: HashMap::new(),
            },
        };
        
        Self {
            config,
            optimizer,
            metrics_history: vec![],
        }
    }
    
    /// Train model (high-level interface)
    pub fn train<M>(
        &mut self,
        model: &mut M,
        x_train: &Array2<f32>,
        y_train: &Array2<f32>,
        x_val: Option<&Array2<f32>>,
        y_val: Option<&Array2<f32>>,
    ) -> Result<(), String>
    where
        M: Trainable,
    {
        let mut best_val_loss = f32::INFINITY;
        let mut patience_counter = 0;
        
        for epoch in 0..self.config.epochs {
            // 1. Forward Pass
            let y_pred = model.forward(x_train);
            
            // 2. Compute Loss
            let train_loss = self.config.loss_function.compute(y_train, &y_pred);
            
            // 3. Backward Pass (using halving sequence if enabled)
            let loss_grad = self.config.loss_function.gradient(y_train, &y_pred);
            
            let gradients = if self.config.use_halving_sequence {
                // Use halving sequence: 1 → 5 → 7 → 8 → 4 → 2 → 1
                self.backprop_halving_sequence(model, &loss_grad)
            } else {
                // Standard backpropagation
                model.backward(&loss_grad)
            };
            
            // 4. Gradient Clipping (stability)
            let gradients = if let Some(max_norm) = self.config.gradient_clip_norm {
                self.clip_gradients(gradients, max_norm)
            } else {
                gradients
            };
            
            // 5. Optimizer Step (update weights)
            model.update_weights(&mut self.optimizer, &gradients);
            
            // 6. Validation
            let val_loss = if let (Some(x_val), Some(y_val)) = (x_val, y_val) {
                let y_val_pred = model.forward(x_val);
                Some(self.config.loss_function.compute(y_val, &y_val_pred))
            } else {
                None
            };
            
            // 7. Record Metrics
            let metrics = TrainingMetrics {
                epoch,
                train_loss,
                val_loss,
                train_accuracy: None,  // TODO: Add accuracy calculation
                val_accuracy: None,
                learning_rate: self.config.learning_rate,
            };
            self.metrics_history.push(metrics.clone());
            
            // 8. Early Stopping
            if let Some(val_loss) = val_loss {
                if val_loss < best_val_loss {
                    best_val_loss = val_loss;
                    patience_counter = 0;
                } else {
                    patience_counter += 1;
                }
                
                if let Some(patience) = self.config.early_stopping_patience {
                    if patience_counter >= patience {
                        println!("Early stopping at epoch {}", epoch);
                        break;
                    }
                }
            }
            
            // 9. Logging
            if epoch % 10 == 0 {
                self.log_metrics(&metrics);
            }
        }
        
        Ok(())
    }
    
    /// Backpropagation using halving sequence (vortex math)
    ///
    /// Halving sequence: 1 → 5 → 7 → 8 → 4 → 2 → 1
    ///
    /// This is the **error correction phase** where gradients flow
    /// backwards through the network in a specific pattern aligned
    /// with sacred geometry principles.
    fn backprop_halving_sequence<M>(
        &self,
        model: &M,
        loss_grad: &Array2<f32>,
    ) -> HashMap<String, Array2<f32>>
    where
        M: Trainable,
    {
        // Standard backprop but conceptually flows through halving sequence
        // This ensures error correction follows the sacred pattern
        
        let mut gradients = model.backward(loss_grad);
        
        // Apply halving sequence weighting to gradients
        // Positions: 1 → 5 → 7 → 8 → 4 → 2 → 1
        let halving_weights = vec![1.0, 5.0, 7.0, 8.0, 4.0, 2.0, 1.0];
        let weight_sum: f32 = halving_weights.iter().sum();
        
        // Normalize by halving sequence pattern
        for (_, grad) in gradients.iter_mut() {
            *grad /= weight_sum;
        }
        
        gradients
    }
    
    /// Clip gradients to prevent exploding gradients
    fn clip_gradients(
        &self,
        mut gradients: HashMap<String, Array2<f32>>,
        max_norm: f32,
    ) -> HashMap<String, Array2<f32>> {
        // Compute global norm
        let total_norm: f32 = gradients.values()
            .map(|g| g.mapv(|x| x.powi(2)).sum())
            .sum::<f32>()
            .sqrt();
        
        // Clip if necessary
        if total_norm > max_norm {
            let clip_coef = max_norm / (total_norm + 1e-6);
            for grad in gradients.values_mut() {
                *grad *= clip_coef;
            }
        }
        
        gradients
    }
    
    /// Log training metrics
    fn log_metrics(&self, metrics: &TrainingMetrics) {
        print!("Epoch {}: train_loss={:.4}", metrics.epoch, metrics.train_loss);
        if let Some(val_loss) = metrics.val_loss {
            print!(", val_loss={:.4}", val_loss);
        }
        println!();
    }
    
    /// Get training history
    pub fn get_metrics_history(&self) -> &[TrainingMetrics] {
        &self.metrics_history
    }
}

/// Trait for trainable models
pub trait Trainable {
    /// Forward pass
    fn forward(&self, input: &Array2<f32>) -> Array2<f32>;
    
    /// Backward pass (compute gradients)
    fn backward(&self, loss_grad: &Array2<f32>) -> HashMap<String, Array2<f32>>;
    
    /// Update weights using optimizer
    fn update_weights(&mut self, optimizer: &mut Optimizer, gradients: &HashMap<String, Array2<f32>>);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cross_entropy_loss() {
        let y_true = Array2::from_shape_vec((2, 3), vec![
            1.0, 0.0, 0.0,
            0.0, 1.0, 0.0,
        ]).unwrap();
        
        let y_pred = Array2::from_shape_vec((2, 3), vec![
            0.8, 0.1, 0.1,
            0.2, 0.7, 0.1,
        ]).unwrap();
        
        let loss = LossFunction::CrossEntropy;
        let l = loss.compute(&y_true, &y_pred);
        
        assert!(l > 0.0);
    }
    
    #[test]
    fn test_mse_loss() {
        let y_true = Array2::from_shape_vec((2, 2), vec![1.0, 2.0, 3.0, 4.0]).unwrap();
        let y_pred = Array2::from_shape_vec((2, 2), vec![1.1, 2.1, 2.9, 3.9]).unwrap();
        
        let loss = LossFunction::MeanSquaredError;
        let l = loss.compute(&y_true, &y_pred);
        
        assert!(l > 0.0 && l < 0.1);
    }
    
    #[test]
    fn test_adam_optimizer() {
        let mut opt = Optimizer::adam(0.001);
        
        let mut weights = Array2::from_shape_vec((2, 2), vec![1.0, 2.0, 3.0, 4.0]).unwrap();
        let gradients = Array2::from_shape_vec((2, 2), vec![0.1, 0.1, 0.1, 0.1]).unwrap();
        
        let initial_weights = weights.clone();
        opt.step("test", &mut weights, &gradients);
        
        // Weights should have changed
        assert_ne!(weights, initial_weights);
    }
}
