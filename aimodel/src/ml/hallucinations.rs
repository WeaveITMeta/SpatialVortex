//! Vortex Context Preserver (VCP)
//!
//! Subspace hallucination detection + sacred interventions at positions 3, 6, 9.

use crate::data::models::BeamTensor;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct VortexContextPreserver {
    #[allow(dead_code)]
    subspace_rank: usize,
    magnification_factor: f32,
    detection_threshold: f32,
}

impl Default for VortexContextPreserver {
    fn default() -> Self {
        Self { subspace_rank: 8, magnification_factor: 1.15, detection_threshold: 0.3 }
    }
}

impl VortexContextPreserver {
    pub fn new() -> Self { Self::default() }

    pub fn process_with_interventions(&self, beams: &mut [BeamTensor], enable: bool) -> Vec<HallucinationResult> {
        beams.iter_mut().map(|beam| {
            if enable && matches!(beam.position, 3 | 6 | 9) {
                beam.confidence = (beam.confidence * self.magnification_factor).min(1.0);
            }
            let risk = self.calculate_risk(beam);
            HallucinationResult {
                position: beam.position,
                risk_score: risk,
                is_hallucination: risk > self.detection_threshold,
                confidence: beam.confidence,
                intervention_applied: enable && matches!(beam.position, 3 | 6 | 9),
            }
        }).collect()
    }

    fn calculate_risk(&self, beam: &BeamTensor) -> f32 {
        let confidence_risk = 1.0 - beam.confidence;
        let sacred_reduction = if matches!(beam.position, 3 | 6 | 9) { 0.15 } else { 0.0 };
        (confidence_risk - sacred_reduction).max(0.0)
    }

    pub fn signal_strength(&self, beams: &[BeamTensor]) -> f32 {
        if beams.is_empty() { return 0.0; }
        beams.iter().map(|b| b.confidence).sum::<f32>() / beams.len() as f32
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HallucinationResult {
    pub position: u8,
    pub risk_score: f32,
    pub is_hallucination: bool,
    pub confidence: f32,
    pub intervention_applied: bool,
}
