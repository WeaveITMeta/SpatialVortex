//! # AIModel - Distilled SpatialVortex AGI/ASI Seed
//!
//! Sacred-geometry-centric AI framework with continuous learning through vortex dynamics.
//!
//! ## Core Components
//! - **FluxMatrixEngine** - Vortex cycles (1→2→4→8→7→5→1), 3-6-9 sacred anchors
//! - **VortexRunner** - Continuous exponential learning with u64::MAX cycle reset
//! - **ExhaustivePathwayOptimizer** - Exact O(n!) enumeration with entropic objective
//! - **ThinkingEngine** - 2^n exponential thinking cycles
//!
//! ## ML Components
//! - **VCP** - Vortex Context Preserver (hallucination detection)
//! - **EBRM** - Energy-Based Reasoning Model
//! - **SSO** - Spectral Sphere Optimizer (μP-aligned)
//! - **CALM** - Continuous Autoregressive Language Models
//!
//! ## Cognition
//! - **ThinkingEngine** - Reasoning with sacred geometry
//! - **MemoryStore** - RocksDB-backed persistent memory
//! - **RAGEngine** - Retrieval-augmented generation
//! - **ToolRegistry** - External tool use for learning
//!
//! ## Key Algorithms
//! - **Entropic Objective**: `J_β(θ) = E_s[log E_a~π[exp(β(s) R(s,a))]]`
//! - **Adaptive β(s)**: Per-state temperature for KL-bounded policy
//! - **PUCT Selection**: `Q(s) = max(children)`, `P(s) ∝ rank`, UCB bonus
//! - **9! Layering**: Exact 362,880 permutation enumeration (not Stirling's approximation)
//!
//! ## 2026 Stack
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
    CALMEngine,
    CALMConfig,
    ExhaustivePathwayOptimizer,
    PathwayConfig,
    ScoredPathway,
    CompoundingModel,
    StackedResult,
};

// AI components
pub use ai::{
    AIConsensusEngine,
    FluxReasoningChain,
};

// Cognition components
pub use cognition::{
    ThinkingEngine,
    ThinkingConfig,
    ThoughtChain,
    VortexRunner,
    VortexState,
    ToolRegistry,
    ToolCall,
    ToolResult,
    RAGEngine,
    RAGConfig,
    MemoryStore,
};
