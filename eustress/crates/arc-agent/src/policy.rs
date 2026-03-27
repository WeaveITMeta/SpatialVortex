use eustress_arc_types::{ArcStep, PolicyDecision};

/// Thin façade over `eustress_arc_policy`.
/// Swap the inner call to `decide_heuristic` / `decide_world_model` when
/// advancing to Phase 2 / Phase 3.
pub fn decide(history: &[ArcStep], available_actions: &[String]) -> PolicyDecision {
    eustress_arc_policy::decide(history, available_actions)
}
