//! Gap-aware loss functions respecting sacred exclusion principle
//!
//! Loss functions that account for the gaps at positions 0, 3, 6, 9
//! in the Vortex Math system.

/// Components of the gap-aware loss
#[derive(Debug, Clone)]
pub struct LossComponents {
    /// Standard cross-entropy or MSE loss
    pub flow_loss: f64,
    /// Sacred alignment penalty (distance from sacred positions)
    pub sacred_loss: f64,
    /// Center regularization (position 0)
    pub center_reg: f64,
    /// Total combined loss
    pub total: f64,
}

/// Gap-aware loss function for Vortex Math training
///
/// Combines multiple loss components:
/// - Flow loss: Standard prediction error
/// - Sacred loss: Encourages alignment with sacred geometry
/// - Center regularization: Prevents over-reliance on position 0
///
/// # Examples
///
/// ```
/// use spatial_vortex::training::GapAwareLoss;
///
/// let loss_fn = GapAwareLoss::new(0.1, 0.05, 0.02);
///
/// let predictions = vec![0.8, 0.1, 0.1];
/// let targets = vec![1.0, 0.0, 0.0];
///
/// let loss = loss_fn.compute(&predictions, &targets);
/// println!("Total loss: {:.4}", loss.total);
/// ```
pub struct GapAwareLoss {
    /// Sacred alignment weight (alpha)
    pub alpha: f64,
    /// Center regularization weight (beta)
    pub beta: f64,
    /// Stochastic exploration weight (gamma)
    pub gamma: f64,
}

impl GapAwareLoss {
    /// Creates a new gap-aware loss function
    ///
    /// # Arguments
    ///
    /// * `alpha` - Sacred alignment weight (typically 0.05-0.2)
    /// * `beta` - Center regularization weight (typically 0.01-0.1)
    /// * `gamma` - Stochastic exploration weight (typically 0.01-0.05)
    pub fn new(alpha: f64, beta: f64, gamma: f64) -> Self {
        Self { alpha, beta, gamma }
    }
    
    /// Computes the complete loss with all components
    ///
    /// # Arguments
    ///
    /// * `predictions` - Model predictions
    /// * `targets` - Ground truth targets
    ///
    /// # Returns
    ///
    /// Loss components including total
    pub fn compute(&self, predictions: &[f64], targets: &[f64]) -> LossComponents {
        let flow_loss = self.cross_entropy(predictions, targets);
        let sacred_loss = self.sacred_alignment_penalty(predictions);
        let center_reg = self.center_regularization(predictions);
        
        let total = flow_loss 
            + self.alpha * sacred_loss 
            + self.beta * center_reg;
        
        LossComponents {
            flow_loss,
            sacred_loss,
            center_reg,
            total,
        }
    }
    
    /// Cross-entropy loss for predictions
    ///
    /// Standard loss measuring prediction error
    fn cross_entropy(&self, predictions: &[f64], targets: &[f64]) -> f64 {
        let mut loss = 0.0;
        for (pred, target) in predictions.iter().zip(targets) {
            let pred_clamped = pred.max(1e-10).min(1.0 - 1e-10);
            loss -= target * pred_clamped.ln() 
                  + (1.0 - target) * (1.0 - pred_clamped).ln();
        }
        loss / predictions.len() as f64
    }
    
    /// Sacred alignment penalty
    ///
    /// Encourages alignment with positions 3, 6, 9 by penalizing
    /// distance from the sacred triangle
    fn sacred_alignment_penalty(&self, predictions: &[f64]) -> f64 {
        // Compute distance from sacred positions
        // This is a simplified version - in practice would use ELP coordinates
        
        // Sacred positions: 3, 6, 9
        let sacred_indices = [3, 6, 9];
        
        // Sum of predictions at non-sacred positions (should be minimized)
        let mut non_sacred_sum = 0.0;
        for (i, &pred) in predictions.iter().enumerate() {
            if !sacred_indices.contains(&(i as i32)) {
                non_sacred_sum += pred;
            }
        }
        
        non_sacred_sum / predictions.len() as f64
    }
    
    /// Center regularization (position 0)
    ///
    /// Prevents model from relying too heavily on position 0 (center)
    /// by adding dropout-like regularization
    fn center_regularization(&self, predictions: &[f64]) -> f64 {
        // If position 0 exists, penalize high confidence there
        if predictions.len() > 0 {
            predictions[0].powi(2)
        } else {
            0.0
        }
    }
    
    /// Mean squared error (alternative to cross-entropy)
    pub fn mse(&self, predictions: &[f64], targets: &[f64]) -> f64 {
        let mut sum = 0.0;
        for (pred, target) in predictions.iter().zip(targets) {
            let diff = pred - target;
            sum += diff * diff;
        }
        sum / predictions.len() as f64
    }
}

impl Default for GapAwareLoss {
    fn default() -> Self {
        Self::new(0.1, 0.05, 0.02)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_loss_creation() {
        let loss_fn = GapAwareLoss::new(0.1, 0.05, 0.02);
        assert_eq!(loss_fn.alpha, 0.1);
        assert_eq!(loss_fn.beta, 0.05);
        assert_eq!(loss_fn.gamma, 0.02);
    }
    
    #[test]
    fn test_cross_entropy() {
        let loss_fn = GapAwareLoss::default();
        
        let predictions = vec![0.9, 0.1];
        let targets = vec![1.0, 0.0];
        
        let ce = loss_fn.cross_entropy(&predictions, &targets);
        assert!(ce > 0.0);
        assert!(ce < 1.0); // Good predictions should have low loss
    }
    
    #[test]
    fn test_mse() {
        let loss_fn = GapAwareLoss::default();
        
        let predictions = vec![0.9, 0.1, 0.5];
        let targets = vec![1.0, 0.0, 0.5];
        
        let mse = loss_fn.mse(&predictions, &targets);
        assert!(mse > 0.0);
        assert!(mse < 0.1); // Close predictions
    }
    
    #[test]
    fn test_complete_loss() {
        let loss_fn = GapAwareLoss::new(0.1, 0.05, 0.02);
        
        let predictions = vec![0.8, 0.1, 0.05, 0.03, 0.02];
        let targets = vec![1.0, 0.0, 0.0, 0.0, 0.0];
        
        let loss = loss_fn.compute(&predictions, &targets);
        
        assert!(loss.flow_loss > 0.0);
        assert!(loss.total > 0.0);
        assert!(loss.total >= loss.flow_loss); // Total includes additional terms
    }
    
    #[test]
    fn test_sacred_alignment() {
        let loss_fn = GapAwareLoss::default();
        
        let predictions = vec![0.0; 10];
        let penalty = loss_fn.sacred_alignment_penalty(&predictions);
        
        assert_eq!(penalty, 0.0); // All zeros, no penalty
    }
    
    #[test]
    fn test_center_regularization() {
        let loss_fn = GapAwareLoss::default();
        
        let high_center = vec![0.9, 0.1];
        let low_center = vec![0.1, 0.9];
        
        let reg_high = loss_fn.center_regularization(&high_center);
        let reg_low = loss_fn.center_regularization(&low_center);
        
        assert!(reg_high > reg_low); // High center should be penalized more
    }
    
    #[test]
    fn test_loss_components() {
        let loss_fn = GapAwareLoss::new(0.1, 0.05, 0.02);
        
        let predictions = vec![0.5, 0.3, 0.2];
        let targets = vec![1.0, 0.0, 0.0];
        
        let components = loss_fn.compute(&predictions, &targets);
        
        // Verify total is sum of weighted components
        let expected_total = components.flow_loss 
            + 0.1 * components.sacred_loss 
            + 0.05 * components.center_reg;
        
        assert!((components.total - expected_total).abs() < 1e-6);
    }
}
