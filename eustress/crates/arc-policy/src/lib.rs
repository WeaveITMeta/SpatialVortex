pub mod scene_mirror;
pub mod space_writer;
pub mod symbolic_decomposer;
pub mod world_model;

use eustress_arc_types::{ArcStep, PolicyDecision};

// ─── Phase 1: Random valid action (baseline) ────────────────────────────────

/// Select a uniformly random valid action from the observation's available
/// action set.  This is the Phase 1 baseline; replace with [`decide_heuristic`]
/// (Phase 2) or [`decide_world_model`] (Phase 3) when ready.
pub fn decide(history: &[ArcStep], available_actions: &[String]) -> PolicyDecision {
    if available_actions.is_empty() {
        return PolicyDecision {
            action: String::new(),
            confidence: 0.0,
            reasoning: "no available actions".into(),
        };
    }

    // Uniform random — no dependency on history in Phase 1.
    let idx = rand::random::<usize>() % available_actions.len();
    let action = available_actions[idx].clone();

    PolicyDecision {
        action,
        confidence: 0.0,
        reasoning: format!(
            "random baseline ({} actions available, {} steps in history)",
            available_actions.len(),
            history.len()
        ),
    }
}

// ─── Phase 2 stub: Frequency heuristic (no ML) ──────────────────────────────

/// Weight actions by their frequency in successful (goal_reached) past episodes.
pub fn decide_heuristic(
    history: &[ArcStep],
    available_actions: &[String],
    success_action_counts: &std::collections::HashMap<String, u64>,
) -> PolicyDecision {
    if available_actions.is_empty() {
        return decide(history, available_actions);
    }

    let best = available_actions
        .iter()
        .max_by_key(|a| success_action_counts.get(*a).copied().unwrap_or(0))
        .cloned()
        .unwrap_or_else(|| available_actions[0].clone());

    let total: u64 = success_action_counts.values().sum();
    let count = success_action_counts.get(&best).copied().unwrap_or(0);
    let confidence = if total > 0 {
        count as f32 / total as f32
    } else {
        0.0
    };

    PolicyDecision {
        action: best,
        confidence,
        reasoning: format!(
            "frequency heuristic: seen {count}/{total} times in goal-reached episodes"
        ),
    }
}

// ─── Phase 3: Vortex World Model (full Eustress pipeline) ───────────────────

/// World-model decision using full Eustress vortex-core infrastructure:
/// CausalGraph, HypothesisTree, Grid2D WorldState, solve loop.
///
/// The VortexWorldModel must be maintained across calls (it learns).
pub fn decide_world_model(
    model: &mut world_model::VortexWorldModel,
    step: &ArcStep,
) -> PolicyDecision {
    model.decide(step)
}
