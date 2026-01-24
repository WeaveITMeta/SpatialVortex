//! Energy-Based Reasoning Model (EBRM)
//!
//! Global energy scoring for reasoning traces.

use crate::data::models::BeamTensor;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct EnergyBasedReasoningModel {
    energy_threshold: f32,
    refinement_steps: usize,
    learning_rate: f32,
}

impl Default for EnergyBasedReasoningModel {
    fn default() -> Self {
        Self { energy_threshold: 0.5, refinement_steps: 10, learning_rate: 0.01 }
    }
}

impl EnergyBasedReasoningModel {
    pub fn new() -> Self { Self::default() }

    pub fn score_trace(&self, trace: &[BeamTensor]) -> TraceEnergy {
        if trace.is_empty() { return TraceEnergy::default(); }

        let energies: Vec<f32> = trace.iter().map(|b| {
            let elp = b.attributes.elp_tensor();
            let mag = (elp[0].powi(2) + elp[1].powi(2) + elp[2].powi(2)).sqrt();
            let sacred_bonus = if matches!(b.position, 3 | 6 | 9) { 0.1 } else { 0.0 };
            (b.confidence * 0.6 + (mag / 15.0).min(1.0) * 0.4 + sacred_bonus).min(1.0)
        }).collect();

        let global = energies.iter().sum::<f32>() / trace.len() as f32;
        let sacred_count = trace.iter().filter(|b| matches!(b.position, 3 | 6 | 9)).count();

        TraceEnergy {
            global_energy: global,
            sacred_alignment: sacred_count as f32 / trace.len() as f32,
            is_valid: global >= self.energy_threshold,
        }
    }

    pub fn refine_trace(&self, trace: &mut [BeamTensor]) -> f32 {
        let initial = self.score_trace(trace).global_energy;
        for _step in 0..self.refinement_steps {
            for beam in trace.iter_mut() {
                if beam.confidence < 0.8 { beam.confidence += self.learning_rate; }
                if matches!(beam.position, 3 | 6 | 9) {
                    let e = beam.attributes.ethos();
                    beam.attributes.set_ethos(e + self.learning_rate);
                }
            }
        }
        self.score_trace(trace).global_energy - initial
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TraceEnergy {
    pub global_energy: f32,
    pub sacred_alignment: f32,
    pub is_valid: bool,
}
