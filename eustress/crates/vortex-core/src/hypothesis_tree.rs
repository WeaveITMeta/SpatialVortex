//! HypothesisTree — parallel simulation branch management.
//!
//! A tree of simulation branches where each branch tests a hypothesis.
//! "Quantum" in the sense that multiple branches exist simultaneously
//! until observation (scoring) collapses them.

use crate::causal_graph::Hypothesis;
use crate::types::*;
use crate::world_state::WorldState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, info};

/// Prevents runaway simulation. Essential for real-time 3D scenarios.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SimulationBudget {
    /// Max total simulation steps across all branches.
    pub max_total_steps: usize,
    /// Max depth per branch (program length).
    pub max_branch_depth: usize,
    /// Max concurrent active branches.
    pub max_active_branches: usize,
    /// Minimum score to avoid pruning.
    pub prune_threshold: f32,
    /// Time limit (for real-time 3D, this matters).
    #[serde(skip)]
    pub time_limit: Option<Duration>,
}

impl Default for SimulationBudget {
    fn default() -> Self {
        Self {
            max_total_steps: 10_000,
            max_branch_depth: 20,
            max_active_branches: 32,
            prune_threshold: 0.05,
            time_limit: Some(Duration::from_secs(30)),
        }
    }
}

impl SimulationBudget {
    /// Conservative budget for ARC (fast, exact match).
    pub fn arc() -> Self {
        Self {
            max_total_steps: 5_000,
            max_branch_depth: 15,
            max_active_branches: 64,
            prune_threshold: 0.1,
            time_limit: Some(Duration::from_secs(10)),
        }
    }

    /// Generous budget for 3D workshop scenarios.
    pub fn workshop() -> Self {
        Self {
            max_total_steps: 50_000,
            max_branch_depth: 50,
            max_active_branches: 16,
            prune_threshold: 0.01,
            time_limit: Some(Duration::from_secs(120)),
        }
    }
}

/// State of a simulation branch.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum BranchState {
    /// Waiting to be simulated.
    Pending,
    /// Currently executing in a sandbox.
    Simulating,
    /// Simulation complete, awaiting scoring.
    AwaitingScore,
    /// Scored — either succeeded or failed.
    Scored { success: bool },
    /// Branched into children (partial success, exploring refinements).
    Branched,
    /// Pruned (score too low to continue).
    Pruned,
}

/// A node in the hypothesis tree.
#[derive(Clone, Debug)]
pub struct HypothesisNode {
    pub id: BranchId,
    pub parent: Option<BranchId>,
    pub hypothesis: Hypothesis,
    pub state: BranchState,
    pub score: Option<Score>,
    pub depth: usize,
    pub children: Vec<BranchId>,
    /// Intermediate states captured during simulation.
    pub trajectory: Vec<DSLOp>,
}

/// A completed branch with its full results.
#[derive(Clone, Debug)]
pub struct CompletedBranch {
    pub id: BranchId,
    pub hypothesis: Hypothesis,
    pub score: Score,
    pub trajectory: Vec<DSLOp>,
    pub success: bool,
}

/// The tree of simulation branches.
///
/// Each branch tests a hypothesis. Branches can be scored, pruned,
/// or refined into child branches. The tree tracks budget and provides
/// the best completed branch when the search terminates.
pub struct HypothesisTree {
    nodes: HashMap<BranchId, HypothesisNode>,
    /// Branches ready to simulate (FIFO queue, priority by confidence).
    pending: Vec<BranchId>,
    completed: Vec<CompletedBranch>,
    budget: SimulationBudget,
    total_steps: usize,
    start_time: Instant,
}

impl HypothesisTree {
    pub fn new(budget: SimulationBudget) -> Self {
        Self {
            nodes: HashMap::new(),
            pending: Vec::new(),
            completed: Vec::new(),
            budget,
            total_steps: 0,
            start_time: Instant::now(),
        }
    }

    /// Add a new root-level branch from a hypothesis.
    pub fn add_branch(&mut self, hypothesis: Hypothesis) -> BranchId {
        let id = BranchId::new();
        let node = HypothesisNode {
            id,
            parent: None,
            hypothesis,
            state: BranchState::Pending,
            score: None,
            depth: 0,
            children: vec![],
            trajectory: vec![],
        };
        self.nodes.insert(id, node);
        self.pending.push(id);
        id
    }

    /// Add a child branch (refinement of an existing branch).
    pub fn add_child_branch(&mut self, parent: BranchId, hypothesis: Hypothesis) -> BranchId {
        let parent_depth = self.nodes.get(&parent).map_or(0, |n| n.depth);
        let id = BranchId::new();
        let node = HypothesisNode {
            id,
            parent: Some(parent),
            hypothesis,
            state: BranchState::Pending,
            score: None,
            depth: parent_depth + 1,
            children: vec![],
            trajectory: vec![],
        };
        self.nodes.insert(id, node);
        self.pending.push(id);

        if let Some(parent_node) = self.nodes.get_mut(&parent) {
            parent_node.children.push(id);
            parent_node.state = BranchState::Branched;
        }

        id
    }

    /// Get the next branch to simulate.
    /// Returns None if no branches remain or budget exhausted.
    pub fn next_branch(&mut self) -> Option<BranchId> {
        if self.budget_exhausted() {
            return None;
        }

        // Sort pending by prior confidence (highest first)
        self.pending.sort_by(|a, b| {
            let ca = self.nodes.get(a).map_or(0.0, |n| n.hypothesis.prior_confidence);
            let cb = self.nodes.get(b).map_or(0.0, |n| n.hypothesis.prior_confidence);
            cb.partial_cmp(&ca).unwrap_or(std::cmp::Ordering::Equal)
        });

        while let Some(id) = self.pending.pop() {
            if let Some(node) = self.nodes.get_mut(&id) {
                if node.state == BranchState::Pending {
                    node.state = BranchState::Simulating;
                    return Some(id);
                }
            }
        }

        None
    }

    /// Simulate a branch against a world state.
    /// Executes the hypothesis program step by step, scoring after each op.
    pub fn simulate_branch<W: WorldState>(
        &mut self,
        branch_id: BranchId,
        initial_state: &W,
        goal_state: &W,
    ) -> Option<Score> {
        let program = {
            let node = self.nodes.get(&branch_id)?;
            node.hypothesis.program.clone()
        };

        let mut current = initial_state.clone();
        let mut best_score = current.score_against(goal_state);
        let mut trajectory = Vec::new();

        for (i, op) in program.iter().enumerate() {
            if self.budget_exhausted() {
                debug!("Budget exhausted at step {} of branch {}", i, branch_id);
                break;
            }

            current = current.apply(op);
            self.total_steps += 1;
            trajectory.push(op.clone());

            let score = current.score_against(goal_state);
            if score.accuracy > best_score.accuracy {
                best_score = score.clone();
            }

            // Early termination on perfect match
            if score.exact_match {
                info!("Branch {} found exact match at step {}", branch_id, i);
                best_score = score;
                break;
            }
        }

        // Update node
        if let Some(node) = self.nodes.get_mut(&branch_id) {
            node.trajectory = trajectory;
            node.score = Some(best_score.clone());
            node.state = BranchState::AwaitingScore;
        }

        Some(best_score)
    }

    /// Score a branch and decide: commit, prune, or branch.
    pub fn score_branch(&mut self, branch_id: BranchId, score: Score) {
        let success = score.exact_match;
        let should_prune = score.accuracy < self.budget.prune_threshold as f64;

        if let Some(node) = self.nodes.get_mut(&branch_id) {
            node.score = Some(score.clone());

            if success || should_prune {
                node.state = if success {
                    BranchState::Scored { success: true }
                } else {
                    BranchState::Pruned
                };

                self.completed.push(CompletedBranch {
                    id: branch_id,
                    hypothesis: node.hypothesis.clone(),
                    score,
                    trajectory: node.trajectory.clone(),
                    success,
                });
            } else {
                // Partial success — leave for refinement
                node.state = BranchState::Scored { success: false };
                self.completed.push(CompletedBranch {
                    id: branch_id,
                    hypothesis: node.hypothesis.clone(),
                    score,
                    trajectory: node.trajectory.clone(),
                    success: false,
                });
            }
        }
    }

    /// Check if there are active (pending or simulating) branches.
    pub fn has_active_branches(&self) -> bool {
        !self.pending.is_empty()
            || self.nodes.values().any(|n| n.state == BranchState::Simulating)
    }

    /// Check if simulation budget is exhausted.
    pub fn budget_exhausted(&self) -> bool {
        if self.total_steps >= self.budget.max_total_steps {
            return true;
        }
        if let Some(limit) = self.budget.time_limit {
            if self.start_time.elapsed() > limit {
                return true;
            }
        }
        // Check if we already found a perfect solution
        if self.completed.iter().any(|c| c.success) {
            return true;
        }
        false
    }

    /// Get the best completed branch (by score).
    pub fn best_completed_branch(&self) -> Option<&CompletedBranch> {
        self.completed.iter()
            .max_by(|a, b| {
                // Prefer success, then highest accuracy
                match (a.success, b.success) {
                    (true, false) => std::cmp::Ordering::Greater,
                    (false, true) => std::cmp::Ordering::Less,
                    _ => a.score.accuracy.partial_cmp(&b.score.accuracy)
                        .unwrap_or(std::cmp::Ordering::Equal),
                }
            })
    }

    /// Get all completed branches, sorted by score descending.
    pub fn all_completed(&self) -> Vec<&CompletedBranch> {
        let mut branches: Vec<_> = self.completed.iter().collect();
        branches.sort_by(|a, b| {
            b.score.accuracy.partial_cmp(&a.score.accuracy)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        branches
    }

    /// Statistics about the tree.
    pub fn stats(&self) -> TreeStats {
        TreeStats {
            total_branches: self.nodes.len(),
            pending: self.pending.len(),
            completed: self.completed.len(),
            successful: self.completed.iter().filter(|c| c.success).count(),
            pruned: self.nodes.values().filter(|n| n.state == BranchState::Pruned).count(),
            total_steps: self.total_steps,
            elapsed: self.start_time.elapsed(),
            best_score: self.best_completed_branch().map(|b| b.score.accuracy).unwrap_or(0.0),
        }
    }
}

/// Summary statistics for the hypothesis tree.
#[derive(Clone, Debug)]
pub struct TreeStats {
    pub total_branches: usize,
    pub pending: usize,
    pub completed: usize,
    pub successful: usize,
    pub pruned: usize,
    pub total_steps: usize,
    pub elapsed: Duration,
    pub best_score: f64,
}

impl std::fmt::Display for TreeStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "branches={} completed={} success={} pruned={} steps={} best={:.3} elapsed={:.1}s",
            self.total_branches, self.completed, self.successful,
            self.pruned, self.total_steps, self.best_score,
            self.elapsed.as_secs_f64()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_defaults() {
        let b = SimulationBudget::default();
        assert_eq!(b.max_total_steps, 10_000);
        assert_eq!(b.max_active_branches, 32);
    }

    #[test]
    fn test_add_and_retrieve_branch() {
        let mut tree = HypothesisTree::new(SimulationBudget::default());
        let h = Hypothesis {
            description: "test".into(),
            program: vec![],
            source_law: None,
            prior_confidence: 0.5,
            motivating_properties: vec![],
        };
        let id = tree.add_branch(h);
        assert!(tree.has_active_branches());
        let next = tree.next_branch();
        assert_eq!(next, Some(id));
    }

    #[test]
    fn test_budget_exhausted_on_success() {
        let mut tree = HypothesisTree::new(SimulationBudget::default());
        let h = Hypothesis {
            description: "test".into(),
            program: vec![],
            source_law: None,
            prior_confidence: 0.5,
            motivating_properties: vec![],
        };
        let id = tree.add_branch(h);
        tree.next_branch();
        tree.score_branch(id, Score::perfect());
        assert!(tree.budget_exhausted());
    }
}
