//! Data Models Module
//!
//! Core data structures for sacred geometry:
//! - BeamTensor, FluxMatrix
//! - Attributes system (ELP compatibility)

pub mod attributes;
pub mod models;

pub use attributes::{Attributes, AttributeValue, AttributeAccessor, Tags};
#[allow(deprecated)]
pub use models::{BeamTensor, ELPTensor, FluxMatrix, BeadTensor};
