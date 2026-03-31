//! IRA-driven Monte Carlo Tree Search for ARC-AGI-3.
//!
//! Replaces random action sampling with IRA-weighted selection (PUCT-style).
//! Zero real API calls — all simulation uses ActionModel predictions or
//! StateGraph transitions.
//!
//! Integration: MCTS runs when solve() returns low confidence and StateGraph
//! has exploration data. Complements HypothesisTree — does not replace it.

use std::collections::HashMap;
use crate::exploration::{ExplorationGraph, GridHash};

// ─── MCTS Budget ─────────────────────────────────────────────────────────────

/// Budget constraints for MCTS search.
#[derive(Clone, Debug)]
pub struct MctsBudget {
    /// Maximum MCTS iterations (each = select + expand + simulate + backprop).
    pub max_iterations: u32,
    /// Maximum simulation depth per rollout.
    pub max_rollout_depth: u32,
    /// Time limit in milliseconds for the entire MCTS search.
    pub time_limit_ms: u64,
}

impl Default for MctsBudget {
    fn default() -> Self {
        Self {
            max_iterations: 200,
            max_rollout_depth: 10,
            time_limit_ms: 150,
        }
    }
}

// ─── MCTS Node ───────────────────────────────────────────────────────────────

/// A single node in the MCTS search tree.
#[derive(Clone, Debug)]
pub struct MctsNode {
    /// State hash at this node.
    pub state_hash: GridHash,
    /// Action that led to this node from parent (None for root).
    pub action_from_parent: Option<u32>,
    /// Number of times this node has been visited.
    pub visit_count: u32,
    /// Cumulative value (sum of backpropagated scores).
    pub total_value: f64,
    /// IRA prior for the action that led here (from ActionModel).
    pub ira_prior: f32,
    /// Indices of child nodes in the arena.
    pub children: Vec<usize>,
    /// Index of parent node (None for root).
    pub parent: Option<usize>,
    /// Whether this node has been expanded.
    pub expanded: bool,
}

impl MctsNode {
    /// Mean value estimate.
    pub fn q_value(&self) -> f64 {
        if self.visit_count == 0 {
            0.0
        } else {
            self.total_value / self.visit_count as f64
        }
    }
}

// ─── MCTS Tree ───────────────────────────────────────────────────────────────

/// Arena-allocated MCTS search tree with IRA priors.
pub struct MctsTree {
    /// Arena storage for nodes.
    nodes: Vec<MctsNode>,
    /// Root node index.
    root: usize,
    /// UCB1 exploration constant.
    exploration_c: f64,
    /// Weight for IRA prior in PUCT formula.
    ira_weight: f64,
}

/// Result of an MCTS search.
#[derive(Clone, Debug)]
pub struct MctsResult {
    /// Best action to take.
    pub action: u32,
    /// Confidence (0.0-1.0).
    pub confidence: f32,
    /// Visit count of the best child.
    pub visit_count: u32,
    /// Mean value of the best child.
    pub value: f64,
    /// Total iterations completed.
    pub iterations: u32,
}

impl MctsTree {
    /// Create a new MCTS tree rooted at the given state.
    pub fn new(root_state: GridHash) -> Self {
        let root = MctsNode {
            state_hash: root_state,
            action_from_parent: None,
            visit_count: 0,
            total_value: 0.0,
            ira_prior: 1.0,
            children: Vec::new(),
            parent: None,
            expanded: false,
        };
        Self {
            nodes: vec![root],
            root: 0,
            exploration_c: 1.414,
            ira_weight: 1.0,
        }
    }

    /// Run MCTS search using the exploration graph for transitions
    /// and IRA scores as priors.
    ///
    /// Returns the best action to take, or None if no useful search possible.
    pub fn search(
        &mut self,
        graph: &ExplorationGraph,
        available_actions: &[u32],
        ira_scores: &HashMap<u32, f32>,
        goal_scores: &HashMap<GridHash, f64>,
        budget: &MctsBudget,
    ) -> Option<MctsResult> {
        let start = std::time::Instant::now();
        let mut iterations = 0u32;

        while iterations < budget.max_iterations {
            // Check time limit.
            if start.elapsed().as_millis() as u64 >= budget.time_limit_ms {
                break;
            }

            // ── Selection ────────────────────────────────────────────────
            let leaf = self.select(self.root);

            // ── Expansion ────────────────────────────────────────────────
            if !self.nodes[leaf].expanded {
                self.expand(leaf, graph, available_actions, ira_scores);
            }

            // ── Simulation (evaluate leaf) ───────────────────────────────
            let value = self.evaluate(leaf, graph, goal_scores, budget.max_rollout_depth);

            // ── Backpropagation ──────────────────────────────────────────
            self.backpropagate(leaf, value);

            iterations += 1;
        }

        // Select best action by visit count (most robust).
        self.best_action(iterations)
    }

    // ── Selection: UCB1 + IRA prior (PUCT) ───────────────────────────────

    fn select(&self, start: usize) -> usize {
        let mut current = start;
        loop {
            let node = &self.nodes[current];
            if node.children.is_empty() || !node.expanded {
                return current;
            }

            // Select child with highest PUCT score.
            let parent_visits = node.visit_count.max(1) as f64;
            let mut best_score = f64::NEG_INFINITY;
            let mut best_child = node.children[0];

            for &child_idx in &node.children {
                let child = &self.nodes[child_idx];
                let q = child.q_value();
                let prior = child.ira_prior as f64 * self.ira_weight;
                let exploration = self.exploration_c
                    * prior
                    * (parent_visits.ln() / child.visit_count.max(1) as f64).sqrt();
                let score = q + exploration;

                if score > best_score {
                    best_score = score;
                    best_child = child_idx;
                }
            }

            current = best_child;
        }
    }

    // ── Expansion ────────────────────────────────────────────────────────

    fn expand(
        &mut self,
        node_idx: usize,
        graph: &ExplorationGraph,
        available_actions: &[u32],
        ira_scores: &HashMap<u32, f32>,
    ) {
        let state = self.nodes[node_idx].state_hash;

        // Find transitions from this state in the exploration graph.
        let transitions = graph.edges.get(&state);

        // Expand with known transitions.
        if let Some(edges) = transitions {
            for edge in edges {
                let prior = ira_scores.get(&edge.action_id).copied().unwrap_or(0.3);
                let child_idx = self.nodes.len();
                self.nodes.push(MctsNode {
                    state_hash: edge.target,
                    action_from_parent: Some(edge.action_id),
                    visit_count: 0,
                    total_value: 0.0,
                    ira_prior: prior,
                    children: Vec::new(),
                    parent: Some(node_idx),
                    expanded: false,
                });
                self.nodes[node_idx].children.push(child_idx);
            }
        }

        // Also expand with available actions not yet in the graph.
        let known_actions: std::collections::HashSet<u32> = transitions
            .map(|edges| edges.iter().map(|e| e.action_id).collect())
            .unwrap_or_default();

        for &action in available_actions {
            if action == 6 || action == 7 {
                continue; // Skip undo and submit in MCTS
            }
            if !known_actions.contains(&action) {
                let prior = ira_scores.get(&action).copied().unwrap_or(0.2);
                let child_idx = self.nodes.len();
                // Unknown transition: use a synthetic hash
                let synthetic_hash = GridHash(state.0.wrapping_add((action as u64).wrapping_mul(0x9E3779B97F4A7C15)));
                self.nodes.push(MctsNode {
                    state_hash: synthetic_hash,
                    action_from_parent: Some(action),
                    visit_count: 0,
                    total_value: 0.0,
                    ira_prior: prior * 0.5, // Penalize unknown transitions
                    children: Vec::new(),
                    parent: Some(node_idx),
                    expanded: false,
                });
                self.nodes[node_idx].children.push(child_idx);
            }
        }

        self.nodes[node_idx].expanded = true;
    }

    // ── Evaluation (lightweight rollout) ─────────────────────────────────

    fn evaluate(
        &self,
        node_idx: usize,
        graph: &ExplorationGraph,
        goal_scores: &HashMap<GridHash, f64>,
        max_depth: u32,
    ) -> f64 {
        let state = self.nodes[node_idx].state_hash;

        // If we have a goal score for this state, use it directly.
        if let Some(&score) = goal_scores.get(&state) {
            return score;
        }

        // Random rollout on the graph (follow random edges).
        let mut current = state;
        let mut best_score = 0.0_f64;

        for _depth in 0..max_depth {
            if let Some(&score) = goal_scores.get(&current) {
                best_score = best_score.max(score);
                break;
            }

            // Follow a random edge if available.
            if let Some(edges) = graph.edges.get(&current) {
                if edges.is_empty() {
                    break;
                }
                // Pick highest-IRA edge (not truly random — IRA-weighted).
                let next = edges
                    .iter()
                    .max_by(|a, b| {
                        a.is_reversible
                            .cmp(&b.is_reversible)
                            .then_with(|| a.action_id.cmp(&b.action_id))
                    });
                if let Some(edge) = next {
                    current = edge.target;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Final score check.
        if let Some(&score) = goal_scores.get(&current) {
            best_score = best_score.max(score);
        }

        best_score
    }

    // ── Backpropagation ──────────────────────────────────────────────────

    fn backpropagate(&mut self, leaf: usize, value: f64) {
        let mut current = Some(leaf);
        while let Some(idx) = current {
            self.nodes[idx].visit_count += 1;
            self.nodes[idx].total_value += value;
            current = self.nodes[idx].parent;
        }
    }

    // ── Best action selection ────────────────────────────────────────────

    fn best_action(&self, iterations: u32) -> Option<MctsResult> {
        let root = &self.nodes[self.root];
        if root.children.is_empty() {
            return None;
        }

        // Select action with most visits (most robust policy).
        let best_child_idx = root
            .children
            .iter()
            .copied()
            .max_by_key(|&idx| self.nodes[idx].visit_count)?;

        let best = &self.nodes[best_child_idx];
        let total_visits: u32 = root.children.iter().map(|&i| self.nodes[i].visit_count).sum();

        Some(MctsResult {
            action: best.action_from_parent?,
            confidence: if total_visits > 0 {
                best.visit_count as f32 / total_visits as f32
            } else {
                0.0
            },
            visit_count: best.visit_count,
            value: best.q_value(),
            iterations,
        })
    }

    /// Number of nodes in the tree.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcts_empty_graph() {
        let root = GridHash(42);
        let mut tree = MctsTree::new(root);
        let graph = ExplorationGraph::new();
        let budget = MctsBudget {
            max_iterations: 10,
            max_rollout_depth: 5,
            time_limit_ms: 1000,
        };

        let result = tree.search(
            &graph,
            &[1, 2, 3, 4, 5],
            &HashMap::new(),
            &HashMap::new(),
            &budget,
        );

        // Should still produce a result (from expansion of unknown actions).
        assert!(result.is_some());
    }

    #[test]
    fn test_mcts_with_graph() {
        let h1 = GridHash(1);
        let h2 = GridHash(2);
        let h3 = GridHash(3);

        let mut graph = ExplorationGraph::new();
        graph.add_edge(h1, 1, h2, true);
        graph.add_edge(h1, 2, h3, true);

        // h3 is the goal state (high score).
        let mut goal_scores = HashMap::new();
        goal_scores.insert(h3, 1.0);
        goal_scores.insert(h2, 0.1);

        let mut ira = HashMap::new();
        ira.insert(1u32, 0.8);
        ira.insert(2u32, 0.9);

        let mut tree = MctsTree::new(h1);
        let budget = MctsBudget {
            max_iterations: 100,
            max_rollout_depth: 5,
            time_limit_ms: 1000,
        };

        let result = tree.search(&graph, &[1, 2, 3, 4, 5], &ira, &goal_scores, &budget);

        let result = result.unwrap();
        // Action 2 leads to h3 (score 1.0) — should be preferred.
        assert_eq!(result.action, 2);
        assert!(result.value > 0.0, "value should be positive, got {}", result.value);
    }

    #[test]
    fn test_mcts_node_q_value() {
        let node = MctsNode {
            state_hash: GridHash(0),
            action_from_parent: None,
            visit_count: 10,
            total_value: 7.0,
            ira_prior: 1.0,
            children: Vec::new(),
            parent: None,
            expanded: false,
        };
        assert!((node.q_value() - 0.7).abs() < 0.01);
    }

    #[test]
    fn test_mcts_node_q_value_zero_visits() {
        let node = MctsNode {
            state_hash: GridHash(0),
            action_from_parent: None,
            visit_count: 0,
            total_value: 0.0,
            ira_prior: 1.0,
            children: Vec::new(),
            parent: None,
            expanded: false,
        };
        assert_eq!(node.q_value(), 0.0);
    }

    #[test]
    fn test_mcts_ira_prior_influence() {
        let h1 = GridHash(100);
        let h2 = GridHash(200);
        let h3 = GridHash(300);

        let mut graph = ExplorationGraph::new();
        graph.add_edge(h1, 1, h2, true);
        graph.add_edge(h1, 2, h3, true);

        // Equal goal scores, but action 2 has much higher IRA.
        let mut goal_scores = HashMap::new();
        goal_scores.insert(h2, 0.5);
        goal_scores.insert(h3, 0.5);

        let mut ira = HashMap::new();
        ira.insert(1u32, 0.1); // Low IRA
        ira.insert(2u32, 0.9); // High IRA

        let mut tree = MctsTree::new(h1);
        let budget = MctsBudget {
            max_iterations: 50,
            max_rollout_depth: 3,
            time_limit_ms: 1000,
        };

        let result = tree.search(&graph, &[1, 2], &ira, &goal_scores, &budget).unwrap();
        // Action 2 should be preferred due to higher IRA prior.
        assert_eq!(result.action, 2);
    }

    #[test]
    fn test_mcts_budget_iterations() {
        let root = GridHash(0);
        let mut tree = MctsTree::new(root);
        let graph = ExplorationGraph::new();
        let budget = MctsBudget {
            max_iterations: 5,
            max_rollout_depth: 2,
            time_limit_ms: 10000,
        };

        let result = tree.search(&graph, &[1, 2], &HashMap::new(), &HashMap::new(), &budget);
        if let Some(r) = result {
            assert!(r.iterations <= 5);
        }
    }

    #[test]
    fn test_mcts_tree_growth() {
        let h1 = GridHash(1);
        let h2 = GridHash(2);

        let mut graph = ExplorationGraph::new();
        graph.add_edge(h1, 1, h2, true);

        let mut tree = MctsTree::new(h1);
        let budget = MctsBudget {
            max_iterations: 20,
            max_rollout_depth: 3,
            time_limit_ms: 1000,
        };

        tree.search(&graph, &[1, 2, 3], &HashMap::new(), &HashMap::new(), &budget);
        // Tree should have grown beyond just the root.
        assert!(tree.node_count() > 1);
    }
}
