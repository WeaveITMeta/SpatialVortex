//! WorldState trait — the domain abstraction layer.
//!
//! Any domain EustressEngine simulates must implement this.
//! This is the contract between the engine and Vortex.
//!
//! Grid2D, Scene3D, GameState all implement this trait.
//! The `solve()` function is generic over `W: WorldState`.

use crate::types::{DSLOp, Delta, Property, Score};
use std::fmt::Debug;

/// The core abstraction that makes Vortex domain-agnostic.
///
/// Every simulatable domain implements WorldState. The same solve loop,
/// causal graph, and hypothesis tree operate on any WorldState implementor.
pub trait WorldState: Clone + Send + Sync + Debug + 'static {
    /// What properties can Vortex observe about this state?
    /// These feed into the CausalGraph for hypothesis generation.
    fn analyze(&self) -> Vec<Property>;

    /// What DSL actions are available in this state?
    fn available_actions(&self) -> Vec<DSLOp>;

    /// Apply a DSL action, returning the new state.
    /// The action must be one from `available_actions()`.
    fn apply(&self, action: &DSLOp) -> Self;

    /// Compare this state against a goal state.
    /// Returns a score with accuracy in [0.0, 1.0] and exact_match flag.
    fn score_against(&self, goal: &Self) -> Score;

    /// Compute the diff between this state and another.
    /// Used for Iggy delta streams and causal observation.
    fn diff(&self, other: &Self) -> Vec<Delta>;

    /// Serialize for Iggy stream publishing.
    fn to_iggy_payload(&self) -> Vec<u8> {
        // Default: JSON serialization of debug repr
        format!("{:?}", self).into_bytes()
    }

    /// Human-readable summary for logging.
    fn summary(&self) -> String {
        format!("{:?}", self)
    }
}
