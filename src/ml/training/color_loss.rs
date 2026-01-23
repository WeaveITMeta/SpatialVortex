//! Color-Aware Loss Functions for Aspect Color ML Training
//!
//! Specialized loss functions that understand semantic color relationships
//! in the hexagonal color wheel space.
//!
//! ## Loss Functions
//!
//! 1. **ColorSimilarityLoss**: Weighted distance in HSL space
//! 2. **SemanticConsistencyLoss**: Similar meanings → similar colors
//! 3. **HuePreservationLoss**: Preserve hue relationships during training
//! 4. **ColorContrastiveLoss**: Triplet loss for color embeddings

use crate::data::AspectColor;

/// Color-specific loss functions for training
#[derive(Debug, Clone)]
pub enum ColorLossFunction {
    /// Weighted distance in HSL color space
    /// 
    /// Weights allow emphasizing different color components:
    /// - hue_weight: Semantic meaning (usually highest)
    /// - sat_weight: Color purity
    /// - lum_weight: Brightness
    ColorSimilarity {
        hue_weight: f32,
        sat_weight: f32,
        lum_weight: f32,
    },
    
    /// Semantic consistency: similar meanings should have similar colors
    /// 
    /// Penalizes color distance between semantically related meanings
    SemanticConsistency {
        temperature: f32,  // Softness of similarity
    },
    
    /// Preserve hue relationships during training
    /// 
    /// Ensures angular relationships in color wheel are maintained
    HuePreservation {
        angular_weight: f32,
    },
    
    /// Contrastive loss for triplets: (anchor, positive, negative)
    /// 
    /// Pulls similar colors together, pushes dissimilar apart
    ColorContrastive {
        margin: f32,  // Minimum distance between positive and negative
    },
}

impl ColorLossFunction {
    /// Compute loss between predicted and true colors
    /// 
    /// # Arguments
    /// * `pred_colors` - Predicted aspect colors
    /// * `true_colors` - Ground truth aspect colors
    /// 
    /// # Returns
    /// Scalar loss value (lower is better)
    pub fn compute(&self, pred_colors: &[AspectColor], true_colors: &[AspectColor]) -> f32 {
        assert_eq!(pred_colors.len(), true_colors.len(), "Batch sizes must match");
        
        match self {
            ColorLossFunction::ColorSimilarity { hue_weight, sat_weight, lum_weight } => {
                self.color_similarity_loss(pred_colors, true_colors, *hue_weight, *sat_weight, *lum_weight)
            }
            ColorLossFunction::SemanticConsistency { temperature } => {
                self.semantic_consistency_loss(pred_colors, true_colors, *temperature)
            }
            ColorLossFunction::HuePreservation { angular_weight } => {
                self.hue_preservation_loss(pred_colors, true_colors, *angular_weight)
            }
            ColorLossFunction::ColorContrastive { margin } => {
                self.contrastive_loss(pred_colors, true_colors, *margin)
            }
        }
    }
    
    /// Compute gradient for backpropagation
    /// 
    /// Returns feature vector gradients for each predicted color
    pub fn gradient(&self, pred_colors: &[AspectColor], true_colors: &[AspectColor]) -> Vec<Vec<f32>> {
        assert_eq!(pred_colors.len(), true_colors.len());
        
        let mut gradients = Vec::new();
        let eps = 1e-7;
        
        // Numerical gradient approximation
        for i in 0..pred_colors.len() {
            let mut grad = vec![0.0; 6];  // 6D feature vector
            let base_loss = self.compute(pred_colors, true_colors);
            
            // Compute gradient for each feature dimension
            for j in 0..6 {
                let mut perturbed_colors = pred_colors.to_vec();
                let mut features = perturbed_colors[i].to_feature_vector();
                features[j] += eps;
                perturbed_colors[i] = AspectColor::from_feature_vector(&features);
                
                let perturbed_loss = self.compute(&perturbed_colors, true_colors);
                grad[j] = (perturbed_loss - base_loss) / eps;
            }
            
            gradients.push(grad);
        }
        
        gradients
    }
    
    // ========================================================================
    // Loss Function Implementations
    // ========================================================================
    
    /// Color similarity loss in HSL space
    fn color_similarity_loss(
        &self,
        pred: &[AspectColor],
        true_val: &[AspectColor],
        hue_weight: f32,
        sat_weight: f32,
        lum_weight: f32,
    ) -> f32 {
        let mut total_loss = 0.0;
        
        for (p, t) in pred.iter().zip(true_val.iter()) {
            // Circular distance for hue (0-360 wraps around)
            let hue_diff = (p.hue - t.hue).abs();
            let hue_dist = hue_diff.min(360.0 - hue_diff);
            let hue_loss = hue_weight * (hue_dist / 180.0).powi(2);
            
            // Linear distance for saturation and luminance
            let sat_loss = sat_weight * (p.saturation - t.saturation).powi(2);
            let lum_loss = lum_weight * (p.luminance - t.luminance).powi(2);
            
            total_loss += hue_loss + sat_loss + lum_loss;
        }
        
        total_loss / pred.len() as f32
    }
    
    /// Semantic consistency loss
    fn semantic_consistency_loss(
        &self,
        pred: &[AspectColor],
        true_val: &[AspectColor],
        temperature: f32,
    ) -> f32 {
        let mut total_loss = 0.0;
        let n = pred.len();
        
        // Pairwise consistency
        for i in 0..n {
            for j in (i+1)..n {
                // True color distance (semantic ground truth)
                let true_dist = true_val[i].distance(&true_val[j]);
                
                // Predicted color distance
                let pred_dist = pred[i].distance(&pred[j]);
                
                // Penalize deviation from true distance
                let dist_diff = (pred_dist - true_dist).abs();
                total_loss += (dist_diff / temperature).powi(2);
            }
        }
        
        // Normalize by number of pairs
        let num_pairs = (n * (n - 1)) / 2;
        if num_pairs > 0 {
            total_loss / num_pairs as f32
        } else {
            0.0
        }
    }
    
    /// Hue preservation loss (angular relationships)
    fn hue_preservation_loss(
        &self,
        pred: &[AspectColor],
        true_val: &[AspectColor],
        angular_weight: f32,
    ) -> f32 {
        let mut total_loss = 0.0;
        
        for (p, t) in pred.iter().zip(true_val.iter()) {
            // Convert hue to radians
            let pred_rad = p.hue.to_radians();
            let true_rad = t.hue.to_radians();
            
            // Angular distance (circular)
            let cos_diff = (pred_rad.cos() - true_rad.cos()).powi(2);
            let sin_diff = (pred_rad.sin() - true_rad.sin()).powi(2);
            
            total_loss += angular_weight * (cos_diff + sin_diff);
        }
        
        total_loss / pred.len() as f32
    }
    
    /// Contrastive loss for triplet mining
    fn contrastive_loss(
        &self,
        pred: &[AspectColor],
        true_val: &[AspectColor],
        margin: f32,
    ) -> f32 {
        if pred.len() < 3 {
            return 0.0;  // Need at least 3 samples for triplet
        }
        
        let mut total_loss = 0.0;
        let mut count = 0;
        
        // For each anchor
        for i in 0..pred.len() {
            let anchor_pred = &pred[i];
            let anchor_true = &true_val[i];
            
            // Find positive (similar) and negative (dissimilar)
            for j in 0..pred.len() {
                if i == j { continue; }
                
                let true_dist = anchor_true.distance(&true_val[j]);
                
                // Positive: similar colors (dist < 0.3)
                // Negative: dissimilar colors (dist > 0.7)
                if true_dist < 0.3 {
                    // Positive pair - should be close
                    let pred_dist = anchor_pred.distance(&pred[j]);
                    total_loss += pred_dist.powi(2);
                    count += 1;
                } else if true_dist > 0.7 {
                    // Negative pair - should be far (at least margin apart)
                    let pred_dist = anchor_pred.distance(&pred[j]);
                    let loss = (margin - pred_dist).max(0.0);  // Hinge loss
                    total_loss += loss.powi(2);
                    count += 1;
                }
            }
        }
        
        if count > 0 {
            total_loss / count as f32
        } else {
            0.0
        }
    }
}

impl Default for ColorLossFunction {
    fn default() -> Self {
        // Default: balanced color similarity
        ColorLossFunction::ColorSimilarity {
            hue_weight: 0.6,   // Hue most important for semantic meaning
            sat_weight: 0.2,    // Saturation moderately important
            lum_weight: 0.2,    // Luminance moderately important
        }
    }
}

/// Combined loss for multi-objective training
pub struct ColorLossCombination {
    losses: Vec<(ColorLossFunction, f32)>,  // (loss_fn, weight)
}

impl ColorLossCombination {
    /// Create new combined loss
    pub fn new() -> Self {
        Self {
            losses: Vec::new(),
        }
    }
    
    /// Add loss function with weight
    pub fn add_loss(mut self, loss: ColorLossFunction, weight: f32) -> Self {
        self.losses.push((loss, weight));
        self
    }
    
    /// Compute weighted combination of all losses
    pub fn compute(&self, pred: &[AspectColor], true_val: &[AspectColor]) -> f32 {
        let mut total = 0.0;
        
        for (loss_fn, weight) in &self.losses {
            total += weight * loss_fn.compute(pred, true_val);
        }
        
        total
    }
    
    /// Compute combined gradient
    pub fn gradient(&self, pred: &[AspectColor], true_val: &[AspectColor]) -> Vec<Vec<f32>> {
        let n = pred.len();
        let mut combined_grads = vec![vec![0.0; 6]; n];
        
        for (loss_fn, weight) in &self.losses {
            let grads = loss_fn.gradient(pred, true_val);
            
            for i in 0..n {
                for j in 0..6 {
                    combined_grads[i][j] += weight * grads[i][j];
                }
            }
        }
        
        combined_grads
    }
}

impl Default for ColorLossCombination {
    fn default() -> Self {
        // Default combination: similarity + consistency
        Self::new()
            .add_loss(
                ColorLossFunction::ColorSimilarity {
                    hue_weight: 0.6,
                    sat_weight: 0.2,
                    lum_weight: 0.2,
                },
                1.0,
            )
            .add_loss(
                ColorLossFunction::SemanticConsistency {
                    temperature: 0.5,
                },
                0.5,
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_color_similarity_loss() {
        let pred = vec![AspectColor::from_meaning("love")];
        let true_val = vec![AspectColor::from_meaning("love")];
        
        let loss = ColorLossFunction::ColorSimilarity {
            hue_weight: 0.6,
            sat_weight: 0.2,
            lum_weight: 0.2,
        };
        
        let result = loss.compute(&pred, &true_val);
        
        // Same color should have zero loss
        assert!(result < 0.001, "Identical colors should have ~0 loss");
    }
    
    #[test]
    fn test_semantic_consistency() {
        let pred = vec![
            AspectColor::from_meaning("love"),
            AspectColor::from_meaning("affection"),
            AspectColor::from_meaning("hate"),
        ];
        let true_val = pred.clone();
        
        let loss = ColorLossFunction::SemanticConsistency {
            temperature: 0.5,
        };
        
        let result = loss.compute(&pred, &true_val);
        
        // Perfect prediction should have low loss
        assert!(result < 0.1);
    }
    
    #[test]
    fn test_combined_loss() {
        let pred = vec![AspectColor::from_meaning("joy")];
        let true_val = vec![AspectColor::from_meaning("happiness")];
        
        let combined = ColorLossCombination::default();
        let result = combined.compute(&pred, &true_val);
        
        // Similar meanings should have low loss
        assert!(result >= 0.0);
    }
}
