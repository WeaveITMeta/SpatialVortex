//! Spectral Sphere Optimizer (SSO)
//!
//! Based on arXiv 2601.08393: "Controlled LLM Training on Spectral Sphere"
//!
//! Enforces module-wise spectral constraints on weights and updates:
//! - Steepest descent on spectral sphere → μP-aligned stability
//! - Bounded activations, outlier suppression, rapid convergence

use serde::{Deserialize, Serialize};

/// Spectral radius scaling strategy
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum SpectralScaler {
    #[default]
    MuP,      // R = sqrt(d_out / d_in)
    Kaiming,  // R = 1 + sqrt(d_out / d_in)
    AlignAdam,
    Fixed(f32),
}

impl SpectralScaler {
    pub fn compute_radius(&self, d_out: usize, d_in: usize) -> f32 {
        match self {
            SpectralScaler::MuP => (d_out as f32 / d_in as f32).sqrt(),
            SpectralScaler::Kaiming => 1.0 + (d_out as f32 / d_in as f32).sqrt(),
            SpectralScaler::AlignAdam => 1.0,
            SpectralScaler::Fixed(r) => *r,
        }
    }
}

/// SSO Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SSOConfig {
    pub lr: f32,
    pub momentum: f32,
    pub power_iterations: usize,
    pub scaler: SpectralScaler,
    pub weight_decay_1d: f32,
    pub lambda: f32,
    pub newton_schulz_iters: usize,
}

impl Default for SSOConfig {
    fn default() -> Self {
        Self {
            lr: 1e-3,
            momentum: 0.9,
            power_iterations: 10,
            scaler: SpectralScaler::MuP,
            weight_decay_1d: 0.01,
            lambda: 1.0,
            newton_schulz_iters: 5,
        }
    }
}

impl SSOConfig {
    pub fn new() -> Self { Self::default() }
    pub fn with_lr(mut self, lr: f32) -> Self { self.lr = lr; self }
    pub fn with_momentum(mut self, m: f32) -> Self { self.momentum = m; self }
    pub fn with_scaler(mut self, s: SpectralScaler) -> Self { self.scaler = s; self }
}

/// Spectral Sphere Optimizer
///
/// Algorithm:
/// 1. Power iteration → find principal singular vectors (u₁, v₁)
/// 2. Compute tangent projector Θ = u₁v₁ᵀ
/// 3. Project gradient: Φ = msign(G + λΘ)
/// 4. Retract to sphere: W ← W * (R / ||W||₂)
#[derive(Debug, Clone)]
pub struct SpectralSphereOptimizer {
    pub config: SSOConfig,
}

impl SpectralSphereOptimizer {
    pub fn new(config: SSOConfig) -> Self { Self { config } }

    /// Power iteration for spectral norm estimation
    /// Returns (spectral_norm, u1, v1)
    pub fn power_iteration_f32(
        &self,
        weight: &[f32],
        d_out: usize,
        d_in: usize,
        v_init: Option<&[f32]>,
    ) -> (f32, Vec<f32>, Vec<f32>) {
        let mut v: Vec<f32> = v_init.map(|v| v.to_vec()).unwrap_or_else(|| {
            (0..d_in).map(|i| ((i * 7 + 13) % 100) as f32 / 100.0 - 0.5).collect()
        });
        let v_norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        v.iter_mut().for_each(|x| *x /= v_norm);

        let mut u = vec![0.0f32; d_out];
        let mut sigma = 0.0f32;

        for _ in 0..self.config.power_iterations {
            // u = W @ v
            for i in 0..d_out {
                u[i] = (0..d_in).map(|j| weight[i * d_in + j] * v[j]).sum();
            }
            let u_norm: f32 = u.iter().map(|x| x * x).sum::<f32>().sqrt();
            sigma = u_norm;
            u.iter_mut().for_each(|x| *x /= u_norm.max(1e-8));

            // v = W^T @ u
            for j in 0..d_in {
                v[j] = (0..d_out).map(|i| weight[i * d_in + j] * u[i]).sum();
            }
            let v_norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
            v.iter_mut().for_each(|x| *x /= v_norm.max(1e-8));
        }

        (sigma, u, v)
    }

    /// Matrix sign via Newton-Schulz iterations
    pub fn matrix_sign_newton_schulz_f32(
        &self,
        grad: &[f32],
        d_out: usize,
        d_in: usize,
    ) -> Vec<f32> {
        let grad_norm: f32 = grad.iter().map(|x| x * x).sum::<f32>().sqrt();
        let scale = (d_out.max(d_in) as f32).sqrt();
        let mut x: Vec<f32> = grad.iter().map(|g| g / (grad_norm * scale).max(1e-8)).collect();

        for _ in 0..self.config.newton_schulz_iters {
            // X_{k+1} = 1.5 * X_k - 0.5 * X_k @ X_k^T @ X_k
            let mut xxt = vec![0.0f32; d_out * d_out];
            for i in 0..d_out {
                for j in 0..d_out {
                    xxt[i * d_out + j] = (0..d_in).map(|k| x[i * d_in + k] * x[j * d_in + k]).sum();
                }
            }
            let mut xxt_x = vec![0.0f32; d_out * d_in];
            for i in 0..d_out {
                for j in 0..d_in {
                    xxt_x[i * d_in + j] = (0..d_out).map(|k| xxt[i * d_out + k] * x[k * d_in + j]).sum();
                }
            }
            for i in 0..(d_out * d_in) {
                x[i] = x[i] * 1.5 - xxt_x[i] * 0.5;
            }
        }
        x
    }

    /// Full SSO update step for 2D weight matrix
    pub fn update_2d_f32(
        &self,
        weight: &mut [f32],
        grad: &[f32],
        d_out: usize,
        d_in: usize,
        v_state: &mut Option<Vec<f32>>,
    ) {
        // Power iteration
        let (sigma, u1, v1) = self.power_iteration_f32(weight, d_out, d_in, v_state.as_deref());
        *v_state = Some(v1.clone());

        // Target radius
        let target_r = self.config.scaler.compute_radius(d_out, d_in);

        // Retract to sphere
        let scale = target_r / sigma.max(1e-8);
        weight.iter_mut().for_each(|w| *w *= scale);

        // Tangent projector Θ = u₁ @ v₁ᵀ
        let mut theta = vec![0.0f32; d_out * d_in];
        for i in 0..d_out {
            for j in 0..d_in {
                theta[i * d_in + j] = u1[i] * v1[j];
            }
        }

        // Project gradient: G_proj = G + λ * Θ
        let grad_theta_dot: f32 = grad.iter().zip(theta.iter()).map(|(g, t)| g * t).sum();
        let lambda = -grad_theta_dot * self.config.lambda;
        let g_proj: Vec<f32> = grad.iter().zip(theta.iter()).map(|(g, t)| g + lambda * t).collect();

        // Matrix sign
        let phi = self.matrix_sign_newton_schulz_f32(&g_proj, d_out, d_in);

        // Apply update
        let lr = self.config.lr;
        for i in 0..(d_out * d_in) {
            weight[i] -= lr * target_r * phi[i];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spectral_scaler() {
        let scaler = SpectralScaler::MuP;
        assert!((scaler.compute_radius(512, 512) - 1.0).abs() < 0.01);
        assert!(scaler.compute_radius(1024, 512) > 1.0);
    }

    #[test]
    fn test_power_iteration() {
        let sso = SpectralSphereOptimizer::new(SSOConfig::default());
        let weight: Vec<f32> = (0..16).map(|i| (i as f32 - 8.0) / 10.0).collect();
        let (sigma, u, v) = sso.power_iteration_f32(&weight, 4, 4, None);
        assert!(sigma > 0.0);
        assert_eq!(u.len(), 4);
        assert_eq!(v.len(), 4);
    }
}
