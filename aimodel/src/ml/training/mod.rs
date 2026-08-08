//! Training Module - SpectralSphereOptimizer (SSO)
//!
//! - `spectral_sphere_optimizer` - Pure Rust SSO implementation
//! - `burn_sso` - Burn-native SSO with tensor operations
//! - `sso_optimizer` - SSO as a Burn `SimpleOptimizer`, so autodiff gradients
//!   can actually reach it (`.backward()` -> `GradientsParams` -> SSO step)

pub mod spectral_sphere_optimizer;
pub mod burn_sso;
pub mod sso_optimizer;

pub use spectral_sphere_optimizer::{SpectralSphereOptimizer, SSOConfig, SpectralScaler};
pub use burn_sso::{BurnSSO, SSOState, AdaptiveSSO};
pub use sso_optimizer::{Sso, SsoConfig, SsoOptimState};
