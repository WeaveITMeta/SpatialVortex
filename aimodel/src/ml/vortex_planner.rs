//! Vortex Rule-Based Symbolic Planner
//!
//! # Table of Contents
//! 1. PlanStep         — Single reasoning step with vortex position + rule engine used
//! 2. PlanTrace        — Full trace of sacred gate outcomes + confidence per step
//! 3. SacredGateResult — Immutable verification outcome at positions 3, 6, 9
//! 4. VortexSymbolicPlanner — Zero-shot real-time planner over existing rule engines
//! 5. Integration      — `plan_babi_question()` entry point for bAbI tasks 3–20
//!
//! # Design Principles (from vortex-rules.md + PDDL-Instruct paper)
//! - Zero training: everything runs from compiled rule engines at inference time
//! - Vortex cycle (1→2→4→8→7→5) maps to explicit logical CoT steps
//! - Sacred positions (3, 6, 9) are IMMUTABLE external verifiers — observe only
//! - No gradient or weight update — pure runtime statistics via calibration
//! - Every gate decision is recorded in PlanTrace for full auditability

use std::collections::HashMap;
use crate::ml::transitive_flux::{TransitiveFluxReasoner, CountingMode};
use crate::data::inference_audit::InferenceTrace;

// =============================================================================
// 1. PlanStep — Single reasoning step
// =============================================================================

/// A single reasoning step produced at one vortex cycle position
#[derive(Debug, Clone)]
pub struct PlanStep {
    /// Vortex position that generated this step (1,2,4,8,7,5)
    pub vortex_position: u8,
    /// Human-readable description of what was done at this position
    pub description: String,
    /// Best candidate answer after this step (empty = not yet resolved)
    pub candidate: String,
    /// Confidence in this candidate (0.0–1.0)
    pub confidence: f32,
    /// Which rule engine produced the candidate
    pub rule_engine: &'static str,
}

// =============================================================================
// 2. SacredGateResult — Immutable verification at positions 3, 6, 9
// =============================================================================

/// Result of a sacred position verification gate (observe-only, never mutates)
#[derive(Debug, Clone)]
pub struct SacredGateResult {
    /// Sacred position (3, 6, or 9)
    pub position: u8,
    /// Whether verification passed
    pub passed: bool,
    /// Signal emitted (proximity / coherence / validation)
    pub signal: &'static str,
    /// Confidence in the gate outcome
    pub confidence: f32,
    /// Optional note for trace output
    pub note: String,
}

// =============================================================================
// 3. PlanTrace — Full annotated trace across all positions
// =============================================================================

/// Full trace of a planning execution for auditability
#[derive(Debug, Clone)]
pub struct PlanTrace {
    /// Original query
    pub query: String,
    /// Steps taken at each vortex position
    pub steps: Vec<PlanStep>,
    /// Sacred gate outcomes
    pub gates: Vec<SacredGateResult>,
    /// Final answer emitted
    pub final_answer: String,
    /// Final confidence
    pub final_confidence: f32,
    /// Which bAbI task category (e.g., "path", "counting", "spatial", "deduction")
    pub task_category: String,
}

impl PlanTrace {
    /// Attach this trace to an existing InferenceTrace (for audit integration)
    pub fn record_into(&self, trace: &mut InferenceTrace) {
        for gate in &self.gates {
            let signal = format!("SacredGate-{}: {} ({})", gate.position, gate.signal, gate.note);
            trace.record_decision(&signal, if gate.passed { "passed" } else { "failed" }, gate.confidence);
        }
        trace.record_decision(
            &format!("planner({})", self.task_category),
            &self.final_answer,
            self.final_confidence,
        );
    }
}

// =============================================================================
// 4. VortexSymbolicPlanner
// =============================================================================

/// Zero-shot real-time symbolic planner using the vortex rule engines.
///
/// Does NOT use any pretrained weights, datasets, or training loops.
/// Everything is determined by deterministic rule application at inference time.
pub struct VortexSymbolicPlanner {
    /// Embedded transitive flux reasoner (shared state across positions)
    flux: TransitiveFluxReasoner,
    /// Simple runtime calibration: accuracy window per task category
    calibration: HashMap<String, (u32, u32)>, // (correct, total) per category
}

impl VortexSymbolicPlanner {
    pub fn new() -> Self {
        let mut flux = TransitiveFluxReasoner::new(64);
        flux.set_counting_mode(CountingMode::Sequential);
        Self {
            flux,
            calibration: HashMap::new(),
        }
    }

    // -------------------------------------------------------------------------
    // Public entry point: plan over a bAbI question
    // -------------------------------------------------------------------------

    /// Main entry point. Given a bAbI question (context + query in one string)
    /// and a list of multiple-choice answers, returns (choice_index, confidence, trace).
    ///
    /// Runs the full 6-step vortex cycle with 3 sacred verification gates.
    pub fn plan_babi_question(
        &mut self,
        full_question: &str,
        choices: &[String],
    ) -> Option<(usize, f32, PlanTrace)> {
        let question_lower = full_question.to_lowercase();
        let category = self.detect_category(&question_lower);

        let mut trace = PlanTrace {
            query: question_lower.chars().take(80).collect(),
            steps: Vec::new(),
            gates: Vec::new(),
            final_answer: String::new(),
            final_confidence: 0.0,
            task_category: category.clone(),
        };

        // ===========================================================
        // POSITION 1: Entity & initial state extraction
        // Rule engine: TransitiveFluxReasoner::extract_context (all modes)
        // CRITICAL: Extract from context lines ONLY — not the question line.
        // If the question line is fed to extract_relations, bAbI task 17 questions
        // like "Is X to the right of Y?" pollute the relations graph with false facts.
        // ===========================================================
        self.flux.clear();

        // Split: everything before the last line is context; last line is the query
        let context_only = {
            let lines: Vec<&str> = question_lower.lines().collect();
            if lines.len() > 1 {
                lines[..lines.len() - 1].join("\n")
            } else {
                question_lower.clone()
            }
        };
        let query_line = question_lower.lines().last().unwrap_or(&question_lower).to_string();

        self.flux.extract_relations(&context_only);
        self.flux.extract_locations(&context_only);
        self.flux.extract_counts(&context_only);

        let entity_count = self.flux.adjacency.len();
        let has_relations = entity_count > 0 || !self.flux.relations_is_empty();

        trace.steps.push(PlanStep {
            vortex_position: 1,
            description: format!("Extracted {} location nodes, {} relations from context",
                entity_count, self.flux.adjacency.len()),
            candidate: String::new(),
            confidence: if has_relations { 0.8 } else { 0.3 },
            rule_engine: "TransitiveFluxReasoner::extract_context",
        });

        // ===========================================================
        // POSITION 2: Build basic relations / adjacency graph
        // Rule engine: TransitiveFluxReasoner adjacency already built at pos 1
        // ===========================================================
        let adjacency_size = self.flux.adjacency.len();
        trace.steps.push(PlanStep {
            vortex_position: 2,
            description: format!("Adjacency graph: {} nodes", adjacency_size),
            candidate: String::new(),
            confidence: if adjacency_size > 0 { 0.85 } else { 0.2 },
            rule_engine: "TransitiveFluxReasoner::adjacency",
        });

        // ===========================================================
        // SACRED GATE 3: External verification — does context have enough
        // relational structure to support symbolic planning?
        // Immutable: only emits proximity signal, never modifies state.
        // Passes on: adjacency graph OR locations map OR spatial/size relations
        // - adjacency > 0   → tasks 1,2,3,4,19 (movement/pathfinding)
        // - has_locations   → tasks 6,9 (location yes/no)
        // - has_relations   → tasks 17,18 (spatial/size yes/no via transitive graph)
        // ===========================================================
        let has_locations = !self.flux.locations_populated();
        let has_relations = !self.flux.relations_is_empty();
        // Pickup/drop events in context signal tasks 7 and 8 (counting/possession)
        let has_pickup_events = context_only.contains("picked up") || context_only.contains("grabbed")
            || context_only.contains("got the") || context_only.contains("took the")
            || context_only.contains("gave the") || context_only.contains("gave ")
            || context_only.contains("dropped") || context_only.contains("put down")
            || context_only.contains("discarded") || context_only.contains("handed the");
        let gate3_passed = adjacency_size > 0 || has_locations || has_relations || has_pickup_events;
        let gate3_conf = if gate3_passed { 0.9 } else { 0.3 };
        trace.gates.push(SacredGateResult {
            position: 3,
            signal: "proximity",
            passed: gate3_passed,
            confidence: gate3_conf,
            note: format!("adjacency={} entities", adjacency_size),
        });

        // If no relational structure at gate 3, symbolic planner cannot contribute
        if !gate3_passed {
            return None;
        }

        // ===========================================================
        // POSITION 4: Sub-event resolution
        // Covers: counting (task 7), possession lists (task 8),
        //         location yes/no (tasks 3,6,9), negation (task 9)
        // Rule engine: extract_counts + locations map
        // ===========================================================
        // query_line: the actual question text; context_only: story lines for item tracking
        let candidate_pos4 = self.try_counting_answer(&query_line, choices)
            .or_else(|| self.try_possession_answer(&context_only, &query_line, choices))
            .or_else(|| self.try_give_transfer_answer(&context_only, &query_line, choices))
            .or_else(|| self.try_indefinite_maybe_answer(&context_only, &query_line, choices))
            .or_else(|| self.try_temporal_location_answer(&context_only, &query_line, choices))
            .or_else(|| self.try_location_yesno_answer(&query_line, choices));
        trace.steps.push(PlanStep {
            vortex_position: 4,
            description: "Counting / possession / location yes-no resolution".to_string(),
            candidate: candidate_pos4.as_ref().map(|(_, a, _)| a.clone()).unwrap_or_default(),
            confidence: candidate_pos4.as_ref().map(|(_, _, c)| *c).unwrap_or(0.0),
            rule_engine: "TransitiveFluxReasoner::extract_counts+locations",
        });

        // ===========================================================
        // POSITION 5: Partial plan synthesis
        // Covers: path finding (task 19), spatial yes/no (tasks 17,18),
        //         directional neighbor query (tasks 4,17),
        //         possession/transfer queries (task 5)
        // Rule engine: find_path / score_yes_no / adjacency neighbor lookup
        // ===========================================================
        let candidate_pos5 = self.try_path_or_spatial_answer(&query_line, choices)
            .or_else(|| self.try_directional_neighbor_answer(&query_line, choices));
        trace.steps.push(PlanStep {
            vortex_position: 5,
            description: "Path / spatial / directional neighbor resolution".to_string(),
            candidate: candidate_pos5.as_ref().map(|(_, a, _)| a.clone()).unwrap_or_default(),
            confidence: candidate_pos5.as_ref().map(|(_, _, c)| *c).unwrap_or(0.0),
            rule_engine: "TransitiveFluxReasoner::find_path/score_yes_no/adjacency",
        });

        // ===========================================================
        // SACRED GATE 6: Balance & coherence check
        // Immutable: emits coherence signal. Checks whether pos4/pos5
        // produce a non-empty candidate with sufficient confidence.
        // ===========================================================
        let best_so_far = [&candidate_pos4, &candidate_pos5]
            .iter()
            .filter_map(|c| c.as_ref())
            .max_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

        let gate6_passed = best_so_far.map(|(_, ans, conf)| !ans.is_empty() && *conf > 0.4).unwrap_or(false);
        let gate6_conf = best_so_far.map(|(_, _, c)| *c).unwrap_or(0.0);
        trace.gates.push(SacredGateResult {
            position: 6,
            signal: "coherence",
            passed: gate6_passed,
            confidence: gate6_conf,
            note: format!("best_candidate_conf={:.2}", gate6_conf),
        });

        // ===========================================================
        // POSITION 7: Confidence refinement via calibration
        // Rule engine: TransitiveFluxReasoner::get_calibrated_confidence
        // ===========================================================
        let calibrated_conf = if gate6_passed {
            self.flux.get_calibrated_confidence(gate6_conf)
        } else {
            0.0
        };
        trace.steps.push(PlanStep {
            vortex_position: 7,
            description: format!("Calibration: raw={:.2} → calibrated={:.2}", gate6_conf, calibrated_conf),
            candidate: best_so_far.map(|(_, a, _)| a.clone()).unwrap_or_default(),
            confidence: calibrated_conf,
            rule_engine: "TransitiveFluxReasoner::get_calibrated_confidence",
        });

        // ===========================================================
        // POSITION 8: Full plan assembly — pick best candidate from pos 4/5/7
        // Rule engine: score_answer_comprehensive (routing)
        // ===========================================================
        let assembled = self.assemble_best_answer(
            &query_line,
            choices,
            &[candidate_pos4.as_ref(), candidate_pos5.as_ref()],
        );

        let (best_idx, best_ans, best_conf) = match assembled {
            Some(triple) => triple,
            None => {
                // No evidence from any resolver — abstain entirely
                trace.steps.push(PlanStep {
                    vortex_position: 8,
                    description: "No evidence — abstaining".to_string(),
                    candidate: String::new(),
                    confidence: 0.0,
                    rule_engine: "VortexSymbolicPlanner::assemble_best_answer",
                });
                return None;
            }
        };

        trace.steps.push(PlanStep {
            vortex_position: 8,
            description: format!("Plan assembly: chose idx={} ans=\"{}\" conf={:.2}", best_idx, best_ans, best_conf),
            candidate: best_ans.clone(),
            confidence: best_conf,
            rule_engine: "VortexSymbolicPlanner::assemble_best_answer",
        });

        // ===========================================================
        // SACRED GATE 9: Full validation — does the assembled plan
        // match an available choice and exceed the confidence threshold?
        // Immutable: emits validation signal, never modifies state.
        // ===========================================================
        let threshold = self.runtime_threshold(&category);
        let gate9_passed = best_conf >= threshold && !best_ans.is_empty();
        trace.gates.push(SacredGateResult {
            position: 9,
            signal: "validation",
            passed: gate9_passed,
            confidence: best_conf,
            note: format!("threshold={:.2} choice_idx={}", threshold, best_idx),
        });

        trace.final_answer = best_ans;
        trace.final_confidence = best_conf;

        if gate9_passed {
            Some((best_idx, best_conf, trace))
        } else {
            // Return partial trace as None so caller falls through to other experts
            None
        }
    }

    // -------------------------------------------------------------------------
    // Update runtime calibration (call after each question)
    // -------------------------------------------------------------------------

    /// Update runtime statistics — pure accuracy tracking, no weight update
    pub fn update_calibration(&mut self, category: &str, was_correct: bool, confidence: f32) {
        let entry = self.calibration.entry(category.to_string()).or_insert((0, 0));
        if was_correct { entry.0 += 1; }
        entry.1 += 1;
        // Feed back into flux calibration
        self.flux.update_calibration(was_correct, confidence);
    }

    // -------------------------------------------------------------------------
    // Position 4 helper A: counting (task 7)
    // -------------------------------------------------------------------------

    fn try_counting_answer(
        &self,
        question_lower: &str,
        choices: &[String],
    ) -> Option<(usize, String, f32)> {
        if !question_lower.contains("how many") {
            return None;
        }
        if let Some((count, conf)) = self.flux.answer_counting_question(question_lower) {
            // Guard: if count=0 but no pickup/drop events were observed in context,
            // the context may be truncated — abstain rather than commit wrong "none".
            if count == 0 && !self.flux.has_entity_events() {
                return None;
            }
            // Boost confidence for count=0 when events ARE present —
            // count=0 with known events is reliable evidence (entity tracked, has 0 items).
            let conf = if count == 0 && self.flux.has_entity_events() {
                conf.max(0.50)
            } else {
                conf
            };
            let count_str = count.to_string();
            let count_words = ["none", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine"];
            for (idx, choice) in choices.iter().enumerate() {
                let cl = choice.to_lowercase();
                let matches = cl == count_str
                    || (count >= 0 && count < 10 && cl == count_words[count as usize])
                    || cl.contains(&count_str);
                if matches {
                    return Some((idx, choice.clone(), conf));
                }
            }
        }
        None
    }

    // -------------------------------------------------------------------------
    // Position 4 helper B: possession / carrying (tasks 5, 8)
    // "What is X carrying?" / "What is X holding?" / "What did X give to Y?"
    // -------------------------------------------------------------------------

    fn try_possession_answer(
        &self,
        context_only: &str,
        query_line: &str,
        choices: &[String],
    ) -> Option<(usize, String, f32)> {
        let is_carrying = query_line.contains("carrying")
            || query_line.contains("holding")
            || query_line.contains("what is") && query_line.contains(" have ");
        if !is_carrying {
            return None;
        }

        // Extract entity being asked about ("what is X carrying?")
        let entity = self.extract_entity_from_possession_query(query_line);
        if entity.is_empty() {
            return None;
        }

        // Scan context lines (not the question itself) for pickup/drop events
        let items = self.extract_carried_items(context_only, &entity);
        if items.is_empty() {
            // Entity has nothing — if pickup events exist in context, answer "nothing"
            if self.flux.has_entity_events() {
                for (idx, choice) in choices.iter().enumerate() {
                    let cl = choice.to_lowercase();
                    if cl == "nothing" || cl == "none" || cl == "nothing." {
                        return Some((idx, choice.clone(), 0.60));
                    }
                }
            }
            return None;
        }

        // Items may be comma-separated in bAbI (e.g., "football,apple")
        let item_str = items.join(",");
        let conf = 0.75f32;

        // Try to match against choices (exact or partial)
        for (idx, choice) in choices.iter().enumerate() {
            let cl = choice.to_lowercase();
            // Single item match
            if items.len() == 1 && cl == items[0] {
                return Some((idx, choice.clone(), conf));
            }
            // Multi-item: choice contains all items
            if items.iter().all(|item| cl.contains(item.as_str())) {
                return Some((idx, choice.clone(), conf));
            }
            // Choice is in items
            if items.contains(&cl) {
                return Some((idx, choice.clone(), conf * 0.8));
            }
        }
        // Fallback: return the best matching choice by overlap
        let best = choices.iter().enumerate().max_by_key(|(_, c)| {
            let cl = c.to_lowercase();
            items.iter().filter(|item| cl.contains(item.as_str()) || item.contains(&cl)).count()
        });
        if let Some((idx, choice)) = best {
            if items.iter().any(|item| choice.to_lowercase().contains(item.as_str())) {
                return Some((idx, choice.clone(), conf * 0.6));
            }
        }
        None
    }

    /// Extract entity name from "what is X carrying?" style queries
    fn extract_entity_from_possession_query(&self, q: &str) -> String {
        // Pattern: "what is X carrying?" / "what is X holding?"
        for pattern in &["what is ", "what does "] {
            if let Some(pos) = q.find(pattern) {
                let after = &q[pos + pattern.len()..];
                let end = after.find(|c: char| c == ' ').unwrap_or(after.len());
                let entity = after[..end].trim().to_string();
                if !entity.is_empty() && entity != "the" {
                    return entity;
                }
            }
        }
        String::new()
    }

    /// Extract items currently carried by an entity from context lines
    /// Parses pickup/drop events in order to maintain current possession state
    fn extract_carried_items(&self, context: &str, entity: &str) -> Vec<String> {
        let entity_lower = entity.to_lowercase();
        let mut carrying: std::collections::HashSet<String> = std::collections::HashSet::new();

        let acquire_patterns = ["picked up", "got the", "got ", "grabbed", "took", "received"];
        let drop_patterns = ["dropped", "put down", "discarded", "left"];
        let give_pattern = "gave the ";
        let give_pattern2 = "gave ";
        let to_sep = " to ";

        for line in context.lines() {
            let line_lower = line.to_lowercase();
            let content = line_lower.trim_start_matches(|c: char| c.is_ascii_digit() || c == ' ');

            // Handle "X gave Y to Z": drop Y from X, add Y to Z
            for gp in &[give_pattern, give_pattern2] {
                if content.contains(gp) {
                    if let Some(gp_pos) = content.find(gp) {
                        let giver = content[..gp_pos].trim().split_whitespace().last().unwrap_or("").to_string();
                        let after_gp = &content[gp_pos + gp.len()..];
                        if let Some(to_pos) = after_gp.find(to_sep) {
                            let obj = after_gp[..to_pos].trim().to_string();
                            let receiver = after_gp[to_pos + to_sep.len()..]
                                .trim().trim_end_matches('.').split_whitespace().next().unwrap_or("").to_string();
                            if !obj.is_empty() && !receiver.is_empty() {
                                if giver == entity_lower {
                                    carrying.remove(&obj);
                                }
                                if receiver == entity_lower {
                                    carrying.insert(obj.clone());
                                }
                            }
                        }
                    }
                }
            }

            // Check if this line involves our entity (for pickup/drop)
            if !content.starts_with(&entity_lower) && !content.contains(&format!(" {} ", entity_lower)) {
                continue;
            }

            for pattern in &acquire_patterns {
                if content.contains(pattern) {
                    if let Some(pos) = content.find(pattern) {
                        let after = content[pos + pattern.len()..].trim();
                        let item = after
                            .trim_start_matches("the ")
                            .split_whitespace()
                            .next()
                            .unwrap_or("")
                            .trim_end_matches(|c: char| !c.is_alphanumeric())
                            .to_string();
                        // Filter out non-item words ("there", empty)
                        if !item.is_empty() && item.len() > 1 && item != "there" {
                            carrying.insert(item);
                        }
                    }
                }
            }
            for pattern in &drop_patterns {
                if content.contains(pattern) {
                    if let Some(pos) = content.find(pattern) {
                        let after = content[pos + pattern.len()..].trim();
                        let item = after
                            .trim_start_matches("the ")
                            .split_whitespace()
                            .next()
                            .unwrap_or("")
                            .trim_end_matches(|c: char| !c.is_alphanumeric())
                            .to_string();
                        carrying.remove(&item);
                    }
                }
            }
        }
        let mut items: Vec<String> = carrying.into_iter().collect();
        items.sort();
        items
    }

    // -------------------------------------------------------------------------
    // Position 4 helper C: give/transfer tracking (task 5)
    // "Who gave X to Y?", "What did X give to Y?", "Who received X?"
    // -------------------------------------------------------------------------

    fn try_give_transfer_answer(
        &self,
        context_only: &str,
        query_line: &str,
        choices: &[String],
    ) -> Option<(usize, String, f32)> {
        // Only fire on give-style questions
        let is_give_q = query_line.contains("who gave") || query_line.contains("who handed")
            || query_line.contains("who passed") || query_line.contains("who gave")
            || query_line.contains("what did") && (query_line.contains("give") || query_line.contains("hand") || query_line.contains("pass"))
            || query_line.contains("who did") && (query_line.contains("give") || query_line.contains("hand") || query_line.contains("pass") || query_line.contains("receive"))
            || query_line.contains("who received") || query_line.contains("who got the");
        if !is_give_q {
            return None;
        }

        // Transfer patterns: "X gave Y to Z", "X handed Y to Z", "X passed Y to Z"
        let transfer_patterns = ["gave the ", "gave ", "handed the ", "handed ", "passed the ", "passed "];
        let to_separator = " to ";

        // Track: object → (giver, receiver) from the most recent transfer event
        let mut transfers: std::collections::HashMap<String, (String, String)> = std::collections::HashMap::new();

        for line in context_only.lines() {
            let content = line.trim().to_lowercase();
            let content = content.trim_start_matches(|c: char| c.is_ascii_digit() || c == ' ');

            for pattern in &transfer_patterns {
                if let Some(pat_pos) = content.find(pattern) {
                    let giver = content[..pat_pos].trim().split_whitespace().last().unwrap_or("").to_string();
                    if giver.is_empty() { continue; }
                    let after_pat = &content[pat_pos + pattern.len()..];
                    if let Some(to_pos) = after_pat.find(to_separator) {
                        let object = after_pat[..to_pos].trim().to_string();
                        let receiver = after_pat[to_pos + to_separator.len()..]
                            .trim().trim_end_matches('.').split_whitespace().next().unwrap_or("").to_string();
                        if !object.is_empty() && !receiver.is_empty() {
                            transfers.insert(object.clone(), (giver.clone(), receiver.clone()));
                        }
                    }
                    break;
                }
            }
        }
        if transfers.is_empty() {
            return None;
        }

        // Determine what's being asked
        let answer: Option<String> = if query_line.contains("who gave") || query_line.contains("who handed") || query_line.contains("who passed") {
            // "Who gave the football to Jeff?" — find giver of the object
            let object = Self::extract_object_from_query(query_line);
            transfers.get(&object).map(|(giver, _)| giver.clone())
                .or_else(|| transfers.values().map(|(g, _)| g.clone()).next())
        } else if query_line.contains("what did") {
            // "What did Fred give to Jeff?" — find object given
            let giver = query_line.split_whitespace().nth(2).unwrap_or("").to_string();
            transfers.iter().find(|(_, (g, _))| g == &giver)
                .map(|(obj, _)| obj.clone())
        } else if query_line.contains("who did") && (query_line.contains("give") || query_line.contains("hand")) {
            // "Who did Fred give the football to?" → receiver
            let object = Self::extract_object_from_query(query_line);
            transfers.get(&object).map(|(_, recv)| recv.clone())
                .or_else(|| transfers.values().map(|(_, r)| r.clone()).next())
        } else if query_line.contains("who received") || query_line.contains("who got the") {
            let object = Self::extract_object_from_query(query_line);
            transfers.get(&object).map(|(_, recv)| recv.clone())
                .or_else(|| transfers.values().map(|(_, r)| r.clone()).next())
        } else {
            None
        };

        let answer = answer?;
        for (idx, choice) in choices.iter().enumerate() {
            let cl = choice.to_lowercase();
            if cl == answer || cl.contains(&answer) || answer.contains(&cl) {
                return Some((idx, choice.clone(), 0.80));
            }
        }
        None
    }

    fn extract_object_from_query(query: &str) -> String {
        // Find "the X" after gave/gave/received/got
        for marker in &["gave the ", "handed the ", "received the ", "got the ", "give the "] {
            if let Some(pos) = query.find(marker) {
                let after = &query[pos + marker.len()..];
                let end = after.find(|c: char| c == ' ' || c == '?').unwrap_or(after.len());
                return after[..end].trim().to_string();
            }
        }
        String::new()
    }

    // -------------------------------------------------------------------------
    // Position 4 helper D: location yes/no (tasks 3, 6, 9)
    // "Is X in Y?" resolved via locations map
    // -------------------------------------------------------------------------

    // -------------------------------------------------------------------------
    // Position 4 helper D: indefinite knowledge (task 10)
    // "X is either in A or B" + "Is X in Y?" → "maybe"
    // Pure pattern detection — no location map, no ambiguity
    // -------------------------------------------------------------------------

    fn try_indefinite_maybe_answer(
        &self,
        context_only: &str,
        query_line: &str,
        choices: &[String],
    ) -> Option<(usize, String, f32)> {
        // Only fire when "maybe" is a valid choice
        let maybe_idx = choices.iter().position(|c| c.to_lowercase() == "maybe")?;
        // Query must be "Is X in Y?" or "Is X in the Y?"
        let q = query_line.trim();
        if !q.starts_with("is ") {
            return None;
        }
        let has_yes = choices.iter().any(|c| c.to_lowercase() == "yes");
        let has_no  = choices.iter().any(|c| c.to_lowercase() == "no");
        if !has_yes || !has_no {
            return None;
        }

        // Extract entity (word after "is") and queried location (after "in [the]")
        let words: Vec<&str> = q.split_whitespace().collect();
        let entity = words.get(1).copied().unwrap_or("").to_string();
        if entity.is_empty() {
            return None;
        }
        let queried_location = if let Some(pos) = q.find(" in the ") {
            q[pos + 9..].trim_end_matches('?').trim().to_string()
        } else if let Some(pos) = q.find(" in ") {
            q[pos + 4..].trim_end_matches('?').trim().to_string()
        } else {
            return None;
        };
        if queried_location.is_empty() {
            return None;
        }

        // Scan context lines: find the LAST indefinite statement about this entity.
        // Track the two "either A or B" options.
        // A later definite movement clears the indefinite state.
        let mut indefinite_options: Vec<String> = Vec::new();
        for line in context_only.lines() {
            let line = line.trim().to_lowercase();
            let content = line
                .trim_start_matches(|c: char| c.is_ascii_digit())
                .trim_start();
            if content.starts_with(&entity) {
                if content.contains("either in") || content.contains("either in the") {
                    // Parse "X is either in A or B" → extract A and B
                    indefinite_options.clear();
                    if let Some(either_pos) = content.find("either in") {
                        let after = content[either_pos + 9..].trim_start_matches(" the ").trim();
                        // Split on " or " to get both options
                        let parts: Vec<&str> = after.splitn(2, " or ").collect();
                        if parts.len() == 2 {
                            let a = parts[0].trim_start_matches("the ").trim().to_string();
                            let b = parts[1].trim_start_matches("the ")
                                .trim_end_matches('.')
                                .trim()
                                .to_string();
                            indefinite_options.push(a);
                            indefinite_options.push(b);
                        }
                    }
                } else if content.contains("went") || content.contains("moved")
                    || content.contains("travelled") || content.contains("traveled")
                    || content.contains("journeyed") || content.contains("went back")
                    || content.contains("is in ") || content.contains("is in the ")
                    || content.contains("is no longer") || content.contains("is not in")
                {
                    // Definite movement resolves indefiniteness
                    indefinite_options.clear();
                }
            }
        }

        if indefinite_options.is_empty() {
            return None;
        }

        // "maybe" only if queried location IS one of the two indefinite options
        let is_maybe = indefinite_options.iter().any(|opt| {
            opt == &queried_location
                || opt.contains(&queried_location)
                || queried_location.contains(opt.as_str())
        });

        if is_maybe {
            Some((maybe_idx, choices[maybe_idx].clone(), 0.82))
        } else {
            // Queried location is NOT in the options → definitively "no"
            if let Some(no_idx) = choices.iter().position(|c| c.to_lowercase() == "no") {
                Some((no_idx, choices[no_idx].clone(), 0.80))
            } else {
                None
            }
        }
    }

    // -------------------------------------------------------------------------
    // Position 4 helper: temporal location history (task 14)
    // "Where was X before Y?" — track sequence of locations per entity
    // -------------------------------------------------------------------------

    fn try_temporal_location_answer(
        &self,
        context_only: &str,
        query_line: &str,
        choices: &[String],
    ) -> Option<(usize, String, f32)> {
        // Only fire on "where was X before Y?" pattern
        if !query_line.contains("where was") || !query_line.contains("before") {
            return None;
        }
        // Parse: "where was X before the Y?" → entity=X, target_location=Y
        let after_was = query_line.split("where was ").nth(1)?;
        let before_pos = after_was.find(" before ")?;
        let entity = after_was[..before_pos].trim()
            .trim_start_matches("the ").to_string();
        let target_location = after_was[before_pos + 8..]
            .trim().trim_end_matches('?').trim()
            .trim_start_matches("the ").to_string();
        if entity.is_empty() || target_location.is_empty() {
            return None;
        }

        // Determine if we're tracking a person or an object
        // People move directly; objects move via their carriers
        let movement_verbs = ["went to", "moved to", "travelled to", "journeyed to", "went back to"];
        let acquire_verbs = ["picked up the ", "picked up ", "got the ", "grabbed the ", "took the "];
        let drop_verbs = ["dropped the ", "put down the ", "discarded the ", "left the "];

        // Track person locations AND object carriers/locations
        let mut person_locations: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        // object → (carrier OR location_name, is_carried: bool)
        let mut object_carrier: std::collections::HashMap<String, (String, bool)> = std::collections::HashMap::new();
        // Location history for the queried entity: Vec<(order, location)>
        let mut history: Vec<(u32, String)> = Vec::new();
        let mut global_order: u32 = 0;

        let is_person = {
            // Heuristic: persons are capitalized names in bAbI; objects are common nouns
            // Check if entity appears as subject of movement verbs
            let el = entity.to_lowercase();
            context_only.lines().any(|l| {
                let c = l.trim().to_lowercase();
                let c = c.trim_start_matches(|ch: char| ch.is_ascii_digit() || ch == ' ');
                c.starts_with(&el) && movement_verbs.iter().any(|v| c.contains(v))
            })
        };

        for line in context_only.lines() {
            let line_lower = line.trim().to_lowercase();
            let content = line_lower.trim_start_matches(|c: char| c.is_ascii_digit() || c == ' ').trim();
            if content.is_empty() { global_order += 1; continue; }

            // Extract time marker for ordering
            let time_bonus = if content.contains("yesterday") { 0u32 }
                else if content.contains("this morning") { 100 }
                else if content.contains("this afternoon") { 200 }
                else if content.contains("this evening") { 300 }
                else { 0 };
            let order = time_bonus + global_order;

            // Strip leading time markers before extracting subject
            let content_stripped = content
                .trim_start_matches("yesterday ")
                .trim_start_matches("this morning ")
                .trim_start_matches("this afternoon ")
                .trim_start_matches("this evening ")
                .trim();
            // Extract subject (first word after time marker)
            let subject = content_stripped.split_whitespace().next().unwrap_or("").to_string();

            // Track person movements
            for verb in &movement_verbs {
                if content.contains(verb) {
                    if let Some(verb_pos) = content.find(verb) {
                        let after_verb = &content[verb_pos + verb.len()..];
                        let location = after_verb.trim()
                            .trim_start_matches("the ")
                            .split(|c: char| c == '.' || c == '\n')
                            .next().unwrap_or("").trim()
                            .trim_end_matches("this morning")
                            .trim_end_matches("this afternoon")
                            .trim_end_matches("this evening")
                            .trim_end_matches("yesterday")
                            .trim_end_matches('.').trim().to_string();
                        if !location.is_empty() {
                            person_locations.insert(subject.clone(), location.clone());
                            // If this person is the queried entity (person mode)
                            if is_person && subject == entity {
                                history.push((order, location.clone()));
                            }
                            // If this person carries a tracked object, update object location
                            if !is_person {
                                // Check all carried objects
                                for (obj, (carrier, carried)) in object_carrier.iter_mut() {
                                    if *carried && *carrier == subject {
                                        // Object moves with carrier
                                        if *obj == entity {
                                            history.push((order, location.clone()));
                                        }
                                    }
                                }
                            }
                        }
                    }
                    break;
                }
            }

            // Track object pickups: "X picked up the Y" / "X got the Y" / "X grabbed the Y"
            for verb in &acquire_verbs {
                if content.contains(verb) {
                    if let Some(pos) = content.find(verb) {
                        let after = &content[pos + verb.len()..];
                        let obj = after.trim()
                            .split_whitespace().next().unwrap_or("")
                            .trim_end_matches('.').trim_end_matches(|c: char| !c.is_alphanumeric())
                            .to_string();
                        if !obj.is_empty() && obj != "there" && obj.len() > 1 {
                            object_carrier.insert(obj.clone(), (subject.clone(), true));
                            // If this object is our query target, record its location as carrier's location
                            if !is_person && obj == entity {
                                if let Some(loc) = person_locations.get(&subject) {
                                    history.push((order, loc.clone()));
                                }
                            }
                        }
                    }
                    break;
                }
            }

            // Track object drops: "X dropped the Y" / "X put down the Y" / "X left the Y"
            for verb in &drop_verbs {
                if content.contains(verb) {
                    if let Some(pos) = content.find(verb) {
                        let after = &content[pos + verb.len()..];
                        let obj = after.trim()
                            .split_whitespace().next().unwrap_or("")
                            .trim_end_matches('.').trim_end_matches(|c: char| !c.is_alphanumeric())
                            .to_string();
                        // Disambiguate "left": only treat as drop if obj is a known carried object,
                        // not a location. "sandra left the hallway" = departure, not drop.
                        let is_real_drop = if verb.starts_with("left") {
                            object_carrier.contains_key(&obj)
                        } else {
                            true
                        };
                        if !obj.is_empty() && obj.len() > 1 && is_real_drop {
                            // Object is now at carrier's current location
                            if let Some(loc) = person_locations.get(&subject) {
                                object_carrier.insert(obj.clone(), (loc.clone(), false));
                                if !is_person && obj == entity {
                                    history.push((order, loc.clone()));
                                }
                            }
                        }
                    }
                    break;
                }
            }

            // Track give: "X gave the Y to Z"
            if content.contains("gave") {
                for gp in &["gave the ", "gave "] {
                    if let Some(gp_pos) = content.find(gp) {
                        let after_gp = &content[gp_pos + gp.len()..];
                        if let Some(to_pos) = after_gp.find(" to ") {
                            let obj = after_gp[..to_pos].trim().to_string();
                            let receiver = after_gp[to_pos + 4..].trim()
                                .trim_end_matches('.').split_whitespace().next().unwrap_or("").to_string();
                            if !obj.is_empty() && !receiver.is_empty() {
                                object_carrier.insert(obj.clone(), (receiver.clone(), true));
                                if !is_person && obj == entity {
                                    if let Some(loc) = person_locations.get(&receiver) {
                                        history.push((order, loc.clone()));
                                    }
                                }
                            }
                        }
                        break;
                    }
                }
            }

            global_order += 1;
        }

        if history.len() < 2 {
            return None;
        }

        // Sort by time order
        history.sort_by_key(|(t, _)| *t);

        // Find the location immediately before target_location
        let mut prev_location: Option<String> = None;
        for (_, loc) in &history {
            if loc == &target_location || loc.contains(&target_location) || target_location.contains(loc.as_str()) {
                // Found when entity arrived at target — prev_location is the answer
                if let Some(answer) = &prev_location {
                    for (idx, choice) in choices.iter().enumerate() {
                        let cl = choice.to_lowercase();
                        if cl == *answer || cl.contains(answer.as_str()) || answer.contains(cl.as_str()) {
                            return Some((idx, choice.clone(), 0.80));
                        }
                    }
                    return None;
                }
            }
            prev_location = Some(loc.clone());
        }
        None
    }

    fn try_location_yesno_answer(
        &self,
        query_line: &str,
        choices: &[String],
    ) -> Option<(usize, String, f32)> {
        // Only fire on yes/no questions about location ("Is X in Y?")
        let has_yes = choices.iter().any(|c| c.to_lowercase() == "yes");
        let has_no  = choices.iter().any(|c| c.to_lowercase() == "no");
        if !has_yes || !has_no {
            return None;
        }
        // Query must start with "is " (already lowercased)
        let q = query_line.trim();
        if !q.starts_with("is ") {
            return None;
        }
        if !q.contains(" in the ") && !q.contains(" in ") {
            return None;
        }
        // Extract entity (first word after "is")
        let entity = q.split_whitespace().nth(1).unwrap_or("").to_string();
        if entity.is_empty() {
            return None;
        }
        // Extract queried location (after "in the " or "in ")
        let queried_location = if let Some(pos) = q.find(" in the ") {
            q[pos + 9..].trim_end_matches('?').trim().to_string()
        } else if let Some(pos) = q.find(" in ") {
            q[pos + 4..].trim_end_matches('?').trim().to_string()
        } else {
            return None;
        };
        if queried_location.is_empty() {
            return None;
        }
        // Only fire when we know the entity's location from context
        let known_location = self.flux.get_entity_location(&entity)?;
        // Handle negation sentinel: "!hallway" means entity is explicitly NOT there
        let answer_text = if let Some(neg_loc) = known_location.strip_prefix('!') {
            // Entity is explicitly NOT at neg_loc
            if neg_loc == queried_location.trim()
                || neg_loc.contains(&queried_location)
                || queried_location.contains(neg_loc)
            {
                "no"
            } else {
                // Negated for a different location — we don't know where it actually is
                return None;
            }
        } else if known_location.trim() == queried_location.trim()
            || known_location.contains(&queried_location)
            || queried_location.contains(&known_location)
        {
            "yes"
        } else {
            "no"
        };
        for (idx, choice) in choices.iter().enumerate() {
            if choice.to_lowercase() == answer_text {
                return Some((idx, choice.clone(), 0.80));
            }
        }
        None
    }

    #[allow(dead_code)]
    fn try_location_yesno_answer_v1(
        &self,
        question_lower: &str,
        choices: &[String],
    ) -> Option<(usize, String, f32)> {
        // Only fire on yes/no(/maybe) questions about location
        let has_yes = choices.iter().any(|c| c.to_lowercase() == "yes");
        let has_no  = choices.iter().any(|c| c.to_lowercase() == "no");
        if !has_yes || !has_no {
            return None;
        }
        let has_maybe = choices.iter().any(|c| c.to_lowercase() == "maybe");

        // Parse "is X in Y?" — extract entity and location from the last line
        let last_line = question_lower.lines().last().unwrap_or("").trim();
        if !last_line.starts_with("is ") {
            return None;
        }

        let is_in = last_line.contains(" in the ") || last_line.contains(" in ");
        if !is_in {
            return None;
        }

        // Extract entity (first word after "is")
        let entity = last_line
            .split_whitespace()
            .nth(1)
            .unwrap_or("")
            .to_string();

        // Extract location (after "in the" or "in")
        let location = if let Some(pos) = last_line.find(" in the ") {
            last_line[pos + 9..]
                .trim_end_matches('?')
                .trim()
                .to_string()
        } else if let Some(pos) = last_line.find(" in ") {
            last_line[pos + 4..]
                .trim_end_matches('?')
                .trim()
                .to_string()
        } else {
            return None;
        };

        if entity.is_empty() || location.is_empty() {
            return None;
        }

        // Detect indefinite knowledge: "X is either in A or B" anywhere in context
        // This is task 10 — if the most recent statement about entity is "either...or",
        // the correct answer is "maybe" (not yes/no)
        let indefinite = {
            // Find the LAST line about this entity in the context
            let mut last_either = false;
            let mut last_movement = false;
            for line in question_lower.lines() {
                let ltrim = line.trim_start_matches(|c: char| c.is_ascii_digit() || c == ' ');
                if ltrim.starts_with(&entity) {
                    if ltrim.contains("either in") || ltrim.contains("either in the") {
                        last_either = true;
                        last_movement = false;
                    } else if ltrim.contains("went") || ltrim.contains("moved")
                        || ltrim.contains("travelled") || ltrim.contains("traveled")
                        || ltrim.contains("journeyed") || ltrim.contains("is in")
                        || ltrim.contains("went back") || ltrim.contains("is no longer")
                    {
                        last_either = false;
                        last_movement = true;
                    }
                }
            }
            last_either && !last_movement
        };

        if indefinite {
            // Indefinite knowledge → answer is "maybe" if available, else abstain
            if has_maybe {
                for (idx, choice) in choices.iter().enumerate() {
                    if choice.to_lowercase() == "maybe" {
                        return Some((idx, choice.clone(), 0.75));
                    }
                }
            }
            return None; // No "maybe" choice — abstain and let other experts handle
        }

        // Look up entity's current location in the locations map
        let entity_location = self.flux.get_entity_location(&entity);

        let (answer_text, conf) = match entity_location {
            Some(loc) if loc == location => ("yes", 0.85f32),
            Some(_loc) => ("no", 0.85f32), // entity is definitively somewhere else
            None => {
                // Check explicit negation in context: "X is no longer in Y" / "X is not in Y"
                let negation = question_lower.contains(&format!("{} is no longer in {}", entity, location))
                    || question_lower.contains(&format!("{} is not in {}", entity, location));
                if negation {
                    ("no", 0.75f32)
                } else {
                    return None; // Location unknown — let other experts handle
                }
            }
        };

        for (idx, choice) in choices.iter().enumerate() {
            if choice.to_lowercase() == answer_text {
                return Some((idx, choice.clone(), conf));
            }
        }
        None
    }

    // -------------------------------------------------------------------------
    // Position 5 helper A: path finding + spatial yes/no
    // -------------------------------------------------------------------------

    fn try_path_or_spatial_answer(
        &self,
        question_lower: &str,
        choices: &[String],
    ) -> Option<(usize, String, f32)> {
        // Path finding: "how do you go from X to Y?"
        if question_lower.contains("how do you go from") || question_lower.contains("how to get from") {
            if let Some((path_ans, conf)) = self.flux.answer_path_question(question_lower) {
                for (idx, choice) in choices.iter().enumerate() {
                    let cl = choice.to_lowercase();
                    if cl == path_ans || cl.contains(&path_ans) || path_ans.contains(&cl) {
                        return Some((idx, choice.clone(), conf));
                    }
                }
            }
        }

        // Spatial/size yes/no: "is X left of Y?", "is X bigger than Y?"
        let is_spatial = question_lower.contains("left of") || question_lower.contains("right of")
            || question_lower.contains("above") || question_lower.contains("below");
        let is_size = question_lower.contains("bigger than") || question_lower.contains("smaller than")
            || question_lower.contains("fits inside") || question_lower.contains("fits in");

        if is_spatial || is_size {
            // --- Primary: 2D coordinate system (handles cross-relation chains) ---
            // Parse source, relation, target from query_line using score_yes_no's pattern table
            let spatial_patterns = [
                ("to the left of", "left_of"),
                ("left of", "left_of"),
                ("to the right of", "right_of"),
                ("right of", "right_of"),
                ("above", "above"),
                ("below", "below"),
            ];
            let size_patterns = [
                ("bigger than", "bigger_than"),
                ("larger than", "bigger_than"),
                ("smaller than", "smaller_than"),
                ("fits inside", "fits_inside"),
                ("fits in", "fits_inside"),
                ("fit inside", "fits_inside"),
                ("fit in", "fits_inside"),
            ];

            let mut parsed_source = String::new();
            let mut parsed_rel = String::new();
            let mut parsed_target = String::new();

            for (pattern, rel_type) in spatial_patterns.iter().chain(size_patterns.iter()) {
                if question_lower.contains(pattern) {
                    parsed_rel = rel_type.to_string();
                    // Source: after "does the", "does", "is the", or "is" up to the pattern
                    let subject_markers = ["does the ", "does ", "is the ", "is "];
                    'src_search: for marker in &subject_markers {
                        if let Some(pos) = question_lower.find(marker) {
                            let after = &question_lower[pos + marker.len()..];
                            if let Some(pp) = after.find(pattern) {
                                parsed_source = after[..pp].trim().to_string();
                                break 'src_search;
                            }
                        }
                    }
                    // Target: after pattern, trim "the " and "?"
                    if let Some(pp) = question_lower.find(pattern) {
                        let after = &question_lower[pp + pattern.len()..];
                        parsed_target = after.trim()
                            .trim_end_matches('?')
                            .trim()
                            .trim_start_matches("the ")
                            .to_string();
                    }
                    break;
                }
            }

            // Try coordinate-based answer first (handles cross-relation 2-hop chains)
            if !parsed_source.is_empty() && !parsed_target.is_empty() && !parsed_rel.is_empty() {
                // Spatial: 2D coordinate system
                if let Some((holds, coord_conf)) = self.flux.query_spatial_coords(
                    &parsed_source, &parsed_rel, &parsed_target,
                ) {
                    let answer_text = if holds { "yes" } else { "no" };
                    for (idx, choice) in choices.iter().enumerate() {
                        if choice.to_lowercase() == answer_text {
                            return Some((idx, choice.clone(), coord_conf));
                        }
                    }
                }
                // Size: rank-based ordering (handles bigger_than/smaller_than/fits_inside chains)
                if let Some((holds, rank_conf)) = self.flux.query_size_rank(
                    &parsed_source, &parsed_rel, &parsed_target,
                ) {
                    let answer_text = if holds { "yes" } else { "no" };
                    for (idx, choice) in choices.iter().enumerate() {
                        if choice.to_lowercase() == answer_text {
                            return Some((idx, choice.clone(), rank_conf));
                        }
                    }
                }
            }

            // --- Fallback: transitive graph scoring (same-relation direct/inverse) ---
            // Only use when coordinate system didn't have enough entities assigned
            let yes_score = self.flux.score_yes_no(question_lower, question_lower, "yes");
            let no_score = self.flux.score_yes_no(question_lower, question_lower, "no");

            let (answer_text, score_diff) = if yes_score > no_score {
                ("yes", yes_score - no_score)
            } else {
                ("no", no_score - yes_score)
            };

            // Threshold of 25.0 filters out the uncertain "has both entities but no path" case
            // (score_diff = 20.0, confidence = 0.5) — only commit on strong direct/inverse evidence.
            if score_diff > 25.0 {
                let conf = (score_diff / 40.0).clamp(0.4, 1.0);
                for (idx, choice) in choices.iter().enumerate() {
                    if choice.to_lowercase() == answer_text {
                        return Some((idx, choice.clone(), conf));
                    }
                }
            }
        }

        None
    }

    // -------------------------------------------------------------------------
    // Position 5 helper B: directional neighbor query (tasks 4, 17)
    // "What is north/east/west/south of X?" or "What is X north/east/west/south of?"
    // -------------------------------------------------------------------------

    fn try_directional_neighbor_answer(
        &self,
        question_lower: &str,
        choices: &[String],
    ) -> Option<(usize, String, f32)> {
        let directions = [("north of", "north_of"), ("south of", "south_of"),
                         ("east of", "east_of"), ("west of", "west_of")];

        let last_line = question_lower.lines().last().unwrap_or("").trim();

        // Pattern A: "What is [direction] X?" — find neighbors going direction from X
        // e.g., "What is north of the garden?" — find node north_of garden in adjacency
        for (dir_text, rel_type) in &directions {
            if last_line.contains(dir_text) {
                if let Some(pos) = last_line.find(dir_text) {
                    let target = last_line[pos + dir_text.len()..]
                        .trim()
                        .trim_end_matches('?')
                        .trim_start_matches("the ")
                        .trim()
                        .to_string();
                    if target.is_empty() { continue; }

                    // Search adjacency: who has rel_type → target?
                    if let Some(neighbor) = self.flux.find_neighbor_by_relation(&target, rel_type) {
                        let conf = 0.80f32;
                        for (idx, choice) in choices.iter().enumerate() {
                            let cl = choice.to_lowercase();
                            if cl == neighbor || cl.contains(&neighbor) || neighbor.contains(&cl) {
                                return Some((idx, choice.clone(), conf));
                            }
                        }
                    }
                }
            }
        }

        // Pattern B: "What is X [direction] of?" — find what X points toward
        // e.g., "What is the bathroom east of?" — find node such that bathroom east_of node
        for (dir_text, rel_type) in &directions {
            let of_pattern = format!("{} of?", dir_text.trim_end_matches(" of"));
            if last_line.contains(&of_pattern) || (last_line.contains(dir_text) && last_line.ends_with("of?")) {
                // Extract entity before the direction
                let entity = if let Some(pos) = last_line.find(" is ") {
                    let after = &last_line[pos + 4..];
                    after.split_whitespace()
                        .take_while(|w| !directions.iter().any(|(d, _)| d.starts_with(w)))
                        .collect::<Vec<_>>()
                        .join(" ")
                        .trim_start_matches("the ")
                        .to_string()
                } else { String::new() };

                if entity.is_empty() { continue; }

                // Find the inverse: who is the target that entity has rel_type over?
                let inverse_rel = match *rel_type {
                    "north_of" => "south_of",
                    "south_of" => "north_of",
                    "east_of"  => "west_of",
                    "west_of"  => "east_of",
                    _          => continue,
                };
                if let Some(neighbor) = self.flux.find_neighbor_by_relation(&entity, inverse_rel) {
                    let conf = 0.80f32;
                    for (idx, choice) in choices.iter().enumerate() {
                        let cl = choice.to_lowercase();
                        if cl == neighbor || cl.contains(&neighbor) || neighbor.contains(&cl) {
                            return Some((idx, choice.clone(), conf));
                        }
                    }
                }
            }
        }

        None
    }

    // -------------------------------------------------------------------------
    // Position 8 helper: pick best from all candidate pools
    // -------------------------------------------------------------------------

    fn assemble_best_answer(
        &self,
        question_lower: &str,
        choices: &[String],
        candidates: &[Option<&(usize, String, f32)>],
    ) -> Option<(usize, String, f32)> {
        // Take the highest-confidence candidate from the input pools
        let best = candidates.iter()
            .filter_map(|c| *c)
            .max_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

        if let Some((idx, ans, conf)) = best {
            return Some((*idx, ans.clone(), *conf));
        }

        // Fallback: score all choices only when we have relations in the graph
        // (no-relations → no evidence → abstain, return None)
        if self.flux.relations_is_empty() && self.flux.adjacency.is_empty() {
            return None;
        }

        let mut best_idx = 0;
        let mut best_score = f32::NEG_INFINITY;
        for (idx, choice) in choices.iter().enumerate() {
            let score = self.flux.score_yes_no(question_lower, question_lower, &choice.to_lowercase());
            if score > best_score {
                best_score = score;
                best_idx = idx;
            }
        }
        // Only commit when score reflects actual direct/inverse evidence.
        // score_yes_no returns 40.0 for confident direct match, 20.0 for inverse/indirect.
        // The "has_source && has_target but no path" uncertain fallback in query_relation
        // returns confidence=0.5 → no_score=20.0 — this is unreliable, do NOT commit.
        // Only commit when yes_score or no_score reflects a clear winner (diff > 15).
        let yes_score = self.flux.score_yes_no(question_lower, question_lower, "yes");
        let no_score  = self.flux.score_yes_no(question_lower, question_lower, "no");
        let score_diff = (yes_score - no_score).abs();
        if score_diff > 15.0 && best_score > 15.0 {
            let conf = (best_score / 40.0).clamp(0.3, 0.9);
            Some((best_idx, choices.get(best_idx).cloned().unwrap_or_default(), conf))
        } else {
            None
        }
    }

    // -------------------------------------------------------------------------
    // Category detection — maps question text to task type
    // -------------------------------------------------------------------------

    fn detect_category(&self, question_lower: &str) -> String {
        if question_lower.contains("how do you go from") || question_lower.contains("how to get from") {
            "path".to_string()
        } else if question_lower.contains("how many") {
            "counting".to_string()
        } else if question_lower.contains("left of") || question_lower.contains("right of")
            || question_lower.contains("above") || question_lower.contains("below") {
            "spatial".to_string()
        } else if question_lower.contains("bigger than") || question_lower.contains("smaller than")
            || question_lower.contains("fits inside") || question_lower.contains("fits in") {
            "size".to_string()
        } else if question_lower.contains("where is") || question_lower.contains("what is")
            || question_lower.contains("who is") || question_lower.contains("carrying")
            || question_lower.contains("holding") {
            "possession_location".to_string()
        } else if question_lower.contains("yes") || question_lower.contains("no") {
            "yesno".to_string()
        } else {
            "general".to_string()
        }
    }

    // -------------------------------------------------------------------------
    // Runtime confidence threshold — adapts based on observed accuracy
    // -------------------------------------------------------------------------

    fn runtime_threshold(&self, category: &str) -> f32 {
        if let Some((correct, total)) = self.calibration.get(category) {
            if *total >= 5 {
                let accuracy = *correct as f32 / *total as f32;
                // If accuracy is high, lower the threshold to catch more
                // If accuracy is low, raise it to avoid false positives
                return (0.85 - accuracy * 0.4).clamp(0.35, 0.80);
            }
        }
        // Default thresholds by category
        match category {
            "path"               => 0.45, // path finding is very precise
            "counting"           => 0.50,
            "spatial"            => 0.50,
            "size"               => 0.50,
            "possession_location"=> 0.55,
            "yesno"              => 0.55,
            _                    => 0.60,
        }
    }
}

impl Default for VortexSymbolicPlanner {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// 5. Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_planning_babi19() {
        let mut planner = VortexSymbolicPlanner::new();
        let context = "The garden is west of the bathroom.\nThe bedroom is north of the hallway.\nThe office is south of the hallway.\nThe bathroom is north of the bedroom.\nThe kitchen is east of the bedroom.\nHow do you go from the bathroom to the hallway?";
        let choices = vec!["s,s".to_string(), "n,n".to_string(), "e,w".to_string(), "w,e".to_string(), "s,n".to_string()];
        let result = planner.plan_babi_question(context, &choices);
        assert!(result.is_some(), "Should produce a plan for path question");
        let (idx, conf, trace) = result.unwrap();
        assert_eq!(choices[idx], "s,s", "Expected s,s path but got {}", choices[idx]);
        assert!(conf > 0.4, "Confidence should be > 0.4");
        assert_eq!(trace.task_category, "path");
        assert_eq!(trace.gates.len(), 3, "Should have 3 sacred gates");
    }

    #[test]
    fn test_spatial_yesno_babi17_above() {
        let mut planner = VortexSymbolicPlanner::new();
        // Context: red_square below blue_square
        // Query: is blue_square above red_square? → yes
        let context = "The red square is below the blue square.\nIs the blue square above the red square?";
        let choices = vec!["yes".to_string(), "no".to_string(), "maybe".to_string(), "unknown".to_string()];
        let result = planner.plan_babi_question(context, &choices);
        assert!(result.is_some(), "Planner should fire for spatial yes/no");
        let (idx, conf, _trace) = result.unwrap();
        assert_eq!(choices[idx], "yes", "Expected yes (blue above red) but got '{}'", choices[idx]);
        assert!(conf > 0.4, "Confidence should be > 0.4, got {}", conf);
    }

    #[test]
    fn test_spatial_yesno_babi17_transitive() {
        let mut planner = VortexSymbolicPlanner::new();
        // Context: pink left of triangle, triangle left of red → pink left of red (transitive)
        // Query: is pink to the right of red? → no (pink is LEFT of red, not right)
        let context = "The pink rectangle is to the left of the triangle.\nThe triangle is to the left of the red square.\nIs the pink rectangle to the right of the red square?";
        let choices = vec!["no".to_string(), "yes".to_string(), "maybe".to_string(), "unknown".to_string()];
        let result = planner.plan_babi_question(context, &choices);
        assert!(result.is_some(), "Planner should fire for transitive spatial");
        let (idx, conf, _trace) = result.unwrap();
        assert_eq!(choices[idx], "no", "Expected no (pink left of red, not right) but got '{}'", choices[idx]);
        assert!(conf > 0.4, "Confidence should be > 0.4, got {}", conf);
    }

    #[test]
    fn test_counting_babi7() {
        let mut planner = VortexSymbolicPlanner::new();
        let context = "Daniel picked up the apple. Daniel picked up the football. Daniel dropped the apple.\nHow many objects is Daniel carrying?";
        let choices = vec!["one".to_string(), "two".to_string(), "three".to_string(), "zero".to_string(), "four".to_string()];
        let result = planner.plan_babi_question(context, &choices);
        assert!(result.is_some(), "Should produce a plan for counting question");
        let (idx, _conf, trace) = result.unwrap();
        assert_eq!(choices[idx], "one", "Daniel has 1 item but got {}", choices[idx]);
        assert_eq!(trace.task_category, "counting");
    }

    #[test]
    fn test_spatial_yes_no_babi17() {
        let mut planner = VortexSymbolicPlanner::new();
        let context = "The pink rectangle is to the left of the triangle. The triangle is to the left of the red square.\nIs the pink rectangle to the left of the red square?";
        let choices = vec!["yes".to_string(), "no".to_string()];
        let result = planner.plan_babi_question(context, &choices);
        assert!(result.is_some(), "Should handle spatial yes/no");
        let (idx, conf, trace) = result.unwrap();
        assert_eq!(choices[idx], "yes", "Pink rect IS left of red square");
        assert!(conf > 0.4);
        assert_eq!(trace.task_category, "spatial");
    }

    #[test]
    fn test_sacred_gates_always_3() {
        let mut planner = VortexSymbolicPlanner::new();
        let context = "The garden is west of the bathroom.\nHow do you go from the bathroom to the garden?";
        let choices = vec!["w".to_string(), "e".to_string(), "n".to_string(), "s".to_string()];
        if let Some((_, _, trace)) = planner.plan_babi_question(context, &choices) {
            assert_eq!(trace.gates.len(), 3, "Always exactly 3 sacred gates (3,6,9)");
            assert_eq!(trace.gates[0].position, 3);
            assert_eq!(trace.gates[1].position, 6);
            assert_eq!(trace.gates[2].position, 9);
        }
    }

    #[test]
    fn test_calibration_adapts() {
        let mut planner = VortexSymbolicPlanner::new();
        // Simulate 10 correct answers → threshold should lower
        for _ in 0..10 {
            planner.update_calibration("path", true, 0.8);
        }
        let threshold = planner.runtime_threshold("path");
        assert!(threshold < 0.55, "High accuracy should lower threshold, got {}", threshold);
    }

    #[test]
    fn test_no_relational_context_returns_none() {
        let mut planner = VortexSymbolicPlanner::new();
        // A question with no extractable relations — gate 3 should fail
        let context = "What is the capital of France?";
        let choices = vec!["Paris".to_string(), "London".to_string(), "Berlin".to_string()];
        let result = planner.plan_babi_question(context, &choices);
        // With no adjacency, gate 3 fails and planner returns None — falls through to other experts
        assert!(result.is_none(), "Should return None when no relational structure found");
    }
}
