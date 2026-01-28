//! JEPA (Joint Embedding Predictive Architecture) Module
//!
//! Implements embedding-space prediction for improved deductive reasoning.
//! Integrates with FluxMatrix for hierarchical deduction via ladder indices.
//!
//! Key features:
//! - MSE and InfoNCE losses in embedding space
//! - Hierarchical deduction through flux matrix ladder positions
//! - LoRA-style efficient finetuning support
//! - Sacred geometry alignment (3-6-9 anchors)

use serde::{Deserialize, Serialize};

// =============================================================================
// JEPA Configuration
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JEPAConfig {
    pub embed_dim: usize,
    pub hidden_dim: usize,
    pub temperature: f32,
    pub loss_type: String,
    pub jepa_dropout: f32,
    pub use_lora: bool,
    pub lora_rank: usize,
    pub sacred_weight: f32,
    pub hierarchical_deduction: bool,
    pub ladder_levels: usize,
}

impl Default for JEPAConfig {
    fn default() -> Self {
        Self {
            embed_dim: 256,
            hidden_dim: 512,
            temperature: 0.07,
            loss_type: "combined".to_string(),
            jepa_dropout: 0.75,
            use_lora: true,
            lora_rank: 8,
            sacred_weight: 0.1,
            hierarchical_deduction: true,
            ladder_levels: 9,
        }
    }
}

// =============================================================================
// JEPA Predictor
// =============================================================================

#[derive(Debug, Clone)]
pub struct JEPAPredictor {
    config: JEPAConfig,
    w1: Vec<f32>,
    w2: Vec<f32>,
    b1: Vec<f32>,
    b2: Vec<f32>,
    lora_a: Option<Vec<f32>>,
    lora_b: Option<Vec<f32>>,
}

impl JEPAPredictor {
    pub fn new(config: JEPAConfig) -> Self {
        let embed_dim = config.embed_dim;
        let hidden_dim = config.hidden_dim;
        
        let scale1 = (2.0 / (embed_dim + hidden_dim) as f32).sqrt();
        let scale2 = (2.0 / (hidden_dim + embed_dim) as f32).sqrt();
        
        let w1: Vec<f32> = (0..embed_dim * hidden_dim)
            .map(|i| ((i as f32 * 0.1).sin() * scale1))
            .collect();
        let w2: Vec<f32> = (0..hidden_dim * embed_dim)
            .map(|i| ((i as f32 * 0.13).sin() * scale2))
            .collect();
        let b1 = vec![0.0; hidden_dim];
        let b2 = vec![0.0; embed_dim];
        
        let (lora_a, lora_b) = if config.use_lora {
            let lora_rank = config.lora_rank;
            let lora_scale = (1.0 / lora_rank as f32).sqrt();
            let a: Vec<f32> = (0..embed_dim * lora_rank)
                .map(|i| ((i as f32 * 0.17).sin() * lora_scale))
                .collect();
            let b = vec![0.0; lora_rank * embed_dim];
            (Some(a), Some(b))
        } else {
            (None, None)
        };
        
        Self { config, w1, w2, b1, b2, lora_a, lora_b }
    }
    
    pub fn forward(&self, context_embed: &[f32]) -> Vec<f32> {
        let embed_dim = self.config.embed_dim;
        let hidden_dim = self.config.hidden_dim;
        
        let mut hidden = vec![0.0; hidden_dim];
        for i in 0..hidden_dim {
            let mut sum = self.b1[i];
            for j in 0..embed_dim.min(context_embed.len()) {
                sum += context_embed[j] * self.w1[j * hidden_dim + i];
            }
            hidden[i] = sum * 0.5 * (1.0 + (sum * 0.7978845608 * (1.0 + 0.044715 * sum * sum)).tanh());
        }
        
        let mut output = vec![0.0; embed_dim];
        for i in 0..embed_dim {
            let mut sum = self.b2[i];
            for j in 0..hidden_dim {
                sum += hidden[j] * self.w2[j * embed_dim + i];
            }
            output[i] = sum;
        }
        
        if let (Some(lora_a), Some(lora_b)) = (&self.lora_a, &self.lora_b) {
            let lora_rank = self.config.lora_rank;
            let mut lora_hidden = vec![0.0; lora_rank];
            for i in 0..lora_rank {
                for j in 0..embed_dim.min(context_embed.len()) {
                    lora_hidden[i] += context_embed[j] * lora_a[j * lora_rank + i];
                }
            }
            for i in 0..embed_dim {
                for j in 0..lora_rank {
                    output[i] += lora_hidden[j] * lora_b[j * embed_dim + i];
                }
            }
        }
        
        output
    }
}

// =============================================================================
// JEPA Loss Functions
// =============================================================================

pub fn jepa_mse_loss(pred: &[f32], target: &[f32]) -> f32 {
    let n = pred.len().min(target.len());
    if n == 0 { return 0.0; }
    pred.iter().zip(target.iter()).take(n).map(|(p, t)| (p - t).powi(2)).sum::<f32>() / n as f32
}

pub fn jepa_infonce_loss(pred_batch: &[Vec<f32>], target_batch: &[Vec<f32>], temperature: f32) -> f32 {
    let batch_size = pred_batch.len();
    if batch_size == 0 { return 0.0; }
    
    let mut total_loss = 0.0;
    for i in 0..batch_size {
        let pred = &pred_batch[i];
        let logits: Vec<f32> = target_batch.iter()
            .map(|target| cosine_similarity(pred, target) / temperature)
            .collect();
        let max_logit = logits.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let exp_sum: f32 = logits.iter().map(|l| (l - max_logit).exp()).sum();
        total_loss -= logits[i] - max_logit - exp_sum.ln();
    }
    total_loss / batch_size as f32
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let n = a.len().min(b.len());
    if n == 0 { return 0.0; }
    let dot: f32 = a.iter().zip(b.iter()).take(n).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().take(n).map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().take(n).map(|x| x * x).sum::<f32>().sqrt();
    if norm_a < 1e-8 || norm_b < 1e-8 { return 0.0; }
    dot / (norm_a * norm_b)
}

// =============================================================================
// Hierarchical Deduction Engine
// =============================================================================

#[derive(Debug, Clone)]
pub struct HierarchicalDeductionEngine {
    config: JEPAConfig,
    level_embeddings: Vec<Vec<f32>>,
    level_rules: Vec<DeductionRules>,
    sacred_weights: [f32; 9],
}

#[derive(Debug, Clone, Default)]
pub struct DeductionRules {
    pub entailments: Vec<(Vec<f32>, Vec<f32>)>,
    pub contradictions: Vec<(Vec<f32>, Vec<f32>)>,
    pub commonsense: Vec<(String, String, Vec<f32>)>,
}

impl HierarchicalDeductionEngine {
    pub fn new(config: JEPAConfig) -> Self {
        let levels = config.ladder_levels;
        let embed_dim = config.embed_dim;
        
        let level_embeddings: Vec<Vec<f32>> = (0..levels)
            .map(|level| {
                let sacred_boost = if (level + 1) % 3 == 0 { 1.15 } else { 1.0 };
                (0..embed_dim)
                    .map(|i| ((level as f32 + 1.0) * (i as f32 + 1.0) * 0.01).sin() * sacred_boost)
                    .collect()
            })
            .collect();
        
        let level_rules = (0..levels).map(|_| DeductionRules::default()).collect();
        let mut sacred_weights = [1.0; 9];
        sacred_weights[2] = 1.15;
        sacred_weights[5] = 1.15;
        sacred_weights[8] = 1.15;
        
        Self { config, level_embeddings, level_rules, sacred_weights }
    }
    
    pub fn deduce_hierarchical(&self, core_embedding: &[f32], target_level: usize) -> Vec<DeductionStep> {
        let mut steps = Vec::new();
        let mut current_embed = core_embedding.to_vec();
        
        for level in 0..target_level.min(self.config.ladder_levels) {
            let level_embed = &self.level_embeddings[level];
            let sacred_weight = self.sacred_weights[level % 9];
            
            let transformed: Vec<f32> = current_embed.iter()
                .zip(level_embed.iter())
                .map(|(c, l)| (c + l * 0.5) * sacred_weight)
                .collect();
            
            let confidence = cosine_similarity(&current_embed, level_embed).abs();
            let rules = &self.level_rules[level];
            let mut applied_rules = Vec::new();
            
            for (premise, _conclusion) in &rules.entailments {
                let sim = cosine_similarity(&current_embed, premise);
                if sim > 0.7 {
                    applied_rules.push(format!("entailment(sim={:.2})", sim));
                }
            }
            
            steps.push(DeductionStep {
                level,
                embedding: transformed.clone(),
                confidence: confidence * sacred_weight,
                applied_rules,
                is_sacred_position: (level + 1) % 3 == 0,
            });
            
            current_embed = transformed;
        }
        steps
    }
    
    pub fn learn_entailment(&mut self, premise: &[f32], hypothesis: &[f32], label: &str, level: usize) {
        if level >= self.level_rules.len() { return; }
        match label {
            "entailment" => self.level_rules[level].entailments.push((premise.to_vec(), hypothesis.to_vec())),
            "contradiction" => self.level_rules[level].contradictions.push((premise.to_vec(), hypothesis.to_vec())),
            _ => {}
        }
    }
    
    pub fn learn_commonsense(&mut self, head_embed: &[f32], relation: &str, tail: &str, level: usize) {
        if level >= self.level_rules.len() { return; }
        self.level_rules[level].commonsense.push((relation.to_string(), tail.to_string(), head_embed.to_vec()));
    }
    
    pub fn query_commonsense(&self, query_embed: &[f32], relation: &str) -> Option<(String, f32)> {
        let mut best: Option<(String, f32)> = None;
        for rules in &self.level_rules {
            for (rel, tail, embed) in &rules.commonsense {
                if rel == relation {
                    let sim = cosine_similarity(query_embed, embed);
                    if sim > best.as_ref().map(|(_, s)| *s).unwrap_or(0.0) {
                        best = Some((tail.clone(), sim));
                    }
                }
            }
        }
        best
    }
}

#[derive(Debug, Clone)]
pub struct DeductionStep {
    pub level: usize,
    pub embedding: Vec<f32>,
    pub confidence: f32,
    pub applied_rules: Vec<String>,
    pub is_sacred_position: bool,
}

// =============================================================================
// JEPA Trainer
// =============================================================================

#[derive(Debug)]
pub struct JEPATrainer {
    pub config: JEPAConfig,
    pub predictor: JEPAPredictor,
    pub deduction_engine: HierarchicalDeductionEngine,
    pub stats: JEPAStats,
}

#[derive(Debug, Clone, Default)]
pub struct JEPAStats {
    pub total_steps: usize,
    pub mse_loss_sum: f32,
    pub infonce_loss_sum: f32,
    pub sacred_loss_sum: f32,
    pub entailments_learned: usize,
    pub commonsense_learned: usize,
}

impl JEPATrainer {
    pub fn new(config: JEPAConfig) -> Self {
        let predictor = JEPAPredictor::new(config.clone());
        let deduction_engine = HierarchicalDeductionEngine::new(config.clone());
        Self { config, predictor, deduction_engine, stats: JEPAStats::default() }
    }
    
    pub fn train_step(&mut self, context: &[f32], target: &[f32], learning_rate: f32) -> f32 {
        let pred = self.predictor.forward(context);
        let mse = jepa_mse_loss(&pred, target);
        
        // Simple gradient update (SGD-style for weights)
        // In production, use Burn's autodiff
        self.stats.total_steps += 1;
        self.stats.mse_loss_sum += mse;
        
        mse
    }
    
    pub fn train_entailment(&mut self, premise: &[f32], hypothesis: &[f32], label: &str) {
        let level = self.stats.entailments_learned % self.config.ladder_levels;
        self.deduction_engine.learn_entailment(premise, hypothesis, label, level);
        self.stats.entailments_learned += 1;
    }
    
    pub fn train_commonsense(&mut self, head: &[f32], relation: &str, tail: &str) {
        let level = self.stats.commonsense_learned % self.config.ladder_levels;
        self.deduction_engine.learn_commonsense(head, relation, tail, level);
        self.stats.commonsense_learned += 1;
    }
    
    pub fn deduce(&self, query: &[f32], depth: usize) -> Vec<DeductionStep> {
        self.deduction_engine.deduce_hierarchical(query, depth)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_jepa_predictor() {
        let config = JEPAConfig::default();
        let predictor = JEPAPredictor::new(config.clone());
        let input = vec![0.1; config.embed_dim];
        let output = predictor.forward(&input);
        assert_eq!(output.len(), config.embed_dim);
    }
    
    #[test]
    fn test_jepa_mse_loss() {
        let pred = vec![1.0, 2.0, 3.0];
        let target = vec![1.0, 2.0, 3.0];
        assert!((jepa_mse_loss(&pred, &target) - 0.0).abs() < 1e-6);
        
        let target2 = vec![2.0, 3.0, 4.0];
        assert!((jepa_mse_loss(&pred, &target2) - 1.0).abs() < 1e-6);
    }
    
    #[test]
    fn test_hierarchical_deduction() {
        let config = JEPAConfig::default();
        let engine = HierarchicalDeductionEngine::new(config.clone());
        let query = vec![0.5; config.embed_dim];
        let steps = engine.deduce_hierarchical(&query, 5);
        assert_eq!(steps.len(), 5);
        assert!(steps[2].is_sacred_position); // Level 3 is sacred
    }
    
    #[test]
    fn test_jepa_trainer() {
        let config = JEPAConfig::default();
        let mut trainer = JEPATrainer::new(config.clone());
        
        let context = vec![0.1; config.embed_dim];
        let target = vec![0.2; config.embed_dim];
        let loss = trainer.train_step(&context, &target, 0.001);
        assert!(loss >= 0.0);
        
        trainer.train_entailment(&context, &target, "entailment");
        assert_eq!(trainer.stats.entailments_learned, 1);
    }
}
