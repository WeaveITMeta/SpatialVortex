//! Verified Patterning: Truth-Grounded Feedback Loop
//!
//! Implements the dual of interpretability with scientific verification:
//! - **Interpretability**: Data → Model (what structure emerges?)
//! - **Patterning**: Model → Data (what data produces this structure?)
//!
//! Key principle: Never reinforce patterns without verification.
//! All feedback must pass through evidence gates before strengthening.
//!
//! ## Scientific Method Integration
//! 1. **Hypothesis**: Pattern X produces structure Y
//! 2. **Prediction**: If we train on X, we should see Y
//! 3. **Experiment**: Actually train/test
//! 4. **Verification**: Compare prediction vs reality
//! 5. **Conclusion**: Only reinforce if verified
//!
//! ## Continuous Test-Time Learning
//! - Real-time adaptation with verification gates
//! - Benchmark-driven optimization for SOTA
//! - Wheat/chaff separation via evidence thresholds

use crate::data::models::BeamTensor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

// =============================================================================
// Table of Contents
// =============================================================================
// 1. Core Types: Evidence, Hypothesis, VerificationResult
// 2. VerificationGate: Scientific method implementation
// 3. PatternHypothesis: Testable claims about data→structure
// 4. VerifiedPattern: Patterns that passed verification
// 5. ContinuousLearner: Test-time learning with verification
// 6. BenchmarkTracker: SOTA progress monitoring
// 7. VerifiedPatterningEngine: Main orchestrator
// =============================================================================

// =============================================================================
// 1. Core Types
// =============================================================================

/// Evidence supporting or refuting a hypothesis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    /// Unique identifier
    pub id: String,
    /// Source of evidence (benchmark, test, external validation)
    pub source: EvidenceSource,
    /// Confidence in this evidence (0.0 to 1.0)
    pub confidence: f64,
    /// Does this support or refute the hypothesis?
    pub supports: bool,
    /// Raw data/measurements
    pub measurements: Vec<f64>,
    /// Timestamp when evidence was collected
    pub timestamp_ms: u64,
    /// Reproducibility score (how consistent across runs)
    pub reproducibility: f64,
}

/// Source of evidence for verification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EvidenceSource {
    /// Standard benchmark (e.g., MMLU, HumanEval)
    Benchmark { name: String, version: String },
    /// Controlled A/B test
    ABTest { control_id: String, variant_id: String },
    /// External ground truth (human labels, known facts)
    GroundTruth { source: String },
    /// Mathematical proof or derivation
    FormalProof { theorem: String },
    /// Empirical observation with statistical significance
    Empirical { p_value: f64, sample_size: usize },
    /// Cross-validation result
    CrossValidation { k_folds: usize },
}

/// Result of verification process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// Was the hypothesis verified?
    pub verified: bool,
    /// Confidence in verification (0.0 to 1.0)
    pub confidence: f64,
    /// Evidence that led to this conclusion
    pub evidence: Vec<Evidence>,
    /// If not verified, why?
    pub rejection_reason: Option<String>,
    /// Suggested next steps
    pub recommendations: Vec<String>,
}

impl VerificationResult {
    pub fn verified(confidence: f64, evidence: Vec<Evidence>) -> Self {
        Self {
            verified: true,
            confidence,
            evidence,
            rejection_reason: None,
            recommendations: vec![],
        }
    }

    pub fn rejected(reason: String, evidence: Vec<Evidence>) -> Self {
        Self {
            verified: false,
            confidence: 0.0,
            evidence,
            rejection_reason: Some(reason),
            recommendations: vec![
                "Review hypothesis assumptions".to_string(),
                "Collect more evidence".to_string(),
                "Consider alternative explanations".to_string(),
            ],
        }
    }
}

// =============================================================================
// 2. VerificationGate: Scientific Method Implementation
// =============================================================================

/// Configuration for verification gates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationConfig {
    /// Minimum confidence to pass verification
    pub min_confidence: f64,
    /// Minimum number of evidence sources required
    pub min_evidence_sources: usize,
    /// Minimum reproducibility score
    pub min_reproducibility: f64,
    /// Maximum p-value for statistical significance
    pub max_p_value: f64,
    /// Require at least one ground truth source?
    pub require_ground_truth: bool,
}

impl Default for VerificationConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.8,
            min_evidence_sources: 2,
            min_reproducibility: 0.7,
            max_p_value: 0.05,
            require_ground_truth: false,
        }
    }
}

/// Gate that patterns must pass before reinforcement
pub struct VerificationGate {
    config: VerificationConfig,
    /// History of verification attempts
    history: Vec<(String, VerificationResult)>,
}

impl VerificationGate {
    pub fn new(config: VerificationConfig) -> Self {
        Self {
            config,
            history: Vec::new(),
        }
    }

    /// Verify a hypothesis against collected evidence
    pub fn verify(&mut self, hypothesis_id: &str, evidence: &[Evidence]) -> VerificationResult {
        // Check minimum evidence sources
        if evidence.len() < self.config.min_evidence_sources {
            let result = VerificationResult::rejected(
                format!(
                    "Insufficient evidence: {} sources, need {}",
                    evidence.len(),
                    self.config.min_evidence_sources
                ),
                evidence.to_vec(),
            );
            self.history.push((hypothesis_id.to_string(), result.clone()));
            return result;
        }

        // Check for ground truth if required
        if self.config.require_ground_truth {
            let has_ground_truth = evidence.iter().any(|e| {
                matches!(e.source, EvidenceSource::GroundTruth { .. } | EvidenceSource::FormalProof { .. })
            });
            if !has_ground_truth {
                let result = VerificationResult::rejected(
                    "No ground truth or formal proof provided".to_string(),
                    evidence.to_vec(),
                );
                self.history.push((hypothesis_id.to_string(), result.clone()));
                return result;
            }
        }

        // Calculate aggregate confidence
        let supporting: Vec<&Evidence> = evidence.iter().filter(|e| e.supports).collect();
        let refuting: Vec<&Evidence> = evidence.iter().filter(|e| !e.supports).collect();

        if supporting.is_empty() {
            let result = VerificationResult::rejected(
                "No supporting evidence found".to_string(),
                evidence.to_vec(),
            );
            self.history.push((hypothesis_id.to_string(), result.clone()));
            return result;
        }

        // Weighted confidence based on evidence quality
        let total_support: f64 = supporting.iter()
            .map(|e| e.confidence * e.reproducibility)
            .sum();
        let total_refute: f64 = refuting.iter()
            .map(|e| e.confidence * e.reproducibility)
            .sum();

        let net_confidence = if total_support + total_refute > 0.0 {
            total_support / (total_support + total_refute)
        } else {
            0.0
        };

        // Check reproducibility
        let avg_reproducibility: f64 = supporting.iter()
            .map(|e| e.reproducibility)
            .sum::<f64>() / supporting.len() as f64;

        if avg_reproducibility < self.config.min_reproducibility {
            let result = VerificationResult::rejected(
                format!(
                    "Low reproducibility: {:.2}, need {:.2}",
                    avg_reproducibility, self.config.min_reproducibility
                ),
                evidence.to_vec(),
            );
            self.history.push((hypothesis_id.to_string(), result.clone()));
            return result;
        }

        // Check statistical significance for empirical evidence
        for e in evidence {
            if let EvidenceSource::Empirical { p_value, .. } = &e.source {
                if *p_value > self.config.max_p_value {
                    let result = VerificationResult::rejected(
                        format!(
                            "Statistical significance not met: p={:.4}, need p<{:.4}",
                            p_value, self.config.max_p_value
                        ),
                        evidence.to_vec(),
                    );
                    self.history.push((hypothesis_id.to_string(), result.clone()));
                    return result;
                }
            }
        }

        // Final confidence check
        if net_confidence < self.config.min_confidence {
            let result = VerificationResult::rejected(
                format!(
                    "Confidence too low: {:.2}, need {:.2}",
                    net_confidence, self.config.min_confidence
                ),
                evidence.to_vec(),
            );
            self.history.push((hypothesis_id.to_string(), result.clone()));
            return result;
        }

        // Passed all checks
        let result = VerificationResult::verified(net_confidence, evidence.to_vec());
        self.history.push((hypothesis_id.to_string(), result.clone()));
        result
    }

    /// Get verification history
    pub fn history(&self) -> &[(String, VerificationResult)] {
        &self.history
    }

    /// Get pass rate
    pub fn pass_rate(&self) -> f64 {
        if self.history.is_empty() {
            return 0.0;
        }
        let passed = self.history.iter().filter(|(_, r)| r.verified).count();
        passed as f64 / self.history.len() as f64
    }
}

// =============================================================================
// 3. PatternHypothesis: Testable Claims
// =============================================================================

/// A testable hypothesis about data→structure relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternHypothesis {
    /// Unique identifier
    pub id: String,
    /// Human-readable description
    pub description: String,
    /// The pattern (training data characteristics)
    pub pattern: PatternSpec,
    /// Expected structure in the model
    pub expected_structure: StructureSpec,
    /// Predictions that can be tested
    pub predictions: Vec<Prediction>,
    /// Current status
    pub status: HypothesisStatus,
    /// When this hypothesis was created
    pub created_at_ms: u64,
}

/// Specification of a training data pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternSpec {
    /// Type of pattern
    pub pattern_type: PatternType,
    /// Key features of this pattern
    pub features: HashMap<String, f64>,
    /// Example data points (as beam tensors)
    pub examples: Vec<BeamTensor>,
}

/// Types of patterns in training data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    /// Factual knowledge (verifiable against ground truth)
    Factual { domain: String },
    /// Reasoning pattern (logical structure)
    Reasoning { logic_type: String },
    /// Linguistic pattern (style, syntax)
    Linguistic { feature: String },
    /// Preference pattern (from RLHF/DPO)
    Preference { dimension: String },
}

/// Expected structure in the learned model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureSpec {
    /// Type of structure
    pub structure_type: StructureType,
    /// Measurable properties
    pub properties: HashMap<String, f64>,
}

/// Types of internal model structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StructureType {
    /// Latent representation cluster
    LatentCluster { dimensions: Vec<usize> },
    /// Attention pattern
    AttentionPattern { heads: Vec<usize> },
    /// Activation pattern
    ActivationPattern { layers: Vec<usize> },
    /// Behavioral capability
    Capability { name: String },
}

/// A testable prediction from a hypothesis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction {
    /// What we predict will happen
    pub statement: String,
    /// How to measure it
    pub metric: String,
    /// Expected value
    pub expected_value: f64,
    /// Acceptable tolerance
    pub tolerance: f64,
    /// Has this been tested?
    pub tested: bool,
    /// Actual observed value (if tested)
    pub observed_value: Option<f64>,
}

/// Status of a hypothesis
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HypothesisStatus {
    /// Newly proposed, not yet tested
    Proposed,
    /// Currently being tested
    Testing,
    /// Verified with evidence
    Verified { confidence: f64 },
    /// Refuted by evidence
    Refuted { reason: String },
    /// Needs more evidence
    Inconclusive,
}

// =============================================================================
// 4. VerifiedPattern: Patterns That Passed Verification
// =============================================================================

/// A pattern that has been verified and can be used for reinforcement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiedPattern {
    /// The original hypothesis
    pub hypothesis: PatternHypothesis,
    /// Verification result
    pub verification: VerificationResult,
    /// How many times this pattern has been reinforced
    pub reinforcement_count: u64,
    /// Cumulative benefit from reinforcement
    pub cumulative_benefit: f64,
    /// Last reinforcement timestamp
    pub last_reinforced_ms: u64,
}

impl VerifiedPattern {
    pub fn new(hypothesis: PatternHypothesis, verification: VerificationResult) -> Self {
        // Initial benefit based on verification confidence
        let initial_benefit = verification.confidence;
        Self {
            hypothesis,
            verification,
            reinforcement_count: 0,
            cumulative_benefit: initial_benefit,
            last_reinforced_ms: 0,
        }
    }

    /// Record a reinforcement event
    pub fn reinforce(&mut self, benefit: f64, timestamp_ms: u64) {
        self.reinforcement_count += 1;
        self.cumulative_benefit += benefit;
        self.last_reinforced_ms = timestamp_ms;
    }
}

// =============================================================================
// 5. ContinuousLearner: Test-Time Learning with Verification
// =============================================================================

/// Configuration for continuous learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContinuousLearningConfig {
    /// How often to check for new patterns (ms)
    pub check_interval_ms: u64,
    /// Maximum patterns to evaluate per cycle
    pub max_patterns_per_cycle: usize,
    /// Minimum improvement to accept a pattern
    pub min_improvement_threshold: f64,
    /// Enable optimistic exploration?
    pub optimistic_exploration: bool,
    /// Decay rate for old patterns
    pub pattern_decay_rate: f64,
}

impl Default for ContinuousLearningConfig {
    fn default() -> Self {
        Self {
            check_interval_ms: 1000,
            max_patterns_per_cycle: 10,
            min_improvement_threshold: 0.01,
            optimistic_exploration: true,
            pattern_decay_rate: 0.99,
        }
    }
}

/// Continuous learner that operates at test time
pub struct ContinuousLearner {
    config: ContinuousLearningConfig,
    /// Verification gate for all patterns
    gate: VerificationGate,
    /// Pending hypotheses awaiting verification
    pending: Vec<PatternHypothesis>,
    /// Verified patterns ready for reinforcement
    verified: Vec<VerifiedPattern>,
    /// Rejected patterns (for learning what doesn't work)
    rejected: Vec<(PatternHypothesis, String)>,
    /// Current learning cycle
    cycle: u64,
    /// Last update time
    last_update: Instant,
}

impl ContinuousLearner {
    pub fn new(config: ContinuousLearningConfig, verification_config: VerificationConfig) -> Self {
        Self {
            config,
            gate: VerificationGate::new(verification_config),
            pending: Vec::new(),
            verified: Vec::new(),
            rejected: Vec::new(),
            cycle: 0,
            last_update: Instant::now(),
        }
    }

    /// Submit a new hypothesis for verification
    pub fn submit_hypothesis(&mut self, hypothesis: PatternHypothesis) {
        self.pending.push(hypothesis);
    }

    /// Run one learning cycle
    pub fn run_cycle(&mut self, evidence_collector: &dyn EvidenceCollector) -> CycleResult {
        self.cycle += 1;
        let start = Instant::now();

        let mut verified_count = 0;
        let mut rejected_count = 0;
        let mut improvements = Vec::new();

        // Process pending hypotheses
        let to_process: Vec<_> = self.pending
            .drain(..)
            .take(self.config.max_patterns_per_cycle)
            .collect();

        for mut hypothesis in to_process {
            hypothesis.status = HypothesisStatus::Testing;

            // Collect evidence for this hypothesis
            let evidence = evidence_collector.collect(&hypothesis);

            // Verify through the gate
            let result = self.gate.verify(&hypothesis.id, &evidence);

            if result.verified {
                hypothesis.status = HypothesisStatus::Verified { 
                    confidence: result.confidence 
                };
                
                let pattern = VerifiedPattern::new(hypothesis, result);
                improvements.push(pattern.verification.confidence);
                self.verified.push(pattern);
                verified_count += 1;
            } else {
                let reason = result.rejection_reason.clone().unwrap_or_default();
                hypothesis.status = HypothesisStatus::Refuted { reason: reason.clone() };
                self.rejected.push((hypothesis, reason));
                rejected_count += 1;
            }
        }

        // Apply decay to old patterns
        for pattern in &mut self.verified {
            pattern.cumulative_benefit *= self.config.pattern_decay_rate;
        }

        // Remove patterns that have decayed below threshold
        self.verified.retain(|p| p.cumulative_benefit > 0.001 || p.reinforcement_count < 10);

        let duration = start.elapsed();
        self.last_update = Instant::now();

        CycleResult {
            cycle: self.cycle,
            duration,
            verified_count,
            rejected_count,
            avg_improvement: if improvements.is_empty() { 
                0.0 
            } else { 
                improvements.iter().sum::<f64>() / improvements.len() as f64 
            },
            total_verified: self.verified.len(),
            total_rejected: self.rejected.len(),
            gate_pass_rate: self.gate.pass_rate(),
        }
    }

    /// Get verified patterns for reinforcement
    pub fn get_verified_patterns(&self) -> &[VerifiedPattern] {
        &self.verified
    }

    /// Get top N patterns by cumulative benefit
    pub fn top_patterns(&self, n: usize) -> Vec<&VerifiedPattern> {
        let mut sorted: Vec<_> = self.verified.iter().collect();
        sorted.sort_by(|a, b| b.cumulative_benefit.partial_cmp(&a.cumulative_benefit).unwrap());
        sorted.into_iter().take(n).collect()
    }
}

/// Result of a learning cycle
#[derive(Debug, Clone)]
pub struct CycleResult {
    pub cycle: u64,
    pub duration: Duration,
    pub verified_count: usize,
    pub rejected_count: usize,
    pub avg_improvement: f64,
    pub total_verified: usize,
    pub total_rejected: usize,
    pub gate_pass_rate: f64,
}

/// Trait for collecting evidence about hypotheses
pub trait EvidenceCollector {
    fn collect(&self, hypothesis: &PatternHypothesis) -> Vec<Evidence>;
}

// =============================================================================
// 6. BenchmarkTracker: SOTA Progress Monitoring
// =============================================================================

/// A benchmark result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Benchmark name
    pub name: String,
    /// Version/split
    pub version: String,
    /// Score achieved
    pub score: f64,
    /// Maximum possible score
    pub max_score: f64,
    /// Current SOTA score (for comparison)
    pub sota_score: f64,
    /// Timestamp
    pub timestamp_ms: u64,
    /// Model configuration used
    pub config_hash: String,
}

impl BenchmarkResult {
    /// How far from SOTA?
    pub fn gap_to_sota(&self) -> f64 {
        self.sota_score - self.score
    }

    /// Percentage of SOTA achieved
    pub fn sota_percentage(&self) -> f64 {
        if self.sota_score > 0.0 {
            (self.score / self.sota_score) * 100.0
        } else {
            0.0
        }
    }

    /// Is this a new SOTA?
    pub fn is_new_sota(&self) -> bool {
        self.score > self.sota_score
    }
}

/// Tracks progress toward SOTA on benchmarks
pub struct BenchmarkTracker {
    /// Results by benchmark name
    results: HashMap<String, Vec<BenchmarkResult>>,
    /// Known SOTA scores
    sota_scores: HashMap<String, f64>,
    /// Target benchmarks to optimize for
    targets: Vec<String>,
}

impl BenchmarkTracker {
    pub fn new() -> Self {
        let mut sota_scores = HashMap::new();
        // Initialize with known SOTA scores (as of training cutoff)
        sota_scores.insert("MMLU".to_string(), 90.0);
        sota_scores.insert("HumanEval".to_string(), 92.0);
        sota_scores.insert("GSM8K".to_string(), 95.0);
        sota_scores.insert("MATH".to_string(), 70.0);
        sota_scores.insert("ARC-Challenge".to_string(), 96.0);
        sota_scores.insert("HellaSwag".to_string(), 95.0);
        sota_scores.insert("TruthfulQA".to_string(), 75.0);

        Self {
            results: HashMap::new(),
            sota_scores,
            targets: vec![
                "MMLU".to_string(),
                "HumanEval".to_string(),
                "GSM8K".to_string(),
            ],
        }
    }

    /// Record a benchmark result
    pub fn record(&mut self, result: BenchmarkResult) {
        self.results
            .entry(result.name.clone())
            .or_insert_with(Vec::new)
            .push(result);
    }

    /// Get progress on a benchmark
    pub fn progress(&self, benchmark: &str) -> Option<BenchmarkProgress> {
        let results = self.results.get(benchmark)?;
        if results.is_empty() {
            return None;
        }

        let latest = results.last()?;
        let best = results.iter().max_by(|a, b| a.score.partial_cmp(&b.score).unwrap())?;
        let sota = self.sota_scores.get(benchmark).copied().unwrap_or(100.0);

        // Calculate improvement trend
        let trend = if results.len() >= 2 {
            let recent: Vec<_> = results.iter().rev().take(5).collect();
            let first = recent.last().map(|r| r.score).unwrap_or(0.0);
            let last = recent.first().map(|r| r.score).unwrap_or(0.0);
            last - first
        } else {
            0.0
        };

        Some(BenchmarkProgress {
            benchmark: benchmark.to_string(),
            latest_score: latest.score,
            best_score: best.score,
            sota_score: sota,
            gap_to_sota: sota - best.score,
            trend,
            num_evaluations: results.len(),
        })
    }

    /// Get overall progress summary
    pub fn summary(&self) -> ProgressSummary {
        let mut benchmarks = Vec::new();
        let mut total_gap = 0.0;
        let mut total_percentage = 0.0;
        let mut count = 0;

        for target in &self.targets {
            if let Some(progress) = self.progress(target) {
                total_gap += progress.gap_to_sota;
                total_percentage += (progress.best_score / progress.sota_score) * 100.0;
                count += 1;
                benchmarks.push(progress);
            }
        }

        ProgressSummary {
            benchmarks,
            avg_gap_to_sota: if count > 0 { total_gap / count as f64 } else { 0.0 },
            avg_sota_percentage: if count > 0 { total_percentage / count as f64 } else { 0.0 },
        }
    }

    /// Update SOTA score (when new results are published)
    pub fn update_sota(&mut self, benchmark: &str, score: f64) {
        self.sota_scores.insert(benchmark.to_string(), score);
    }
}

/// Progress on a single benchmark
#[derive(Debug, Clone)]
pub struct BenchmarkProgress {
    pub benchmark: String,
    pub latest_score: f64,
    pub best_score: f64,
    pub sota_score: f64,
    pub gap_to_sota: f64,
    pub trend: f64,
    pub num_evaluations: usize,
}

/// Overall progress summary
#[derive(Debug, Clone)]
pub struct ProgressSummary {
    pub benchmarks: Vec<BenchmarkProgress>,
    pub avg_gap_to_sota: f64,
    pub avg_sota_percentage: f64,
}

// =============================================================================
// 7. VerifiedPatterningEngine: Main Orchestrator
// =============================================================================

/// Main engine for verified patterning
pub struct VerifiedPatterningEngine {
    /// Continuous learner with verification
    learner: ContinuousLearner,
    /// Benchmark tracker
    benchmarks: BenchmarkTracker,
    /// Pattern→Structure mappings that have been verified
    verified_mappings: HashMap<String, VerifiedPattern>,
    /// Optimistic but unverified hypotheses (for exploration)
    optimistic_queue: Vec<PatternHypothesis>,
}

impl VerifiedPatterningEngine {
    pub fn new(
        learning_config: ContinuousLearningConfig,
        verification_config: VerificationConfig,
    ) -> Self {
        Self {
            learner: ContinuousLearner::new(learning_config, verification_config),
            benchmarks: BenchmarkTracker::new(),
            verified_mappings: HashMap::new(),
            optimistic_queue: Vec::new(),
        }
    }

    /// Submit a hypothesis with optimistic exploration
    pub fn propose(&mut self, hypothesis: PatternHypothesis) {
        // Add to optimistic queue for exploration
        if self.learner.config.optimistic_exploration {
            self.optimistic_queue.push(hypothesis.clone());
        }
        // Submit for verification
        self.learner.submit_hypothesis(hypothesis);
    }

    /// Run verification cycle
    pub fn verify_cycle(&mut self, collector: &dyn EvidenceCollector) -> CycleResult {
        let result = self.learner.run_cycle(collector);

        // Move newly verified patterns to mappings
        for pattern in self.learner.get_verified_patterns() {
            self.verified_mappings.insert(
                pattern.hypothesis.id.clone(),
                pattern.clone(),
            );
        }

        // Remove verified patterns from optimistic queue
        let verified_ids: std::collections::HashSet<_> = self.verified_mappings.keys().collect();
        self.optimistic_queue.retain(|h| !verified_ids.contains(&h.id));

        result
    }

    /// Record benchmark result and generate evidence
    pub fn record_benchmark(&mut self, result: BenchmarkResult) -> Vec<Evidence> {
        let is_improvement = self.benchmarks
            .progress(&result.name)
            .map(|p| result.score > p.best_score)
            .unwrap_or(true);

        self.benchmarks.record(result.clone());

        // Generate evidence from benchmark
        vec![Evidence {
            id: format!("bench_{}_{}", result.name, result.timestamp_ms),
            source: EvidenceSource::Benchmark {
                name: result.name.clone(),
                version: result.version,
            },
            confidence: if is_improvement { 0.9 } else { 0.7 },
            supports: is_improvement,
            measurements: vec![result.score, result.sota_score],
            timestamp_ms: result.timestamp_ms,
            reproducibility: 0.95, // Benchmarks are highly reproducible
        }]
    }

    /// Get patterns ready for reinforcement (verified + high confidence)
    pub fn patterns_for_reinforcement(&self) -> Vec<&VerifiedPattern> {
        self.verified_mappings
            .values()
            .filter(|p| p.verification.confidence >= 0.8)
            .collect()
    }

    /// Get optimistic patterns for exploration (not yet verified)
    pub fn patterns_for_exploration(&self) -> &[PatternHypothesis] {
        &self.optimistic_queue
    }

    /// Get benchmark progress
    pub fn benchmark_progress(&self) -> ProgressSummary {
        self.benchmarks.summary()
    }

    /// Wheat/chaff separation: get only the verified, beneficial patterns
    pub fn wheat(&self) -> Vec<&VerifiedPattern> {
        self.verified_mappings
            .values()
            .filter(|p| p.cumulative_benefit > 0.0 && p.verification.verified)
            .collect()
    }

    /// Get rejected patterns (chaff) for analysis
    pub fn chaff(&self) -> &[(PatternHypothesis, String)] {
        &self.learner.rejected
    }
}

impl Default for BenchmarkTracker {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    struct MockEvidenceCollector;

    impl EvidenceCollector for MockEvidenceCollector {
        fn collect(&self, hypothesis: &PatternHypothesis) -> Vec<Evidence> {
            // Return mock evidence based on hypothesis
            vec![
                Evidence {
                    id: format!("mock_1_{}", hypothesis.id),
                    source: EvidenceSource::Empirical { 
                        p_value: 0.01, 
                        sample_size: 1000 
                    },
                    confidence: 0.9,
                    supports: true,
                    measurements: vec![0.85, 0.87, 0.86],
                    timestamp_ms: 1000,
                    reproducibility: 0.95,
                },
                Evidence {
                    id: format!("mock_2_{}", hypothesis.id),
                    source: EvidenceSource::CrossValidation { k_folds: 5 },
                    confidence: 0.85,
                    supports: true,
                    measurements: vec![0.84, 0.86, 0.85, 0.87, 0.85],
                    timestamp_ms: 1001,
                    reproducibility: 0.9,
                },
            ]
        }
    }

    #[test]
    fn test_verification_gate_passes_valid_evidence() {
        let config = VerificationConfig::default();
        let mut gate = VerificationGate::new(config);

        let evidence = vec![
            Evidence {
                id: "e1".to_string(),
                source: EvidenceSource::Empirical { p_value: 0.01, sample_size: 1000 },
                confidence: 0.9,
                supports: true,
                measurements: vec![0.85],
                timestamp_ms: 1000,
                reproducibility: 0.95,
            },
            Evidence {
                id: "e2".to_string(),
                source: EvidenceSource::CrossValidation { k_folds: 5 },
                confidence: 0.85,
                supports: true,
                measurements: vec![0.84],
                timestamp_ms: 1001,
                reproducibility: 0.9,
            },
        ];

        let result = gate.verify("test_hypothesis", &evidence);
        assert!(result.verified);
        assert!(result.confidence >= 0.8);
    }

    #[test]
    fn test_verification_gate_rejects_insufficient_evidence() {
        let config = VerificationConfig {
            min_evidence_sources: 3,
            ..Default::default()
        };
        let mut gate = VerificationGate::new(config);

        let evidence = vec![
            Evidence {
                id: "e1".to_string(),
                source: EvidenceSource::Empirical { p_value: 0.01, sample_size: 1000 },
                confidence: 0.9,
                supports: true,
                measurements: vec![0.85],
                timestamp_ms: 1000,
                reproducibility: 0.95,
            },
        ];

        let result = gate.verify("test_hypothesis", &evidence);
        assert!(!result.verified);
        assert!(result.rejection_reason.unwrap().contains("Insufficient"));
    }

    #[test]
    fn test_verification_gate_rejects_low_p_value() {
        let config = VerificationConfig::default();
        let mut gate = VerificationGate::new(config);

        let evidence = vec![
            Evidence {
                id: "e1".to_string(),
                source: EvidenceSource::Empirical { p_value: 0.1, sample_size: 100 }, // p > 0.05
                confidence: 0.9,
                supports: true,
                measurements: vec![0.85],
                timestamp_ms: 1000,
                reproducibility: 0.95,
            },
            Evidence {
                id: "e2".to_string(),
                source: EvidenceSource::CrossValidation { k_folds: 5 },
                confidence: 0.85,
                supports: true,
                measurements: vec![0.84],
                timestamp_ms: 1001,
                reproducibility: 0.9,
            },
        ];

        let result = gate.verify("test_hypothesis", &evidence);
        assert!(!result.verified);
        assert!(result.rejection_reason.unwrap().contains("significance"));
    }

    #[test]
    fn test_continuous_learner_cycle() {
        let learning_config = ContinuousLearningConfig::default();
        let verification_config = VerificationConfig::default();
        let mut learner = ContinuousLearner::new(learning_config, verification_config);

        let hypothesis = PatternHypothesis {
            id: "h1".to_string(),
            description: "Test hypothesis".to_string(),
            pattern: PatternSpec {
                pattern_type: PatternType::Factual { domain: "math".to_string() },
                features: HashMap::new(),
                examples: vec![],
            },
            expected_structure: StructureSpec {
                structure_type: StructureType::Capability { name: "arithmetic".to_string() },
                properties: HashMap::new(),
            },
            predictions: vec![],
            status: HypothesisStatus::Proposed,
            created_at_ms: 1000,
        };

        learner.submit_hypothesis(hypothesis);

        let collector = MockEvidenceCollector;
        let result = learner.run_cycle(&collector);

        assert_eq!(result.cycle, 1);
        assert_eq!(result.verified_count, 1);
        assert_eq!(result.rejected_count, 0);
    }

    #[test]
    fn test_benchmark_tracker() {
        let mut tracker = BenchmarkTracker::new();

        tracker.record(BenchmarkResult {
            name: "MMLU".to_string(),
            version: "v1".to_string(),
            score: 85.0,
            max_score: 100.0,
            sota_score: 90.0,
            timestamp_ms: 1000,
            config_hash: "abc".to_string(),
        });

        let progress = tracker.progress("MMLU").unwrap();
        assert_eq!(progress.latest_score, 85.0);
        assert_eq!(progress.gap_to_sota, 5.0);
    }

    #[test]
    fn test_wheat_chaff_separation() {
        let learning_config = ContinuousLearningConfig::default();
        let verification_config = VerificationConfig::default();
        let mut engine = VerifiedPatterningEngine::new(learning_config, verification_config);

        let hypothesis = PatternHypothesis {
            id: "h1".to_string(),
            description: "Good hypothesis".to_string(),
            pattern: PatternSpec {
                pattern_type: PatternType::Factual { domain: "science".to_string() },
                features: HashMap::new(),
                examples: vec![],
            },
            expected_structure: StructureSpec {
                structure_type: StructureType::Capability { name: "reasoning".to_string() },
                properties: HashMap::new(),
            },
            predictions: vec![],
            status: HypothesisStatus::Proposed,
            created_at_ms: 1000,
        };

        engine.propose(hypothesis);

        let collector = MockEvidenceCollector;
        engine.verify_cycle(&collector);

        // Should have wheat (verified patterns)
        let wheat = engine.wheat();
        assert!(!wheat.is_empty());

        // Chaff should be empty (no rejections with mock collector)
        let chaff = engine.chaff();
        assert!(chaff.is_empty());
    }
}
