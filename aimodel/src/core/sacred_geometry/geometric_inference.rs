//! Geometric Reasoning Inference Engine
//! 
//! Rule-based inference for geometric-to-flux position mapping
//! Handles 5 task types with specialized logic for each

use crate::models::ELPTensor;
use std::f64::consts::PI;

/// Task types for geometric reasoning
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GeometricTaskType {
    Transformation,
    SpatialRelations,
    PositionMapping,
    PatternCompletion,
    SacredRecognition,
}

impl GeometricTaskType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "Transformation" => GeometricTaskType::Transformation,
            "SpatialRelations" => GeometricTaskType::SpatialRelations,
            "PositionMapping" => GeometricTaskType::PositionMapping,
            "PatternCompletion" => GeometricTaskType::PatternCompletion,
            "SacredRecognition" => GeometricTaskType::SacredRecognition,
            _ => GeometricTaskType::PositionMapping, // Default
        }
    }
}

/// Geometric input for inference
#[derive(Debug, Clone)]
pub struct GeometricInput {
    pub angle: f64,          // 0-360 degrees
    pub distance: f64,       // 0-10 units
    pub complexity: f64,     // 0-1 scale
    pub task_type: GeometricTaskType,
}

/// Rule-based geometric inference engine
#[derive(Debug)]
pub struct GeometricInferenceEngine {
    /// Sacred positions in vortex mathematics [3, 6, 9]
    sacred_positions: [u8; 3],
    
    /// Debugging enabled
    pub debug: bool,
}

impl Default for GeometricInferenceEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl GeometricInferenceEngine {
    pub fn new() -> Self {
        Self {
            sacred_positions: [3, 6, 9],
            debug: false,
        }
    }
    
    /// Enable debug output
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }
    
    /// Main inference function - returns position 0-9
    pub fn infer_position(&self, input: &GeometricInput) -> u8 {
        if self.debug {
            println!("  🧠 Geometric Inference");
            println!("     Type: {:?}", input.task_type);
            println!("     Angle: {:.1}°, Distance: {:.2}, Complexity: {:.2}", 
                input.angle, input.distance, input.complexity);
        }
        
        let position = match input.task_type {
            GeometricTaskType::SacredRecognition => self.infer_sacred(input),
            GeometricTaskType::PositionMapping => self.infer_position_mapping(input),
            GeometricTaskType::Transformation => self.infer_transformation(input),
            GeometricTaskType::SpatialRelations => self.infer_spatial_relations(input),
            GeometricTaskType::PatternCompletion => self.infer_pattern_completion(input),
        };
        
        if self.debug {
            println!("     → Predicted: {}", position);
        }
        
        position
    }
    
    /// Sacred recognition: MUST return 3, 6, or 9
    /// Sacred positions at 120° intervals
    fn infer_sacred(&self, input: &GeometricInput) -> u8 {
        let normalized = input.angle.rem_euclid(360.0);
        
        // Divide circle into 3 equal parts (120° each)
        // 0-120° → 3 (Ethos)
        // 120-240° → 6 (Pathos)  
        // 240-360° → 9 (Logos)
        if normalized < 120.0 {
            3
        } else if normalized < 240.0 {
            6
        } else {
            9
        }
    }
    
    /// Position mapping: Direct angle to flux position
    /// 360° / 10 positions = 36° per position
    fn infer_position_mapping(&self, input: &GeometricInput) -> u8 {
        let normalized = input.angle.rem_euclid(360.0);
        
        // Map to 0-9: each position covers 36°
        let position = (normalized / 36.0).round() as u8;
        position % 10
    }
    
    /// Transformation: Angle-based with distance modifier
    fn infer_transformation(&self, input: &GeometricInput) -> u8 {
        let normalized = input.angle.rem_euclid(360.0);
        
        // Base position from angle
        let base = (normalized / 36.0) as i16;
        
        // Distance affects transformation magnitude (scale: 0-10 → 0-3 positions)
        let dist_modifier = (input.distance * 0.3) as i16;
        
        // Complexity affects direction
        let complexity_dir = if input.complexity > 0.5 { 1 } else { -1 };
        
        let position = (base + dist_modifier * complexity_dir).rem_euclid(10) as u8;
        position
    }
    
    /// Spatial relations: Distance-primary with angle adjustment
    fn infer_spatial_relations(&self, input: &GeometricInput) -> u8 {
        // Distance maps to base position (0-10 → 0-9)
        let base = (input.distance * 0.9).round() as i16;
        
        // Angle provides directional adjustment
        // Divide angle into quadrants
        let angle_adjustment = ((input.angle / 90.0) as i16) % 4 - 2; // -2 to +1
        
        let position = (base + angle_adjustment).rem_euclid(10) as u8;
        position
    }
    
    /// Pattern completion: Complexity-based
    fn infer_pattern_completion(&self, input: &GeometricInput) -> u8 {
        // Complexity directly maps to position
        // 0.0 → 0, 1.0 → 9
        let position = (input.complexity * 9.0).round() as u8;
        position.min(9)
    }
    
    /// Get confidence score for prediction
    pub fn confidence(&self, input: &GeometricInput, predicted: u8) -> f64 {
        let base_confidence = match input.task_type {
            GeometricTaskType::SacredRecognition => {
                // High confidence if predicted is sacred
                if self.sacred_positions.contains(&predicted) {
                    0.9
                } else {
                    0.3 // Should never happen with correct implementation
                }
            },
            GeometricTaskType::PositionMapping => 0.75,
            GeometricTaskType::Transformation => 0.60,
            GeometricTaskType::SpatialRelations => 0.65,
            GeometricTaskType::PatternCompletion => 0.55,
        };
        
        // Boost confidence based on complexity (simpler = more confident)
        let complexity_factor = 1.0 - (input.complexity * 0.2);
        
        // Sacred position bonus
        let sacred_bonus = if self.sacred_positions.contains(&predicted) {
            0.15 // +15% as per TERMINOLOGY.md
        } else {
            0.0
        };
        
        (base_confidence * complexity_factor + sacred_bonus).min(1.0)
    }
    
    /// Check if position is sacred (3, 6, or 9)
    pub fn is_sacred(&self, position: u8) -> bool {
        self.sacred_positions.contains(&position)
    }
}

/// Convert angle to ELP tensor for semantic embedding
pub fn angle_to_elp(angle: f64) -> ELPTensor {
    let normalized = angle.rem_euclid(360.0);
    let radians = normalized * PI / 180.0;
    
    // Map angle to ELP space using sacred triangle
    // 0° (East): High Logos (logic/rationality)
    // 120° (NW): High Ethos (character/ethics)
    // 240° (SW): High Pathos (emotion/passion)
    
    // Each component peaks at its designated angle and decreases with distance
    // Using negative angles so peaks are at the right positions
    let ethos = (0.5 + 0.5 * (radians - 2.0 * PI / 3.0).cos()).clamp(0.0, 1.0);
    let logos = (0.5 + 0.5 * radians.cos()).clamp(0.0, 1.0);
    let pathos = (0.5 + 0.5 * (radians - 4.0 * PI / 3.0).cos()).clamp(0.0, 1.0);
    
    ELPTensor::new(ethos, logos, pathos)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sacred_recognition() {
        let engine = GeometricInferenceEngine::new();
        
        // Test sacred positions at 120° intervals
        let input1 = GeometricInput {
            angle: 60.0,  // First third (0-120°)
            distance: 5.0,
            complexity: 0.5,
            task_type: GeometricTaskType::SacredRecognition,
        };
        assert_eq!(engine.infer_position(&input1), 3);
        
        let input2 = GeometricInput {
            angle: 180.0,  // Second third (120-240°)
            distance: 5.0,
            complexity: 0.5,
            task_type: GeometricTaskType::SacredRecognition,
        };
        assert_eq!(engine.infer_position(&input2), 6);
        
        let input3 = GeometricInput {
            angle: 300.0,  // Third third (240-360°)
            distance: 5.0,
            complexity: 0.5,
            task_type: GeometricTaskType::SacredRecognition,
        };
        assert_eq!(engine.infer_position(&input3), 9);
    }
    
    #[test]
    fn test_position_mapping() {
        let engine = GeometricInferenceEngine::new();
        
        // 0° should map to position 0
        let input = GeometricInput {
            angle: 0.0,
            distance: 5.0,
            complexity: 0.5,
            task_type: GeometricTaskType::PositionMapping,
        };
        assert_eq!(engine.infer_position(&input), 0);
        
        // 180° should map to position 5
        let input = GeometricInput {
            angle: 180.0,
            distance: 5.0,
            complexity: 0.5,
            task_type: GeometricTaskType::PositionMapping,
        };
        assert_eq!(engine.infer_position(&input), 5);
        
        // 36° should map to position 1
        let input = GeometricInput {
            angle: 36.0,
            distance: 5.0,
            complexity: 0.5,
            task_type: GeometricTaskType::PositionMapping,
        };
        assert_eq!(engine.infer_position(&input), 1);
    }
    
    #[test]
    fn test_confidence_scoring() {
        let engine = GeometricInferenceEngine::new();
        
        let input = GeometricInput {
            angle: 60.0,
            distance: 5.0,
            complexity: 0.5,
            task_type: GeometricTaskType::SacredRecognition,
        };
        
        // Sacred position should have high confidence
        let conf_sacred = engine.confidence(&input, 3);
        assert!(conf_sacred > 0.8, "Sacred confidence should be >0.8, got {}", conf_sacred);
        
        // Non-sacred position should have lower confidence
        let conf_nonsacred = engine.confidence(&input, 5);
        assert!(conf_nonsacred < 0.5, "Non-sacred confidence should be <0.5, got {}", conf_nonsacred);
    }
    
    #[test]
    fn test_is_sacred() {
        let engine = GeometricInferenceEngine::new();
        
        assert!(engine.is_sacred(3));
        assert!(engine.is_sacred(6));
        assert!(engine.is_sacred(9));
        
        assert!(!engine.is_sacred(0));
        assert!(!engine.is_sacred(1));
        assert!(!engine.is_sacred(5));
    }
    
    #[test]
    fn test_angle_to_elp() {
        // Test cardinal directions
        let elp_0 = angle_to_elp(0.0);
        assert!(elp_0.logos > 0.8, "0° should have high logos");
        
        let elp_120 = angle_to_elp(120.0);
        assert!(elp_120.ethos > 0.8, "120° should have high ethos");
        
        let elp_240 = angle_to_elp(240.0);
        assert!(elp_240.pathos > 0.8, "240° should have high pathos");
    }
}
