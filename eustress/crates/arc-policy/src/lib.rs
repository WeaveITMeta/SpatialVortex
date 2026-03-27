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
/// Activate by swapping the call-site in the agent loop.
///
/// `success_action_counts` is built from [`eustress_common::ArcEpisodeRecord`]
/// log replay — maintained externally, passed in here to keep this module pure.
pub fn decide_heuristic(
    history: &[ArcStep],
    available_actions: &[String],
    success_action_counts: &std::collections::HashMap<String, u64>,
) -> PolicyDecision {
    if available_actions.is_empty() {
        return decide(history, available_actions);
    }

    // Pick the available action with the highest success-episode frequency.
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

// ─── Phase 3 stub: World model (Monte Carlo transition model) ────────────────

/// Placeholder — wired in Phase 3 once TransitionTable + Bayesian MC rollouts
/// are available from eustress-common's Scenarios engine.
///
/// Signature mirrors the eventual interface so the call-site swap is trivial.
pub fn decide_world_model(
    history: &[ArcStep],
    available_actions: &[String],
    // transition_table: &TransitionTable,  // uncomment in Phase 3
) -> PolicyDecision {
    // Fall through to Phase 2 / Phase 1 until world model is wired.
    decide(history, available_actions)
}
