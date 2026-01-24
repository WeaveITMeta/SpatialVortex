//! Machine Learning Module
//!
//! - **VCP** - Vortex Context Preserver (hallucination detection)
//! - **EBRM** - Energy-Based Reasoning Model
//! - **SSO** - Spectral Sphere Optimizer (μP-aligned)
//! - **CALM** - Continuous Autoregressive Language Models
//! - **BurnSSO** - Burn-native SSO with tensor operations
//! - **BurnCALM** - Burn-native CALM autoencoder
//! - **backends** - GPU acceleration (tch/wgpu)

pub mod hallucinations;
pub mod ebrm;
pub mod training;
pub mod calm;
pub mod burn_calm;
pub mod vortex_discovery;
pub mod backends;

pub use hallucinations::{VortexContextPreserver, HallucinationResult};
pub use ebrm::{EnergyBasedReasoningModel, TraceEnergy};
pub use training::{SpectralSphereOptimizer, SSOConfig, SpectralScaler, BurnSSO, SSOState, AdaptiveSSO};
pub use calm::{CALMEngine, CALMConfig};
pub use burn_calm::{BurnCALM, BurnCALMConfig, LatentEnergyScorer};
pub use vortex_discovery::{VortexDiscovery, DiscoveryConfig};
pub use backends::backend_info;
