//! The domain-agnostic solve function.
//!
//! ```ignore
//! let arc_solution = solve::<Grid2D>(&input, &output, &mut causal_graph, &budget);
//! let workshop_solution = solve::<Scene3D>(&initial, &goal, &mut causal_graph, &budget);
//! ```
//!
//! The SAME function handles ARC grids and 3D tool-use tasks.
//! Only the type parameter changes.

use crate::causal_graph::CausalGraph;
use crate::hypothesis_tree::{HypothesisTree, SimulationBudget};
use crate::symbol_resolver::SymbolResolver;
use crate::types::*;
use crate::world_state::WorldState;
use tracing::{debug, info};

/// Result of the solve loop.
#[derive(Clone, Debug)]
pub struct SolveResult {
    /// The action sequence that achieved the best score.
    pub program: Vec<DSLOp>,
    /// Score of the best branch.
    pub score: Score,
    /// Whether exact match was achieved.
    pub solved: bool,
    /// Episode record for causal graph integration.
    pub episode: EpisodeRecord,
    /// Properties observed in the initial state.
    pub observed_properties: Vec<Property>,
}

/// Domain-agnostic solve loop.
///
/// OBSERVE → HYPOTHESIZE → SIMULATE → EVALUATE → INTERNALIZE
///
/// Works on Grid2D, Scene3D, GameState, or anything that implements WorldState.
pub fn solve<W: WorldState>(
    world: &W,
    goal: &W,
    causal_graph: &mut CausalGraph,
    budget: &SimulationBudget,
) -> SolveResult {
    let task_id = uuid::Uuid::new_v4().to_string();
    let start = std::time::Instant::now();

    // ── 1. OBSERVE ───────────────────────────────────────────────────
    let properties = world.analyze();
    let available = world.available_actions();

    info!(
        "solve: {} properties detected, {} actions available",
        properties.len(), available.len()
    );

    // ── 2. HYPOTHESIZE ───────────────────────────────────────────────
    let hypotheses = causal_graph.suggest_hypotheses(
        &properties,
        &GoalPredicate::ExactMatch,
        budget.max_active_branches,
    );

    debug!("CausalGraph suggested {} hypotheses", hypotheses.len());

    // If causal graph has no suggestions, generate exploratory hypotheses
    // from available actions (brute-force enumeration as fallback)
    let mut all_hypotheses = hypotheses;
    if all_hypotheses.len() < budget.max_active_branches {
        let remaining = budget.max_active_branches - all_hypotheses.len();
        let exploratory = generate_exploratory_hypotheses(&available, remaining, budget.max_branch_depth);
        debug!("Adding {} exploratory hypotheses", exploratory.len());
        all_hypotheses.extend(exploratory);
    }

    // ── 3. SIMULATE ──────────────────────────────────────────────────
    let mut tree = HypothesisTree::new(budget.clone());

    for h in &all_hypotheses {
        tree.add_branch(h.clone());
    }

    while let Some(branch_id) = tree.next_branch() {
        if let Some(score) = tree.simulate_branch(branch_id, world, goal) {
            tree.score_branch(branch_id, score.clone());

            // If score is close but not exact, try refinements
            if !score.exact_match && score.accuracy > 0.5 {
                // Generate refinement hypotheses
                let refinements = generate_refinement_hypotheses(
                    &available,
                    &all_hypotheses,
                    3, // max refinement children
                );
                for r in refinements {
                    tree.add_child_branch(branch_id, r);
                }
            }
        }
    }

    // ── 4. EVALUATE ──────────────────────────────────────────────────
    let stats = tree.stats();
    info!("Solve complete: {}", stats);

    let (program, score, solved) = if let Some(best) = tree.best_completed_branch() {
        (best.trajectory.clone(), best.score.clone(), best.success)
    } else {
        (vec![], Score::zero(), false)
    };

    // Compute state deltas for the episode record
    let mut state_deltas = Vec::new();
    let mut current = world.clone();
    for op in &program {
        let next = current.apply(op);
        let deltas = current.diff(&next);
        state_deltas.extend(deltas);
        current = next;
    }

    let episode = EpisodeRecord {
        episode_id: uuid::Uuid::new_v4().to_string(),
        task_id: task_id.clone(),
        observed_properties: properties.clone(),
        actions_taken: program.clone(),
        state_deltas,
        final_score: score.clone(),
        success: solved,
        duration_ms: start.elapsed().as_millis() as u64,
    };

    // ── 5. INTERNALIZE ───────────────────────────────────────────────
    causal_graph.integrate_episode(&episode);

    // Run SymbolResolver to check for new law unifications
    let mut resolver = SymbolResolver::new();
    let new_laws = resolver.scan_and_unify(causal_graph);
    if new_laws > 0 {
        info!("SymbolResolver created {} new laws after episode", new_laws);
    }

    SolveResult {
        program,
        score,
        solved,
        episode,
        observed_properties: properties,
    }
}

/// Generate exploratory hypotheses from available actions.
/// Single-action programs for characterization, plus short random compositions.
fn generate_exploratory_hypotheses(
    available: &[DSLOp],
    max_count: usize,
    _max_depth: usize,
) -> Vec<crate::causal_graph::Hypothesis> {
    use crate::causal_graph::Hypothesis;

    let mut hypotheses = Vec::new();

    // Single-action characterization
    for action in available.iter().take(max_count) {
        hypotheses.push(Hypothesis {
            description: format!("explore: single {}", action.name),
            program: vec![action.clone()],
            source_law: None,
            prior_confidence: 0.1,
            motivating_properties: vec![],
        });
    }

    // Two-action compositions (if budget allows)
    if hypotheses.len() < max_count && available.len() >= 2 {
        let remaining = max_count - hypotheses.len();
        let mut count = 0;
        'outer: for a in available {
            for b in available {
                if count >= remaining {
                    break 'outer;
                }
                hypotheses.push(Hypothesis {
                    description: format!("explore: {} → {}", a.name, b.name),
                    program: vec![a.clone(), b.clone()],
                    source_law: None,
                    prior_confidence: 0.05,
                    motivating_properties: vec![],
                });
                count += 1;
            }
        }
    }

    hypotheses
}

/// Generate refinement hypotheses for a promising but imperfect branch.
fn generate_refinement_hypotheses(
    available: &[DSLOp],
    _existing: &[crate::causal_graph::Hypothesis],
    max_count: usize,
) -> Vec<crate::causal_graph::Hypothesis> {
    use crate::causal_graph::Hypothesis;

    // Simple refinement: append each available action as a fix-up step
    available.iter().take(max_count).map(|action| {
        Hypothesis {
            description: format!("refine: append {}", action.name),
            program: vec![action.clone()],
            source_law: None,
            prior_confidence: 0.2,
            motivating_properties: vec![],
        }
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Trivial WorldState for testing the solve loop.
    #[derive(Clone, Debug)]
    struct TestWorld {
        value: i32,
        goal_value: i32,
    }

    impl WorldState for TestWorld {
        fn analyze(&self) -> Vec<Property> {
            vec![Property {
                name: "value".into(),
                domain: Domain::Universal,
                value: PropertyValue::Int(self.value as i64),
            }]
        }

        fn available_actions(&self) -> Vec<DSLOp> {
            vec![
                DSLOp { name: "increment".into(), domain: Domain::Universal, parameters: vec![] },
                DSLOp { name: "double".into(), domain: Domain::Universal, parameters: vec![] },
            ]
        }

        fn apply(&self, action: &DSLOp) -> Self {
            match action.name.as_str() {
                "increment" => TestWorld { value: self.value + 1, goal_value: self.goal_value },
                "double" => TestWorld { value: self.value * 2, goal_value: self.goal_value },
                _ => self.clone(),
            }
        }

        fn score_against(&self, goal: &Self) -> Score {
            if self.value == goal.value {
                Score::perfect()
            } else {
                let diff = (self.value - goal.value).unsigned_abs() as f64;
                let max_diff = goal.value.unsigned_abs().max(1) as f64;
                Score::from_accuracy((1.0 - diff / max_diff).max(0.0))
            }
        }

        fn diff(&self, other: &Self) -> Vec<Delta> {
            if self.value != other.value {
                vec![Delta {
                    kind: "value_change".into(),
                    description: format!("{} -> {}", self.value, other.value),
                    magnitude: (other.value - self.value) as f64,
                }]
            } else {
                vec![]
            }
        }
    }

    #[test]
    fn test_solve_simple() {
        let world = TestWorld { value: 0, goal_value: 1 };
        let goal = TestWorld { value: 1, goal_value: 1 };
        let mut graph = CausalGraph::new();
        let budget = SimulationBudget {
            max_total_steps: 1000,
            max_branch_depth: 5,
            max_active_branches: 16,
            prune_threshold: 0.0,
            time_limit: None,
        };

        let result = solve(&world, &goal, &mut graph, &budget);
        assert!(result.solved, "Should solve value=0 -> value=1 via increment");
        assert!(!result.program.is_empty());
    }
}
