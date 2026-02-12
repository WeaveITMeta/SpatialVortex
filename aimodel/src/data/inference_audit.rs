//! Inference Audit System
//!
//! Table of Contents:
//! 1. InferenceTrace — Complete trace of one question's inference journey
//! 2. ExpertContribution — Per-expert score contribution with attribution
//! 3. DecisionPoint — Each branch/gate in the inference flow
//! 4. AuditCollector — Accumulates traces across questions for ablation analysis
//! 5. AblationReport — Per-expert impact analysis (accuracy with/without each expert)
//!
//! Purpose: Instrument every decision in generative_inference so we can:
//! - See exactly why the model chose each answer
//! - Measure each expert's marginal contribution to accuracy
//! - Identify dead/harmful experts for removal
//! - Debug the thought process from start to finish

use std::collections::HashMap;
use std::fmt;

// =============================================================================
// 1. InferenceTrace — One question's complete inference journey
// =============================================================================

/// Complete trace of how the model answered one question
#[derive(Debug, Clone)]
pub struct InferenceTrace {
    /// Question ID
    pub question_id: String,
    /// Source benchmark (mmlu, arc, truthfulqa, etc.)
    pub source: String,
    /// The question text (truncated for display)
    pub question_short: String,
    /// Number of choices
    pub num_choices: usize,
    /// Correct answer index
    pub correct_idx: usize,
    /// Predicted answer index
    pub predicted_idx: usize,
    /// Final confidence
    pub confidence: f32,
    /// Whether the prediction was correct
    pub is_correct: bool,
    /// Which decision path was taken (pipeline, unified, expert-high, quantum, etc.)
    pub decision_path: String,
    /// All decision points encountered
    pub decisions: Vec<DecisionPoint>,
    /// Per-expert contributions for each choice
    pub expert_contributions: Vec<Vec<ExpertContribution>>,
    /// Raw logits per choice (before softmax)
    pub raw_logits: Vec<f32>,
    /// Final probabilities per choice (after softmax)
    pub final_probs: Vec<f32>,
    /// Wall-clock time for this question (ms)
    pub elapsed_ms: u64,
}

impl InferenceTrace {
    /// Create a new empty trace
    pub fn new(question_id: &str, source: &str, question: &str, num_choices: usize, correct_idx: usize) -> Self {
        Self {
            question_id: question_id.to_string(),
            source: source.to_string(),
            question_short: question.chars().take(80).collect(),
            num_choices,
            correct_idx,
            predicted_idx: 0,
            confidence: 0.0,
            is_correct: false,
            decision_path: String::new(),
            decisions: Vec::new(),
            expert_contributions: vec![Vec::new(); num_choices],
            raw_logits: vec![0.0; num_choices],
            final_probs: vec![0.0; num_choices],
            elapsed_ms: 0,
        }
    }

    /// Record a decision point
    pub fn record_decision(&mut self, name: &str, outcome: &str, confidence: f32) {
        self.decisions.push(DecisionPoint {
            name: name.to_string(),
            outcome: outcome.to_string(),
            confidence,
        });
    }

    /// Record an expert's contribution to a specific choice
    pub fn record_expert(&mut self, choice_idx: usize, expert_name: &str, score: f32) {
        if choice_idx < self.expert_contributions.len() {
            self.expert_contributions[choice_idx].push(ExpertContribution {
                expert_name: expert_name.to_string(),
                score,
            });
        }
    }

    /// Finalize the trace with the prediction result
    pub fn finalize(&mut self, predicted_idx: usize, confidence: f32, decision_path: &str, logits: &[f32], probs: &[f32]) {
        self.predicted_idx = predicted_idx;
        self.confidence = confidence;
        self.is_correct = predicted_idx == self.correct_idx;
        self.decision_path = decision_path.to_string();
        self.raw_logits = logits.to_vec();
        self.final_probs = probs.to_vec();
    }

    /// Get the expert that contributed most to the winning choice
    pub fn top_expert_for_winner(&self) -> Option<(&str, f32)> {
        self.expert_contributions.get(self.predicted_idx)
            .and_then(|contribs| {
                contribs.iter()
                    .max_by(|a, b| a.score.abs().partial_cmp(&b.score.abs()).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|c| (c.expert_name.as_str(), c.score))
            })
    }

    /// Get the expert that contributed most to the correct choice
    pub fn top_expert_for_correct(&self) -> Option<(&str, f32)> {
        self.expert_contributions.get(self.correct_idx)
            .and_then(|contribs| {
                contribs.iter()
                    .max_by(|a, b| a.score.abs().partial_cmp(&b.score.abs()).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|c| (c.expert_name.as_str(), c.score))
            })
    }

    /// Compute what the answer would be if we removed a specific expert
    pub fn answer_without_expert(&self, expert_name: &str) -> usize {
        let mut adjusted_logits = self.raw_logits.clone();
        for (ci, contribs) in self.expert_contributions.iter().enumerate() {
            for c in contribs {
                if c.expert_name == expert_name {
                    if ci < adjusted_logits.len() {
                        adjusted_logits[ci] -= c.score;
                    }
                }
            }
        }
        adjusted_logits.iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(idx, _)| idx)
            .unwrap_or(0)
    }
}

impl fmt::Display for InferenceTrace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.is_correct { "OK" } else { "WRONG" };
        writeln!(f, "=== [{}] {} ({})", status, self.question_id, self.source)?;
        writeln!(f, "    Q: {}", self.question_short)?;
        writeln!(f, "    Path: {} | Conf: {:.2} | Time: {}ms", self.decision_path, self.confidence, self.elapsed_ms)?;
        
        // Show decision points
        for d in &self.decisions {
            writeln!(f, "    GATE: {} → {} (conf={:.2})", d.name, d.outcome, d.confidence)?;
        }
        
        // Show per-choice breakdown
        for ci in 0..self.num_choices {
            let marker = if ci == self.correct_idx && ci == self.predicted_idx { " [CORRECT+PREDICTED]" }
                        else if ci == self.correct_idx { " [CORRECT]" }
                        else if ci == self.predicted_idx { " [PREDICTED]" }
                        else { "" };
            writeln!(f, "    Choice[{}]{}: logit={:.1} prob={:.3}", ci, marker, 
                self.raw_logits.get(ci).unwrap_or(&0.0),
                self.final_probs.get(ci).unwrap_or(&0.0))?;
            
            if let Some(contribs) = self.expert_contributions.get(ci) {
                let mut sorted: Vec<_> = contribs.iter().filter(|c| c.score.abs() > 0.1).collect();
                sorted.sort_by(|a, b| b.score.abs().partial_cmp(&a.score.abs()).unwrap_or(std::cmp::Ordering::Equal));
                for c in sorted.iter().take(5) {
                    let sign = if c.score >= 0.0 { "+" } else { "" };
                    writeln!(f, "      {:15} {}{:.1}", c.expert_name, sign, c.score)?;
                }
            }
        }
        Ok(())
    }
}

// =============================================================================
// 2. ExpertContribution — One expert's score for one choice
// =============================================================================

#[derive(Debug, Clone)]
pub struct ExpertContribution {
    pub expert_name: String,
    pub score: f32,
}

// =============================================================================
// 3. DecisionPoint — A gate/branch in the inference flow
// =============================================================================

#[derive(Debug, Clone)]
pub struct DecisionPoint {
    /// Name of the gate (e.g., "pipeline", "unified", "quantum_vs_expert")
    pub name: String,
    /// Outcome (e.g., "committed", "fell_through", "expert-high")
    pub outcome: String,
    /// Confidence at this decision point
    pub confidence: f32,
}

// =============================================================================
// 4. AuditCollector — Accumulates traces for ablation analysis
// =============================================================================

/// Collects inference traces and computes ablation statistics
pub struct AuditCollector {
    /// All traces collected
    pub traces: Vec<InferenceTrace>,
    /// Per-source accuracy tracking
    pub source_accuracy: HashMap<String, (usize, usize)>, // (correct, total)
    /// Per-decision-path tracking
    pub path_accuracy: HashMap<String, (usize, usize)>,
}

impl AuditCollector {
    pub fn new() -> Self {
        Self {
            traces: Vec::new(),
            source_accuracy: HashMap::new(),
            path_accuracy: HashMap::new(),
        }
    }

    /// Record a completed trace
    pub fn record(&mut self, trace: InferenceTrace) {
        // Update source accuracy
        let entry = self.source_accuracy.entry(trace.source.clone()).or_insert((0, 0));
        if trace.is_correct { entry.0 += 1; }
        entry.1 += 1;

        // Update path accuracy
        let entry = self.path_accuracy.entry(trace.decision_path.clone()).or_insert((0, 0));
        if trace.is_correct { entry.0 += 1; }
        entry.1 += 1;

        self.traces.push(trace);
    }

    /// Total accuracy
    pub fn accuracy(&self) -> f64 {
        let correct = self.traces.iter().filter(|t| t.is_correct).count();
        if self.traces.is_empty() { 0.0 } else { correct as f64 / self.traces.len() as f64 }
    }

    /// Generate ablation report: what happens if we remove each expert?
    pub fn ablation_report(&self) -> AblationReport {
        // Collect all expert names
        let mut expert_names: Vec<String> = Vec::new();
        for trace in &self.traces {
            for contribs in &trace.expert_contributions {
                for c in contribs {
                    if !expert_names.contains(&c.expert_name) {
                        expert_names.push(c.expert_name.clone());
                    }
                }
            }
        }
        expert_names.sort();

        let baseline_correct = self.traces.iter().filter(|t| t.is_correct).count();
        let total = self.traces.len();

        let mut expert_impacts: Vec<ExpertImpact> = Vec::new();

        for expert in &expert_names {
            let mut correct_without = 0usize;
            let mut helped = 0usize;  // Expert removal would flip correct → wrong
            let mut hurt = 0usize;    // Expert removal would flip wrong → correct
            let mut neutral = 0usize; // No change

            for trace in &self.traces {
                let answer_without = trace.answer_without_expert(expert);
                let would_be_correct = answer_without == trace.correct_idx;
                if would_be_correct { correct_without += 1; }

                if trace.is_correct && !would_be_correct {
                    helped += 1; // Expert was needed for this correct answer
                } else if !trace.is_correct && would_be_correct {
                    hurt += 1; // Expert caused this wrong answer
                } else {
                    neutral += 1;
                }
            }

            // Compute average absolute contribution
            let mut total_abs_score = 0.0f64;
            let mut score_count = 0usize;
            for trace in &self.traces {
                for contribs in &trace.expert_contributions {
                    for c in contribs {
                        if c.expert_name == *expert {
                            total_abs_score += c.score.abs() as f64;
                            score_count += 1;
                        }
                    }
                }
            }
            let avg_abs_score = if score_count > 0 { total_abs_score / score_count as f64 } else { 0.0 };

            let accuracy_without = if total > 0 { correct_without as f64 / total as f64 } else { 0.0 };
            let accuracy_delta = accuracy_without - (baseline_correct as f64 / total.max(1) as f64);

            expert_impacts.push(ExpertImpact {
                expert_name: expert.clone(),
                helped,
                hurt,
                neutral,
                accuracy_without,
                accuracy_delta,
                avg_abs_score,
            });
        }

        // Sort by impact: most harmful first (positive delta = removing it helps)
        expert_impacts.sort_by(|a, b| b.accuracy_delta.partial_cmp(&a.accuracy_delta).unwrap_or(std::cmp::Ordering::Equal));

        AblationReport {
            total_questions: total,
            baseline_accuracy: if total > 0 { baseline_correct as f64 / total as f64 } else { 0.0 },
            expert_impacts,
            source_accuracy: self.source_accuracy.clone(),
            path_accuracy: self.path_accuracy.clone(),
        }
    }

    /// Print a summary of the most recent N traces
    pub fn print_recent(&self, n: usize) {
        for trace in self.traces.iter().rev().take(n).rev() {
            println!("{}", trace);
        }
    }
}

// =============================================================================
// 5. AblationReport — Per-expert impact analysis
// =============================================================================

/// Impact of removing one expert from the ensemble
#[derive(Debug, Clone)]
pub struct ExpertImpact {
    /// Expert name
    pub expert_name: String,
    /// Questions where this expert was essential (correct → wrong without it)
    pub helped: usize,
    /// Questions where this expert was harmful (wrong → correct without it)
    pub hurt: usize,
    /// Questions where removing it changes nothing
    pub neutral: usize,
    /// Accuracy if this expert were removed
    pub accuracy_without: f64,
    /// Delta from baseline (positive = removing helps, negative = removing hurts)
    pub accuracy_delta: f64,
    /// Average absolute score contribution
    pub avg_abs_score: f64,
}

/// Complete ablation analysis
#[derive(Debug, Clone)]
pub struct AblationReport {
    pub total_questions: usize,
    pub baseline_accuracy: f64,
    pub expert_impacts: Vec<ExpertImpact>,
    pub source_accuracy: HashMap<String, (usize, usize)>,
    pub path_accuracy: HashMap<String, (usize, usize)>,
}

impl fmt::Display for AblationReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "\n{}", "=".repeat(70))?;
        writeln!(f, "  INFERENCE AUDIT REPORT")?;
        writeln!(f, "{}", "=".repeat(70))?;
        writeln!(f, "  Total questions: {}", self.total_questions)?;
        writeln!(f, "  Baseline accuracy: {:.1}%", self.baseline_accuracy * 100.0)?;
        
        // Decision path breakdown
        writeln!(f, "\n  --- Decision Path Breakdown ---")?;
        let mut paths: Vec<_> = self.path_accuracy.iter().collect();
        paths.sort_by_key(|(_, (c, t))| std::cmp::Reverse(*t));
        for (path, (correct, total)) in &paths {
            let acc = if *total > 0 { *correct as f64 / *total as f64 * 100.0 } else { 0.0 };
            writeln!(f, "  {:25} {}/{} ({:.1}%)", path, correct, total, acc)?;
        }

        // Source breakdown
        writeln!(f, "\n  --- Per-Source Accuracy ---")?;
        let mut sources: Vec<_> = self.source_accuracy.iter().collect();
        sources.sort_by_key(|(name, _)| name.clone());
        for (source, (correct, total)) in &sources {
            let acc = if *total > 0 { *correct as f64 / *total as f64 * 100.0 } else { 0.0 };
            writeln!(f, "  {:25} {}/{} ({:.1}%)", source, correct, total, acc)?;
        }

        // Expert ablation table
        writeln!(f, "\n  --- Expert Ablation Analysis ---")?;
        writeln!(f, "  {:20} {:>6} {:>6} {:>6} {:>8} {:>8} {:>8}",
            "Expert", "Helped", "Hurt", "Neut.", "Acc w/o", "Delta", "AvgScore")?;
        writeln!(f, "  {:-<80}", "")?;
        
        for impact in &self.expert_impacts {
            let delta_str = if impact.accuracy_delta > 0.001 {
                format!("+{:.1}%", impact.accuracy_delta * 100.0)
            } else if impact.accuracy_delta < -0.001 {
                format!("{:.1}%", impact.accuracy_delta * 100.0)
            } else {
                "0.0%".to_string()
            };
            
            let verdict = if impact.accuracy_delta > 0.02 { " ← REMOVE" }
                         else if impact.accuracy_delta > 0.005 { " ← WEAK" }
                         else if impact.hurt > impact.helped { " ← NET HARM" }
                         else { "" };
            
            writeln!(f, "  {:20} {:>6} {:>6} {:>6} {:>7.1}% {:>8} {:>8.1}{}",
                impact.expert_name,
                impact.helped,
                impact.hurt,
                impact.neutral,
                impact.accuracy_without * 100.0,
                delta_str,
                impact.avg_abs_score,
                verdict,
            )?;
        }

        // Summary recommendations
        let removable: Vec<_> = self.expert_impacts.iter()
            .filter(|e| e.accuracy_delta > 0.005 || (e.hurt > e.helped && e.helped == 0))
            .collect();
        let harmful: Vec<_> = self.expert_impacts.iter()
            .filter(|e| e.hurt > e.helped)
            .collect();
        let essential: Vec<_> = self.expert_impacts.iter()
            .filter(|e| e.accuracy_delta < -0.02)
            .collect();

        writeln!(f, "\n  --- Recommendations ---")?;
        if !removable.is_empty() {
            writeln!(f, "  REMOVE ({} experts): {}", removable.len(),
                removable.iter().map(|e| e.expert_name.as_str()).collect::<Vec<_>>().join(", "))?;
        }
        if !harmful.is_empty() {
            writeln!(f, "  NET HARMFUL ({} experts): {}", harmful.len(),
                harmful.iter().map(|e| e.expert_name.as_str()).collect::<Vec<_>>().join(", "))?;
        }
        if !essential.is_empty() {
            writeln!(f, "  ESSENTIAL ({} experts): {}", essential.len(),
                essential.iter().map(|e| e.expert_name.as_str()).collect::<Vec<_>>().join(", "))?;
        }

        writeln!(f, "{}"  , "=".repeat(70))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_ablation() {
        let mut trace = InferenceTrace::new("q1", "mmlu", "What is 2+2?", 4, 2);
        trace.record_expert(0, "semantic", 5.0);
        trace.record_expert(0, "rag", 3.0);
        trace.record_expert(1, "semantic", 4.0);
        trace.record_expert(1, "rag", 6.0);
        trace.record_expert(2, "semantic", 8.0);  // correct
        trace.record_expert(2, "rag", 2.0);
        trace.record_expert(3, "semantic", 3.0);
        trace.record_expert(3, "rag", 1.0);
        trace.finalize(2, 0.8, "expert-high", &[8.0, 10.0, 10.0, 4.0], &[0.1, 0.35, 0.45, 0.1]);

        // Without semantic, choice 1 wins (rag=6 > rag=2)
        let without_semantic = trace.answer_without_expert("semantic");
        assert_eq!(without_semantic, 1); // rag dominates: 3, 6, 2, 1

        let mut collector = AuditCollector::new();
        collector.record(trace);
        let report = collector.ablation_report();
        assert_eq!(report.total_questions, 1);
        assert_eq!(report.baseline_accuracy, 1.0);
    }
}
