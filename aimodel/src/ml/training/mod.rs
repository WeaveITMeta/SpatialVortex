//! Training Module - SpectralSphereOptimizer (SSO)
//!
//! - `spectral_sphere_optimizer` - Pure Rust SSO implementation
//! - `burn_sso` - Burn-native SSO with tensor operations

pub mod spectral_sphere_optimizer;
pub mod burn_sso;

pub use spectral_sphere_optimizer::{SpectralSphereOptimizer, SSOConfig, SpectralScaler};
pub use burn_sso::{BurnSSO, SSOState, AdaptiveSSO};
