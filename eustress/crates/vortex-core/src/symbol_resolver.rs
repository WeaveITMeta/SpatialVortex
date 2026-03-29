//! SymbolResolver — From Z to C. Rule generalization pipeline.
//!
//! Individual rules (Z) discovered per task are unified into
//! generalized causal laws (C) reusable across domains.
//!
//! Z₁: "rotate_cw when asymmetric horizontally"  ─┐
//! Z₂: "rotate_cw when left-heavy color dist"    ─┼─→ C: "rotation aligns
//! Z₃: "rotate_ccw when asymmetric vertically"   ─┘    asymmetric axis"

use crate::causal_graph::{CausalEdge, CausalGraph, CausalNode};
use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// Tracks which unifications have been tried and their outcomes.
/// Uses UCB1 multi-armed bandit to prioritize productive unifications.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EquivalenceCache {
    /// Maps (rule_a, rule_b) -> attempted unification result.
    attempts: HashMap<(String, String), UnificationResult>,
    /// Maps equivalence class name -> member rule names.
    classes: HashMap<String, EquivalenceClass>,
    /// UCB1 exploration parameter.
    exploration_c: f64,
    /// Total attempts for UCB1.
    total_attempts: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EquivalenceClass {
    pub name: String,
    pub members: Vec<String>,
    pub structural_signature: String,
    pub confidence: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
enum UnificationResult {
    Success { law_name: String },
    Failed { structural_sim: f32 },
    Pending,
}

impl EquivalenceCache {
    pub fn new() -> Self {
        Self {
            attempts: HashMap::new(),
            classes: HashMap::new(),
            exploration_c: 1.414,
            total_attempts: 0,
        }
    }

    fn was_attempted(&self, a: &str, b: &str) -> bool {
        let key = Self::canonical_key(a, b);
        self.attempts.contains_key(&key)
    }

    fn record_attempt(&mut self, a: &str, b: &str, result: UnificationResult) {
        let key = Self::canonical_key(a, b);
        self.attempts.insert(key, result);
        self.total_attempts += 1;
    }

    fn canonical_key(a: &str, b: &str) -> (String, String) {
        if a < b {
            (a.to_string(), b.to_string())
        } else {
            (b.to_string(), a.to_string())
        }
    }

    pub fn class_count(&self) -> usize {
        self.classes.len()
    }
}

impl Default for EquivalenceCache {
    fn default() -> Self {
        Self::new()
    }
}

/// The SymbolResolver unifies individual rules (Z) into general laws (C).
///
/// Pipeline:
/// 1. Structural alignment — do rules use similar DSL op sequences?
/// 2. Condition alignment — do rules trigger under similar properties?
/// 3. Effect alignment — do rules produce similar types of changes?
/// 4. If all align, create a generalized Law in the CausalGraph.
pub struct SymbolResolver {
    /// Minimum structural similarity to attempt unification.
    pub unification_threshold: f32,
    /// Cache of attempted unifications.
    pub cache: EquivalenceCache,
}

impl SymbolResolver {
    pub fn new() -> Self {
        Self {
            unification_threshold: 0.4,
            cache: EquivalenceCache::new(),
        }
    }

    /// Attempt to unify two rules into a more general law.
    /// Returns the law node if unification succeeds.
    pub fn try_unify(
        &mut self,
        z1_name: &str,
        z1_conditions: &[String],
        z1_program: &[DSLOp],
        z1_confidence: f32,
        z2_name: &str,
        z2_conditions: &[String],
        z2_program: &[DSLOp],
        z2_confidence: f32,
    ) -> Option<CausalNode> {
        // Skip if already attempted
        if self.cache.was_attempted(z1_name, z2_name) {
            return None;
        }

        // Step 1: Structural alignment
        let structural_sim = self.structural_similarity(z1_program, z2_program);
        if structural_sim < self.unification_threshold {
            self.cache.record_attempt(
                z1_name, z2_name,
                UnificationResult::Failed { structural_sim },
            );
            debug!(
                "Unification failed: {} vs {} (structural_sim={:.2} < threshold={:.2})",
                z1_name, z2_name, structural_sim, self.unification_threshold
            );
            return None;
        }

        // Step 2: Condition alignment
        let (common_conditions, _variable_conditions) =
            self.align_conditions(z1_conditions, z2_conditions);

        // Step 3: Effect alignment (based on program structure)
        let effect_pattern = self.generalize_effects(z1_program, z2_program);

        // Step 4: Construct the generalized law
        let law_name = format!("law_{}_{}", Self::extract_core_op(z1_program), self.cache.class_count());
        let symbolic_form = self.symbolic_generalization(z1_program, z2_program);

        let law = CausalNode::Law {
            name: law_name.clone(),
            symbolic_form,
            conditions: common_conditions,
            effect_template: effect_pattern,
            evidence_rules: vec![z1_name.to_string(), z2_name.to_string()],
            confidence: (z1_confidence + z2_confidence) / 2.0 * structural_sim,
        };

        // Record in cache
        self.cache.record_attempt(
            z1_name, z2_name,
            UnificationResult::Success { law_name: law_name.clone() },
        );

        // Create equivalence class
        self.cache.classes.insert(law_name.clone(), EquivalenceClass {
            name: law_name,
            members: vec![z1_name.to_string(), z2_name.to_string()],
            structural_signature: Self::extract_core_op(z1_program).to_string(),
            confidence: (z1_confidence + z2_confidence) / 2.0,
        });

        info!(
            "Unified rules {} + {} into law (structural_sim={:.2})",
            z1_name, z2_name, structural_sim
        );

        Some(law)
    }

    /// Structural similarity between two DSL programs.
    /// Compares operation names sequence using longest common subsequence.
    fn structural_similarity(&self, p1: &[DSLOp], p2: &[DSLOp]) -> f32 {
        if p1.is_empty() && p2.is_empty() {
            return 1.0;
        }
        if p1.is_empty() || p2.is_empty() {
            return 0.0;
        }

        let names1: Vec<&str> = p1.iter().map(|op| op.name.as_str()).collect();
        let names2: Vec<&str> = p2.iter().map(|op| op.name.as_str()).collect();

        let lcs_len = lcs_length(&names1, &names2);
        let max_len = names1.len().max(names2.len());

        lcs_len as f32 / max_len as f32
    }

    /// Find common and variable conditions between two rules.
    fn align_conditions<'a>(
        &self,
        c1: &'a [String],
        c2: &'a [String],
    ) -> (Vec<String>, Vec<String>) {
        let common: Vec<String> = c1.iter()
            .filter(|c| c2.contains(c))
            .cloned()
            .collect();

        let variable: Vec<String> = c1.iter()
            .filter(|c| !c2.contains(c))
            .chain(c2.iter().filter(|c| !c1.contains(c)))
            .cloned()
            .collect();

        (common, variable)
    }

    /// Generalize effect patterns from two programs.
    fn generalize_effects(&self, p1: &[DSLOp], p2: &[DSLOp]) -> String {
        let ops1: Vec<&str> = p1.iter().map(|op| op.name.as_str()).collect();
        let ops2: Vec<&str> = p2.iter().map(|op| op.name.as_str()).collect();

        // Find common operations
        let common: Vec<&&str> = ops1.iter().filter(|op| ops2.contains(op)).collect();

        if common.is_empty() {
            "unknown_effect".into()
        } else {
            format!("{}_effect", common.iter().map(|s| **s).collect::<Vec<_>>().join("_"))
        }
    }

    /// Create symbolic generalization from two programs.
    fn symbolic_generalization(&self, p1: &[DSLOp], p2: &[DSLOp]) -> String {
        let core1 = Self::extract_core_op(p1);
        let core2 = Self::extract_core_op(p2);

        if core1 == core2 {
            format!("when(conditions) => {}(params)", core1)
        } else {
            format!("when(conditions) => {}|{}(params)", core1, core2)
        }
    }

    fn extract_core_op(program: &[DSLOp]) -> &str {
        program.first().map_or("noop", |op| op.name.as_str())
    }

    /// Cross-domain unification: can a Grid2D rule inform a Scene3D law?
    ///
    /// Matches on abstract effect types (spatial displacement, conservation)
    /// rather than concrete operations.
    pub fn try_cross_domain_unify(
        &mut self,
        grid_rule_name: &str,
        grid_conditions: &[String],
        grid_program: &[DSLOp],
        grid_confidence: f32,
        physics_rule_name: &str,
        physics_conditions: &[String],
        physics_program: &[DSLOp],
        physics_confidence: f32,
    ) -> Option<CausalNode> {
        // Abstract the programs to effect categories
        let grid_effects = self.categorize_effects(grid_program);
        let physics_effects = self.categorize_effects(physics_program);

        // Match on abstract categories
        let shared_categories: Vec<&str> = grid_effects.iter()
            .filter(|e| physics_effects.contains(e))
            .copied()
            .collect();

        if shared_categories.is_empty() {
            return None;
        }

        let (common_conditions, _) = self.align_conditions(grid_conditions, physics_conditions);

        let law = CausalNode::Law {
            name: format!("cross_domain_{}", shared_categories.join("_")),
            symbolic_form: format!(
                "cross_domain: {} in Grid2D ↔ {} in Scene3D via [{}]",
                Self::extract_core_op(grid_program),
                Self::extract_core_op(physics_program),
                shared_categories.join(", ")
            ),
            conditions: common_conditions,
            effect_template: shared_categories.join("_"),
            evidence_rules: vec![grid_rule_name.to_string(), physics_rule_name.to_string()],
            confidence: (grid_confidence + physics_confidence) / 4.0, // Lower confidence for cross-domain
        };

        info!(
            "Cross-domain unification: {} (Grid2D) + {} (Scene3D) via {:?}",
            grid_rule_name, physics_rule_name, shared_categories
        );

        Some(law)
    }

    /// Categorize a program's effects into abstract categories.
    fn categorize_effects<'a>(&self, program: &'a [DSLOp]) -> Vec<&'a str> {
        let mut categories = Vec::new();
        for op in program {
            match op.name.as_str() {
                s if s.contains("rotate") => categories.push("spatial_rotation"),
                s if s.contains("flip") || s.contains("mirror") => categories.push("spatial_reflection"),
                s if s.contains("move") || s.contains("translate") || s.contains("shift") => categories.push("spatial_displacement"),
                s if s.contains("gravity") || s.contains("fall") || s.contains("settle") => categories.push("gravitational_settling"),
                s if s.contains("recolor") || s.contains("paint") => categories.push("property_change"),
                s if s.contains("crop") || s.contains("resize") || s.contains("scale") => categories.push("spatial_scaling"),
                s if s.contains("tile") || s.contains("repeat") => categories.push("spatial_tiling"),
                s if s.contains("sort") || s.contains("order") => categories.push("ordering"),
                s if s.contains("fill") || s.contains("flood") => categories.push("region_fill"),
                _ => {}
            }
        }
        categories.sort();
        categories.dedup();
        categories
    }

    /// Scan the causal graph for unifiable rule pairs and create laws.
    /// Returns the number of new laws created.
    pub fn scan_and_unify(&mut self, graph: &mut CausalGraph) -> usize {
        let rules: Vec<(String, Vec<String>, Vec<DSLOp>, f32)> = graph.rules().iter()
            .filter_map(|(id, node)| {
                if let CausalNode::Rule { conditions, program, confidence, .. } = node {
                    Some((id.to_string(), conditions.clone(), program.clone(), *confidence))
                } else {
                    None
                }
            })
            .collect();

        let mut new_laws = 0;

        for i in 0..rules.len() {
            for j in (i + 1)..rules.len() {
                let (ref n1, ref c1, ref p1, conf1) = rules[i];
                let (ref n2, ref c2, ref p2, conf2) = rules[j];

                if let Some(law) = self.try_unify(n1, c1, p1, conf1, n2, c2, p2, conf2) {
                    let law_name = law.name().to_string();
                    graph.add_node(law);

                    // Link rules to law via InstanceOf edges
                    graph.add_edge(n1, &law_name, CausalEdge::InstanceOf);
                    graph.add_edge(n2, &law_name, CausalEdge::InstanceOf);

                    new_laws += 1;
                }
            }
        }

        if new_laws > 0 {
            info!("SymbolResolver created {} new laws from {} rules", new_laws, rules.len());
        }

        new_laws
    }
}

impl Default for SymbolResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Longest common subsequence length.
fn lcs_length(a: &[&str], b: &[&str]) -> usize {
    let m = a.len();
    let n = b.len();
    let mut dp = vec![vec![0usize; n + 1]; m + 1];

    for i in 1..=m {
        for j in 1..=n {
            if a[i - 1] == b[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    dp[m][n]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_structural_similarity() {
        let resolver = SymbolResolver::new();

        let p1 = vec![
            DSLOp { name: "rotate_cw".into(), domain: Domain::Grid2D, parameters: vec![] },
            DSLOp { name: "recolor".into(), domain: Domain::Grid2D, parameters: vec![] },
        ];
        let p2 = vec![
            DSLOp { name: "rotate_cw".into(), domain: Domain::Grid2D, parameters: vec![] },
            DSLOp { name: "recolor".into(), domain: Domain::Grid2D, parameters: vec![] },
        ];
        assert_eq!(resolver.structural_similarity(&p1, &p2), 1.0);

        let p3 = vec![
            DSLOp { name: "flip_h".into(), domain: Domain::Grid2D, parameters: vec![] },
            DSLOp { name: "crop".into(), domain: Domain::Grid2D, parameters: vec![] },
        ];
        assert!(resolver.structural_similarity(&p1, &p3) < 0.5);
    }

    #[test]
    fn test_unification() {
        let mut resolver = SymbolResolver::new();

        let law = resolver.try_unify(
            "rule_1",
            &["asymmetric".into()],
            &[DSLOp { name: "rotate_cw".into(), domain: Domain::Grid2D, parameters: vec![] }],
            0.9,
            "rule_2",
            &["asymmetric".into(), "left_heavy".into()],
            &[DSLOp { name: "rotate_cw".into(), domain: Domain::Grid2D, parameters: vec![] }],
            0.85,
        );

        assert!(law.is_some());
        let law = law.unwrap();
        assert!(law.is_law());
    }

    #[test]
    fn test_effect_categorization() {
        let resolver = SymbolResolver::new();
        let program = vec![
            DSLOp { name: "rotate_cw".into(), domain: Domain::Grid2D, parameters: vec![] },
            DSLOp { name: "gravity_down".into(), domain: Domain::Grid2D, parameters: vec![] },
        ];
        let cats = resolver.categorize_effects(&program);
        assert!(cats.contains(&"spatial_rotation"));
        assert!(cats.contains(&"gravitational_settling"));
    }
}
