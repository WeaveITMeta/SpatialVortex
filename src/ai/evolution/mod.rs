//! Evolution Loop - The Self-Improvement Cycle
//! 
//! This module implements the "Sleep Cycle" of the AGI.
//! It analyzes recent experiences (from Postgres) and uses Reinforcement Learning
//! to optimize the "Sacred Weights" that govern future reasoning.

use crate::ai::flux_reasoning::FluxReasoningChain;
use crate::ml::rl_gradient_optimizer::{RLGradientOptimizer, GradientState, GradientAction, RewardSignal};
use crate::storage::SpatialDatabase;
use crate::data::elp_attributes::DynamicELP;
use crate::core::sacred_geometry::flux_matrix::FluxMatrixEngine;
use nalgebra::DVector;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

/// The evolving brain state
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BrainState {
    /// Current optimal weights for sacred positions [0..9]
    pub sacred_weights: [f32; 10],
    /// Generation number (how many sleep cycles)
    pub generation: u64,
    /// Global confidence score
    pub evolution_score: f32,
}

impl Default for BrainState {
    fn default() -> Self {
        let mut sacred_weights = [1.0; 10];
        sacred_weights[3] = 1.5;
        sacred_weights[6] = 1.5;
        sacred_weights[9] = 1.5;
        
        Self {
            sacred_weights,
            generation: 0,
            evolution_score: 0.0,
        }
    }
}

pub struct EvolutionEngine {
    db: SpatialDatabase,
    optimizer: Arc<Mutex<RLGradientOptimizer>>,
    current_state: Arc<Mutex<BrainState>>,
}

impl EvolutionEngine {
    pub fn new(db: SpatialDatabase) -> Self {
        Self {
            db,
            optimizer: Arc::new(Mutex::new(RLGradientOptimizer::new(0.01, 64))),
            current_state: Arc::new(Mutex::new(BrainState::default())),
        }
    }
    
    /// Run a "Sleep Cycle" - Analyze recent patterns and update brain
    pub async fn run_sleep_cycle(&self) -> Result<BrainState> {
        tracing::info!("💤 Entering Sleep Cycle (Evolution)...");
        
        // 1. Fetch recent successful patterns from DB
        // In a real impl, we'd deserialized full chains.
        // For now, we mock the "experience replay"
        let experiences = self.fetch_recent_experiences().await?;
        
        let mut optimizer = self.optimizer.lock().await;
        let mut state = self.current_state.lock().await;
        
        for exp in experiences {
            // Convert experience to GradientState
            let gradient_state = self.chain_to_gradient_state(&exp);
            
            // Let RL agent decide how to adjust weights
            let action = optimizer.select_action(&gradient_state);
            
            // Apply action to weights (temporarily)
            let mut temp_weights = state.sacred_weights;
            match action {
                GradientAction::Reward(boost) => {
                    // Boost the sacred positions used in this chain
                    for &pos in &exp.sacred_milestones {
                        temp_weights[pos as usize] *= boost;
                    }
                },
                GradientAction::Penalize(dampen) => {
                     for &pos in &exp.sacred_milestones {
                        temp_weights[pos as usize] *= dampen;
                    }
                },
                _ => {}
            }
            
            // Calculate Reward: Did this chain actually succeed?
            let reward = RewardSignal {
                immediate: if exp.chain_confidence > 0.8 { 1.0 } else { -0.5 },
                value: 0.0, // Simplified
                is_positive: exp.chain_confidence > 0.8,
                should_halve: false,
            };
            
            // Store experience for PPO training
            // (Simplified: we just use the final state as next_state for now)
            optimizer.store_experience(
                gradient_state.clone(),
                action,
                reward,
                gradient_state, // Dummy next state
                true
            );
        }
        
        // 2. Train the optimizer (update policy)
        let loss = optimizer.train_on_experience()?;
        tracing::info!("🧠 Brain Plasticity Update: Loss = {:.4}", loss);
        
        // 3. Evolve the weights based on the new policy
        // We nudge the global weights slightly in the direction of the successful experiments
        state.generation += 1;
        
        // Save state (in real app, write to disk/db)
        
        Ok(state.clone())
    }
    
    // Helper to mocked fetch
    async fn fetch_recent_experiences(&self) -> Result<Vec<FluxReasoningChain>> {
        // In production: SELECT data FROM flux_matrices WHERE ...
        Ok(vec![]) 
    }
    
    fn chain_to_gradient_state(&self, chain: &FluxReasoningChain) -> GradientState {
        // Convert a reasoning chain into a state the RL optimizer can understand
        let last_thought = chain.thoughts.last().unwrap();
        
        // Calculate color
        let color = crate::data::elp_attributes::AttributeColor::from_elp_dominance(
            last_thought.elp_state.ethos as f32, 
            last_thought.elp_state.logos as f32, 
            last_thought.elp_state.pathos as f32
        );

        GradientState {
            gradients: DVector::from_element(10, 0.0), // Placeholder
            elp: DynamicELP {
                ethos: nalgebra::Vector3::new(last_thought.elp_state.ethos as f32, 0.0, 0.0),
                logos: nalgebra::Vector3::new(last_thought.elp_state.logos as f32, 0.0, 0.0),
                pathos: nalgebra::Vector3::new(last_thought.elp_state.pathos as f32, 0.0, 0.0),
                dominant_value: last_thought.vortex_position,
                state: crate::data::elp_attributes::AttributeState::Stable,
                color,
                importance: 0.5,
                trajectory: vec![],
                timestamp: chrono::Utc::now(),
            },
            flux_position: last_thought.vortex_position,
            confidence: chain.chain_confidence,
            cumulative_reward: 0.0,
        }
    }
}
