//! Cross-subject inference and knowledge transfer
//!
//! Enables inference across different subject domains using the shared
//! sacred geometric structure as a bridge.

use super::subject_domain::{SubjectDomain, SubjectMatrix};
use crate::models::ELPTensor;
use std::collections::HashMap;

/// Cross-subject inference result
#[derive(Debug, Clone)]
pub struct CrossSubjectResult {
    /// Source subject domain
    pub source: SubjectDomain,
    /// Target subject domain
    pub target: SubjectDomain,
    /// Source position
    pub source_position: u8,
    /// Source concept
    pub source_concept: String,
    /// Target position (mapped)
    pub target_position: u8,
    /// Target concept (mapped)
    pub target_concept: String,
    /// Inference confidence
    pub confidence: f64,
}

/// Cross-subject inference engine
///
/// Maps concepts between different subject domains using sacred positions
/// as bridges. Enables questions like:
/// - "What is the ethical equivalent of a logical proof?" → Virtue
/// - "What emotion corresponds to an axiom?" → Ecstasy
/// - "What logical concept relates to honor?" → Theorem
///
/// # Examples
///
/// ```
/// use spatial_vortex::federated::{CrossSubjectInference, SubjectDomain};
///
/// let mut inference = CrossSubjectInference::new();
///
/// // Map from Logic to Ethics
/// let result = inference.map_concept(
///     SubjectDomain::Logic,
///     9, // Proof
///     SubjectDomain::Ethics
/// );
///
/// // Result: Virtue (both at position 9)
/// ```
pub struct CrossSubjectInference {
    /// Subject matrices for lookup
    subjects: HashMap<SubjectDomain, SubjectMatrix>,
}

impl CrossSubjectInference {
    /// Creates a new cross-subject inference engine
    pub fn new() -> Self {
        let mut subjects = HashMap::new();
        
        // Initialize all subject domains
        for domain in SubjectDomain::all() {
            subjects.insert(domain, SubjectMatrix::new(domain));
        }
        
        Self { subjects }
    }
    
    /// Maps a concept from source domain to target domain
    ///
    /// Uses position mapping through sacred geometry:
    /// - Direct mapping if same position exists
    /// - Sacred bridge for cross-domain concepts
    /// - ELP channel similarity for fuzzy matching
    pub fn map_concept(
        &self,
        source_domain: SubjectDomain,
        source_position: u8,
        target_domain: SubjectDomain,
    ) -> Option<CrossSubjectResult> {
        let source_matrix = self.subjects.get(&source_domain)?;
        let target_matrix = self.subjects.get(&target_domain)?;
        
        let source_concept = source_matrix.get_concept(source_position)?.clone();
        
        // Strategy 1: Direct position mapping
        if let Some(target_concept) = target_matrix.get_concept(source_position) {
            return Some(CrossSubjectResult {
                source: source_domain,
                target: target_domain,
                source_position,
                source_concept,
                target_position: source_position,
                target_concept: target_concept.clone(),
                confidence: if source_matrix.is_sacred(source_position) {
                    1.0 // Perfect confidence for sacred positions
                } else {
                    0.8 // High confidence for direct mapping
                },
            });
        }
        
        // Strategy 2: Sacred bridge (map through nearest sacred position)
        let nearest_sacred = self.find_nearest_sacred(source_position);
        if let Some(target_concept) = target_matrix.get_concept(nearest_sacred) {
            return Some(CrossSubjectResult {
                source: source_domain,
                target: target_domain,
                source_position,
                source_concept,
                target_position: nearest_sacred,
                target_concept: target_concept.clone(),
                confidence: 0.6, // Medium confidence via bridge
            });
        }
        
        None
    }
    
    /// Finds nearest sacred position (3, 6, or 9)
    fn find_nearest_sacred(&self, position: u8) -> u8 {
        let sacred = [3, 6, 9];
        *sacred.iter()
            .min_by_key(|&&s| (s as i16 - position as i16).abs())
            .unwrap()
    }
    
    /// Gets all sacred mappings between two domains
    ///
    /// Returns concept pairs at positions 3, 6, 9 that bridge domains
    pub fn sacred_bridges(
        &self,
        source: SubjectDomain,
        target: SubjectDomain,
    ) -> Vec<CrossSubjectResult> {
        let mut bridges = Vec::new();
        
        for &pos in &[3, 6, 9] {
            if let Some(result) = self.map_concept(source, pos, target) {
                bridges.push(result);
            }
        }
        
        bridges
    }
    
    /// Computes concept similarity across domains using ELP projection
    ///
    /// Projects concepts into shared ELP space and measures distance
    pub fn concept_similarity(
        &self,
        domain1: SubjectDomain,
        pos1: u8,
        domain2: SubjectDomain,
        pos2: u8,
    ) -> f64 {
        // Map positions to ELP coordinates
        let elp1 = self.position_to_elp(domain1, pos1);
        let elp2 = self.position_to_elp(domain2, pos2);
        
        // Compute normalized distance
        let distance = ((elp1.ethos - elp2.ethos).powi(2)
            + (elp1.logos - elp2.logos).powi(2)
            + (elp1.pathos - elp2.pathos).powi(2))
            .sqrt();
        
        // Convert to similarity (0 to 1)
        1.0 / (1.0 + distance)
    }
    
    /// Projects a position to ELP coordinates based on domain
    fn position_to_elp(&self, domain: SubjectDomain, position: u8) -> ELPTensor {
        // Simplified projection based on domain dominance
        let base = (position as f64 / 9.0) * 13.0; // Scale to 13
        
        match domain {
            SubjectDomain::Ethics => ELPTensor::new(base, 0.0, 0.0),
            SubjectDomain::Logic => ELPTensor::new(0.0, base, 0.0),
            SubjectDomain::Emotion => ELPTensor::new(0.0, 0.0, base),
        }
    }
}

impl Default for CrossSubjectInference {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_inference_creation() {
        let inference = CrossSubjectInference::new();
        assert_eq!(inference.subjects.len(), 3);
    }
    
    #[test]
    fn test_direct_position_mapping() {
        let inference = CrossSubjectInference::new();
        
        // Map Proof (Logic 9) to Ethics
        let result = inference.map_concept(
            SubjectDomain::Logic,
            9,
            SubjectDomain::Ethics,
        );
        
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.source_concept, "Proof");
        assert_eq!(result.target_concept, "Virtue");
        assert_eq!(result.target_position, 9);
        assert_eq!(result.confidence, 1.0); // Sacred position
    }
    
    #[test]
    fn test_sacred_bridges() {
        let inference = CrossSubjectInference::new();
        
        let bridges = inference.sacred_bridges(
            SubjectDomain::Logic,
            SubjectDomain::Emotion,
        );
        
        assert_eq!(bridges.len(), 3); // Three sacred positions
        
        // Check the mappings
        assert_eq!(bridges[0].source_concept, "Axiom");
        assert_eq!(bridges[0].target_concept, "Ecstasy");
        
        assert_eq!(bridges[1].source_concept, "Theorem");
        assert_eq!(bridges[1].target_concept, "Despair");
        
        assert_eq!(bridges[2].source_concept, "Proof");
        assert_eq!(bridges[2].target_concept, "Euphoria");
    }
    
    #[test]
    fn test_concept_similarity() {
        let inference = CrossSubjectInference::new();
        
        // Same position across domains should have some similarity
        let sim = inference.concept_similarity(
            SubjectDomain::Ethics,
            9,
            SubjectDomain::Logic,
            9,
        );
        
        assert!(sim > 0.0);
        assert!(sim <= 1.0);
    }
    
    #[test]
    fn test_nearest_sacred() {
        let inference = CrossSubjectInference::new();
        
        assert_eq!(inference.find_nearest_sacred(0), 3);
        assert_eq!(inference.find_nearest_sacred(4), 3);
        assert_eq!(inference.find_nearest_sacred(5), 6);
        assert_eq!(inference.find_nearest_sacred(8), 9);
    }
    
    #[test]
    fn test_elp_projection() {
        let inference = CrossSubjectInference::new();
        
        let elp_ethics = inference.position_to_elp(SubjectDomain::Ethics, 9);
        assert_eq!(elp_ethics.ethos, 13.0);
        assert_eq!(elp_ethics.logos, 0.0);
        
        let elp_logic = inference.position_to_elp(SubjectDomain::Logic, 9);
        assert_eq!(elp_logic.logos, 13.0);
        assert_eq!(elp_logic.ethos, 0.0);
    }
}
