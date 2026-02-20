//! Sacred Geometry Module
//!
//! Core mathematical principles:
//! - 3-6-9 sacred triangle
//! - 1→2→4→8→7→5→1 vortex flow
//! - Digital root mathematics

pub mod flux_matrix;
pub mod geometric_inference;
pub mod vortex_math;
pub mod pattern_coherence;

pub use flux_matrix::FluxMatrixEngine;
pub use geometric_inference::{
    GeometricInferenceEngine,
    InferenceObservation as GeometricObservation,
    ClusterInference,
    ReducedEmbedding,
    LearnedWeights,
    BiDirectionalResult,
};
pub use vortex_math::VortexPositioningEngine;
pub use pattern_coherence::PatternCoherenceTracker;
