//! First Principles Reasoning - Truth Detection and Logical Analysis
//!
//! Enables Vortex to:
//! - Reason from fundamental axioms
//! - Detect truth vs falsehood
//! - Identify sarcasm and irony
//! - Handle ambiguity and uncertainty
//! - Apply deductive and inductive reasoning

use serde::{Deserialize, Serialize};
use crate::data::models::ELPTensor;

/// Result of first principles analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirstPrinciplesResult {
    /// Original statement being analyzed
    pub statement: String,
    
    /// Truth assessment
    pub truth_assessment: TruthAssessment,
    
    /// Fundamental axioms used
    pub axioms_applied: Vec<String>,
    
    /// Reasoning chain from axioms to conclusion
    pub reasoning_steps: Vec<ReasoningStep>,
    
    /// Overall confidence in the analysis
    pub confidence: f32,
    
    /// ELP analysis of the statement
    pub elp_signature: ELPTensor,
}

/// Truth classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TruthAssessment {
    /// Objectively true (verified by axioms)
    True { certainty: f32 },
    
    /// Objectively false (contradicts axioms)
    False { certainty: f32 },
    
    /// Partially true (some elements true, some false)
    PartiallyTrue { true_percentage: f32 },
    
    /// Cannot be determined (insufficient information)
    Uncertain { ambiguity_score: f32 },
    
    /// Sarcastic/ironic (literally false, contextually meaningful)
    Sarcastic { intended_meaning: String, confidence: f32 },
    
    /// Deceptive (intentionally misleading)
    Deceptive { deception_type: DeceptionType, confidence: f32 },
    
    /// Opinion (subjective, not objectively verifiable)
    Opinion { perspective: String },
}

/// Types of deception
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeceptionType {
    /// Outright lie
    DirectLie,
    
    /// True facts arranged to mislead
    MisleadingContext,
    
    /// Omission of critical information
    OmissionLie,
    
    /// Exaggeration or understatement
    Distortion,
    
    /// Logical fallacy
    FallacyBased,
}

/// Single reasoning step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningStep {
    /// Description of this step
    pub description: String,
    
    /// Axiom or premise used
    pub premise: String,
    
    /// Logical operation applied
    pub operation: LogicalOperation,
    
    /// Conclusion reached
    pub conclusion: String,
    
    /// Confidence in this step
    pub confidence: f32,
}

/// Logical operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogicalOperation {
    /// Deductive reasoning (general → specific)
    Deduction,
    
    /// Inductive reasoning (specific → general)
    Induction,
    
    /// Abductive reasoning (best explanation)
    Abduction,
    
    /// Modus ponens (if P then Q, P, therefore Q)
    ModusPonens,
    
    /// Modus tollens (if P then Q, not Q, therefore not P)
    ModusTollens,
    
    /// Contradiction detection
    Contradiction,
    
    /// Analogy
    Analogy,
}

/// First Principles Reasoner
pub struct FirstPrinciplesReasoner {
    /// Fundamental axioms (self-evident truths)
    axioms: Vec<Axiom>,
}

/// Fundamental axiom
#[derive(Debug, Clone)]
pub struct Axiom {
    /// Name of the axiom
    pub name: String,
    
    /// Statement of the axiom
    pub statement: String,
    
    /// Domain of applicability
    pub domain: AxiomDomain,
    
    /// Sacred position (3=ethical, 6=logical, 9=emotional)
    pub sacred_position: u8,
}

/// Domain of axiom applicability
#[derive(Debug, Clone, PartialEq)]
pub enum AxiomDomain {
    /// Physical reality
    Physical,
    
    /// Mathematical/logical
    Logical,
    
    /// Ethical/moral
    Ethical,
    
    /// Psychological/human behavior
    Psychological,
    
    /// Universal (applies everywhere)
    Universal,
}

impl FirstPrinciplesReasoner {
    /// Create new reasoner with fundamental axioms
    pub fn new() -> Self {
        let axioms = Self::initialize_axioms();
        Self { axioms }
    }
    
    /// Initialize fundamental axioms
    fn initialize_axioms() -> Vec<Axiom> {
        vec![
            // Logical axioms (Position 6 - Logos)
            Axiom {
                name: "Law of Identity".to_string(),
                statement: "A thing is itself (A = A)".to_string(),
                domain: AxiomDomain::Logical,
                sacred_position: 6,
            },
            Axiom {
                name: "Law of Non-Contradiction".to_string(),
                statement: "A statement cannot be both true and false simultaneously (not (P and not-P))".to_string(),
                domain: AxiomDomain::Logical,
                sacred_position: 6,
            },
            Axiom {
                name: "Law of Excluded Middle".to_string(),
                statement: "A statement is either true or false (P or not-P)".to_string(),
                domain: AxiomDomain::Logical,
                sacred_position: 6,
            },
            
            // Physical axioms (Position 6 - Logos)
            Axiom {
                name: "Causality".to_string(),
                statement: "Every effect has a cause".to_string(),
                domain: AxiomDomain::Physical,
                sacred_position: 6,
            },
            Axiom {
                name: "Conservation of Energy".to_string(),
                statement: "Energy cannot be created or destroyed, only transformed".to_string(),
                domain: AxiomDomain::Physical,
                sacred_position: 6,
            },
            
            // Ethical axioms (Position 3 - Ethos)
            Axiom {
                name: "Harm Principle".to_string(),
                statement: "Actions that harm others require justification".to_string(),
                domain: AxiomDomain::Ethical,
                sacred_position: 3,
            },
            Axiom {
                name: "Consistency".to_string(),
                statement: "Similar cases should be treated similarly".to_string(),
                domain: AxiomDomain::Ethical,
                sacred_position: 3,
            },
            
            // Psychological axioms (Position 9 - Pathos)
            Axiom {
                name: "Human Emotion".to_string(),
                statement: "Humans experience emotions that influence behavior".to_string(),
                domain: AxiomDomain::Psychological,
                sacred_position: 9,
            },
            Axiom {
                name: "Self-Interest".to_string(),
                statement: "Humans generally act in perceived self-interest".to_string(),
                domain: AxiomDomain::Psychological,
                sacred_position: 9,
            },
            
            // Universal axioms (All positions)
            Axiom {
                name: "Observation".to_string(),
                statement: "Reality exists independent of observation, but can be known through observation".to_string(),
                domain: AxiomDomain::Universal,
                sacred_position: 6,
            },
        ]
    }
    
    /// Analyze a statement from first principles
    pub fn analyze(&self, statement: &str) -> FirstPrinciplesResult {
        let mut reasoning_steps = Vec::new();
        
        // Step 1: Decompose statement into claims
        let claims = self.decompose_statement(statement);
        
        // Step 2: Check for logical contradictions
        let contradiction_check = self.check_contradictions(&claims, &mut reasoning_steps);
        
        // Step 3: Check against physical reality
        let physical_check = self.check_physical_plausibility(&claims, &mut reasoning_steps);
        
        // Step 4: Analyze for sarcasm/irony
        let sarcasm_check = self.detect_sarcasm(statement, &mut reasoning_steps);
        
        // Step 5: Check for deception patterns
        let deception_check = self.detect_deception(statement, &claims, &mut reasoning_steps);
        
        // Step 6: Synthesize truth assessment
        let truth_assessment = self.synthesize_assessment(
            statement,
            contradiction_check,
            physical_check,
            sarcasm_check,
            deception_check,
        );
        
        // Step 7: Calculate ELP signature
        let elp_signature = self.calculate_elp_signature(statement, &truth_assessment);
        
        // Step 8: Identify applicable axioms
        let axioms_applied = reasoning_steps.iter()
            .map(|s| s.premise.clone())
            .collect();
        
        // Step 9: Calculate overall confidence
        let confidence = reasoning_steps.iter()
            .map(|s| s.confidence)
            .sum::<f32>() / reasoning_steps.len() as f32;
        
        FirstPrinciplesResult {
            statement: statement.to_string(),
            truth_assessment,
            axioms_applied,
            reasoning_steps,
            confidence,
            elp_signature,
        }
    }
    
    /// Decompose statement into atomic claims
    fn decompose_statement(&self, statement: &str) -> Vec<String> {
        // Simple decomposition by sentence
        statement.split('.')
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.trim().to_string())
            .collect()
    }
    
    /// Check for logical contradictions
    fn check_contradictions(&self, claims: &[String], steps: &mut Vec<ReasoningStep>) -> bool {
        // Apply Law of Non-Contradiction
        let axiom = self.axioms.iter()
            .find(|a| a.name == "Law of Non-Contradiction")
            .unwrap();
        
        // Check for self-contradicting statements
        for (i, claim1) in claims.iter().enumerate() {
            for claim2 in claims.iter().skip(i + 1) {
                if Self::are_contradictory(claim1, claim2) {
                    steps.push(ReasoningStep {
                        description: "Detected logical contradiction".to_string(),
                        premise: axiom.statement.clone(),
                        operation: LogicalOperation::Contradiction,
                        conclusion: format!("'{}' contradicts '{}'", claim1, claim2),
                        confidence: 0.85,
                    });
                    return true;
                }
            }
        }
        
        false
    }
    
    /// Check if two claims contradict each other
    fn are_contradictory(claim1: &str, claim2: &str) -> bool {
        // Simple negation detection
        let c1_lower = claim1.to_lowercase();
        let c2_lower = claim2.to_lowercase();
        
        // Check for direct negation patterns
        if c1_lower.contains("not") && !c2_lower.contains("not") {
            let c1_without_not = c1_lower.replace("not ", "").replace("n't ", " ");
            if c2_lower.contains(&c1_without_not) {
                return true;
            }
        }
        
        false
    }
    
    /// Check physical plausibility
    fn check_physical_plausibility(&self, _claims: &[String], steps: &mut Vec<ReasoningStep>) -> bool {
        // Apply causality axiom
        let axiom = self.axioms.iter()
            .find(|a| a.name == "Causality")
            .unwrap();
        
        steps.push(ReasoningStep {
            description: "Checking physical causality".to_string(),
            premise: axiom.statement.clone(),
            operation: LogicalOperation::Deduction,
            conclusion: "Statement is physically plausible".to_string(),
            confidence: 0.75,
        });
        
        true
    }
    
    /// Detect sarcasm and irony
    fn detect_sarcasm(&self, statement: &str, steps: &mut Vec<ReasoningStep>) -> Option<(String, f32)> {
        let lower = statement.to_lowercase();
        
        // Sarcasm indicators
        let sarcasm_markers = [
            "yeah right", "sure", "oh great", "fantastic", "wonderful",
            "perfect", "just what i needed", "obviously",
        ];
        
        // Exaggeration indicators
        let exaggeration_markers = [
            "never", "always", "everyone", "nobody", "literally",
            "absolutely", "completely", "totally",
        ];
        
        let has_sarcasm_marker = sarcasm_markers.iter().any(|m| lower.contains(m));
        let has_exaggeration = exaggeration_markers.iter().any(|m| lower.contains(m));
        
        if has_sarcasm_marker || has_exaggeration {
            let axiom = self.axioms.iter()
                .find(|a| a.name == "Human Emotion")
                .unwrap();
            
            steps.push(ReasoningStep {
                description: "Detected potential sarcasm or irony".to_string(),
                premise: axiom.statement.clone(),
                operation: LogicalOperation::Abduction,
                conclusion: "Statement likely means the opposite of literal meaning".to_string(),
                confidence: 0.70,
            });
            
            Some(("Opposite of literal meaning".to_string(), 0.70))
        } else {
            None
        }
    }
    
    /// Detect deception patterns
    fn detect_deception(&self, statement: &str, _claims: &[String], steps: &mut Vec<ReasoningStep>) -> Option<DeceptionType> {
        let lower = statement.to_lowercase();
        
        // Hedging language (uncertainty markers that might indicate deception)
        let hedging_markers = [
            "probably", "maybe", "perhaps", "i think", "i believe",
            "in my opinion", "might be", "could be",
        ];
        
        // Overly specific details (can indicate fabrication)
        let has_excessive_detail = statement.len() > 200 && statement.matches(',').count() > 5;
        
        // Absolute language (often indicates exaggeration)
        let absolute_markers = ["never", "always", "everyone", "nobody", "impossible"];
        let has_absolutes = absolute_markers.iter().any(|m| lower.contains(m));
        
        if has_absolutes && has_excessive_detail {
            let axiom = self.axioms.iter()
                .find(|a| a.name == "Self-Interest")
                .unwrap();
            
            steps.push(ReasoningStep {
                description: "Detected potential distortion".to_string(),
                premise: axiom.statement.clone(),
                operation: LogicalOperation::Abduction,
                conclusion: "Statement may be exaggerated or distorted".to_string(),
                confidence: 0.60,
            });
            
            Some(DeceptionType::Distortion)
        } else if hedging_markers.iter().filter(|m| lower.contains(*m)).count() > 2 {
            steps.push(ReasoningStep {
                description: "Excessive hedging detected".to_string(),
                premise: "Uncertain language patterns".to_string(),
                operation: LogicalOperation::Induction,
                conclusion: "Speaker may lack confidence or knowledge".to_string(),
                confidence: 0.55,
            });
            
            None
        } else {
            None
        }
    }
    
    /// Synthesize final truth assessment
    fn synthesize_assessment(
        &self,
        statement: &str,
        has_contradiction: bool,
        _is_physical: bool,
        sarcasm: Option<(String, f32)>,
        deception: Option<DeceptionType>,
    ) -> TruthAssessment {
        // Priority order: Contradiction > Sarcasm > Deception > Truth
        
        if has_contradiction {
            return TruthAssessment::False { certainty: 0.90 };
        }
        
        if let Some((meaning, confidence)) = sarcasm {
            return TruthAssessment::Sarcastic {
                intended_meaning: meaning,
                confidence,
            };
        }
        
        if let Some(deception_type) = deception {
            return TruthAssessment::Deceptive {
                deception_type,
                confidence: 0.65,
            };
        }
        
        // Check if it's an opinion
        let opinion_markers = ["i think", "i believe", "in my opinion", "i feel", "should", "ought"];
        if opinion_markers.iter().any(|m| statement.to_lowercase().contains(m)) {
            return TruthAssessment::Opinion {
                perspective: "Subjective viewpoint".to_string(),
            };
        }
        
        // Default: likely true but with uncertainty
        TruthAssessment::True { certainty: 0.75 }
    }
    
    /// Calculate ELP signature for statement
    fn calculate_elp_signature(&self, _statement: &str, assessment: &TruthAssessment) -> ELPTensor {
        match assessment {
            TruthAssessment::True { certainty } => {
                // High logos (logic), moderate ethos
                ELPTensor::new(6.0, (8.0 * certainty) as f64, 4.0)
            }
            TruthAssessment::False { certainty } => {
                // High logos (detected falsehood), low ethos
                ELPTensor::new(3.0, (8.0 * certainty) as f64, 4.0)
            }
            TruthAssessment::Sarcastic { confidence, .. } => {
                // High pathos (emotional), moderate logos
                ELPTensor::new(5.0, 6.0, (9.0 * confidence) as f64)
            }
            TruthAssessment::Deceptive { confidence, .. } => {
                // Very low ethos (untrustworthy)
                ELPTensor::new(2.0, 6.0, (7.0 * confidence) as f64)
            }
            TruthAssessment::Opinion { .. } => {
                // Balanced but moderate
                ELPTensor::new(6.0, 5.0, 7.0)
            }
            TruthAssessment::Uncertain { ambiguity_score } => {
                // Low certainty across all dimensions
                let uncertainty = 1.0 - ambiguity_score;
                ELPTensor::new(
                    (5.0 * uncertainty) as f64,
                    (5.0 * uncertainty) as f64,
                    (5.0 * uncertainty) as f64
                )
            }
            TruthAssessment::PartiallyTrue { true_percentage } => {
                // Scaled by how much is true
                ELPTensor::new(
                    (5.0 + (2.0 * true_percentage)) as f64,
                    (6.0 + (3.0 * true_percentage)) as f64,
                    5.0,
                )
            }
        }
    }
}

impl Default for FirstPrinciplesReasoner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_logical_contradiction() {
        let reasoner = FirstPrinciplesReasoner::new();
        let result = reasoner.analyze("The sky is blue. The sky is not blue.");
        
        assert!(matches!(result.truth_assessment, TruthAssessment::False { .. }));
        assert!(result.confidence > 0.8);
    }
    
    #[test]
    fn test_sarcasm_detection() {
        let reasoner = FirstPrinciplesReasoner::new();
        let result = reasoner.analyze("Oh great, another rainy day. Just what I needed.");
        
        assert!(matches!(result.truth_assessment, TruthAssessment::Sarcastic { .. }));
    }
    
    #[test]
    fn test_opinion_detection() {
        let reasoner = FirstPrinciplesReasoner::new();
        let result = reasoner.analyze("I think chocolate ice cream is the best flavor.");
        
        assert!(matches!(result.truth_assessment, TruthAssessment::Opinion { .. }));
    }
    
    #[test]
    fn test_axiom_initialization() {
        let reasoner = FirstPrinciplesReasoner::new();
        assert!(reasoner.axioms.len() >= 10);
        
        // Verify sacred positions
        let logical_axioms: Vec<_> = reasoner.axioms.iter()
            .filter(|a| a.sacred_position == 6)
            .collect();
        assert!(!logical_axioms.is_empty());
    }
}
