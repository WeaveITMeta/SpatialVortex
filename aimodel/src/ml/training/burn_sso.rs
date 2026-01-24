//! Burn-native Spectral Sphere Optimizer (SSO)
//!
//! Implements Burn's Optimizer trait for μP-aligned spectral sphere optimization.
//! Based on arXiv 2601.08393: "Controlled LLM Training on Spectral Sphere"

use burn::tensor::{backend::Backend, Tensor, ElementConversion};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

use super::spectral_sphere_optimizer::{SSOConfig, SpectralScaler};

/// SSO state for a single parameter tensor
#[derive(Clone, Debug)]
pub struct SSOState<B: Backend, const D: usize> {
    /// Right singular vector estimate (for power iteration warm start)
    pub v_estimate: Option<Tensor<B, 1>>,
    /// Momentum buffer
    pub momentum: Option<Tensor<B, D>>,
    /// Step count
    pub step: usize,
}

impl<B: Backend, const D: usize> Default for SSOState<B, D> {
    fn default() -> Self {
        Self {
            v_estimate: None,
            momentum: None,
            step: 0,
        }
    }
}

/// Burn-native Spectral Sphere Optimizer
#[derive(Clone, Debug)]
pub struct BurnSSO<B: Backend> {
    config: SSOConfig,
    _backend: PhantomData<B>,
}

impl<B: Backend> BurnSSO<B> {
    pub fn new(config: SSOConfig) -> Self {
        Self {
            config,
            _backend: PhantomData,
        }
    }

    /// Power iteration to find principal singular vectors
    /// Returns (spectral_norm, u1, v1)
    pub fn power_iteration(
        &self,
        weight: &Tensor<B, 2>,
        v_init: Option<Tensor<B, 1>>,
    ) -> (f32, Tensor<B, 1>, Tensor<B, 1>) {
        let [d_out, d_in] = weight.dims();
        let device = weight.device();

        // Initialize v randomly or from previous estimate
        let mut v = v_init.unwrap_or_else(|| {
            Tensor::random([d_in], burn::tensor::Distribution::Normal(0.0, 1.0), &device)
        });
        
        // Normalize v
        let v_norm = v.clone().powf_scalar(2.0).sum().sqrt();
        v = v / v_norm;

        let mut u = Tensor::zeros([d_out], &device);
        let mut sigma = 0.0f32;

        for _ in 0..self.config.power_iterations {
            // u = W @ v
            u = weight.clone().matmul(v.clone().unsqueeze_dim(1)).squeeze(1);
            let u_norm = u.clone().powf_scalar(2.0).sum().sqrt();
            sigma = u_norm.clone().into_scalar().elem();
            u = u / u_norm.clamp_min(1e-8);

            // v = W^T @ u
            v = weight.clone().transpose().matmul(u.clone().unsqueeze_dim(1)).squeeze(1);
            let v_norm = v.clone().powf_scalar(2.0).sum().sqrt();
            v = v / v_norm.clamp_min(1e-8);
        }

        (sigma, u, v)
    }

    /// Newton-Schulz iterations for matrix sign approximation
    /// msign(G) ≈ U @ V^T where G = U @ Σ @ V^T
    pub fn matrix_sign_newton_schulz(&self, grad: Tensor<B, 2>) -> Tensor<B, 2> {
        let [d_out, d_in] = grad.dims();
        
        // Normalize for numerical stability
        let grad_norm: f32 = grad.clone().powf_scalar(2.0).sum().sqrt().into_scalar().elem();
        let scale = (d_out.max(d_in) as f32).sqrt();
        let divisor = (grad_norm * scale).max(1e-8);
        let mut x = grad / divisor;

        // Newton-Schulz: X_{k+1} = 1.5 * X_k - 0.5 * X_k @ X_k^T @ X_k
        for _ in 0..self.config.newton_schulz_iters {
            let xxt = x.clone().matmul(x.clone().transpose());
            let xxt_x = xxt.matmul(x.clone());
            x = x.clone() * 1.5 - xxt_x * 0.5;
        }

        x
    }

    /// SSO update for 2D weight tensor
    pub fn update_2d(
        &self,
        weight: Tensor<B, 2>,
        grad: Tensor<B, 2>,
        state: &mut SSOState<B, 2>,
        lr: f32,
    ) -> Tensor<B, 2> {
        let [d_out, d_in] = weight.dims();

        // Power iteration for spectral norm and singular vectors
        let (sigma, u1, v1) = self.power_iteration(&weight, state.v_estimate.take());
        state.v_estimate = Some(v1.clone());

        // Target radius based on scaler
        let target_r = self.config.scaler.compute_radius(d_out, d_in);

        // Retract weight to spectral sphere (pre-update correction)
        let scale = target_r / sigma.max(1e-8);
        let retracted = weight * scale;

        // Tangent projector: Θ = u₁ @ v₁ᵀ
        let theta = u1.clone().unsqueeze_dim(1).matmul(v1.unsqueeze_dim(0));

        // Project gradient with Lagrange multiplier
        // λ = -⟨G, Θ⟩ * config.lambda
        let grad_theta_dot: f32 = (grad.clone() * theta.clone()).sum().into_scalar().elem();
        let lambda = -grad_theta_dot * self.config.lambda;
        let g_proj = grad + theta * lambda;

        // Matrix sign for steepest descent direction
        let phi = self.matrix_sign_newton_schulz(g_proj);

        // Apply momentum if configured
        let update = if self.config.momentum > 0.0 {
            let momentum_update = if let Some(ref m) = state.momentum {
                m.clone() * self.config.momentum + phi.clone() * (1.0 - self.config.momentum)
            } else {
                phi.clone()
            };
            state.momentum = Some(momentum_update.clone());
            momentum_update
        } else {
            phi
        };

        state.step += 1;

        // Final update: W ← W_retracted - lr * R * Φ
        retracted - update * (lr * target_r)
    }

    /// SSO update for 1D parameter (embeddings, norms) - standard weight decay
    pub fn update_1d(
        &self,
        weight: Tensor<B, 1>,
        grad: Tensor<B, 1>,
        lr: f32,
    ) -> Tensor<B, 1> {
        let wd = self.config.weight_decay_1d;
        weight.clone() * (1.0 - lr * wd) - grad * lr
    }
}

/// SSO Record for serialization (config only, state is transient)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SSORecord {
    pub config: SSOConfig,
}

/// Adaptive SSO that handles different tensor dimensions
#[derive(Clone, Debug)]
pub struct AdaptiveSSO<B: Backend> {
    inner: BurnSSO<B>,
    lr: f32,
}

impl<B: Backend> AdaptiveSSO<B> {
    pub fn new(config: SSOConfig) -> Self {
        let lr = config.lr;
        Self {
            inner: BurnSSO::new(config),
            lr,
        }
    }

    pub fn with_lr(mut self, lr: f32) -> Self {
        self.lr = lr;
        self
    }

    /// Update a 2D tensor (weight matrices)
    pub fn step_2d(
        &self,
        weight: Tensor<B, 2>,
        grad: Tensor<B, 2>,
        state: &mut SSOState<B, 2>,
    ) -> Tensor<B, 2> {
        self.inner.update_2d(weight, grad, state, self.lr)
    }

    /// Update a 1D tensor (biases, embeddings)
    pub fn step_1d(
        &self,
        weight: Tensor<B, 1>,
        grad: Tensor<B, 1>,
    ) -> Tensor<B, 1> {
        self.inner.update_1d(weight, grad, self.lr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use burn::backend::NdArray;

    type TestBackend = NdArray<f32>;

    #[test]
    fn test_power_iteration() {
        let config = SSOConfig::default();
        let sso: BurnSSO<TestBackend> = BurnSSO::new(config);
        
        let device = Default::default();
        let weight: Tensor<TestBackend, 2> = Tensor::random([4, 4], burn::tensor::Distribution::Normal(0.0, 1.0), &device);
        
        let (sigma, u, v) = sso.power_iteration(&weight, None);
        
        assert!(sigma > 0.0);
        assert_eq!(u.dims(), [4]);
        assert_eq!(v.dims(), [4]);
    }

    #[test]
    fn test_sso_update() {
        let config = SSOConfig::new().with_lr(0.01);
        let sso: BurnSSO<TestBackend> = BurnSSO::new(config);
        
        let device = Default::default();
        let weight: Tensor<TestBackend, 2> = Tensor::random([8, 8], burn::tensor::Distribution::Normal(0.0, 1.0), &device);
        let grad: Tensor<TestBackend, 2> = Tensor::random([8, 8], burn::tensor::Distribution::Normal(0.0, 0.1), &device);
        
        let mut state = SSOState::default();
        let updated = sso.update_2d(weight.clone(), grad, &mut state, 0.01);
        
        assert_eq!(updated.dims(), [8, 8]);
        assert!(state.v_estimate.is_some());
        assert_eq!(state.step, 1);
    }

    #[test]
    fn test_spectral_constraint() {
        let config = SSOConfig::new().with_scaler(SpectralScaler::MuP);
        let sso: BurnSSO<TestBackend> = BurnSSO::new(config);
        
        let device = Default::default();
        // Create weight with known spectral norm
        let weight: Tensor<TestBackend, 2> = Tensor::random([16, 16], burn::tensor::Distribution::Normal(0.0, 1.0), &device);
        let grad: Tensor<TestBackend, 2> = Tensor::random([16, 16], burn::tensor::Distribution::Normal(0.0, 0.1), &device);
        
        let mut state = SSOState::default();
        let updated = sso.update_2d(weight, grad, &mut state, 0.01);
        
        // Check spectral norm is close to target (sqrt(16/16) = 1.0 for MuP)
        let (sigma, _, _) = sso.power_iteration(&updated, None);
        // After one step, should be close to target
        assert!(sigma > 0.5 && sigma < 2.0);
    }
}
