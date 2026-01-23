//! Subject domain definitions and mappings
//!
//! Defines the semantic mappings for different knowledge domains
//! onto the sacred geometric structure.

use std::collections::HashMap;

/// Different subject domains that can be represented
/// in the flux matrix geometry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SubjectDomain {
    /// Ethical principles (Ethos-dominant)
    Ethics,
    /// Logical concepts (Logos-dominant)
    Logic,
    /// Emotional spectrum (Pathos-dominant)
    Emotion,
}

impl SubjectDomain {
    /// Returns the dominant ELP channel for this subject
    pub fn dominant_channel(&self) -> &'static str {
        match self {
            SubjectDomain::Ethics => "Ethos",
            SubjectDomain::Logic => "Logos",
            SubjectDomain::Emotion => "Pathos",
        }
    }
    
    /// Returns all available subject domains
    pub fn all() -> Vec<SubjectDomain> {
        vec![
            SubjectDomain::Ethics,
            SubjectDomain::Logic,
            SubjectDomain::Emotion,
        ]
    }
}

/// Subject-specific flux matrix with semantic mappings
pub struct SubjectMatrix {
    /// The subject domain
    pub domain: SubjectDomain,
    /// Position → Concept mappings
    pub position_mappings: HashMap<u8, String>,
    /// Weights for this subject (learned parameters)
    pub weights: Vec<f64>,
}

impl SubjectMatrix {
    /// Creates a new subject matrix with predefined mappings
    pub fn new(domain: SubjectDomain) -> Self {
        let position_mappings = Self::create_mappings(domain);
        
        Self {
            domain,
            position_mappings,
            weights: vec![1.0; 10], // Initialize with uniform weights
        }
    }
    
    /// Creates semantic mappings based on the provided images
    fn create_mappings(domain: SubjectDomain) -> HashMap<u8, String> {
        let mut mappings = HashMap::new();
        
        match domain {
            SubjectDomain::Ethics => {
                // Ethical Principles (Ethos-dominant, red channel)
                mappings.insert(0, "Principle".to_string());
                mappings.insert(1, "Duty".to_string());
                mappings.insert(2, "Dignity".to_string());
                mappings.insert(3, "Integrity".to_string());  // Sacred
                mappings.insert(4, "Nobility".to_string());
                mappings.insert(5, "Responsibility".to_string());
                mappings.insert(6, "Honor".to_string());      // Sacred
                mappings.insert(7, "Character".to_string());
                mappings.insert(8, "Loyalty".to_string());
                mappings.insert(9, "Virtue".to_string());     // Sacred
            },
            SubjectDomain::Logic => {
                // Logical Concepts (Logos-dominant, blue channel)
                mappings.insert(0, "Reason".to_string());
                mappings.insert(1, "Hypothesis".to_string());
                mappings.insert(2, "Analysis".to_string());
                mappings.insert(3, "Axiom".to_string());      // Sacred
                mappings.insert(4, "Validation".to_string());
                mappings.insert(5, "Inference".to_string());
                mappings.insert(6, "Theorem".to_string());    // Sacred
                mappings.insert(7, "Synthesis".to_string());
                mappings.insert(8, "Deduction".to_string());
                mappings.insert(9, "Proof".to_string());      // Sacred
            },
            SubjectDomain::Emotion => {
                // Emotional Spectrum (Pathos-dominant, green channel)
                mappings.insert(0, "Curiosity".to_string());
                mappings.insert(1, "Hope".to_string());
                mappings.insert(2, "Serenity".to_string());
                mappings.insert(3, "Ecstasy".to_string());    // Sacred
                mappings.insert(4, "Surprise".to_string());
                mappings.insert(5, "Anger".to_string());
                mappings.insert(6, "Despair".to_string());    // Sacred
                mappings.insert(7, "Grief".to_string());
                mappings.insert(8, "Fear".to_string());
                mappings.insert(9, "Euphoria".to_string());   // Sacred
            },
        }
        
        mappings
    }
    
    /// Gets the concept name for a position
    pub fn get_concept(&self, position: u8) -> Option<&String> {
        self.position_mappings.get(&position)
    }
    
    /// Checks if a position is sacred (3, 6, 9)
    pub fn is_sacred(&self, position: u8) -> bool {
        matches!(position, 3 | 6 | 9)
    }
    
    /// Returns all sacred positions for this subject
    pub fn sacred_concepts(&self) -> Vec<(u8, &String)> {
        vec![
            (3, self.position_mappings.get(&3).unwrap()),
            (6, self.position_mappings.get(&6).unwrap()),
            (9, self.position_mappings.get(&9).unwrap()),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_subject_domains() {
        assert_eq!(SubjectDomain::Ethics.dominant_channel(), "Ethos");
        assert_eq!(SubjectDomain::Logic.dominant_channel(), "Logos");
        assert_eq!(SubjectDomain::Emotion.dominant_channel(), "Pathos");
    }
    
    #[test]
    fn test_ethics_mappings() {
        let ethics = SubjectMatrix::new(SubjectDomain::Ethics);
        
        assert_eq!(ethics.get_concept(9), Some(&"Virtue".to_string()));
        assert_eq!(ethics.get_concept(6), Some(&"Honor".to_string()));
        assert_eq!(ethics.get_concept(3), Some(&"Integrity".to_string()));
        assert_eq!(ethics.get_concept(0), Some(&"Principle".to_string()));
    }
    
    #[test]
    fn test_logic_mappings() {
        let logic = SubjectMatrix::new(SubjectDomain::Logic);
        
        assert_eq!(logic.get_concept(9), Some(&"Proof".to_string()));
        assert_eq!(logic.get_concept(6), Some(&"Theorem".to_string()));
        assert_eq!(logic.get_concept(3), Some(&"Axiom".to_string()));
        assert_eq!(logic.get_concept(0), Some(&"Reason".to_string()));
    }
    
    #[test]
    fn test_emotion_mappings() {
        let emotion = SubjectMatrix::new(SubjectDomain::Emotion);
        
        assert_eq!(emotion.get_concept(9), Some(&"Euphoria".to_string()));
        assert_eq!(emotion.get_concept(6), Some(&"Despair".to_string()));
        assert_eq!(emotion.get_concept(3), Some(&"Ecstasy".to_string()));
        assert_eq!(emotion.get_concept(0), Some(&"Curiosity".to_string()));
    }
    
    #[test]
    fn test_sacred_positions() {
        let ethics = SubjectMatrix::new(SubjectDomain::Ethics);
        
        assert!(ethics.is_sacred(3));
        assert!(ethics.is_sacred(6));
        assert!(ethics.is_sacred(9));
        assert!(!ethics.is_sacred(0));
        assert!(!ethics.is_sacred(1));
    }
    
    #[test]
    fn test_sacred_concepts() {
        let emotion = SubjectMatrix::new(SubjectDomain::Emotion);
        let sacred = emotion.sacred_concepts();
        
        assert_eq!(sacred.len(), 3);
        assert_eq!(sacred[0], (3, &"Ecstasy".to_string()));
        assert_eq!(sacred[1], (6, &"Despair".to_string()));
        assert_eq!(sacred[2], (9, &"Euphoria".to_string()));
    }
}
