//! VortexDiscovery - Test-Time Adaptation
//!
//! Lightweight LoRA-style adapter per hard query.
//! Self-generate candidates → score via EBRM + vortex consistency + sacred alignment.
//! Refine adapter weights iteratively (test-time gradient steps).

use crate::data::models::BeamTensor;
use crate::ml::ebrm::EnergyBasedReasoningModel;
use serde::{Deserialize, Serialize};

/// VortexDiscovery Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    /// LoRA rank (low-rank adaptation dimension)
    pub lora_rank: usize,
    /// Number of test-time gradient steps
    pub adaptation_steps: usize,
    /// Learning rate for adaptation
    pub adaptation_lr: f32,
    /// Number of candidates to generate
    pub num_candidates: usize,
    /// Entropy threshold to trigger discovery
    pub entropy_threshold: f32,
    /// Sacred alignment weight in scoring
    pub sacred_weight: f32,
    /// Vortex consistency weight in scoring
    pub vortex_weight: f32,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            lora_rank: 8,
            adaptation_steps: 5,
            adaptation_lr: 0.01,
            num_candidates: 4,
            entropy_threshold: 0.7,
            sacred_weight: 0.3,
            vortex_weight: 0.2,
        }
    }
}

impl DiscoveryConfig {
    pub fn new() -> Self { Self::default() }
    pub fn with_lora_rank(mut self, r: usize) -> Self { self.lora_rank = r; self }
    pub fn with_adaptation_steps(mut self, s: usize) -> Self { self.adaptation_steps = s; self }
}

/// LoRA adapter weights (A and B matrices for low-rank decomposition)
#[derive(Debug, Clone)]
pub struct LoRAAdapter {
    /// Down-projection: d_model → lora_rank
    pub a_weights: Vec<f32>,
    /// Up-projection: lora_rank → d_model
    pub b_weights: Vec<f32>,
    /// Model dimension
    pub d_model: usize,
    /// LoRA rank
    pub rank: usize,
}

impl LoRAAdapter {
    pub fn new(d_model: usize, rank: usize) -> Self {
        // Initialize A with small random values, B with zeros (standard LoRA init)
        let a_weights: Vec<f32> = (0..d_model * rank)
            .map(|i| ((i * 7 + 13) % 100) as f32 / 1000.0 - 0.05)
            .collect();
        let b_weights = vec![0.0f32; rank * d_model];

        Self { a_weights, b_weights, d_model, rank }
    }

    /// Apply LoRA: output = x + B @ A @ x
    pub fn apply(&self, x: &[f32]) -> Vec<f32> {
        let d = self.d_model.min(x.len());
        let r = self.rank;

        // A @ x: d_model → rank
        let mut hidden = vec![0.0f32; r];
        for i in 0..r {
            for j in 0..d {
                hidden[i] += self.a_weights[i * d + j] * x[j];
            }
        }

        // B @ hidden: rank → d_model
        let mut delta = vec![0.0f32; d];
        for i in 0..d {
            for j in 0..r {
                delta[i] += self.b_weights[j * d + i] * hidden[j];
            }
        }

        // Residual: x + delta
        let mut output = x[..d].to_vec();
        for i in 0..d {
            output[i] += delta[i];
        }
        output
    }

    /// Update adapter weights with gradient step
    pub fn update(&mut self, grad_a: &[f32], grad_b: &[f32], lr: f32) {
        for (w, g) in self.a_weights.iter_mut().zip(grad_a.iter()) {
            *w -= lr * g;
        }
        for (w, g) in self.b_weights.iter_mut().zip(grad_b.iter()) {
            *w -= lr * g;
        }
    }
}

/// Discovery result with scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryResult {
    pub beams: Vec<BeamTensor>,
    pub energy_score: f32,
    pub sacred_score: f32,
    pub vortex_score: f32,
    pub combined_score: f32,
    pub adaptation_steps_used: usize,
}

/// VortexDiscovery Engine
#[derive(Debug, Clone)]
pub struct VortexDiscovery {
    pub config: DiscoveryConfig,
    ebrm: EnergyBasedReasoningModel,
    adapter: LoRAAdapter,
}

impl VortexDiscovery {
    pub fn new(config: DiscoveryConfig, d_model: usize) -> Self {
        let adapter = LoRAAdapter::new(d_model, config.lora_rank);
        Self {
            config,
            ebrm: EnergyBasedReasoningModel::new(),
            adapter,
        }
    }

    /// Check if query should trigger discovery (high entropy = novel/hard)
    pub fn should_trigger(&self, beams: &[BeamTensor]) -> bool {
        if beams.is_empty() { return false; }

        // Calculate entropy of digit distributions
        let mut total_entropy = 0.0f32;
        for beam in beams {
            let sum: f32 = beam.digits.iter().sum();
            if sum > 0.0 {
                let entropy: f32 = beam.digits.iter()
                    .map(|&d| {
                        let p = d / sum;
                        if p > 1e-6 { -p * p.ln() } else { 0.0 }
                    })
                    .sum();
                total_entropy += entropy / 9.0_f32.ln(); // Normalize by max entropy
            }
        }
        let avg_entropy = total_entropy / beams.len() as f32;

        avg_entropy > self.config.entropy_threshold
    }

    /// Score a candidate trace
    fn score_candidate(&self, beams: &[BeamTensor]) -> (f32, f32, f32, f32) {
        let energy = self.ebrm.score_trace(beams);
        
        // Vortex consistency: check if positions follow vortex pattern
        let vortex_score = self.calculate_vortex_consistency(beams);
        
        let combined = energy.global_energy * (1.0 - self.config.sacred_weight - self.config.vortex_weight)
            + energy.sacred_alignment * self.config.sacred_weight
            + vortex_score * self.config.vortex_weight;

        (energy.global_energy, energy.sacred_alignment, vortex_score, combined)
    }

    /// Calculate vortex consistency (how well positions follow 1→2→4→8→7→5→1)
    fn calculate_vortex_consistency(&self, beams: &[BeamTensor]) -> f32 {
        if beams.len() < 2 { return 1.0; }

        let expected_next = |p: u8| -> u8 {
            match p {
                1 => 2, 2 => 4, 4 => 8, 8 => 7, 7 => 5, 5 => 1,
                _ => p,
            }
        };

        let mut consistent = 0;
        for window in beams.windows(2) {
            let curr = window[0].position;
            let next = window[1].position;
            // Allow exact match or sacred positions
            if next == expected_next(curr) || matches!(next, 3 | 6 | 9) {
                consistent += 1;
            }
        }

        consistent as f32 / (beams.len() - 1) as f32
    }

    /// Generate candidates with adapter perturbations
    fn generate_candidates(&self, base_beams: &[BeamTensor]) -> Vec<Vec<BeamTensor>> {
        let mut candidates = Vec::with_capacity(self.config.num_candidates);

        for c in 0..self.config.num_candidates {
            let mut candidate: Vec<BeamTensor> = base_beams.iter().cloned().collect();
            
            // Apply adapter to digit distributions
            for beam in candidate.iter_mut() {
                let adapted = self.adapter.apply(&beam.digits);
                for (i, &v) in adapted.iter().enumerate().take(9) {
                    beam.digits[i] = v.max(0.0); // Keep non-negative
                }
                // Renormalize
                let sum: f32 = beam.digits.iter().sum();
                if sum > 0.0 {
                    beam.digits.iter_mut().for_each(|d| *d /= sum);
                }
                
                // Add small perturbation for diversity
                let noise = 0.05 * (c as f32 + 1.0) / self.config.num_candidates as f32;
                for d in beam.digits.iter_mut() {
                    *d = (*d + noise * ((*d * 1000.0) % 1.0 - 0.5)).max(0.0);
                }
            }

            candidates.push(candidate);
        }

        candidates
    }

    /// Run discovery: generate candidates, score, refine adapter
    pub fn discover(&mut self, base_beams: &[BeamTensor]) -> DiscoveryResult {
        let mut best_result = DiscoveryResult {
            beams: base_beams.to_vec(),
            energy_score: 0.0,
            sacred_score: 0.0,
            vortex_score: 0.0,
            combined_score: 0.0,
            adaptation_steps_used: 0,
        };

        for step in 0..self.config.adaptation_steps {
            // Generate candidates
            let candidates = self.generate_candidates(base_beams);

            // Score each candidate
            let mut best_idx = 0;
            let mut best_score = f32::NEG_INFINITY;
            let mut scores = Vec::with_capacity(candidates.len());

            for (i, candidate) in candidates.iter().enumerate() {
                let (energy, sacred, vortex, combined) = self.score_candidate(candidate);
                scores.push((energy, sacred, vortex, combined));
                if combined > best_score {
                    best_score = combined;
                    best_idx = i;
                }
            }

            // Update best result
            let (energy, sacred, vortex, combined) = scores[best_idx];
            if combined > best_result.combined_score {
                best_result = DiscoveryResult {
                    beams: candidates[best_idx].clone(),
                    energy_score: energy,
                    sacred_score: sacred,
                    vortex_score: vortex,
                    combined_score: combined,
                    adaptation_steps_used: step + 1,
                };
            }

            // Simple gradient approximation: move adapter toward best candidate
            // (In full impl, would compute actual gradients)
            let lr = self.config.adaptation_lr / (step as f32 + 1.0);
            for (_i, w) in self.adapter.b_weights.iter_mut().enumerate() {
                *w += lr * 0.01 * (best_idx as f32 - self.config.num_candidates as f32 / 2.0);
            }
        }

        best_result
    }

    /// Reset adapter to initial state
    pub fn reset_adapter(&mut self) {
        self.adapter = LoRAAdapter::new(self.adapter.d_model, self.config.lora_rank);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lora_adapter() {
        let adapter = LoRAAdapter::new(16, 4);
        let input: Vec<f32> = (0..16).map(|i| i as f32 / 16.0).collect();
        let output = adapter.apply(&input);
        assert_eq!(output.len(), 16);
    }

    #[test]
    fn test_discovery_trigger() {
        let discovery = VortexDiscovery::new(DiscoveryConfig::default(), 9);
        
        // Low entropy (uniform) should not trigger
        let uniform_beams: Vec<BeamTensor> = (0..4).map(|_| BeamTensor::default()).collect();
        // Default BeamTensor has uniform distribution, which has max entropy
        // So it should trigger
        assert!(discovery.should_trigger(&uniform_beams));
    }

    #[test]
    fn test_discovery_run() {
        let mut discovery = VortexDiscovery::new(
            DiscoveryConfig::new().with_adaptation_steps(2),
            9
        );
        let beams: Vec<BeamTensor> = (0..4).map(|_| BeamTensor::default()).collect();
        
        let result = discovery.discover(&beams);
        assert!(result.combined_score >= 0.0);
        assert!(result.adaptation_steps_used <= 2);
    }
}
