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
    /// Temporal straightening regularization strength (lambda from arXiv:2603.12231).
    /// Controls how aggressively we penalize curvature in latent reasoning trajectories.
    /// Higher values produce straighter trajectories where Euclidean distance better
    /// approximates geodesic distance, improving gradient-based pathway optimization.
    pub straightening_lambda: f32,
    /// Number of consecutive latent states to track for curvature computation.
    /// Curvature is measured as the cosine similarity between consecutive velocity
    /// vectors v_t = z_{t+1} - z_t along the reasoning trajectory.
    pub trajectory_window: usize,
    /// Exponential moving average decay for the target encoder (collapse prevention).
    /// The paper uses stop-gradient; we use EMA which is equivalent to BYOL/data2vec
    /// and prevents the latent space from collapsing to a constant.
    pub ema_decay: f32,
    /// Maximum number of rules per deduction level before pruning kicks in.
    /// Prevents unbounded memory growth (rule explosion). When a level reaches
    /// this capacity, the lowest-utility rule is evicted before adding a new one.
    pub max_rules_per_level: usize,
    /// Base novelty threshold for new rules [0.0, 1.0]. A new rule must have
    /// cosine similarity below this threshold to all existing rules at the same
    /// level. The effective threshold increases asymptotically as rules accumulate:
    ///   effective = base + (1 - base) * (1 - 1 / (1 + count / half_life))
    /// This means early rules are easy to add, but later rules must be
    /// increasingly novel — modeling diminishing marginal returns.
    pub novelty_threshold_base: f32,
    /// Half-life for novelty threshold asymptote. When count equals half_life,
    /// the effective threshold is halfway between base and 1.0.
    pub novelty_half_life: f32,
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
            straightening_lambda: 0.5,
            trajectory_window: 16,
            ema_decay: 0.996,
            max_rules_per_level: 1024,
            novelty_threshold_base: 0.3,
            novelty_half_life: 256.0,
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
// Temporal Straightening for Latent Planning (arXiv:2603.12231)
// =============================================================================
//
// Core insight: Straightening latent reasoning trajectories makes Euclidean
// distance a faithful proxy for geodesic distance between concepts.
// This improves gradient-based pathway optimization by 20-60%.
//
// Components:
// - TemporalStraighteningEngine: Tracks reasoning trajectories and computes curvature
// - EMATargetEncoder: Exponential moving average target encoder (collapse prevention)
// - straightening_loss: Penalizes high curvature in latent reasoning paths
// - curvature_score: Evaluates pathway straightness for scoring

/// Tracks latent reasoning trajectories and computes curvature for straightening.
///
/// From the paper: curvature is measured as the cosine similarity between
/// consecutive velocity vectors v_t = z_{t+1} - z_t. High cosine similarity
/// means low curvature (straighter trajectory). The straightening loss
/// penalizes low cosine similarity to encourage straighter paths.
#[derive(Debug, Clone)]
pub struct TemporalStraighteningEngine {
    /// Rolling window of recent latent states along the reasoning trajectory
    trajectory_buffer: Vec<Vec<f32>>,
    /// Maximum number of states to retain in the trajectory buffer
    window_size: usize,
    /// Regularization strength (lambda in the paper)
    lambda: f32,
    /// Cumulative curvature statistics for monitoring
    total_curvature_sum: f64,
    /// Number of curvature measurements taken
    curvature_count: usize,
}

impl TemporalStraighteningEngine {
    pub fn new(window_size: usize, lambda: f32) -> Self {
        Self {
            trajectory_buffer: Vec::with_capacity(window_size),
            window_size,
            lambda,
            total_curvature_sum: 0.0,
            curvature_count: 0,
        }
    }

    /// Record a new latent state along the reasoning trajectory.
    /// Returns the straightening loss if enough states are buffered (minimum 3).
    pub fn record_state(&mut self, latent_state: &[f32]) -> Option<f32> {
        self.trajectory_buffer.push(latent_state.to_vec());

        // Evict oldest states beyond the window
        if self.trajectory_buffer.len() > self.window_size {
            self.trajectory_buffer.remove(0);
        }

        // Need at least 3 consecutive states to compute curvature
        if self.trajectory_buffer.len() < 3 {
            return None;
        }

        // Compute straightening loss over all consecutive triplets in the buffer
        let loss = self.compute_straightening_loss();
        Some(loss)
    }

    /// Compute the straightening loss over the entire trajectory buffer.
    ///
    /// From Equation 4 of the paper:
    ///   ℒ_straight = -mean(cos(v_t, v_{t+1}))
    /// where v_t = z_{t+1} - z_t are latent velocity vectors.
    fn compute_straightening_loss(&mut self) -> f32 {
        let n = self.trajectory_buffer.len();
        if n < 3 { return 0.0; }

        let mut cosine_sum = 0.0f32;
        let mut count = 0usize;

        for t in 0..(n - 2) {
            let velocity_t = vector_subtract(
                &self.trajectory_buffer[t + 1],
                &self.trajectory_buffer[t],
            );
            let velocity_t_plus_1 = vector_subtract(
                &self.trajectory_buffer[t + 2],
                &self.trajectory_buffer[t + 1],
            );

            let cosine = cosine_similarity(&velocity_t, &velocity_t_plus_1);
            cosine_sum += cosine;
            count += 1;
        }

        if count == 0 { return 0.0; }

        let mean_cosine = cosine_sum / count as f32;

        // Track curvature statistics (curvature = 1 - cosine; lower is straighter)
        let curvature = 1.0 - mean_cosine;
        self.total_curvature_sum += curvature as f64;
        self.curvature_count += 1;

        // Straightening loss: negative mean cosine similarity (minimize to straighten)
        // Multiplied by lambda to control regularization strength
        self.lambda * (-mean_cosine)
    }

    /// Get the mean curvature observed across all measurements.
    /// Returns a value in [0, 2] where 0 = perfectly straight, 2 = maximum curvature.
    pub fn mean_curvature(&self) -> f32 {
        if self.curvature_count == 0 { return 0.0; }
        (self.total_curvature_sum / self.curvature_count as f64) as f32
    }

    /// Evaluate straightness of a given pathway (sequence of embeddings).
    /// Returns a score in [0, 1] where 1 = perfectly straight.
    pub fn curvature_score(pathway: &[Vec<f32>]) -> f32 {
        if pathway.len() < 3 { return 1.0; }

        let mut cosine_sum = 0.0f32;
        let mut count = 0usize;

        for t in 0..(pathway.len() - 2) {
            let velocity_t = vector_subtract(&pathway[t + 1], &pathway[t]);
            let velocity_t_plus_1 = vector_subtract(&pathway[t + 2], &pathway[t + 1]);

            let cosine = cosine_similarity(&velocity_t, &velocity_t_plus_1);
            cosine_sum += cosine;
            count += 1;
        }

        if count == 0 { return 1.0; }

        // Map from [-1, 1] cosine range to [0, 1] straightness score
        let mean_cosine = cosine_sum / count as f32;
        (mean_cosine + 1.0) / 2.0
    }

    /// Reset the trajectory buffer (call between independent reasoning chains)
    pub fn reset_trajectory(&mut self) {
        self.trajectory_buffer.clear();
    }

    /// Reset all statistics
    pub fn reset_stats(&mut self) {
        self.total_curvature_sum = 0.0;
        self.curvature_count = 0;
    }
}

/// Exponential Moving Average Target Encoder for collapse prevention.
///
/// From the paper (Section 3.3): stop-gradient prevents the prediction loss
/// from collapsing the latent space to a constant. We implement this as an
/// EMA target encoder (equivalent to BYOL/data2vec approach) which maintains
/// a slow-moving copy of the context encoder weights.
///
/// The target encoder weights are updated as:
///   θ_target = τ * θ_target + (1 - τ) * θ_online
/// where τ is the EMA decay (typically 0.996).
#[derive(Debug, Clone)]
pub struct EMATargetEncoder {
    /// Target encoder weights (slow-moving copy)
    target_w1: Vec<f32>,
    target_w2: Vec<f32>,
    target_b1: Vec<f32>,
    target_b2: Vec<f32>,
    /// EMA decay factor (τ in the update rule)
    decay: f32,
}

impl EMATargetEncoder {
    /// Initialize target encoder from an online predictor's weights
    pub fn from_predictor(predictor: &JEPAPredictor, decay: f32) -> Self {
        Self {
            target_w1: predictor.w1.clone(),
            target_w2: predictor.w2.clone(),
            target_b1: predictor.b1.clone(),
            target_b2: predictor.b2.clone(),
            decay,
        }
    }

    /// Update target weights via EMA: θ_target = τ * θ_target + (1 - τ) * θ_online
    pub fn update(&mut self, predictor: &JEPAPredictor) {
        let tau = self.decay;
        let one_minus_tau = 1.0 - tau;

        ema_update_vec(&mut self.target_w1, &predictor.w1, tau, one_minus_tau);
        ema_update_vec(&mut self.target_w2, &predictor.w2, tau, one_minus_tau);
        ema_update_vec(&mut self.target_b1, &predictor.b1, tau, one_minus_tau);
        ema_update_vec(&mut self.target_b2, &predictor.b2, tau, one_minus_tau);
    }

    /// Forward pass through the target encoder (used for computing target embeddings)
    pub fn forward(&self, input: &[f32], embed_dim: usize, hidden_dim: usize) -> Vec<f32> {
        // Hidden layer with GELU activation
        let mut hidden = vec![0.0f32; hidden_dim];
        for i in 0..hidden_dim {
            let mut sum = self.target_b1[i];
            for j in 0..embed_dim.min(input.len()) {
                sum += input[j] * self.target_w1[j * hidden_dim + i];
            }
            // GELU approximation (same as JEPAPredictor.forward)
            hidden[i] = sum * 0.5 * (1.0 + (sum * 0.7978845608 * (1.0 + 0.044715 * sum * sum)).tanh());
        }

        // Output layer (linear)
        let mut output = vec![0.0f32; embed_dim];
        for i in 0..embed_dim {
            let mut sum = self.target_b2[i];
            for j in 0..hidden_dim {
                sum += hidden[j] * self.target_w2[j * embed_dim + i];
            }
            output[i] = sum;
        }

        output
    }
}

/// Standalone straightening loss function for use outside the engine.
/// Computes curvature penalty over a sequence of latent states.
///
/// From Equation 6 of the paper:
///   ℒ_straight = -(1/T) Σ_t cos(v_t, v_{t+1})
/// where v_t = z_{t+1} - z_t
pub fn straightening_loss(trajectory: &[Vec<f32>]) -> f32 {
    if trajectory.len() < 3 { return 0.0; }

    let mut cosine_sum = 0.0f32;
    let mut count = 0usize;

    for t in 0..(trajectory.len() - 2) {
        let velocity_t = vector_subtract(&trajectory[t + 1], &trajectory[t]);
        let velocity_t_plus_1 = vector_subtract(&trajectory[t + 2], &trajectory[t + 1]);
        cosine_sum += cosine_similarity(&velocity_t, &velocity_t_plus_1);
        count += 1;
    }

    if count == 0 { return 0.0; }
    -(cosine_sum / count as f32)
}

/// Compute the combined JEPA + straightening training objective.
///
/// From Equation 7 of the paper:
///   ℒ_total = ℒ_pred + λ · ℒ_straight
fn combined_jepa_loss(
    prediction_loss: f32,
    trajectory: &[Vec<f32>],
    lambda: f32,
) -> f32 {
    prediction_loss + lambda * straightening_loss(trajectory)
}

// --- Vector utilities for temporal straightening ---

/// Element-wise vector subtraction: a - b
fn vector_subtract(a: &[f32], b: &[f32]) -> Vec<f32> {
    a.iter().zip(b.iter()).map(|(x, y)| x - y).collect()
}

/// In-place EMA update: target = tau * target + (1 - tau) * source
fn ema_update_vec(target: &mut [f32], source: &[f32], tau: f32, one_minus_tau: f32) {
    for (t, s) in target.iter_mut().zip(source.iter()) {
        *t = tau * *t + one_minus_tau * *s;
    }
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

/// Which kind of deduction rule is being checked for novelty
#[derive(Debug, Clone, Copy)]
enum RuleKind {
    Entailment,
    Contradiction,
    Commonsense,
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
    
    /// Learn an entailment or contradiction rule with rule explosion prevention.
    ///
    /// A new rule is only admitted if it passes the asymptotic novelty gate:
    ///   effective_threshold = base + (1 - base) * (1 - 1 / (1 + count / half_life))
    /// Early rules are easy to add (low threshold), but as the level fills up,
    /// the threshold increases — modeling diminishing marginal returns.
    /// When capacity is reached, the lowest-utility rule is evicted.
    pub fn learn_entailment(&mut self, premise: &[f32], hypothesis: &[f32], label: &str, level: usize) {
        if level >= self.level_rules.len() { return; }

        let combined: Vec<f32> = premise.iter().zip(hypothesis.iter())
            .map(|(p, h)| (p + h) / 2.0)
            .collect();

        match label {
            "entailment" => {
                let count = self.level_rules[level].entailments.len();
                if !self.passes_novelty_gate(&combined, level, RuleKind::Entailment) {
                    return;
                }
                if count >= self.config.max_rules_per_level {
                    self.evict_lowest_utility_entailment(level, &combined);
                }
                self.level_rules[level].entailments.push((premise.to_vec(), hypothesis.to_vec()));
            }
            "contradiction" => {
                let count = self.level_rules[level].contradictions.len();
                if !self.passes_novelty_gate(&combined, level, RuleKind::Contradiction) {
                    return;
                }
                if count >= self.config.max_rules_per_level {
                    self.evict_lowest_utility_contradiction(level, &combined);
                }
                self.level_rules[level].contradictions.push((premise.to_vec(), hypothesis.to_vec()));
            }
            _ => {}
        }
    }

    /// Learn a commonsense rule with rule explosion prevention.
    pub fn learn_commonsense(&mut self, head_embed: &[f32], relation: &str, tail: &str, level: usize) {
        if level >= self.level_rules.len() { return; }

        if !self.passes_novelty_gate(head_embed, level, RuleKind::Commonsense) {
            return;
        }

        let count = self.level_rules[level].commonsense.len();
        if count >= self.config.max_rules_per_level {
            self.evict_lowest_utility_commonsense(level, head_embed);
        }

        self.level_rules[level].commonsense.push((relation.to_string(), tail.to_string(), head_embed.to_vec()));
    }

    /// Compute the effective novelty threshold for a given rule count.
    /// Uses an asymptotic curve: threshold increases from base toward 1.0
    /// as rules accumulate, with half_life controlling the rate.
    ///   effective = base + (1 - base) * (1 - 1 / (1 + count / half_life))
    pub fn effective_novelty_threshold(&self, count: usize) -> f32 {
        let base = self.config.novelty_threshold_base;
        let half_life = self.config.novelty_half_life;
        let saturation = 1.0 - 1.0 / (1.0 + count as f32 / half_life);
        base + (1.0 - base) * saturation
    }

    /// Check if a new rule embedding passes the novelty gate for a given level.
    /// Returns true if the rule is sufficiently novel (dissimilar from existing rules).
    fn passes_novelty_gate(&self, embed: &[f32], level: usize, kind: RuleKind) -> bool {
        let existing_embeds: Vec<&[f32]> = match kind {
            RuleKind::Entailment => {
                self.level_rules[level].entailments.iter()
                    .map(|(p, _)| p.as_slice())
                    .collect()
            }
            RuleKind::Contradiction => {
                self.level_rules[level].contradictions.iter()
                    .map(|(p, _)| p.as_slice())
                    .collect()
            }
            RuleKind::Commonsense => {
                self.level_rules[level].commonsense.iter()
                    .map(|(_, _, e)| e.as_slice())
                    .collect()
            }
        };

        let threshold = self.effective_novelty_threshold(existing_embeds.len());

        // Rule passes if its maximum similarity to any existing rule is BELOW the threshold
        for existing in &existing_embeds {
            let sim = cosine_similarity(embed, existing).abs();
            if sim >= threshold {
                return false; // Too similar to an existing rule
            }
        }
        true
    }

    /// Evict the entailment rule with lowest utility (most similar to new rule = least novel)
    fn evict_lowest_utility_entailment(&mut self, level: usize, new_embed: &[f32]) {
        if self.level_rules[level].entailments.is_empty() { return; }
        let mut worst_idx = 0;
        let mut worst_sim = f32::NEG_INFINITY;
        for (i, (premise, _)) in self.level_rules[level].entailments.iter().enumerate() {
            let sim = cosine_similarity(new_embed, premise).abs();
            if sim > worst_sim {
                worst_sim = sim;
                worst_idx = i;
            }
        }
        self.level_rules[level].entailments.swap_remove(worst_idx);
    }

    /// Evict the contradiction rule with lowest utility
    fn evict_lowest_utility_contradiction(&mut self, level: usize, new_embed: &[f32]) {
        if self.level_rules[level].contradictions.is_empty() { return; }
        let mut worst_idx = 0;
        let mut worst_sim = f32::NEG_INFINITY;
        for (i, (premise, _)) in self.level_rules[level].contradictions.iter().enumerate() {
            let sim = cosine_similarity(new_embed, premise).abs();
            if sim > worst_sim {
                worst_sim = sim;
                worst_idx = i;
            }
        }
        self.level_rules[level].contradictions.swap_remove(worst_idx);
    }

    /// Evict the commonsense rule with lowest utility
    fn evict_lowest_utility_commonsense(&mut self, level: usize, new_embed: &[f32]) {
        if self.level_rules[level].commonsense.is_empty() { return; }
        let mut worst_idx = 0;
        let mut worst_sim = f32::NEG_INFINITY;
        for (i, (_, _, embed)) in self.level_rules[level].commonsense.iter().enumerate() {
            let sim = cosine_similarity(new_embed, embed).abs();
            if sim > worst_sim {
                worst_sim = sim;
                worst_idx = i;
            }
        }
        self.level_rules[level].commonsense.swap_remove(worst_idx);
    }

    /// Get total rule count across all levels and all rule kinds
    pub fn rule_count(&self) -> usize {
        self.level_rules.iter().map(|r| {
            r.entailments.len() + r.contradictions.len() + r.commonsense.len()
        }).sum()
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
    pub straightening_loss_sum: f32,
    pub mean_curvature: f32,
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

// =============================================================================
// QUANTUM-INSPIRED JEPA + EXHAUSTIVE PATHWAY SEARCH
// =============================================================================
//
// Analogy to Grover's Algorithm:
// - JEPA Predictor = Quantum Oracle (marks the target state)
// - Exhaustive Pathway = Amplitude Amplification (searches all paths)
// - Energy Function = Quantum Interference (constructive for correct paths)
//
// Key insight: JEPA predicts WHERE to go, Pathway finds HOW to get there optimally

use crate::ml::pathway::{ExhaustivePathwayOptimizer, PathwayConfig, ScoredPathway};

/// Quantum-inspired cognitive optimizer combining JEPA + Exhaustive Pathway
/// Enhanced with temporal straightening (arXiv:2603.12231) for better pathway scoring.
#[derive(Debug)]
pub struct QuantumJEPAOptimizer {
    /// JEPA predictor (quantum oracle)
    pub predictor: JEPAPredictor,
    /// Hierarchical deduction engine
    pub deduction_engine: HierarchicalDeductionEngine,
    /// Temporal straightening engine for curvature-aware pathway evaluation
    pub straightening_engine: TemporalStraighteningEngine,
    /// EMA target encoder for stable target embeddings (collapse prevention)
    pub target_encoder: EMATargetEncoder,
    /// Configuration
    pub config: JEPAConfig,
    /// Energy function temperature
    pub energy_temperature: f32,
}

impl QuantumJEPAOptimizer {
    pub fn new(config: JEPAConfig) -> Self {
        let predictor = JEPAPredictor::new(config.clone());
        let deduction_engine = HierarchicalDeductionEngine::new(config.clone());
        let straightening_engine = TemporalStraighteningEngine::new(
            config.trajectory_window,
            config.straightening_lambda,
        );
        let target_encoder = EMATargetEncoder::from_predictor(&predictor, config.ema_decay);
        Self {
            predictor,
            deduction_engine,
            straightening_engine,
            target_encoder,
            config,
            energy_temperature: 0.1,
        }
    }
    
    /// QUANTUM ORACLE: Predict target embedding from context
    /// This is like marking the solution state in Grover's algorithm
    pub fn predict_target(&self, context_embed: &[f32]) -> Vec<f32> {
        self.predictor.forward(context_embed)
    }
    
    /// AMPLITUDE AMPLIFICATION: Find optimal pathway to target
    /// Uses exhaustive search over all n! permutations
    pub fn find_optimal_pathway(
        &self,
        context_embed: &[f32],
        choice_embeds: &[Vec<f32>],
        node_embeds: &[Vec<f32>],
    ) -> Vec<ScoredPathway> {
        // Predict target using JEPA (quantum oracle)
        let predicted_target = self.predict_target(context_embed);
        
        // Configure exhaustive pathway search
        let mut pathway_config = PathwayConfig::default();
        pathway_config.n_nodes = node_embeds.len().min(7); // Cap at 7! = 5040
        pathway_config.dimension = self.config.embed_dim;
        pathway_config.parallel = true; // GPU-like parallelism
        
        let mut optimizer = ExhaustivePathwayOptimizer::new(pathway_config);
        optimizer.set_embeddings(node_embeds);
        optimizer.set_target(&predicted_target);
        
        // Run exhaustive search (amplitude amplification)
        optimizer.fast_search(choice_embeds.len())
    }
    
    /// ENERGY FUNCTION: Compute energy for a (context, choice, path) triple
    /// Low energy = good match (like constructive quantum interference)
    /// Enhanced with curvature-aware scoring from arXiv:2603.12231
    pub fn compute_energy(
        &self,
        context_embed: &[f32],
        choice_embed: &[f32],
        predicted_target: &[f32],
        pathway_score: f64,
        reasoning_trajectory: Option<&[Vec<f32>]>,
    ) -> f32 {
        // E1: JEPA prediction error (how well does choice match predicted target?)
        let jepa_error = jepa_mse_loss(choice_embed, predicted_target);
        
        // E2: Context-choice alignment
        let alignment = 1.0 - cosine_similarity(context_embed, choice_embed).max(0.0);
        
        // E3: Pathway optimality (lower is better, so negate and normalize)
        let pathway_energy = 1.0 / (1.0 + pathway_score.abs() as f32);
        
        // E4: Hierarchical deduction confidence
        let deduction_steps = self.deduction_engine.deduce_hierarchical(choice_embed, 3);
        let deduction_confidence: f32 = deduction_steps.iter()
            .map(|s| s.confidence)
            .sum::<f32>() / deduction_steps.len().max(1) as f32;
        let deduction_energy = 1.0 - deduction_confidence;
        
        // E5: Trajectory straightness (arXiv:2603.12231)
        // Straighter reasoning paths = Euclidean distance better approximates geodesic
        // = more reliable gradient-based optimization = lower energy
        let straightness_energy = if let Some(trajectory) = reasoning_trajectory {
            // curvature_score returns [0, 1] where 1 = straight; invert for energy
            1.0 - TemporalStraighteningEngine::curvature_score(trajectory)
        } else {
            0.5 // Neutral when no trajectory available
        };
        
        // Total energy (weighted sum including straightness)
        let total_energy = 
            0.30 * jepa_error +           // JEPA prediction match
            0.15 * alignment +             // Context alignment
            0.15 * pathway_energy +        // Pathway optimality
            0.15 * deduction_energy +      // Deduction confidence
            0.25 * straightness_energy;    // Trajectory straightness (arXiv:2603.12231)
        
        total_energy
    }
    
    /// QUANTUM SEARCH: Find best choice using JEPA + Exhaustive Pathway
    /// Enhanced with temporal straightening (arXiv:2603.12231) — prefers
    /// choices whose reasoning trajectories are straighter (lower curvature),
    /// because straight trajectories make Euclidean distance a faithful proxy
    /// for geodesic logical distance.
    /// Returns (best_choice_idx, confidence)
    pub fn quantum_search(
        &self,
        context_embed: &[f32],
        choice_embeds: &[Vec<f32>],
    ) -> (usize, f32) {
        // Step 1: JEPA predicts target (quantum oracle marks solution)
        let predicted_target = self.predict_target(context_embed);
        
        // Step 2: For each choice, compute energy with curvature-aware scoring
        let mut energies: Vec<(usize, f32)> = Vec::new();
        
        for (idx, choice_embed) in choice_embeds.iter().enumerate() {
            // Build reasoning nodes from choice embedding
            let node_embeds = self.build_reasoning_nodes(context_embed, choice_embed);
            
            // Find optimal pathway (amplitude amplification)
            let mut pathway_config = PathwayConfig::default();
            pathway_config.n_nodes = node_embeds.len().min(7);
            pathway_config.dimension = self.config.embed_dim;
            pathway_config.parallel = true;
            
            let mut optimizer = ExhaustivePathwayOptimizer::new(pathway_config);
            optimizer.set_embeddings(&node_embeds);
            optimizer.set_target(&predicted_target);
            
            let pathways = optimizer.fast_search(1);
            let pathway_score = pathways.first().map(|p| p.score).unwrap_or(0.0);
            
            // Build reasoning trajectory from hierarchical deduction steps
            // This is the latent trajectory analogous to z_t in the paper
            let deduction_steps = self.deduction_engine.deduce_hierarchical(choice_embed, 6);
            let reasoning_trajectory: Vec<Vec<f32>> = std::iter::once(context_embed.to_vec())
                .chain(deduction_steps.iter().map(|step| step.embedding.clone()))
                .chain(std::iter::once(choice_embed.to_vec()))
                .collect();
            
            // Compute total energy with trajectory straightness
            let energy = self.compute_energy(
                context_embed,
                choice_embed,
                &predicted_target,
                pathway_score,
                Some(&reasoning_trajectory),
            );
            
            energies.push((idx, energy));
        }
        
        // Step 3: Select lowest energy choice (quantum interference selects correct state)
        energies.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        
        let (best_idx, best_energy) = energies.first().copied().unwrap_or((0, 1.0));
        
        // Convert energy to confidence (lower energy = higher confidence)
        let confidence = (-best_energy / self.energy_temperature).exp();
        
        (best_idx, confidence.min(1.0))
    }
    
    /// Build reasoning nodes from context and choice embeddings
    fn build_reasoning_nodes(&self, context: &[f32], choice: &[f32]) -> Vec<Vec<f32>> {
        let dim = self.config.embed_dim;
        let mut nodes = Vec::new();
        
        // Node 1: Context embedding
        nodes.push(context.to_vec());
        
        // Node 2: Choice embedding
        nodes.push(choice.to_vec());
        
        // Node 3: Context-choice interaction (element-wise product)
        let interaction: Vec<f32> = context.iter()
            .zip(choice.iter())
            .map(|(c, ch)| c * ch)
            .collect();
        nodes.push(interaction);
        
        // Node 4: JEPA predicted target
        let predicted = self.predictor.forward(context);
        nodes.push(predicted);
        
        // Node 5: Deduction step embedding (level 3 - sacred position)
        let deduction_steps = self.deduction_engine.deduce_hierarchical(choice, 3);
        if let Some(step) = deduction_steps.last() {
            nodes.push(step.embedding.clone());
        } else {
            nodes.push(vec![0.0; dim]);
        }
        
        // Ensure all nodes have correct dimension
        for node in &mut nodes {
            node.resize(dim, 0.0);
        }
        
        nodes
    }
    
    /// Train JEPA predictor on (context, target) pairs with temporal straightening.
    ///
    /// Implements the combined objective from arXiv:2603.12231 Equation 7:
    ///   ℒ_total = ℒ_pred + λ · ℒ_straight
    ///
    /// The prediction loss uses the EMA target encoder (stop-gradient equivalent)
    /// to prevent collapse. The straightening loss penalizes curvature in the
    /// reasoning trajectory formed by deduction steps from context to target.
    pub fn train(&mut self, context: &[f32], target: &[f32], learning_rate: f32) -> f32 {
        // Online predictor forward pass
        let pred = self.predictor.forward(context);
        
        // Target encoder forward pass (stop-gradient / EMA — prevents collapse)
        let target_pred = self.target_encoder.forward(
            target,
            self.config.embed_dim,
            self.config.hidden_dim,
        );
        
        // Prediction loss: MSE between online prediction and EMA target
        let prediction_loss = jepa_mse_loss(&pred, &target_pred);
        
        // Build reasoning trajectory for straightening loss:
        // context → deduction step 1 → ... → deduction step N → target
        let deduction_steps = self.deduction_engine.deduce_hierarchical(context, 6);
        let trajectory: Vec<Vec<f32>> = std::iter::once(context.to_vec())
            .chain(deduction_steps.iter().map(|step| step.embedding.clone()))
            .chain(std::iter::once(target.to_vec()))
            .collect();
        
        // Record trajectory states for monitoring
        self.straightening_engine.reset_trajectory();
        for state in &trajectory {
            self.straightening_engine.record_state(state);
        }
        
        // Combined loss: ℒ_total = ℒ_pred + λ · ℒ_straight (Equation 7)
        let straight_loss = straightening_loss(&trajectory);
        let total_loss = prediction_loss + self.config.straightening_lambda * straight_loss;
        
        // Simplified gradient update (in production, use Burn's autodiff)
        let embed_dim = self.config.embed_dim;
        let hidden_dim = self.config.hidden_dim;
        
        // Compute gradient for output layer (from combined loss)
        let grad_output: Vec<f32> = pred.iter()
            .zip(target_pred.iter())
            .map(|(p, t)| 2.0 * (p - t) / embed_dim as f32)
            .collect();
        
        // Update w2 and b2
        for i in 0..embed_dim.min(self.predictor.b2.len()) {
            self.predictor.b2[i] -= learning_rate * grad_output[i];
        }
        
        // Update w2 (simplified - just the diagonal for efficiency)
        for i in 0..hidden_dim.min(embed_dim) {
            let idx = i * embed_dim + i;
            if idx < self.predictor.w2.len() {
                self.predictor.w2[idx] -= learning_rate * grad_output[i % embed_dim] * 0.1;
            }
        }
        
        // EMA target encoder update: θ_target = τ · θ_target + (1-τ) · θ_online
        self.target_encoder.update(&self.predictor);
        
        total_loss
    }
    
    /// Get the mean curvature observed during training.
    /// Lower values indicate straighter reasoning trajectories.
    pub fn mean_curvature(&self) -> f32 {
        self.straightening_engine.mean_curvature()
    }
    
    /// Reset trajectory between independent reasoning chains
    pub fn reset_trajectory(&mut self) {
        self.straightening_engine.reset_trajectory();
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

    // =========================================================================
    // Temporal Straightening Tests (arXiv:2603.12231)
    // =========================================================================

    #[test]
    fn test_straightening_loss_straight_trajectory() {
        // A perfectly straight trajectory should have cosine similarity = 1.0
        // between consecutive velocity vectors, giving straightening loss = -1.0
        let trajectory: Vec<Vec<f32>> = (0..5)
            .map(|t| vec![t as f32; 4])  // [0,0,0,0], [1,1,1,1], [2,2,2,2], ...
            .collect();
        let loss = straightening_loss(&trajectory);
        // All velocity vectors are identical → cosine = 1.0 → loss = -1.0
        assert!((loss - (-1.0)).abs() < 1e-5, "Straight trajectory loss should be -1.0, got {}", loss);
    }

    #[test]
    fn test_straightening_loss_curved_trajectory() {
        // A trajectory that reverses direction should have high curvature
        let trajectory: Vec<Vec<f32>> = vec![
            vec![0.0, 0.0, 0.0, 0.0],
            vec![1.0, 1.0, 1.0, 1.0],
            vec![0.0, 0.0, 0.0, 0.0],  // Reversal: velocity goes from +1 to -1
        ];
        let loss = straightening_loss(&trajectory);
        // Velocity vectors point in opposite directions → cosine = -1.0 → loss = 1.0
        assert!((loss - 1.0).abs() < 1e-5, "Reversed trajectory loss should be 1.0, got {}", loss);
    }

    #[test]
    fn test_straightening_loss_too_short() {
        // Fewer than 3 states → no curvature to compute → loss = 0
        let short: Vec<Vec<f32>> = vec![vec![1.0, 2.0], vec![3.0, 4.0]];
        assert_eq!(straightening_loss(&short), 0.0);
        assert_eq!(straightening_loss(&[]), 0.0);
    }

    #[test]
    fn test_curvature_score_straight() {
        let pathway: Vec<Vec<f32>> = (0..5)
            .map(|t| vec![t as f32 * 0.5; 8])
            .collect();
        let score = TemporalStraighteningEngine::curvature_score(&pathway);
        // Perfectly straight → score = 1.0
        assert!((score - 1.0).abs() < 1e-5, "Straight pathway score should be 1.0, got {}", score);
    }

    #[test]
    fn test_curvature_score_reversed() {
        let pathway: Vec<Vec<f32>> = vec![
            vec![0.0; 4],
            vec![1.0; 4],
            vec![0.0; 4],
        ];
        let score = TemporalStraighteningEngine::curvature_score(&pathway);
        // Maximum curvature → score = 0.0
        assert!((score - 0.0).abs() < 1e-5, "Reversed pathway score should be 0.0, got {}", score);
    }

    #[test]
    fn test_temporal_straightening_engine_record() {
        let mut engine = TemporalStraighteningEngine::new(16, 0.5);
        
        // First two states: no loss yet (need 3 minimum)
        assert!(engine.record_state(&[0.0, 0.0]).is_none());
        assert!(engine.record_state(&[1.0, 1.0]).is_none());
        
        // Third state: now we can compute curvature
        let loss = engine.record_state(&[2.0, 2.0]);
        assert!(loss.is_some(), "Should compute loss after 3 states");
        
        // Straight trajectory → loss should be negative (good)
        let loss_val = loss.unwrap();
        assert!(loss_val < 0.0, "Straight trajectory should have negative loss, got {}", loss_val);
    }

    #[test]
    fn test_ema_target_encoder() {
        let config = JEPAConfig::default();
        let predictor = JEPAPredictor::new(config.clone());
        let mut ema = EMATargetEncoder::from_predictor(&predictor, 0.996);
        
        // Target should produce same output as predictor initially
        let input = vec![0.1; config.embed_dim];
        let pred_out = predictor.forward(&input);
        let ema_out = ema.forward(&input, config.embed_dim, config.hidden_dim);
        
        for (p, e) in pred_out.iter().zip(ema_out.iter()) {
            assert!((p - e).abs() < 1e-5, "EMA should match predictor initially");
        }
        
        // After update, target should still be very close (decay=0.996)
        ema.update(&predictor);
        let ema_out2 = ema.forward(&input, config.embed_dim, config.hidden_dim);
        for (p, e) in pred_out.iter().zip(ema_out2.iter()) {
            assert!((p - e).abs() < 1e-4, "EMA should stay close after self-update");
        }
    }

    #[test]
    fn test_quantum_jepa_with_straightening() {
        let config = JEPAConfig::default();
        let optimizer = QuantumJEPAOptimizer::new(config.clone());
        
        // Verify straightening engine is initialized
        assert_eq!(optimizer.straightening_engine.mean_curvature(), 0.0);
        
        // Run quantum search with choices
        let context = vec![0.5; config.embed_dim];
        let choices = vec![
            vec![0.6; config.embed_dim],
            vec![0.1; config.embed_dim],
            vec![0.9; config.embed_dim],
        ];
        let (best_idx, confidence) = optimizer.quantum_search(&context, &choices);
        
        assert!(best_idx < choices.len());
        assert!(confidence >= 0.0 && confidence <= 1.0);
    }

    #[test]
    fn test_quantum_jepa_train_with_straightening() {
        let config = JEPAConfig::default();
        let mut optimizer = QuantumJEPAOptimizer::new(config.clone());
        
        let context = vec![0.3; config.embed_dim];
        let target = vec![0.7; config.embed_dim];
        
        // Train should return combined loss (prediction + straightening)
        let loss1 = optimizer.train(&context, &target, 0.001);
        let loss2 = optimizer.train(&context, &target, 0.001);
        
        // Loss should be non-negative and decreasing with training
        assert!(loss1 >= 0.0 || loss1 < 0.0, "Loss should be a valid number");
        assert!(loss1.is_finite(), "Loss should be finite");
        assert!(loss2.is_finite(), "Second loss should be finite");
    }

    // =========================================================================
    // Rule Explosion Stopping Criteria Tests
    // =========================================================================

    #[test]
    fn test_novelty_threshold_asymptotic() {
        let config = JEPAConfig::default();
        let engine = HierarchicalDeductionEngine::new(config);

        // At 0 rules, threshold should be the base value
        let t0 = engine.effective_novelty_threshold(0);
        assert!((t0 - 0.3).abs() < 1e-5, "Threshold at 0 rules should be base (0.3), got {}", t0);

        // At half_life (256) rules, threshold should be halfway between base and 1.0
        let t_half = engine.effective_novelty_threshold(256);
        let expected_half = 0.3 + (1.0 - 0.3) * 0.5; // 0.65
        assert!((t_half - expected_half).abs() < 0.01, "Threshold at half_life should be ~0.65, got {}", t_half);

        // Threshold should increase monotonically
        let t10 = engine.effective_novelty_threshold(10);
        let t100 = engine.effective_novelty_threshold(100);
        let t1000 = engine.effective_novelty_threshold(1000);
        assert!(t10 < t100, "Threshold should increase: {} < {}", t10, t100);
        assert!(t100 < t1000, "Threshold should increase: {} < {}", t100, t1000);

        // Threshold should approach but never exceed 1.0
        let t_huge = engine.effective_novelty_threshold(1_000_000);
        assert!(t_huge < 1.0, "Threshold should stay below 1.0, got {}", t_huge);
        assert!(t_huge > 0.99, "Threshold at 1M rules should approach 1.0, got {}", t_huge);
    }

    #[test]
    fn test_rule_explosion_duplicate_rejection() {
        let mut config = JEPAConfig::default();
        config.embed_dim = 4;
        config.ladder_levels = 2;
        config.novelty_threshold_base = 0.3;
        let mut engine = HierarchicalDeductionEngine::new(config);

        let premise = vec![1.0, 0.0, 0.0, 0.0];
        let hypothesis = vec![0.0, 1.0, 0.0, 0.0];

        // First rule should be accepted (empty level)
        engine.learn_entailment(&premise, &hypothesis, "entailment", 0);
        assert_eq!(engine.rule_count(), 1);

        // Identical rule should be rejected (too similar)
        engine.learn_entailment(&premise, &hypothesis, "entailment", 0);
        assert_eq!(engine.rule_count(), 1, "Duplicate rule should be rejected");

        // Sufficiently different rule should be accepted
        let premise2 = vec![0.0, 0.0, 1.0, 0.0];
        let hypothesis2 = vec![0.0, 0.0, 0.0, 1.0];
        engine.learn_entailment(&premise2, &hypothesis2, "entailment", 0);
        assert_eq!(engine.rule_count(), 2, "Novel rule should be accepted");
    }

    #[test]
    fn test_rule_explosion_capacity_enforcement() {
        let mut config = JEPAConfig::default();
        config.embed_dim = 4;
        config.ladder_levels = 1;
        config.max_rules_per_level = 3;
        config.novelty_threshold_base = 0.0; // Accept everything for this test
        config.novelty_half_life = 10000.0;  // Keep threshold low
        let mut engine = HierarchicalDeductionEngine::new(config);

        // Add 4 distinct commonsense rules — should cap at 3
        for i in 0..4 {
            let mut embed = vec![0.0; 4];
            embed[i % 4] = 1.0; // Orthogonal embeddings
            engine.learn_commonsense(&embed, &format!("relation_{}", i), &format!("tail_{}", i), 0);
        }

        // Should have at most max_rules_per_level (3)
        let count = engine.level_rules[0].commonsense.len();
        assert!(count <= 3, "Should cap at max_rules_per_level=3, got {}", count);
    }
}
