//! The system under evaluation, as the harness (014) sees it.
//!
//! Lives in the kernel so the solver side (`vortex-api`) can implement it
//! without depending on the graders, and the graders (`vortex-harness`) can
//! drive it without depending on the solvers.

use crate::task::{Response, Task};

/// The system under evaluation. `respond` takes `&self`: the evaluated
/// system cannot accumulate state from a run (no test-time training on
/// labels).
pub trait Agent: Sync {
    fn name(&self) -> String;
    fn respond(&self, task: &Task) -> Response;
}
