//! CALM - Continuous Autoregressive Language Models
//!
//! High-fidelity autoencoder compressing K semantic chunks → continuous latent.
//! Autoregress in latent space with energy-based prediction.
//! Decode back → K× fewer steps, smoother vortex orbits.

use crate::data::models::BeamTensor;
use crate::ml::ebrm::EnergyBasedReasoningModel;
use serde::{Deserialize, Serialize};

/// CALM Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CALMConfig {
    /// Latent dimension for continuous space
    pub latent_dim: usize,
    /// Number of semantic chunks to compress
    pub chunk_size: usize,
    /// Compression ratio (K)
    pub compression_ratio: usize,
    /// Energy threshold for latent predictions
    pub energy_threshold: f32,
    /// Enable speculative decoding
    pub speculative_decoding: bool,
    /// Batch size for parallel decoding
    pub batch_size: usize,
}

impl Default for CALMConfig {
    fn default() -> Self {
        Self {
            latent_dim: 256,
            chunk_size: 8,
            compression_ratio: 4,
            energy_threshold: 0.5,
            speculative_decoding: true,
            batch_size: 4,
        }
    }
}

impl CALMConfig {
    pub fn new() -> Self { Self::default() }
    pub fn with_latent_dim(mut self, dim: usize) -> Self { self.latent_dim = dim; self }
    pub fn with_compression_ratio(mut self, k: usize) -> Self { self.compression_ratio = k; self }
}

/// Continuous latent representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatentState {
    /// Continuous latent vector
    pub latent: Vec<f32>,
    /// Energy score from EBRM
    pub energy: f32,
    /// Sacred alignment (0-1)
    pub sacred_alignment: f32,
    /// Step in the latent sequence
    pub step: usize,
}

impl LatentState {
    pub fn new(latent_dim: usize) -> Self {
        Self {
            latent: vec![0.0; latent_dim],
            energy: 0.0,
            sacred_alignment: 0.0,
            step: 0,
        }
    }
}

/// CALM Engine - Continuous Autoregressive Language Model
///
/// Integrates with EBRM for energy-based latent space prediction.
/// Provides K× speedup through latent compression.
#[derive(Debug, Clone)]
pub struct CALMEngine {
    pub config: CALMConfig,
    ebrm: EnergyBasedReasoningModel,
    /// Encoder weights (simplified - would be Burn tensors in full impl)
    encoder_weights: Vec<f32>,
    /// Decoder weights
    decoder_weights: Vec<f32>,
    /// Latent predictor weights
    predictor_weights: Vec<f32>,
}

impl CALMEngine {
    pub fn new(config: CALMConfig) -> Self {
        let latent_dim = config.latent_dim;
        let input_dim = config.chunk_size * 9; // 9 digits per BeamTensor
        
        Self {
            config,
            ebrm: EnergyBasedReasoningModel::new(),
            encoder_weights: vec![0.01; input_dim * latent_dim],
            decoder_weights: vec![0.01; latent_dim * input_dim],
            predictor_weights: vec![0.01; latent_dim * latent_dim],
        }
    }

    /// Encode a chunk of BeamTensors to continuous latent
    pub fn encode(&self, beams: &[BeamTensor]) -> LatentState {
        let chunk_size = self.config.chunk_size.min(beams.len());
        let latent_dim = self.config.latent_dim;
        
        // Flatten beam digits to input
        let mut input = Vec::with_capacity(chunk_size * 9);
        for beam in beams.iter().take(chunk_size) {
            input.extend_from_slice(&beam.digits);
        }
        
        // Pad if needed
        while input.len() < self.config.chunk_size * 9 {
            input.push(0.0);
        }

        // Simple linear encoding (would be neural network in full impl)
        let mut latent = vec![0.0f32; latent_dim];
        let input_dim = self.config.chunk_size * 9;
        for i in 0..latent_dim {
            for j in 0..input_dim.min(input.len()) {
                latent[i] += input[j] * self.encoder_weights[i * input_dim + j];
            }
            // Apply tanh activation
            latent[i] = latent[i].tanh();
        }

        // Score with EBRM
        let energy = self.ebrm.score_trace(beams);

        LatentState {
            latent,
            energy: energy.global_energy,
            sacred_alignment: energy.sacred_alignment,
            step: 0,
        }
    }

    /// Predict next latent state (autoregressive in latent space)
    pub fn predict_next(&self, state: &LatentState) -> LatentState {
        let latent_dim = self.config.latent_dim;
        let mut next_latent = vec![0.0f32; latent_dim];

        // Linear prediction + residual connection
        for i in 0..latent_dim {
            for j in 0..latent_dim {
                next_latent[i] += state.latent[j] * self.predictor_weights[i * latent_dim + j];
            }
            // Residual + tanh
            next_latent[i] = (next_latent[i] + state.latent[i] * 0.5).tanh();
        }

        // Energy decays slightly without grounding
        let energy_decay = 0.95;

        LatentState {
            latent: next_latent,
            energy: state.energy * energy_decay,
            sacred_alignment: state.sacred_alignment,
            step: state.step + 1,
        }
    }

    /// Decode latent state back to BeamTensors
    pub fn decode(&self, state: &LatentState) -> Vec<BeamTensor> {
        let chunk_size = self.config.chunk_size;
        let latent_dim = self.config.latent_dim;
        let output_dim = chunk_size * 9;

        // Linear decoding
        let mut output = vec![0.0f32; output_dim];
        for i in 0..output_dim {
            for j in 0..latent_dim {
                output[i] += state.latent[j] * self.decoder_weights[j * output_dim + i];
            }
            // Sigmoid to get probabilities
            output[i] = 1.0 / (1.0 + (-output[i]).exp());
        }

        // Convert to BeamTensors
        let mut beams = Vec::with_capacity(chunk_size);
        for c in 0..chunk_size {
            let mut digits = [0.0f32; 9];
            for d in 0..9 {
                digits[d] = output[c * 9 + d];
            }
            // Normalize to sum to 1
            let sum: f32 = digits.iter().sum();
            if sum > 0.0 {
                digits.iter_mut().for_each(|d| *d /= sum);
            }

            let mut beam = BeamTensor::default();
            beam.digits = digits;
            beam.confidence = state.energy;
            beam.position = (c as u8 % 9) + 1;
            beams.push(beam);
        }

        beams
    }

    /// Generate K steps in latent space, decode once
    /// This is the K× speedup: instead of K autoregressive token steps,
    /// we do K latent predictions then decode all at once.
    pub fn generate_compressed(&self, initial_beams: &[BeamTensor], steps: usize) -> Vec<BeamTensor> {
        let mut state = self.encode(initial_beams);
        
        // Autoregress in latent space (fast!)
        for _ in 0..steps {
            state = self.predict_next(&state);
            
            // Early exit if energy drops too low
            if state.energy < self.config.energy_threshold {
                break;
            }
        }

        // Decode back to BeamTensors (single decode for K steps)
        self.decode(&state)
    }

    /// Speculative decoding: generate multiple candidates, score with EBRM
    pub fn generate_speculative(&self, initial_beams: &[BeamTensor], steps: usize) -> Vec<BeamTensor> {
        if !self.config.speculative_decoding {
            return self.generate_compressed(initial_beams, steps);
        }

        let batch_size = self.config.batch_size;
        let mut candidates: Vec<Vec<BeamTensor>> = Vec::with_capacity(batch_size);

        // Generate multiple candidates with slight perturbations
        for b in 0..batch_size {
            let mut state = self.encode(initial_beams);
            
            // Add small noise for diversity
            let noise_scale = 0.1 * (b as f32 + 1.0) / batch_size as f32;
            for l in state.latent.iter_mut() {
                *l += noise_scale * ((*l * 1000.0) % 1.0 - 0.5);
            }

            for _ in 0..steps {
                state = self.predict_next(&state);
            }

            candidates.push(self.decode(&state));
        }

        // Score each candidate with EBRM, pick best
        let mut best_idx = 0;
        let mut best_energy = 0.0f32;
        for (i, candidate) in candidates.iter().enumerate() {
            let energy = self.ebrm.score_trace(candidate);
            if energy.global_energy > best_energy {
                best_energy = energy.global_energy;
                best_idx = i;
            }
        }

        candidates.swap_remove(best_idx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calm_encode_decode() {
        let config = CALMConfig::new().with_latent_dim(64);
        let engine = CALMEngine::new(config);

        let beams: Vec<BeamTensor> = (0..4).map(|_| BeamTensor::default()).collect();
        let state = engine.encode(&beams);
        
        assert_eq!(state.latent.len(), 64);
        assert!(state.energy >= 0.0);

        let decoded = engine.decode(&state);
        assert!(!decoded.is_empty());
    }

    #[test]
    fn test_calm_generate() {
        let engine = CALMEngine::new(CALMConfig::default());
        let initial: Vec<BeamTensor> = (0..4).map(|_| BeamTensor::default()).collect();
        
        let result = engine.generate_compressed(&initial, 3);
        assert!(!result.is_empty());
    }
}
