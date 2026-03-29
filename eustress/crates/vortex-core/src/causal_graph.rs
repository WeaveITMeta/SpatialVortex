//! CausalGraph — Vortex's long-term understanding.
//!
//! A directed graph of causal relationships discovered through simulation.
//! Domain-agnostic: nodes represent state variables, actions, properties,
//! discovered rules (Z), and generalized causal laws (C).
//!
//! The graph powers hypothesis generation (Y branches) and counterfactual
//! reasoning without re-simulation.

use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

// ─── Node types ──────────────────────────────────────────────────────────

/// A node in the causal knowledge graph.
/// Domain-agnostic: works for grids, physics, games, or any WorldState.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CausalNode {
    /// An observable property of the world state.
    /// Examples: "grid.symmetry.horizontal", "object.position.y", "score"
    StateVariable {
        name: String,
        domain: Domain,
        value_type: ValueType,
    },

    /// An action that can be taken (a DSL primitive or composition).
    /// Examples: "rotate_cw", "apply_force(10N, 0.5m)", "move_left"
    Action {
        name: String,
        domain: Domain,
        parameters: Vec<ParameterSpec>,
    },

    /// A structural property detected by analysis.
    /// Examples: "input_has_horizontal_symmetry", "objects_are_sorted_by_size"
    Property {
        name: String,
        detector: String,
    },

    /// A discovered rule — a Z that hasn't been generalized to C yet.
    /// Created by individual simulation branches.
    Rule {
        name: String,
        conditions: Vec<String>,
        program: Vec<DSLOp>,
        confidence: f32,
        evidence_count: usize,
    },

    /// A generalized causal law — a C derived from multiple Z instances.
    /// Created by the SymbolResolver when it unifies multiple Rules.
    Law {
        name: String,
        symbolic_form: String,
        conditions: Vec<String>,
        effect_template: String,
        evidence_rules: Vec<String>,
        confidence: f32,
    },
}

impl CausalNode {
    pub fn name(&self) -> &str {
        match self {
            CausalNode::StateVariable { name, .. }
            | CausalNode::Action { name, .. }
            | CausalNode::Property { name, .. }
            | CausalNode::Rule { name, .. }
            | CausalNode::Law { name, .. } => name,
        }
    }

    pub fn is_law(&self) -> bool {
        matches!(self, CausalNode::Law { .. })
    }

    pub fn is_rule(&self) -> bool {
        matches!(self, CausalNode::Rule { .. })
    }

    pub fn confidence(&self) -> f32 {
        match self {
            CausalNode::Rule { confidence, .. } | CausalNode::Law { confidence, .. } => *confidence,
            _ => 1.0,
        }
    }
}

// ─── Edge types ──────────────────────────────────────────────────────────

/// Edges encode HOW nodes relate.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CausalEdge {
    /// "Action A causes StateVariable B to change by delta"
    /// Discovered by simulation branches (Y).
    Causes {
        strength: f32,
        delta_distribution: Distribution,
        context_conditions: Vec<String>,
        evidence_episodes: Vec<String>,
    },

    /// "Property P enables Action A" — A is only effective when P holds.
    Enables {
        strength: f32,
        evidence_episodes: Vec<String>,
    },

    /// "Property P1 implies Property P2" — structural relationship.
    Implies {
        strength: f32,
        counterexamples: usize,
    },

    /// "Rule Z is an instance of Law C" — SymbolResolver link.
    InstanceOf,

    /// "Law C1 composes with Law C2" — sequential application is valid.
    ComposesWith {
        order: CompositionOrder,
    },
}

impl CausalEdge {
    pub fn strength(&self) -> f32 {
        match self {
            CausalEdge::Causes { strength, .. }
            | CausalEdge::Enables { strength, .. }
            | CausalEdge::Implies { strength, .. } => *strength,
            CausalEdge::InstanceOf => 1.0,
            CausalEdge::ComposesWith { .. } => 1.0,
        }
    }
}

// ─── Hypothesis (output of suggest_hypotheses) ──────────────────────────

/// A candidate simulation branch suggested by the causal graph.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Hypothesis {
    pub description: String,
    pub program: Vec<DSLOp>,
    pub source_law: Option<String>,
    pub prior_confidence: f32,
    pub motivating_properties: Vec<Property>,
}

/// Predicted effect from a counterfactual query.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PredictedDelta {
    pub predicted_changes: Vec<Delta>,
    pub confidence: f32,
    pub reasoning: String,
}

// ─── Graph storage ───────────────────────────────────────────────────────

type NodeId = String;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Edge {
    from: NodeId,
    to: NodeId,
    edge: CausalEdge,
}

// ─── CausalGraph ─────────────────────────────────────────────────────────

/// The causal knowledge graph. Domain-agnostic long-term memory.
///
/// Nodes: state variables, actions, properties, rules (Z), laws (C).
/// Edges: causes, enables, implies, instance-of, composes-with.
///
/// The graph grows as Vortex solves tasks. Laws discovered in one domain
/// (e.g., "gravity settles objects" from Grid2D) can inform hypotheses
/// in other domains (e.g., Scene3D block stacking).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CausalGraph {
    nodes: HashMap<NodeId, CausalNode>,
    edges: Vec<Edge>,
    /// Index: node_id -> indices into `edges` where this node is the source.
    outgoing: HashMap<NodeId, Vec<usize>>,
    /// Index: node_id -> indices into `edges` where this node is the target.
    incoming: HashMap<NodeId, Vec<usize>>,
}

impl CausalGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            outgoing: HashMap::new(),
            incoming: HashMap::new(),
        }
    }

    /// Create a graph pre-populated with universal physics laws.
    pub fn with_physics_laws() -> Self {
        let mut g = Self::new();

        // Gravity law: unsupported objects fall to lowest valid position
        g.add_node(CausalNode::Law {
            name: "gravity_settles".into(),
            symbolic_form: "unsupported(obj) => position(obj).y = min_valid_y(obj)".into(),
            conditions: vec!["has_unsupported_objects".into()],
            effect_template: "objects_move_down".into(),
            evidence_rules: vec![],
            confidence: 0.5,
        });

        // Symmetry-rotation law
        g.add_node(CausalNode::Law {
            name: "symmetry_rotation".into(),
            symbolic_form: "asymmetric(axis_k) => rotate(align_k_to_target)".into(),
            conditions: vec!["input_asymmetric".into()],
            effect_template: "alignment_improved".into(),
            evidence_rules: vec![],
            confidence: 0.3,
        });

        // Conservation law
        g.add_node(CausalNode::Law {
            name: "color_conservation".into(),
            symbolic_form: "histogram(input) ~= histogram(output) => recolor_preserves_count".into(),
            conditions: vec!["color_histogram_match".into()],
            effect_template: "colors_remapped".into(),
            evidence_rules: vec![],
            confidence: 0.3,
        });

        g
    }

    // ─── Node operations ─────────────────────────────────────────────

    pub fn add_node(&mut self, node: CausalNode) -> NodeId {
        let id = node.name().to_string();
        self.nodes.insert(id.clone(), node);
        id
    }

    pub fn get_node(&self, id: &str) -> Option<&CausalNode> {
        self.nodes.get(id)
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Get all Law nodes.
    pub fn laws(&self) -> Vec<(&NodeId, &CausalNode)> {
        self.nodes.iter().filter(|(_, n)| n.is_law()).collect()
    }

    /// Get all Rule nodes.
    pub fn rules(&self) -> Vec<(&NodeId, &CausalNode)> {
        self.nodes.iter().filter(|(_, n)| n.is_rule()).collect()
    }

    // ─── Edge operations ─────────────────────────────────────────────

    pub fn add_edge(&mut self, from: &str, to: &str, edge: CausalEdge) {
        let idx = self.edges.len();
        self.edges.push(Edge {
            from: from.to_string(),
            to: to.to_string(),
            edge,
        });
        self.outgoing.entry(from.to_string()).or_default().push(idx);
        self.incoming.entry(to.to_string()).or_default().push(idx);
    }

    /// Get outgoing edges from a node.
    pub fn outgoing_edges(&self, node_id: &str) -> Vec<(&str, &CausalEdge)> {
        self.outgoing.get(node_id).map_or(vec![], |indices| {
            indices.iter().map(|&i| {
                let e = &self.edges[i];
                (e.to.as_str(), &e.edge)
            }).collect()
        })
    }

    /// Strengthen an existing Causes edge, or create a new one.
    fn strengthen_or_create_cause(
        &mut self,
        action_name: &str,
        effect_name: &str,
        delta_value: f64,
        episode_id: &str,
        conditions: &[String],
    ) {
        // Look for existing edge
        if let Some(indices) = self.outgoing.get(action_name) {
            for &idx in indices {
                let e = &mut self.edges[idx];
                if e.to == effect_name {
                    if let CausalEdge::Causes { ref mut strength, ref mut delta_distribution, ref mut evidence_episodes, .. } = e.edge {
                        delta_distribution.update(delta_value);
                        evidence_episodes.push(episode_id.to_string());
                        // Bayesian update: strength moves toward 1.0 with more evidence
                        *strength = 1.0 - (1.0 - *strength) * 0.9;
                        return;
                    }
                }
            }
        }

        // Ensure nodes exist
        if !self.nodes.contains_key(action_name) {
            self.add_node(CausalNode::Action {
                name: action_name.to_string(),
                domain: Domain::Universal,
                parameters: vec![],
            });
        }
        if !self.nodes.contains_key(effect_name) {
            self.add_node(CausalNode::StateVariable {
                name: effect_name.to_string(),
                domain: Domain::Universal,
                value_type: ValueType::Float,
            });
        }

        self.add_edge(action_name, effect_name, CausalEdge::Causes {
            strength: 0.5,
            delta_distribution: Distribution::single(delta_value),
            context_conditions: conditions.to_vec(),
            evidence_episodes: vec![episode_id.to_string()],
        });
    }

    // ─── Hypothesis generation ───────────────────────────────────────

    /// Given observed state properties, suggest hypotheses (Y branches)
    /// ranked by prior confidence from the causal graph.
    pub fn suggest_hypotheses(
        &self,
        observed_properties: &[Property],
        _goal: &GoalPredicate,
        max_hypotheses: usize,
    ) -> Vec<Hypothesis> {
        let prop_names: Vec<&str> = observed_properties.iter()
            .map(|p| p.name.as_str())
            .collect();

        let mut hypotheses = Vec::new();

        // 1. Find Laws whose conditions match observed properties
        for (id, node) in &self.nodes {
            if let CausalNode::Law { conditions, symbolic_form, confidence, .. } = node {
                let match_count = conditions.iter()
                    .filter(|c| prop_names.iter().any(|p| p.contains(c.as_str()) || c.contains(p)))
                    .count();

                if match_count > 0 || conditions.is_empty() {
                    let relevance = if conditions.is_empty() {
                        0.1
                    } else {
                        match_count as f32 / conditions.len() as f32
                    };

                    // Find actions connected to this law via edges
                    let programs = self.actions_from_law(id);

                    hypotheses.push(Hypothesis {
                        description: format!("Law '{}': {}", id, symbolic_form),
                        program: programs,
                        source_law: Some(id.clone()),
                        prior_confidence: confidence * relevance,
                        motivating_properties: observed_properties.iter()
                            .filter(|p| conditions.iter().any(|c| p.name.contains(c.as_str())))
                            .cloned()
                            .collect(),
                    });
                }
            }

            // Also consider Rules that haven't been generalized yet
            if let CausalNode::Rule { conditions, program, confidence, .. } = node {
                let match_count = conditions.iter()
                    .filter(|c| prop_names.iter().any(|p| p.contains(c.as_str()) || c.contains(p)))
                    .count();

                if match_count > 0 {
                    let relevance = match_count as f32 / conditions.len().max(1) as f32;

                    hypotheses.push(Hypothesis {
                        description: format!("Rule '{}': {} ops", id, program.len()),
                        program: program.clone(),
                        source_law: None,
                        prior_confidence: confidence * relevance * 0.8, // Rules slightly less trusted than Laws
                        motivating_properties: observed_properties.iter()
                            .filter(|p| conditions.iter().any(|c| p.name.contains(c.as_str())))
                            .cloned()
                            .collect(),
                    });
                }
            }
        }

        // Sort by confidence descending
        hypotheses.sort_by(|a, b| b.prior_confidence.partial_cmp(&a.prior_confidence).unwrap_or(std::cmp::Ordering::Equal));
        hypotheses.truncate(max_hypotheses);

        debug!("CausalGraph suggested {} hypotheses from {} nodes", hypotheses.len(), self.nodes.len());
        hypotheses
    }

    /// Find action programs connected to a law node.
    fn actions_from_law(&self, law_id: &str) -> Vec<DSLOp> {
        let mut ops = Vec::new();

        // Walk InstanceOf edges backwards to find Rules, then get their programs
        if let Some(indices) = self.incoming.get(law_id) {
            for &idx in indices {
                let e = &self.edges[idx];
                if matches!(e.edge, CausalEdge::InstanceOf) {
                    if let Some(CausalNode::Rule { program, .. }) = self.nodes.get(&e.from) {
                        ops.extend(program.iter().cloned());
                    }
                }
            }
        }

        ops
    }

    // ─── Episode integration ─────────────────────────────────────────

    /// After a simulation branch completes, update the graph.
    /// This is where Y's results become C's evidence.
    pub fn integrate_episode(&mut self, episode: &EpisodeRecord) {
        info!(
            "Integrating episode {} (task={}, success={}, score={:.2})",
            episode.episode_id, episode.task_id, episode.success, episode.final_score.accuracy
        );

        let condition_names: Vec<String> = episode.observed_properties.iter()
            .map(|p| p.name.clone())
            .collect();

        // 1. For each action and observed delta, strengthen/create Causes edges
        for (i, action) in episode.actions_taken.iter().enumerate() {
            if let Some(delta) = episode.state_deltas.get(i) {
                self.strengthen_or_create_cause(
                    &action.name,
                    &delta.kind,
                    delta.magnitude,
                    &episode.episode_id,
                    &condition_names,
                );
            }
        }

        // 2. If the episode was successful, create a Rule node
        if episode.success && !episode.actions_taken.is_empty() {
            let rule_name = format!("rule_{}_{}", episode.task_id, episode.episode_id);
            self.add_node(CausalNode::Rule {
                name: rule_name.clone(),
                conditions: condition_names,
                program: episode.actions_taken.clone(),
                confidence: episode.final_score.accuracy as f32,
                evidence_count: 1,
            });

            debug!("Created rule node: {}", rule_name);
        }

        // 3. Update existing Law confidence based on episode outcome
        for (_id, node) in self.nodes.iter_mut() {
            if let CausalNode::Law { conditions, confidence, evidence_rules, .. } = node {
                // If the episode's properties match this law's conditions
                let relevant = conditions.iter().any(|c| {
                    episode.observed_properties.iter().any(|p| p.name.contains(c.as_str()))
                });
                if relevant {
                    if episode.success {
                        *confidence = (*confidence * 0.9 + 0.1).min(1.0);
                        evidence_rules.push(episode.episode_id.clone());
                    } else {
                        *confidence = (*confidence * 0.95).max(0.01);
                    }
                }
            }
        }
    }

    // ─── Counterfactual reasoning ────────────────────────────────────

    /// "What would happen if we did action X instead of Y?"
    /// Uses the graph's causal laws to estimate without simulation.
    pub fn counterfactual(
        &self,
        _actual_action: &DSLOp,
        alternative_action: &DSLOp,
        context: &[Property],
    ) -> PredictedDelta {
        let mut predicted_changes = Vec::new();
        let mut total_confidence = 0.0f32;
        let mut n = 0;

        // Walk Causes edges from the alternative action
        if let Some(indices) = self.outgoing.get(&alternative_action.name) {
            for &idx in indices {
                let e = &self.edges[idx];
                if let CausalEdge::Causes { strength, ref delta_distribution, ref context_conditions, .. } = e.edge {
                    // Check if context conditions match
                    let context_match = context_conditions.is_empty() || context_conditions.iter().any(|c| {
                        context.iter().any(|p| p.name.contains(c.as_str()))
                    });

                    if context_match {
                        predicted_changes.push(Delta {
                            kind: e.to.clone(),
                            description: format!("{} -> {} (mean delta: {:.2})", alternative_action.name, e.to, delta_distribution.mean),
                            magnitude: delta_distribution.mean,
                        });
                        total_confidence += strength;
                        n += 1;
                    }
                }
            }
        }

        let confidence = if n > 0 { total_confidence / n as f32 } else { 0.0 };

        PredictedDelta {
            predicted_changes,
            confidence,
            reasoning: format!(
                "Counterfactual: {} with {} matching causal paths",
                alternative_action.name, n
            ),
        }
    }
}

impl Default for CausalGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_node_and_edge() {
        let mut g = CausalGraph::new();
        g.add_node(CausalNode::Action {
            name: "rotate_cw".into(),
            domain: Domain::Grid2D,
            parameters: vec![],
        });
        g.add_node(CausalNode::StateVariable {
            name: "cell_changes".into(),
            domain: Domain::Grid2D,
            value_type: ValueType::Int,
        });
        g.add_edge("rotate_cw", "cell_changes", CausalEdge::Causes {
            strength: 0.8,
            delta_distribution: Distribution::single(24.0),
            context_conditions: vec![],
            evidence_episodes: vec!["ep1".into()],
        });
        assert_eq!(g.node_count(), 2);
        assert_eq!(g.edge_count(), 1);
        assert_eq!(g.outgoing_edges("rotate_cw").len(), 1);
    }

    #[test]
    fn test_suggest_hypotheses_from_law() {
        let mut g = CausalGraph::with_physics_laws();
        let props = vec![Property {
            name: "has_unsupported_objects".into(),
            domain: Domain::Grid2D,
            value: PropertyValue::Bool(true),
        }];
        let h = g.suggest_hypotheses(&props, &GoalPredicate::ExactMatch, 10);
        assert!(!h.is_empty(), "Should suggest at least one hypothesis for gravity");
    }

    #[test]
    fn test_episode_integration() {
        let mut g = CausalGraph::new();
        let episode = EpisodeRecord {
            episode_id: "ep_test".into(),
            task_id: "task_1".into(),
            observed_properties: vec![Property {
                name: "asymmetric".into(),
                domain: Domain::Grid2D,
                value: PropertyValue::Bool(true),
            }],
            actions_taken: vec![DSLOp {
                name: "rotate_cw".into(),
                domain: Domain::Grid2D,
                parameters: vec![],
            }],
            state_deltas: vec![Delta {
                kind: "cell_changes".into(),
                description: "24 cells changed".into(),
                magnitude: 24.0,
            }],
            final_score: Score::perfect(),
            success: true,
            duration_ms: 100,
        };
        g.integrate_episode(&episode);
        assert!(g.node_count() >= 2);
        assert!(g.rules().len() >= 1);
    }
}
