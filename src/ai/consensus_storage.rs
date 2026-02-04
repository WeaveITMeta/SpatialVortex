//! Consensus Storage - Convert Vector Fields to StoredFluxMatrix
//!
//! Solves the TODO: Text-only LLM responses now have proper flux matrix structure
//! for storage in Confidence Lake.
//!
//! ## Conversion Process
//!
//! 1. **ELP Distributions**: Map response vectors → position-based distributions
//! 2. **Pitch Curve**: Aggregate confidence trajectories into time series
//! 3. **BeadTensor**: Snapshot consensus moment with prosody-like metrics
//! 4. **Metadata**: Rich tags including diversity, approach types, sacred resonance

use crate::ai::vector_consensus::{ConsensusVectorField, ELPPoint};
use crate::data::models::{StoredFluxMatrix, ELPTensor};
use crate::data::attributes::Attributes;
use uuid::Uuid;
use chrono::Utc;
use anyhow::Result;

#[cfg(feature = "voice")]
use crate::voice_pipeline::{BeadTensor, ELPTensor as VoiceELP};

impl ConsensusVectorField {
    /// Convert consensus field to StoredFluxMatrix for Confidence Lake
    ///
    /// This is the primary integration point with the storage layer.
    /// Transforms geometric consensus representation into rich memory format.
    pub fn to_stored_flux_matrix(&self, synthesized_text: String) -> Result<StoredFluxMatrix> {
        // Create attributes with ELP values from consensus center
        let mut attributes = Attributes::with_elp(
            self.consensus_center.ethos as f32,
            self.consensus_center.logos as f32,
            self.consensus_center.pathos as f32,
        );
        
        // Add consensus-specific attributes
        attributes.set_confidence(self.field_confidence as f32);
        attributes.set_confidence(self.field_confidence as f32); // Consensus confidence = signal strength
        
        Ok(StoredFluxMatrix {
            id: Uuid::new_v4(),
            
            // Universal attributes (replaces individual ELP distributions)
            attributes,
            
            // Legacy channel distributions for backward compatibility
            ethos_distribution: Some(self.ethos_distribution_by_position()),
            logos_distribution: Some(self.logos_distribution_by_position()),
            pathos_distribution: Some(self.pathos_distribution_by_position()),
            
            // Confidence trajectories aggregated as "pitch curve"
            // (analogous to voice prosody in BeadTensor)
            pitch_curve: self.aggregate_confidence_curves(),
            
            // Synthesized Vortex response text
            text: synthesized_text,
            
            // Consensus moment snapshot as BeamTensor
            tensor: self.to_bead_tensor()?,
            
            // Model version tag
            model_version: format!("vector-consensus-v1-models-{}", self.vectors.len()),
            
            // Metadata
            created_at: Utc::now(),
            context_tags: self.get_consensus_tags(),
        })
    }
    
    /// Generate Ethos distribution across flux positions 1-9
    ///
    /// Each response vector contributes to its flux position weighted by trend.
    fn ethos_distribution_by_position(&self) -> [f32; 9] {
        self.elp_distribution_by_position(|elp| elp.ethos as f32)
    }
    
    /// Generate Logos distribution across flux positions 1-9
    fn logos_distribution_by_position(&self) -> [f32; 9] {
        self.elp_distribution_by_position(|elp| elp.logos as f32)
    }
    
    /// Generate Pathos distribution across flux positions 1-9
    fn pathos_distribution_by_position(&self) -> [f32; 9] {
        self.elp_distribution_by_position(|elp| elp.pathos as f32)
    }
    
    /// Generic helper for building ELP channel distributions
    fn elp_distribution_by_position<F>(&self, extract: F) -> [f32; 9]
    where
        F: Fn(&ELPPoint) -> f32,
    {
        let mut dist = [0.0; 9];
        
        for v in &self.vectors {
            let pos = v.flux_position as usize;
            if pos > 0 && pos <= 9 {
                let channel_value = extract(&v.elp);
                let weight = (v.trend_weight() * v.final_confidence()) as f32;
                dist[pos - 1] += channel_value * weight;
            }
        }
        
        // Normalize to sum to 1.0
        let sum: f32 = dist.iter().sum();
        if sum > 0.0 {
            dist.iter_mut().for_each(|x| *x /= sum);
        } else {
            // Fallback: uniform distribution
            dist.iter_mut().for_each(|x| *x = 1.0 / 9.0);
        }
        
        dist
    }
    
    /// Aggregate confidence trajectories into single time-series curve
    ///
    /// Represents "prosody" of consensus formation (rising/falling confidence over time).
    fn aggregate_confidence_curves(&self) -> Vec<f32> {
        if self.vectors.is_empty() {
            return vec![];
        }
        
        // Find maximum trajectory length
        let max_len = self.vectors
            .iter()
            .map(|v| v.confidence_trajectory.len())
            .max()
            .unwrap_or(1);
        
        let mut aggregated = vec![0.0_f64; max_len];
        let mut counts = vec![0; max_len];
        
        // Sum all trajectories
        for v in &self.vectors {
            let weight = v.trend_weight();
            for (i, &conf) in v.confidence_trajectory.iter().enumerate() {
                aggregated[i] += conf * weight;
                counts[i] += 1;
            }
        }
        
        // Average
        for i in 0..max_len {
            if counts[i] > 0 {
                aggregated[i] /= counts[i] as f64;
            }
        }
        
        // Convert to f32
        aggregated.iter().map(|&v| v as f32).collect()
    }
    
    /// Convert consensus to BeadTensor snapshot
    ///
    /// Treats consensus moment like a "voice bead" with:
    /// - ELP coordinates from consensus center
    /// - "Pitch" = field confidence
    /// - "Loudness" = diversity score
    /// - "Curviness" = sacred resonance
    #[cfg(feature = "voice")]
    fn to_bead_tensor(&self) -> Result<BeadTensor> {
        // Convert ELP to voice ELP format
        let elp_values = VoiceELP::new(
            self.consensus_center.ethos as f64,
            self.consensus_center.logos as f64,
            self.consensus_center.pathos as f64,
        );
        
        // Map consensus metrics to voice-like properties
        let pitch_hz = 200.0 + (self.field_confidence as f64 * 200.0);  // 200-400 Hz
        let loudness_db = -30.0 + (self.diversity_score as f64 * 20.0);  // -30 to -10 dB
        let confidence = self.field_confidence as f64;
        
        let mut bead = BeadTensor::new(
            elp_values,
            pitch_hz,
            loudness_db,
            confidence,
        );
        
        // Set curviness to sacred resonance (signed for direction)
        bead.curviness_signed = self.sacred_resonance as f64;
        
        Ok(bead)
    }
    
    /// Fallback BeadTensor when voice feature is disabled
    #[cfg(not(feature = "voice"))]
    fn to_bead_tensor(&self) -> Result<crate::data::models::BeamTensor> {
        use crate::data::models::BeamTensor;
        
        // Create BeamTensor using available data
        let mut beam = BeamTensor::default();
        beam.set_ethos(self.consensus_center.ethos as f32);
        beam.set_logos(self.consensus_center.logos as f32);
        beam.set_pathos(self.consensus_center.pathos as f32);
        beam.confidence = self.field_confidence as f32;
        beam.curviness_signed = self.sacred_resonance as f32;
        beam.timestamp = chrono::Utc::now().timestamp() as f64;
        
        Ok(beam)
    }
    
    /// Generate embedding vector for RAG/vector store queries
    ///
    /// 5D vector: [ethos, logos, pathos, confidence, diversity]
    pub fn as_embedding_vector(&self) -> Vec<f32> {
        vec![
            self.consensus_center.ethos as f32,
            self.consensus_center.logos as f32,
            self.consensus_center.pathos as f32,
            self.field_confidence as f32,
            self.diversity_score as f32,
        ]
    }
    
    /// Generate semantic query string for RAG retrieval
    pub fn to_rag_query(&self) -> String {
        let dominant_channel = if self.consensus_center.ethos > self.consensus_center.logos
            && self.consensus_center.ethos > self.consensus_center.pathos {
            "ethical principles"
        } else if self.consensus_center.logos > self.consensus_center.pathos {
            "logical analysis"
        } else {
            "emotional impact"
        };
        
        format!(
            "Retrieve documents emphasizing {} with diversity={:.2}",
            dominant_channel,
            self.diversity_score
        )
    }
}

/// Storage decision logic for Confidence Lake
pub struct ConsensusStoragePolicy {
    /// Minimum field confidence for storage (default: 0.6)
    pub min_confidence: f32,
    
    /// Minimum diversity score for storage (default: 0.5)
    pub min_diversity: f32,
    
    /// Require sacred resonance threshold (default: None)
    pub min_sacred_resonance: Option<f32>,
    
    /// Maximum storage per session to prevent overflow (default: 100)
    pub max_per_session: usize,
    
    /// Current session count
    session_count: usize,
}

impl Default for ConsensusStoragePolicy {
    fn default() -> Self {
        Self {
            min_confidence: 0.6,
            min_diversity: 0.5,
            min_sacred_resonance: None,
            max_per_session: 100,
            session_count: 0,
        }
    }
}

impl ConsensusStoragePolicy {
    /// Create new storage policy with custom thresholds
    pub fn new(min_confidence: f32, min_diversity: f32) -> Self {
        Self {
            min_confidence,
            min_diversity,
            ..Default::default()
        }
    }
    
    /// Check if consensus field should be stored
    pub fn should_store(&mut self, field: &ConsensusVectorField) -> bool {
        // Check session limit
        if self.session_count >= self.max_per_session {
            tracing::warn!("📊 Consensus storage limit reached for this session");
            return false;
        }
        
        // Check confidence threshold
        if (field.field_confidence as f32) < self.min_confidence {
            tracing::debug!(
                "📊 Consensus below confidence threshold: {:.2} < {:.2}",
                field.field_confidence,
                self.min_confidence
            );
            return false;
        }
        
        // Check diversity threshold
        if (field.diversity_score as f32) < self.min_diversity {
            tracing::debug!(
                "📊 Consensus below diversity threshold: {:.2} < {:.2}",
                field.diversity_score,
                self.min_diversity
            );
            return false;
        }
        
        // Check sacred resonance if required
        if let Some(min_sacred) = self.min_sacred_resonance {
            if (field.sacred_resonance as f32) < min_sacred {
                tracing::debug!(
                    "📊 Consensus below sacred resonance threshold: {:.2} < {:.2}",
                    field.sacred_resonance,
                    min_sacred
                );
                return false;
            }
        }
        
        self.session_count += 1;
        true
    }
    
    /// Reset session count (call at session boundaries)
    pub fn reset_session(&mut self) {
        self.session_count = 0;
    }
    
    /// Get storage statistics
    pub fn stats(&self) -> String {
        format!(
            "Stored {}/{} consensus fields (conf≥{:.2}, div≥{:.2})",
            self.session_count,
            self.max_per_session,
            self.min_confidence,
            self.min_diversity
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::vector_consensus::ResponseVector;
    
    #[test]
    fn test_consensus_to_flux_matrix() {
        let mut vectors = vec![
            ResponseVector::new("Response 1".into(), "llama3.2".into(), 0.8, 100),
            ResponseVector::new("Response 2".into(), "mixtral".into(), 0.85, 120),
        ];
        
        vectors[0].elp = ELPTensor { ethos: 5.0, logos: 7.0, pathos: 6.0 };
        vectors[0].flux_position = 3;
        vectors[1].elp = ELPTensor { ethos: 6.0, logos: 8.0, pathos: 5.0 };
        vectors[1].flux_position = 6;
        
        let field = ConsensusVectorField::from_responses(vectors);
        let matrix = field.to_stored_flux_matrix("Synthesized response".into()).unwrap();
        
        assert_eq!(matrix.text, "Synthesized response");
        assert!(matrix.context_tags.contains(&"consensus_v1".to_string()));
        assert_eq!(matrix.ethos_distribution.as_ref().unwrap().len(), 9);
        assert_eq!(matrix.logos_distribution.as_ref().unwrap().len(), 9);
        assert_eq!(matrix.pathos_distribution.as_ref().unwrap().len(), 9);
    }
    
    #[test]
    fn test_storage_policy() {
        let mut policy = ConsensusStoragePolicy::default();
        
        // High quality consensus
        let mut vectors = vec![
            ResponseVector::new("R1".into(), "m1".into(), 0.9, 100),
            ResponseVector::new("R2".into(), "m2".into(), 0.85, 100),
        ];
        vectors[0].elp = ELPTensor { ethos: 5.0, logos: 9.0, pathos: 4.0 };
        vectors[1].elp = ELPTensor { ethos: 9.0, logos: 5.0, pathos: 4.0 };
        let field = ConsensusVectorField::from_responses(vectors);
        
        assert!(policy.should_store(&field));
        
        // Low confidence consensus
        let mut vectors = vec![
            ResponseVector::new("R1".into(), "m1".into(), 0.3, 100),
        ];
        vectors[0].elp = ELPTensor { ethos: 5.0, logos: 5.0, pathos: 5.0 };
        let field = ConsensusVectorField::from_responses(vectors);
        
        assert!(!policy.should_store(&field));
    }
}
