//! Causal Model — Symbolic directed graph of physical/logical laws
//!
//! Holds causal relationships as a directed graph where edges carry symbolic
//! partial derivative functions. Given a delta on any variable, the graph
//! propagates effects downstream using the chain rule, answering:
//!
//!   "If velocity changed by Δv, what happened to kinetic_energy and drag?"
//!
//! This is **not** learned correlation — it is symbolic calculus grounded in
//! physics / game rules, with edge strengths refined by episode outcomes.
//!
//! # Key operations
//!
//! - `counterfactual_query(var, delta, ctx)` → downstream effects
//! - `do_intervention(cause, target_effect)` → required cause delta
//! - `update_edge_strengths(episode_record)` → reinforce/weaken edges
//!
//! # Integration
//!
//! Consumes: `ArcEpisodeRecord` (from eustress-common, read-only)
//! Produces: causal explanations + intervention plans for the PolicyHead
//! Lives in: `aimodel/src/ml/` (SpatialVortex, not EustressEngine)

use std::collections::HashMap;

use super::sacred_moe::{PHI, PHI_INV};

// ─── Constants ───────────────────────────────────────────────────────────────

/// Maximum propagation depth through the causal graph (sacred limit).
const MAX_PROPAGATION_DEPTH: usize = 9;
/// Edge strength decay per hop in multi-step causal chains.
const STRENGTH_DECAY: f32 = 0.9;
/// Minimum edge strength before pruning.
const MIN_EDGE_STRENGTH: f32 = 0.01;
/// Learning rate for edge strength updates from episode outcomes.
const EDGE_LEARNING_RATE: f32 = 0.05;

// ─── Symbolic Derivative ─────────────────────────────────────────────────────

/// A symbolic partial derivative: ∂effect/∂cause evaluated in a given context.
///
/// Represented as a simple expression tree that can be evaluated numerically
/// when concrete variable values are provided.
#[derive(Debug, Clone)]
pub enum SymbolicDerivative {
    /// Constant: ∂y/∂x = k  (e.g., F = ma → ∂F/∂a = m)
    Constant(f64),
    /// Linear in a context variable: ∂y/∂x = k * var
    /// e.g., KE = ½mv² → ∂KE/∂v = m*v
    Linear { coefficient: f64, variable: String },
    /// Power law: ∂y/∂x = k * var^n
    /// e.g., drag ∝ v² → ∂drag/∂v = 2*C_d*v
    Power {
        coefficient: f64,
        variable: String,
        exponent: f64,
    },
    /// Product of two derivatives (chain rule intermediate).
    Product(Box<SymbolicDerivative>, Box<SymbolicDerivative>),
    /// Sum of derivatives (multiple paths).
    Sum(Vec<SymbolicDerivative>),
    /// Inverse: 1/derivative (for do-calculus inversion).
    Inverse(Box<SymbolicDerivative>),
}

impl SymbolicDerivative {
    /// Evaluate the derivative given concrete variable bindings.
    pub fn evaluate(&self, context: &HashMap<String, f64>) -> f64 {
        match self {
            Self::Constant(k) => *k,
            Self::Linear {
                coefficient,
                variable,
            } => {
                let val = context.get(variable).copied().unwrap_or(1.0);
                coefficient * val
            }
            Self::Power {
                coefficient,
                variable,
                exponent,
            } => {
                let val = context.get(variable).copied().unwrap_or(1.0);
                coefficient * val.powf(*exponent)
            }
            Self::Product(a, b) => a.evaluate(context) * b.evaluate(context),
            Self::Sum(terms) => terms.iter().map(|t| t.evaluate(context)).sum(),
            Self::Inverse(inner) => {
                let val = inner.evaluate(context);
                if val.abs() < 1e-12 {
                    0.0 // avoid division by zero
                } else {
                    1.0 / val
                }
            }
        }
    }

    /// Signature: a deterministic fingerprint of the derivative structure
    /// (used by SymbolResolver for equivalence detection).
    pub fn signature(&self) -> DerivativeSignature {
        match self {
            Self::Constant(k) => DerivativeSignature {
                kind: "const".into(),
                variables: vec![],
                degree: 0,
                hash: float_hash(*k),
            },
            Self::Linear {
                coefficient,
                variable,
            } => DerivativeSignature {
                kind: "linear".into(),
                variables: vec![variable.clone()],
                degree: 1,
                hash: float_hash(*coefficient),
            },
            Self::Power {
                coefficient,
                variable,
                exponent,
            } => DerivativeSignature {
                kind: "power".into(),
                variables: vec![variable.clone()],
                degree: *exponent as u32,
                hash: float_hash(*coefficient) ^ float_hash(*exponent),
            },
            Self::Product(a, b) => {
                let sa = a.signature();
                let sb = b.signature();
                DerivativeSignature {
                    kind: "product".into(),
                    variables: [sa.variables, sb.variables].concat(),
                    degree: sa.degree + sb.degree,
                    hash: sa.hash.wrapping_mul(31) ^ sb.hash,
                }
            }
            Self::Sum(terms) => {
                let sigs: Vec<_> = terms.iter().map(|t| t.signature()).collect();
                let max_degree = sigs.iter().map(|s| s.degree).max().unwrap_or(0);
                let mut vars: Vec<String> =
                    sigs.iter().flat_map(|s| s.variables.clone()).collect();
                vars.sort();
                vars.dedup();
                let hash = sigs.iter().fold(0u64, |acc, s| acc ^ s.hash);
                DerivativeSignature {
                    kind: "sum".into(),
                    variables: vars,
                    degree: max_degree,
                    hash,
                }
            }
            Self::Inverse(inner) => {
                let si = inner.signature();
                DerivativeSignature {
                    kind: "inverse".into(),
                    variables: si.variables,
                    degree: si.degree,
                    hash: si.hash.wrapping_mul(0xDEAD_BEEF),
                }
            }
        }
    }
}

/// Structural fingerprint of a derivative — used for equivalence detection
/// across episodes that may use different variable names.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DerivativeSignature {
    pub kind: String,
    pub variables: Vec<String>,
    pub degree: u32,
    pub hash: u64,
}

fn float_hash(v: f64) -> u64 {
    u64::from_ne_bytes(v.to_ne_bytes())
}

// ─── Causal Edge ─────────────────────────────────────────────────────────────

/// Directed edge: cause → effect, carrying the symbolic ∂effect/∂cause.
#[derive(Debug, Clone)]
pub struct CausalEdge {
    pub cause: String,
    pub effect: String,
    /// Symbolic partial derivative: ∂effect/∂cause.
    pub derivative: SymbolicDerivative,
    /// Edge strength in [0, 1]. Updated by episode outcomes.
    pub strength: f32,
    /// How many times this edge was traversed during inference.
    pub traversal_count: u64,
    /// Optional human-readable law name (e.g., "Hooke's law", "ideal gas").
    pub law_name: Option<String>,
}

// ─── Causal Effect ───────────────────────────────────────────────────────────

/// Result of propagating a delta through the causal graph.
#[derive(Debug, Clone)]
pub struct CausalEffect {
    pub variable: String,
    /// Computed delta on this variable.
    pub delta: f64,
    /// Confidence = product of edge strengths along the causal path.
    pub confidence: f32,
    /// The chain of edges traversed to reach this effect.
    pub causal_chain: Vec<String>,
    /// Depth in the propagation tree.
    pub depth: usize,
}

// ─── Intervention ────────────────────────────────────────────────────────────

/// Result of a do-calculus inversion: do(cause = x) to achieve target effect.
#[derive(Debug, Clone)]
pub struct Intervention {
    pub cause: String,
    pub required_delta: f64,
    pub target_variable: String,
    pub target_delta: f64,
    /// Confidence in the intervention (product of edge strengths).
    pub confidence: f32,
    /// The causal path used.
    pub via_chain: Vec<String>,
}

// ─── Causal Graph ────────────────────────────────────────────────────────────

/// Directed acyclic graph of causal relationships with symbolic derivatives.
///
/// The graph is populated with known physical/logical laws at init time and
/// refined by episode outcomes via `update_edge_strengths`.
#[derive(Debug, Clone)]
pub struct CausalModel {
    /// All edges indexed by cause variable.
    edges_by_cause: HashMap<String, Vec<CausalEdge>>,
    /// Reverse index: effect → edges that produce it.
    edges_by_effect: HashMap<String, Vec<CausalEdge>>,
    /// All known variable names.
    variables: Vec<String>,
    /// Cumulative episode count (for learning rate annealing).
    episode_count: u64,
}

impl CausalModel {
    pub fn new() -> Self {
        Self {
            edges_by_cause: HashMap::new(),
            edges_by_effect: HashMap::new(),
            variables: Vec::new(),
            episode_count: 0,
        }
    }

    /// Create a model pre-loaded with common physical laws.
    pub fn with_physics_laws() -> Self {
        let mut model = Self::new();

        // Newton's second law: F = ma → ∂F/∂a = m, ∂F/∂m = a
        model.add_edge(CausalEdge {
            cause: "acceleration".into(),
            effect: "force".into(),
            derivative: SymbolicDerivative::Linear {
                coefficient: 1.0,
                variable: "mass".into(),
            },
            strength: 1.0,
            traversal_count: 0,
            law_name: Some("Newton's second law (F=ma)".into()),
        });
        model.add_edge(CausalEdge {
            cause: "mass".into(),
            effect: "force".into(),
            derivative: SymbolicDerivative::Linear {
                coefficient: 1.0,
                variable: "acceleration".into(),
            },
            strength: 1.0,
            traversal_count: 0,
            law_name: Some("Newton's second law (F=ma)".into()),
        });

        // Kinetic energy: KE = ½mv² → ∂KE/∂v = mv
        model.add_edge(CausalEdge {
            cause: "velocity".into(),
            effect: "kinetic_energy".into(),
            derivative: SymbolicDerivative::Linear {
                coefficient: 1.0,
                variable: "momentum".into(), // m*v
            },
            strength: 1.0,
            traversal_count: 0,
            law_name: Some("Kinetic energy (½mv²)".into()),
        });

        // Drag: F_d ∝ v² → ∂F_d/∂v = 2*C_d*v
        model.add_edge(CausalEdge {
            cause: "velocity".into(),
            effect: "drag".into(),
            derivative: SymbolicDerivative::Power {
                coefficient: 2.0,
                variable: "velocity".into(),
                exponent: 1.0, // derivative of v² is 2v
            },
            strength: 0.9,
            traversal_count: 0,
            law_name: Some("Drag force (∝v²)".into()),
        });

        // Hooke's law: F = -kx → ∂F/∂x = -k
        model.add_edge(CausalEdge {
            cause: "displacement".into(),
            effect: "spring_force".into(),
            derivative: SymbolicDerivative::Linear {
                coefficient: -1.0,
                variable: "spring_constant".into(),
            },
            strength: 1.0,
            traversal_count: 0,
            law_name: Some("Hooke's law (F=-kx)".into()),
        });

        // Ideal gas: PV = nRT → ∂P/∂T = nR/V
        model.add_edge(CausalEdge {
            cause: "temperature".into(),
            effect: "pressure".into(),
            derivative: SymbolicDerivative::Product(
                Box::new(SymbolicDerivative::Linear {
                    coefficient: 1.0,
                    variable: "moles".into(),
                }),
                Box::new(SymbolicDerivative::Inverse(Box::new(
                    SymbolicDerivative::Linear {
                        coefficient: 1.0,
                        variable: "volume".into(),
                    },
                ))),
            ),
            strength: 1.0,
            traversal_count: 0,
            law_name: Some("Ideal gas law (PV=nRT)".into()),
        });

        // Position ↔ velocity: ∂position/∂t = velocity (constant derivative)
        model.add_edge(CausalEdge {
            cause: "velocity".into(),
            effect: "position".into(),
            derivative: SymbolicDerivative::Constant(1.0), // dx/dv * dt
            strength: 1.0,
            traversal_count: 0,
            law_name: Some("Kinematics (dx = v·dt)".into()),
        });

        model
    }

    // ── Graph construction ───────────────────────────────────────────────

    /// Add a causal edge to the graph.
    pub fn add_edge(&mut self, edge: CausalEdge) {
        let cause = edge.cause.clone();
        let effect = edge.effect.clone();

        // Track variables
        if !self.variables.contains(&cause) {
            self.variables.push(cause.clone());
        }
        if !self.variables.contains(&effect) {
            self.variables.push(effect.clone());
        }

        self.edges_by_effect
            .entry(effect)
            .or_default()
            .push(edge.clone());
        self.edges_by_cause.entry(cause).or_default().push(edge);
    }

    /// Add a causal edge from ARC-specific observations.
    /// Useful for dynamically discovered rules during episodes.
    pub fn add_discovered_rule(
        &mut self,
        cause: &str,
        effect: &str,
        derivative: SymbolicDerivative,
        law_name: &str,
    ) {
        self.add_edge(CausalEdge {
            cause: cause.into(),
            effect: effect.into(),
            derivative,
            strength: 0.5, // start uncertain for discovered rules
            traversal_count: 0,
            law_name: Some(law_name.into()),
        });
    }

    // ── Counterfactual query ─────────────────────────────────────────────

    /// "If `variable` changed by `delta`, what are all downstream effects?"
    ///
    /// Propagates the delta through the causal graph using the chain rule,
    /// up to `MAX_PROPAGATION_DEPTH` hops. Returns all affected variables
    /// sorted by confidence (descending).
    pub fn counterfactual_query(
        &mut self,
        variable: &str,
        delta: f64,
        context: &HashMap<String, f64>,
    ) -> Vec<CausalEffect> {
        let mut effects = Vec::new();
        let mut visited = HashMap::new();

        self.propagate(
            variable,
            delta,
            context,
            1.0,   // initial confidence
            0,     // depth
            &mut effects,
            &mut visited,
            &mut Vec::new(),
        );

        // Sort by confidence descending
        effects.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
        effects
    }

    fn propagate(
        &mut self,
        cause: &str,
        delta: f64,
        context: &HashMap<String, f64>,
        confidence: f32,
        depth: usize,
        effects: &mut Vec<CausalEffect>,
        visited: &mut HashMap<String, bool>,
        chain: &mut Vec<String>,
    ) {
        if depth >= MAX_PROPAGATION_DEPTH || confidence < MIN_EDGE_STRENGTH {
            return;
        }
        if visited.contains_key(cause) {
            return; // prevent cycles
        }
        visited.insert(cause.to_string(), true);
        chain.push(cause.to_string());

        // Clone edges to avoid borrow conflict
        let downstream: Vec<CausalEdge> = self
            .edges_by_cause
            .get(cause)
            .cloned()
            .unwrap_or_default();

        for edge in &downstream {
            let partial = edge.derivative.evaluate(context);
            let effect_delta = delta * partial;
            let edge_confidence = confidence * edge.strength * STRENGTH_DECAY;

            effects.push(CausalEffect {
                variable: edge.effect.clone(),
                delta: effect_delta,
                confidence: edge_confidence,
                causal_chain: chain.clone(),
                depth,
            });

            // Increment traversal count
            if let Some(edges) = self.edges_by_cause.get_mut(cause) {
                for e in edges.iter_mut() {
                    if e.effect == edge.effect {
                        e.traversal_count += 1;
                    }
                }
            }

            // Recurse: propagate effect_delta downstream
            self.propagate(
                &edge.effect,
                effect_delta,
                context,
                edge_confidence,
                depth + 1,
                effects,
                visited,
                chain,
            );
        }

        chain.pop();
        visited.remove(cause);
    }

    // ── Do-calculus (intervention) ───────────────────────────────────────

    /// "What value of `cause` produces `target_delta` on `target`?"
    ///
    /// Inverts the causal chain: x = Δeffect / (∂effect/∂cause).
    /// Searches all paths from cause → target and returns the one with
    /// highest confidence.
    pub fn do_intervention(
        &self,
        cause: &str,
        target: &str,
        target_delta: f64,
        context: &HashMap<String, f64>,
    ) -> Option<Intervention> {
        let paths = self.find_all_paths(cause, target, MAX_PROPAGATION_DEPTH);

        let mut best: Option<Intervention> = None;

        for path in paths {
            // Compute cumulative derivative along the path (chain rule)
            let mut cumulative = 1.0f64;
            let mut path_confidence = 1.0f32;
            let mut chain_names = Vec::new();

            for window in path.windows(2) {
                let (from, to) = (&window[0], &window[1]);
                if let Some(edge) = self.find_edge(from, to) {
                    let partial = edge.derivative.evaluate(context);
                    cumulative *= partial;
                    path_confidence *= edge.strength * STRENGTH_DECAY;
                    if let Some(ref name) = edge.law_name {
                        chain_names.push(name.clone());
                    }
                }
            }

            if cumulative.abs() < 1e-12 {
                continue; // non-invertible path
            }

            let required_delta = target_delta / cumulative;

            let is_better = best
                .as_ref()
                .map_or(true, |b| path_confidence > b.confidence);

            if is_better {
                best = Some(Intervention {
                    cause: cause.into(),
                    required_delta,
                    target_variable: target.into(),
                    target_delta,
                    confidence: path_confidence,
                    via_chain: chain_names,
                });
            }
        }

        best
    }

    // ── Edge strength learning ───────────────────────────────────────────

    /// Update edge strengths based on an episode outcome.
    ///
    /// Edges that were traversed during a successful episode get reinforced;
    /// edges traversed during failure get weakened. Strength is clamped to
    /// [MIN_EDGE_STRENGTH, 1.0].
    ///
    /// Call this with the result from each ArcEpisodeRecord.
    pub fn update_from_episode(
        &mut self,
        goal_reached: bool,
        final_score: f32,
        actions_taken: &[String],
    ) {
        self.episode_count += 1;

        // Annealing: reduce learning rate over time
        let annealed_lr =
            EDGE_LEARNING_RATE / (1.0 + (self.episode_count as f32).sqrt() * 0.1);

        let reward_signal = if goal_reached {
            final_score * PHI_INV // positive reinforcement, φ-scaled
        } else {
            -final_score * (1.0 - PHI_INV) // negative, gentler
        };

        // Reinforce/weaken edges that were traversed
        for edges in self.edges_by_cause.values_mut() {
            for edge in edges.iter_mut() {
                if edge.traversal_count > 0 {
                    let update = annealed_lr * reward_signal
                        * (edge.traversal_count as f32).ln().max(1.0);
                    edge.strength = (edge.strength + update).clamp(MIN_EDGE_STRENGTH, 1.0);
                    edge.traversal_count = 0; // reset for next episode
                }
            }
        }

        // Also update reverse index
        for edges in self.edges_by_effect.values_mut() {
            for edge in edges.iter_mut() {
                edge.traversal_count = 0;
            }
        }
    }

    // ── Queries ──────────────────────────────────────────────────────────

    /// All variables that are directly caused by `variable`.
    pub fn downstream_of(&self, variable: &str) -> Vec<&str> {
        self.edges_by_cause
            .get(variable)
            .map(|edges| edges.iter().map(|e| e.effect.as_str()).collect())
            .unwrap_or_default()
    }

    /// All variables that directly cause `variable`.
    pub fn upstream_of(&self, variable: &str) -> Vec<&str> {
        self.edges_by_effect
            .get(variable)
            .map(|edges| edges.iter().map(|e| e.cause.as_str()).collect())
            .unwrap_or_default()
    }

    /// Number of causal edges in the graph.
    pub fn edge_count(&self) -> usize {
        self.edges_by_cause.values().map(|v| v.len()).sum()
    }

    /// Number of known variables.
    pub fn variable_count(&self) -> usize {
        self.variables.len()
    }

    /// All known variable names in the graph.
    pub fn variable_names(&self) -> &[String] {
        &self.variables
    }

    /// Get all edges from a specific cause (for SymbolResolver).
    pub fn edges_from(&self, cause: &str) -> &[CausalEdge] {
        self.edges_by_cause
            .get(cause)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    // ── Internal helpers ─────────────────────────────────────────────────

    fn find_edge(&self, from: &str, to: &str) -> Option<&CausalEdge> {
        self.edges_by_cause
            .get(from)?
            .iter()
            .find(|e| e.effect == to)
    }

    /// Find all simple paths from `start` to `end` (BFS, max depth).
    fn find_all_paths(
        &self,
        start: &str,
        end: &str,
        max_depth: usize,
    ) -> Vec<Vec<String>> {
        let mut all_paths = Vec::new();
        let mut current_path = vec![start.to_string()];
        self.dfs_paths(start, end, max_depth, &mut current_path, &mut all_paths);
        all_paths
    }

    fn dfs_paths(
        &self,
        current: &str,
        end: &str,
        max_depth: usize,
        path: &mut Vec<String>,
        all_paths: &mut Vec<Vec<String>>,
    ) {
        if current == end && path.len() > 1 {
            all_paths.push(path.clone());
            return;
        }
        if path.len() > max_depth {
            return;
        }

        if let Some(edges) = self.edges_by_cause.get(current) {
            for edge in edges {
                if !path.contains(&edge.effect) {
                    path.push(edge.effect.clone());
                    self.dfs_paths(&edge.effect, end, max_depth, path, all_paths);
                    path.pop();
                }
            }
        }
    }
}

impl Default for CausalModel {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn physics_context() -> HashMap<String, f64> {
        let mut ctx = HashMap::new();
        ctx.insert("mass".into(), 10.0);
        ctx.insert("velocity".into(), 5.0);
        ctx.insert("momentum".into(), 50.0); // m*v
        ctx.insert("acceleration".into(), 2.0);
        ctx.insert("spring_constant".into(), 100.0);
        ctx.insert("moles".into(), 1.0);
        ctx.insert("volume".into(), 22.4);
        ctx
    }

    #[test]
    fn test_counterfactual_velocity_change() {
        let mut model = CausalModel::with_physics_laws();
        let ctx = physics_context();

        // "If velocity increases by 1.0, what happens?"
        let effects = model.counterfactual_query("velocity", 1.0, &ctx);

        // Should affect kinetic_energy (via ∂KE/∂v = momentum = 50)
        let ke_effect = effects.iter().find(|e| e.variable == "kinetic_energy");
        assert!(ke_effect.is_some(), "Velocity should affect kinetic energy");
        let ke = ke_effect.unwrap();
        assert!(
            (ke.delta - 50.0).abs() < 0.01,
            "ΔKE should be ~50 (momentum * Δv), got {}",
            ke.delta
        );

        // Should affect drag (via ∂drag/∂v = 2*v = 10)
        let drag_effect = effects.iter().find(|e| e.variable == "drag");
        assert!(drag_effect.is_some(), "Velocity should affect drag");
        let drag = drag_effect.unwrap();
        assert!(
            (drag.delta - 10.0).abs() < 0.01,
            "Δdrag should be ~10 (2*v*Δv), got {}",
            drag.delta
        );
    }

    #[test]
    fn test_do_intervention() {
        let model = CausalModel::with_physics_laws();
        let ctx = physics_context();

        // "What acceleration produces a force of 20N?"
        // F = ma → a = F/m = 20/10 = 2
        let intervention = model.do_intervention(
            "acceleration",
            "force",
            20.0,
            &ctx,
        );
        assert!(intervention.is_some());
        let inv = intervention.unwrap();
        assert!(
            (inv.required_delta - 2.0).abs() < 0.01,
            "Should need Δa = 2.0 (20/10), got {}",
            inv.required_delta
        );
    }

    #[test]
    fn test_edge_strength_reinforcement() {
        let mut model = CausalModel::new();
        // Add a rule with initial strength 0.5 (room to grow)
        model.add_discovered_rule(
            "action",
            "reward",
            SymbolicDerivative::Constant(1.0),
            "test rule",
        );
        let ctx = HashMap::new();

        // Traverse (sets traversal_count > 0)
        let _effects = model.counterfactual_query("action", 1.0, &ctx);

        let initial_strength = model
            .edges_from("action")
            .iter()
            .find(|e| e.effect == "reward")
            .map(|e| e.strength)
            .unwrap();
        assert!((initial_strength - 0.5).abs() < 0.01, "Should start at 0.5");

        // Reinforce with a successful episode
        model.update_from_episode(true, 0.9, &[]);

        let updated_strength = model
            .edges_from("action")
            .iter()
            .find(|e| e.effect == "reward")
            .map(|e| e.strength)
            .unwrap();

        assert!(
            updated_strength > initial_strength,
            "Edge strength should increase after success: {initial_strength} → {updated_strength}"
        );
    }

    #[test]
    fn test_derivative_signature() {
        let d1 = SymbolicDerivative::Linear {
            coefficient: 2.0,
            variable: "mass".into(),
        };
        let d2 = SymbolicDerivative::Linear {
            coefficient: 2.0,
            variable: "mass".into(),
        };
        let d3 = SymbolicDerivative::Linear {
            coefficient: 3.0,
            variable: "mass".into(),
        };

        assert_eq!(d1.signature(), d2.signature(), "Same derivatives should match");
        assert_ne!(
            d1.signature(),
            d3.signature(),
            "Different coefficients should differ"
        );
    }

    #[test]
    fn test_discovered_rule() {
        let mut model = CausalModel::new();
        model.add_discovered_rule(
            "grid_colour",
            "reward",
            SymbolicDerivative::Constant(1.0),
            "ARC colour-reward rule",
        );

        assert_eq!(model.edge_count(), 1);
        assert_eq!(model.variable_count(), 2);

        let downstream = model.downstream_of("grid_colour");
        assert_eq!(downstream, vec!["reward"]);
    }

    #[test]
    fn test_upstream_downstream() {
        let model = CausalModel::with_physics_laws();
        let downstream = model.downstream_of("velocity");
        assert!(downstream.contains(&"kinetic_energy"));
        assert!(downstream.contains(&"drag"));
        assert!(downstream.contains(&"position"));

        let upstream = model.upstream_of("force");
        assert!(upstream.contains(&"acceleration"));
        assert!(upstream.contains(&"mass"));
    }
}
