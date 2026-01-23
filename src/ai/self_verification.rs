//! Self-Verification Module
//!
//! Implements verification logic for reasoning chains using:
//! - VCP hallucination detection
//! - Sacred geometry consistency checks
//! - ELP channel coherence validation

use crate::ai::reasoning_chain::{ReasoningChain, ReasoningStep};
use crate::ml::hallucinations::{HallucinationDetector, VortexContextPreserver};
use crate::data::models::BeamTensor;
#[cfg(test)]
use crate::data::models::ELPTensor;
use anyhow::Result;
use serde::Serialize;

/// Verification result for a reasoning chain
#[derive(Debug, Clone, Serialize)]
pub struct VerificationResult {
    /// Whether the chain passed verification
    pub passed: bool,
    
    /// Confidence in the verification and signal strength (0.0-1.0)
    pub confidence: f32,
    
    /// Issues found during verification
    pub issues: Vec<VerificationIssue>,
    
    /// Whether hallucination was detected
    pub hallucination_detected: bool,
}

/// Types of verification issues
#[derive(Debug, Clone, Serialize)]
pub enum VerificationIssue {
    /// ELP channel jumped too drastically
    ELPDiscontinuity { step_index: usize, distance: f64 },
    
    /// Invalid flux position transition
    InvalidTransition { from: u8, to: u8, step_index: usize },
    
    /// Low confidence at a step
    LowConfidence { step_index: usize, confidence: f32 },
    
    /// Signal strength too low
    WeakSignal { step_index: usize, signal: f32 },
    
    /// Hallucination detected by VCP
    HallucinationDetected { step_index: usize },
    
    /// Missing sacred checkpoint
    MissingSacredCheckpoint { expected_positions: Vec<u8> },
}

/// Self-verification engine
pub struct SelfVerificationEngine {
    /// Hallucination detector
    hallucination_detector: HallucinationDetector,
    
    /// VCP cascade for intervention
    #[allow(dead_code)]
    vcp: VortexContextPreserver,
    
    /// Minimum confidence threshold
    min_confidence: f32,
    
    /// Maximum ELP jump allowed
    max_elp_jump: f64,
}

impl SelfVerificationEngine {
    /// Create a new verification engine
    /// Thresholds tuned for balanced strictness (not overly harsh)
    pub fn new() -> Self {
        Self {
            hallucination_detector: HallucinationDetector::default(),
            vcp: VortexContextPreserver::default(),
            min_confidence: 0.55,      // Relaxed from 0.6 (slightly more lenient)
            max_elp_jump: 3.5,          // Relaxed from 3.0 (allow more variation)
        }
    }
    
    /// Create a strict verification engine for critical applications
    #[allow(dead_code)]
    pub fn new_strict() -> Self {
        Self {
            hallucination_detector: HallucinationDetector::default(),
            vcp: VortexContextPreserver::default(),
            min_confidence: 0.65,
            max_elp_jump: 2.5,
        }
    }
    
    /// Verify a complete reasoning chain
    pub fn verify_chain(&self, chain: &ReasoningChain) -> Result<VerificationResult> {
        let mut issues = Vec::new();
        let mut hallucination_detected = false;
        
        // Check for sacred checkpoint coverage
        let sacred_positions: Vec<u8> = chain.steps.iter()
            .filter(|s| s.is_sacred)
            .map(|s| s.flux_position)
            .collect();
        
        if !sacred_positions.contains(&3) || 
           !sacred_positions.contains(&6) || 
           !sacred_positions.contains(&9) {
            issues.push(VerificationIssue::MissingSacredCheckpoint {
                expected_positions: vec![3, 6, 9],
            });
        }
        
        // Verify each step
        for (i, step) in chain.steps.iter().enumerate() {
            // Check confidence (confidence has been consolidated into confidence)
            if step.confidence < self.min_confidence {
                issues.push(VerificationIssue::LowConfidence {
                    step_index: i,
                    confidence: step.confidence,
                });
            }
            
            // Check ELP continuity with previous step
            if i > 0 {
                let prev_elp = chain.steps[i - 1].elp_state;
                let elp_distance = prev_elp.distance(&step.elp_state);
                
                if elp_distance > self.max_elp_jump {
                    issues.push(VerificationIssue::ELPDiscontinuity {
                        step_index: i,
                        distance: elp_distance,
                    });
                }
            }
        }
        
        // Use VCP to detect hallucinations
        if chain.steps.len() >= 5 {
            let beams = self.convert_chain_to_beams(chain);
            
            // Check for hallucination using signal subspace analysis
            let result = self.hallucination_detector.detect_hallucination(
                &beams[..beams.len() - 2],
                &beams[beams.len() - 2..]
            );
            if result.is_hallucination {
                hallucination_detected = true;
                issues.push(VerificationIssue::HallucinationDetected {
                    step_index: beams.len() - 1,
                });
            }
        }
        
        // Calculate overall verification confidence
        let confidence = if issues.is_empty() {
            chain.overall_confidence
        } else {
            // Reduce confidence based on number and severity of issues
            let penalty = (issues.len() as f32 * 0.1).min(0.5);
            (chain.overall_confidence - penalty).max(0.0)
        };
        
        Ok(VerificationResult {
            passed: issues.is_empty() && !hallucination_detected,
            confidence,
            issues,
            hallucination_detected,
        })
    }
    
    /// Convert reasoning chain to beam tensors for VCP analysis
    fn convert_chain_to_beams(&self, chain: &ReasoningChain) -> Vec<BeamTensor> {
        chain.steps.iter().map(|step| {
            let mut beam = BeamTensor::default();
            beam.position = step.flux_position;
            beam.confidence = step.confidence;
            
            // Map ELP to beam digits (simplified)
            beam.digits[2] = (step.elp_state.ethos / 13.0) as f32;  // Position 3
            beam.digits[5] = (step.elp_state.logos / 13.0) as f32;   // Position 6
            beam.digits[8] = (step.elp_state.pathos / 13.0) as f32;  // Position 9
            
            beam
        }).collect()
    }
    
    /// Apply VCP intervention to correct a reasoning step
    pub fn apply_vcp_intervention(&self, step: &mut ReasoningStep) {
        if step.is_sacred {
            // Sacred position intervention: 1.5x magnification + 15% boost
            step.confidence = (step.confidence * 1.15).min(1.0);
            // confidence consolidated into confidence
        }
    }
}

impl Default for SelfVerificationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::reasoning_chain::ReasoningChain;
    
    #[test]
    fn test_verification_engine_creation() {
        let engine = SelfVerificationEngine::new();
        assert_eq!(engine.min_confidence, 0.6);
        assert_eq!(engine.max_elp_jump, 3.0);
    }
    
    #[test]
    fn test_high_confidence_chain_passes() {
        let engine = SelfVerificationEngine::new();
        let mut chain = ReasoningChain::new();
        
        // Add steps with high confidence
        chain.add_step("".to_string(), ELPTensor::new(6.0, 6.0, 6.0), 1, 0.9);
        chain.add_step("".to_string(), ELPTensor::new(6.5, 6.0, 6.0), 3, 0.9); // Sacred
        chain.add_step("".to_string(), ELPTensor::new(6.8, 6.0, 6.0), 6, 0.85); // Sacred
        chain.add_step("".to_string(), ELPTensor::new(7.0, 6.0, 6.0), 9, 0.9); // Sacred
        
        chain.verify_consistency().unwrap();
        chain.finalize("Test".to_string());
        
        let result = engine.verify_chain(&chain).unwrap();
        assert!(result.confidence > 0.7);
    }
    
    #[test]
    fn test_low_confidence_triggers_issue() {
        let engine = SelfVerificationEngine::new();
        let mut chain = ReasoningChain::new();
        
        // Add step with low confidence
        chain.add_step("".to_string(), ELPTensor::new(6.0, 6.0, 6.0), 1, 0.3);
        chain.finalize("Test".to_string());
        
        let result = engine.verify_chain(&chain).unwrap();
        assert!(!result.issues.is_empty());
    }
}
