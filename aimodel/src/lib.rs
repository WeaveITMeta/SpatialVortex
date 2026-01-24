//! AIModel - Distilled SpatialVortex AGI/ASI Seed
//!
//! Sacred-geometry-centric AI framework with:
//! - **FluxMatrixEngine** - Vortex cycles (1→2→4→8→7→5→1), 3-6-9 anchors
//! - **VCP** - Vortex Context Preserver (hallucination detection)
//! - **EBRM** - Energy-Based Reasoning Model
//! - **SSO** - Spectral Sphere Optimizer (μP-aligned)
//! - **AIConsensusEngine** - Multi-LLM fusion
//! - **Cognition** - Thinking, Memory, Constitution, RAG
//!
//! ## 2026 Distilled Stack
//! - `ort` - ONNX Runtime inference
//! - `burn` - ML training framework
//! - `wtransport` - WebTransport/QUIC
//! - `rocksdb` - Hot-path storage
//! - `embedvec` - Vector embeddings
//! - `bevy` - 3D visualization

pub mod error;
pub mod data;
pub mod core;
pub mod ml;
pub mod ai;
pub mod storage;
pub mod visualization;
pub mod transport;
pub mod cognition;
pub mod serving;

// Re-exports
pub use error::{Result, AIModelError};
pub use data::attributes::{Attributes, AttributeValue, AttributeAccessor};
pub use data::models::{BeamTensor, FluxMatrix};

// Core sacred geometry
pub use core::sacred_geometry::{
    FluxMatrixEngine,
    GeometricInferenceEngine,
    VortexPositioningEngine,
    PatternCoherenceTracker,
};

// ML components
pub use ml::{
    VortexContextPreserver,
    EnergyBasedReasoningModel,
    SpectralSphereOptimizer,
    SSOConfig,
};

// AI components
pub use ai::{
    AIConsensusEngine,
    FluxReasoningChain,
};
