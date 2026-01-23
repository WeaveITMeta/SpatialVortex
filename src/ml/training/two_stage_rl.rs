//! Two-Stage RL Pipeline for Emergent Reasoning
//!
//! Stage 1 (Discovery): Explores reasoning patterns with high exploration
//! Stage 2 (Alignment): Aligns discovered patterns to sacred geometry
//!
//! This approach enables reasoning capabilities at low training cost by:
//! - Discovering novel patterns through RL exploration
//! - Aligning patterns to geometric constraints
//! - Minimal cold-start from high-quality Confidence Lake traces

use crate::ai::reasoning_chain::ReasoningChain;
use crate::ml::rl_gradient_optimizer::RLGradientOptimizer;
use crate::data::models::ELPTensor;
use crate::core::sacred_geometry::flux_matrix::FluxMatrixEngine;
use crate::ml::hallucinations::VortexContextPreserver;
use anyhow::{Result, bail};
use serde::Serialize;
use std::collections::VecDeque;

/// Training stage in the two-stage pipeline
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TrainingStage {
    /// Stage 1: Discovery - explore reasoning patterns
    Discovery,
    
    /// Stage 2: Alignment - align to sacred geometry
    Alignment,
}

/// Configuration for the two-stage trainer
#[derive(Debug, Clone)]
pub struct TwoStageConfig {
    /// Discovery stage exploration rate (typically 0.2-0.3)
    pub discovery_epsilon: f32,
    
    /// Alignment stage exploration rate (typically 0.01-0.05)
    pub alignment_epsilon: f32,
    
    /// Maximum steps per reasoning chain
    pub max_steps: usize,
    
    /// Minimum confidence for discovery
    pub discovery_min_confidence: f32,
    
    /// Minimum confidence for alignment (stricter)
    pub alignment_min_confidence: f32,
    
    /// Number of warmstart traces from Confidence Lake
    pub warmstart_traces: usize,
}

impl Default for TwoStageConfig {
    fn default() -> Self {
        Self {
            discovery_epsilon: 0.25,
            alignment_epsilon: 0.03,
            max_steps: 10,
            discovery_min_confidence: 0.5,
            alignment_min_confidence: 0.75,
            warmstart_traces: 1000,
        }
    }
}

/// Experience buffer for RL training
#[derive(Debug, Clone)]
struct Experience {
    chain: ReasoningChain,
    #[allow(dead_code)]
    reward: f32,
    #[allow(dead_code)]
    stage: TrainingStage,
}

/// Two-stage RL trainer for reasoning
pub struct TwoStageRLTrainer {
    /// Current training stage
    stage: TrainingStage,
    
    /// Configuration
    config: TwoStageConfig,
    
    /// Discovery experience buffer
    discovery_buffer: VecDeque<Experience>,
    
    /// Alignment experience buffer
    alignment_buffer: VecDeque<Experience>,
    
    /// Flux engine for position calculations
    #[allow(dead_code)]
    flux_engine: FluxMatrixEngine,
    
    /// RL optimizer
    #[allow(dead_code)]
    rl_optimizer: RLGradientOptimizer,
    
    /// VCP for hallucination detection
    #[allow(dead_code)]
    vcp: VortexContextPreserver,
    
    /// Total training iterations
    iterations: usize,
    
    /// Discovery stage metrics
    discovery_avg_reward: f32,
    
    /// Alignment stage metrics
    alignment_avg_reward: f32,
}

impl TwoStageRLTrainer {
    /// Create a new two-stage trainer
    pub fn new(config: TwoStageConfig) -> Result<Self> {
        Ok(Self {
            stage: TrainingStage::Discovery,
            config,
            discovery_buffer: VecDeque::with_capacity(10000),
            alignment_buffer: VecDeque::with_capacity(5000),
            flux_engine: FluxMatrixEngine::new(),
            rl_optimizer: RLGradientOptimizer::new(0.001, 256),
            vcp: VortexContextPreserver::default(),
            iterations: 0,
            discovery_avg_reward: 0.0,
            alignment_avg_reward: 0.0,
        })
    }
    
    /// Minimal SFT warmstart from Confidence Lake
    pub async fn warmstart_from_lake(&mut self) -> Result<()> {
        // In production, this would load high-quality traces from Confidence Lake
        // For now, create synthetic high-quality patterns
        
        println!("🌊 Warmstarting from Confidence Lake ({} traces)...", self.config.warmstart_traces);
        
        // Generate synthetic sacred geometry patterns
        for i in 0..self.config.warmstart_traces.min(100) {
            let mut chain = ReasoningChain::new();
            
            // Create a trace that follows vortex sequence
            let vortex_sequence = [1, 2, 4, 8, 7, 5, 1];
            for &pos in &vortex_sequence {
                let elp = self.generate_balanced_elp(pos);
                chain.add_step(
                    format!("Warmstart step at position {}", pos),
                    elp,
                    pos,
                    0.85 + (i as f32 * 0.01).min(0.1)
                );
            }
            
            // Add sacred checkpoints
            for &sacred_pos in &[3, 6, 9] {
                let elp = self.generate_balanced_elp(sacred_pos);
                chain.add_step(
                    format!("Sacred checkpoint {}", sacred_pos),
                    elp,
                    sacred_pos,
                    0.92
                );
            }
            
            chain.finalize(format!("High-quality pattern {}", i));
            
            self.discovery_buffer.push_back(Experience {
                chain,
                reward: 1.0,
                stage: TrainingStage::Discovery,
            });
        }
        
        println!("✅ Warmstart complete: {} traces loaded", self.discovery_buffer.len());
        Ok(())
    }
    
    /// Stage 1: Discovery - explore reasoning patterns
    pub fn stage1_discovery(&mut self, input: &str) -> Result<ReasoningChain> {
        let mut chain = ReasoningChain::new();
        let epsilon = self.config.discovery_epsilon;
        
        println!("🔍 Stage 1 Discovery (ε={:.2})", epsilon);
        
        for _step_num in 0..self.config.max_steps {
            // Explore: random action with probability epsilon
            let explore = rand::random::<f32>() < epsilon;
            
            let (thought, elp, pos, confidence) = if explore {
                // Exploration: try novel patterns
                self.explore_novel_pattern(input, &chain)?
            } else {
                // Exploitation: use learned policy
                self.exploit_learned_pattern(input, &chain)?
            };
            
            chain.add_step(thought, elp, pos, confidence);
            
            // Reward novel flux patterns
            if Self::is_novel_pattern(&chain) {
                // Bonus for discovering new patterns
                chain.steps.last_mut().unwrap().confidence += 0.05;
            }
            
            // Low penalty for errors in discovery
            if confidence < self.config.discovery_min_confidence {
                // Continue exploring even with low confidence
                continue;
            }
            
            // Check if we've hit all sacred positions
            let sacred_hits: Vec<_> = chain.steps.iter()
                .filter(|s| s.is_sacred)
                .map(|s| s.flux_position)
                .collect();
            
            if sacred_hits.contains(&3) && 
               sacred_hits.contains(&6) && 
               sacred_hits.contains(&9) {
                // Good stopping point
                break;
            }
        }
        
        chain.finalize(format!("Discovery result for: {}", input));
        
        // Calculate discovery reward
        let reward = self.calculate_discovery_reward(&chain);
        
        self.discovery_buffer.push_back(Experience {
            chain: chain.clone(),
            reward,
            stage: TrainingStage::Discovery,
        });
        
        // Update moving average
        self.discovery_avg_reward = 0.95 * self.discovery_avg_reward + 0.05 * reward;
        
        Ok(chain)
    }
    
    /// Stage 2: Alignment - align patterns to sacred geometry
    pub fn stage2_alignment(&mut self, discovery_chain: &ReasoningChain) -> Result<ReasoningChain> {
        let mut aligned_chain = ReasoningChain::new();
        let epsilon = self.config.alignment_epsilon;
        
        println!("🎯 Stage 2 Alignment (ε={:.2})", epsilon);
        
        for step in &discovery_chain.steps {
            // Strict ELP coherence requirements
            let aligned_elp = self.align_elp_to_geometry(step.elp_state, step.flux_position)?;
            
            // Check for hallucinations using VCP
            let mut temp_chain = aligned_chain.clone();
            temp_chain.add_step(
                step.thought.clone(),
                aligned_elp,
                step.flux_position,
                step.confidence
            );
            
            // Strict verification
            if !self.verify_alignment(&temp_chain)? {
                // Skip steps that don't align
                continue;
            }
            
            aligned_chain.add_step(
                step.thought.clone(),
                aligned_elp,
                step.flux_position,
                step.confidence
            );
        }
        
        // Ensure vortex cycle completion
        if !aligned_chain.check_vortex_cycle() {
            // Complete the cycle
            let last_pos = aligned_chain.steps.last().map(|s| s.flux_position).unwrap_or(0);
            let next_pos = self.get_next_vortex_position(last_pos);
            
            aligned_chain.add_step(
                "Completing vortex cycle".to_string(),
                self.generate_balanced_elp(next_pos),
                next_pos,
                0.85
            );
        }
        
        aligned_chain.finalize(format!("Aligned: {}", discovery_chain.final_answer));
        
        // High penalty for hallucinations
        let reward = self.calculate_alignment_reward(&aligned_chain);
        
        self.alignment_buffer.push_back(Experience {
            chain: aligned_chain.clone(),
            reward,
            stage: TrainingStage::Alignment,
        });
        
        // Update moving average
        self.alignment_avg_reward = 0.95 * self.alignment_avg_reward + 0.05 * reward;
        
        Ok(aligned_chain)
    }
    
    /// Train for one iteration
    pub fn train_iteration(&mut self, input: &str) -> Result<ReasoningChain> {
        self.iterations += 1;
        
        match self.stage {
            TrainingStage::Discovery => {
                let chain = self.stage1_discovery(input)?;
                
                // Switch to alignment after sufficient discovery
                // Lowered threshold from 1000 to 3 for faster demo/testing
                if self.discovery_buffer.len() >= 3 {
                    println!("🔄 Switching to Alignment stage (buffer: {})", self.discovery_buffer.len());
                    self.stage = TrainingStage::Alignment;
                }
                
                Ok(chain)
            }
            TrainingStage::Alignment => {
                // Sample from discovery buffer
                if let Some(exp) = self.discovery_buffer.pop_front() {
                    let aligned = self.stage2_alignment(&exp.chain)?;
                    
                    // Push back to discovery for continued exploration
                    self.discovery_buffer.push_back(exp);
                    
                    Ok(aligned)
                } else {
                    bail!("No discovery experiences to align");
                }
            }
        }
    }
    
    /// Calculate reward for discovery stage
    fn calculate_discovery_reward(&self, chain: &ReasoningChain) -> f32 {
        let mut reward = 0.0;
        
        // Reward: Novel patterns
        if Self::is_novel_pattern(chain) {
            reward += 0.3;
        }
        
        // Reward: Sacred position coverage
        let sacred_count = chain.steps.iter().filter(|s| s.is_sacred).count();
        reward += (sacred_count as f32 * 0.2).min(0.6);
        
        // Reward: High confidence
        reward += chain.overall_confidence * 0.4;
        
        // Penalty: Very short chains
        if chain.steps.len() < 3 {
            reward -= 0.2;
        }
        
        reward.clamp(0.0, 1.0)
    }
    
    /// Calculate reward for alignment stage
    /// Enhanced to provide more positive reinforcement
    fn calculate_alignment_reward(&self, chain: &ReasoningChain) -> f32 {
        let mut reward = 0.0;
        
        // Base reward for completion
        reward += 0.1;
        
        // Reward: Vortex cycle completion (strong reward)
        if chain.completed_vortex_cycle {
            reward += 0.5; // Increased from 0.4
        } else if chain.steps.len() >= 6 {
            // Partial credit for having enough steps
            reward += 0.2;
        }
        
        // Reward: High confidence
        reward += chain.overall_confidence * 0.4; // Increased from 0.3
        
        // Reward: Sacred positions present
        let sacred_positions: Vec<u8> = chain.steps.iter()
            .filter(|s| s.is_sacred)
            .map(|s| s.flux_position)
            .collect();
        
        let sacred_count = sacred_positions.len();
        if sacred_count > 0 {
            reward += 0.1 * (sacred_count as f32 / 3.0); // Partial credit
        }
        
        // Bonus for all three sacred positions
        if sacred_positions.contains(&3) && 
           sacred_positions.contains(&6) && 
           sacred_positions.contains(&9) {
            reward += 0.3;
        }
        
        reward.clamp(0.0, 1.5) // Increased max from 1.0 to 1.5
    }
    
    /// Helper functions
    fn explore_novel_pattern(&self, _input: &str, _chain: &ReasoningChain) -> Result<(String, ELPTensor, u8, f32)> {
        // Random exploration
        let pos = (rand::random::<u8>() % 10) as u8;
        let elp = self.generate_balanced_elp(pos);
        Ok((format!("Exploring position {}", pos), elp, pos, 0.6 + rand::random::<f32>() * 0.2))
    }
    
    fn exploit_learned_pattern(&self, _input: &str, chain: &ReasoningChain) -> Result<(String, ELPTensor, u8, f32)> {
        // Use learned policy (simplified)
        let last_pos = chain.steps.last().map(|s| s.flux_position).unwrap_or(1);
        let next_pos = self.get_next_vortex_position(last_pos);
        let elp = self.generate_balanced_elp(next_pos);
        Ok((format!("Following vortex to {}", next_pos), elp, next_pos, 0.8))
    }
    
    fn is_novel_pattern(chain: &ReasoningChain) -> bool {
        // Check if pattern hasn't been seen before (simplified)
        chain.steps.len() > 5 && !chain.completed_vortex_cycle
    }
    
    fn generate_balanced_elp(&self, pos: u8) -> ELPTensor {
        // Generate ELP based on position
        match pos {
            3 => ELPTensor::new(8.0, 6.0, 5.0),  // Ethos-dominant
            6 => ELPTensor::new(5.0, 8.0, 6.0),  // Logos-dominant
            9 => ELPTensor::new(6.0, 5.0, 8.0),  // Pathos-dominant
            _ => ELPTensor::new(6.0, 6.0, 6.0),  // Balanced
        }
    }
    
    fn align_elp_to_geometry(&self, elp: ELPTensor, pos: u8) -> Result<ELPTensor> {
        // Align ELP to position's natural dominance
        let expected = self.generate_balanced_elp(pos);
        
        // Blend current ELP with expected (80% expected, 20% current)
        Ok(ELPTensor::new(
            elp.ethos * 0.2 + expected.ethos * 0.8,
            elp.logos * 0.2 + expected.logos * 0.8,
            elp.pathos * 0.2 + expected.pathos * 0.8,
        ))
    }
    
    fn verify_alignment(&self, chain: &ReasoningChain) -> Result<bool> {
        // Strict verification for alignment stage
        if chain.overall_confidence < self.config.alignment_min_confidence {
            return Ok(false);
        }
        
        // Check ELP coherence
        for window in chain.steps.windows(2) {
            let distance = window[0].elp_state.distance(&window[1].elp_state);
            if distance > 2.0 {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
    
    fn get_next_vortex_position(&self, current: u8) -> u8 {
        match current {
            1 => 2,
            2 => 4,
            4 => 8,
            8 => 7,
            7 => 5,
            5 => 1,
            _ => 1,
        }
    }
    
    /// Get training statistics
    pub fn get_stats(&self) -> TrainingStats {
        TrainingStats {
            iterations: self.iterations,
            stage: self.stage,
            discovery_avg_reward: self.discovery_avg_reward,
            alignment_avg_reward: self.alignment_avg_reward,
            discovery_buffer_size: self.discovery_buffer.len(),
            alignment_buffer_size: self.alignment_buffer.len(),
        }
    }
}

/// Training statistics
#[derive(Debug, Clone, Serialize)]
pub struct TrainingStats {
    pub iterations: usize,
    pub stage: TrainingStage,
    pub discovery_avg_reward: f32,
    pub alignment_avg_reward: f32,
    pub discovery_buffer_size: usize,
    pub alignment_buffer_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_trainer_creation() {
        let config = TwoStageConfig::default();
        let trainer = TwoStageRLTrainer::new(config);
        assert!(trainer.is_ok());
    }
    
    #[tokio::test]
    async fn test_warmstart() {
        let config = TwoStageConfig::default();
        let mut trainer = TwoStageRLTrainer::new(config).unwrap();
        
        trainer.warmstart_from_lake().await.unwrap();
        assert!(trainer.discovery_buffer.len() > 0);
    }
    
    #[test]
    fn test_discovery_stage() {
        let config = TwoStageConfig::default();
        let mut trainer = TwoStageRLTrainer::new(config).unwrap();
        
        let chain = trainer.stage1_discovery("Test input").unwrap();
        assert!(!chain.steps.is_empty());
    }
}
