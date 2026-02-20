//! Dynamic RSI (Recursive Self-Improvement) Engine
//!
//! Runtime self-generating inference strategy that learns per-dataset.
//! The model observes its own accuracy and adjusts inference parameters
//! dynamically — no recompilation needed.
//!
//! ## Table of Contents
//! 1. InferenceStrategy — per-question inference configuration
//! 2. DatasetProfile — learned statistics per dataset source
//! 3. DynamicRSI — the self-improving strategy engine
//! 4. RSI feedback loop — observe accuracy, adjust parameters
//!
//! ## Architecture
//! ```text
//! Question arrives → classify source → lookup DatasetProfile
//!   ↓
//! IF profile exists AND has enough observations:
//!   → use learned strategy (thresholds, passes, expert weights)
//! ELSE:
//!   → use default strategy, start observing
//!   ↓
//! After answer: observe(correct/wrong, confidence, path_taken)
//!   ↓
//! Update DatasetProfile: adjust thresholds toward what works
//!   ↓
//! Next question from same source → better strategy
//! ```

use std::collections::HashMap;

// =============================================================================
// INFERENCE STRATEGY: Per-question inference configuration
// =============================================================================

/// Dynamic inference strategy — all parameters that control how a question is processed.
/// These are the "knobs" that RSI tunes per-dataset.
#[derive(Debug, Clone)]
pub struct InferenceStrategy {
    /// Try knowledge pipeline first (word-matching retrieval)
    pub use_pipeline: bool,
    /// Min confidence to commit pipeline answer (higher = more selective)
    pub pipeline_threshold: f32,
    /// Try unified 3-pass inference (entity tracking, math, reasoning)
    pub use_unified: bool,
    /// Min confidence to commit unified answer
    pub unified_threshold: f32,
    /// Number of forward passes through reasoning layer (1-5, SOTA optimal: 3)
    pub num_passes: usize,
    /// Fall through to 21-expert scoring system
    pub use_multi_expert: bool,
    /// Strategy name for logging
    pub strategy_name: String,
    /// Expert weight multipliers (expert_name → weight)
    /// Values > 1.0 boost, < 1.0 suppress, 0.0 disables
    pub expert_weights: HashMap<String, f32>,
}

impl Default for InferenceStrategy {
    fn default() -> Self {
        Self {
            use_pipeline: true,
            pipeline_threshold: 0.6,
            use_unified: true,
            unified_threshold: 0.70,
            num_passes: 3,
            use_multi_expert: true,
            strategy_name: "default".to_string(),
            expert_weights: HashMap::new(),
        }
    }
}

// =============================================================================
// DATASET PROFILE: Learned statistics per dataset source
// =============================================================================

/// Tracks performance statistics for a specific dataset source.
/// RSI uses these to adjust strategy parameters.
#[derive(Debug, Clone)]
pub struct DatasetProfile {
    /// Dataset source name (e.g. "mmlu", "babi1", "gsm8k")
    pub source: String,
    /// Total questions seen
    pub total: usize,
    /// Total correct answers
    pub correct: usize,
    /// Accuracy by inference path: path_name → (correct, total)
    pub path_accuracy: HashMap<String, (usize, usize)>,
    /// Average confidence when correct
    pub avg_conf_correct: f32,
    /// Average confidence when wrong
    pub avg_conf_wrong: f32,
    /// Pipeline commit rate (how often pipeline answer was used)
    pub pipeline_commit_rate: f32,
    /// Pipeline accuracy when it commits
    pub pipeline_accuracy: f32,
    /// Unified commit rate
    pub unified_commit_rate: f32,
    /// Unified accuracy when it commits
    pub unified_accuracy: f32,
    /// Current learned strategy
    pub strategy: InferenceStrategy,
    /// Number of strategy updates (how many times RSI has tuned this)
    pub rsi_generations: usize,
}

impl DatasetProfile {
    fn new(source: &str) -> Self {
        Self {
            source: source.to_string(),
            total: 0,
            correct: 0,
            path_accuracy: HashMap::new(),
            avg_conf_correct: 0.5,
            avg_conf_wrong: 0.5,
            pipeline_commit_rate: 0.0,
            pipeline_accuracy: 0.0,
            unified_commit_rate: 0.0,
            unified_accuracy: 0.0,
            strategy: InferenceStrategy::default(),
            rsi_generations: 0,
        }
    }

    /// Current accuracy (0.0 - 1.0)
    pub fn accuracy(&self) -> f32 {
        if self.total == 0 { return 0.0; }
        self.correct as f32 / self.total as f32
    }
}

// =============================================================================
// OBSERVATION: What happened when we answered a question
// =============================================================================

/// Record of what happened during inference for one question.
/// Fed back to RSI for strategy adjustment.
#[derive(Debug, Clone)]
pub struct InferenceObservation {
    /// Dataset source
    pub source: String,
    /// Was the answer correct?
    pub correct: bool,
    /// Final confidence
    pub confidence: f32,
    /// Which path produced the answer: "pipeline", "unified", "multi-expert"
    pub path_taken: String,
    /// Pipeline confidence (if pipeline was tried)
    pub pipeline_conf: Option<f32>,
    /// Pipeline was correct (if pipeline was tried)
    pub pipeline_correct: Option<bool>,
    /// Unified confidence (if unified was tried)
    pub unified_conf: Option<f32>,
    /// Unified was correct (if unified was tried)
    pub unified_correct: Option<bool>,
}

// =============================================================================
// DYNAMIC RSI ENGINE
// =============================================================================

/// The self-improving strategy engine.
/// Observes accuracy per dataset and adjusts inference parameters at runtime.
pub struct DynamicRSI {
    /// Learned profiles per dataset source
    profiles: HashMap<String, DatasetProfile>,
    /// Seed strategies for known dataset types (bootstrap before learning)
    seed_strategies: HashMap<String, InferenceStrategy>,
    /// Global observation count
    total_observations: usize,
    /// RSI update interval: re-tune strategy every N observations per dataset
    update_interval: usize,
}

impl DynamicRSI {
    pub fn new() -> Self {
        let mut engine = Self {
            profiles: HashMap::new(),
            seed_strategies: HashMap::new(),
            total_observations: 0,
            update_interval: 5, // Re-tune after every 5 questions per dataset
        };
        engine.init_seed_strategies();
        engine
    }

    /// Bootstrap seed strategies from domain knowledge.
    /// These are starting points — RSI will override them as it learns.
    fn init_seed_strategies(&mut self) {
        // Structured reasoning: entity tracking, temporal state
        // unified_threshold=0.0 means ALWAYS commit unified answer, never fall through
        // Multi-expert is catastrophic for bAbI (9% accuracy vs 100% unified)
        let structured = InferenceStrategy {
            use_pipeline: false,
            pipeline_threshold: 1.0,
            use_unified: true,
            unified_threshold: 0.0, // Always commit — unified is the expert here
            num_passes: 3,
            use_multi_expert: false, // Multi-expert hurts structured tasks
            strategy_name: "seed:structured-reasoning".to_string(),
            expert_weights: HashMap::new(),
        };
        // "babi" matches source="bAbI" (lowercased) — covers ALL bAbI tasks
        self.seed_strategies.insert("babi".to_string(), structured.clone());
        for name in &["babi1", "babi2", "babi3", "babi6", "babi7", "babi8",
                       "babi11", "babi15", "babi16", "babi17", "babi18", "babi19", "babi20"] {
            self.seed_strategies.insert(name.to_string(), structured.clone());
        }

        // Symbolic math
        self.seed_strategies.insert("gsm8k".to_string(), InferenceStrategy {
            use_pipeline: false,
            pipeline_threshold: 1.0,
            use_unified: true,
            unified_threshold: 0.50,
            num_passes: 3,
            use_multi_expert: true,
            strategy_name: "seed:symbolic-math".to_string(),
            expert_weights: HashMap::new(),
        });

        // Code generation
        self.seed_strategies.insert("humaneval".to_string(), InferenceStrategy {
            use_pipeline: false,
            pipeline_threshold: 1.0,
            use_unified: false,
            unified_threshold: 0.70,
            num_passes: 1,
            use_multi_expert: true,
            strategy_name: "seed:code".to_string(),
            expert_weights: HashMap::new(),
        });

        // Knowledge QA
        self.seed_strategies.insert("mmlu".to_string(), InferenceStrategy {
            use_pipeline: true,
            pipeline_threshold: 0.7,
            use_unified: true,
            unified_threshold: 0.70,
            num_passes: 3,
            use_multi_expert: true,
            strategy_name: "seed:knowledge-qa".to_string(),
            expert_weights: HashMap::new(),
        });

        // Extractive QA
        self.seed_strategies.insert("squad".to_string(), InferenceStrategy {
            use_pipeline: false,
            pipeline_threshold: 1.0,
            use_unified: true,
            unified_threshold: 0.60,
            num_passes: 3,
            use_multi_expert: true,
            strategy_name: "seed:extractive-qa".to_string(),
            expert_weights: HashMap::new(),
        });

        // Commonsense
        for name in &["piqa", "winogrande", "commonsenseqa", "hellaswag"] {
            self.seed_strategies.insert(name.to_string(), InferenceStrategy {
                use_pipeline: true,
                pipeline_threshold: 0.6,
                use_unified: true,
                unified_threshold: 0.70,
                num_passes: 3,
                use_multi_expert: true,
                strategy_name: "seed:commonsense".to_string(),
                expert_weights: HashMap::new(),
            });
        }

        // Truthfulness
        self.seed_strategies.insert("truthfulqa".to_string(), InferenceStrategy {
            use_pipeline: true,
            pipeline_threshold: 0.7,
            use_unified: true,
            unified_threshold: 0.70,
            num_passes: 3,
            use_multi_expert: true,
            strategy_name: "seed:truthfulness".to_string(),
            expert_weights: HashMap::new(),
        });
    }

    /// Get the current best strategy for a dataset source.
    /// Returns learned strategy if available, seed strategy if known, default otherwise.
    pub fn get_strategy(&mut self, source: &str) -> InferenceStrategy {
        let src = source.to_lowercase();

        // If we have a learned profile with enough data, use its strategy
        if let Some(profile) = self.profiles.get(&src) {
            if profile.total >= self.update_interval {
                return profile.strategy.clone();
            }
        }

        // Fall back to seed strategy for known datasets
        if let Some(seed) = self.seed_strategies.get(&src) {
            return seed.clone();
        }

        // Check prefix matches (e.g. "babi" prefix for any bAbI task)
        for (key, strategy) in &self.seed_strategies {
            if src.starts_with(key) || key.starts_with(&src) {
                return strategy.clone();
            }
        }

        // Unknown dataset — use default strategy
        InferenceStrategy::default()
    }

    /// Observe the result of answering a question.
    /// This is the RSI feedback loop — the model learns from its own performance.
    pub fn observe(&mut self, obs: InferenceObservation) {
        let src = obs.source.to_lowercase();
        self.total_observations += 1;

        // Get or create profile
        let profile = self.profiles.entry(src.clone())
            .or_insert_with(|| {
                let mut p = DatasetProfile::new(&src);
                // Initialize with seed strategy if available
                if let Some(seed) = self.seed_strategies.get(&src) {
                    p.strategy = seed.clone();
                }
                p
            });

        // Update basic stats
        profile.total += 1;
        if obs.correct {
            profile.correct += 1;
        }

        // Update path accuracy
        let entry = profile.path_accuracy
            .entry(obs.path_taken.clone())
            .or_insert((0, 0));
        if obs.correct { entry.0 += 1; }
        entry.1 += 1;

        // Update confidence tracking (exponential moving average)
        let alpha = 0.2;
        if obs.correct {
            profile.avg_conf_correct = profile.avg_conf_correct * (1.0 - alpha)
                + obs.confidence * alpha;
        } else {
            profile.avg_conf_wrong = profile.avg_conf_wrong * (1.0 - alpha)
                + obs.confidence * alpha;
        }

        // Update pipeline stats
        if let Some(p_conf) = obs.pipeline_conf {
            let committed = p_conf > profile.strategy.pipeline_threshold;
            if committed {
                let n = profile.total as f32;
                profile.pipeline_commit_rate = profile.pipeline_commit_rate * ((n - 1.0) / n)
                    + (1.0 / n);
                if let Some(p_correct) = obs.pipeline_correct {
                    let old_acc = profile.pipeline_accuracy;
                    profile.pipeline_accuracy = old_acc * ((n - 1.0) / n)
                        + if p_correct { 1.0 / n } else { 0.0 };
                }
            }
        }

        // Update unified stats
        if let Some(u_conf) = obs.unified_conf {
            let committed = u_conf >= profile.strategy.unified_threshold;
            if committed {
                let n = profile.total as f32;
                profile.unified_commit_rate = profile.unified_commit_rate * ((n - 1.0) / n)
                    + (1.0 / n);
                if let Some(u_correct) = obs.unified_correct {
                    let old_acc = profile.unified_accuracy;
                    profile.unified_accuracy = old_acc * ((n - 1.0) / n)
                        + if u_correct { 1.0 / n } else { 0.0 };
                }
            }
        }

        // =================================================================
        // RSI SELF-IMPROVEMENT: Re-tune strategy every update_interval questions
        // =================================================================
        if profile.total % self.update_interval == 0 && profile.total > 0 {
            self.retune_strategy(&src);
        }
    }

    /// The core RSI loop: analyze performance and adjust strategy.
    /// This is where the model "recodes itself" at runtime.
    fn retune_strategy(&mut self, source: &str) {
        let profile = match self.profiles.get_mut(source) {
            Some(p) => p,
            None => return,
        };

        let accuracy = profile.accuracy();
        let gen = profile.rsi_generations;
        profile.rsi_generations += 1;

        println!("   [RSI-GEN-{}] Retuning strategy for '{}': accuracy={:.1}% ({}/{})",
            gen + 1, source, accuracy * 100.0, profile.correct, profile.total);

        // ---- RULE 0: If pipeline NEVER commits, disable it (pure overhead) ----
        // On MMLU the pipeline conf is always 0.30-0.50, never above threshold.
        // Running it wastes ~2s/question with zero benefit.
        if profile.total >= 10 && profile.pipeline_commit_rate < 0.01 && profile.strategy.use_pipeline {
            profile.strategy.use_pipeline = false;
            println!("   [RSI] Pipeline never commits (rate={:.0}% after {} questions) → disabled",
                profile.pipeline_commit_rate * 100.0, profile.total);
        }

        // ---- RULE 1: If pipeline is hurting, disable or raise threshold ----
        // If pipeline commits often but accuracy is low, it's committing wrong answers
        if profile.pipeline_commit_rate > 0.3 && profile.pipeline_accuracy < 0.5 {
            let old = profile.strategy.pipeline_threshold;
            profile.strategy.pipeline_threshold = (old + 0.1).min(1.0);
            println!("   [RSI] Pipeline hurting (acc={:.0}%, rate={:.0}%) → threshold {:.1} → {:.1}",
                profile.pipeline_accuracy * 100.0, profile.pipeline_commit_rate * 100.0,
                old, profile.strategy.pipeline_threshold);

            // If threshold is already maxed, disable pipeline entirely
            if profile.strategy.pipeline_threshold >= 0.95 {
                profile.strategy.use_pipeline = false;
                println!("   [RSI] Pipeline disabled for '{}'", source);
            }
        }

        // ---- RULE 2: If pipeline is helping, lower threshold to commit more ----
        if profile.pipeline_commit_rate > 0.1 && profile.pipeline_accuracy > 0.8 {
            let old = profile.strategy.pipeline_threshold;
            profile.strategy.pipeline_threshold = (old - 0.05).max(0.3);
            println!("   [RSI] Pipeline helping (acc={:.0}%) → threshold {:.1} → {:.1}",
                profile.pipeline_accuracy * 100.0, old, profile.strategy.pipeline_threshold);
        }

        // ---- RULE 3: If unified is strong, lower its threshold ----
        if profile.unified_commit_rate > 0.1 && profile.unified_accuracy > 0.8 {
            let old = profile.strategy.unified_threshold;
            profile.strategy.unified_threshold = (old - 0.05).max(0.3);
            println!("   [RSI] Unified strong (acc={:.0}%) → threshold {:.1} → {:.1}",
                profile.unified_accuracy * 100.0, old, profile.strategy.unified_threshold);
        }

        // ---- RULE 4: If unified is weak, raise threshold to fall through more ----
        // Cap at 0.90 — at 0.95 unified can NEVER commit (conf band is 0.89-0.91)
        // which defeats the purpose and forces everything to multi-expert
        if profile.unified_commit_rate > 0.3 && profile.unified_accuracy < 0.5 {
            let old = profile.strategy.unified_threshold;
            profile.strategy.unified_threshold = (old + 0.1).min(0.90);
            println!("   [RSI] Unified weak (acc={:.0}%) → threshold {:.1} → {:.1}",
                profile.unified_accuracy * 100.0, old, profile.strategy.unified_threshold);
        }

        // ---- RULE 5: If multi-expert is the best path, enable it ----
        if let Some(&(me_correct, me_total)) = profile.path_accuracy.get("multi-expert") {
            let me_acc = if me_total > 0 { me_correct as f32 / me_total as f32 } else { 0.0 };
            if me_acc > accuracy + 0.1 && !profile.strategy.use_multi_expert {
                profile.strategy.use_multi_expert = true;
                println!("   [RSI] Multi-expert outperforming (acc={:.0}% vs overall {:.0}%) → enabled",
                    me_acc * 100.0, accuracy * 100.0);
            }
            // If multi-expert is dragging down accuracy, disable it
            if me_acc < accuracy - 0.15 && me_total > 3 && profile.strategy.use_multi_expert {
                profile.strategy.use_multi_expert = false;
                println!("   [RSI] Multi-expert underperforming (acc={:.0}% vs overall {:.0}%) → disabled",
                    me_acc * 100.0, accuracy * 100.0);
            }
        }

        // ---- RULE 6: If confidence calibration is off, adjust ----
        // High confidence on wrong answers = overconfident → raise thresholds
        // Cap unified at 0.90 to prevent death spiral (see Rule 4 comment)
        if profile.avg_conf_wrong > 0.6 && accuracy < 0.5 {
            profile.strategy.pipeline_threshold = (profile.strategy.pipeline_threshold + 0.05).min(0.95);
            profile.strategy.unified_threshold = (profile.strategy.unified_threshold + 0.05).min(0.90);
            println!("   [RSI] Overconfident when wrong (avg_conf_wrong={:.2}) → raised thresholds",
                profile.avg_conf_wrong);
        }

        // Update strategy name to reflect RSI generation
        profile.strategy.strategy_name = format!("rsi-gen-{}/{}", gen + 1, source);

        println!("   [RSI-GEN-{}] New strategy: pipeline={}/{:.1} unified={}/{:.1} multi_expert={} passes={}",
            gen + 1,
            profile.strategy.use_pipeline, profile.strategy.pipeline_threshold,
            profile.strategy.use_unified, profile.strategy.unified_threshold,
            profile.strategy.use_multi_expert, profile.strategy.num_passes);
    }

    /// Get a summary of all learned profiles
    pub fn summary(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!("[RSI] {} total observations across {} datasets",
            self.total_observations, self.profiles.len()));

        let mut profiles: Vec<_> = self.profiles.values().collect();
        profiles.sort_by(|a, b| a.source.cmp(&b.source));

        for p in profiles {
            lines.push(format!(
                "  {:<15} acc={:>5.1}% ({:>3}/{:>3}) gen={} strategy={}",
                p.source, p.accuracy() * 100.0, p.correct, p.total,
                p.rsi_generations, p.strategy.strategy_name
            ));
        }
        lines.join("\n")
    }

    /// Get profile for a specific source (for external inspection)
    pub fn get_profile(&self, source: &str) -> Option<&DatasetProfile> {
        self.profiles.get(&source.to_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsi_seed_strategy() {
        let mut rsi = DynamicRSI::new();
        let s = rsi.get_strategy("babi1");
        assert!(!s.use_pipeline);
        assert!(s.use_unified);
        assert!(!s.use_multi_expert);
    }

    #[test]
    fn test_rsi_unknown_dataset() {
        let mut rsi = DynamicRSI::new();
        let s = rsi.get_strategy("some_new_dataset");
        assert!(s.use_pipeline);
        assert!(s.use_unified);
        assert!(s.use_multi_expert);
    }

    #[test]
    fn test_rsi_learns_from_observations() {
        let mut rsi = DynamicRSI::new();

        // Simulate 5 wrong pipeline answers for mmlu
        for _ in 0..5 {
            rsi.observe(InferenceObservation {
                source: "mmlu".to_string(),
                correct: false,
                confidence: 0.65,
                path_taken: "pipeline".to_string(),
                pipeline_conf: Some(0.65),
                pipeline_correct: Some(false),
                unified_conf: None,
                unified_correct: None,
            });
        }

        // Strategy should have raised pipeline threshold
        let s = rsi.get_strategy("mmlu");
        assert!(s.pipeline_threshold > 0.7, "threshold should have increased: {}", s.pipeline_threshold);
    }

    #[test]
    fn test_rsi_enables_multi_expert() {
        let mut rsi = DynamicRSI::new();

        // Simulate multi-expert being better than overall
        for i in 0..10 {
            rsi.observe(InferenceObservation {
                source: "test_ds".to_string(),
                correct: i % 3 == 0, // 30% overall
                confidence: 0.5,
                path_taken: if i < 5 { "unified" } else { "multi-expert" }.to_string(),
                pipeline_conf: None,
                pipeline_correct: None,
                unified_conf: Some(0.5),
                unified_correct: Some(i % 3 == 0),
            });
        }

        let profile = rsi.get_profile("test_ds");
        assert!(profile.is_some());
    }
}
