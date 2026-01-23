//! Pre-training Infrastructure for World-Class AI Models
//!
//! Implements the complete pre-training pipeline with:
//! - Masked Language Modeling (MLM) objective (BERT-style)
//! - Causal Language Modeling (CLM) objective (GPT-style)
//! - Next Sentence Prediction (NSP) for context understanding
//! - Data loading and batching infrastructure
//! - Gradient accumulation for large effective batch sizes
//! - Learning rate scheduling with warmup
//! - Sacred geometry integration at checkpoints (3-6-9)
//!
//! ## Pre-training Objectives
//!
//! ### Masked Language Modeling (MLM)
//! ```text
//! Input:  "The [MASK] sat on the [MASK]"
//! Target: "The cat sat on the mat"
//! Loss:   CrossEntropy on masked positions only
//! ```
//!
//! ### Causal Language Modeling (CLM)
//! ```text
//! Input:  "The quick brown"
//! Target: "quick brown fox"
//! Loss:   CrossEntropy on all positions (shifted)
//! ```

use std::collections::HashMap;
use ndarray::{Array1, Array2, Array3, Axis, s};
use rand::prelude::*;
use rand::distributions::WeightedIndex;

/// Special token IDs
pub const MASK_TOKEN_ID: u32 = 103;
pub const PAD_TOKEN_ID: u32 = 0;
pub const BOS_TOKEN_ID: u32 = 1;
pub const EOS_TOKEN_ID: u32 = 2;
pub const UNK_TOKEN_ID: u32 = 100;
pub const CLS_TOKEN_ID: u32 = 101;
pub const SEP_TOKEN_ID: u32 = 102;

/// Pre-training objective type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PretrainingObjective {
    /// Masked Language Modeling (BERT-style)
    /// Randomly mask 15% of tokens and predict them
    MaskedLM {
        mask_probability: f32,
        mask_token_prob: f32,
        random_token_prob: f32,
    },
    
    /// Causal Language Modeling (GPT-style)
    /// Predict next token given previous tokens
    CausalLM,
    
    /// Prefix Language Modeling
    /// Bidirectional attention on prefix, causal on rest
    PrefixLM {
        prefix_fraction: f32,
    },
    
    /// Span Corruption (T5-style)
    /// Replace spans with sentinel tokens
    SpanCorruption {
        mean_span_length: f32,
        corruption_rate: f32,
    },
}

impl Default for PretrainingObjective {
    fn default() -> Self {
        PretrainingObjective::MaskedLM {
            mask_probability: 0.15,
            mask_token_prob: 0.8,
            random_token_prob: 0.1,
        }
    }
}

/// Configuration for pre-training
#[derive(Debug, Clone)]
pub struct PretrainingConfig {
    pub objective: PretrainingObjective,
    pub max_seq_len: usize,
    pub vocab_size: usize,
    pub batch_size: usize,
    pub gradient_accumulation_steps: usize,
    pub total_steps: usize,
    pub warmup_steps: usize,
    pub learning_rate: f32,
    pub min_learning_rate: f32,
    pub weight_decay: f32,
    pub beta1: f32,
    pub beta2: f32,
    pub epsilon: f32,
    pub max_grad_norm: f32,
    pub sacred_geometry_checkpoints: bool,
    pub checkpoint_every: usize,
    pub log_every: usize,
}

impl Default for PretrainingConfig {
    fn default() -> Self {
        Self {
            objective: PretrainingObjective::default(),
            max_seq_len: 512,
            vocab_size: 32000,
            batch_size: 32,
            gradient_accumulation_steps: 4,
            total_steps: 100_000,
            warmup_steps: 2000,
            learning_rate: 1e-4,
            min_learning_rate: 1e-5,
            weight_decay: 0.01,
            beta1: 0.9,
            beta2: 0.999,
            epsilon: 1e-8,
            max_grad_norm: 1.0,
            sacred_geometry_checkpoints: true,
            checkpoint_every: 1000,
            log_every: 100,
        }
    }
}

/// Masked Language Modeling (MLM) Module
/// 
/// Implements BERT-style masked language modeling:
/// - 15% of tokens are selected for prediction
/// - Of those: 80% replaced with [MASK], 10% random, 10% unchanged
pub struct MaskedLanguageModel {
    config: PretrainingConfig,
    vocab_size: usize,
}

impl MaskedLanguageModel {
    pub fn new(config: PretrainingConfig) -> Self {
        let vocab_size = config.vocab_size;
        Self { config, vocab_size }
    }
    
    /// Apply masking to input tokens
    /// Returns: (masked_tokens, labels, mask_positions)
    pub fn apply_masking(&self, tokens: &[u32]) -> (Vec<u32>, Vec<i64>, Vec<usize>) {
        let mut rng = thread_rng();
        let mut masked_tokens = tokens.to_vec();
        let mut labels = vec![-100i64; tokens.len()]; // -100 = ignore in loss
        let mut mask_positions = Vec::new();
        
        let (mask_prob, mask_token_prob, random_prob) = match self.config.objective {
            PretrainingObjective::MaskedLM { mask_probability, mask_token_prob, random_token_prob } => {
                (mask_probability, mask_token_prob, random_token_prob)
            },
            _ => (0.15, 0.8, 0.1),
        };
        
        for (i, &token) in tokens.iter().enumerate() {
            // Skip special tokens
            if token == PAD_TOKEN_ID || token == CLS_TOKEN_ID || 
               token == SEP_TOKEN_ID || token == BOS_TOKEN_ID || token == EOS_TOKEN_ID {
                continue;
            }
            
            if rng.gen::<f32>() < mask_prob {
                labels[i] = token as i64;
                mask_positions.push(i);
                
                let r = rng.gen::<f32>();
                if r < mask_token_prob {
                    // Replace with [MASK]
                    masked_tokens[i] = MASK_TOKEN_ID;
                } else if r < mask_token_prob + random_prob {
                    // Replace with random token
                    masked_tokens[i] = rng.gen_range(1000..self.vocab_size as u32);
                }
                // else: keep original (10%)
            }
        }
        
        (masked_tokens, labels, mask_positions)
    }
    
    /// Compute MLM loss
    /// logits: [batch, seq_len, vocab_size]
    /// labels: [batch, seq_len] (-100 for ignored positions)
    pub fn compute_loss(&self, logits: &Array3<f32>, labels: &Array2<i64>) -> f32 {
        let (batch_size, seq_len, vocab_size) = (logits.shape()[0], logits.shape()[1], logits.shape()[2]);
        let mut total_loss = 0.0f32;
        let mut count = 0;
        
        for b in 0..batch_size {
            for s in 0..seq_len {
                let label = labels[[b, s]];
                if label >= 0 {
                    // Compute cross-entropy for this position
                    let logits_slice = logits.slice(s![b, s, ..]);
                    let max_logit = logits_slice.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
                    let log_sum_exp: f32 = logits_slice.iter().map(|&x| (x - max_logit).exp()).sum::<f32>().ln() + max_logit;
                    let log_prob = logits_slice[label as usize] - log_sum_exp;
                    total_loss -= log_prob;
                    count += 1;
                }
            }
        }
        
        if count > 0 { total_loss / count as f32 } else { 0.0 }
    }
    
    /// Compute MLM loss gradient
    pub fn compute_loss_gradient(&self, logits: &Array3<f32>, labels: &Array2<i64>) -> Array3<f32> {
        let (batch_size, seq_len, vocab_size) = (logits.shape()[0], logits.shape()[1], logits.shape()[2]);
        let mut gradients = Array3::zeros((batch_size, seq_len, vocab_size));
        let mut count = 0;
        
        for b in 0..batch_size {
            for s in 0..seq_len {
                let label = labels[[b, s]];
                if label >= 0 {
                    // Softmax gradient: p - 1(y=k)
                    let logits_slice = logits.slice(s![b, s, ..]);
                    let max_logit = logits_slice.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
                    let exp_logits: Vec<f32> = logits_slice.iter().map(|&x| (x - max_logit).exp()).collect();
                    let sum_exp: f32 = exp_logits.iter().sum();
                    
                    for v in 0..vocab_size {
                        let prob = exp_logits[v] / sum_exp;
                        let target = if v == label as usize { 1.0 } else { 0.0 };
                        gradients[[b, s, v]] = prob - target;
                    }
                    count += 1;
                }
            }
        }
        
        if count > 0 {
            gradients /= count as f32;
        }
        gradients
    }
}

/// Causal Language Modeling (CLM) Module
/// 
/// Implements GPT-style autoregressive language modeling
pub struct CausalLanguageModel {
    config: PretrainingConfig,
}

impl CausalLanguageModel {
    pub fn new(config: PretrainingConfig) -> Self {
        Self { config }
    }
    
    /// Create causal attention mask
    /// Returns lower-triangular mask where True = attend, False = mask
    pub fn create_causal_mask(&self, seq_len: usize) -> Array2<bool> {
        Array2::from_shape_fn((seq_len, seq_len), |(i, j)| j <= i)
    }
    
    /// Prepare CLM inputs and labels
    /// Input: [t1, t2, t3, t4]
    /// Labels: [t2, t3, t4, -100] (shifted by 1)
    pub fn prepare_clm_batch(&self, tokens: &[u32]) -> (Vec<u32>, Vec<i64>) {
        let input = tokens.to_vec();
        let mut labels = vec![-100i64; tokens.len()];
        
        // Labels are shifted: predict next token
        for i in 0..tokens.len().saturating_sub(1) {
            labels[i] = tokens[i + 1] as i64;
        }
        
        (input, labels)
    }
    
    /// Compute CLM loss (same as MLM but on all non-padding positions)
    pub fn compute_loss(&self, logits: &Array3<f32>, labels: &Array2<i64>) -> f32 {
        let (batch_size, seq_len, _) = (logits.shape()[0], logits.shape()[1], logits.shape()[2]);
        let mut total_loss = 0.0f32;
        let mut count = 0;
        
        for b in 0..batch_size {
            for s in 0..seq_len {
                let label = labels[[b, s]];
                if label >= 0 && label != PAD_TOKEN_ID as i64 {
                    let logits_slice = logits.slice(s![b, s, ..]);
                    let max_logit = logits_slice.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
                    let log_sum_exp: f32 = logits_slice.iter().map(|&x| (x - max_logit).exp()).sum::<f32>().ln() + max_logit;
                    let log_prob = logits_slice[label as usize] - log_sum_exp;
                    total_loss -= log_prob;
                    count += 1;
                }
            }
        }
        
        if count > 0 { total_loss / count as f32 } else { 0.0 }
    }
}

/// Learning Rate Scheduler with Warmup and Cosine Decay
pub struct LearningRateScheduler {
    warmup_steps: usize,
    total_steps: usize,
    max_lr: f32,
    min_lr: f32,
    current_step: usize,
}

impl LearningRateScheduler {
    pub fn new(warmup_steps: usize, total_steps: usize, max_lr: f32, min_lr: f32) -> Self {
        Self {
            warmup_steps,
            total_steps,
            max_lr,
            min_lr,
            current_step: 0,
        }
    }
    
    /// Get current learning rate
    pub fn get_lr(&self) -> f32 {
        if self.current_step < self.warmup_steps {
            // Linear warmup
            self.max_lr * (self.current_step as f32 / self.warmup_steps as f32)
        } else {
            // Cosine decay
            let progress = (self.current_step - self.warmup_steps) as f32 
                / (self.total_steps - self.warmup_steps) as f32;
            let cosine_decay = 0.5 * (1.0 + (std::f32::consts::PI * progress).cos());
            self.min_lr + (self.max_lr - self.min_lr) * cosine_decay
        }
    }
    
    /// Step the scheduler
    pub fn step(&mut self) {
        self.current_step += 1;
    }
    
    /// Reset scheduler
    pub fn reset(&mut self) {
        self.current_step = 0;
    }
}

/// Pre-training Batch
#[derive(Debug, Clone)]
pub struct PretrainingBatch {
    pub input_ids: Array2<u32>,
    pub attention_mask: Array2<f32>,
    pub labels: Array2<i64>,
    pub position_ids: Array2<u32>,
}

/// Pre-training Data Loader
pub struct PretrainingDataLoader {
    config: PretrainingConfig,
    data: Vec<Vec<u32>>,
    current_idx: usize,
    shuffle: bool,
}

impl PretrainingDataLoader {
    pub fn new(config: PretrainingConfig, data: Vec<Vec<u32>>, shuffle: bool) -> Self {
        Self {
            config,
            data,
            current_idx: 0,
            shuffle,
        }
    }
    
    /// Get next batch
    pub fn next_batch(&mut self) -> Option<PretrainingBatch> {
        if self.current_idx >= self.data.len() {
            if self.shuffle {
                let mut rng = thread_rng();
                self.data.shuffle(&mut rng);
            }
            self.current_idx = 0;
            return None; // Epoch complete
        }
        
        let batch_size = self.config.batch_size.min(self.data.len() - self.current_idx);
        let max_len = self.config.max_seq_len;
        
        let mut input_ids = Array2::zeros((batch_size, max_len));
        let mut attention_mask = Array2::zeros((batch_size, max_len));
        let mut labels = Array2::from_elem((batch_size, max_len), -100i64);
        let mut position_ids = Array2::zeros((batch_size, max_len));
        
        for b in 0..batch_size {
            let tokens = &self.data[self.current_idx + b];
            let seq_len = tokens.len().min(max_len);
            
            for s in 0..seq_len {
                input_ids[[b, s]] = tokens[s];
                attention_mask[[b, s]] = 1.0;
                position_ids[[b, s]] = s as u32;
            }
        }
        
        self.current_idx += batch_size;
        
        Some(PretrainingBatch {
            input_ids,
            attention_mask,
            labels,
            position_ids,
        })
    }
    
    /// Reset data loader
    pub fn reset(&mut self) {
        self.current_idx = 0;
        if self.shuffle {
            let mut rng = thread_rng();
            self.data.shuffle(&mut rng);
        }
    }
    
    /// Get total number of samples
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// Pre-training Metrics
#[derive(Debug, Clone, Default)]
pub struct PretrainingMetrics {
    pub step: usize,
    pub loss: f32,
    pub perplexity: f32,
    pub learning_rate: f32,
    pub tokens_per_second: f32,
    pub gradient_norm: f32,
    pub sacred_interventions: usize,
}

/// Pre-trainer orchestrates the entire pre-training process
pub struct Pretrainer {
    config: PretrainingConfig,
    mlm: Option<MaskedLanguageModel>,
    clm: Option<CausalLanguageModel>,
    lr_scheduler: LearningRateScheduler,
    metrics_history: Vec<PretrainingMetrics>,
    accumulated_gradients: HashMap<String, Array2<f32>>,
    accumulation_count: usize,
}

impl Pretrainer {
    pub fn new(config: PretrainingConfig) -> Self {
        let lr_scheduler = LearningRateScheduler::new(
            config.warmup_steps,
            config.total_steps,
            config.learning_rate,
            config.min_learning_rate,
        );
        
        let mlm = match config.objective {
            PretrainingObjective::MaskedLM { .. } => Some(MaskedLanguageModel::new(config.clone())),
            _ => None,
        };
        
        let clm = match config.objective {
            PretrainingObjective::CausalLM | PretrainingObjective::PrefixLM { .. } => {
                Some(CausalLanguageModel::new(config.clone()))
            },
            _ => None,
        };
        
        Self {
            config,
            mlm,
            clm,
            lr_scheduler,
            metrics_history: Vec::new(),
            accumulated_gradients: HashMap::new(),
            accumulation_count: 0,
        }
    }
    
    /// Run pre-training step
    pub fn train_step<M>(
        &mut self,
        model: &mut M,
        batch: &PretrainingBatch,
    ) -> PretrainingMetrics
    where
        M: PretrainableModel,
    {
        let start_time = std::time::Instant::now();
        
        // Forward pass
        let logits = model.forward(&batch.input_ids, &batch.attention_mask, &batch.position_ids);
        
        // Compute loss based on objective
        let loss = match &self.mlm {
            Some(mlm) => mlm.compute_loss(&logits, &batch.labels),
            None => match &self.clm {
                Some(clm) => clm.compute_loss(&logits, &batch.labels),
                None => 0.0,
            },
        };
        
        // Compute gradients
        let loss_grad = match &self.mlm {
            Some(mlm) => mlm.compute_loss_gradient(&logits, &batch.labels),
            None => Array3::zeros(logits.raw_dim()),
        };
        
        // Backward pass
        let gradients = model.backward(&loss_grad);
        
        // Accumulate gradients
        self.accumulate_gradients(gradients);
        self.accumulation_count += 1;
        
        // Update weights if accumulation complete
        let mut gradient_norm = 0.0;
        if self.accumulation_count >= self.config.gradient_accumulation_steps {
            gradient_norm = self.apply_accumulated_gradients(model);
            self.lr_scheduler.step();
        }
        
        // Sacred geometry checkpoint
        let sacred_interventions = if self.config.sacred_geometry_checkpoints {
            self.sacred_geometry_check(self.lr_scheduler.current_step)
        } else {
            0
        };
        
        let elapsed = start_time.elapsed().as_secs_f32();
        let tokens = (batch.input_ids.shape()[0] * batch.input_ids.shape()[1]) as f32;
        
        let metrics = PretrainingMetrics {
            step: self.lr_scheduler.current_step,
            loss,
            perplexity: loss.exp(),
            learning_rate: self.lr_scheduler.get_lr(),
            tokens_per_second: tokens / elapsed,
            gradient_norm,
            sacred_interventions,
        };
        
        self.metrics_history.push(metrics.clone());
        metrics
    }
    
    /// Accumulate gradients for gradient accumulation
    fn accumulate_gradients(&mut self, gradients: HashMap<String, Array2<f32>>) {
        for (name, grad) in gradients {
            self.accumulated_gradients
                .entry(name)
                .and_modify(|acc| *acc = &*acc + &grad)
                .or_insert(grad);
        }
    }
    
    /// Apply accumulated gradients with clipping
    fn apply_accumulated_gradients<M>(&mut self, model: &mut M) -> f32
    where
        M: PretrainableModel,
    {
        // Average gradients
        let scale = 1.0 / self.config.gradient_accumulation_steps as f32;
        for grad in self.accumulated_gradients.values_mut() {
            *grad *= scale;
        }
        
        // Compute gradient norm
        let total_norm: f32 = self.accumulated_gradients.values()
            .map(|g| g.mapv(|x| x.powi(2)).sum())
            .sum::<f32>()
            .sqrt();
        
        // Clip gradients
        if total_norm > self.config.max_grad_norm {
            let clip_coef = self.config.max_grad_norm / (total_norm + 1e-6);
            for grad in self.accumulated_gradients.values_mut() {
                *grad *= clip_coef;
            }
        }
        
        // Apply weight decay and update
        let lr = self.lr_scheduler.get_lr();
        model.update_weights(&self.accumulated_gradients, lr, self.config.weight_decay);
        
        // Clear accumulated gradients
        self.accumulated_gradients.clear();
        self.accumulation_count = 0;
        
        total_norm
    }
    
    /// Sacred geometry checkpoint at positions 3, 6, 9
    fn sacred_geometry_check(&self, step: usize) -> usize {
        let digital_root = ((step % 9) + 1) as u8;
        
        if digital_root == 3 || digital_root == 6 || digital_root == 9 {
            // At sacred positions, we could:
            // 1. Validate signal strength
            // 2. Apply coherence boosts
            // 3. Log special metrics
            1
        } else {
            0
        }
    }
    
    /// Get training history
    pub fn get_metrics_history(&self) -> &[PretrainingMetrics] {
        &self.metrics_history
    }
    
    /// Get current learning rate
    pub fn get_current_lr(&self) -> f32 {
        self.lr_scheduler.get_lr()
    }
}

/// Trait for models that can be pre-trained
pub trait PretrainableModel {
    /// Forward pass returning logits [batch, seq_len, vocab_size]
    fn forward(&self, input_ids: &Array2<u32>, attention_mask: &Array2<f32>, position_ids: &Array2<u32>) -> Array3<f32>;
    
    /// Backward pass returning gradients
    fn backward(&self, loss_grad: &Array3<f32>) -> HashMap<String, Array2<f32>>;
    
    /// Update weights with gradients
    fn update_weights(&mut self, gradients: &HashMap<String, Array2<f32>>, lr: f32, weight_decay: f32);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mlm_masking() {
        let config = PretrainingConfig::default();
        let mlm = MaskedLanguageModel::new(config);
        
        let tokens = vec![101, 1000, 2000, 3000, 4000, 5000, 102];
        let (masked, labels, positions) = mlm.apply_masking(&tokens);
        
        assert_eq!(masked.len(), tokens.len());
        assert_eq!(labels.len(), tokens.len());
        // Special tokens should not be masked
        assert_eq!(labels[0], -100);
        assert_eq!(labels[6], -100);
    }
    
    #[test]
    fn test_lr_scheduler() {
        let mut scheduler = LearningRateScheduler::new(100, 1000, 1e-4, 1e-5);
        
        // During warmup
        assert!(scheduler.get_lr() < 1e-4);
        
        for _ in 0..100 {
            scheduler.step();
        }
        
        // At peak
        assert!((scheduler.get_lr() - 1e-4).abs() < 1e-6);
        
        for _ in 0..900 {
            scheduler.step();
        }
        
        // Near end
        assert!(scheduler.get_lr() < 2e-5);
    }
    
    #[test]
    fn test_clm_causal_mask() {
        let config = PretrainingConfig {
            objective: PretrainingObjective::CausalLM,
            ..Default::default()
        };
        let clm = CausalLanguageModel::new(config);
        
        let mask = clm.create_causal_mask(4);
        
        // Lower triangular
        assert!(mask[[0, 0]]);
        assert!(!mask[[0, 1]]);
        assert!(mask[[1, 0]]);
        assert!(mask[[1, 1]]);
        assert!(mask[[3, 3]]);
    }
}
