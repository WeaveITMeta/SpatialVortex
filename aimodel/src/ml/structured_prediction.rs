//! Structured Prediction — CRF-like Cascading Trait Dependency Updates
//!
//! Table of Contents:
//! 1. TraitDependencyGraph — Directed graph of trait dependencies
//! 2. DependencyEdge — Weighted edge with propagation rules
//! 3. CRFPotentials — Pairwise and unary potentials for trait consistency
//! 4. BeliefPropagation — Message-passing for joint trait optimization
//! 5. CascadeEngine — Propagates updates through dependency graph
//! 6. ConsistencyChecker — Validates trait coherence after updates
//!
//! Architecture:
//! Structured prediction enforces coherence in trait updates by modeling
//! dependencies. Updating a "causal inference" trait cascades appropriately
//! to linked "probabilistic weighting" ones. Uses belief propagation
//! (loopy BP) on the trait dependency graph to find jointly optimal states.
//! This replaces independent trait updates with graph-aware propagation.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

use crate::storage::trait_ledger::{TraitValue, TraitDelta};

// =============================================================================
// 1. TraitDependencyGraph — Directed graph of trait dependencies
// =============================================================================

/// A directed graph encoding dependencies between traits.
/// When trait A is updated, dependent traits B, C, ... may need cascading updates.
pub struct TraitDependencyGraph {
    /// Adjacency list: trait_name → [(dependent_trait, edge)]
    edges: HashMap<String, Vec<DependencyEdge>>,
    /// Reverse adjacency: trait_name → [traits that depend on it]
    reverse_edges: HashMap<String, Vec<String>>,
    /// All trait names in the graph
    nodes: HashSet<String>,
    /// Unary potentials: trait_name → preferred value distribution
    unary_potentials: HashMap<String, UnaryPotential>,
}

/// A dependency edge between two traits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyEdge {
    /// Source trait
    pub source: String,
    /// Target trait (dependent)
    pub target: String,
    /// Propagation weight (how strongly source affects target)
    pub weight: f64,
    /// Propagation rule
    pub rule: PropagationRule,
    /// Pairwise potential (compatibility function)
    pub pairwise_potential: PairwisePotential,
}

/// Rules for how updates propagate along edges
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PropagationRule {
    /// Linear: target_delta = weight * source_delta
    Linear,
    /// Dampened: target_delta = weight * source_delta * decay^distance
    Dampened { decay: f64 },
    /// Threshold: only propagate if source_delta > threshold
    Threshold { min_delta: f64 },
    /// Inverse: target moves opposite to source (negative correlation)
    Inverse,
    /// Conditional: only propagate if source value meets condition
    Conditional { min_value: f64, max_value: f64 },
}

// =============================================================================
// 2. CRFPotentials — Pairwise and unary potentials for trait consistency
// =============================================================================

/// Unary potential: preference for a trait's value independent of neighbors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnaryPotential {
    /// Preferred value (mode of the distribution)
    pub preferred: f64,
    /// Precision (inverse variance) — higher = stronger preference
    pub precision: f64,
}

impl UnaryPotential {
    /// Evaluate potential: how compatible is this value with the preference?
    pub fn evaluate(&self, value: f64) -> f64 {
        let diff = value - self.preferred;
        (-0.5 * self.precision * diff * diff).exp()
    }
}

/// Pairwise potential: compatibility between two connected traits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PairwisePotential {
    /// Attractive: traits should have similar values
    Attractive { strength: f64 },
    /// Repulsive: traits should have different values
    Repulsive { strength: f64 },
    /// Correlation: traits should maintain a fixed ratio
    Correlation { ratio: f64, tolerance: f64 },
    /// Custom: arbitrary compatibility function parameters
    Custom { params: Vec<f64> },
}

impl PairwisePotential {
    /// Evaluate compatibility between two trait values
    pub fn evaluate(&self, value_a: f64, value_b: f64) -> f64 {
        match self {
            PairwisePotential::Attractive { strength } => {
                let diff = value_a - value_b;
                (-0.5 * strength * diff * diff).exp()
            }
            PairwisePotential::Repulsive { strength } => {
                let diff = value_a - value_b;
                1.0 - (-0.5 * strength * diff * diff).exp()
            }
            PairwisePotential::Correlation { ratio, tolerance } => {
                let expected_b = value_a * ratio;
                let diff = value_b - expected_b;
                (-0.5 * diff * diff / (tolerance * tolerance)).exp()
            }
            PairwisePotential::Custom { params } => {
                // Simple polynomial: params[0] + params[1]*a*b + params[2]*(a-b)^2
                let p0 = params.first().copied().unwrap_or(1.0);
                let p1 = params.get(1).copied().unwrap_or(0.0);
                let p2 = params.get(2).copied().unwrap_or(0.0);
                (p0 + p1 * value_a * value_b - p2 * (value_a - value_b).powi(2)).max(0.0)
            }
        }
    }
}

impl TraitDependencyGraph {
    /// Create a new empty graph
    pub fn new() -> Self {
        Self {
            edges: HashMap::new(),
            reverse_edges: HashMap::new(),
            nodes: HashSet::new(),
            unary_potentials: HashMap::new(),
        }
    }

    /// Add a trait node
    pub fn add_node(&mut self, name: &str) {
        self.nodes.insert(name.to_string());
    }

    /// Add a dependency edge
    pub fn add_edge(&mut self, edge: DependencyEdge) {
        self.nodes.insert(edge.source.clone());
        self.nodes.insert(edge.target.clone());
        self.reverse_edges.entry(edge.target.clone())
            .or_default()
            .push(edge.source.clone());
        self.edges.entry(edge.source.clone())
            .or_default()
            .push(edge);
    }

    /// Set unary potential for a trait
    pub fn set_unary_potential(&mut self, name: &str, potential: UnaryPotential) {
        self.unary_potentials.insert(name.to_string(), potential);
    }

    /// Get all dependents of a trait (direct)
    pub fn dependents(&self, name: &str) -> Vec<&DependencyEdge> {
        self.edges.get(name)
            .map(|edges| edges.iter().collect())
            .unwrap_or_default()
    }

    /// Get all traits that this trait depends on
    pub fn dependencies(&self, name: &str) -> Vec<&str> {
        self.reverse_edges.get(name)
            .map(|deps| deps.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Get topological order for cascade propagation
    pub fn topological_order(&self) -> Vec<String> {
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        for node in &self.nodes {
            in_degree.insert(node.as_str(), 0);
        }
        for edges in self.edges.values() {
            for edge in edges {
                *in_degree.entry(edge.target.as_str()).or_insert(0) += 1;
            }
        }

        let mut queue: VecDeque<String> = in_degree.iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&name, _)| name.to_string())
            .collect();

        let mut order = Vec::new();
        while let Some(node) = queue.pop_front() {
            order.push(node.clone());
            if let Some(edges) = self.edges.get(&node) {
                for edge in edges {
                    if let Some(deg) = in_degree.get_mut(edge.target.as_str()) {
                        *deg = deg.saturating_sub(1);
                        if *deg == 0 {
                            queue.push_back(edge.target.clone());
                        }
                    }
                }
            }
        }

        order
    }

    /// Node count
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Edge count
    pub fn edge_count(&self) -> usize {
        self.edges.values().map(|v| v.len()).sum()
    }
}

// =============================================================================
// 3. BeliefPropagation — Message-passing for joint trait optimization
// =============================================================================

/// Loopy belief propagation on the trait dependency graph.
/// Finds jointly optimal trait values respecting all pairwise constraints.
pub struct BeliefPropagation {
    /// Number of discretization bins for continuous values
    num_bins: usize,
    /// Value range [min, max] for discretization
    value_range: (f64, f64),
    /// Maximum iterations
    max_iterations: usize,
    /// Convergence threshold
    convergence_threshold: f64,
    /// Damping factor (0-1, higher = more damping)
    damping: f64,
}

/// Result of belief propagation
#[derive(Debug, Clone)]
pub struct BPResult {
    /// Optimal values for each trait
    pub optimal_values: HashMap<String, f64>,
    /// Marginal beliefs for each trait (discretized)
    pub beliefs: HashMap<String, Vec<f64>>,
    /// Number of iterations to convergence
    pub iterations: usize,
    /// Whether converged
    pub converged: bool,
    /// Total energy (lower = better joint configuration)
    pub energy: f64,
}

impl BeliefPropagation {
    /// Create with default parameters
    pub fn new() -> Self {
        Self {
            num_bins: 20,
            value_range: (0.0, 1.0),
            max_iterations: 50,
            convergence_threshold: 1e-4,
            damping: 0.5,
        }
    }

    /// Set parameters
    pub fn with_params(mut self, num_bins: usize, max_iter: usize, damping: f64) -> Self {
        self.num_bins = num_bins;
        self.max_iterations = max_iter;
        self.damping = damping;
        self
    }

    /// Run belief propagation on the graph with current trait values
    pub fn run(
        &self,
        graph: &TraitDependencyGraph,
        current_values: &HashMap<String, f64>,
    ) -> BPResult {
        let bin_width = (self.value_range.1 - self.value_range.0) / self.num_bins as f64;
        let bin_centers: Vec<f64> = (0..self.num_bins)
            .map(|i| self.value_range.0 + (i as f64 + 0.5) * bin_width)
            .collect();

        // Initialize beliefs (uniform or from unary potentials)
        let mut beliefs: HashMap<String, Vec<f64>> = HashMap::new();
        for node in &graph.nodes {
            let mut belief = vec![1.0 / self.num_bins as f64; self.num_bins];
            if let Some(potential) = graph.unary_potentials.get(node) {
                for (i, center) in bin_centers.iter().enumerate() {
                    belief[i] = potential.evaluate(*center);
                }
                // Normalize
                let sum: f64 = belief.iter().sum();
                if sum > 0.0 {
                    for b in &mut belief {
                        *b /= sum;
                    }
                }
            }
            beliefs.insert(node.clone(), belief);
        }

        // Initialize messages: (source, target) → message vector
        let mut messages: HashMap<(String, String), Vec<f64>> = HashMap::new();
        for edges in graph.edges.values() {
            for edge in edges {
                let uniform = vec![1.0 / self.num_bins as f64; self.num_bins];
                messages.insert((edge.source.clone(), edge.target.clone()), uniform.clone());
                messages.insert((edge.target.clone(), edge.source.clone()), uniform);
            }
        }

        // Iterate
        let mut converged = false;
        let mut iterations = 0;

        for iter in 0..self.max_iterations {
            iterations = iter + 1;
            let mut max_change = 0.0f64;

            // Update messages for each edge
            for edges in graph.edges.values() {
                for edge in edges {
                    // Message from source to target
                    let new_msg = self.compute_message(
                        &edge.source, &edge.target, &edge.pairwise_potential,
                        &beliefs, &messages, &bin_centers,
                    );

                    let key = (edge.source.clone(), edge.target.clone());
                    if let Some(old_msg) = messages.get(&key) {
                        // Damped update
                        let damped: Vec<f64> = old_msg.iter().zip(new_msg.iter())
                            .map(|(o, n)| self.damping * o + (1.0 - self.damping) * n)
                            .collect();
                        let change: f64 = damped.iter().zip(old_msg.iter())
                            .map(|(a, b)| (a - b).abs())
                            .sum();
                        max_change = max_change.max(change);
                        messages.insert(key, damped);
                    }
                }
            }

            // Update beliefs
            for node in &graph.nodes {
                let mut belief = vec![1.0; self.num_bins];

                // Unary potential
                if let Some(potential) = graph.unary_potentials.get(node) {
                    for (i, center) in bin_centers.iter().enumerate() {
                        belief[i] *= potential.evaluate(*center);
                    }
                }

                // Incoming messages
                if let Some(deps) = graph.reverse_edges.get(node) {
                    for dep in deps {
                        let key = (dep.clone(), node.clone());
                        if let Some(msg) = messages.get(&key) {
                            for (i, m) in msg.iter().enumerate() {
                                belief[i] *= m;
                            }
                        }
                    }
                }

                // Normalize
                let sum: f64 = belief.iter().sum();
                if sum > 0.0 {
                    for b in &mut belief {
                        *b /= sum;
                    }
                }
                beliefs.insert(node.clone(), belief);
            }

            if max_change < self.convergence_threshold {
                converged = true;
                break;
            }
        }

        // Extract optimal values (MAP estimate = argmax of beliefs)
        let mut optimal_values = HashMap::new();
        let mut energy = 0.0;
        for (node, belief) in &beliefs {
            let (best_idx, best_prob) = belief.iter().enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or((0, &0.0));
            optimal_values.insert(node.clone(), bin_centers[best_idx]);
            energy -= best_prob.ln().max(-100.0); // Negative log-likelihood
        }

        BPResult {
            optimal_values,
            beliefs,
            iterations,
            converged,
            energy,
        }
    }

    /// Compute message from source to target
    fn compute_message(
        &self,
        source: &str,
        target: &str,
        potential: &PairwisePotential,
        beliefs: &HashMap<String, Vec<f64>>,
        messages: &HashMap<(String, String), Vec<f64>>,
        bin_centers: &[f64],
    ) -> Vec<f64> {
        let source_belief = match beliefs.get(source) {
            Some(b) => b,
            None => return vec![1.0 / self.num_bins as f64; self.num_bins],
        };

        let mut msg = vec![0.0; self.num_bins];

        for (j, &target_val) in bin_centers.iter().enumerate() {
            let mut sum = 0.0;
            for (i, &source_val) in bin_centers.iter().enumerate() {
                let pairwise = potential.evaluate(source_val, target_val);
                // Exclude message from target back to source (cavity)
                let key = (target.to_string(), source.to_string());
                let cavity = messages.get(&key)
                    .and_then(|m| m.get(i))
                    .copied()
                    .unwrap_or(1.0);
                let belief_without_target = source_belief[i] / cavity.max(1e-10);
                sum += pairwise * belief_without_target;
            }
            msg[j] = sum;
        }

        // Normalize
        let total: f64 = msg.iter().sum();
        if total > 0.0 {
            for m in &mut msg {
                *m /= total;
            }
        }

        msg
    }
}

// =============================================================================
// 4. CascadeEngine — Propagates updates through dependency graph
// =============================================================================

/// Propagates trait updates through the dependency graph.
/// When a trait is updated, dependent traits receive cascading updates
/// according to the propagation rules on each edge.
pub struct CascadeEngine {
    /// The dependency graph
    pub graph: TraitDependencyGraph,
    /// Belief propagation solver
    pub bp: BeliefPropagation,
    /// Maximum cascade depth
    pub max_depth: usize,
    /// Minimum delta to propagate (below = stop cascade)
    pub min_propagation_delta: f64,
    /// Cascade statistics
    pub stats: CascadeStats,
}

/// Statistics for cascade operations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CascadeStats {
    /// Total cascades triggered
    pub total_cascades: u64,
    /// Total traits updated via cascade
    pub total_cascade_updates: u64,
    /// Average cascade depth
    pub avg_cascade_depth: f64,
    /// Maximum cascade depth observed
    pub max_cascade_depth_observed: usize,
    /// BP runs
    pub bp_runs: u64,
}

/// Result of a cascade operation
#[derive(Debug, Clone)]
pub struct CascadeResult {
    /// Traits that were updated (name → new delta)
    pub updates: HashMap<String, f64>,
    /// Cascade depth reached
    pub depth: usize,
    /// Total traits affected
    pub traits_affected: usize,
    /// Whether BP was used for joint optimization
    pub used_bp: bool,
}

impl CascadeEngine {
    /// Create a new cascade engine
    pub fn new(graph: TraitDependencyGraph) -> Self {
        Self {
            graph,
            bp: BeliefPropagation::new(),
            max_depth: 9, // Sacred 9
            min_propagation_delta: 0.001,
            stats: CascadeStats::default(),
        }
    }

    /// Propagate an update from a source trait through the graph.
    /// Returns all cascading updates that should be applied.
    pub fn cascade_update(
        &mut self,
        source_trait: &str,
        source_delta: f64,
    ) -> CascadeResult {
        self.stats.total_cascades += 1;
        let mut updates: HashMap<String, f64> = HashMap::new();
        let mut visited: HashSet<String> = HashSet::new();
        let mut queue: VecDeque<(String, f64, usize)> = VecDeque::new();

        queue.push_back((source_trait.to_string(), source_delta, 0));
        visited.insert(source_trait.to_string());

        let mut max_depth = 0usize;

        while let Some((current, delta, depth)) = queue.pop_front() {
            if depth > self.max_depth || delta.abs() < self.min_propagation_delta {
                continue;
            }
            max_depth = max_depth.max(depth);

            // Get dependents
            let dependents: Vec<DependencyEdge> = self.graph.dependents(&current)
                .into_iter().cloned().collect();

            for edge in dependents {
                if visited.contains(&edge.target) {
                    continue; // Avoid cycles
                }

                // Compute propagated delta
                let propagated = self.apply_propagation_rule(&edge, delta, depth);

                if propagated.abs() >= self.min_propagation_delta {
                    updates.insert(edge.target.clone(), propagated);
                    visited.insert(edge.target.clone());
                    queue.push_back((edge.target, propagated, depth + 1));
                    self.stats.total_cascade_updates += 1;
                }
            }
        }

        // Update stats
        self.stats.max_cascade_depth_observed = self.stats.max_cascade_depth_observed.max(max_depth);
        let n = self.stats.total_cascades as f64;
        self.stats.avg_cascade_depth = (self.stats.avg_cascade_depth * (n - 1.0) + max_depth as f64) / n;

        CascadeResult {
            traits_affected: updates.len(),
            depth: max_depth,
            updates,
            used_bp: false,
        }
    }

    /// Run belief propagation for joint optimization of all traits
    pub fn optimize_jointly(
        &mut self,
        current_values: &HashMap<String, f64>,
    ) -> BPResult {
        self.stats.bp_runs += 1;
        self.bp.run(&self.graph, current_values)
    }

    /// Apply propagation rule to compute cascaded delta
    fn apply_propagation_rule(&self, edge: &DependencyEdge, source_delta: f64, depth: usize) -> f64 {
        match &edge.rule {
            PropagationRule::Linear => {
                edge.weight * source_delta
            }
            PropagationRule::Dampened { decay } => {
                edge.weight * source_delta * decay.powi(depth as i32)
            }
            PropagationRule::Threshold { min_delta } => {
                if source_delta.abs() > *min_delta {
                    edge.weight * source_delta
                } else {
                    0.0
                }
            }
            PropagationRule::Inverse => {
                -edge.weight * source_delta
            }
            PropagationRule::Conditional { min_value, max_value } => {
                // Only propagate if source delta keeps value in range
                // (simplified: just check magnitude)
                if source_delta.abs() >= *min_value && source_delta.abs() <= *max_value {
                    edge.weight * source_delta
                } else {
                    0.0
                }
            }
        }
    }

    /// Build a standard dependency graph for the SpatialVortex trait system
    pub fn build_standard_graph() -> TraitDependencyGraph {
        let mut graph = TraitDependencyGraph::new();

        // Core ELP traits
        graph.add_node("ethos");
        graph.add_node("logos");
        graph.add_node("pathos");
        graph.add_node("confidence");
        graph.add_node("coherence");
        graph.add_node("causal_strength");
        graph.add_node("temporal_weight");
        graph.add_node("semantic_similarity");
        graph.add_node("entity_salience");

        // ELP → confidence (all three contribute)
        for source in &["ethos", "logos", "pathos"] {
            graph.add_edge(DependencyEdge {
                source: source.to_string(),
                target: "confidence".to_string(),
                weight: 0.33,
                rule: PropagationRule::Linear,
                pairwise_potential: PairwisePotential::Attractive { strength: 1.0 },
            });
        }

        // Confidence → coherence
        graph.add_edge(DependencyEdge {
            source: "confidence".to_string(),
            target: "coherence".to_string(),
            weight: 0.5,
            rule: PropagationRule::Dampened { decay: 0.9 },
            pairwise_potential: PairwisePotential::Correlation { ratio: 0.8, tolerance: 0.2 },
        });

        // Causal strength → confidence
        graph.add_edge(DependencyEdge {
            source: "causal_strength".to_string(),
            target: "confidence".to_string(),
            weight: 0.4,
            rule: PropagationRule::Linear,
            pairwise_potential: PairwisePotential::Attractive { strength: 0.8 },
        });

        // Semantic similarity → entity salience
        graph.add_edge(DependencyEdge {
            source: "semantic_similarity".to_string(),
            target: "entity_salience".to_string(),
            weight: 0.6,
            rule: PropagationRule::Dampened { decay: 0.85 },
            pairwise_potential: PairwisePotential::Attractive { strength: 0.5 },
        });

        // Temporal weight → causal strength (recent events have stronger causal links)
        graph.add_edge(DependencyEdge {
            source: "temporal_weight".to_string(),
            target: "causal_strength".to_string(),
            weight: 0.3,
            rule: PropagationRule::Threshold { min_delta: 0.05 },
            pairwise_potential: PairwisePotential::Correlation { ratio: 0.5, tolerance: 0.3 },
        });

        // Set unary potentials (preferred values)
        graph.set_unary_potential("confidence", UnaryPotential { preferred: 0.7, precision: 2.0 });
        graph.set_unary_potential("coherence", UnaryPotential { preferred: 0.8, precision: 1.5 });
        graph.set_unary_potential("ethos", UnaryPotential { preferred: 0.5, precision: 1.0 });
        graph.set_unary_potential("logos", UnaryPotential { preferred: 0.5, precision: 1.0 });
        graph.set_unary_potential("pathos", UnaryPotential { preferred: 0.5, precision: 1.0 });

        graph
    }
}

// =============================================================================
// 5. ConsistencyChecker — Validates trait coherence after updates
// =============================================================================

/// Validates that trait values are consistent with the dependency graph
pub struct ConsistencyChecker {
    /// Maximum allowed inconsistency score
    pub max_inconsistency: f64,
}

impl ConsistencyChecker {
    pub fn new(max_inconsistency: f64) -> Self {
        Self { max_inconsistency }
    }

    /// Check consistency of current trait values against the graph
    pub fn check(
        &self,
        graph: &TraitDependencyGraph,
        values: &HashMap<String, f64>,
    ) -> ConsistencyResult {
        let mut violations = Vec::new();
        let mut total_score = 0.0;
        let mut edge_count = 0;

        for edges in graph.edges.values() {
            for edge in edges {
                let source_val = values.get(&edge.source).copied().unwrap_or(0.5);
                let target_val = values.get(&edge.target).copied().unwrap_or(0.5);

                let compatibility = edge.pairwise_potential.evaluate(source_val, target_val);
                total_score += compatibility;
                edge_count += 1;

                if compatibility < self.max_inconsistency {
                    violations.push(ConsistencyViolation {
                        source: edge.source.clone(),
                        target: edge.target.clone(),
                        source_value: source_val,
                        target_value: target_val,
                        compatibility,
                        expected_range: self.expected_range(&edge.pairwise_potential, source_val),
                    });
                }
            }
        }

        let avg_consistency = if edge_count > 0 {
            total_score / edge_count as f64
        } else {
            1.0
        };

        ConsistencyResult {
            consistent: violations.is_empty(),
            avg_consistency,
            violations,
            edges_checked: edge_count,
        }
    }

    /// Compute expected range for target given source value and potential
    fn expected_range(&self, potential: &PairwisePotential, source_val: f64) -> (f64, f64) {
        match potential {
            PairwisePotential::Attractive { strength } => {
                let spread = 1.0 / strength.sqrt();
                (source_val - spread, source_val + spread)
            }
            PairwisePotential::Correlation { ratio, tolerance } => {
                let center = source_val * ratio;
                (center - tolerance, center + tolerance)
            }
            _ => (0.0, 1.0),
        }
    }
}

/// Result of consistency check
#[derive(Debug, Clone)]
pub struct ConsistencyResult {
    /// Whether all constraints are satisfied
    pub consistent: bool,
    /// Average consistency score (0-1)
    pub avg_consistency: f64,
    /// List of violations
    pub violations: Vec<ConsistencyViolation>,
    /// Number of edges checked
    pub edges_checked: usize,
}

/// A specific consistency violation
#[derive(Debug, Clone)]
pub struct ConsistencyViolation {
    pub source: String,
    pub target: String,
    pub source_value: f64,
    pub target_value: f64,
    pub compatibility: f64,
    pub expected_range: (f64, f64),
}
