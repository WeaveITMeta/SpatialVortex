//! SSO as a first-class Burn optimizer.
//!
//! `BurnSSO` (see [`super::burn_sso`]) already implements the full Spectral
//! Sphere / Muon update — power iteration for the spectral norm, retraction
//! onto the spectral sphere, the tangent projector `Θ = u₁v₁ᵀ`, the
//! Lagrange-projected gradient, and Newton-Schulz `msign`. What it lacked was
//! any way for Burn to *hand it gradients*: nothing in the crate called
//! `update_2d`, and `.backward()` appeared zero times repo-wide, so the
//! optimizer had no model attached to it.
//!
//! This module supplies the missing seam. It implements Burn's
//! [`SimpleOptimizer`] so SSO can be wrapped by `OptimizerAdaptor` and driven
//! by any `AutodiffModule` exactly like `Adam` or `Sgd`:
//!
//! ```rust,ignore
//! let mut optim = SsoConfig::new().with_lr(1e-3).init::<B, MyModel<B>>();
//! let loss = model.forward(x).sub(y).powf_scalar(2.0).mean();
//! let grads = GradientsParams::from_grads(loss.backward(), &model);
//! model = optim.step(lr, model, grads);
//! ```
//!
//! ## Rank dispatch
//! Muon's orthogonalized update is defined for 2-D weight matrices. Rank-2
//! tensors take the full SSO path; every other rank (biases, norms, 1-D
//! embeddings) falls back to `update_1d`'s decoupled weight decay + SGD, which
//! is the standard Muon convention.
//!
//! ## Known simplification
//! `update_2d` warm-starts its power iteration from the previous step's right
//! singular vector. Threading a `Tensor<B, 1>` through a rank-generic
//! `State<D>` is awkward, so this adapter cold-starts the power iteration each
//! step. That costs iterations, not correctness — the momentum buffer, which
//! *is* correctness-relevant, is carried across steps.

use burn::optim::adaptor::OptimizerAdaptor;
use burn::optim::SimpleOptimizer;
use burn::module::AutodiffModule;
use burn::record::Record;
use burn::tensor::backend::{AutodiffBackend, Backend};
use burn::tensor::Tensor;
use burn::LearningRate;

use super::burn_sso::{BurnSSO, SSOState};
use super::spectral_sphere_optimizer::{SSOConfig, SpectralScaler};

/// Per-parameter optimizer state.
///
/// Only `momentum` is carried across steps; see the module note on the
/// power-iteration warm start.
#[derive(Record, Clone)]
pub struct SsoOptimState<B: Backend, const D: usize> {
    /// Muon momentum buffer, in the parameter's own rank.
    pub momentum: Option<Tensor<B, D>>,
    /// Number of steps applied to this parameter.
    pub step: usize,
}

impl<B: Backend, const D: usize> SsoOptimState<B, D> {
    /// Fresh state for a parameter that has not been stepped yet.
    pub fn empty() -> Self {
        Self { momentum: None, step: 0 }
    }
}

/// Burn optimizer wrapping [`BurnSSO`].
#[derive(Clone)]
pub struct Sso<B: Backend> {
    inner: BurnSSO<B>,
    config: SSOConfig,
}

impl<B: Backend> Sso<B> {
    /// Build directly from an [`SSOConfig`].
    pub fn new(config: SSOConfig) -> Self {
        Self { inner: BurnSSO::new(config.clone()), config }
    }
}

impl<B: Backend> SimpleOptimizer<B> for Sso<B> {
    type State<const D: usize> = SsoOptimState<B, D>;

    fn step<const D: usize>(
        &self,
        lr: LearningRate,
        tensor: Tensor<B, D>,
        grad: Tensor<B, D>,
        state: Option<Self::State<D>>,
    ) -> (Tensor<B, D>, Option<Self::State<D>>) {
        let mut st = state.unwrap_or_else(SsoOptimState::empty);
        let dims = tensor.dims();

        // Rank-2 parameters get the real SSO/Muon update.
        if D == 2 {
            let (d_out, d_in) = (dims[0], dims[1]);

            let w2: Tensor<B, 2> = tensor.reshape([d_out, d_in]);
            let g2: Tensor<B, 2> = grad.reshape([d_out, d_in]);

            // Rebuild BurnSSO's own state, carrying momentum forward.
            let mut sso_state = SSOState::<B, 2>::default();
            sso_state.momentum = st
                .momentum
                .take()
                .map(|m| m.reshape([d_out, d_in]));
            sso_state.step = st.step;

            let updated = self.inner.update_2d(w2, g2, &mut sso_state, lr as f32);

            st.momentum = sso_state.momentum.map(|m| m.reshape(dims));
            st.step = sso_state.step;

            return (updated.reshape(dims), Some(st));
        }

        // Every other rank: decoupled weight decay + SGD, per Muon convention.
        let n: usize = dims.iter().product();
        let w1: Tensor<B, 1> = tensor.reshape([n]);
        let g1: Tensor<B, 1> = grad.reshape([n]);
        let updated = self.inner.update_1d(w1, g1, lr as f32);

        st.step += 1;
        (updated.reshape(dims), Some(st))
    }

    fn to_device<const D: usize>(mut state: Self::State<D>, device: &B::Device) -> Self::State<D> {
        state.momentum = state.momentum.map(|m| m.to_device(device));
        state
    }
}

/// Configuration builder that produces a ready-to-use Burn optimizer.
#[derive(Clone, Debug)]
pub struct SsoConfig {
    config: SSOConfig,
}

impl Default for SsoConfig {
    fn default() -> Self {
        Self { config: SSOConfig::default() }
    }
}

impl SsoConfig {
    /// Start from SSO defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the learning rate carried inside [`SSOConfig`].
    ///
    /// Note that Burn also passes a `LearningRate` into `step`; that is the one
    /// actually applied. This setter exists so the config round-trips.
    pub fn with_lr(mut self, lr: f32) -> Self {
        self.config.lr = lr;
        self
    }

    /// Muon momentum coefficient (0.0 disables the buffer).
    pub fn with_momentum(mut self, m: f32) -> Self {
        self.config.momentum = m;
        self
    }

    /// Spectral radius scaler (`MuP` is the μP-aligned default).
    pub fn with_scaler(mut self, s: SpectralScaler) -> Self {
        self.config.scaler = s;
        self
    }

    /// Newton-Schulz iteration count used by `msign`.
    pub fn with_newton_schulz_iters(mut self, n: usize) -> Self {
        self.config.newton_schulz_iters = n;
        self
    }

    /// Build the Burn-compatible optimizer for a given autodiff module.
    pub fn init<B: AutodiffBackend, M: AutodiffModule<B>>(
        &self,
    ) -> OptimizerAdaptor<Sso<B::InnerBackend>, M, B> {
        OptimizerAdaptor::from(Sso::new(self.config.clone()))
    }
}
