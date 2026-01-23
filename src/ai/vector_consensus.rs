//! Vector Field Consensus - Geometric Multi-Model Aggregation
//!
//! Maps LLM responses into 3D ELP vector space and performs confidence-weighted
//! aggregation with diversity bonuses. Exploits geometric structure of reasoning.
//!
//! ## Key Concepts
//!
//! - **Response Vector**: LLM response mapped to ELP space with confidence trajectory
//! - **Confidence Gradient**: Rising confidence = strengthening argument (upweighted)
//! - **Approach Diversity**: Different problem-solving types get diversity bonus
//! - **Consensus Centroid**: Weighted center in ELP space represents synthesis
//!
//! ## Integration
//!
//! - **Confidence Lake**: Consensus fields stored as rich `StoredFluxMatrix` memories
//! - **RAG**: Use consensus ELP as semantic query vector
//! - **Predictive Processing**: Predict expected consensus, learn from surprise
//! - **Meta-Cognition**: Detect groupthink, blind spots, calibration issues

use crate::data::models::ELPTensor;
use serde::{Serialize, Deserialize};
use std::collections::HashSet;
use chrono::Utc;

/// Problem-solving approach classification based on ELP dominance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProblemSolvingType {
    /// Logos-dominant: analytical, logical, systematic (flux position 6 area)
    Analytical,
    
    /// Pathos-dominant: creative, intuitive, emotional (flux position 5 area)
    Creative,
    
    /// Ethos-dominant: ethical, principled, value-based (flux position 3 area)
    Ethical,
    
    /// Balanced: procedural, structured, comprehensive (flux position 9 area)
    Procedural,
    
    /// Multi-approach: synthesizing multiple perspectives (sacred positions)
    Synthesizing,
}

impl ProblemSolvingType {
    /// Get human-readable description
    pub fn description(&self) -> &str {
        match self {
            Self::Analytical => "Analytical/Logical",
            Self::Creative => "Creative/Intuitive",
            Self::Ethical => "Ethical/Principled",
            Self::Procedural => "Procedural/Structured",
            Self::Synthesizing => "Synthesizing/Multi-perspective",
        }
    }
}

/// Single LLM response mapped to vector space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseVector {
    /// ELP position in 3D space
    pub elp: ELPTensor,
    
    /// Flux position (0-9) for sacred anchoring
    pub flux_position: u8,
    
    /// Confidence trajectory samples (captured during generation)
    /// Rising trajectory = model "finding its footing" = more trustworthy
    pub confidence_trajectory: Vec<f32>,
    
    /// Problem-solving approach classification
    pub approach_type: ProblemSolvingType,
    
    /// Raw response text
    pub text: String,
    
    /// Source model name
    pub model_name: String,
    
    /// Response latency (milliseconds)
    pub latency_ms: u64,
}

impl ResponseVector {
    /// Create new response vector (ELP and flux position to be calculated)
    pub fn new(
        text: String,
        model_name: String,
        confidence: f32,
        latency_ms: u64,
    ) -> Self {
        Self {
            elp: ELPTensor { ethos: 0.0, logos: 0.0, pathos: 0.0 },
            flux_position: 0,
            confidence_trajectory: vec![confidence],
            approach_type: ProblemSolvingType::Procedural,
            text,
            model_name,
            latency_ms,
        }
    }
    
    /// Classify problem-solving approach from ELP distribution
    pub fn classify_approach(&mut self) {
        let e = self.elp.ethos;
        let l = self.elp.logos;
        let p = self.elp.pathos;
        
        let total = e + l + p;
        if total == 0.0 {
            self.approach_type = ProblemSolvingType::Procedural;
            return;
        }
        
        let e_ratio = e / total;
        let l_ratio = l / total;
        let p_ratio = p / total;
        
        // Determine dominant channel
        self.approach_type = if l_ratio > 0.5 {
            ProblemSolvingType::Analytical
        } else if p_ratio > 0.5 {
            ProblemSolvingType::Creative
        } else if e_ratio > 0.5 {
            ProblemSolvingType::Ethical
        } else if [3, 6, 9].contains(&self.flux_position) {
            ProblemSolvingType::Synthesizing
        } else {
            ProblemSolvingType::Procedural
        };
    }
    
    /// Calculate confidence trend gradient (positive = rising, negative = falling)
    ///
    /// Uses linear regression to compute slope of confidence trajectory.
    /// Rising confidence indicates model convergence = more trustworthy.
    pub fn confidence_gradient(&self) -> f32 {
        if self.confidence_trajectory.len() < 2 {
            return 0.0;
        }
        
        // Linear regression: y = mx + b, return m (slope)
        let n = self.confidence_trajectory.len() as f32;
        let x_mean = (n - 1.0) / 2.0;  // 0, 1, 2, ... n-1
        let y_mean: f32 = self.confidence_trajectory.iter().sum::<f32>() / n;
        
        let mut numerator = 0.0;
        let mut denominator = 0.0;
        
        for (i, &y) in self.confidence_trajectory.iter().enumerate() {
            let x = i as f32;
            numerator += (x - x_mean) * (y - y_mean);
            denominator += (x - x_mean).powi(2);
        }
        
        if denominator == 0.0 {
            0.0
        } else {
            numerator / denominator
        }
    }
    
    /// Calculate response weight based on confidence trend
    ///
    /// - Rising confidence (positive gradient): up to 1.5x weight
    /// - Falling confidence (negative gradient): down to 0.3x weight
    /// - Flat confidence: 1.0x weight
    pub fn trend_weight(&self) -> f32 {
        let gradient = self.confidence_gradient();
        
        if gradient > 0.0 {
            // Upward trend: bonus up to 50%
            (1.0 + gradient.min(0.5)).max(1.0)
        } else {
            // Downward trend: penalty down to 70%
            (1.0 + gradient).max(0.3)
        }
    }
    
    /// Get final confidence value (last in trajectory)
    pub fn final_confidence(&self) -> f32 {
        *self.confidence_trajectory.last().unwrap_or(&0.5)
    }
}

/// Aggregated consensus vector field from multiple LLM responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusVectorField {
    /// All response vectors (filtered for upward trends)
    pub vectors: Vec<ResponseVector>,
    
    /// Weighted centroid in ELP space (consensus center)
    pub consensus_center: ELPTensor,
    
    /// Approach diversity score (0.0-1.0)
    /// Higher = more unique problem-solving approaches
    pub diversity_score: f32,
    
    /// Aggregated field confidence (0.0-1.0)
    pub field_confidence: f32,
    
    /// Sacred position resonance (0.0-1.0)
    /// Higher when vectors cluster near positions 3, 6, 9
    pub sacred_resonance: f32,
    
    /// Timestamp of consensus formation
    pub timestamp: chrono::DateTime<Utc>,
}

impl ConsensusVectorField {
    /// Build consensus vector field from LLM responses
    ///
    /// Process:
    /// 1. Classify each approach type
    /// 2. Filter by upward confidence trend (gradient > -0.1)
    /// 3. Calculate diversity score
    /// 4. Compute weighted centroid with trend + diversity weights
    /// 5. Aggregate confidence
    /// 6. Calculate sacred resonance
    pub fn from_responses(mut vectors: Vec<ResponseVector>) -> Self {
        // Step 1: Classify approach types
        for v in &mut vectors {
            v.classify_approach();
        }
        
        // Step 2: Filter for upward/stable trends
        // Allow slight decline (-0.1) to handle noise
        vectors.retain(|v| v.confidence_gradient() > -0.1);
        
        if vectors.is_empty() {
            // Fallback: return default field
            return Self {
                vectors: vec![],
                consensus_center: ELPTensor { ethos: 5.0, logos: 5.0, pathos: 5.0 },
                diversity_score: 0.0,
                field_confidence: 0.0,
                sacred_resonance: 0.0,
                timestamp: Utc::now(),
            };
        }
        
        // Step 3: Calculate diversity
        let diversity_score = Self::calculate_diversity(&vectors);
        
        // Step 4: Weighted centroid
        let consensus_center = Self::weighted_centroid(&vectors, diversity_score);
        
        // Step 5: Aggregate confidence
        let field_confidence = Self::aggregate_confidence(&vectors);
        
        // Step 6: Sacred resonance
        let sacred_resonance = Self::calculate_sacred_resonance(&vectors);
        
        Self {
            vectors,
            consensus_center,
            diversity_score,
            field_confidence,
            sacred_resonance,
            timestamp: Utc::now(),
        }
    }
    
    /// Calculate approach diversity (unique types / total responses)
    fn calculate_diversity(vectors: &[ResponseVector]) -> f32 {
        let unique_approaches: HashSet<_> = vectors
            .iter()
            .map(|v| v.approach_type)
            .collect();
        
        unique_approaches.len() as f32 / vectors.len() as f32
    }
    
    /// Compute weighted centroid in ELP space
    ///
    /// Weight = trend_weight * base_confidence * diversity_bonus
    fn weighted_centroid(vectors: &[ResponseVector], diversity_bonus: f32) -> ELPTensor {
        let mut weighted_sum = ELPTensor {
            ethos: 0.0,
            logos: 0.0,
            pathos: 0.0,
        };
        
        let mut total_weight: f64 = 0.0;
        
        for v in vectors {
            // Calculate composite weight
            let base_weight = v.trend_weight() * v.final_confidence();
            let diversity_multiplier = 1.0 + diversity_bonus * 0.5;  // Up to 1.5x
            let weight = (base_weight * diversity_multiplier) as f64;
            
            weighted_sum.ethos += v.elp.ethos * weight;
            weighted_sum.logos += v.elp.logos * weight;
            weighted_sum.pathos += v.elp.pathos * weight;
            
            total_weight += weight;
        }
        
        if total_weight == 0.0 {
            return ELPTensor { ethos: 5.0, logos: 5.0, pathos: 5.0 };
        }
        
        ELPTensor {
            ethos: weighted_sum.ethos / total_weight,
            logos: weighted_sum.logos / total_weight,
            pathos: weighted_sum.pathos / total_weight,
        }
    }
    
    /// Aggregate confidence with gradient weighting
    fn aggregate_confidence(vectors: &[ResponseVector]) -> f32 {
        let mut weighted_conf = 0.0;
        let mut total_weight = 0.0;
        
        for v in vectors {
            let weight = v.trend_weight();
            let conf = v.final_confidence();
            
            weighted_conf += conf * weight;
            total_weight += weight;
        }
        
        if total_weight == 0.0 {
            return 0.5;
        }
        
        weighted_conf / total_weight
    }
    
    /// Calculate sacred position resonance
    ///
    /// Higher when vectors cluster near sacred positions (3, 6, 9)
    fn calculate_sacred_resonance(vectors: &[ResponseVector]) -> f32 {
        let sacred_positions = [3u8, 6, 9];
        let mut resonance_sum = 0.0;
        
        for v in vectors {
            if sacred_positions.contains(&v.flux_position) {
                // At sacred position: full resonance
                resonance_sum += 1.0;
            } else {
                // Distance to nearest sacred position
                let min_distance = sacred_positions.iter()
                    .map(|&sp| (v.flux_position as i16 - sp as i16).abs())
                    .min()
                    .unwrap_or(9) as f32;
                
                // Inverse distance: closer = higher resonance
                resonance_sum += 1.0 / (1.0 + min_distance);
            }
        }
        
        resonance_sum / vectors.len() as f32
    }
    
    /// Get dominant approach type in consensus
    pub fn dominant_approach(&self) -> Option<ProblemSolvingType> {
        let mut counts: std::collections::HashMap<ProblemSolvingType, usize> = 
            std::collections::HashMap::new();
        
        for v in &self.vectors {
            *counts.entry(v.approach_type).or_insert(0) += 1;
        }
        
        counts.into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(approach, _)| approach)
    }
    
    /// Check if consensus meets quality thresholds for storage
    pub fn is_high_quality(&self, min_confidence: f32, min_diversity: f32) -> bool {
        self.field_confidence >= min_confidence && self.diversity_score >= min_diversity
    }
    
    /// Generate tags for Confidence Lake storage
    pub fn get_consensus_tags(&self) -> Vec<String> {
        let mut tags = vec![
            format!("consensus_v1"),
            format!("confidence_{:.2}", self.field_confidence),
            format!("diversity_{:.2}", self.diversity_score),
            format!("sacred_resonance_{:.2}", self.sacred_resonance),
            format!("model_count_{}", self.vectors.len()),
        ];
        
        if let Some(dominant) = self.dominant_approach() {
            tags.push(format!("approach_{:?}", dominant).to_lowercase());
        }
        
        // Add sacred marker if high resonance
        if self.sacred_resonance > 0.7 {
            tags.push("sacred_consensus".to_string());
        }
        
        tags
    }
    
    /// Generate summary for logging/debugging
    pub fn summary(&self) -> String {
        format!(
            "Consensus: {} vectors, ELP=({:.2},{:.2},{:.2}), conf={:.2}, div={:.2}, sacred={:.2}",
            self.vectors.len(),
            self.consensus_center.ethos,
            self.consensus_center.logos,
            self.consensus_center.pathos,
            self.field_confidence,
            self.diversity_score,
            self.sacred_resonance
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_confidence_gradient() {
        let mut rv = ResponseVector::new(
            "test".to_string(),
            "model".to_string(),
            0.5,
            100,
        );
        
        // Rising confidence
        rv.confidence_trajectory = vec![0.5, 0.6, 0.7, 0.8];
        assert!(rv.confidence_gradient() > 0.0);
        assert!(rv.trend_weight() > 1.0);
        
        // Falling confidence
        rv.confidence_trajectory = vec![0.8, 0.7, 0.6, 0.5];
        assert!(rv.confidence_gradient() < 0.0);
        assert!(rv.trend_weight() < 1.0);
    }
    
    #[test]
    fn test_approach_classification() {
        let mut rv = ResponseVector::new("test".to_string(), "model".to_string(), 0.5, 100);
        
        // Logos-dominant
        rv.elp = ELPTensor { ethos: 3.0, logos: 9.0, pathos: 3.0 };
        rv.classify_approach();
        assert_eq!(rv.approach_type, ProblemSolvingType::Analytical);
        
        // Pathos-dominant
        rv.elp = ELPTensor { ethos: 3.0, logos: 3.0, pathos: 9.0 };
        rv.classify_approach();
        assert_eq!(rv.approach_type, ProblemSolvingType::Creative);
        
        // Ethos-dominant
        rv.elp = ELPTensor { ethos: 9.0, logos: 3.0, pathos: 3.0 };
        rv.classify_approach();
        assert_eq!(rv.approach_type, ProblemSolvingType::Ethical);
    }
    
    #[test]
    fn test_diversity_calculation() {
        let mut vectors = vec![
            ResponseVector::new("t1".into(), "m1".into(), 0.8, 100),
            ResponseVector::new("t2".into(), "m2".into(), 0.8, 100),
            ResponseVector::new("t3".into(), "m3".into(), 0.8, 100),
        ];
        
        // All same type
        vectors[0].elp = ELPTensor { ethos: 3.0, logos: 9.0, pathos: 3.0 };
        vectors[1].elp = ELPTensor { ethos: 3.0, logos: 9.0, pathos: 3.0 };
        vectors[2].elp = ELPTensor { ethos: 3.0, logos: 9.0, pathos: 3.0 };
        
        for v in &mut vectors {
            v.classify_approach();
        }
        
        let diversity = ConsensusVectorField::calculate_diversity(&vectors);
        assert_eq!(diversity, 1.0 / 3.0);  // Low diversity
        
        // All different types
        vectors[1].elp = ELPTensor { ethos: 9.0, logos: 3.0, pathos: 3.0 };
        vectors[2].elp = ELPTensor { ethos: 3.0, logos: 3.0, pathos: 9.0 };
        
        for v in &mut vectors {
            v.classify_approach();
        }
        
        let diversity = ConsensusVectorField::calculate_diversity(&vectors);
        assert_eq!(diversity, 1.0);  // Maximum diversity
    }
}
