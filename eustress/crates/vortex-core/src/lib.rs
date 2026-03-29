//! Vortex Core — Domain-agnostic learning architecture
//!
//! Universal loop: OBSERVE → HYPOTHESIZE → SIMULATE → EVALUATE → INTERNALIZE
//!
//! The same `solve::<W: WorldState>()` function handles Grid2D, Scene3D,
//! GameState, or any domain that implements the WorldState trait.

pub mod world_state;
pub mod causal_graph;
pub mod hypothesis_tree;
pub mod symbol_resolver;
pub mod solve;
pub mod types;

pub use world_state::WorldState;
pub use causal_graph::CausalGraph;
pub use hypothesis_tree::HypothesisTree;
pub use symbol_resolver::SymbolResolver;
pub use solve::solve;
pub use types::*;
