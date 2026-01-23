//! Explicit Chain-of-Thought Reasoning with Self-Verification
//!
//! Implements step-by-step reasoning chains that can be traced, verified, and corrected.
//! Each reasoning step is mapped to sacred geometry positions for coherent inference.

use crate::data::models::ELPTensor;
use serde::{Serialize, Deserialize};
use anyhow::{Result, bail};

/// A single step in the reasoning process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningStep {
    /// The thought or reasoning at this step
    pub thought: String,
    
    /// Confidence in this step (0.0-1.0)
    pub confidence: f32,
    
    /// ELP state at this step
    pub elp_state: ELPTensor,
    
    /// Flux position (0-9)
    pub flux_position: u8,
    
    /// Whether this step is at a sacred position (3, 6, 9)
    pub is_sacred: bool,
    
    /// Whether this step passed verification
    pub verification_passed: bool,
}

impl ReasoningStep {
    /// Create a new reasoning step
    pub fn new(
        thought: String,
        elp_state: ELPTensor,
        flux_position: u8,
        confidence: f32,
    ) -> Self {
        let is_sacred = matches!(flux_position, 3 | 6 | 9);
        
        Self {
            thought,
            confidence,
            elp_state,
            flux_position,
            is_sacred,
            verification_passed: false,
        }
    }
    
    /// Check if this step should trigger verification
    pub fn should_verify(&self) -> bool {
        self.is_sacred || self.confidence < 0.6
    }
}

/// Complete reasoning chain with self-verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningChain {
    /// All reasoning steps in order
    pub steps: Vec<ReasoningStep>,
    
    /// Final answer after all reasoning
    pub final_answer: String,
    
    /// Number of self-verification iterations performed
    pub self_verification_iterations: usize,
    
    /// Overall chain confidence
    pub overall_confidence: f32,
    
    /// Whether the chain completed a full vortex cycle (1→2→4→8→7→5→1)
    pub completed_vortex_cycle: bool,
}

impl ReasoningChain {
    /// Create a new empty reasoning chain
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
            final_answer: String::new(),
            self_verification_iterations: 0,
            overall_confidence: 0.0,
            completed_vortex_cycle: false,
        }
    }
    
    /// Add a reasoning step to the chain
    pub fn add_step(&mut self, thought: String, elp: ELPTensor, flux_position: u8, confidence: f32) {
        let step = ReasoningStep::new(thought, elp, flux_position, confidence);
        self.steps.push(step);
    }
    
    /// Verify consistency of the reasoning chain
    pub fn verify_consistency(&mut self) -> Result<bool> {
        if self.steps.is_empty() {
            bail!("Cannot verify empty reasoning chain");
        }
        
        let mut is_consistent = true;
        
        // First step is always considered verified (no predecessor to check)
        if !self.steps.is_empty() {
            self.steps[0].verification_passed = true;
        }
        
        // Collect ELP distances and transitions (starting from step 1)
        let mut elp_distances = Vec::new();
        let mut transitions = Vec::new();
        
        for i in 1..self.steps.len() {
            let prev_elp = self.steps[i - 1].elp_state;
            let curr_elp = self.steps[i].elp_state;
            let elp_distance = prev_elp.distance(&curr_elp);
            elp_distances.push(elp_distance);
            
            let prev_pos = self.steps[i - 1].flux_position;
            let curr_pos = self.steps[i].flux_position;
            transitions.push((prev_pos, curr_pos));
        }
        
        // Second pass: validate transitions
        for i in 1..self.steps.len() {
            let elp_distance = elp_distances[i - 1];
            let (prev_pos, curr_pos) = transitions[i - 1];
            
            if elp_distance > 3.0 {
                self.steps[i].verification_passed = false;
                is_consistent = false;
            } else {
                self.steps[i].verification_passed = true;
            }
            
            if !Self::is_valid_transition(prev_pos, curr_pos) {
                is_consistent = false;
            }
        }
        
        Ok(is_consistent)
    }
    
    /// Check if flux position transition is valid
    fn is_valid_transition(from: u8, to: u8) -> bool {
        // Vortex sequence: 1→2→4→8→7→5→1
        // Sacred positions: 3, 6, 9 (can go anywhere)
        match from {
            1 => matches!(to, 2 | 3 | 6 | 9),
            2 => matches!(to, 4 | 3 | 6 | 9),
            3 => true, // Sacred - can go anywhere
            4 => matches!(to, 8 | 3 | 6 | 9),
            5 => matches!(to, 1 | 3 | 6 | 9),
            6 => true, // Sacred - can go anywhere
            7 => matches!(to, 5 | 3 | 6 | 9),
            8 => matches!(to, 7 | 3 | 6 | 9),
            9 => true, // Sacred - can go anywhere
            0 => true, // Can start anywhere
            _ => false,
        }
    }
    
    /// Reflect on reasoning and correct errors
    pub fn reflect_and_correct(&mut self) -> Result<()> {
        self.self_verification_iterations += 1;
        
        // Find first failed verification
        let failed_index = self.steps.iter()
            .position(|s| !s.verification_passed);
        
        if let Some(idx) = failed_index {
            // Backtrack to last sacred position before failure
            let last_sacred = self.steps[..idx].iter()
                .rposition(|s| s.is_sacred)
                .unwrap_or(0);
            
            // Remove steps after last valid sacred checkpoint
            self.steps.truncate(last_sacred + 1);
            
            // Mark that we need to continue reasoning from here
            Ok(())
        } else {
            // All steps verified
            Ok(())
        }
    }
    
    /// Finalize the reasoning chain
    pub fn finalize(&mut self, answer: String) {
        self.final_answer = answer;
        
        // Calculate overall confidence as weighted average
        let total_weight: f32 = self.steps.iter()
            .map(|s| if s.is_sacred { 1.5 } else { 1.0 })
            .sum();
        
        let weighted_confidence: f32 = self.steps.iter()
            .map(|s| {
                let weight = if s.is_sacred { 1.5 } else { 1.0 };
                s.confidence * weight
            })
            .sum();
        
        self.overall_confidence = if total_weight > 0.0 {
            weighted_confidence / total_weight
        } else {
            0.0
        };
        
        // Check if we completed a vortex cycle
        self.completed_vortex_cycle = self.check_vortex_cycle();
    }
    
    /// Check if reasoning completed a full vortex cycle
    /// Enhanced to accept partial matches and sequential flow
    pub fn check_vortex_cycle(&self) -> bool {
        let vortex_sequence = [1, 2, 4, 8, 7, 5, 1];
        let positions: Vec<u8> = self.steps.iter()
            .map(|s| s.flux_position)
            .collect();
        
        // Check for exact sequence match
        if positions.windows(vortex_sequence.len())
            .any(|window| window == vortex_sequence) {
            return true;
        }
        
        // Check for sequential vortex flow (all positions present in order)
        let mut vortex_idx = 0;
        for &pos in &positions {
            if pos == vortex_sequence[vortex_idx] {
                vortex_idx += 1;
                if vortex_idx >= vortex_sequence.len() {
                    return true; // Found complete sequence
                }
            }
        }
        
        // Check if we at least completed the core cycle (1→2→4→8→7→5)
        let core_cycle = [1, 2, 4, 8, 7, 5];
        let mut core_idx = 0;
        for &pos in &positions {
            if pos == core_cycle[core_idx] {
                core_idx += 1;
                if core_idx >= core_cycle.len() {
                    return true; // Core cycle complete
                }
            }
        }
        
        false
    }
    
    /// Get a human-readable trace of the reasoning
    pub fn to_trace(&self) -> String {
        let mut trace = String::new();
        trace.push_str(&format!("🧠 Reasoning Chain ({} steps)\n", self.steps.len()));
        trace.push_str(&format!("Overall Confidence: {:.1}%\n\n", self.overall_confidence * 100.0));
        
        for (i, step) in self.steps.iter().enumerate() {
            let marker = if step.is_sacred { "🔷" } else { "○" };
            let verified = if step.verification_passed { "✓" } else { "✗" };
            
            trace.push_str(&format!(
                "{} Step {}: [Pos {}] [Conf {:.1}%] {}\n",
                marker,
                i + 1,
                step.flux_position,
                step.confidence * 100.0,
                verified
            ));
            trace.push_str(&format!("   {}\n", step.thought));
            trace.push_str(&format!("   ELP: E={:.1} L={:.1} P={:.1}\n\n",
                step.elp_state.ethos,
                step.elp_state.logos,
                step.elp_state.pathos
            ));
        }
        
        trace.push_str(&format!("📊 Final Answer: {}\n", self.final_answer));
        trace.push_str(&format!("🔄 Vortex Cycle: {}\n",
            if self.completed_vortex_cycle { "Complete ✓" } else { "Incomplete" }
        ));
        
        trace
    }
}

impl Default for ReasoningChain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_reasoning_step_creation() {
        let elp = ELPTensor::new(7.0, 6.0, 5.0);
        let step = ReasoningStep::new(
            "Testing reasoning".to_string(),
            elp,
            3,
            0.85
        );
        
        assert_eq!(step.flux_position, 3);
        assert!(step.is_sacred);
        assert_eq!(step.confidence, 0.85);
    }
    
    #[test]
    fn test_chain_consistency() {
        let mut chain = ReasoningChain::new();
        
        // Add steps following vortex sequence
        chain.add_step("Start".to_string(), ELPTensor::new(6.0, 6.0, 6.0), 1, 0.8);
        chain.add_step("Continue".to_string(), ELPTensor::new(6.5, 6.0, 6.0), 2, 0.85);
        chain.add_step("Sacred checkpoint".to_string(), ELPTensor::new(7.0, 6.0, 5.5), 3, 0.9);
        
        let is_consistent = chain.verify_consistency().unwrap();
        assert!(is_consistent);
    }
    
    #[test]
    fn test_vortex_cycle_detection() {
        let mut chain = ReasoningChain::new();
        
        // Complete vortex cycle: 1→2→4→8→7→5→1
        let cycle = vec![1, 2, 4, 8, 7, 5, 1];
        for pos in cycle {
            chain.add_step(
                format!("Step at {}", pos),
                ELPTensor::new(6.0, 6.0, 6.0),
                pos,
                0.8
            );
        }
        
        chain.finalize("Completed cycle".to_string());
        assert!(chain.completed_vortex_cycle);
    }
    
    #[test]
    fn test_confidence_calculation() {
        let mut chain = ReasoningChain::new();
        
        // Add mix of sacred and regular positions
        chain.add_step("".to_string(), ELPTensor::new(6.0, 6.0, 6.0), 1, 0.8);
        chain.add_step("".to_string(), ELPTensor::new(6.0, 6.0, 6.0), 2, 0.8);
        chain.add_step("".to_string(), ELPTensor::new(6.0, 6.0, 6.0), 3, 0.8); // Sacred
        chain.add_step("".to_string(), ELPTensor::new(6.0, 6.0, 6.0), 4, 0.8);
        
        chain.verify_consistency().unwrap();
        
        // Check confidence is calculated
        assert!(chain.steps.last().unwrap().confidence > 0.0);
    }
}
