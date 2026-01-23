//! Generators Module
//!
//! Contains content generation components:
//! - Subject generators
//! - Visual generators
//! - Grammar graphs
//! - Subject domain definitions

#[cfg(not(target_arch = "wasm32"))]
pub mod subject;
#[cfg(not(target_arch = "wasm32"))]
pub mod visual;
pub mod grammar;
pub mod subjects;

// Re-exports
#[cfg(not(target_arch = "wasm32"))]
pub use subject::*;
#[cfg(not(target_arch = "wasm32"))]
pub use visual::*;
pub use grammar::*;
pub use subjects::*;
