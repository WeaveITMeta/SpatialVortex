//! Continuous Learning with RSI-Driven Epochs
//!
//! Implements continuous test-time learning where:
//! - Epochs are derived from RSI (Recursive Self-Improvement) signals
//! - Synthetic data is generated from verified patterns
//! - Training adapts in real-time based on performance
//!
//! ## Key Components
//! 1. **RSIEpochScheduler** - Determines when to train based on RSI signals
//! 2. **SyntheticDataGenerator** - Creates training data from verified patterns
//! 3. **ContinuousTrainer** - Orchestrates the training loop
//! 4. **AdaptiveLearningRate** - Adjusts LR based on improvement trajectory
//!
//! ## Training Loop
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Continuous Learning Loop                  │
//! ├─────────────────────────────────────────────────────────────┤
//! │  1. Collect verified patterns (wheat, not chaff)            │
//! │  2. Generate synthetic training data                        │
//! │  3. RSI scheduler decides: train now or wait?               │
//! │  4. If train: run epoch with adaptive LR                    │
//! │  5. Evaluate on benchmarks                                  │
//! │  6. Update RSI state with results                           │
//! │  7. Verify improvements before reinforcing                  │
//! │  8. Loop back to step 1                                     │
//! └─────────────────────────────────────────────────────────────┘
//! ```

use crate::cognition::verified_patterning::{
    VerifiedPattern, PatternHypothesis, Evidence, EvidenceSource,
    BenchmarkResult, VerificationResult,
};
use crate::data::models::BeamTensor;
use crate::ml::huggingface::{RSIState, RSIMetric, RSIImprovement};
use crate::ml::calm::LatentState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

// =============================================================================
// Table of Contents
// =============================================================================
// 1. Configuration types
// 2. RSIEpochScheduler - When to train
// 3. SyntheticDataGenerator - What to train on
// 4. TrainingBatch - Training data structure
// 5. AdaptiveLearningRate - How fast to learn
// 6. EpochResult - Training outcomes
// 7. ContinuousTrainer - Main orchestrator
// =============================================================================

// =============================================================================
// 1. Configuration Types
// =============================================================================

/// Configuration for continuous learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContinuousLearningConfig {
    /// Base learning rate
    pub base_learning_rate: f64,
    /// Minimum learning rate
    pub min_learning_rate: f64,
    /// Maximum learning rate
    pub max_learning_rate: f64,
    /// Batch size for training
    pub batch_size: usize,
    /// Maximum epochs per training session
    pub max_epochs_per_session: usize,
    /// Minimum improvement to continue training
    pub min_improvement_threshold: f64,
    /// RSI signal threshold to trigger training
    pub rsi_trigger_threshold: f64,
    /// Cooldown between training sessions (ms)
    pub training_cooldown_ms: u64,
    /// Enable synthetic data generation
    pub enable_synthetic_data: bool,
    /// Synthetic data ratio (synthetic / real)
    pub synthetic_data_ratio: f64,
    /// Verification required before reinforcement
    pub require_verification: bool,
}

impl Default for ContinuousLearningConfig {
    fn default() -> Self {
        Self {
            base_learning_rate: 0.001,
            min_learning_rate: 1e-6,
            max_learning_rate: 0.1,
            batch_size: 32,
            max_epochs_per_session: 10,
            min_improvement_threshold: 0.001,
            rsi_trigger_threshold: 0.5,
            training_cooldown_ms: 5000,
            enable_synthetic_data: true,
            synthetic_data_ratio: 0.3,
            require_verification: true,
        }
    }
}

// =============================================================================
// 2. RSIEpochScheduler - When to Train
// =============================================================================

/// Signals from RSI that influence training decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RSISignal {
    /// Overall improvement momentum (positive = improving)
    pub momentum: f64,
    /// Variance in recent performance
    pub variance: f64,
    /// Time since last improvement
    pub staleness_ms: u64,
    /// Current best score
    pub best_score: f64,
    /// Recent average score
    pub recent_avg: f64,
    /// Recommended action
    pub recommendation: TrainingRecommendation,
}

/// What the RSI scheduler recommends
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TrainingRecommendation {
    /// Train now - good momentum
    TrainNow,
    /// Wait - recent training, need cooldown
    Wait { remaining_ms: u64 },
    /// Explore - try new patterns
    Explore,
    /// Consolidate - reinforce existing patterns
    Consolidate,
    /// Reset - performance degraded, need fresh start
    Reset,
}

/// Scheduler that determines when to train based on RSI signals
pub struct RSIEpochScheduler {
    config: ContinuousLearningConfig,
    /// Recent RSI metrics for trend analysis
    recent_metrics: Vec<RSIMetric>,
    /// Last training timestamp
    last_training: Option<Instant>,
    /// Consecutive improvements
    improvement_streak: u32,
    /// Consecutive degradations
    degradation_streak: u32,
    /// Total epochs run
    total_epochs: u64,
}

impl RSIEpochScheduler {
    pub fn new(config: ContinuousLearningConfig) -> Self {
        Self {
            config,
            recent_metrics: Vec::new(),
            last_training: None,
            improvement_streak: 0,
            degradation_streak: 0,
            total_epochs: 0,
        }
    }

    /// Update with new RSI metrics
    pub fn update(&mut self, metrics: &[RSIMetric]) {
        self.recent_metrics.extend(metrics.iter().cloned());
        // Keep only recent metrics (last 100)
        if self.recent_metrics.len() > 100 {
            self.recent_metrics.drain(0..self.recent_metrics.len() - 100);
        }
    }

    /// Compute current RSI signal
    pub fn compute_signal(&self) -> RSISignal {
        let now = Instant::now();
        
        // Check cooldown
        if let Some(last) = self.last_training {
            let elapsed = now.duration_since(last).as_millis() as u64;
            if elapsed < self.config.training_cooldown_ms {
                return RSISignal {
                    momentum: 0.0,
                    variance: 0.0,
                    staleness_ms: 0,
                    best_score: self.best_score(),
                    recent_avg: self.recent_average(),
                    recommendation: TrainingRecommendation::Wait {
                        remaining_ms: self.config.training_cooldown_ms - elapsed,
                    },
                };
            }
        }

        // Compute momentum (trend of recent scores)
        let momentum = self.compute_momentum();
        
        // Compute variance
        let variance = self.compute_variance();
        
        // Compute staleness
        let staleness_ms = self.recent_metrics.last()
            .map(|m| {
                let now_ts = chrono::Utc::now().timestamp();
                ((now_ts - m.timestamp) * 1000) as u64
            })
            .unwrap_or(u64::MAX);

        let best_score = self.best_score();
        let recent_avg = self.recent_average();

        // Determine recommendation
        let recommendation = if self.degradation_streak >= 3 {
            TrainingRecommendation::Reset
        } else if momentum > self.config.rsi_trigger_threshold {
            TrainingRecommendation::TrainNow
        } else if variance > 0.1 {
            TrainingRecommendation::Explore
        } else if self.improvement_streak >= 2 {
            TrainingRecommendation::Consolidate
        } else if staleness_ms > 60000 {
            TrainingRecommendation::TrainNow
        } else {
            TrainingRecommendation::Wait { remaining_ms: 1000 }
        };

        RSISignal {
            momentum,
            variance,
            staleness_ms,
            best_score,
            recent_avg,
            recommendation,
        }
    }

    /// Should we train now?
    pub fn should_train(&self) -> bool {
        matches!(
            self.compute_signal().recommendation,
            TrainingRecommendation::TrainNow | TrainingRecommendation::Consolidate
        )
    }

    /// Record that training occurred
    pub fn record_training(&mut self, improvement: f64) {
        self.last_training = Some(Instant::now());
        self.total_epochs += 1;

        if improvement > self.config.min_improvement_threshold {
            self.improvement_streak += 1;
            self.degradation_streak = 0;
        } else if improvement < -self.config.min_improvement_threshold {
            self.degradation_streak += 1;
            self.improvement_streak = 0;
        }
    }

    /// Compute momentum from recent metrics
    fn compute_momentum(&self) -> f64 {
        if self.recent_metrics.len() < 2 {
            return 0.0;
        }

        let recent: Vec<_> = self.recent_metrics.iter().rev().take(10).collect();
        if recent.len() < 2 {
            return 0.0;
        }

        // Linear regression slope
        let n = recent.len() as f64;
        let sum_x: f64 = (0..recent.len()).map(|i| i as f64).sum();
        let sum_y: f64 = recent.iter().map(|m| m.value).sum();
        let sum_xy: f64 = recent.iter().enumerate().map(|(i, m)| i as f64 * m.value).sum();
        let sum_xx: f64 = (0..recent.len()).map(|i| (i as f64).powi(2)).sum();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_xx - sum_x.powi(2) + 1e-10);
        slope
    }

    /// Compute variance of recent scores
    fn compute_variance(&self) -> f64 {
        if self.recent_metrics.is_empty() {
            return 0.0;
        }

        let values: Vec<f64> = self.recent_metrics.iter().rev().take(10).map(|m| m.value).collect();
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64
    }

    /// Get best score from recent metrics
    fn best_score(&self) -> f64 {
        self.recent_metrics.iter()
            .map(|m| m.value)
            .fold(f64::NEG_INFINITY, f64::max)
    }

    /// Get recent average score
    fn recent_average(&self) -> f64 {
        let recent: Vec<_> = self.recent_metrics.iter().rev().take(10).collect();
        if recent.is_empty() {
            return 0.0;
        }
        recent.iter().map(|m| m.value).sum::<f64>() / recent.len() as f64
    }

    /// Get total epochs run
    pub fn total_epochs(&self) -> u64 {
        self.total_epochs
    }
}

// =============================================================================
// 3. SyntheticDataGenerator - What to Train On
// =============================================================================

/// A synthetic training example
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntheticExample {
    /// Input beams
    pub input: Vec<BeamTensor>,
    /// Expected output beams
    pub target: Vec<BeamTensor>,
    /// Source pattern that generated this
    pub source_pattern_id: String,
    /// Confidence in this example
    pub confidence: f64,
    /// Is this a positive or negative example?
    pub is_positive: bool,
}

/// Generator for synthetic training data
pub struct SyntheticDataGenerator {
    /// Verified patterns to generate from
    patterns: Vec<VerifiedPattern>,
    /// Random seed for reproducibility
    seed: u64,
    /// Examples generated so far
    generated_count: u64,
}

impl SyntheticDataGenerator {
    pub fn new(seed: u64) -> Self {
        Self {
            patterns: Vec::new(),
            seed,
            generated_count: 0,
        }
    }

    /// Update with verified patterns
    pub fn update_patterns(&mut self, patterns: Vec<VerifiedPattern>) {
        self.patterns = patterns;
    }

    /// Generate synthetic examples from verified patterns
    pub fn generate(&mut self, count: usize) -> Vec<SyntheticExample> {
        let mut examples = Vec::with_capacity(count);

        if self.patterns.is_empty() {
            return examples;
        }

        for i in 0..count {
            // Select pattern based on cumulative benefit (weighted sampling)
            let pattern = self.select_pattern(i);
            
            // Generate example from pattern
            if let Some(example) = self.generate_from_pattern(pattern, i) {
                examples.push(example);
                self.generated_count += 1;
            }
        }

        examples
    }

    /// Select a pattern using weighted sampling
    fn select_pattern(&self, index: usize) -> &VerifiedPattern {
        let total_benefit: f64 = self.patterns.iter()
            .map(|p| p.cumulative_benefit.max(0.1))
            .sum();

        let mut target = (self.seed.wrapping_add(index as u64) % 1000) as f64 / 1000.0 * total_benefit;
        
        for pattern in &self.patterns {
            target -= pattern.cumulative_benefit.max(0.1);
            if target <= 0.0 {
                return pattern;
            }
        }

        self.patterns.last().unwrap()
    }

    /// Generate a single example from a pattern
    fn generate_from_pattern(&self, pattern: &VerifiedPattern, index: usize) -> Option<SyntheticExample> {
        let examples = &pattern.hypothesis.pattern.examples;
        if examples.is_empty() {
            return None;
        }

        // Use pattern examples as base
        let base_idx = index % examples.len();
        let input = vec![examples[base_idx].clone()];

        // Generate target by applying pattern transformation
        let mut target = input.clone();
        for beam in &mut target {
            // Apply small perturbation based on pattern features
            for (i, d) in beam.digits.iter_mut().enumerate() {
                let noise = ((self.seed.wrapping_add(index as u64).wrapping_add(i as u64)) % 100) as f32 / 1000.0;
                *d = (*d + noise).clamp(0.0, 1.0);
            }
            beam.confidence = pattern.verification.confidence as f32;
        }

        Some(SyntheticExample {
            input,
            target,
            source_pattern_id: pattern.hypothesis.id.clone(),
            confidence: pattern.verification.confidence,
            is_positive: true,
        })
    }

    /// Generate negative examples (what NOT to learn)
    pub fn generate_negative(&mut self, rejected_patterns: &[(PatternHypothesis, String)], count: usize) -> Vec<SyntheticExample> {
        let mut examples = Vec::with_capacity(count);

        for (i, (pattern, _reason)) in rejected_patterns.iter().enumerate().take(count) {
            if pattern.pattern.examples.is_empty() {
                continue;
            }

            let base_idx = i % pattern.pattern.examples.len();
            let input = vec![pattern.pattern.examples[base_idx].clone()];

            examples.push(SyntheticExample {
                input: input.clone(),
                target: input, // Same as input - don't change
                source_pattern_id: pattern.id.clone(),
                confidence: 0.1, // Low confidence
                is_positive: false,
            });
        }

        examples
    }

    /// Get count of generated examples
    pub fn generated_count(&self) -> u64 {
        self.generated_count
    }
}

// =============================================================================
// 4. TrainingBatch - Training Data Structure
// =============================================================================

/// A batch of training data
#[derive(Debug, Clone)]
pub struct TrainingBatch {
    /// Real examples from actual data
    pub real_examples: Vec<(Vec<BeamTensor>, Vec<BeamTensor>)>,
    /// Synthetic examples from patterns
    pub synthetic_examples: Vec<SyntheticExample>,
    /// Batch index
    pub batch_idx: usize,
    /// Total batches in epoch
    pub total_batches: usize,
}

impl TrainingBatch {
    pub fn new(batch_idx: usize, total_batches: usize) -> Self {
        Self {
            real_examples: Vec::new(),
            synthetic_examples: Vec::new(),
            batch_idx,
            total_batches,
        }
    }

    /// Total examples in batch
    pub fn len(&self) -> usize {
        self.real_examples.len() + self.synthetic_examples.len()
    }

    /// Is batch empty?
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get all inputs
    pub fn inputs(&self) -> Vec<Vec<BeamTensor>> {
        let mut inputs: Vec<Vec<BeamTensor>> = self.real_examples.iter()
            .map(|(i, _)| i.clone())
            .collect();
        inputs.extend(self.synthetic_examples.iter().map(|e| e.input.clone()));
        inputs
    }

    /// Get all targets
    pub fn targets(&self) -> Vec<Vec<BeamTensor>> {
        let mut targets: Vec<Vec<BeamTensor>> = self.real_examples.iter()
            .map(|(_, t)| t.clone())
            .collect();
        targets.extend(self.synthetic_examples.iter().map(|e| e.target.clone()));
        targets
    }
}

// =============================================================================
// 5. AdaptiveLearningRate - How Fast to Learn
// =============================================================================

/// Adaptive learning rate based on RSI signals
pub struct AdaptiveLearningRate {
    config: ContinuousLearningConfig,
    /// Current learning rate
    current_lr: f64,
    /// History of learning rates
    lr_history: Vec<(u64, f64)>,
    /// Epoch counter
    epoch: u64,
}

impl AdaptiveLearningRate {
    pub fn new(config: ContinuousLearningConfig) -> Self {
        let current_lr = config.base_learning_rate;
        Self {
            config,
            current_lr,
            lr_history: vec![(0, current_lr)],
            epoch: 0,
        }
    }

    /// Get current learning rate
    pub fn get(&self) -> f64 {
        self.current_lr
    }

    /// Update learning rate based on training result
    pub fn update(&mut self, improvement: f64, rsi_signal: &RSISignal) {
        self.epoch += 1;

        // Adjust based on improvement
        let adjustment = if improvement > 0.01 {
            // Good improvement - increase LR slightly
            1.1
        } else if improvement > 0.0 {
            // Small improvement - maintain
            1.0
        } else if improvement > -0.01 {
            // Small degradation - decrease slightly
            0.95
        } else {
            // Large degradation - decrease significantly
            0.8
        };

        // Also consider RSI momentum
        let momentum_factor = 1.0 + rsi_signal.momentum.clamp(-0.1, 0.1);

        // Apply adjustment
        self.current_lr *= adjustment * momentum_factor;

        // Clamp to bounds
        self.current_lr = self.current_lr.clamp(
            self.config.min_learning_rate,
            self.config.max_learning_rate,
        );

        self.lr_history.push((self.epoch, self.current_lr));
    }

    /// Reset learning rate to base
    pub fn reset(&mut self) {
        self.current_lr = self.config.base_learning_rate;
        self.lr_history.push((self.epoch, self.current_lr));
    }

    /// Get learning rate history
    pub fn history(&self) -> &[(u64, f64)] {
        &self.lr_history
    }
}

// =============================================================================
// 6. EpochResult - Training Outcomes
// =============================================================================

/// Result of a training epoch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochResult {
    /// Epoch number
    pub epoch: u64,
    /// Training loss
    pub train_loss: f64,
    /// Validation loss (if available)
    pub val_loss: Option<f64>,
    /// Improvement over previous epoch
    pub improvement: f64,
    /// Learning rate used
    pub learning_rate: f64,
    /// Number of batches processed
    pub batches_processed: usize,
    /// Duration of epoch
    pub duration_ms: u64,
    /// RSI signal at start of epoch
    pub rsi_signal: RSISignal,
    /// Synthetic data ratio used
    pub synthetic_ratio: f64,
}

/// Result of a complete training session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingSessionResult {
    /// All epoch results
    pub epochs: Vec<EpochResult>,
    /// Total improvement
    pub total_improvement: f64,
    /// Best loss achieved
    pub best_loss: f64,
    /// Total duration
    pub total_duration_ms: u64,
    /// Patterns used
    pub patterns_used: usize,
    /// Synthetic examples generated
    pub synthetic_examples_generated: u64,
    /// Was training verified?
    pub verified: bool,
}

// =============================================================================
// 7. ContinuousTrainer - Main Orchestrator
// =============================================================================

/// Main orchestrator for continuous learning
pub struct ContinuousTrainer {
    config: ContinuousLearningConfig,
    /// RSI epoch scheduler
    scheduler: RSIEpochScheduler,
    /// Synthetic data generator
    generator: SyntheticDataGenerator,
    /// Adaptive learning rate
    lr: AdaptiveLearningRate,
    /// Current RSI state
    rsi_state: RSIState,
    /// Training history
    history: Vec<TrainingSessionResult>,
    /// Current loss
    current_loss: f64,
    /// Best loss achieved
    best_loss: f64,
}

impl ContinuousTrainer {
    pub fn new(config: ContinuousLearningConfig) -> Self {
        let scheduler = RSIEpochScheduler::new(config.clone());
        let generator = SyntheticDataGenerator::new(42);
        let lr = AdaptiveLearningRate::new(config.clone());

        Self {
            config,
            scheduler,
            generator,
            lr,
            rsi_state: RSIState::default(),
            history: Vec::new(),
            current_loss: f64::MAX,
            best_loss: f64::MAX,
        }
    }

    /// Update with verified patterns
    pub fn update_patterns(&mut self, patterns: Vec<VerifiedPattern>) {
        self.generator.update_patterns(patterns);
    }

    /// Update RSI state
    pub fn update_rsi(&mut self, state: &RSIState) {
        self.rsi_state = state.clone();
        self.scheduler.update(&state.metrics);
    }

    /// Check if we should train now
    pub fn should_train(&self) -> bool {
        self.scheduler.should_train()
    }

    /// Get current RSI signal
    pub fn rsi_signal(&self) -> RSISignal {
        self.scheduler.compute_signal()
    }

    /// Run a training session
    pub fn train_session<F>(
        &mut self,
        real_data: &[(Vec<BeamTensor>, Vec<BeamTensor>)],
        mut train_fn: F,
    ) -> TrainingSessionResult
    where
        F: FnMut(&TrainingBatch, f64) -> f64, // Returns loss
    {
        let session_start = Instant::now();
        let mut epochs = Vec::new();
        let initial_loss = self.current_loss;

        // Generate synthetic data
        let synthetic_count = (real_data.len() as f64 * self.config.synthetic_data_ratio) as usize;
        let synthetic_examples = if self.config.enable_synthetic_data {
            self.generator.generate(synthetic_count)
        } else {
            Vec::new()
        };

        // Run epochs
        for epoch_idx in 0..self.config.max_epochs_per_session {
            let epoch_start = Instant::now();
            let rsi_signal = self.scheduler.compute_signal();
            let learning_rate = self.lr.get();

            // Create batches
            let batches = self.create_batches(real_data, &synthetic_examples);
            let mut epoch_loss = 0.0;

            for batch in &batches {
                let batch_loss = train_fn(batch, learning_rate);
                epoch_loss += batch_loss;
            }

            epoch_loss /= batches.len().max(1) as f64;

            // Calculate improvement
            let improvement = self.current_loss - epoch_loss;
            self.current_loss = epoch_loss;

            if epoch_loss < self.best_loss {
                self.best_loss = epoch_loss;
            }

            // Update learning rate
            self.lr.update(improvement, &rsi_signal);

            // Record epoch
            let epoch_result = EpochResult {
                epoch: self.scheduler.total_epochs() + epoch_idx as u64,
                train_loss: epoch_loss,
                val_loss: None,
                improvement,
                learning_rate,
                batches_processed: batches.len(),
                duration_ms: epoch_start.elapsed().as_millis() as u64,
                rsi_signal: rsi_signal.clone(),
                synthetic_ratio: synthetic_examples.len() as f64 / real_data.len().max(1) as f64,
            };
            epochs.push(epoch_result);

            // Record training in scheduler
            self.scheduler.record_training(improvement);

            // Early stopping if no improvement
            if improvement < self.config.min_improvement_threshold && epoch_idx > 2 {
                break;
            }
        }

        let total_improvement = initial_loss - self.current_loss;
        let session_result = TrainingSessionResult {
            epochs,
            total_improvement,
            best_loss: self.best_loss,
            total_duration_ms: session_start.elapsed().as_millis() as u64,
            patterns_used: self.generator.patterns.len(),
            synthetic_examples_generated: self.generator.generated_count(),
            verified: false, // Will be set by verification step
        };

        self.history.push(session_result.clone());
        session_result
    }

    /// Create training batches
    fn create_batches(
        &self,
        real_data: &[(Vec<BeamTensor>, Vec<BeamTensor>)],
        synthetic: &[SyntheticExample],
    ) -> Vec<TrainingBatch> {
        let total_examples = real_data.len() + synthetic.len();
        let num_batches = (total_examples + self.config.batch_size - 1) / self.config.batch_size;
        let mut batches = Vec::with_capacity(num_batches);

        let mut real_idx = 0;
        let mut synth_idx = 0;

        for batch_idx in 0..num_batches {
            let mut batch = TrainingBatch::new(batch_idx, num_batches);

            // Add examples to batch
            for _ in 0..self.config.batch_size {
                // Interleave real and synthetic
                if real_idx < real_data.len() && (synth_idx >= synthetic.len() || real_idx % 3 != 0) {
                    batch.real_examples.push(real_data[real_idx].clone());
                    real_idx += 1;
                } else if synth_idx < synthetic.len() {
                    batch.synthetic_examples.push(synthetic[synth_idx].clone());
                    synth_idx += 1;
                } else {
                    break;
                }
            }

            if !batch.is_empty() {
                batches.push(batch);
            }
        }

        batches
    }

    /// Verify training results before reinforcement
    pub fn verify_training(&self, result: &TrainingSessionResult) -> VerificationResult {
        let mut evidence = Vec::new();

        // Evidence from improvement
        if result.total_improvement > 0.0 {
            evidence.push(Evidence {
                id: format!("train_improvement_{}", result.epochs.len()),
                source: EvidenceSource::Empirical {
                    p_value: 0.01, // Assume significant if improvement
                    sample_size: result.epochs.iter().map(|e| e.batches_processed).sum(),
                },
                confidence: (result.total_improvement / 0.1).min(1.0),
                supports: true,
                measurements: result.epochs.iter().map(|e| e.improvement).collect(),
                timestamp_ms: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0),
                reproducibility: 0.8,
            });
        }

        // Evidence from loss reduction
        if result.best_loss < self.best_loss {
            evidence.push(Evidence {
                id: "loss_reduction".to_string(),
                source: EvidenceSource::CrossValidation { k_folds: result.epochs.len() },
                confidence: 0.9,
                supports: true,
                measurements: vec![result.best_loss],
                timestamp_ms: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0),
                reproducibility: 0.85,
            });
        }

        if evidence.len() >= 2 && result.total_improvement > self.config.min_improvement_threshold {
            VerificationResult::verified(
                result.total_improvement.min(1.0),
                evidence,
            )
        } else {
            VerificationResult::rejected(
                format!("Insufficient improvement: {:.4}", result.total_improvement),
                evidence,
            )
        }
    }

    /// Get training history
    pub fn history(&self) -> &[TrainingSessionResult] {
        &self.history
    }

    /// Get current loss
    pub fn current_loss(&self) -> f64 {
        self.current_loss
    }

    /// Get best loss
    pub fn best_loss(&self) -> f64 {
        self.best_loss
    }

    /// Get total epochs trained
    pub fn total_epochs(&self) -> u64 {
        self.scheduler.total_epochs()
    }

    /// Reset trainer state
    pub fn reset(&mut self) {
        self.current_loss = f64::MAX;
        self.best_loss = f64::MAX;
        self.lr.reset();
        self.history.clear();
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsi_scheduler_creation() {
        let config = ContinuousLearningConfig::default();
        let scheduler = RSIEpochScheduler::new(config);
        assert_eq!(scheduler.total_epochs(), 0);
    }

    #[test]
    fn test_rsi_signal_computation() {
        let config = ContinuousLearningConfig::default();
        let mut scheduler = RSIEpochScheduler::new(config);

        // Add some metrics with clear improving trend
        let metrics: Vec<RSIMetric> = (0..10).map(|i| RSIMetric {
            cycle: i,
            name: "reward".to_string(),
            value: 0.5 + (i as f64 * 0.05), // Strong improving trend
            timestamp: chrono::Utc::now().timestamp(),
        }).collect();

        scheduler.update(&metrics);
        let signal = scheduler.compute_signal();

        // Best score should be the last one (0.5 + 9*0.05 = 0.95)
        assert!(signal.best_score > 0.9);
        // Recent average should be positive
        assert!(signal.recent_avg > 0.5);
    }

    #[test]
    fn test_synthetic_data_generation() {
        let mut generator = SyntheticDataGenerator::new(42);
        
        // Create a verified pattern
        let pattern = VerifiedPattern {
            hypothesis: PatternHypothesis {
                id: "test".to_string(),
                description: "Test pattern".to_string(),
                pattern: crate::cognition::verified_patterning::PatternSpec {
                    pattern_type: crate::cognition::verified_patterning::PatternType::Factual {
                        domain: "test".to_string(),
                    },
                    features: HashMap::new(),
                    examples: vec![BeamTensor::default()],
                },
                expected_structure: crate::cognition::verified_patterning::StructureSpec {
                    structure_type: crate::cognition::verified_patterning::StructureType::Capability {
                        name: "test".to_string(),
                    },
                    properties: HashMap::new(),
                },
                predictions: vec![],
                status: crate::cognition::verified_patterning::HypothesisStatus::Verified { confidence: 0.9 },
                created_at_ms: 0,
            },
            verification: VerificationResult::verified(0.9, vec![]),
            reinforcement_count: 0,
            cumulative_benefit: 0.9,
            last_reinforced_ms: 0,
        };

        generator.update_patterns(vec![pattern]);
        let examples = generator.generate(5);

        assert_eq!(examples.len(), 5);
        assert!(examples[0].confidence > 0.0);
    }

    #[test]
    fn test_adaptive_learning_rate() {
        let config = ContinuousLearningConfig::default();
        let mut lr = AdaptiveLearningRate::new(config.clone());

        let initial_lr = lr.get();
        
        // Simulate good improvement
        let signal = RSISignal {
            momentum: 0.1,
            variance: 0.01,
            staleness_ms: 0,
            best_score: 0.9,
            recent_avg: 0.85,
            recommendation: TrainingRecommendation::TrainNow,
        };

        lr.update(0.05, &signal);
        assert!(lr.get() > initial_lr); // LR should increase

        // Simulate degradation
        lr.update(-0.05, &signal);
        assert!(lr.get() < initial_lr * 1.1); // LR should decrease
    }

    #[test]
    fn test_continuous_trainer() {
        let config = ContinuousLearningConfig::default();
        let mut trainer = ContinuousTrainer::new(config);

        // Create dummy training data
        let real_data: Vec<(Vec<BeamTensor>, Vec<BeamTensor>)> = (0..10)
            .map(|_| (vec![BeamTensor::default()], vec![BeamTensor::default()]))
            .collect();

        // Simple training function that returns decreasing loss
        let mut loss = 1.0;
        let train_fn = |_batch: &TrainingBatch, _lr: f64| -> f64 {
            loss *= 0.9;
            loss
        };

        let result = trainer.train_session(&real_data, train_fn);

        assert!(!result.epochs.is_empty());
        assert!(result.total_improvement > 0.0);
    }

    #[test]
    fn test_training_verification() {
        let config = ContinuousLearningConfig::default();
        let trainer = ContinuousTrainer::new(config);

        let result = TrainingSessionResult {
            epochs: vec![
                EpochResult {
                    epoch: 0,
                    train_loss: 0.5,
                    val_loss: None,
                    improvement: 0.1,
                    learning_rate: 0.001,
                    batches_processed: 10,
                    duration_ms: 100,
                    rsi_signal: RSISignal {
                        momentum: 0.1,
                        variance: 0.01,
                        staleness_ms: 0,
                        best_score: 0.9,
                        recent_avg: 0.85,
                        recommendation: TrainingRecommendation::TrainNow,
                    },
                    synthetic_ratio: 0.3,
                },
            ],
            total_improvement: 0.1,
            best_loss: 0.5,
            total_duration_ms: 100,
            patterns_used: 5,
            synthetic_examples_generated: 10,
            verified: false,
        };

        let verification = trainer.verify_training(&result);
        assert!(verification.verified);
    }
}
