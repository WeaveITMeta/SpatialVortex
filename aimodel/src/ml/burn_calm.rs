//! Burn-native CALM (Continuous Autoregressive Language Models)
//!
//! High-fidelity autoencoder with Burn tensors:
//! - Compress K semantic chunks → continuous latent
//! - Autoregress in latent space (energy-based prediction)
//! - Decode back → K× fewer steps, smoother vortex orbits

use burn::tensor::{backend::Backend, Tensor, ElementConversion};
use burn::nn::{Linear, LinearConfig};
use burn::module::Module;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

/// CALM Configuration for Burn
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurnCALMConfig {
    /// Input dimension (chunk_size * features_per_token)
    pub input_dim: usize,
    /// Latent dimension for continuous space
    pub latent_dim: usize,
    /// Hidden dimension for encoder/decoder
    pub hidden_dim: usize,
    /// Number of encoder layers
    pub encoder_layers: usize,
    /// Number of decoder layers
    pub decoder_layers: usize,
    /// Dropout rate
    pub dropout: f64,
    /// Compression ratio (K)
    pub compression_ratio: usize,
}

impl Default for BurnCALMConfig {
    fn default() -> Self {
        Self {
            input_dim: 72,      // 8 tokens * 9 digits
            latent_dim: 256,
            hidden_dim: 512,
            encoder_layers: 2,
            decoder_layers: 2,
            dropout: 0.1,
            compression_ratio: 4,
        }
    }
}

impl BurnCALMConfig {
    pub fn new() -> Self { Self::default() }
    
    pub fn with_latent_dim(mut self, dim: usize) -> Self {
        self.latent_dim = dim;
        self
    }
    
    pub fn with_hidden_dim(mut self, dim: usize) -> Self {
        self.hidden_dim = dim;
        self
    }
}

/// Encoder module: input → latent
#[derive(Module, Debug)]
pub struct CALMEncoder<B: Backend> {
    input_proj: Linear<B>,
    hidden: Linear<B>,
    latent_proj: Linear<B>,
}

impl<B: Backend> CALMEncoder<B> {
    pub fn new(config: &BurnCALMConfig, device: &B::Device) -> Self {
        Self {
            input_proj: LinearConfig::new(config.input_dim, config.hidden_dim)
                .init(device),
            hidden: LinearConfig::new(config.hidden_dim, config.hidden_dim)
                .init(device),
            latent_proj: LinearConfig::new(config.hidden_dim, config.latent_dim)
                .init(device),
        }
    }

    pub fn forward(&self, x: Tensor<B, 2>) -> Tensor<B, 2> {
        // x: [batch, input_dim] → [batch, latent_dim]
        let h = self.input_proj.forward(x);
        let h = h.tanh();
        let h = self.hidden.forward(h);
        let h = h.tanh();
        self.latent_proj.forward(h)
    }
}

/// Decoder module: latent → output
#[derive(Module, Debug)]
pub struct CALMDecoder<B: Backend> {
    latent_proj: Linear<B>,
    hidden: Linear<B>,
    output_proj: Linear<B>,
}

impl<B: Backend> CALMDecoder<B> {
    pub fn new(config: &BurnCALMConfig, device: &B::Device) -> Self {
        Self {
            latent_proj: LinearConfig::new(config.latent_dim, config.hidden_dim)
                .init(device),
            hidden: LinearConfig::new(config.hidden_dim, config.hidden_dim)
                .init(device),
            output_proj: LinearConfig::new(config.hidden_dim, config.input_dim)
                .init(device),
        }
    }

    pub fn forward(&self, z: Tensor<B, 2>) -> Tensor<B, 2> {
        // z: [batch, latent_dim] → [batch, input_dim]
        let h = self.latent_proj.forward(z);
        let h = h.tanh();
        let h = self.hidden.forward(h);
        let h = h.tanh();
        let out = self.output_proj.forward(h);
        // Sigmoid approximation for probability-like outputs
        // sigmoid(x) = 1 / (1 + exp(-x))
        let neg_out = out.clone().neg();
        let exp_neg = neg_out.exp();
        let one: Tensor<B, 2> = Tensor::ones_like(&out);
        one.clone() / (one + exp_neg)
    }
}

/// Latent predictor: z_t → z_{t+1}
#[derive(Module, Debug)]
pub struct LatentPredictor<B: Backend> {
    proj1: Linear<B>,
    proj2: Linear<B>,
}

impl<B: Backend> LatentPredictor<B> {
    pub fn new(config: &BurnCALMConfig, device: &B::Device) -> Self {
        Self {
            proj1: LinearConfig::new(config.latent_dim, config.latent_dim)
                .init(device),
            proj2: LinearConfig::new(config.latent_dim, config.latent_dim)
                .init(device),
        }
    }

    pub fn forward(&self, z: Tensor<B, 2>) -> Tensor<B, 2> {
        // Residual prediction: z_{t+1} = z_t + f(z_t)
        let delta = self.proj1.forward(z.clone());
        let delta = delta.tanh();
        let delta = self.proj2.forward(delta);
        (z + delta * 0.1).tanh() // Small residual + tanh for stability
    }
}

/// Full CALM Autoencoder with Burn
#[derive(Module, Debug)]
pub struct BurnCALM<B: Backend> {
    encoder: CALMEncoder<B>,
    decoder: CALMDecoder<B>,
    predictor: LatentPredictor<B>,
}

impl<B: Backend> BurnCALM<B> {
    pub fn new(config: BurnCALMConfig, device: &B::Device) -> Self {
        Self {
            encoder: CALMEncoder::new(&config, device),
            decoder: CALMDecoder::new(&config, device),
            predictor: LatentPredictor::new(&config, device),
        }
    }

    /// Encode input to latent space
    pub fn encode(&self, x: Tensor<B, 2>) -> Tensor<B, 2> {
        self.encoder.forward(x)
    }

    /// Decode latent to output
    pub fn decode(&self, z: Tensor<B, 2>) -> Tensor<B, 2> {
        self.decoder.forward(z)
    }

    /// Predict next latent state
    pub fn predict_next(&self, z: Tensor<B, 2>) -> Tensor<B, 2> {
        self.predictor.forward(z)
    }

    /// Full autoencoder forward pass
    pub fn forward(&self, x: Tensor<B, 2>) -> Tensor<B, 2> {
        let z = self.encode(x);
        self.decode(z)
    }

    /// Reconstruction loss (MSE)
    pub fn reconstruction_loss(&self, x: Tensor<B, 2>) -> Tensor<B, 1> {
        let x_recon = self.forward(x.clone());
        let diff = x - x_recon;
        diff.powf_scalar(2.0).mean()
    }

    /// Generate K steps in latent space, decode once
    /// This is the K× speedup
    pub fn generate_compressed(&self, x: Tensor<B, 2>, steps: usize) -> Tensor<B, 2> {
        let mut z = self.encode(x);
        
        for _ in 0..steps {
            z = self.predict_next(z);
        }
        
        self.decode(z)
    }

    /// Speculative generation: multiple candidates
    pub fn generate_speculative(
        &self,
        x: Tensor<B, 2>,
        steps: usize,
        num_candidates: usize,
    ) -> Vec<Tensor<B, 2>> {
        let device = x.device();
        let z_base = self.encode(x);
        let [batch, latent_dim] = z_base.dims();
        
        let mut candidates = Vec::with_capacity(num_candidates);
        
        for c in 0..num_candidates {
            // Add small noise for diversity
            let noise_scale = 0.1 * (c as f32 + 1.0) / num_candidates as f32;
            let noise: Tensor<B, 2> = Tensor::random(
                [batch, latent_dim],
                burn::tensor::Distribution::Normal(0.0, noise_scale as f64),
                &device,
            );
            let mut z = z_base.clone() + noise;
            
            for _ in 0..steps {
                z = self.predict_next(z);
            }
            
            candidates.push(self.decode(z));
        }
        
        candidates
    }

}

/// Energy scorer for latent states (integrates with EBRM concept)
#[derive(Clone, Debug)]
pub struct LatentEnergyScorer<B: Backend> {
    _backend: PhantomData<B>,
}

impl<B: Backend> LatentEnergyScorer<B> {
    pub fn new() -> Self {
        Self { _backend: PhantomData }
    }

    /// Score a latent state based on its "energy"
    /// Lower energy = more stable/coherent state
    pub fn score(&self, z: &Tensor<B, 2>) -> f32 {
        // Energy = mean squared magnitude (simpler proxy)
        let energy: f32 = z.clone().powf_scalar(2.0).mean().into_scalar().elem();
        // Invert so higher score = better (lower energy)
        1.0 / (1.0 + energy)
    }

    /// Score multiple candidates, return best index
    pub fn select_best(&self, candidates: &[Tensor<B, 2>]) -> usize {
        candidates.iter()
            .enumerate()
            .map(|(i, z)| (i, self.score(z)))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0)
    }
}

impl<B: Backend> Default for LatentEnergyScorer<B> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use burn::backend::NdArray;

    type TestBackend = NdArray<f32>;

    #[test]
    fn test_calm_forward() {
        let device = Default::default();
        let config = BurnCALMConfig::new().with_latent_dim(64);
        let calm: BurnCALM<TestBackend> = BurnCALM::new(config, &device);

        let x: Tensor<TestBackend, 2> = Tensor::random(
            [4, 72],
            burn::tensor::Distribution::Normal(0.0, 1.0),
            &device,
        );

        let x_recon = calm.forward(x.clone());
        assert_eq!(x_recon.dims(), [4, 72]);
    }

    #[test]
    fn test_calm_encode_decode() {
        let device = Default::default();
        let config = BurnCALMConfig::new();
        let calm: BurnCALM<TestBackend> = BurnCALM::new(config, &device);

        let x: Tensor<TestBackend, 2> = Tensor::random(
            [2, 72],
            burn::tensor::Distribution::Normal(0.0, 1.0),
            &device,
        );

        let z = calm.encode(x.clone());
        assert_eq!(z.dims(), [2, 256]); // latent_dim = 256

        let x_recon = calm.decode(z);
        assert_eq!(x_recon.dims(), [2, 72]);
    }

    #[test]
    fn test_calm_generate_compressed() {
        let device = Default::default();
        let config = BurnCALMConfig::new();
        let calm: BurnCALM<TestBackend> = BurnCALM::new(config, &device);

        let x: Tensor<TestBackend, 2> = Tensor::random(
            [1, 72],
            burn::tensor::Distribution::Normal(0.0, 1.0),
            &device,
        );

        let output = calm.generate_compressed(x, 3);
        assert_eq!(output.dims(), [1, 72]);
    }

    #[test]
    fn test_latent_energy_scorer() {
        let device = Default::default();
        let scorer: LatentEnergyScorer<TestBackend> = LatentEnergyScorer::new();

        let z1: Tensor<TestBackend, 2> = Tensor::random(
            [1, 64],
            burn::tensor::Distribution::Normal(0.0, 0.5),
            &device,
        );
        let z2: Tensor<TestBackend, 2> = Tensor::random(
            [1, 64],
            burn::tensor::Distribution::Normal(0.0, 2.0),
            &device,
        );

        let score1 = scorer.score(&z1);
        let score2 = scorer.score(&z2);

        // Lower variance (z1) should have higher score
        assert!(score1 > score2);
    }
}
