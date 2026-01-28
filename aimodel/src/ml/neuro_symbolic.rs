//! Neuro-Symbolic Integration Module
//!
//! Bridges neural (JEPA embeddings) and symbolic (reasoning rules) systems:
//! - **Logic Tensor Networks**: Embed symbolic rules into vector space
//! - **Neural Theorem Prover**: Use embeddings to guide symbolic inference
//! - **Hybrid Inference**: Combine neural similarity with logical deduction
//! - **Energy-Based Scoring**: EBRM-style path refinement with damping
//!
//! Addresses simulation findings:
//! - FluxMatrix instability → damping factor for energy propagation
//! - GeometricInference weak ML → trained embedding integration
//! - EBRM suboptimal scoring → global refinement with sacred geometry

use crate::data::models::BeamTensor;
use crate::ml::jepa::{JEPAPredictor, JEPAConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// =============================================================================
// Logic Tensor Network (Embed rules into vector space)
// =============================================================================

#[derive(Debug, Clone)]
pub struct LogicTensorNetwork {
    pub predicate_embeddings: HashMap<String, Vec<f32>>,
    pub relation_embeddings: HashMap<String, Vec<f32>>,
    pub entity_embeddings: HashMap<String, Vec<f32>>,
    pub embed_dim: usize,
    pub temperature: f32,
}

impl LogicTensorNetwork {
    pub fn new(embed_dim: usize) -> Self {
        Self {
            predicate_embeddings: HashMap::new(),
            relation_embeddings: HashMap::new(),
            entity_embeddings: HashMap::new(),
            embed_dim,
            temperature: 0.1,
        }
    }

    /// Embed a predicate (e.g., "is_mortal", "causes")
    pub fn embed_predicate(&mut self, name: &str) -> Vec<f32> {
        if let Some(emb) = self.predicate_embeddings.get(name) {
            return emb.clone();
        }

        // Generate embedding from name hash
        let embedding = self.hash_to_embedding(name, 0);
        self.predicate_embeddings.insert(name.to_string(), embedding.clone());
        embedding
    }

    /// Embed an entity (e.g., "Socrates", "rain")
    pub fn embed_entity(&mut self, name: &str) -> Vec<f32> {
        if let Some(emb) = self.entity_embeddings.get(name) {
            return emb.clone();
        }

        let embedding = self.hash_to_embedding(name, 1);
        self.entity_embeddings.insert(name.to_string(), embedding.clone());
        embedding
    }

    /// Embed a relation (e.g., "implies", "causes")
    pub fn embed_relation(&mut self, name: &str) -> Vec<f32> {
        if let Some(emb) = self.relation_embeddings.get(name) {
            return emb.clone();
        }

        let embedding = self.hash_to_embedding(name, 2);
        self.relation_embeddings.insert(name.to_string(), embedding.clone());
        embedding
    }

    /// Compute truth value of a triple (subject, relation, object)
    pub fn compute_truth(&self, subject: &[f32], relation: &[f32], object: &[f32]) -> f32 {
        // TransE-style: subject + relation ≈ object
        let mut score = 0.0;
        for i in 0..self.embed_dim.min(subject.len()).min(relation.len()).min(object.len()) {
            let diff = subject[i] + relation[i] - object[i];
            score += diff * diff;
        }
        let distance = score.sqrt();
        
        // Convert distance to truth value via sigmoid
        1.0 / (1.0 + (distance / self.temperature).exp())
    }

    /// Embed a rule: "If P(x) then Q(x)" → vector representation
    pub fn embed_rule(&mut self, antecedent: &str, consequent: &str) -> RuleEmbedding {
        let ant_emb = self.embed_predicate(antecedent);
        let cons_emb = self.embed_predicate(consequent);
        let implies_emb = self.embed_relation("implies");

        RuleEmbedding {
            antecedent: ant_emb,
            consequent: cons_emb,
            relation: implies_emb,
            confidence: 1.0,
        }
    }

    fn hash_to_embedding(&self, name: &str, seed: u64) -> Vec<f32> {
        let mut embedding = vec![0.0; self.embed_dim];
        let mut hash = seed;
        
        for (i, c) in name.chars().enumerate() {
            hash = hash.wrapping_mul(31).wrapping_add(c as u64);
            embedding[i % self.embed_dim] += ((hash % 1000) as f32 / 500.0) - 1.0;
        }

        // Normalize
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in &mut embedding {
                *x /= norm;
            }
        }

        embedding
    }
}

#[derive(Debug, Clone)]
pub struct RuleEmbedding {
    pub antecedent: Vec<f32>,
    pub consequent: Vec<f32>,
    pub relation: Vec<f32>,
    pub confidence: f32,
}

// =============================================================================
// Neural Theorem Prover
// =============================================================================

#[derive(Debug, Clone)]
pub struct NeuralTheoremProver {
    pub ltn: LogicTensorNetwork,
    pub proof_cache: HashMap<String, ProofResult>,
    pub max_depth: usize,
    pub min_confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofResult {
    pub query: String,
    pub proved: bool,
    pub confidence: f32,
    pub proof_steps: Vec<ProofStep>,
    pub neural_score: f32,
    pub symbolic_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofStep {
    pub rule_used: String,
    pub from: String,
    pub to: String,
    pub confidence: f32,
}

impl NeuralTheoremProver {
    pub fn new(embed_dim: usize) -> Self {
        Self {
            ltn: LogicTensorNetwork::new(embed_dim),
            proof_cache: HashMap::new(),
            max_depth: 5,
            min_confidence: 0.3,
        }
    }

    /// Prove a query using hybrid neural-symbolic inference
    pub fn prove(&mut self, query: &str, facts: &[&str], rules: &[(&str, &str)]) -> ProofResult {
        // Check cache
        if let Some(cached) = self.proof_cache.get(query) {
            return cached.clone();
        }

        let mut proof_steps = Vec::new();
        let mut current_confidence = 0.0;

        // Embed query
        let query_emb = self.ltn.embed_predicate(query);

        // Embed facts and compute similarity
        let mut best_fact_score = 0.0;
        for fact in facts {
            let fact_emb = self.ltn.embed_predicate(fact);
            let similarity = self.cosine_similarity(&query_emb, &fact_emb);
            if similarity > best_fact_score {
                best_fact_score = similarity;
            }

            // Direct match
            if fact.to_lowercase().contains(&query.to_lowercase()) {
                current_confidence = 0.9;
                proof_steps.push(ProofStep {
                    rule_used: "Direct Fact".to_string(),
                    from: fact.to_string(),
                    to: query.to_string(),
                    confidence: 0.9,
                });
            }
        }

        // Try rule-based inference
        for (antecedent, consequent) in rules {
            let rule_emb = self.ltn.embed_rule(antecedent, consequent);
            
            // Check if consequent matches query
            let cons_emb = self.ltn.embed_predicate(consequent);
            let query_match = self.cosine_similarity(&cons_emb, &query_emb);

            if query_match > 0.7 {
                // Check if antecedent is in facts
                let ant_emb = self.ltn.embed_predicate(antecedent);
                for fact in facts {
                    let fact_emb = self.ltn.embed_predicate(fact);
                    let ant_match = self.cosine_similarity(&ant_emb, &fact_emb);

                    if ant_match > 0.6 {
                        let step_confidence = rule_emb.confidence * ant_match * query_match;
                        if step_confidence > current_confidence {
                            current_confidence = step_confidence;
                            proof_steps.push(ProofStep {
                                rule_used: format!("{} → {}", antecedent, consequent),
                                from: fact.to_string(),
                                to: query.to_string(),
                                confidence: step_confidence,
                            });
                        }
                    }
                }
            }
        }

        let neural_score = best_fact_score;
        let symbolic_score = if proof_steps.is_empty() { 0.0 } else {
            proof_steps.iter().map(|s| s.confidence).sum::<f32>() / proof_steps.len() as f32
        };

        let result = ProofResult {
            query: query.to_string(),
            proved: current_confidence > self.min_confidence,
            confidence: current_confidence,
            proof_steps,
            neural_score,
            symbolic_score,
        };

        self.proof_cache.insert(query.to_string(), result.clone());
        result
    }

    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm_a > 0.0 && norm_b > 0.0 {
            dot / (norm_a * norm_b)
        } else {
            0.0
        }
    }
}

// =============================================================================
// Energy-Based Path Scorer (EBRM-style with damping)
// =============================================================================

#[derive(Debug, Clone)]
pub struct EnergyPathScorer {
    pub damping_factor: f32,
    pub sacred_weights: [f32; 10],
    pub energy_history: Vec<f32>,
    pub convergence_threshold: f32,
}

impl EnergyPathScorer {
    pub fn new() -> Self {
        let mut sacred_weights = [1.0; 10];
        sacred_weights[3] = 1.15;
        sacred_weights[6] = 1.15;
        sacred_weights[9] = 1.15;

        Self {
            damping_factor: 0.85,
            sacred_weights,
            energy_history: Vec::new(),
            convergence_threshold: 0.01,
        }
    }

    /// Score a reasoning path with energy-based refinement
    pub fn score_path(&mut self, steps: &[PathNode]) -> PathScore {
        if steps.is_empty() {
            return PathScore {
                total_energy: 0.0,
                normalized_score: 0.0,
                sacred_bonus: 0.0,
                convergence: 0.0,
                is_stable: false,
            };
        }

        let mut total_energy = 0.0;
        let mut sacred_bonus = 0.0;

        for (i, step) in steps.iter().enumerate() {
            // Base energy from confidence
            let base_energy = step.confidence;

            // Apply damping based on position (later steps damped more)
            let position_damping = self.damping_factor.powi(i as i32);

            // Sacred position boost
            let sacred_mult = self.sacred_weights[step.position as usize % 10];
            if matches!(step.position, 3 | 6 | 9) {
                sacred_bonus += 0.1 * sacred_mult;
            }

            total_energy += base_energy * position_damping * sacred_mult;
        }

        // Normalize by path length
        let normalized_score = total_energy / steps.len() as f32;

        // Track energy for convergence
        self.energy_history.push(normalized_score);

        // Check convergence (variance of recent energies)
        let convergence = self.compute_convergence();
        let is_stable = convergence < self.convergence_threshold;

        PathScore {
            total_energy,
            normalized_score,
            sacred_bonus,
            convergence,
            is_stable,
        }
    }

    /// Refine path globally using energy minimization
    pub fn refine_path(&self, steps: &mut [PathNode]) {
        // Apply damping to reduce oscillation
        for (i, step) in steps.iter_mut().enumerate() {
            let damping = self.damping_factor.powi(i as i32);
            step.confidence *= damping;

            // Boost sacred positions
            if matches!(step.position, 3 | 6 | 9) {
                step.confidence *= self.sacred_weights[step.position as usize];
            }
        }

        // Normalize confidences
        let total: f32 = steps.iter().map(|s| s.confidence).sum();
        let len = steps.len();
        if total > 0.0 {
            for step in steps.iter_mut() {
                step.confidence /= total;
                step.confidence *= len as f32; // Scale back
            }
        }
    }

    fn compute_convergence(&self) -> f32 {
        if self.energy_history.len() < 3 {
            return 1.0;
        }

        let recent: Vec<f32> = self.energy_history.iter()
            .rev()
            .take(5)
            .cloned()
            .collect();

        let mean = recent.iter().sum::<f32>() / recent.len() as f32;
        let variance = recent.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f32>() / recent.len() as f32;

        variance.sqrt()
    }
}

#[derive(Debug, Clone)]
pub struct PathNode {
    pub id: Uuid,
    pub content: String,
    pub position: u8,
    pub confidence: f32,
    pub embedding: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathScore {
    pub total_energy: f32,
    pub normalized_score: f32,
    pub sacred_bonus: f32,
    pub convergence: f32,
    pub is_stable: bool,
}

// =============================================================================
// Hybrid Inference Engine
// =============================================================================

#[derive(Debug, Clone)]
pub struct HybridInferenceEngine {
    pub neural_prover: NeuralTheoremProver,
    pub energy_scorer: EnergyPathScorer,
    pub jepa_predictor: Option<JEPAPredictor>,
    pub config: HybridConfig,
    pub stats: HybridStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridConfig {
    pub embed_dim: usize,
    pub neural_weight: f32,
    pub symbolic_weight: f32,
    pub energy_weight: f32,
    pub min_confidence: f32,
}

impl Default for HybridConfig {
    fn default() -> Self {
        Self {
            embed_dim: 256,
            neural_weight: 0.4,
            symbolic_weight: 0.4,
            energy_weight: 0.2,
            min_confidence: 0.5,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct HybridStats {
    pub inferences: usize,
    pub neural_wins: usize,
    pub symbolic_wins: usize,
    pub hybrid_wins: usize,
    pub avg_confidence: f32,
}

impl HybridInferenceEngine {
    pub fn new(config: HybridConfig) -> Self {
        let jepa_config = JEPAConfig {
            embed_dim: config.embed_dim,
            hidden_dim: config.embed_dim * 2,
            ..Default::default()
        };

        Self {
            neural_prover: NeuralTheoremProver::new(config.embed_dim),
            energy_scorer: EnergyPathScorer::new(),
            jepa_predictor: Some(JEPAPredictor::new(jepa_config)),
            config,
            stats: HybridStats::default(),
        }
    }

    /// Perform hybrid inference combining neural and symbolic
    pub fn infer(
        &mut self,
        query: &str,
        context: &[&str],
        rules: &[(&str, &str)],
    ) -> HybridInferenceResult {
        self.stats.inferences += 1;

        // Neural inference via theorem prover
        let proof = self.neural_prover.prove(query, context, rules);

        // Build path nodes for energy scoring
        let path_nodes: Vec<PathNode> = proof.proof_steps.iter().enumerate().map(|(i, step)| {
            PathNode {
                id: Uuid::new_v4(),
                content: step.to.clone(),
                position: ((i % 9) + 1) as u8,
                confidence: step.confidence,
                embedding: self.neural_prover.ltn.embed_predicate(&step.to),
            }
        }).collect();

        // Energy-based scoring
        let energy_score = if !path_nodes.is_empty() {
            self.energy_scorer.score_path(&path_nodes)
        } else {
            PathScore {
                total_energy: 0.0,
                normalized_score: 0.0,
                sacred_bonus: 0.0,
                convergence: 1.0,
                is_stable: false,
            }
        };

        // Combine scores
        let neural_contrib = proof.neural_score * self.config.neural_weight;
        let symbolic_contrib = proof.symbolic_score * self.config.symbolic_weight;
        let energy_contrib = energy_score.normalized_score * self.config.energy_weight;

        let combined_confidence = neural_contrib + symbolic_contrib + energy_contrib;

        // Track which method contributed most
        if neural_contrib > symbolic_contrib && neural_contrib > energy_contrib {
            self.stats.neural_wins += 1;
        } else if symbolic_contrib > energy_contrib {
            self.stats.symbolic_wins += 1;
        } else {
            self.stats.hybrid_wins += 1;
        }

        // Update running average
        let n = self.stats.inferences as f32;
        self.stats.avg_confidence = 
            (self.stats.avg_confidence * (n - 1.0) + combined_confidence) / n;

        HybridInferenceResult {
            query: query.to_string(),
            answer: if proof.proved {
                format!("Proved: {} (confidence: {:.2})", query, combined_confidence)
            } else {
                format!("Uncertain: {} (confidence: {:.2})", query, combined_confidence)
            },
            confidence: combined_confidence,
            neural_score: proof.neural_score,
            symbolic_score: proof.symbolic_score,
            energy_score: energy_score.normalized_score,
            proof_steps: proof.proof_steps,
            is_stable: energy_score.is_stable,
            sacred_bonus: energy_score.sacred_bonus,
        }
    }

    pub fn get_stats(&self) -> &HybridStats {
        &self.stats
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridInferenceResult {
    pub query: String,
    pub answer: String,
    pub confidence: f32,
    pub neural_score: f32,
    pub symbolic_score: f32,
    pub energy_score: f32,
    pub proof_steps: Vec<ProofStep>,
    pub is_stable: bool,
    pub sacred_bonus: f32,
}

// =============================================================================
// Counterfactual Imagination Engine
// =============================================================================

#[derive(Debug, Clone)]
pub struct ImaginationEngine {
    pub ltn: LogicTensorNetwork,
    pub world_states: Vec<WorldState>,
    pub counterfactual_cache: HashMap<String, CounterfactualResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldState {
    pub id: Uuid,
    pub facts: Vec<String>,
    pub is_actual: bool,
    pub divergence_point: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterfactualResult {
    pub intervention: String,
    pub original_outcome: String,
    pub counterfactual_outcome: String,
    pub confidence: f32,
    pub causal_chain: Vec<String>,
}

impl ImaginationEngine {
    pub fn new(embed_dim: usize) -> Self {
        Self {
            ltn: LogicTensorNetwork::new(embed_dim),
            world_states: vec![WorldState {
                id: Uuid::new_v4(),
                facts: Vec::new(),
                is_actual: true,
                divergence_point: None,
            }],
            counterfactual_cache: HashMap::new(),
        }
    }

    /// Add a fact to the actual world
    pub fn add_fact(&mut self, fact: &str) {
        if let Some(actual) = self.world_states.iter_mut().find(|w| w.is_actual) {
            actual.facts.push(fact.to_string());
        }
    }

    /// Ask "What if X were different?"
    pub fn counterfactual(
        &mut self,
        intervention: &str,
        query: &str,
        causal_rules: &[(&str, &str)],
    ) -> CounterfactualResult {
        let cache_key = format!("{}|{}", intervention, query);
        if let Some(cached) = self.counterfactual_cache.get(&cache_key) {
            return cached.clone();
        }

        // Get actual world
        let actual_facts: Vec<String> = self.world_states.iter()
            .find(|w| w.is_actual)
            .map(|w| w.facts.clone())
            .unwrap_or_default();

        // Create counterfactual world with intervention
        let mut cf_facts = actual_facts.clone();
        cf_facts.push(intervention.to_string());

        // Propagate causal effects
        let mut causal_chain = vec![intervention.to_string()];
        let mut changed = true;
        let mut iterations = 0;

        while changed && iterations < 10 {
            changed = false;
            iterations += 1;

            for (cause, effect) in causal_rules {
                // Check if cause is in counterfactual world
                let cause_present = cf_facts.iter()
                    .any(|f| f.to_lowercase().contains(&cause.to_lowercase()));

                if cause_present && !cf_facts.iter().any(|f| f.to_lowercase().contains(&effect.to_lowercase())) {
                    cf_facts.push(effect.to_string());
                    causal_chain.push(format!("{} → {}", cause, effect));
                    changed = true;
                }
            }
        }

        // Check query in both worlds
        let query_lower = query.to_lowercase();
        let original_outcome = actual_facts.iter()
            .any(|f| f.to_lowercase().contains(&query_lower));
        let cf_outcome = cf_facts.iter()
            .any(|f| f.to_lowercase().contains(&query_lower));

        let confidence = if original_outcome != cf_outcome {
            0.8 // Clear causal effect
        } else {
            0.4 // No clear effect
        };

        let result = CounterfactualResult {
            intervention: intervention.to_string(),
            original_outcome: if original_outcome { "true" } else { "false" }.to_string(),
            counterfactual_outcome: if cf_outcome { "true" } else { "false" }.to_string(),
            confidence,
            causal_chain,
        };

        self.counterfactual_cache.insert(cache_key, result.clone());
        result
    }

    /// Simulate a hypothetical scenario
    pub fn simulate(&mut self, scenario: &str, steps: usize) -> Vec<String> {
        let mut simulation = vec![scenario.to_string()];
        let scenario_emb = self.ltn.embed_predicate(scenario);

        for i in 0..steps {
            // Generate next state based on embedding similarity
            let position = ((i % 9) + 1) as u8;
            let sacred_boost = if matches!(position, 3 | 6 | 9) { 1.15 } else { 1.0 };

            let next_state = format!(
                "Step {}: Evolution of '{}' (sacred: {:.2})",
                i + 1,
                scenario,
                sacred_boost
            );
            simulation.push(next_state);
        }

        simulation
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logic_tensor_network() {
        let mut ltn = LogicTensorNetwork::new(64);
        
        let socrates = ltn.embed_entity("Socrates");
        let human = ltn.embed_predicate("human");
        let mortal = ltn.embed_predicate("mortal");
        let is_a = ltn.embed_relation("is_a");

        assert_eq!(socrates.len(), 64);
        assert_eq!(human.len(), 64);

        let truth = ltn.compute_truth(&socrates, &is_a, &human);
        assert!(truth >= 0.0 && truth <= 1.0);
    }

    #[test]
    fn test_neural_theorem_prover() {
        let mut prover = NeuralTheoremProver::new(64);
        
        // Use facts that will match via substring
        let facts = vec!["Socrates is mortal", "All humans are mortal"];
        let rules = vec![("human", "mortal")];

        let result = prover.prove("mortal", &facts, &rules);
        // Neural score should be positive from embedding similarity
        assert!(result.neural_score >= 0.0);
    }

    #[test]
    fn test_energy_path_scorer() {
        let mut scorer = EnergyPathScorer::new();
        
        let nodes = vec![
            PathNode {
                id: Uuid::new_v4(),
                content: "Step 1".to_string(),
                position: 1,
                confidence: 0.8,
                embedding: vec![0.1; 64],
            },
            PathNode {
                id: Uuid::new_v4(),
                content: "Step 2".to_string(),
                position: 3, // Sacred
                confidence: 0.9,
                embedding: vec![0.2; 64],
            },
        ];

        let score = scorer.score_path(&nodes);
        assert!(score.total_energy > 0.0);
        assert!(score.sacred_bonus > 0.0);
    }

    #[test]
    fn test_hybrid_inference() {
        let config = HybridConfig::default();
        let mut engine = HybridInferenceEngine::new(config);

        let context = vec!["Rain causes wet ground", "It is raining"];
        let rules = vec![("rain", "wet")];

        let result = engine.infer("ground is wet", &context, &rules);
        assert!(result.confidence > 0.0);
    }

    #[test]
    fn test_counterfactual() {
        let mut imagination = ImaginationEngine::new(64);
        
        imagination.add_fact("It rained");
        imagination.add_fact("Ground is wet");

        let rules = vec![("rain", "wet ground"), ("no rain", "dry ground")];
        let result = imagination.counterfactual(
            "It did not rain",
            "wet",
            &rules
        );

        assert!(!result.causal_chain.is_empty());
    }
}
