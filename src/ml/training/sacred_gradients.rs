//! Sacred gradient fields - Attraction to positions 3, 6, 9
//!
//! Implements gradient fields that attract learning toward sacred positions
//! which serve as checkpoints and attention mechanisms.

use crate::models::ELPTensor;

/// Sacred gradient field for geometric learning
///
/// Creates attraction fields around positions 3, 6, 9 that influence
/// gradient flow during training.
///
/// # Sacred Positions
///
/// - **Position 3** (Ethos): Character/Authority anchor
/// - **Position 6** (Pathos): Emotion/Expression anchor  
/// - **Position 9** (Logos): Logic/Analytical anchor
///
/// # Examples
///
/// ```
/// use spatial_vortex::training::SacredGradientField;
///
/// let field = SacredGradientField::new(1.0); // strength = 1.0
///
/// let gradient = field.compute_sacred_gradient(5, 8.0, 10.0, 6.0);
/// println!("Sacred gradient: {:?}", gradient);
/// ```
pub struct SacredGradientField {
    /// Field strength (how much sacred positions attract)
    strength: f64,
    /// Sacred positions (3, 6, 9)
    sacred_positions: Vec<u8>,
}

impl SacredGradientField {
    /// Creates a new sacred gradient field
    ///
    /// # Arguments
    ///
    /// * `strength` - Attraction strength (typically 0.1 to 2.0)
    pub fn new(strength: f64) -> Self {
        Self {
            strength,
            sacred_positions: vec![3, 6, 9],
        }
    }
    
    /// Computes sacred gradient for a position
    ///
    /// Gradients are attracted toward nearest sacred position
    /// based on distance in ELP space.
    ///
    /// # Arguments
    ///
    /// * `current_position` - Current position (0-9)
    /// * `ethos` - Ethos coordinate
    /// * `logos` - Logos coordinate  
    /// * `pathos` - Pathos coordinate
    ///
    /// # Returns
    ///
    /// ELP gradient tensor pointing toward sacred positions
    pub fn compute_sacred_gradient(
        &self,
        current_position: u8,
        ethos: f64,
        logos: f64,
        pathos: f64,
    ) -> ELPTensor {
        // Find nearest sacred position
        let (nearest_sacred, distance) = self.find_nearest_sacred(
            current_position,
            ethos,
            logos,
            pathos,
        );
        
        // Compute attraction vector
        let (target_e, target_l, target_p) = self.sacred_coordinates(nearest_sacred);
        
        // Gradient points toward target, scaled by inverse distance
        let scale = if distance > 0.0 {
            self.strength / distance
        } else {
            0.0
        };
        
        ELPTensor::new(
            (target_e - ethos) * scale,
            (target_l - logos) * scale,
            (target_p - pathos) * scale,
        )
    }
    
    /// Finds nearest sacred position
    fn find_nearest_sacred(
        &self,
        _current_position: u8,
        ethos: f64,
        logos: f64,
        pathos: f64,
    ) -> (u8, f64) {
        let mut nearest = 3;
        let mut min_distance = f64::MAX;
        
        for &sacred in &self.sacred_positions {
            let (s_e, s_l, s_p) = self.sacred_coordinates(sacred);
            let distance = self.elp_distance(
                ethos, logos, pathos,
                s_e, s_l, s_p,
            );
            
            if distance < min_distance {
                min_distance = distance;
                nearest = sacred;
            }
        }
        
        (nearest, min_distance)
    }
    
    /// Returns ELP coordinates for sacred positions
    ///
    /// Sacred Triangle in ELP space:
    /// - Position 3 (Ethos): (13, 0, 0)
    /// - Position 6 (Pathos): (0, 0, 13)
    /// - Position 9 (Logos): (0, 13, 0)
    fn sacred_coordinates(&self, position: u8) -> (f64, f64, f64) {
        match position {
            3 => (13.0, 0.0, 0.0),   // Pure Ethos
            6 => (0.0, 0.0, 13.0),   // Pure Pathos
            9 => (0.0, 13.0, 0.0),   // Pure Logos
            _ => (0.0, 0.0, 0.0),
        }
    }
    
    /// Computes distance in ELP space
    fn elp_distance(
        &self,
        e1: f64, l1: f64, p1: f64,
        e2: f64, l2: f64, p2: f64,
    ) -> f64 {
        let de = e2 - e1;
        let dl = l2 - l1;
        let dp = p2 - p1;
        (de * de + dl * dl + dp * dp).sqrt()
    }
    
    /// Applies sacred gradient to existing gradient
    pub fn apply_sacred_gradient(
        &self,
        gradient: &mut ELPTensor,
        current_position: u8,
        elp: &ELPTensor,
    ) {
        let sacred_grad = self.compute_sacred_gradient(
            current_position,
            elp.ethos,
            elp.logos,
            elp.pathos,
        );
        
        // Add sacred gradient to existing gradient
        gradient.ethos += sacred_grad.ethos;
        gradient.logos += sacred_grad.logos;
        gradient.pathos += sacred_grad.pathos;
    }
    
    /// Returns field strength
    pub fn strength(&self) -> f64 {
        self.strength
    }
    
    /// Sets field strength
    pub fn set_strength(&mut self, strength: f64) {
        self.strength = strength;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sacred_field_creation() {
        let field = SacredGradientField::new(1.0);
        assert_eq!(field.strength(), 1.0);
        assert_eq!(field.sacred_positions.len(), 3);
    }
    
    #[test]
    fn test_sacred_coordinates() {
        let field = SacredGradientField::new(1.0);
        
        let (e, l, p) = field.sacred_coordinates(3);
        assert_eq!((e, l, p), (13.0, 0.0, 0.0)); // Pure Ethos
        
        let (e, l, p) = field.sacred_coordinates(6);
        assert_eq!((e, l, p), (0.0, 0.0, 13.0)); // Pure Pathos
        
        let (e, l, p) = field.sacred_coordinates(9);
        assert_eq!((e, l, p), (0.0, 13.0, 0.0)); // Pure Logos
    }
    
    #[test]
    fn test_gradient_toward_sacred() {
        let field = SacredGradientField::new(1.0);
        
        // Point far from sacred positions
        let gradient = field.compute_sacred_gradient(1, 0.0, 0.0, 0.0);
        
        // Should have non-zero gradient pulling toward nearest sacred
        assert!(gradient.ethos.abs() > 0.0 || 
                gradient.logos.abs() > 0.0 || 
                gradient.pathos.abs() > 0.0);
    }
    
    #[test]
    fn test_nearest_sacred_position() {
        let field = SacredGradientField::new(1.0);
        
        // Point close to Ethos (position 3)
        let (nearest, _) = field.find_nearest_sacred(1, 10.0, 0.0, 0.0);
        assert_eq!(nearest, 3);
        
        // Point close to Logos (position 9)
        let (nearest, _) = field.find_nearest_sacred(1, 0.0, 10.0, 0.0);
        assert_eq!(nearest, 9);
        
        // Point close to Pathos (position 6)
        let (nearest, _) = field.find_nearest_sacred(1, 0.0, 0.0, 10.0);
        assert_eq!(nearest, 6);
    }
    
    #[test]
    fn test_distance_calculation() {
        let field = SacredGradientField::new(1.0);
        
        // Distance from origin to (3, 4, 0) should be 5
        let distance = field.elp_distance(0.0, 0.0, 0.0, 3.0, 4.0, 0.0);
        assert!((distance - 5.0).abs() < 0.001);
    }
    
    #[test]
    fn test_apply_sacred_gradient() {
        let field = SacredGradientField::new(1.0);
        
        let mut gradient = ELPTensor::new(1.0, 1.0, 1.0);
        let elp = ELPTensor::new(0.0, 0.0, 0.0);
        
        field.apply_sacred_gradient(&mut gradient, 1, &elp);
        
        // Gradient should have changed (sacred field applied)
        assert!(gradient.ethos != 1.0 || gradient.logos != 1.0 || gradient.pathos != 1.0);
    }
}
