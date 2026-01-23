//! Dynamic ELP Attributes System
//!
//! Evolves static ELP (Ethos, Logos, Pathos) into dynamic, context-aware attributes
//! derived from sacred 3-6-9 positions with comparative signal dynamics for knowledge retention.

use nalgebra::Vector3;
use serde::{Serialize, Deserialize};
use crate::models::ELPTensor;
use anyhow::Result;

/// Categorical states for attribute stability
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AttributeState {
    /// Low variance, high retention - stable baseline
    Stable,
    /// High variance, requires comparative analysis
    Fluctuating,
    /// Sacred-derived novelty, emerging patterns
    Emergent,
    /// Critical imbalance requiring intervention
    Critical,
}

/// Color representation for perceptual/emotional encoding
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AttributeColor {
    pub r: f32,
    pub g: f32, 
    pub b: f32,
    pub a: f32,  // Alpha for intensity
}

impl AttributeColor {
    /// Create from value (0.0-1.0) mapping to color spectrum
    pub fn from_value(value: f32) -> Self {
        // Map value to HSV color space for intuitive visualization
        let hue = value.clamp(0.0, 1.0) * 360.0;
        let (r, g, b) = Self::hsv_to_rgb(hue, 1.0, value);
        Self { r, g, b, a: 1.0 }
    }
    
    /// Create from ELP dominance
    pub fn from_elp_dominance(ethos: f32, logos: f32, pathos: f32) -> Self {
        // Ethos = Blue/Purple (trust, stability)
        // Logos = Green (logic, growth)  
        // Pathos = Red/Orange (emotion, energy)
        let total = ethos + logos + pathos;
        if total == 0.0 {
            return Self { r: 0.5, g: 0.5, b: 0.5, a: 0.5 };
        }
        
        let e_norm = ethos / total;
        let l_norm = logos / total;
        let p_norm = pathos / total;
        
        Self {
            r: p_norm * 0.9 + e_norm * 0.3,  // Red dominated by pathos
            g: l_norm * 0.9 + e_norm * 0.2,  // Green dominated by logos
            b: e_norm * 0.9 + l_norm * 0.3,  // Blue dominated by ethos
            a: total.min(1.0),  // Intensity from total signal
        }
    }
    
    /// Convert HSV to RGB color space
    fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
        let c = v * s;
        let h_prime = h / 60.0;
        let x = c * (1.0 - ((h_prime % 2.0) - 1.0).abs());
        let m = v - c;
        
        let (r, g, b) = if h_prime < 1.0 {
            (c, x, 0.0)
        } else if h_prime < 2.0 {
            (x, c, 0.0)
        } else if h_prime < 3.0 {
            (0.0, c, x)
        } else if h_prime < 4.0 {
            (0.0, x, c)
        } else if h_prime < 5.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };
        
        (r + m, g + m, b + m)
    }
}

/// Subject generated from flux positions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FluxSubject {
    pub id: String,
    pub name: String,
    pub position: u8,
    pub seed_value: u8,  // 3, 6, or 9
    pub context: String,
    pub variance: f32,
    pub ethos_weight: f32,
    pub logos_weight: f32,
    pub pathos_weight: f32,
    /// Aspect orientation (semantic meaning with intentional color)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect: Option<crate::data::aspect_color::AspectOrientation>,
}

impl FluxSubject {
    /// Create from sacred position with semantic context
    pub fn from_sacred_position(position: u8, context: &str) -> Self {
        assert!([3, 6, 9].contains(&position), "Position must be sacred (3, 6, 9)");
        
        // Derive weights based on sacred position characteristics
        let (ethos, logos, pathos, name) = match position {
            3 => (0.9, 0.6, 0.5, "Creative Ethics"),      // Position 3: Good/Easy - Ethos dominant
            6 => (0.5, 0.9, 0.4, "Analytical Logic"),     // Position 6: Bad/Hard - Logos dominant  
            9 => (0.6, 0.5, 0.9, "Divine Emotion"),       // Position 9: Divine - Pathos dominant
            _ => unreachable!(),
        };
        
        // Create aspect from semantic meaning (context + name)
        // Color derives from MEANING, not position
        let semantic_meaning = format!("{} {}", name, context);
        let aspect = crate::data::aspect_color::AspectOrientation::from_meaning(
            &semantic_meaning, 
            0.15  // Default variance
        );
        
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            position,
            seed_value: position,
            context: context.to_string(),
            variance: 0.1,  // Start with low variance
            ethos_weight: ethos,
            logos_weight: logos,
            pathos_weight: pathos,
            aspect: Some(aspect),
        }
    }
    
    /// Calculate ethos delta from baseline
    pub fn ethos_delta(&self) -> Vector3<f32> {
        Vector3::new(
            self.ethos_weight * self.variance,
            self.ethos_weight * 0.5,
            self.ethos_weight * 0.3,
        )
    }
    
    /// Calculate logos delta from baseline
    pub fn logos_delta(&self) -> Vector3<f32> {
        Vector3::new(
            self.logos_weight * 0.3,
            self.logos_weight * self.variance,
            self.logos_weight * 0.5,
        )
    }
    
    /// Calculate pathos delta from baseline
    pub fn pathos_delta(&self) -> Vector3<f32> {
        Vector3::new(
            self.pathos_weight * 0.5,
            self.pathos_weight * 0.3,
            self.pathos_weight * self.variance,
        )
    }
    
    /// Get current variance level
    pub fn variance(&self) -> f32 {
        self.variance
    }
    
    /// Update variance based on input
    pub fn update_variance(&mut self, signal: f32) {
        self.variance = (self.variance * 0.9 + signal * 0.1).clamp(0.0, 1.0);
    }
    
    /// Get aspect orientation (semantic meaning with color)
    pub fn aspect_orientation(&self) -> Option<&crate::data::aspect_color::AspectOrientation> {
        self.aspect.as_ref()
    }
    
    /// Get aspect color (from semantic meaning)
    pub fn aspect_color(&self) -> Option<crate::data::aspect_color::AspectColor> {
        self.aspect.as_ref().map(|a| a.color)
    }
    
    /// Set aspect from semantic meaning
    pub fn set_aspect_from_meaning(&mut self, meaning: &str, variance: f32) {
        self.aspect = Some(crate::data::aspect_color::AspectOrientation::from_meaning(meaning, variance));
    }
}

/// Dynamic ELP Attributes with comparative signal dynamics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicELP {
    /// 3D vectors for multi-axis depth
    pub ethos: Vector3<f32>,
    pub logos: Vector3<f32>,
    pub pathos: Vector3<f32>,
    
    /// Static baseline from sacred position (3, 6, or 9)
    pub dominant_value: u8,
    
    /// Current state of the attribute
    pub state: AttributeState,
    
    /// Color representation for visualization
    pub color: AttributeColor,
    
    /// Importance score for retention
    pub importance: f32,
    
    /// Historical trajectory for learning
    pub trajectory: Vec<(f32, f32, f32)>,  // (ethos_norm, logos_norm, pathos_norm)
    
    /// Timestamp for temporal dynamics
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl DynamicELP {
    /// Create from flux subject and sacred seed
    pub fn from_subject(subject: &FluxSubject, sacred_seed: u8) -> Self {
        assert!([3, 6, 9].contains(&sacred_seed), "Seed must be sacred (3, 6, 9)");
        
        // Establish static baseline from sacred position
        let base_value = sacred_seed as f32 / 9.0;
        let base_vec = Vector3::new(base_value, base_value * 0.5, base_value * 0.3);
        
        // Apply dynamic deltas from subject
        let ethos = base_vec + subject.ethos_delta();
        let logos = base_vec + subject.logos_delta();
        let pathos = base_vec + subject.pathos_delta();
        
        // Determine state based on variance
        let state = match subject.variance() {
            v if v < 0.2 => AttributeState::Stable,
            v if v < 0.5 => AttributeState::Fluctuating,
            v if v < 0.8 => AttributeState::Emergent,
            _ => AttributeState::Critical,
        };
        
        // Calculate initial importance
        let importance = Self::calculate_importance(&ethos, &logos, &pathos, base_vec);
        
        // Create color from dominance
        let color = AttributeColor::from_elp_dominance(
            ethos.norm(),
            logos.norm(),
            pathos.norm(),
        );
        
        Self {
            ethos,
            logos,
            pathos,
            dominant_value: sacred_seed,
            state,
            color,
            importance,
            trajectory: vec![(ethos.norm(), logos.norm(), pathos.norm())],
            timestamp: chrono::Utc::now(),
        }
    }
    
    /// Create from static ELP tensor (backward compatibility)
    pub fn from_static_elp(elp: &ELPTensor, position: u8) -> Self {
        let ethos = Vector3::new(elp.ethos as f32 / 9.0, 0.4, 0.3);
        let logos = Vector3::new(0.3, elp.logos as f32 / 9.0, 0.4);
        let pathos = Vector3::new(0.4, 0.3, elp.pathos as f32 / 9.0);
        
        let dominant = if [3, 6, 9].contains(&position) { position } else { 5 };
        
        Self {
            ethos,
            logos,
            pathos,
            dominant_value: dominant,
            state: AttributeState::Stable,
            color: AttributeColor::from_elp_dominance(ethos.norm(), logos.norm(), pathos.norm()),
            importance: 0.5,
            trajectory: vec![(ethos.norm(), logos.norm(), pathos.norm())],
            timestamp: chrono::Utc::now(),
        }
    }
    
    /// Adjust attributes based on input signal (comparative dynamics)
    pub fn adjust(&mut self, input_signal: f32, harmony_factor: f32) {
        let baseline = self.dominant_value as f32 / 9.0;
        let delta = (input_signal - baseline) * harmony_factor;
        
        // Apply adjustments with channel-specific modulation
        match self.dominant_value {
            3 => {
                // Ethos-dominant: Strengthen ethical alignment
                self.ethos += Vector3::new(delta * 1.2, delta * 0.3, 0.0);
                self.logos += Vector3::new(0.0, delta * 0.5, 0.0);
                self.pathos += Vector3::new(0.0, 0.0, delta * 0.3);
            }
            6 => {
                // Logos-dominant: Enhance logical reasoning
                self.ethos += Vector3::new(delta * 0.3, 0.0, 0.0);
                self.logos += Vector3::new(0.0, delta * 1.2, delta * 0.3);
                self.pathos += Vector3::new(0.0, 0.0, delta * 0.4);
            }
            9 => {
                // Pathos-dominant: Amplify emotional resonance (with safeguards)
                self.ethos += Vector3::new(delta * 0.5, 0.0, 0.0);
                self.logos += Vector3::new(0.0, delta * 0.4, 0.0);
                self.pathos += Vector3::new(0.0, 0.0, delta * 0.8);  // Reduced to prevent dominance
            }
            _ => {
                // Non-sacred: Balanced adjustment
                let balanced_delta = Vector3::new(delta * 0.5, delta * 0.5, delta * 0.5);
                self.ethos += balanced_delta;
                self.logos += balanced_delta;
                self.pathos += balanced_delta;
            }
        }
        
        // Apply harmony constraints to prevent channel dominance
        self.enforce_harmony();
        
        // Update trajectory for learning
        self.trajectory.push((self.ethos.norm(), self.logos.norm(), self.pathos.norm()));
        if self.trajectory.len() > 100 {
            self.trajectory.remove(0);  // Keep last 100 states
        }
        
        // Recalculate importance
        let base_vec = Vector3::new(baseline, baseline * 0.5, baseline * 0.3);
        self.importance = Self::calculate_importance(&self.ethos, &self.logos, &self.pathos, base_vec);
        
        // Update state based on new dynamics
        self.update_state();
        
        // Update color
        self.color = AttributeColor::from_elp_dominance(
            self.ethos.norm(),
            self.logos.norm(),
            self.pathos.norm(),
        );
    }
    
    /// Enforce harmony to prevent any channel from dominating
    fn enforce_harmony(&mut self) {
        let ethos_norm = self.ethos.norm();
        let logos_norm = self.logos.norm();
        let pathos_norm = self.pathos.norm();
        let total = ethos_norm + logos_norm + pathos_norm;
        
        if total == 0.0 {
            return;
        }
        
        // Check for dangerous imbalances
        let ethos_ratio = ethos_norm / total;
        let logos_ratio = logos_norm / total;
        let pathos_ratio = pathos_norm / total;
        
        // If any channel exceeds 70% dominance, rebalance
        if pathos_ratio > 0.7 {
            // Pathos dominance detected - apply corrective scaling
            self.pathos *= 0.7;
            self.ethos *= 1.15;  // Boost ethos for ethical grounding
            self.logos *= 1.15;  // Boost logos for logical balance
            self.state = AttributeState::Critical;
        } else if ethos_ratio < 0.2 || logos_ratio < 0.2 {
            // Ethos or Logos critically low - intervention needed
            if ethos_ratio < 0.2 {
                self.ethos *= 1.5;  // Emergency ethos boost
            }
            if logos_ratio < 0.2 {
                self.logos *= 1.5;  // Emergency logos boost
            }
            self.pathos *= 0.8;  // Mild pathos reduction
            self.state = AttributeState::Critical;
        }
        
        // Apply soft clamping to keep values reasonable
        self.ethos = self.clamp_vector(self.ethos, 0.1, 2.0);
        self.logos = self.clamp_vector(self.logos, 0.1, 2.0);
        self.pathos = self.clamp_vector(self.pathos, 0.1, 2.0);
    }
    
    /// Clamp vector components
    fn clamp_vector(&self, v: Vector3<f32>, min: f32, max: f32) -> Vector3<f32> {
        Vector3::new(
            v.x.clamp(min, max),
            v.y.clamp(min, max),
            v.z.clamp(min, max),
        )
    }
    
    /// Calculate importance for retention decisions
    fn calculate_importance(ethos: &Vector3<f32>, logos: &Vector3<f32>, pathos: &Vector3<f32>, baseline: Vector3<f32>) -> f32 {
        // Distance from baseline indicates novelty/importance
        let ethos_deviation = (ethos - baseline).norm();
        let logos_deviation = (logos - baseline).norm();
        let pathos_deviation = (pathos - baseline).norm();
        
        // Weight deviations by sacred geometry (3-6-9 pattern)
        let weighted_importance = 
            ethos_deviation * 0.33 +  // Position 3 weight
            logos_deviation * 0.36 +   // Position 6 weight  
            pathos_deviation * 0.31;   // Position 9 weight (reduced to prevent dominance)
        
        weighted_importance.clamp(0.0, 1.0)
    }
    
    /// Update state based on current dynamics
    fn update_state(&mut self) {
        // Calculate variance from trajectory
        if self.trajectory.len() < 3 {
            self.state = AttributeState::Stable;
            return;
        }
        
        let recent = &self.trajectory[self.trajectory.len().saturating_sub(10)..];
        let mut variance = 0.0;
        
        for window in recent.windows(2) {
            let delta = (window[1].0 - window[0].0).abs() +
                       (window[1].1 - window[0].1).abs() +
                       (window[1].2 - window[0].2).abs();
            variance += delta;
        }
        
        variance /= recent.len() as f32;
        
        self.state = match variance {
            v if v < 0.1 => AttributeState::Stable,
            v if v < 0.3 => AttributeState::Fluctuating,
            v if v < 0.6 => AttributeState::Emergent,
            _ => AttributeState::Critical,
        };
    }
    
    /// Get current importance score
    pub fn importance(&self) -> f32 {
        self.importance
    }
    
    /// Check if knowledge should be retained in Confidence Lake
    pub fn should_retain(&self, threshold: f32) -> bool {
        self.importance >= threshold || self.state == AttributeState::Emergent || self.state == AttributeState::Critical
    }
    
    /// Get harmony score (balance between channels)
    pub fn harmony_score(&self) -> f32 {
        let ethos_norm = self.ethos.norm();
        let logos_norm = self.logos.norm();
        let pathos_norm = self.pathos.norm();
        let total = ethos_norm + logos_norm + pathos_norm;
        
        if total == 0.0 {
            return 0.0;
        }
        
        // Perfect harmony is 1/3 each
        let ethos_ratio = ethos_norm / total;
        let logos_ratio = logos_norm / total;
        let pathos_ratio = pathos_norm / total;
        
        let ideal = 1.0 / 3.0;
        let deviation = (ethos_ratio - ideal).abs() + 
                       (logos_ratio - ideal).abs() + 
                       (pathos_ratio - ideal).abs();
        
        (1.0 - deviation / 2.0).clamp(0.0, 1.0)
    }
    
    /// Convert to static ELP tensor (for backward compatibility)
    pub fn to_static_elp(&self) -> ELPTensor {
        ELPTensor {
            ethos: (self.ethos.norm() * 9.0).min(9.0) as f64,
            logos: (self.logos.norm() * 9.0).min(9.0) as f64,
            pathos: (self.pathos.norm() * 9.0).min(9.0) as f64,
        }
    }
    
    /// Serialize for Confidence Lake storage
    pub fn serialize_for_storage(&self) -> Result<Vec<u8>> {
        Ok(bincode::serialize(self)?)
    }
    
    /// Deserialize from Confidence Lake
    pub fn deserialize_from_storage(data: &[u8]) -> Result<Self> {
        Ok(bincode::deserialize(data)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dynamic_elp_creation() {
        let subject = FluxSubject::from_sacred_position(3, "test context");
        let elp = DynamicELP::from_subject(&subject, 3);
        
        assert_eq!(elp.dominant_value, 3);
        assert!(elp.ethos.norm() > 0.0);
        assert_eq!(elp.state, AttributeState::Stable);
    }
    
    #[test]
    fn test_harmony_enforcement() {
        let subject = FluxSubject::from_sacred_position(9, "emotional context");
        let mut elp = DynamicELP::from_subject(&subject, 9);
        
        // Simulate high pathos signal
        elp.adjust(1.0, 2.0);
        
        // Check that harmony was enforced
        let harmony = elp.harmony_score();
        assert!(harmony > 0.2, "Harmony score should be maintained");
    }
    
    #[test]
    fn test_importance_calculation() {
        let subject = FluxSubject::from_sacred_position(6, "logical context");
        let mut elp = DynamicELP::from_subject(&subject, 6);
        
        let initial_importance = elp.importance();
        
        // Apply significant adjustment
        elp.adjust(0.9, 1.5);
        
        assert!(elp.importance() != initial_importance, "Importance should change with adjustments");
    }
    
    #[test]
    fn test_color_from_elp() {
        let color = AttributeColor::from_elp_dominance(0.9, 0.1, 0.1);
        assert!(color.b > color.r && color.b > color.g, "Blue should dominate for high ethos");
        
        let color2 = AttributeColor::from_elp_dominance(0.1, 0.9, 0.1);
        assert!(color2.g > color2.r && color2.g > color2.b, "Green should dominate for high logos");
    }
    
    #[test]
    fn test_aspect_semantic_independence() {
        // Aspect colors based on SEMANTIC MEANING, not ELP or position
        
        // Very different contexts should produce different colors
        let subject_love = FluxSubject::from_sacred_position(3, "pure unconditional love");
        let subject_math = FluxSubject::from_sacred_position(3, "mathematical equations");
        
        let color_love = subject_love.aspect_color().unwrap();
        let color_math = subject_math.aspect_color().unwrap();
        
        // Completely different semantic meanings should have different colors
        let distance = color_love.distance(&color_math);
        assert!(distance > 0.1, 
            "Very different meanings should have distant colors (distance: {})", distance);
        
        // Verify aspects have semantic meanings
        let aspect_love = subject_love.aspect_orientation().unwrap();
        let aspect_math = subject_math.aspect_orientation().unwrap();
        
        assert_ne!(aspect_love.meaning, aspect_math.meaning, 
            "Different contexts should have different aspect meanings");
        
        // Colors are deterministic for same semantic meaning
        let subject_love_copy = FluxSubject::from_sacred_position(3, "pure unconditional love");
        let color_love_copy = subject_love_copy.aspect_color().unwrap();
        
        // Exact same meaning should produce exact same color
        assert_eq!(color_love.hue, color_love_copy.hue, 
            "Same semantic meaning should produce identical colors");
    }
    
    #[test]
    fn test_aspect_color_from_meaning() {
        let subject = FluxSubject::from_sacred_position(3, "wisdom");
        
        // Should have an aspect with semantic meaning
        assert!(subject.aspect.is_some());
        
        let aspect = subject.aspect_orientation().unwrap();
        assert!(aspect.meaning.contains("wisdom"));
        
        // Should have a color
        let color = subject.aspect_color().unwrap();
        let hex = color.to_hex();
        
        assert!(hex.starts_with('#'));
        assert_eq!(hex.len(), 7);
    }
}
