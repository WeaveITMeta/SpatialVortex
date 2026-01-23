//! Core Mathematical Foundation
//!
//! This module contains the mathematical foundations of SpatialVortex:
//! - Sacred geometry principles
//! - Vortex mathematics  
//! - Formal verification and logic
//! - Normalization functions

pub mod formal_logic;
pub mod sacred_geometry;
pub mod normalization;

// Re-exports
pub use formal_logic::{
    FormalLogicEngine,
    Axiom,
    Theorem,
    VerificationResult,
};

pub use sacred_geometry::{
    FluxMatrixEngine,
    GeometricInference,
    ChangeDot,
};

pub use normalization::*;
