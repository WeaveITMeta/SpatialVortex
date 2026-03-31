//! Undo-safe state-graph exploration for ARC-AGI-3.
//!
//! Uses action 6 (undo) to safely probe the action space without permanent
//! cost. Builds a directed graph of reachable states and characterizes which
//! actions are reversible. Feeds into the MCTS and goal-directed planning
//! layers.
//!
//! Exploration protocol:
//! 1. From current state S, pick an untried action A (not submit=7, not undo=6).
//! 2. Execute A (costs 1 step). Observe new state S'.
//! 3. Execute undo (action 6, costs 1 step). Observe state S''.
//! 4. If S'' == S: A is reversible. Record transition S -A-> S'.
//! 5. If S'' != S: A may be irreversible. Record both transitions.
//!
//! Each explore-undo pair costs 2 API steps. With 5 actions (1-5), full
//! characterization from one state costs 10 steps.

use std::collections::{HashMap, HashSet, VecDeque};
use eustress_vortex_grid2d::Grid2D;

// ─── Grid Hash ───────────────────────────────────────────────────────────────

/// FNV-1a hash of a Grid2D. Matches the hashing approach in arc_trajectory.rs
/// but is self-contained (no dependency on aimodel crate).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridHash(pub u64);

impl GridHash {
    pub fn from_grid(grid: &Grid2D) -> Self {
        const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
        const FNV_PRIME: u64 = 0x0100_0000_01b3;

        let mut h = FNV_OFFSET;
        // Include dimensions for collision resistance.
        for &b in &(grid.height as u64).to_le_bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(FNV_PRIME);
        }
        for &b in &(grid.width as u64).to_le_bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(FNV_PRIME);
        }
        for row in &grid.cells {
            for &cell in row {
                h ^= cell as u64;
                h = h.wrapping_mul(FNV_PRIME);
            }
        }
        Self(h)
    }
}

// ─── State Graph ─────────────────────────────────────────────────────────────

/// Edge in the state transition graph.
#[derive(Clone, Debug)]
pub struct StateEdge {
    pub action_id: u32,
    pub target: GridHash,
    pub is_reversible: bool,
}

/// Lightweight directed graph of grid states and action transitions.
#[derive(Clone, Debug, Default)]
pub struct ExplorationGraph {
    /// Forward edges: source_hash → Vec<edge>.
    pub edges: HashMap<GridHash, Vec<StateEdge>>,
    /// Known states.
    pub nodes: HashSet<GridHash>,
    /// Visit counts per state.
    pub visit_counts: HashMap<GridHash, u32>,
}

impl ExplorationGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a state, return whether it was already known.
    pub fn register_state(&mut self, hash: GridHash) -> bool {
        let existed = self.nodes.contains(&hash);
        self.nodes.insert(hash);
        *self.visit_counts.entry(hash).or_insert(0) += 1;
        existed
    }

    /// Add a transition edge.
    pub fn add_edge(&mut self, from: GridHash, action_id: u32, to: GridHash, reversible: bool) {
        self.nodes.insert(from);
        self.nodes.insert(to);
        self.edges.entry(from).or_default().push(StateEdge {
            action_id,
            target: to,
            is_reversible: reversible,
        });
    }

    /// Which actions have been tried from a given state?
    pub fn actions_tried_from(&self, hash: &GridHash) -> HashSet<u32> {
        self.edges
            .get(hash)
            .map(|edges| edges.iter().map(|e| e.action_id).collect())
            .unwrap_or_default()
    }

    /// Number of distinct states discovered.
    pub fn state_count(&self) -> usize {
        self.nodes.len()
    }

    /// Find all states reachable from `start` within `max_depth` steps.
    pub fn reachable_from(&self, start: GridHash, max_depth: usize) -> Vec<(GridHash, usize)> {
        let mut visited: HashMap<GridHash, usize> = HashMap::new();
        let mut queue = VecDeque::new();
        queue.push_back((start, 0usize));
        visited.insert(start, 0);

        while let Some((node, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }
            if let Some(neighbors) = self.edges.get(&node) {
                for edge in neighbors {
                    if !visited.contains_key(&edge.target) {
                        visited.insert(edge.target, depth + 1);
                        queue.push_back((edge.target, depth + 1));
                    }
                }
            }
        }
        visited.into_iter().collect()
    }

    pub fn clear(&mut self) {
        self.edges.clear();
        self.nodes.clear();
        self.visit_counts.clear();
    }
}

// ─── Exploration Budget ──────────────────────────────────────────────────────

/// Configuration for exploration budget.
#[derive(Clone, Debug)]
pub struct ExplorationBudget {
    /// Maximum steps to spend on exploration (action + undo pairs count as 2).
    pub max_steps: u32,
    /// Re-explore if average IRA drops below this threshold mid-game.
    pub ira_re_explore_threshold: f32,
    /// Never explore after this many total game steps.
    pub hard_cutoff_step: u32,
}

impl Default for ExplorationBudget {
    fn default() -> Self {
        Self {
            max_steps: 12,
            ira_re_explore_threshold: 0.3,
            hard_cutoff_step: 40,
        }
    }
}

// ─── Undo Explorer ───────────────────────────────────────────────────────────

/// Phase of a single explore-undo probe.
#[derive(Clone, Debug, PartialEq, Eq)]
enum ProbePhase {
    /// Ready to try an action.
    ReadyToProbe,
    /// Action was taken, waiting for undo to verify reversibility.
    AwaitingUndo {
        action_tried: u32,
        pre_action_hash: GridHash,
        post_action_hash: GridHash,
    },
}

/// Undo-safe explorer. Manages the explore-undo protocol.
pub struct UndoExplorer {
    budget: ExplorationBudget,
    /// Actions still to explore from the current state.
    action_queue: VecDeque<u32>,
    /// Current probe state.
    phase: ProbePhase,
    /// Actions confirmed as reversible.
    pub reversible_actions: HashSet<u32>,
    /// Actions that may not be reversible.
    pub irreversible_actions: HashSet<u32>,
    /// Steps consumed by exploration.
    steps_used: u32,
    /// The state graph being built.
    pub graph: ExplorationGraph,
    /// Whether exploration is complete (budget exhausted or all actions tried).
    pub finished: bool,
}

impl UndoExplorer {
    pub fn new(budget: ExplorationBudget) -> Self {
        Self {
            budget,
            action_queue: VecDeque::new(),
            phase: ProbePhase::ReadyToProbe,
            reversible_actions: HashSet::new(),
            irreversible_actions: HashSet::new(),
            steps_used: 0,
            graph: ExplorationGraph::new(),
            finished: false,
        }
    }

    pub fn with_default_budget() -> Self {
        Self::new(ExplorationBudget::default())
    }

    /// Reset for a new game/level, preserving discovered reversibility.
    pub fn reset_for_level(&mut self) {
        self.action_queue.clear();
        self.phase = ProbePhase::ReadyToProbe;
        self.steps_used = 0;
        self.finished = false;
        // Keep reversible/irreversible knowledge and graph across levels.
    }

    /// Full reset (new game).
    pub fn reset(&mut self) {
        self.action_queue.clear();
        self.phase = ProbePhase::ReadyToProbe;
        self.reversible_actions.clear();
        self.irreversible_actions.clear();
        self.steps_used = 0;
        self.graph.clear();
        self.finished = false;
    }

    /// Steps consumed so far.
    pub fn steps_used(&self) -> u32 {
        self.steps_used
    }

    /// Should we be exploring right now?
    pub fn should_explore(&self, current_game_step: u32) -> bool {
        !self.finished
            && self.steps_used < self.budget.max_steps
            && current_game_step < self.budget.hard_cutoff_step
    }

    /// Get the next exploration action to execute, or None if done.
    ///
    /// Call this from the DECIDE phase. If it returns Some(action), execute
    /// that action via the API and then call `observe_result()` with the
    /// resulting grid.
    pub fn next_action(
        &mut self,
        available_actions: &[u32],
        current_grid: &Grid2D,
    ) -> Option<u32> {
        if self.finished || self.steps_used >= self.budget.max_steps {
            self.finished = true;
            return None;
        }

        let current_hash = GridHash::from_grid(current_grid);
        self.graph.register_state(current_hash);

        match &self.phase {
            ProbePhase::ReadyToProbe => {
                // Populate action queue if empty.
                if self.action_queue.is_empty() {
                    let tried = self.graph.actions_tried_from(&current_hash);
                    for &a in available_actions {
                        // Skip undo (6) and submit (7) — we only explore game actions.
                        if a == 6 || a == 7 {
                            continue;
                        }
                        if !tried.contains(&a) {
                            self.action_queue.push_back(a);
                        }
                    }
                }

                // Pick next untried action.
                if let Some(action) = self.action_queue.pop_front() {
                    self.phase = ProbePhase::AwaitingUndo {
                        action_tried: action,
                        pre_action_hash: current_hash,
                        post_action_hash: GridHash(0), // filled in observe_result
                    };
                    self.steps_used += 1;
                    Some(action)
                } else {
                    // All actions tried from this state.
                    self.finished = true;
                    None
                }
            }
            ProbePhase::AwaitingUndo { .. } => {
                // We need to undo. Check if undo is available.
                if available_actions.contains(&6) {
                    self.steps_used += 1;
                    Some(6) // undo
                } else {
                    // Can't undo — mark phase complete, move on.
                    self.phase = ProbePhase::ReadyToProbe;
                    None
                }
            }
        }
    }

    /// Call after each step to update the explorer with the resulting grid.
    pub fn observe_result(&mut self, result_grid: &Grid2D, action_taken: u32) {
        let result_hash = GridHash::from_grid(result_grid);
        self.graph.register_state(result_hash);

        match &self.phase {
            ProbePhase::AwaitingUndo {
                action_tried,
                pre_action_hash,
                ..
            } if action_taken != 6 => {
                // We just executed the probe action. Record the transition.
                // Update phase with the post-action hash.
                let pre = *pre_action_hash;
                let action = *action_tried;
                self.graph.add_edge(pre, action, result_hash, false); // reversibility TBD
                self.phase = ProbePhase::AwaitingUndo {
                    action_tried: action,
                    pre_action_hash: pre,
                    post_action_hash: result_hash,
                };
            }
            ProbePhase::AwaitingUndo {
                action_tried,
                pre_action_hash,
                post_action_hash,
            } if action_taken == 6 => {
                // We just executed undo. Check if we returned to pre-action state.
                let action = *action_tried;
                let pre = *pre_action_hash;
                let post = *post_action_hash;

                if result_hash == pre {
                    // Undo worked — action is reversible.
                    self.reversible_actions.insert(action);
                    // Update the edge to mark as reversible.
                    if let Some(edges) = self.graph.edges.get_mut(&pre) {
                        for edge in edges.iter_mut() {
                            if edge.action_id == action && edge.target == post {
                                edge.is_reversible = true;
                            }
                        }
                    }
                    // Also record the undo edge.
                    self.graph.add_edge(post, 6, pre, true);
                } else {
                    // Undo didn't return to original state.
                    self.irreversible_actions.insert(action);
                    self.graph.add_edge(post, 6, result_hash, false);
                }

                // Ready for next probe.
                self.phase = ProbePhase::ReadyToProbe;
            }
            _ => {
                // Not in a probe — just record the state.
            }
        }
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn grid(cells: Vec<Vec<u8>>) -> Grid2D {
        Grid2D::new(cells)
    }

    #[test]
    fn test_grid_hash_deterministic() {
        let g = grid(vec![vec![0, 1, 2], vec![3, 4, 5]]);
        let h1 = GridHash::from_grid(&g);
        let h2 = GridHash::from_grid(&g);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_grid_hash_different_grids() {
        let g1 = grid(vec![vec![0, 1], vec![2, 3]]);
        let g2 = grid(vec![vec![0, 1], vec![2, 4]]);
        assert_ne!(GridHash::from_grid(&g1), GridHash::from_grid(&g2));
    }

    #[test]
    fn test_grid_hash_dimensions_matter() {
        // Same cells but different shapes
        let g1 = grid(vec![vec![1, 2, 3]]);
        let g2 = grid(vec![vec![1], vec![2], vec![3]]);
        assert_ne!(GridHash::from_grid(&g1), GridHash::from_grid(&g2));
    }

    #[test]
    fn test_exploration_graph_basics() {
        let mut graph = ExplorationGraph::new();
        let h1 = GridHash(100);
        let h2 = GridHash(200);

        let existed = graph.register_state(h1);
        assert!(!existed);
        let existed = graph.register_state(h1);
        assert!(existed);

        graph.add_edge(h1, 1, h2, true);
        let tried = graph.actions_tried_from(&h1);
        assert!(tried.contains(&1));
        assert!(!tried.contains(&2));
        assert_eq!(graph.state_count(), 2);
    }

    #[test]
    fn test_explorer_first_probe() {
        let mut explorer = UndoExplorer::with_default_budget();
        let g = grid(vec![vec![0, 0], vec![0, 0]]);
        let available = vec![1, 2, 3, 4, 5, 6, 7];

        assert!(explorer.should_explore(0));

        let action = explorer.next_action(&available, &g);
        assert!(action.is_some());
        let a = action.unwrap();
        assert!(a >= 1 && a <= 5); // Should not pick 6 (undo) or 7 (submit)
    }

    #[test]
    fn test_explorer_undo_cycle() {
        let mut explorer = UndoExplorer::with_default_budget();

        let g_start = grid(vec![vec![0, 0], vec![0, 0]]);
        let g_after = grid(vec![vec![1, 0], vec![0, 0]]);
        let available = vec![1, 2, 3, 4, 5, 6, 7];

        // Step 1: Get probe action
        let action = explorer.next_action(&available, &g_start).unwrap();
        assert_ne!(action, 6);
        assert_ne!(action, 7);

        // Observe result of probe action
        explorer.observe_result(&g_after, action);

        // Step 2: Should request undo
        let undo = explorer.next_action(&available, &g_after);
        assert_eq!(undo, Some(6));

        // Observe result of undo — returns to start
        explorer.observe_result(&g_start, 6);

        // Action should be marked reversible
        assert!(explorer.reversible_actions.contains(&action));
        assert_eq!(explorer.steps_used(), 2);
    }

    #[test]
    fn test_explorer_irreversible_action() {
        let mut explorer = UndoExplorer::with_default_budget();

        let g_start = grid(vec![vec![0, 0], vec![0, 0]]);
        let g_after = grid(vec![vec![1, 0], vec![0, 0]]);
        let g_after_undo = grid(vec![vec![2, 0], vec![0, 0]]); // undo didn't restore!
        let available = vec![1, 2, 3, 4, 5, 6, 7];

        let action = explorer.next_action(&available, &g_start).unwrap();
        explorer.observe_result(&g_after, action);

        let undo = explorer.next_action(&available, &g_after).unwrap();
        assert_eq!(undo, 6);
        explorer.observe_result(&g_after_undo, 6);

        // Action should be marked irreversible
        assert!(explorer.irreversible_actions.contains(&action));
    }

    #[test]
    fn test_explorer_budget_exhaustion() {
        let budget = ExplorationBudget {
            max_steps: 4, // Only 2 probe-undo pairs
            ira_re_explore_threshold: 0.3,
            hard_cutoff_step: 100,
        };
        let mut explorer = UndoExplorer::new(budget);

        let g = grid(vec![vec![0]]);
        let available = vec![1, 2, 3, 6, 7];

        // Burn through budget
        for _ in 0..4 {
            if let Some(a) = explorer.next_action(&available, &g) {
                explorer.observe_result(&g, a);
            }
        }

        // Budget should be exhausted
        assert!(!explorer.should_explore(0));
    }

    #[test]
    fn test_explorer_hard_cutoff() {
        let explorer = UndoExplorer::with_default_budget();
        // Past hard cutoff
        assert!(!explorer.should_explore(100));
    }

    #[test]
    fn test_reachable_from() {
        let mut graph = ExplorationGraph::new();
        let h1 = GridHash(1);
        let h2 = GridHash(2);
        let h3 = GridHash(3);
        let h4 = GridHash(4);

        graph.add_edge(h1, 1, h2, true);
        graph.add_edge(h2, 2, h3, true);
        graph.add_edge(h3, 3, h4, true);

        let reachable = graph.reachable_from(h1, 2);
        let hashes: HashSet<GridHash> = reachable.iter().map(|(h, _)| *h).collect();
        assert!(hashes.contains(&h1));
        assert!(hashes.contains(&h2));
        assert!(hashes.contains(&h3));
        assert!(!hashes.contains(&h4)); // depth 3, beyond max_depth 2
    }
}
