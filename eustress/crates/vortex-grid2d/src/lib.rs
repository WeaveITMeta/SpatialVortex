//! Grid2D WorldState — ARC-AGI-3 domain implementation.
//!
//! Implements the WorldState trait for 2D integer grids (0–15 colors).
//! Provides ~50 DSL primitives for grid transformation and analysis
//! functions for symmetry, objects, color histograms, etc.

pub mod grid;
pub mod dsl;
pub mod analysis;

pub use grid::Grid2D;
pub use dsl::GridDSL;
pub use analysis::GridAnalyzer;
