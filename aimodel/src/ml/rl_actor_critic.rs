//! Reinforcement Learning with Human Feedback as Evidence Synthesis
//!
//! Table of Contents:
//! 1. EvidenceEvent — Structured evidence log replacing scalar rewards
//! 2. StateTransition — Quantified trait update evaluation
//! 3. ActorCriticPolicy — Actor-critic RL for path curation
//! 4. QTable — Tabular Q-learning for discrete state-action spaces
//! 5. HumanFeedbackEncoder — Encodes human inputs as structured events
//! 6. PathCurator — RL-driven path optimization with long-term metrics
//! 7. RLMetrics — Long-term metrics (rollback frequency, alignment scores)
//!
//! Architecture:
//! RL shifts from scalar rewards to an evidentiary framework.
//! Human inputs (validations, corrections, preferences) are encoded as
//! event objects within sliding indexes. Actor-critic evaluates state
//! transitions: how trait updates mitigate contradictions, enhance path
//! stability, or preserve entity identity. Rewards stem from long-term
//! metrics, not immediate task success.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// =============================================================================
// 1. EvidenceEvent — Structured evidence log replacing scalar rewards
// =============================================================================

/// A structured evidence event that enriches trait indexes.
/// Replaces scalar rewards with rich, metadata-bearing event objects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceEvent {
    /// Unique event ID
    pub id: u64,
    /// Timestamp (Unix ms)
    pub timestamp_ms: u64,
    /// Type of evidence
    pub evidence_type: EvidenceType,
    /// The trait(s) this evidence pertains to
    pub trait_names: Vec<String>,
    /// Quantified impact on state
    pub impact: f64,
    /// Confidence in this evidence
    pub confidence: f64,
    /// Provenance: who provided this evidence
    pub source: String,
    /// Metadata key-value pairs
    pub metadata: HashMap<String, String>,
}

/// Types of evidence that can be recorded
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EvidenceType {
    /// Human validation: "this causal link holds"
    Validation,
    /// Human correction: "weaken this correlation"
    Correction,
    /// Human preference: "prioritize recent events over trusted ones"
    Preference,
    /// Automated consistency check result
    ConsistencyCheck,
    /// Path stability measurement
    PathStability,
    /// Entity identity preservation check
    IdentityPreservation,
    /// Contradiction detection
    ContradictionDetected,
    /// Successful inference outcome
    InferenceSuccess,
    /// Failed inference outcome
    InferenceFailure,
}

// =============================================================================
// 2. StateTransition — Quantified trait update evaluation
// =============================================================================

/// A state transition representing a trait update and its consequences.
/// The actor-critic evaluates these to learn optimal update policies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    /// State before the transition (trait values snapshot)
    pub state_before: Vec<f64>,
    /// Action taken (trait delta applied)
    pub action: Vec<f64>,
    /// State after the transition
    pub state_after: Vec<f64>,
    /// Multi-dimensional reward signal
    pub reward: TransitionReward,
    /// Whether this transition was terminal
    pub terminal: bool,
}

/// Multi-dimensional reward signal for state transitions.
/// Rewards stem from long-term metrics, not immediate task success.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionReward {
    /// Contradiction mitigation score (did this reduce contradictions?)
    pub contradiction_mitigation: f64,
    /// Path stability score (did inference divergence decrease?)
    pub path_stability: f64,
    /// Entity identity preservation (did entity coherence improve?)
    pub identity_preservation: f64,
    /// Human alignment score (did human feedback improve?)
    pub human_alignment: f64,
    /// Rollback frequency reduction (fewer rollbacks = better)
    pub rollback_reduction: f64,
}

impl TransitionReward {
    /// Compute scalar reward as weighted sum of components
    pub fn scalar(&self, weights: &RewardWeights) -> f64 {
        self.contradiction_mitigation * weights.contradiction
            + self.path_stability * weights.path_stability
            + self.identity_preservation * weights.identity
            + self.human_alignment * weights.human_alignment
            + self.rollback_reduction * weights.rollback_reduction
    }

    /// Zero reward
    pub fn zero() -> Self {
        Self {
            contradiction_mitigation: 0.0,
            path_stability: 0.0,
            identity_preservation: 0.0,
            human_alignment: 0.0,
            rollback_reduction: 0.0,
        }
    }
}

/// Weights for combining multi-dimensional rewards into scalar
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardWeights {
    pub contradiction: f64,
    pub path_stability: f64,
    pub identity: f64,
    pub human_alignment: f64,
    pub rollback_reduction: f64,
}

impl Default for RewardWeights {
    fn default() -> Self {
        Self {
            contradiction: 0.25,
            path_stability: 0.25,
            identity: 0.15,
            human_alignment: 0.25,
            rollback_reduction: 0.10,
        }
    }
}

// =============================================================================
// 3. ActorCriticPolicy — Actor-critic RL for path curation
// =============================================================================

/// Actor-Critic policy network for trait update optimization.
/// The actor proposes trait deltas; the critic evaluates state value.
/// Both are simple linear models suitable for the trait state space.
pub struct ActorCriticPolicy {
    /// State dimension (number of trait features)
    state_dim: usize,
    /// Action dimension (number of possible delta dimensions)
    action_dim: usize,
    /// Actor weights: state → action (mean of Gaussian policy)
    actor_weights: Vec<Vec<f64>>,
    /// Actor bias
    actor_bias: Vec<f64>,
    /// Critic weights: state → value estimate
    critic_weights: Vec<f64>,
    /// Critic bias
    critic_bias: f64,
    /// Learning rate for actor
    actor_lr: f64,
    /// Learning rate for critic
    critic_lr: f64,
    /// Discount factor (gamma)
    gamma: f64,
    /// Reward weights for multi-dimensional rewards
    reward_weights: RewardWeights,
    /// Experience replay buffer
    replay_buffer: Vec<StateTransition>,
    /// Maximum replay buffer size
    max_buffer_size: usize,
    /// Training step counter
    training_steps: u64,
    /// Running average of value estimates (baseline)
    value_baseline: f64,
}

impl ActorCriticPolicy {
    /// Create a new actor-critic policy
    pub fn new(state_dim: usize, action_dim: usize) -> Self {
        // Xavier initialization for weights
        let scale = (2.0 / (state_dim + action_dim) as f64).sqrt();
        let mut actor_weights = vec![vec![0.0; state_dim]; action_dim];
        let mut seed = 42u64;
        for row in &mut actor_weights {
            for w in row.iter_mut() {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                *w = ((seed >> 33) as f64 / u32::MAX as f64 - 0.5) * scale;
            }
        }

        let mut critic_weights = vec![0.0; state_dim];
        for w in &mut critic_weights {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            *w = ((seed >> 33) as f64 / u32::MAX as f64 - 0.5) * scale;
        }

        Self {
            state_dim,
            action_dim,
            actor_weights,
            actor_bias: vec![0.0; action_dim],
            critic_weights,
            critic_bias: 0.0,
            actor_lr: 0.001,
            critic_lr: 0.005,
            gamma: 0.99,
            reward_weights: RewardWeights::default(),
            replay_buffer: Vec::new(),
            max_buffer_size: 10000,
            training_steps: 0,
            value_baseline: 0.0,
        }
    }

    /// Set learning rates
    pub fn with_learning_rates(mut self, actor_lr: f64, critic_lr: f64) -> Self {
        self.actor_lr = actor_lr;
        self.critic_lr = critic_lr;
        self
    }

    /// Set reward weights
    pub fn with_reward_weights(mut self, weights: RewardWeights) -> Self {
        self.reward_weights = weights;
        self
    }

    /// Actor forward pass: state → proposed action (trait delta)
    pub fn propose_action(&self, state: &[f64]) -> Vec<f64> {
        let mut action = vec![0.0; self.action_dim];
        for (i, row) in self.actor_weights.iter().enumerate() {
            let mut sum = self.actor_bias[i];
            for (j, &s) in state.iter().enumerate().take(self.state_dim) {
                sum += row.get(j).copied().unwrap_or(0.0) * s;
            }
            // Tanh activation to bound actions
            action[i] = sum.tanh();
        }
        action
    }

    /// Critic forward pass: state → value estimate
    pub fn estimate_value(&self, state: &[f64]) -> f64 {
        let mut value = self.critic_bias;
        for (i, &s) in state.iter().enumerate().take(self.state_dim) {
            value += self.critic_weights.get(i).copied().unwrap_or(0.0) * s;
        }
        value
    }

    /// Record a transition in the replay buffer
    pub fn record_transition(&mut self, transition: StateTransition) {
        if self.replay_buffer.len() >= self.max_buffer_size {
            self.replay_buffer.remove(0);
        }
        self.replay_buffer.push(transition);
    }

    /// Train one step using the latest transition (online A2C)
    pub fn train_step(&mut self, transition: &StateTransition) -> f64 {
        let reward = transition.reward.scalar(&self.reward_weights);

        // Critic: estimate V(s) and V(s')
        let v_current = self.estimate_value(&transition.state_before);
        let v_next = if transition.terminal {
            0.0
        } else {
            self.estimate_value(&transition.state_after)
        };

        // TD error (advantage estimate)
        let td_error = reward + self.gamma * v_next - v_current;

        // Update critic: minimize TD error
        for (i, &s) in transition.state_before.iter().enumerate().take(self.state_dim) {
            if let Some(w) = self.critic_weights.get_mut(i) {
                *w += self.critic_lr * td_error * s;
            }
        }
        self.critic_bias += self.critic_lr * td_error;

        // Update actor: policy gradient with advantage
        let proposed = self.propose_action(&transition.state_before);
        for (i, row) in self.actor_weights.iter_mut().enumerate() {
            let action_error = transition.action.get(i).copied().unwrap_or(0.0) - proposed[i];
            // Gradient of tanh: 1 - tanh^2
            let tanh_grad = 1.0 - proposed[i] * proposed[i];
            for (j, &s) in transition.state_before.iter().enumerate().take(self.state_dim) {
                if let Some(w) = row.get_mut(j) {
                    *w += self.actor_lr * td_error * action_error * tanh_grad * s;
                }
            }
            self.actor_bias[i] += self.actor_lr * td_error * action_error * tanh_grad;
        }

        // Update baseline
        self.value_baseline = 0.99 * self.value_baseline + 0.01 * reward;
        self.training_steps += 1;

        td_error
    }

    /// Batch train on replay buffer
    pub fn train_batch(&mut self, batch_size: usize) -> f64 {
        if self.replay_buffer.is_empty() {
            return 0.0;
        }

        let n = self.replay_buffer.len().min(batch_size);
        let start = self.replay_buffer.len().saturating_sub(n);
        let batch: Vec<StateTransition> = self.replay_buffer[start..].to_vec();

        let mut total_td = 0.0;
        for transition in &batch {
            total_td += self.train_step(transition).abs();
        }
        total_td / n as f64
    }

    /// Get training statistics
    pub fn stats(&self) -> ActorCriticStats {
        ActorCriticStats {
            training_steps: self.training_steps,
            replay_buffer_size: self.replay_buffer.len(),
            value_baseline: self.value_baseline,
            actor_weight_norm: self.actor_weights.iter()
                .flat_map(|r| r.iter())
                .map(|w| w * w)
                .sum::<f64>()
                .sqrt(),
            critic_weight_norm: self.critic_weights.iter()
                .map(|w| w * w)
                .sum::<f64>()
                .sqrt(),
        }
    }
}

/// Actor-critic training statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorCriticStats {
    pub training_steps: u64,
    pub replay_buffer_size: usize,
    pub value_baseline: f64,
    pub actor_weight_norm: f64,
    pub critic_weight_norm: f64,
}

// =============================================================================
// 4. QTable — Tabular Q-learning for discrete state-action spaces
// =============================================================================

/// Tabular Q-learning for discrete trait update decisions.
/// Used for coarse-grained decisions like "should we accept this update?"
pub struct QTable {
    /// Q-values: (state_hash, action) → value
    q_values: HashMap<(u64, u32), f64>,
    /// Visit counts for exploration bonus
    visit_counts: HashMap<(u64, u32), u64>,
    /// Learning rate
    alpha: f64,
    /// Discount factor
    gamma: f64,
    /// Exploration rate (epsilon-greedy)
    epsilon: f64,
    /// Epsilon decay rate
    epsilon_decay: f64,
    /// Minimum epsilon
    epsilon_min: f64,
    /// Number of possible actions
    num_actions: u32,
    /// Training episodes
    episodes: u64,
}

/// Discrete actions for the Q-table
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiscreteAction {
    /// Accept the proposed trait update
    Accept = 0,
    /// Reject the proposed trait update
    Reject = 1,
    /// Accept with reduced magnitude
    AcceptReduced = 2,
    /// Defer to human feedback
    DeferToHuman = 3,
    /// Trigger rollback
    TriggerRollback = 4,
}

impl DiscreteAction {
    pub fn from_index(idx: u32) -> Self {
        match idx {
            0 => DiscreteAction::Accept,
            1 => DiscreteAction::Reject,
            2 => DiscreteAction::AcceptReduced,
            3 => DiscreteAction::DeferToHuman,
            4 => DiscreteAction::TriggerRollback,
            _ => DiscreteAction::Reject,
        }
    }

    pub fn index(&self) -> u32 {
        *self as u32
    }
}

impl QTable {
    /// Create a new Q-table
    pub fn new(num_actions: u32) -> Self {
        Self {
            q_values: HashMap::new(),
            visit_counts: HashMap::new(),
            alpha: 0.1,
            gamma: 0.95,
            epsilon: 1.0,
            epsilon_decay: 0.995,
            epsilon_min: 0.05,
            num_actions,
            episodes: 0,
        }
    }

    /// Hash a continuous state into a discrete bucket
    pub fn hash_state(state: &[f64], resolution: f64) -> u64 {
        let mut h = 5381u64;
        for &v in state {
            let bucket = (v / resolution).round() as i64;
            h = h.wrapping_mul(33).wrapping_add(bucket as u64);
        }
        h
    }

    /// Select action using epsilon-greedy policy
    pub fn select_action(&self, state_hash: u64) -> DiscreteAction {
        // Epsilon-greedy exploration
        let rand_val = self.pseudo_random(state_hash.wrapping_add(self.episodes));
        if rand_val < self.epsilon {
            // Random action
            let action_idx = ((rand_val * 1000.0) as u32) % self.num_actions;
            DiscreteAction::from_index(action_idx)
        } else {
            // Greedy: pick best Q-value
            let mut best_action = 0u32;
            let mut best_q = f64::NEG_INFINITY;
            for a in 0..self.num_actions {
                let q = self.get_q(state_hash, a);
                // UCB1 exploration bonus
                let visits = *self.visit_counts.get(&(state_hash, a)).unwrap_or(&1) as f64;
                let bonus = (2.0 * (self.episodes as f64 + 1.0).ln() / visits).sqrt();
                let ucb = q + 0.1 * bonus;
                if ucb > best_q {
                    best_q = ucb;
                    best_action = a;
                }
            }
            DiscreteAction::from_index(best_action)
        }
    }

    /// Get Q-value for state-action pair
    pub fn get_q(&self, state_hash: u64, action: u32) -> f64 {
        *self.q_values.get(&(state_hash, action)).unwrap_or(&0.0)
    }

    /// Update Q-value using TD learning
    pub fn update(
        &mut self,
        state_hash: u64,
        action: u32,
        reward: f64,
        next_state_hash: u64,
        terminal: bool,
    ) {
        let current_q = self.get_q(state_hash, action);

        // Max Q(s', a') for next state
        let max_next_q = if terminal {
            0.0
        } else {
            (0..self.num_actions)
                .map(|a| self.get_q(next_state_hash, a))
                .fold(f64::NEG_INFINITY, f64::max)
        };

        // TD update
        let target = reward + self.gamma * max_next_q;
        let new_q = current_q + self.alpha * (target - current_q);
        self.q_values.insert((state_hash, action), new_q);

        // Update visit count
        *self.visit_counts.entry((state_hash, action)).or_insert(0) += 1;
    }

    /// End of episode: decay epsilon
    pub fn end_episode(&mut self) {
        self.episodes += 1;
        self.epsilon = (self.epsilon * self.epsilon_decay).max(self.epsilon_min);
    }

    /// Deterministic pseudo-random for reproducibility
    fn pseudo_random(&self, seed: u64) -> f64 {
        let x = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (x >> 33) as f64 / u32::MAX as f64
    }

    /// Get Q-table statistics
    pub fn stats(&self) -> QTableStats {
        let total_entries = self.q_values.len();
        let avg_q = if total_entries > 0 {
            self.q_values.values().sum::<f64>() / total_entries as f64
        } else {
            0.0
        };
        QTableStats {
            episodes: self.episodes,
            epsilon: self.epsilon,
            total_entries,
            avg_q_value: avg_q,
            total_visits: self.visit_counts.values().sum::<u64>(),
        }
    }
}

/// Q-table statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QTableStats {
    pub episodes: u64,
    pub epsilon: f64,
    pub total_entries: usize,
    pub avg_q_value: f64,
    pub total_visits: u64,
}

// =============================================================================
// 5. HumanFeedbackEncoder — Encodes human inputs as structured events
// =============================================================================

/// Encodes human feedback into structured evidence events.
/// Supports validations, corrections, and preferences.
pub struct HumanFeedbackEncoder {
    /// Event counter
    next_id: u64,
    /// Sliding index of recent events (ordered by timestamp)
    event_index: Vec<EvidenceEvent>,
    /// Maximum events to retain
    max_events: usize,
    /// Priority weights for different evidence types
    type_weights: HashMap<EvidenceType, f64>,
}

impl HumanFeedbackEncoder {
    /// Create a new encoder
    pub fn new(max_events: usize) -> Self {
        let mut type_weights = HashMap::new();
        type_weights.insert(EvidenceType::Validation, 1.0);
        type_weights.insert(EvidenceType::Correction, 1.5);
        type_weights.insert(EvidenceType::Preference, 0.8);
        type_weights.insert(EvidenceType::ConsistencyCheck, 0.6);
        type_weights.insert(EvidenceType::ContradictionDetected, 2.0);

        Self {
            next_id: 0,
            event_index: Vec::new(),
            max_events,
            type_weights,
        }
    }

    /// Encode a human validation: "this causal link holds"
    pub fn encode_validation(
        &mut self,
        trait_names: Vec<String>,
        confidence: f64,
        source: &str,
    ) -> EvidenceEvent {
        self.create_event(EvidenceType::Validation, trait_names, confidence, confidence, source)
    }

    /// Encode a human correction: "weaken this correlation"
    pub fn encode_correction(
        &mut self,
        trait_names: Vec<String>,
        correction_magnitude: f64,
        source: &str,
    ) -> EvidenceEvent {
        self.create_event(
            EvidenceType::Correction,
            trait_names,
            -correction_magnitude, // Negative impact = weakening
            0.9, // High confidence in corrections
            source,
        )
    }

    /// Encode a human preference: "prioritize recent events over trusted ones"
    pub fn encode_preference(
        &mut self,
        trait_names: Vec<String>,
        preference_strength: f64,
        source: &str,
        metadata: HashMap<String, String>,
    ) -> EvidenceEvent {
        let mut event = self.create_event(
            EvidenceType::Preference,
            trait_names,
            preference_strength,
            0.7,
            source,
        );
        event.metadata = metadata;
        event
    }

    /// Create and index an evidence event
    fn create_event(
        &mut self,
        evidence_type: EvidenceType,
        trait_names: Vec<String>,
        impact: f64,
        confidence: f64,
        source: &str,
    ) -> EvidenceEvent {
        let timestamp_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let event = EvidenceEvent {
            id: self.next_id,
            timestamp_ms,
            evidence_type,
            trait_names,
            impact,
            confidence,
            source: source.to_string(),
            metadata: HashMap::new(),
        };

        self.next_id += 1;

        // Add to sliding index
        if self.event_index.len() >= self.max_events {
            self.event_index.remove(0);
        }
        self.event_index.push(event.clone());

        event
    }

    /// Get events for a specific trait, sorted by recency
    pub fn events_for_trait(&self, trait_name: &str) -> Vec<&EvidenceEvent> {
        self.event_index.iter()
            .filter(|e| e.trait_names.iter().any(|t| t == trait_name))
            .collect()
    }

    /// Get weighted evidence score for a trait
    pub fn evidence_score(&self, trait_name: &str) -> f64 {
        let events = self.events_for_trait(trait_name);
        if events.is_empty() {
            return 0.0;
        }

        let mut weighted_sum = 0.0;
        let mut weight_total = 0.0;

        for event in &events {
            let type_weight = self.type_weights.get(&event.evidence_type).copied().unwrap_or(1.0);
            let recency_weight = 1.0; // Could decay by age
            let w = type_weight * recency_weight * event.confidence;
            weighted_sum += event.impact * w;
            weight_total += w;
        }

        if weight_total > 0.0 {
            weighted_sum / weight_total
        } else {
            0.0
        }
    }

    /// Get total event count
    pub fn event_count(&self) -> usize {
        self.event_index.len()
    }
}

// =============================================================================
// 6. PathCurator — RL-driven path optimization with long-term metrics
// =============================================================================

/// The Path Curator: uses RL to optimize inference path selection.
/// Rewards stem from long-term metrics like reduced rollback frequency
/// or improved human alignment scores, not immediate task success.
pub struct PathCurator {
    /// Actor-critic for continuous trait delta optimization
    pub policy: ActorCriticPolicy,
    /// Q-table for discrete accept/reject decisions
    pub q_table: QTable,
    /// Human feedback encoder
    pub feedback_encoder: HumanFeedbackEncoder,
    /// Long-term metrics tracker
    pub metrics: RLMetrics,
    /// Mode: Abstract (hypothetical paths) or Reality (empirical)
    pub mode: CurationMode,
}

/// Curation mode determines how evidence is weighted
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CurationMode {
    /// Abstract: exploring hypothetical paths, feedback as simulated evidence
    Abstract,
    /// Reality: grounded in empirical validations
    Reality,
}

impl PathCurator {
    /// Create a new path curator
    pub fn new(state_dim: usize, action_dim: usize) -> Self {
        Self {
            policy: ActorCriticPolicy::new(state_dim, action_dim),
            q_table: QTable::new(5), // 5 discrete actions
            feedback_encoder: HumanFeedbackEncoder::new(5000),
            metrics: RLMetrics::new(),
            mode: CurationMode::Reality,
        }
    }

    /// Propose a trait update action given current state
    pub fn propose_update(&self, state: &[f64]) -> Vec<f64> {
        self.policy.propose_action(state)
    }

    /// Decide whether to accept a proposed update
    pub fn decide_acceptance(&self, state: &[f64]) -> DiscreteAction {
        let state_hash = QTable::hash_state(state, 0.1);
        self.q_table.select_action(state_hash)
    }

    /// Record outcome and train
    pub fn record_and_train(
        &mut self,
        transition: StateTransition,
    ) -> f64 {
        // Record in replay buffer
        self.policy.record_transition(transition.clone());

        // Train actor-critic
        let td_error = self.policy.train_step(&transition);

        // Train Q-table
        let state_hash = QTable::hash_state(&transition.state_before, 0.1);
        let next_hash = QTable::hash_state(&transition.state_after, 0.1);
        let reward = transition.reward.scalar(&RewardWeights::default());
        let action = 0u32; // Accept by default
        self.q_table.update(state_hash, action, reward, next_hash, transition.terminal);

        // Update metrics
        self.metrics.record_reward(reward);
        if reward < 0.0 {
            self.metrics.record_rollback();
        }

        td_error
    }

    /// End of episode
    pub fn end_episode(&mut self) {
        self.q_table.end_episode();
        self.metrics.end_episode();
    }
}

// =============================================================================
// 7. RLMetrics — Long-term metrics (rollback frequency, alignment scores)
// =============================================================================

/// Tracks long-term RL metrics for the path curator
pub struct RLMetrics {
    /// Cumulative reward per episode
    episode_rewards: Vec<f64>,
    /// Current episode reward accumulator
    current_episode_reward: f64,
    /// Rollback count per episode
    episode_rollbacks: Vec<u64>,
    /// Current episode rollback count
    current_rollbacks: u64,
    /// Human alignment scores over time
    alignment_scores: Vec<f64>,
    /// Contradiction rates over time
    contradiction_rates: Vec<f64>,
    /// Total episodes
    total_episodes: u64,
}

impl RLMetrics {
    pub fn new() -> Self {
        Self {
            episode_rewards: Vec::new(),
            current_episode_reward: 0.0,
            episode_rollbacks: Vec::new(),
            current_rollbacks: 0,
            alignment_scores: Vec::new(),
            contradiction_rates: Vec::new(),
            total_episodes: 0,
        }
    }

    /// Record a reward
    pub fn record_reward(&mut self, reward: f64) {
        self.current_episode_reward += reward;
    }

    /// Record a rollback event
    pub fn record_rollback(&mut self) {
        self.current_rollbacks += 1;
    }

    /// Record human alignment score
    pub fn record_alignment(&mut self, score: f64) {
        self.alignment_scores.push(score);
    }

    /// Record contradiction rate
    pub fn record_contradiction_rate(&mut self, rate: f64) {
        self.contradiction_rates.push(rate);
    }

    /// End of episode
    pub fn end_episode(&mut self) {
        self.episode_rewards.push(self.current_episode_reward);
        self.episode_rollbacks.push(self.current_rollbacks);
        self.current_episode_reward = 0.0;
        self.current_rollbacks = 0;
        self.total_episodes += 1;
    }

    /// Average reward over last N episodes
    pub fn avg_reward(&self, last_n: usize) -> f64 {
        let n = self.episode_rewards.len().min(last_n);
        if n == 0 { return 0.0; }
        let start = self.episode_rewards.len() - n;
        self.episode_rewards[start..].iter().sum::<f64>() / n as f64
    }

    /// Rollback frequency (rollbacks per episode)
    pub fn rollback_frequency(&self) -> f64 {
        if self.total_episodes == 0 { return 0.0; }
        self.episode_rollbacks.iter().sum::<u64>() as f64 / self.total_episodes as f64
    }

    /// Latest alignment score
    pub fn latest_alignment(&self) -> f64 {
        self.alignment_scores.last().copied().unwrap_or(0.0)
    }

    /// Summary statistics
    pub fn summary(&self) -> RLMetricsSummary {
        RLMetricsSummary {
            total_episodes: self.total_episodes,
            avg_reward_last_100: self.avg_reward(100),
            rollback_frequency: self.rollback_frequency(),
            latest_alignment: self.latest_alignment(),
            latest_contradiction_rate: self.contradiction_rates.last().copied().unwrap_or(0.0),
        }
    }
}

/// Summary of RL metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RLMetricsSummary {
    pub total_episodes: u64,
    pub avg_reward_last_100: f64,
    pub rollback_frequency: f64,
    pub latest_alignment: f64,
    pub latest_contradiction_rate: f64,
}
