//! Data Structures Module
//!
//! Contains core data models and operations:
//! - Models (BeamTensor, BeadTensor, ELPTensor, StoredFluxMatrix)
//! - Attributes (Universal attribute system compatible with EustressEngine)
//! - Compression algorithms
//! - Vector search
//! - E8 lattice integration (embedvec)

pub mod models;
pub mod beam_tensor;
pub mod compression;
pub mod vector_search;
pub mod elp_attributes;
pub mod aspect_color;
pub mod attributes;
pub mod e8_integration;

// Re-export main types
pub use models::{BeamTensor, BeadTensor, StoredFluxMatrix, ELPTensor};
pub use elp_attributes::{DynamicELP, FluxSubject, AttributeState, AttributeColor};
pub use aspect_color::{
    AspectOrientation, 
    AspectColor, 
    SemanticColorSpace,
    AspectTrainingData,
    AspectColorDataset,
};
// Universal Attributes (EustressEngine compatible)
pub use attributes::{Attributes, AttributeValue, AttributeAccessor, Tags, AttributeValueJson};
// E8 Lattice Integration (embedvec)
pub use e8_integration::{
    SacredE8Codec, 
    SacredE8EncodedVector, 
    SacredE8Config,
    E8FluxBridge, 
    EustressVec,
    E8FluxPosition,
    E8ELPTensor,
    E8AmortizedCache,
    ComplexityAnalysis,
};
