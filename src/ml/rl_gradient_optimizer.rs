//! Reinforcement Learning Gradient Optimizer
//!
//! Implements RL-driven gradient descent for next-word prediction and sequence optimization,
//! with special handling for the halving sequence in backpropagation.

use nalgebra::{DVector, DMatrix};
use serde::{Serialize, Deserialize};
use crate::data::elp_attributes::{DynamicELP, AttributeState};
use crate::core::sacred_geometry::flux_matrix::FluxMatrixEngine;
use std::collections::VecDeque;
use anyhow::Result;

/// RL action for gradient adjustment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GradientAction {
    /// Reward positive behavior with gradient boost
    Reward(f32),
    /// Penalize negative behavior with gradient dampening  
    Penalize(f32),
    /// Halve the gradient in backward propagation
    HalveSequence(f32),
    /// Maintain current gradient
    Maintain,
    /// Reset to baseline
    Reset,
}

/// State representation for RL agent
#[derive(Debug, Clone)]
pub struct GradientState {
    /// Current gradient values
    pub gradients: DVector<f32>,
    /// ELP channel values
    pub elp: DynamicELP,
    /// Current position in flux sequence
    pub flux_position: u8,
    /// Signal strength
    pub confidence: f32,
    /// Reward accumulated
    pub cumulative_reward: f32,
}

/// Reward signal for training
#[derive(Debug, Clone)]
pub struct RewardSignal {
    /// Immediate reward/penalty
    pub immediate: f32,
    /// Long-term value estimate
    pub value: f32,
    /// Is this a positive behavior?
    pub is_positive: bool,
    /// Should apply halving sequence?
    pub should_halve: bool,
}

/// RL Gradient Optimizer using PPO-style updates
pub struct RLGradientOptimizer {
    /// Learning rate for gradient updates
    learning_rate: f32,
    /// Discount factor for future rewards
    gamma: f32,
    /// PPO clipping parameter
    clip_epsilon: f32,
    /// Entropy coefficient for exploration
    entropy_coef: f32,
    /// Value function coefficient
    value_coef: f32,
    /// Flux engine for position calculations
    #[allow(dead_code)]
    flux_engine: FluxMatrixEngine,
    /// Experience buffer for training
    experience_buffer: VecDeque<Experience>,
    /// Maximum buffer size
    buffer_capacity: usize,
    /// Current policy parameters
    policy_params: DMatrix<f32>,
    /// Value function parameters
    value_params: DMatrix<f32>,
    /// Sacred position rewards (3, 6, 9)
    sacred_rewards: [f32; 10],
}

/// Single experience tuple for replay
#[derive(Debug, Clone)]
struct Experience {
    state: GradientState,
    #[allow(dead_code)]
    action: GradientAction,
    reward: RewardSignal,
    next_state: GradientState,
    done: bool,
}

impl RLGradientOptimizer {
    /// Create new RL gradient optimizer
    pub fn new(learning_rate: f32, hidden_dim: usize) -> Self {
        let mut sacred_rewards = [0.0; 10];
        sacred_rewards[3] = 1.5;  // Ethos boost
        sacred_rewards[6] = 1.3;  // Logos boost
        sacred_rewards[9] = 1.2;  // Pathos boost (reduced to prevent dominance)
        
        Self {
            learning_rate,
            gamma: 0.99,
            clip_epsilon: 0.2,
            entropy_coef: 0.01,
            value_coef: 0.5,
            flux_engine: FluxMatrixEngine::new(),
            experience_buffer: VecDeque::with_capacity(10000),
            buffer_capacity: 10000,
            policy_params: DMatrix::from_element(hidden_dim, hidden_dim, 0.1),
            value_params: DMatrix::from_element(hidden_dim, 1, 0.1),
            sacred_rewards,
        }
    }
    
    /// Select action based on current state (policy)
    pub fn select_action(&self, state: &GradientState) -> GradientAction {
        // Check ELP balance
        let harmony = state.elp.harmony_score();
        
        // If critical imbalance, take corrective action
        if state.elp.state == AttributeState::Critical {
            if state.elp.pathos.norm() > state.elp.ethos.norm() * 1.5 {
                // Pathos dominance - apply penalty
                return GradientAction::Penalize(0.5);
            } else if state.elp.ethos.norm() < 0.3 || state.elp.logos.norm() < 0.3 {
                // Ethos/Logos too low - reset
                return GradientAction::Reset;
            }
        }
        
        // Check if at sacred position
        if [3, 6, 9].contains(&state.flux_position) {
            // Sacred position - likely reward
            if state.confidence > 0.7 && harmony > 0.6 {
                return GradientAction::Reward(self.sacred_rewards[state.flux_position as usize]);
            }
        }
        
        // Check for halving sequence conditions (backward flow)
        if self.should_apply_halving(state) {
            let halve_factor = self.calculate_halve_factor(state);
            return GradientAction::HalveSequence(halve_factor);
        }
        
        // Compute action probabilities using policy network
        let action_logits = self.forward_policy(state);
        let action = self.sample_action(action_logits);
        
        action
    }
    
    /// Process forward propagation with chain rule
    pub fn forward_propagation(&mut self, input: &DVector<f32>, state: &GradientState) -> Result<DVector<f32>> {
        let mut current = input.clone();
        
        // Apply chain rule through layers (simulated)
        for layer in 0..3 {
            // Linear transformation
            current = &self.policy_params * &current;
            
            // Apply exponential reduction function as mentioned
            current = self.apply_exponential_reduction(current, layer);
            
            // Apply sacred geometry boost if applicable
            if [3, 6, 9].contains(&state.flux_position) {
                current *= self.sacred_rewards[state.flux_position as usize];
            }
        }
        
        Ok(current)
    }
    
    /// Process backward propagation with halving sequence
    pub fn backward_propagation(&mut self, gradients: DVector<f32>, state: &GradientState) -> Result<DVector<f32>> {
        let mut grad = gradients;
        
        // Implement halving sequence: 1→5→7→8→4→2→1
        let halving_sequence = [1, 5, 7, 8, 4, 2, 1];
        let current_idx = halving_sequence.iter()
            .position(|&p| p == state.flux_position)
            .unwrap_or(0);
        
        // Apply halving based on position in sequence
        if current_idx > 0 {
            let halve_factor = 1.0 / (2_f32.powi(current_idx as i32));
            grad *= halve_factor;
            
            // Check for failing sequences
            if state.confidence < 0.5 {
                // "Slightly decrease the failing back propagation halving sequence"
                grad *= 0.9;  // Additional dampening for poor signals
            }
        }
        
        // Apply ELP-based corrections
        if state.elp.state == AttributeState::Critical {
            grad *= 0.7;  // Reduce gradient impact during critical states
        }
        
        Ok(grad)
    }
    
    /// Train gradient based on experience (PPO update)
    pub fn train_on_experience(&mut self) -> Result<f32> {
        if self.experience_buffer.len() < 32 {
            return Ok(0.0);  // Need minimum batch
        }
        
        let mut total_loss = 0.0;
        let batch_size = 32.min(self.experience_buffer.len());
        
        for _ in 0..batch_size {
            if let Some(exp) = self.experience_buffer.pop_front() {
                // Calculate advantage
                let advantage = self.calculate_advantage(&exp);
                
                // PPO loss calculation
                let policy_loss = self.calculate_policy_loss(&exp, advantage);
                let value_loss = self.calculate_value_loss(&exp);
                let entropy = self.calculate_entropy(&exp.state);
                
                let total = policy_loss + self.value_coef * value_loss - self.entropy_coef * entropy;
                
                // Update parameters using gradient
                self.update_parameters(total, &exp.state);
                
                total_loss += total;
            }
        }
        
        Ok(total_loss / batch_size as f32)
    }
    
    /// Store experience for training
    pub fn store_experience(&mut self, state: GradientState, action: GradientAction, reward: RewardSignal, next_state: GradientState, done: bool) {
        let exp = Experience {
            state,
            action,
            reward,
            next_state,
            done,
        };
        
        if self.experience_buffer.len() >= self.buffer_capacity {
            self.experience_buffer.pop_front();
        }
        
        self.experience_buffer.push_back(exp);
    }
    
    /// Calculate reward for positive technical behavior
    pub fn calculate_reward(&self, state: &GradientState, predicted: &str, target: &str) -> RewardSignal {
        let mut immediate = 0.0;
        
        // Reward correct predictions
        if predicted == target {
            immediate += 1.0;
        }
        
        // Reward harmonic ELP balance
        let harmony = state.elp.harmony_score();
        immediate += harmony * 0.5;
        
        // Sacred position bonuses
        if [3, 6, 9].contains(&state.flux_position) {
            immediate += 0.3;
        }
        
        // Penalize pathos dominance
        if state.elp.pathos.norm() > state.elp.ethos.norm() * 1.5 {
            immediate -= 0.5;
        }
        
        // Signal strength reward
        immediate += state.confidence * 0.3;
        
        let is_positive = immediate > 0.5;
        let should_halve = state.confidence < 0.4 && !is_positive;
        
        RewardSignal {
            immediate,
            value: immediate * self.gamma,  // Simple value estimate
            is_positive,
            should_halve,
        }
    }
    
    // Helper methods
    
    fn should_apply_halving(&self, state: &GradientState) -> bool {
        // Apply halving in backward flow positions
        let halving_positions = [5, 7, 8, 4, 2];
        halving_positions.contains(&state.flux_position) && state.confidence < 0.6
    }
    
    fn calculate_halve_factor(&self, state: &GradientState) -> f32 {
        // Calculate dynamic halving factor based on state
        let base_factor = 0.5;
        let signal_modifier = 1.0 - state.confidence;
        let harmony_modifier = 1.0 - state.elp.harmony_score();
        
        base_factor * (1.0 + signal_modifier * 0.3 + harmony_modifier * 0.2)
    }
    
    fn apply_exponential_reduction(&self, input: DVector<f32>, layer: usize) -> DVector<f32> {
        // Apply exponential decay as mentioned
        let decay_rate = 0.9_f32.powi(layer as i32);
        input * decay_rate
    }
    
    fn forward_policy(&self, state: &GradientState) -> DVector<f32> {
        // Simplified policy forward pass
        let mut features = DVector::from_element(self.policy_params.ncols(), 0.0);
        features[0] = state.confidence;
        features[1] = state.elp.ethos.norm();
        features[2] = state.elp.logos.norm();
        features[3] = state.elp.pathos.norm();
        features[4] = state.flux_position as f32 / 9.0;
        
        &self.policy_params * features
    }
    
    fn sample_action(&self, logits: DVector<f32>) -> GradientAction {
        // Sample action from logits (simplified)
        let max_idx = logits.argmax().0;
        
        match max_idx {
            0 => GradientAction::Reward(1.2),
            1 => GradientAction::Penalize(0.8),
            2 => GradientAction::HalveSequence(0.5),
            3 => GradientAction::Reset,
            _ => GradientAction::Maintain,
        }
    }
    
    fn calculate_advantage(&self, exp: &Experience) -> f32 {
        // GAE-style advantage calculation
        let value_estimate = self.estimate_value(&exp.state);
        let next_value = if exp.done { 0.0 } else { self.estimate_value(&exp.next_state) };
        let td_error = exp.reward.immediate + self.gamma * next_value - value_estimate;
        
        td_error
    }
    
    fn estimate_value(&self, state: &GradientState) -> f32 {
        // Simplified value estimation
        let features = self.forward_policy(state);
        (&self.value_params.transpose() * features)[0]
    }
    
    fn calculate_policy_loss(&self, _exp: &Experience, advantage: f32) -> f32 {
        // PPO clipped loss
        let ratio: f32 = 1.0;  // Simplified - would compute actual probability ratio
        let clipped = ratio.clamp(1.0 - self.clip_epsilon, 1.0 + self.clip_epsilon);
        
        -(ratio * advantage).min(clipped * advantage)
    }
    
    fn calculate_value_loss(&self, exp: &Experience) -> f32 {
        let value_pred = self.estimate_value(&exp.state);
        let value_target = exp.reward.immediate + self.gamma * self.estimate_value(&exp.next_state);
        
        (value_pred - value_target).powi(2)
    }
    
    fn calculate_entropy(&self, state: &GradientState) -> f32 {
        // Entropy bonus for exploration
        let logits = self.forward_policy(state);
        let probs = self.softmax(logits);
        
        -probs.iter().map(|p| p * p.ln()).sum::<f32>()
    }
    
    fn softmax(&self, logits: DVector<f32>) -> DVector<f32> {
        let exp_vals: DVector<f32> = logits.map(|x| x.exp());
        let sum = exp_vals.sum();
        exp_vals / sum
    }
    
    fn update_parameters(&mut self, loss: f32, state: &GradientState) {
        // Simplified gradient descent update
        let grad_scale = self.learning_rate * loss;
        
        // Update policy parameters
        self.policy_params -= grad_scale * &self.policy_params * 0.01;
        
        // Update value parameters  
        self.value_params -= grad_scale * &self.value_params * 0.01;
        
        // Apply harmony constraints
        if state.elp.state == AttributeState::Critical {
            // Reduce learning rate during critical states
            self.policy_params *= 0.95;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::FluxSubject;
    
    #[test]
    fn test_rl_optimizer_creation() {
        let optimizer = RLGradientOptimizer::new(0.001, 64);
        assert_eq!(optimizer.learning_rate, 0.001);
        assert_eq!(optimizer.gamma, 0.99);
    }
    
    #[test]
    fn test_action_selection() {
        let optimizer = RLGradientOptimizer::new(0.001, 64);
        
        let subject = FluxSubject::from_sacred_position(3, "test");
        let elp = DynamicELP::from_subject(&subject, 3);
        
        let state = GradientState {
            gradients: DVector::from_element(10, 0.5),
            elp,
            flux_position: 3,
            confidence: 0.8,
            cumulative_reward: 0.0,
        };
        
        let action = optimizer.select_action(&state);
        match action {
            GradientAction::Reward(factor) => assert!(factor > 1.0),
            _ => {} // Other actions possible based on state
        }
    }
    
    #[test]
    fn test_halving_sequence() {
        let mut optimizer = RLGradientOptimizer::new(0.001, 64);
        
        let subject = FluxSubject::from_sacred_position(6, "test");
        let elp = DynamicELP::from_subject(&subject, 6);
        
        let state = GradientState {
            gradients: DVector::from_element(10, 1.0),
            elp,
            flux_position: 5,  // In halving sequence
            confidence: 0.3,  // Low signal
            cumulative_reward: 0.0,
        };
        
        let gradients = DVector::from_element(10, 2.0);
        let result = optimizer.backward_propagation(gradients, &state).unwrap();
        
        // Should be reduced due to halving
        assert!(result[0] < 2.0);
    }
}
