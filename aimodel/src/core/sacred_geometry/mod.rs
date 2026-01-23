//! Sacred Geometry Module
//!
//! Contains the core mathematical principles:
//! - 3-6-9 sacred triangle
//! - 1→2→4→8→7→5→1 vortex flow
//! - Digital root mathematics
//! - Geometric inference

pub mod flux_matrix;
pub mod flux_transformer;
pub mod geometric_inference;
pub mod change_dot;
pub mod angle;
pub mod vortex_math;
pub mod pattern_coherence;
pub mod node_dynamics;
pub mod object_utils;
pub mod matrix_guided_inference;
pub mod continuous_learning;

// Re-export main types
pub use flux_matrix::FluxMatrixEngine;
pub use flux_transformer::FluxTransformer;
pub use geometric_inference::GeometricInferenceEngine;
pub use change_dot::ChangeDotIter;
pub use pattern_coherence::{PatternCoherenceTracker, CoherenceMetrics};
pub use vortex_math::{VortexPositioningEngine, FluxPosition, PositionArchetype};
pub use node_dynamics::{FluxNodeDynamics, EvaluationResult};
pub use object_utils::{create_object_context, estimate_attributes_from_query};
pub use matrix_guided_inference::{
    MatrixGuidedInference, MatrixInferenceContext,
    EnhancementMode, ResponseQualityAnalysis, QualityLevel
};
pub use continuous_learning::{
    ContinuousLearning, UserFeedback, LearningAdjustment, LearningMetrics
};

// Re-export response quality types for convenience
pub use crate::ai::response_quality::{ResponseMode, ResponseQuality, ResponseQualityAnalyzer};

// Legacy aliases for backward compatibility
pub use geometric_inference::GeometricInferenceEngine as GeometricInference;
pub use change_dot::ChangeDotIter as ChangeDot;
pub use change_dot::ChangeDotIter as ChangeDotIterator;
