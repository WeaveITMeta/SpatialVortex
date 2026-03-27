//! Symbol Resolver — Cross-episode variable unification with bandit learning
//!
//! Solves the synonym problem: if episode A uses `v` and episode B uses
//! `velocity`, are they the same variable? Rather than string matching, we
//! compare **derivative signatures** — if ∂a/∂v ≡ ∂b/∂velocity across all
//! shared context variables, they're structurally equivalent.
//!
//! A multi-armed bandit tracks which unifications actually led to improved
//! episode outcomes, so the resolver gets sharper over time.
//!
//! # Integration
//!
//! - Consumes: `CausalModel` edge derivative signatures
//! - Consumes: `ArcEpisodeRecord` outcomes (from eustress-common, read-only)
//! - Produces: `UnifiedSymbol` mappings that CausalModel uses to merge edges
//! - Lives in: `aimodel/src/ml/` (SpatialVortex, not EustressEngine)

use std::collections::HashMap;

use super::causal_model::{CausalModel, DerivativeSignature, SymbolicDerivative};
use super::sacred_moe::PHI_INV;

// ─── Constants ───────────────────────────────────────────────────────────────

/// Minimum signature similarity to propose a unification.
const UNIFICATION_THRESHOLD: f64 = 0.85;
/// Bandit exploration parameter (UCB1).
const UCB_EXPLORATION: f64 = 1.414; // √2
/// Initial bandit arm value (optimistic start).
const INITIAL_ARM_VALUE: f64 = 0.5;
/// Minimum pulls before an arm can be pruned.
const MIN_PULLS_BEFORE_PRUNE: u32 = 5;
/// Arm value below which a unification is rejected.
const PRUNE_THRESHOLD: f64 = 0.15;

// ─── Equivalence Class ──────────────────────────────────────────────────────

/// A set of variable names that have been unified as representing the same
/// underlying concept.
#[derive(Debug, Clone)]
pub struct EquivalenceClass {
    /// Canonical name (shortest, or most frequently successful).
    pub canonical: String,
    /// All aliases that map to this canonical.
    pub aliases: Vec<String>,
    /// The derivative signature that unifies them.
    pub signature: DerivativeSignature,
    /// Confidence in the unification (from bandit).
    pub confidence: f64,
}

// ─── Bandit Arm ──────────────────────────────────────────────────────────────

/// One arm in the multi-armed bandit: represents a proposed unification
/// of two variable names.
#[derive(Debug, Clone)]
struct BanditArm {
    /// The two symbols being unified.
    symbol_a: String,
    symbol_b: String,
    /// Cumulative reward (from episodes where this unification was active).
    total_reward: f64,
    /// Number of times this arm was pulled.
    pull_count: u32,
    /// Is this unification currently active?
    active: bool,
}

impl BanditArm {
    fn new(a: String, b: String) -> Self {
        Self {
            symbol_a: a,
            symbol_b: b,
            total_reward: INITIAL_ARM_VALUE,
            pull_count: 1, // start at 1 to avoid division by zero
            active: false,
        }
    }

    /// Average reward.
    fn mean_reward(&self) -> f64 {
        self.total_reward / self.pull_count as f64
    }

    /// UCB1 score: exploitation (mean) + exploration (uncertainty).
    fn ucb1_score(&self, total_pulls: u32) -> f64 {
        let exploitation = self.mean_reward();
        let exploration =
            UCB_EXPLORATION * ((total_pulls as f64).ln() / self.pull_count as f64).sqrt();
        exploitation + exploration
    }
}

// ─── Equivalence Cache ───────────────────────────────────────────────────────

/// The cache of resolved equivalences + the bandit that learns which
/// unifications are productive.
#[derive(Debug, Clone)]
pub struct EquivalenceCache {
    /// Active equivalence classes (accepted unifications).
    classes: Vec<EquivalenceClass>,
    /// Fast lookup: alias → canonical name.
    alias_map: HashMap<String, String>,
}

impl EquivalenceCache {
    pub fn new() -> Self {
        Self {
            classes: Vec::new(),
            alias_map: HashMap::new(),
        }
    }

    /// Resolve a variable name to its canonical form.
    /// Returns the canonical name if unified, or the original name.
    pub fn resolve<'a>(&'a self, name: &'a str) -> &'a str {
        self.alias_map
            .get(name)
            .map(|s| s.as_str())
            .unwrap_or(name)
    }

    /// Register a unification: `alias` is equivalent to `canonical`.
    pub fn unify(&mut self, canonical: &str, alias: &str, sig: DerivativeSignature, confidence: f64) {
        // Check if either is already mapped
        let resolved_canonical = self.resolve(canonical).to_string();
        let resolved_alias = self.resolve(alias).to_string();

        if resolved_canonical == resolved_alias {
            return; // already unified
        }

        // Add to alias map
        self.alias_map
            .insert(resolved_alias.clone(), resolved_canonical.clone());

        // Add or update equivalence class
        if let Some(class) = self
            .classes
            .iter_mut()
            .find(|c| c.canonical == resolved_canonical)
        {
            if !class.aliases.contains(&resolved_alias) {
                class.aliases.push(resolved_alias);
            }
            class.confidence = class.confidence.max(confidence);
        } else {
            self.classes.push(EquivalenceClass {
                canonical: resolved_canonical,
                aliases: vec![resolved_alias],
                signature: sig,
                confidence,
            });
        }
    }

    /// Remove a unification (when the bandit determines it was unproductive).
    pub fn split(&mut self, alias: &str) {
        self.alias_map.remove(alias);
        for class in &mut self.classes {
            class.aliases.retain(|a| a != alias);
        }
        self.classes.retain(|c| !c.aliases.is_empty() || c.canonical != alias);
    }

    /// All active equivalence classes.
    pub fn classes(&self) -> &[EquivalenceClass] {
        &self.classes
    }

    /// Number of active unifications.
    pub fn unification_count(&self) -> usize {
        self.alias_map.len()
    }
}

impl Default for EquivalenceCache {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Symbol Resolver ─────────────────────────────────────────────────────────

/// Main resolver: compares derivative signatures across the causal graph,
/// proposes unifications, and uses a UCB1 bandit to learn which ones
/// improve episode outcomes.
#[derive(Debug, Clone)]
pub struct SymbolResolver {
    /// Accepted unifications.
    pub cache: EquivalenceCache,
    /// Bandit arms: one per proposed unification.
    arms: Vec<BanditArm>,
    /// Total pulls across all arms (for UCB1 denominator).
    total_pulls: u32,
    /// Episode history: (arm_index, was_active, episode_score).
    history: Vec<(usize, bool, f32)>,
}

impl SymbolResolver {
    pub fn new() -> Self {
        Self {
            cache: EquivalenceCache::new(),
            arms: Vec::new(),
            total_pulls: 0,
            history: Vec::new(),
        }
    }

    /// Scan the causal graph for pairs of variables with matching derivative
    /// signatures. Returns candidate unifications (not yet accepted).
    pub fn discover_candidates(&self, model: &CausalModel) -> Vec<UnificationCandidate> {
        let mut candidates = Vec::new();
        let mut seen_pairs = HashMap::new();

        // Collect all (variable, derivative_signature) pairs from the graph
        let variables = self.collect_variable_signatures(model);

        // Compare all pairs
        for (i, (var_a, sigs_a)) in variables.iter().enumerate() {
            for (var_b, sigs_b) in variables.iter().skip(i + 1) {
                if var_a == var_b {
                    continue;
                }

                // Already unified?
                if self.cache.resolve(var_a) == self.cache.resolve(var_b) {
                    continue;
                }

                // Already proposed as a bandit arm?
                let pair_key = if var_a < var_b {
                    (var_a.clone(), var_b.clone())
                } else {
                    (var_b.clone(), var_a.clone())
                };
                if seen_pairs.contains_key(&pair_key) {
                    continue;
                }

                // Compare derivative signatures
                let similarity = self.signature_similarity(sigs_a, sigs_b);
                if similarity >= UNIFICATION_THRESHOLD {
                    seen_pairs.insert(pair_key, true);
                    candidates.push(UnificationCandidate {
                        symbol_a: var_a.clone(),
                        symbol_b: var_b.clone(),
                        similarity,
                        matching_signatures: sigs_a
                            .iter()
                            .filter(|s| sigs_b.contains(s))
                            .cloned()
                            .collect(),
                    });
                }
            }
        }

        // Sort by similarity descending
        candidates.sort_by(|a, b| {
            b.similarity
                .partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        candidates
    }

    /// Accept a candidate unification: create a bandit arm and activate it.
    pub fn accept_candidate(&mut self, candidate: &UnificationCandidate) {
        // Choose canonical: shorter name wins (heuristic: shorter = more standard)
        let (canonical, alias) = if candidate.symbol_a.len() <= candidate.symbol_b.len() {
            (&candidate.symbol_a, &candidate.symbol_b)
        } else {
            (&candidate.symbol_b, &candidate.symbol_a)
        };

        // Create bandit arm
        let mut arm = BanditArm::new(canonical.clone(), alias.clone());
        arm.active = true;
        self.arms.push(arm);

        // Register in cache
        let sig = candidate
            .matching_signatures
            .first()
            .cloned()
            .unwrap_or(DerivativeSignature {
                kind: "unknown".into(),
                variables: vec![],
                degree: 0,
                hash: 0,
            });
        self.cache
            .unify(canonical, alias, sig, candidate.similarity);
    }

    /// Update the bandit after an episode. Reinforces active unifications that
    /// correlated with good outcomes; weakens those that didn't help.
    pub fn update_from_episode(&mut self, goal_reached: bool, final_score: f32) {
        let reward = if goal_reached {
            final_score as f64
        } else {
            -(1.0 - final_score as f64) * 0.3 // gentler punishment
        };

        for (i, arm) in self.arms.iter_mut().enumerate() {
            if arm.active {
                arm.total_reward += reward;
                arm.pull_count += 1;
                self.total_pulls += 1;
                self.history.push((i, true, final_score));
            }
        }

        // Prune clearly bad unifications
        let mut to_split = Vec::new();
        for arm in &self.arms {
            if arm.pull_count >= MIN_PULLS_BEFORE_PRUNE
                && arm.mean_reward() < PRUNE_THRESHOLD
                && arm.active
            {
                to_split.push(arm.symbol_b.clone());
            }
        }
        for alias in &to_split {
            self.cache.split(alias);
        }
        for arm in &mut self.arms {
            if to_split.contains(&arm.symbol_b) {
                arm.active = false;
            }
        }
    }

    /// Use UCB1 to select which pending (inactive) candidates to try next.
    /// Returns up to `k` arms to activate.
    pub fn select_explorations(&mut self, k: usize) -> Vec<(String, String)> {
        let mut inactive: Vec<(usize, f64)> = self
            .arms
            .iter()
            .enumerate()
            .filter(|(_, arm)| !arm.active)
            .map(|(i, arm)| (i, arm.ucb1_score(self.total_pulls.max(1))))
            .collect();

        inactive.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let mut activated = Vec::new();
        for (idx, _score) in inactive.into_iter().take(k) {
            self.arms[idx].active = true;
            let arm = &self.arms[idx];
            // Re-register in cache
            self.cache.unify(
                &arm.symbol_a,
                &arm.symbol_b,
                DerivativeSignature {
                    kind: "bandit_explore".into(),
                    variables: vec![],
                    degree: 0,
                    hash: 0,
                },
                PHI_INV as f64, // moderate initial confidence
            );
            activated.push((arm.symbol_a.clone(), arm.symbol_b.clone()));
        }
        activated
    }

    /// Resolve a variable name through the equivalence cache.
    pub fn resolve<'a>(&'a self, name: &'a str) -> &'a str {
        self.cache.resolve(name)
    }

    /// Apply all active unifications to a causal model: merge edges that
    /// use aliased variable names so they point to canonical names.
    ///
    /// Returns the number of edges that were rewritten.
    pub fn apply_to_model(&self, model: &mut CausalModel) -> usize {
        let mut rewrites = 0;

        // Collect edges that need rewriting
        let mut edges_to_add = Vec::new();

        for class in self.cache.classes() {
            for alias in &class.aliases {
                // Edges FROM alias → rewrite to FROM canonical
                let alias_edges: Vec<_> = model.edges_from(alias).to_vec();
                for edge in alias_edges {
                    let mut new_edge = edge.clone();
                    new_edge.cause = class.canonical.clone();
                    edges_to_add.push(new_edge);
                    rewrites += 1;
                }
            }
        }

        for edge in edges_to_add {
            model.add_edge(edge);
        }

        rewrites
    }

    // ── Internal helpers ─────────────────────────────────────────────────

    /// Collect all (variable_name, Vec<DerivativeSignature>) from the causal graph.
    fn collect_variable_signatures(
        &self,
        model: &CausalModel,
    ) -> Vec<(String, Vec<DerivativeSignature>)> {
        let mut result: HashMap<String, Vec<DerivativeSignature>> = HashMap::new();

        // For each cause variable, collect the signatures of its outgoing edges
        for var in &self.all_variables(model) {
            let edges = model.edges_from(var);
            let sigs: Vec<DerivativeSignature> =
                edges.iter().map(|e| e.derivative.signature()).collect();
            if !sigs.is_empty() {
                result
                    .entry(var.clone())
                    .or_default()
                    .extend(sigs);
            }
        }

        result.into_iter().collect()
    }

    fn all_variables(&self, model: &CausalModel) -> Vec<String> {
        // Collect all variable names that appear as cause or effect in any edge.
        let mut vars = std::collections::HashSet::new();
        // Known physics variables + dynamically discovered ones
        let known = [
            "velocity", "mass", "acceleration", "force", "kinetic_energy",
            "drag", "position", "displacement", "spring_force",
            "temperature", "pressure", "moles", "volume",
            "momentum", "spring_constant",
            "grid_colour", "reward", "step_count", "goal_distance",
        ];
        for var in &known {
            if !model.edges_from(var).is_empty() || !model.upstream_of(var).is_empty() {
                vars.insert(var.to_string());
            }
        }
        // Also check all variables the model knows about (covers discovered rules)
        for i in 0..model.variable_count() {
            // We need to iterate model variables — use edges_from on known +
            // check upstream to find any that are cause-only or effect-only.
            // Since CausalModel tracks variables internally, let's probe via
            // the edges it exposes.
        }
        // Include all variables the model tracks directly
        for var in model.variable_names() {
            vars.insert(var.clone());
        }
        vars.into_iter().collect()
    }

    /// Compute similarity between two sets of derivative signatures.
    /// Uses Jaccard-like overlap on structural fingerprints.
    fn signature_similarity(
        &self,
        sigs_a: &[DerivativeSignature],
        sigs_b: &[DerivativeSignature],
    ) -> f64 {
        if sigs_a.is_empty() || sigs_b.is_empty() {
            return 0.0;
        }

        let matches = sigs_a.iter().filter(|sa| sigs_b.contains(sa)).count();
        let union = sigs_a.len() + sigs_b.len() - matches;

        if union == 0 {
            0.0
        } else {
            matches as f64 / union as f64
        }
    }
}

impl Default for SymbolResolver {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Unification Candidate ──────────────────────────────────────────────────

/// A proposed unification of two variable names, discovered by comparing
/// their derivative signatures across the causal graph.
#[derive(Debug, Clone)]
pub struct UnificationCandidate {
    pub symbol_a: String,
    pub symbol_b: String,
    /// Jaccard similarity of derivative signature sets.
    pub similarity: f64,
    /// The signatures that matched between the two variables.
    pub matching_signatures: Vec<DerivativeSignature>,
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_identity() {
        let resolver = SymbolResolver::new();
        assert_eq!(resolver.resolve("velocity"), "velocity");
    }

    #[test]
    fn test_unify_and_resolve() {
        let mut resolver = SymbolResolver::new();
        let sig = DerivativeSignature {
            kind: "linear".into(),
            variables: vec!["mass".into()],
            degree: 1,
            hash: 42,
        };
        resolver.cache.unify("velocity", "v", sig, 0.95);

        assert_eq!(resolver.resolve("v"), "velocity");
        assert_eq!(resolver.resolve("velocity"), "velocity");
        assert_eq!(resolver.cache.unification_count(), 1);
    }

    #[test]
    fn test_transitive_unification() {
        let mut resolver = SymbolResolver::new();
        let sig = DerivativeSignature {
            kind: "linear".into(),
            variables: vec![],
            degree: 1,
            hash: 0,
        };
        resolver.cache.unify("velocity", "v", sig.clone(), 0.9);
        resolver.cache.unify("velocity", "speed", sig, 0.85);

        assert_eq!(resolver.resolve("v"), "velocity");
        assert_eq!(resolver.resolve("speed"), "velocity");
    }

    #[test]
    fn test_split_removes_alias() {
        let mut resolver = SymbolResolver::new();
        let sig = DerivativeSignature {
            kind: "linear".into(),
            variables: vec![],
            degree: 1,
            hash: 0,
        };
        resolver.cache.unify("velocity", "v", sig, 0.9);
        assert_eq!(resolver.resolve("v"), "velocity");

        resolver.cache.split("v");
        assert_eq!(resolver.resolve("v"), "v"); // back to self
    }

    #[test]
    fn test_bandit_reinforcement() {
        let mut resolver = SymbolResolver::new();

        // Manually add a bandit arm
        let mut arm = BanditArm::new("velocity".into(), "v".into());
        arm.active = true;
        resolver.arms.push(arm);
        resolver.cache.unify(
            "velocity",
            "v",
            DerivativeSignature {
                kind: "test".into(),
                variables: vec![],
                degree: 0,
                hash: 0,
            },
            0.9,
        );

        // Successful episodes reinforce
        for _ in 0..10 {
            resolver.update_from_episode(true, 0.8);
        }
        assert!(
            resolver.arms[0].mean_reward() > INITIAL_ARM_VALUE,
            "Successful episodes should increase arm value"
        );
    }

    #[test]
    fn test_bandit_prunes_bad_unifications() {
        let mut resolver = SymbolResolver::new();

        let mut arm = BanditArm::new("x".into(), "y".into());
        arm.active = true;
        arm.pull_count = 1;
        arm.total_reward = INITIAL_ARM_VALUE;
        resolver.arms.push(arm);
        resolver.cache.unify(
            "x",
            "y",
            DerivativeSignature {
                kind: "test".into(),
                variables: vec![],
                degree: 0,
                hash: 0,
            },
            0.5,
        );

        // Many failed episodes
        for _ in 0..10 {
            resolver.update_from_episode(false, 0.1);
        }

        // Arm should be pruned (deactivated)
        assert!(
            !resolver.arms[0].active,
            "Bad unification should be pruned after enough failures"
        );
        assert_eq!(resolver.resolve("y"), "y", "Alias should be split");
    }

    #[test]
    fn test_ucb1_explores_untried_arms() {
        let mut resolver = SymbolResolver::new();

        // Add two inactive arms with different histories
        let mut well_tried = BanditArm::new("a".into(), "b".into());
        well_tried.pull_count = 50;
        well_tried.total_reward = 25.0; // mean = 0.5
        resolver.arms.push(well_tried);

        let fresh = BanditArm::new("c".into(), "d".into());
        // fresh: pull_count=1, total_reward=0.5 (initial)
        resolver.arms.push(fresh);

        resolver.total_pulls = 51;

        // UCB1 should prefer the fresh arm (high uncertainty)
        let explorations = resolver.select_explorations(1);
        assert_eq!(explorations.len(), 1);
        assert_eq!(
            explorations[0],
            ("c".to_string(), "d".to_string()),
            "UCB1 should explore the less-tried arm"
        );
    }

    #[test]
    fn test_discover_candidates_with_matching_signatures() {
        let mut model = CausalModel::new();

        // Two variables with identical derivative structures
        model.add_edge(super::super::causal_model::CausalEdge {
            cause: "v".into(),
            effect: "energy".into(),
            derivative: SymbolicDerivative::Linear {
                coefficient: 1.0,
                variable: "mass".into(),
            },
            strength: 1.0,
            traversal_count: 0,
            law_name: None,
        });
        model.add_edge(super::super::causal_model::CausalEdge {
            cause: "velocity".into(),
            effect: "kinetic_e".into(),
            derivative: SymbolicDerivative::Linear {
                coefficient: 1.0,
                variable: "mass".into(),
            },
            strength: 1.0,
            traversal_count: 0,
            law_name: None,
        });

        let resolver = SymbolResolver::new();
        let candidates = resolver.discover_candidates(&model);

        // Should find v ↔ velocity as a candidate (same derivative signature)
        let found = candidates
            .iter()
            .any(|c| {
                (c.symbol_a == "v" && c.symbol_b == "velocity")
                    || (c.symbol_a == "velocity" && c.symbol_b == "v")
            });
        assert!(
            found,
            "Should discover v ↔ velocity as equivalent based on derivative signatures"
        );
    }
}
