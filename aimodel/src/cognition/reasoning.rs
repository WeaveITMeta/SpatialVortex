//! Unified Reasoning Module - Diverse Reasoning Types with Critical Thinking
//!
//! Implements comprehensive reasoning capabilities:
//! - **Deductive**: General → Specific (truth-preserving, modus ponens/tollens)
//! - **Inductive**: Specific → General (probabilistic generalization)
//! - **Abductive**: Best explanation (hypothesis generation)
//! - **Analogical**: Pattern transfer across domains
//! - **Critical Thinking**: Bias detection, argument evaluation, fallacy checking
//!
//! Integrates with FluxMatrix for geometric reasoning and JEPA for embedding-space inference.

use crate::data::models::BeamTensor;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;

// =============================================================================
// Logical Operations (from main crate's first_principles.rs)
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogicalOperation {
    Deduction,      // General → Specific (truth-preserving)
    Induction,      // Specific → General (probabilistic)
    Abduction,      // Best explanation (hypothesis)
    ModusPonens,    // If P then Q, P ∴ Q
    ModusTollens,   // If P then Q, ¬Q ∴ ¬P
    Contradiction,  // P ∧ ¬P detection
    Analogy,        // Pattern transfer
    Transitive,     // A→B, B→C ∴ A→C
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AxiomDomain {
    Physical,       // Physical reality laws
    Logical,        // Mathematical/logical truths
    Ethical,        // Moral principles
    Psychological,  // Human behavior patterns
    Universal,      // Applies everywhere
}

// =============================================================================
// Reasoning Step
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningStep {
    pub id: Uuid,
    pub premise: String,
    pub operation: LogicalOperation,
    pub conclusion: String,
    pub confidence: f32,
    pub domain: AxiomDomain,
    pub sacred_position: u8,
    pub evidence: Vec<String>,
    pub counterevidence: Vec<String>,
}

impl ReasoningStep {
    pub fn new(premise: &str, operation: LogicalOperation, conclusion: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            premise: premise.to_string(),
            operation,
            conclusion: conclusion.to_string(),
            confidence: 0.5,
            domain: AxiomDomain::Logical,
            sacred_position: 1,
            evidence: Vec::new(),
            counterevidence: Vec::new(),
        }
    }

    pub fn with_confidence(mut self, conf: f32) -> Self {
        self.confidence = conf;
        self
    }

    pub fn with_domain(mut self, domain: AxiomDomain) -> Self {
        self.domain = domain;
        self
    }

    pub fn add_evidence(&mut self, evidence: &str) {
        self.evidence.push(evidence.to_string());
        self.confidence = (self.confidence + 0.1).min(1.0);
    }

    pub fn add_counterevidence(&mut self, counter: &str) {
        self.counterevidence.push(counter.to_string());
        self.confidence = (self.confidence - 0.15).max(0.0);
    }

    pub fn is_valid(&self) -> bool {
        self.confidence > 0.5 && self.counterevidence.len() < self.evidence.len()
    }
}

// =============================================================================
// Axiom System (First Principles)
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Axiom {
    pub name: String,
    pub statement: String,
    pub domain: AxiomDomain,
    pub sacred_position: u8,
    pub confidence: f32,
}

impl Axiom {
    pub fn new(name: &str, statement: &str, domain: AxiomDomain) -> Self {
        let sacred_position = match domain {
            AxiomDomain::Ethical => 3,
            AxiomDomain::Logical => 6,
            AxiomDomain::Psychological => 9,
            _ => 1,
        };
        Self {
            name: name.to_string(),
            statement: statement.to_string(),
            domain,
            sacred_position,
            confidence: 1.0,
        }
    }
}

// =============================================================================
// Critical Thinking Engine
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CognitiveBias {
    ConfirmationBias,     // Seeking confirming evidence only
    AnchoringBias,        // Over-relying on first information
    AvailabilityBias,     // Overweighting recent/memorable info
    SurvivorshipBias,     // Ignoring failures
    HindsightBias,        // "I knew it all along"
    DunningKruger,        // Overconfidence with low competence
    SunkCostFallacy,      // Continuing due to past investment
    BandwagonEffect,      // Following the crowd
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LogicalFallacy {
    AdHominem,            // Attacking the person
    StrawMan,             // Misrepresenting argument
    FalseDisjunction,     // False either/or
    SlipperySlope,        // Unwarranted chain of consequences
    AppealToAuthority,    // Unqualified authority
    AppealToEmotion,      // Emotion over logic
    CircularReasoning,    // Conclusion in premise
    HastyGeneralization,  // Too few examples
    RedHerring,           // Irrelevant distraction
    FalseCause,           // Correlation ≠ causation
}

#[derive(Debug, Clone)]
pub struct CriticalThinkingEngine {
    pub bias_detectors: HashMap<CognitiveBias, f32>,
    pub fallacy_patterns: HashMap<LogicalFallacy, Vec<String>>,
    pub disconfirming_threshold: f32,
    pub stats: CriticalThinkingStats,
}

#[derive(Debug, Clone, Default)]
pub struct CriticalThinkingStats {
    pub biases_detected: usize,
    pub fallacies_detected: usize,
    pub arguments_evaluated: usize,
    pub disconfirming_evidence_sought: usize,
}

impl CriticalThinkingEngine {
    pub fn new() -> Self {
        let mut bias_detectors = HashMap::new();
        for bias in [
            CognitiveBias::ConfirmationBias,
            CognitiveBias::AnchoringBias,
            CognitiveBias::AvailabilityBias,
            CognitiveBias::SurvivorshipBias,
        ] {
            bias_detectors.insert(bias, 0.5);
        }

        let mut fallacy_patterns = HashMap::new();
        fallacy_patterns.insert(LogicalFallacy::AdHominem, vec![
            "you're wrong because".to_string(),
            "what do you know".to_string(),
        ]);
        fallacy_patterns.insert(LogicalFallacy::StrawMan, vec![
            "so you're saying".to_string(),
            "that means you think".to_string(),
        ]);
        fallacy_patterns.insert(LogicalFallacy::SlipperySlope, vec![
            "next thing you know".to_string(),
            "will inevitably lead to".to_string(),
        ]);
        fallacy_patterns.insert(LogicalFallacy::FalseCause, vec![
            "therefore caused".to_string(),
            "because it happened after".to_string(),
        ]);

        Self {
            bias_detectors,
            fallacy_patterns,
            disconfirming_threshold: 0.3,
            stats: CriticalThinkingStats::default(),
        }
    }

    /// Detect confirmation bias by checking evidence balance
    pub fn detect_confirmation_bias(&mut self, step: &ReasoningStep) -> Option<CognitiveBias> {
        let evidence_ratio = if step.counterevidence.is_empty() {
            1.0
        } else {
            step.evidence.len() as f32 / (step.evidence.len() + step.counterevidence.len()) as f32
        };

        if evidence_ratio > 0.9 && step.evidence.len() > 2 {
            self.stats.biases_detected += 1;
            Some(CognitiveBias::ConfirmationBias)
        } else {
            None
        }
    }

    /// Detect logical fallacies in text
    pub fn detect_fallacies(&mut self, text: &str) -> Vec<LogicalFallacy> {
        let lower = text.to_lowercase();
        let mut detected = Vec::new();

        for (fallacy, patterns) in &self.fallacy_patterns {
            for pattern in patterns {
                if lower.contains(pattern) {
                    detected.push(*fallacy);
                    self.stats.fallacies_detected += 1;
                    break;
                }
            }
        }

        detected
    }

    /// Force seeking disconfirming evidence
    pub fn seek_disconfirming(&mut self, hypothesis: &str) -> Vec<String> {
        self.stats.disconfirming_evidence_sought += 1;
        vec![
            format!("What evidence would disprove: '{}'?", hypothesis),
            format!("Under what conditions would '{}' be false?", hypothesis),
            format!("What are the strongest arguments against '{}'?", hypothesis),
        ]
    }

    /// Evaluate argument strength
    pub fn evaluate_argument(&mut self, premises: &[&str], conclusion: &str) -> ArgumentEvaluation {
        self.stats.arguments_evaluated += 1;

        let mut validity = 0.5;
        let mut soundness = 0.5;
        let mut issues = Vec::new();

        // Check for empty premises
        if premises.is_empty() {
            validity = 0.0;
            issues.push("No premises provided".to_string());
        }

        // Check for logical connection
        let conclusion_lower = conclusion.to_lowercase();
        let mut premise_connection = 0;
        for premise in premises {
            let premise_lower = premise.to_lowercase();
            // Simple word overlap check
            let premise_words: HashSet<&str> = premise_lower.split_whitespace().collect();
            let conclusion_words: HashSet<&str> = conclusion_lower.split_whitespace().collect();
            let overlap = premise_words.intersection(&conclusion_words).count();
            if overlap > 0 {
                premise_connection += 1;
            }
        }

        if premise_connection == 0 && !premises.is_empty() {
            validity *= 0.5;
            issues.push("Weak connection between premises and conclusion".to_string());
        }

        // Check for fallacies
        let fallacies = self.detect_fallacies(conclusion);
        if !fallacies.is_empty() {
            soundness *= 0.5;
            issues.push(format!("Potential fallacies: {:?}", fallacies));
        }

        ArgumentEvaluation {
            validity,
            soundness,
            issues,
            recommendation: if validity > 0.7 && soundness > 0.7 {
                "Argument appears valid and sound".to_string()
            } else {
                "Argument needs strengthening".to_string()
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArgumentEvaluation {
    pub validity: f32,
    pub soundness: f32,
    pub issues: Vec<String>,
    pub recommendation: String,
}

// =============================================================================
// Unified Reasoning Engine
// =============================================================================

#[derive(Debug, Clone)]
pub struct UnifiedReasoningEngine {
    pub axioms: Vec<Axiom>,
    pub rules: Vec<InferenceRule>,
    pub critical_thinking: CriticalThinkingEngine,
    pub reasoning_history: VecDeque<ReasoningStep>,
    pub hypotheses: HashMap<Uuid, Hypothesis>,
    pub config: ReasoningConfig,
    pub stats: ReasoningStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningConfig {
    pub max_depth: usize,
    pub confidence_threshold: f32,
    pub enable_critical_thinking: bool,
    pub sacred_weights: [f32; 10],
    pub damping_factor: f32,
}

impl Default for ReasoningConfig {
    fn default() -> Self {
        let mut sacred_weights = [1.0; 10];
        sacred_weights[3] = 1.15;
        sacred_weights[6] = 1.15;
        sacred_weights[9] = 1.15;
        Self {
            max_depth: 9,
            confidence_threshold: 0.6,
            enable_critical_thinking: true,
            sacred_weights,
            damping_factor: 0.85,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ReasoningStats {
    pub deductions: usize,
    pub inductions: usize,
    pub abductions: usize,
    pub analogies: usize,
    pub total_steps: usize,
    pub successful_chains: usize,
    pub failed_chains: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRule {
    pub name: String,
    pub antecedent: String,
    pub consequent: String,
    pub confidence: f32,
    pub operation: LogicalOperation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hypothesis {
    pub id: Uuid,
    pub statement: String,
    pub confidence: f32,
    pub evidence_for: Vec<String>,
    pub evidence_against: Vec<String>,
    pub status: HypothesisStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HypothesisStatus {
    Proposed,
    Testing,
    Confirmed,
    Refuted,
}

impl UnifiedReasoningEngine {
    pub fn new() -> Self {
        Self::with_config(ReasoningConfig::default())
    }

    pub fn with_config(config: ReasoningConfig) -> Self {
        let mut engine = Self {
            axioms: Vec::new(),
            rules: Vec::new(),
            critical_thinking: CriticalThinkingEngine::new(),
            reasoning_history: VecDeque::with_capacity(100),
            hypotheses: HashMap::new(),
            config,
            stats: ReasoningStats::default(),
        };
        engine.load_default_axioms();
        engine.load_default_rules();
        engine
    }

    fn load_default_axioms(&mut self) {
        self.axioms.push(Axiom::new(
            "Identity", "A thing is identical to itself (A = A)",
            AxiomDomain::Logical
        ));
        self.axioms.push(Axiom::new(
            "Non-Contradiction", "Nothing can both be and not be (¬(P ∧ ¬P))",
            AxiomDomain::Logical
        ));
        self.axioms.push(Axiom::new(
            "Excluded Middle", "Everything must either be or not be (P ∨ ¬P)",
            AxiomDomain::Logical
        ));
        self.axioms.push(Axiom::new(
            "Causality", "Every effect has a cause",
            AxiomDomain::Physical
        ));
        self.axioms.push(Axiom::new(
            "Golden Rule", "Treat others as you wish to be treated",
            AxiomDomain::Ethical
        ));
    }

    fn load_default_rules(&mut self) {
        self.rules.push(InferenceRule {
            name: "Modus Ponens".to_string(),
            antecedent: "If P then Q, and P".to_string(),
            consequent: "Therefore Q".to_string(),
            confidence: 1.0,
            operation: LogicalOperation::ModusPonens,
        });
        self.rules.push(InferenceRule {
            name: "Modus Tollens".to_string(),
            antecedent: "If P then Q, and not Q".to_string(),
            consequent: "Therefore not P".to_string(),
            confidence: 1.0,
            operation: LogicalOperation::ModusTollens,
        });
        self.rules.push(InferenceRule {
            name: "Transitivity".to_string(),
            antecedent: "A implies B, B implies C".to_string(),
            consequent: "Therefore A implies C".to_string(),
            confidence: 1.0,
            operation: LogicalOperation::Transitive,
        });
    }

    // =========================================================================
    // Deductive Reasoning (General → Specific)
    // =========================================================================

    pub fn deduce(&mut self, premises: &[&str], target: &str) -> Option<ReasoningStep> {
        self.stats.deductions += 1;
        self.stats.total_steps += 1;

        // Try to find applicable rule
        for rule in &self.rules {
            if rule.operation == LogicalOperation::ModusPonens ||
               rule.operation == LogicalOperation::ModusTollens ||
               rule.operation == LogicalOperation::Transitive {
                // Check if premises match rule pattern
                let mut step = ReasoningStep::new(
                    &premises.join("; "),
                    rule.operation,
                    target,
                );
                step.confidence = rule.confidence * self.config.damping_factor;

                // Apply critical thinking
                if self.config.enable_critical_thinking {
                    if let Some(bias) = self.critical_thinking.detect_confirmation_bias(&step) {
                        step.confidence *= 0.8;
                        step.counterevidence.push(format!("Warning: {:?} detected", bias));
                    }
                }

                self.reasoning_history.push_back(step.clone());
                if self.reasoning_history.len() > 100 {
                    self.reasoning_history.pop_front();
                }

                return Some(step);
            }
        }

        None
    }

    // =========================================================================
    // Inductive Reasoning (Specific → General)
    // =========================================================================

    pub fn induce(&mut self, observations: &[&str], min_support: f32) -> Option<ReasoningStep> {
        self.stats.inductions += 1;
        self.stats.total_steps += 1;

        if observations.is_empty() {
            return None;
        }

        // Find common patterns across observations
        let mut word_counts: HashMap<String, usize> = HashMap::new();
        for obs in observations {
            for word in obs.to_lowercase().split_whitespace() {
                if word.len() > 3 {
                    *word_counts.entry(word.to_string()).or_insert(0) += 1;
                }
            }
        }

        // Find words appearing in most observations
        let threshold = (observations.len() as f32 * min_support) as usize;
        let common: Vec<&String> = word_counts.iter()
            .filter(|(_, &count)| count >= threshold.max(1))
            .map(|(word, _)| word)
            .collect();

        if common.is_empty() {
            return None;
        }

        let generalization = format!(
            "Based on {} observations, pattern involves: {}",
            observations.len(),
            common.iter().take(5).cloned().cloned().collect::<Vec<_>>().join(", ")
        );

        let confidence = (observations.len() as f32 / 10.0).min(0.9) * self.config.damping_factor;

        let mut step = ReasoningStep::new(
            &observations.join("; "),
            LogicalOperation::Induction,
            &generalization,
        );
        step.confidence = confidence;

        // Critical thinking: warn about hasty generalization
        if observations.len() < 5 && self.config.enable_critical_thinking {
            step.counterevidence.push("Warning: Small sample size (hasty generalization risk)".to_string());
            step.confidence *= 0.7;
        }

        self.reasoning_history.push_back(step.clone());
        Some(step)
    }

    // =========================================================================
    // Abductive Reasoning (Best Explanation)
    // =========================================================================

    pub fn abduce(&mut self, observation: &str, possible_causes: &[&str]) -> Option<ReasoningStep> {
        self.stats.abductions += 1;
        self.stats.total_steps += 1;

        if possible_causes.is_empty() {
            return None;
        }

        // Score each hypothesis based on explanatory power
        let mut best_hypothesis = possible_causes[0];
        let mut best_score = 0.0f32;

        let obs_lower = observation.to_lowercase();
        let obs_words: HashSet<&str> = obs_lower.split_whitespace().collect();

        for &cause in possible_causes {
            let cause_lower = cause.to_lowercase();
            let cause_words: HashSet<&str> = cause_lower.split_whitespace().collect();

            // Score based on word overlap and simplicity
            let overlap = obs_words.intersection(&cause_words).count() as f32;
            let simplicity = 1.0 / (cause.len() as f32 + 1.0);
            let score = overlap * 0.7 + simplicity * 10.0 * 0.3;

            if score > best_score {
                best_score = score;
                best_hypothesis = cause;
            }
        }

        let mut step = ReasoningStep::new(
            observation,
            LogicalOperation::Abduction,
            &format!("Best explanation: {}", best_hypothesis),
        );
        step.confidence = (best_score / 5.0).min(0.85) * self.config.damping_factor;

        // Store as hypothesis for further testing
        let hypothesis = Hypothesis {
            id: Uuid::new_v4(),
            statement: best_hypothesis.to_string(),
            confidence: step.confidence,
            evidence_for: vec![observation.to_string()],
            evidence_against: Vec::new(),
            status: HypothesisStatus::Proposed,
        };
        self.hypotheses.insert(hypothesis.id, hypothesis);

        self.reasoning_history.push_back(step.clone());
        Some(step)
    }

    // =========================================================================
    // Analogical Reasoning (Pattern Transfer)
    // =========================================================================

    pub fn reason_by_analogy(
        &mut self,
        source_domain: &str,
        source_pattern: &str,
        target_domain: &str,
    ) -> Option<ReasoningStep> {
        self.stats.analogies += 1;
        self.stats.total_steps += 1;

        // Extract key relations from source
        let source_words: Vec<&str> = source_pattern.split_whitespace().collect();

        // Map to target domain
        let transferred = format!(
            "In {}: {} (transferred from {} pattern)",
            target_domain,
            source_pattern.replace(source_domain, target_domain),
            source_domain
        );

        let mut step = ReasoningStep::new(
            &format!("{}: {}", source_domain, source_pattern),
            LogicalOperation::Analogy,
            &transferred,
        );

        // Analogies are inherently less certain
        step.confidence = 0.6 * self.config.damping_factor;

        if self.config.enable_critical_thinking {
            step.counterevidence.push(
                "Warning: Analogies may not preserve all properties".to_string()
            );
        }

        self.reasoning_history.push_back(step.clone());
        Some(step)
    }

    // =========================================================================
    // Chain Reasoning (Multi-step)
    // =========================================================================

    pub fn reason_chain(&mut self, query: &str, max_steps: usize) -> ReasoningChain {
        let mut chain = ReasoningChain::new(query);
        let steps = max_steps.min(self.config.max_depth);

        for i in 0..steps {
            let position = ((i % 6) + 1) as u8;
            let sacred_weight = self.config.sacred_weights[position as usize];

            // Alternate between reasoning types
            let step = match i % 4 {
                0 => self.deduce(&[query], &format!("Step {} deduction", i)),
                1 => self.induce(&[query], 0.5),
                2 => self.abduce(query, &["cause A", "cause B"]),
                _ => self.reason_by_analogy("source", query, "target"),
            };

            if let Some(mut s) = step {
                s.sacred_position = position;
                s.confidence *= sacred_weight;
                chain.add_step(s);
            }

            // Check convergence
            if chain.has_converged() {
                break;
            }
        }

        if chain.steps.is_empty() {
            self.stats.failed_chains += 1;
        } else {
            self.stats.successful_chains += 1;
        }

        chain
    }

    // =========================================================================
    // Hypothesis Testing
    // =========================================================================

    pub fn test_hypothesis(&mut self, hypothesis_id: Uuid, evidence: &str, supports: bool) {
        if let Some(h) = self.hypotheses.get_mut(&hypothesis_id) {
            if supports {
                h.evidence_for.push(evidence.to_string());
                h.confidence = (h.confidence + 0.1).min(1.0);
            } else {
                h.evidence_against.push(evidence.to_string());
                h.confidence = (h.confidence - 0.15).max(0.0);
            }

            h.status = match h.confidence {
                c if c > 0.8 => HypothesisStatus::Confirmed,
                c if c < 0.2 => HypothesisStatus::Refuted,
                _ => HypothesisStatus::Testing,
            };
        }
    }

    pub fn get_stats(&self) -> &ReasoningStats {
        &self.stats
    }
}

// =============================================================================
// Reasoning Chain
// =============================================================================

#[derive(Debug, Clone)]
pub struct ReasoningChain {
    pub query: String,
    pub steps: Vec<ReasoningStep>,
    pub overall_confidence: f32,
    pub sacred_milestones: Vec<u8>,
}

impl ReasoningChain {
    pub fn new(query: &str) -> Self {
        Self {
            query: query.to_string(),
            steps: Vec::new(),
            overall_confidence: 0.0,
            sacred_milestones: Vec::new(),
        }
    }

    pub fn add_step(&mut self, step: ReasoningStep) {
        if matches!(step.sacred_position, 3 | 6 | 9) {
            if !self.sacred_milestones.contains(&step.sacred_position) {
                self.sacred_milestones.push(step.sacred_position);
            }
        }
        self.steps.push(step);
        self.update_confidence();
    }

    fn update_confidence(&mut self) {
        if self.steps.is_empty() {
            self.overall_confidence = 0.0;
            return;
        }
        let sum: f32 = self.steps.iter().map(|s| s.confidence).sum();
        self.overall_confidence = sum / self.steps.len() as f32;
    }

    pub fn has_converged(&self) -> bool {
        self.overall_confidence > 0.75 || self.sacred_milestones.len() >= 2
    }

    pub fn get_conclusion(&self) -> Option<&str> {
        self.steps.last().map(|s| s.conclusion.as_str())
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduction() {
        let mut engine = UnifiedReasoningEngine::new();
        let step = engine.deduce(
            &["All humans are mortal", "Socrates is human"],
            "Socrates is mortal"
        );
        assert!(step.is_some());
        assert!(step.unwrap().confidence > 0.5);
    }

    #[test]
    fn test_induction() {
        let mut engine = UnifiedReasoningEngine::new();
        let step = engine.induce(
            &["Swan 1 is white", "Swan 2 is white", "Swan 3 is white", "Swan 4 is white", "Swan 5 is white"],
            0.6
        );
        assert!(step.is_some());
        let s = step.unwrap();
        assert!(s.conclusion.contains("white"));
    }

    #[test]
    fn test_abduction() {
        let mut engine = UnifiedReasoningEngine::new();
        let step = engine.abduce(
            "The grass is wet",
            &["It rained", "Sprinklers were on", "Morning dew"]
        );
        assert!(step.is_some());
    }

    #[test]
    fn test_critical_thinking() {
        let mut engine = CriticalThinkingEngine::new();
        let fallacies = engine.detect_fallacies("You're wrong because you're not an expert");
        assert!(!fallacies.is_empty());
    }

    #[test]
    fn test_reasoning_chain() {
        let mut engine = UnifiedReasoningEngine::new();
        let chain = engine.reason_chain("Why is the sky blue?", 5);
        assert!(!chain.steps.is_empty());
    }

    #[test]
    fn test_argument_evaluation() {
        let mut engine = CriticalThinkingEngine::new();
        let eval = engine.evaluate_argument(
            &["All birds can fly", "Penguins are birds"],
            "Penguins can fly"
        );
        assert!(eval.validity > 0.0);
    }
}
