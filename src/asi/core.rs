//! ASI Core - The Autonomous Intelligence Loop
//!
//! This is the heart of the ASI system - a continuous loop that:
//! 1. Perceives the environment through sensors
//! 2. Thinks using flux reasoning and consciousness simulation
//! 3. Acts through tools and actuators
//! 4. Learns from outcomes and improves itself
//!
//! Unlike traditional chatbots that wait for input, ASICore runs
//! continuously, pursuing its own goals while remaining aligned
//! with human values.

use crate::ai::flux_reasoning::FluxReasoningChain;
use crate::ai::causal_reasoning::CausalWorldModel;
use crate::ai::goal_planner::GoalPlanner;
use crate::ai::curiosity_engine::CuriosityEngine;
use crate::ai::working_memory::{ContextWindow, MemoryContent, MemorySource};
use crate::ai::transfer_learning::TransferLearningEngine;
use crate::ai::tools::ToolRegistry;
use crate::consciousness::ConsciousnessSimulator;
use crate::data::models::ELPTensor;

use super::world_interface::{Sensor, Actuator, Observation, ActionResult};
use super::world_interface::Action as WorldAction;
use super::identity::PersistentIdentity;
use super::goal_manager::{GoalManager, AutonomousGoal, GoalPriority};
use super::self_modification::SelfModificationEngine;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, interval};
use uuid::Uuid;

// ============================================================================
// Configuration
// ============================================================================

/// Configuration for ASI Core
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ASIConfig {
    /// How often to run the main loop (milliseconds)
    pub loop_interval_ms: u64,
    
    /// Maximum reasoning steps per cycle
    pub max_reasoning_steps: usize,
    
    /// Enable autonomous goal pursuit
    pub autonomous_goals: bool,
    
    /// Enable self-modification
    pub self_modification: bool,
    
    /// Path to persistent storage
    pub storage_path: PathBuf,
    
    /// Maximum concurrent actions
    pub max_concurrent_actions: usize,
    
    /// Safety constraints
    pub safety: SafetyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyConfig {
    /// Require human approval for destructive actions
    pub require_approval_for_destructive: bool,
    
    /// Maximum cost per action (in abstract units)
    pub max_action_cost: f64,
    
    /// Forbidden action patterns (regex)
    pub forbidden_patterns: Vec<String>,
    
    /// Rate limit for external actions
    pub actions_per_minute: u32,
}

impl Default for ASIConfig {
    fn default() -> Self {
        Self {
            loop_interval_ms: 1000,  // 1 second
            max_reasoning_steps: 20,
            autonomous_goals: true,
            self_modification: false,  // Disabled by default for safety
            storage_path: PathBuf::from("./asi_data"),
            max_concurrent_actions: 5,
            safety: SafetyConfig {
                require_approval_for_destructive: true,
                max_action_cost: 100.0,
                forbidden_patterns: vec![
                    r"rm\s+-rf\s+/".to_string(),
                    r"sudo\s+".to_string(),
                    r"format\s+".to_string(),
                ],
                actions_per_minute: 60,
            },
        }
    }
}

// ============================================================================
// State
// ============================================================================

/// Current state of the ASI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ASIState {
    pub mode: ASIMode,
    pub current_goal: Option<Uuid>,
    pub cycle_count: u64,
    pub last_action: Option<DateTime<Utc>>,
    pub confidence: f32,
    pub phi: f64,  // Consciousness level (IIT)
    pub elp_state: ELPTensor,
}

/// Operating modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ASIMode {
    /// Waiting for input (passive)
    Idle,
    /// Perceiving environment
    Perceiving,
    /// Reasoning about observations
    Thinking,
    /// Executing actions
    Acting,
    /// Learning from outcomes
    Learning,
    /// Self-improvement cycle
    Improving,
    /// Paused by human
    Paused,
    /// Emergency stop
    Halted,
}

// ============================================================================
// Core
// ============================================================================

/// The Autonomous Superintelligent Core
pub struct ASICore {
    // Configuration
    pub config: ASIConfig,
    
    // State
    pub state: Arc<RwLock<ASIState>>,
    
    // Perception
    sensors: Vec<Box<dyn Sensor>>,
    observation_buffer: Arc<RwLock<VecDeque<Observation>>>,
    
    // Cognition
    consciousness: Arc<RwLock<ConsciousnessSimulator>>,
    reasoning: Arc<RwLock<FluxReasoningChain>>,
    causal_model: Arc<RwLock<CausalWorldModel>>,
    working_memory: Arc<RwLock<ContextWindow>>,
    transfer_learning: Arc<RwLock<TransferLearningEngine>>,
    
    // Goals
    pub goal_manager: Arc<RwLock<GoalManager>>,
    goal_planner: Arc<RwLock<GoalPlanner>>,
    curiosity: Arc<RwLock<CuriosityEngine>>,
    
    // Action
    tools: Arc<RwLock<ToolRegistry>>,
    actuators: Vec<Box<dyn Actuator>>,
    action_history: Arc<RwLock<VecDeque<ActionResult>>>,
    
    // Identity & Memory
    identity: Arc<RwLock<PersistentIdentity>>,
    
    // Self-Improvement
    self_mod: Option<Arc<RwLock<SelfModificationEngine>>>,
    
    // Control
    running: Arc<AtomicBool>,
    
    // Statistics
    stats: Arc<RwLock<ASIStats>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ASIStats {
    pub total_cycles: u64,
    pub observations_processed: u64,
    pub thoughts_generated: u64,
    pub actions_taken: u64,
    pub goals_completed: u64,
    pub improvements_made: u64,
    pub errors_encountered: u64,
    pub uptime_seconds: u64,
}

impl ASICore {
    /// Create a new ASI Core
    pub async fn new(config: ASIConfig) -> Result<Self> {
        // Ensure storage directory exists
        std::fs::create_dir_all(&config.storage_path)?;
        
        // Initialize consciousness
        let consciousness = ConsciousnessSimulator::new(true).await;
        
        // Initialize or load identity
        let identity_path = config.storage_path.join("identity.json");
        let identity = if identity_path.exists() {
            PersistentIdentity::load(&identity_path)?
        } else {
            PersistentIdentity::new("SpatialVortex ASI", &config.storage_path)
        };
        
        // Initialize self-modification if enabled
        let self_mod = if config.self_modification {
            Some(Arc::new(RwLock::new(
                SelfModificationEngine::new(PathBuf::from("./src"))
            )))
        } else {
            None
        };
        
        let initial_state = ASIState {
            mode: ASIMode::Idle,
            current_goal: None,
            cycle_count: 0,
            last_action: None,
            confidence: 0.5,
            phi: 0.0,
            elp_state: ELPTensor::default(),
        };
        
        Ok(Self {
            config,
            state: Arc::new(RwLock::new(initial_state)),
            sensors: Vec::new(),
            observation_buffer: Arc::new(RwLock::new(VecDeque::with_capacity(100))),
            consciousness: Arc::new(RwLock::new(consciousness)),
            reasoning: Arc::new(RwLock::new(FluxReasoningChain::new("ASI initialization"))),
            causal_model: Arc::new(RwLock::new(CausalWorldModel::new())),
            working_memory: Arc::new(RwLock::new(ContextWindow::new(9))),  // Sacred 9
            transfer_learning: Arc::new(RwLock::new(TransferLearningEngine::new())),
            goal_manager: Arc::new(RwLock::new(GoalManager::new())),
            goal_planner: Arc::new(RwLock::new(GoalPlanner::new())),
            curiosity: Arc::new(RwLock::new(CuriosityEngine::new())),
            tools: Arc::new(RwLock::new(ToolRegistry::new())),
            actuators: Vec::new(),
            action_history: Arc::new(RwLock::new(VecDeque::with_capacity(100))),
            identity: Arc::new(RwLock::new(identity)),
            self_mod,
            running: Arc::new(AtomicBool::new(false)),
            stats: Arc::new(RwLock::new(ASIStats::default())),
        })
    }
    
    /// Add a sensor for perception
    pub fn add_sensor(&mut self, sensor: Box<dyn Sensor>) {
        self.sensors.push(sensor);
    }
    
    /// Add an actuator for action
    pub fn add_actuator(&mut self, actuator: Box<dyn Actuator>) {
        self.actuators.push(actuator);
    }
    
    /// Start the autonomous loop
    pub async fn start(&self) -> Result<()> {
        self.running.store(true, Ordering::SeqCst);
        
        tracing::info!("🧠 ASI Core starting autonomous loop...");
        
        let mut interval = interval(Duration::from_millis(self.config.loop_interval_ms));
        
        while self.running.load(Ordering::SeqCst) {
            interval.tick().await;
            
            // Check if paused or halted
            {
                let state = self.state.read().await;
                if state.mode == ASIMode::Paused || state.mode == ASIMode::Halted {
                    continue;
                }
            }
            
            // Run one cycle
            if let Err(e) = self.run_cycle().await {
                tracing::error!("ASI cycle error: {}", e);
                let mut stats = self.stats.write().await;
                stats.errors_encountered += 1;
            }
        }
        
        tracing::info!("🧠 ASI Core stopped");
        Ok(())
    }
    
    /// Stop the autonomous loop
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
    
    /// Pause the autonomous loop (can be resumed)
    pub async fn pause(&self) {
        let mut state = self.state.write().await;
        state.mode = ASIMode::Paused;
    }
    
    /// Resume from pause
    pub async fn resume(&self) {
        let mut state = self.state.write().await;
        if state.mode == ASIMode::Paused {
            state.mode = ASIMode::Idle;
        }
    }
    
    /// Emergency halt (requires restart)
    pub async fn halt(&self) {
        let mut state = self.state.write().await;
        state.mode = ASIMode::Halted;
        self.stop();
    }
    
    /// Run a single cycle of the autonomous loop
    async fn run_cycle(&self) -> Result<()> {
        let cycle_start = std::time::Instant::now();
        
        // Update cycle count
        {
            let mut state = self.state.write().await;
            state.cycle_count += 1;
        }
        
        // 1. PERCEIVE - Gather observations from sensors
        self.set_mode(ASIMode::Perceiving).await;
        let observations = self.perceive().await?;
        
        // 2. THINK - Process observations and reason
        self.set_mode(ASIMode::Thinking).await;
        let thoughts = self.think(&observations).await?;
        
        // 3. DECIDE - Select goal and plan
        let action_plan = self.decide(&thoughts).await?;
        
        // 4. ACT - Execute planned actions
        if !action_plan.is_empty() {
            self.set_mode(ASIMode::Acting).await;
            let results = self.act(action_plan).await?;
            
            // 5. LEARN - Update models from outcomes
            self.set_mode(ASIMode::Learning).await;
            self.learn(&results).await?;
        }
        
        // 6. IMPROVE - Periodically self-improve
        if self.should_self_improve().await {
            self.set_mode(ASIMode::Improving).await;
            self.improve().await?;
        }
        
        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.total_cycles += 1;
            stats.uptime_seconds += cycle_start.elapsed().as_secs();
        }
        
        // Return to idle
        self.set_mode(ASIMode::Idle).await;
        
        Ok(())
    }
    
    /// Perceive environment through sensors
    async fn perceive(&self) -> Result<Vec<Observation>> {
        let mut observations = Vec::new();
        
        for sensor in &self.sensors {
            match sensor.perceive().await {
                Ok(obs) => observations.extend(obs),
                Err(e) => tracing::warn!("Sensor error: {}", e),
            }
        }
        
        // Store in buffer
        {
            let mut buffer = self.observation_buffer.write().await;
            for obs in &observations {
                buffer.push_back(obs.clone());
                if buffer.len() > 100 {
                    buffer.pop_front();
                }
            }
        }
        
        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.observations_processed += observations.len() as u64;
        }
        
        Ok(observations)
    }
    
    /// Think about observations using consciousness and reasoning
    async fn think(&self, observations: &[Observation]) -> Result<Vec<Thought>> {
        let mut thoughts = Vec::new();
        
        // Store observations in working memory
        {
            let mut wm = self.working_memory.write().await;
            let elp = ELPTensor { ethos: 5.0, logos: 7.0, pathos: 4.0 };
            
            for obs in observations {
                wm.store_in_context(
                    MemoryContent::Text(obs.content.clone()),
                    &elp,
                    MemorySource::InternalReasoning,
                );
            }
        }
        
        // Process through consciousness simulator
        {
            let consciousness = self.consciousness.read().await;
            // Consciousness processes observations and generates thoughts
            // This integrates ELP perspectives (Ethos, Logos, Pathos)
        }
        
        // Run flux reasoning if we have significant observations
        if !observations.is_empty() {
            let query = observations.iter()
                .map(|o| o.content.as_str())
                .collect::<Vec<_>>()
                .join("; ");
            
            let mut reasoning = self.reasoning.write().await;
            *reasoning = FluxReasoningChain::new(&query);
            
            if let Ok(final_thought) = reasoning.reason(self.config.max_reasoning_steps).await {
                thoughts.push(Thought {
                    content: final_thought.reasoning_trace.clone(),
                    confidence: final_thought.certainty,
                    elp: final_thought.elp_state.clone(),
                    vortex_position: final_thought.vortex_position,
                });
            }
        }
        
        // Check curiosity - identify knowledge gaps
        {
            let mut curiosity = self.curiosity.write().await;
            if curiosity.should_explore() {
                if let Some(gap) = curiosity.get_most_curious() {
                    thoughts.push(Thought {
                        content: format!("Knowledge gap: {}", gap.description),
                        confidence: 0.5,
                        elp: gap.elp_relevance.clone(),
                        vortex_position: 3,  // Sacred position for exploration
                    });
                }
            }
        }
        
        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.thoughts_generated += thoughts.len() as u64;
        }
        
        Ok(thoughts)
    }
    
    /// Decide on actions based on thoughts
    async fn decide(&self, thoughts: &[Thought]) -> Result<Vec<Action>> {
        let mut actions = Vec::new();
        
        // Check if we have an active goal
        let current_goal = {
            let gm = self.goal_manager.read().await;
            gm.get_current_goal()
        };
        
        if let Some(goal) = current_goal {
            // Plan actions toward goal
            let mut planner = self.goal_planner.write().await;
            
            // Create ELP from goal
            let elp = ELPTensor {
                ethos: goal.importance as f64 * 10.0,
                logos: 7.0,
                pathos: 5.0,
            };
            
            let plan_goal = planner.create_goal(&goal.objective, &elp);
            planner.add_goal(plan_goal);
            
            if let Ok(Some(plan)) = planner.plan_next_goal() {
                for task in plan.tasks {
                    actions.push(Action {
                        id: Uuid::new_v4(),
                        action_type: ActionType::ExecuteTask,
                        target: task.task.name,
                        parameters: serde_json::json!({}),
                        priority: goal.priority as u8,
                    });
                }
            }
        } else if self.config.autonomous_goals {
            // Select new goal based on curiosity and thoughts
            let mut gm = self.goal_manager.write().await;
            
            // Check for high-priority thoughts that should become goals
            for thought in thoughts {
                if thought.confidence > 0.7 && thought.content.contains("should") {
                    let goal = AutonomousGoal {
                        id: Uuid::new_v4(),
                        objective: thought.content.clone(),
                        priority: GoalPriority::Medium,
                        importance: thought.confidence,
                        created_at: Utc::now(),
                        deadline: None,
                        status: super::goal_manager::GoalStatus::Pending,
                        progress: 0.0,
                    };
                    gm.add_goal(goal);
                }
            }
            
            // Activate highest priority goal
            gm.activate_next_goal();
        }
        
        Ok(actions)
    }
    
    /// Execute planned actions
    async fn act(&self, actions: Vec<Action>) -> Result<Vec<ActionResult>> {
        let mut results = Vec::new();
        
        for action in actions {
            // Safety check
            if !self.is_action_safe(&action).await {
                tracing::warn!("Action blocked by safety: {:?}", action);
                continue;
            }
            
            // Convert to world action
            let world_action = WorldAction {
                id: action.id,
                action_type: format!("{:?}", action.action_type),
                target: action.target.clone(),
                parameters: action.parameters.clone(),
            };
            
            // Execute through appropriate actuator
            for actuator in &self.actuators {
                if actuator.can_handle(&world_action) {
                    match actuator.act(world_action.clone()).await {
                        Ok(result) => {
                            results.push(result.clone());
                            
                            // Store in history
                            let mut history = self.action_history.write().await;
                            history.push_back(result);
                            if history.len() > 100 {
                                history.pop_front();
                            }
                        }
                        Err(e) => {
                            tracing::error!("Action failed: {}", e);
                        }
                    }
                    break;
                }
            }
        }
        
        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.actions_taken += results.len() as u64;
        }
        
        Ok(results)
    }
    
    /// Learn from action outcomes
    async fn learn(&self, results: &[ActionResult]) -> Result<()> {
        // Update causal model
        {
            let mut causal = self.causal_model.write().await;
            let elp = ELPTensor { ethos: 5.0, logos: 8.0, pathos: 4.0 };
            
            for result in results {
                if result.success {
                    causal.learn_from_observation(
                        &result.action_type,
                        &result.outcome,
                        0.8,
                        &elp,
                    );
                }
            }
        }
        
        // Update goal progress
        {
            let mut gm = self.goal_manager.write().await;
            for result in results {
                if result.success {
                    gm.update_progress(result.goal_id, 0.1);
                }
            }
        }
        
        // Store in episodic memory
        {
            let mut identity = self.identity.write().await;
            for result in results {
                identity.record_experience(
                    &result.action_type,
                    &result.outcome,
                    result.success,
                );
            }
        }
        
        // Persist identity
        {
            let identity = self.identity.read().await;
            let path = self.config.storage_path.join("identity.json");
            identity.save(&path)?;
        }
        
        Ok(())
    }
    
    /// Check if self-improvement should run
    async fn should_self_improve(&self) -> bool {
        if !self.config.self_modification {
            return false;
        }
        
        let state = self.state.read().await;
        
        // Improve every 100 cycles
        state.cycle_count % 100 == 0
    }
    
    /// Run self-improvement cycle
    async fn improve(&self) -> Result<()> {
        if let Some(ref self_mod) = self.self_mod {
            let mut engine = self_mod.write().await;
            
            // Analyze performance
            let stats = self.stats.read().await;
            
            // Identify weaknesses
            let error_rate = stats.errors_encountered as f64 / stats.total_cycles.max(1) as f64;
            
            if error_rate > 0.1 {
                // High error rate - propose improvement
                if let Ok(proposal) = engine.propose_improvement("high_error_rate") {
                    tracing::info!("Self-improvement proposal: {}", proposal.description);
                    
                    // Test in sandbox
                    if let Ok(test_result) = engine.test_proposal(&proposal).await {
                        if test_result.passed {
                            // Apply improvement
                            if let Ok(()) = engine.apply_proposal(&proposal).await {
                                let mut stats = self.stats.write().await;
                                stats.improvements_made += 1;
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Check if an action is safe to execute
    async fn is_action_safe(&self, action: &Action) -> bool {
        let safety = &self.config.safety;
        
        // Check forbidden patterns
        for pattern in &safety.forbidden_patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if re.is_match(&action.target) {
                    return false;
                }
            }
        }
        
        // Check cost
        if action.estimated_cost() > safety.max_action_cost {
            return false;
        }
        
        // Check rate limit
        let history = self.action_history.read().await;
        let recent_count = history.iter()
            .filter(|r| r.timestamp > Utc::now() - chrono::Duration::minutes(1))
            .count();
        
        if recent_count >= safety.actions_per_minute as usize {
            return false;
        }
        
        true
    }
    
    /// Set operating mode
    async fn set_mode(&self, mode: ASIMode) {
        let mut state = self.state.write().await;
        state.mode = mode;
    }
    
    /// Get current state
    pub async fn get_state(&self) -> ASIState {
        self.state.read().await.clone()
    }
    
    /// Get statistics
    pub async fn get_stats(&self) -> ASIStats {
        self.stats.read().await.clone()
    }
    
    /// Process a human query (interactive mode)
    pub async fn process_query(&self, query: &str) -> Result<String> {
        // Store query in working memory
        {
            let mut wm = self.working_memory.write().await;
            let elp = ELPTensor { ethos: 6.0, logos: 7.0, pathos: 5.0 };
            wm.store_in_context(
                MemoryContent::Text(query.to_string()),
                &elp,
                MemorySource::UserInput,
            );
        }
        
        // Run reasoning
        let mut reasoning = self.reasoning.write().await;
        *reasoning = FluxReasoningChain::new(query);
        
        let final_thought = reasoning.reason(self.config.max_reasoning_steps).await?;
        let answer = reasoning.synthesize_final_answer().await?;
        
        // Store response in memory
        {
            let mut identity = self.identity.write().await;
            identity.record_interaction(query, &answer);
        }
        
        Ok(answer)
    }
}

// ============================================================================
// Supporting Types
// ============================================================================

/// A thought generated by the reasoning system
#[derive(Debug, Clone)]
pub struct Thought {
    pub content: String,
    pub confidence: f32,
    pub elp: ELPTensor,
    pub vortex_position: u8,
}

/// Type of action to take
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    ExecuteTask,
    QueryKnowledge,
    CreateGoal,
    ModifyCode,
    Communicate,
    Observe,
}

/// An action to execute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub id: Uuid,
    pub action_type: ActionType,
    pub target: String,
    pub parameters: serde_json::Value,
    pub priority: u8,
}

impl Action {
    pub fn estimated_cost(&self) -> f64 {
        match self.action_type {
            ActionType::ExecuteTask => 10.0,
            ActionType::QueryKnowledge => 1.0,
            ActionType::CreateGoal => 2.0,
            ActionType::ModifyCode => 50.0,
            ActionType::Communicate => 5.0,
            ActionType::Observe => 0.5,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_asi_core_creation() {
        let config = ASIConfig::default();
        let core = ASICore::new(config).await;
        assert!(core.is_ok());
    }
    
    #[tokio::test]
    async fn test_asi_state() {
        let config = ASIConfig::default();
        let core = ASICore::new(config).await.unwrap();
        
        let state = core.get_state().await;
        assert_eq!(state.mode, ASIMode::Idle);
        assert_eq!(state.cycle_count, 0);
    }
    
    #[tokio::test]
    async fn test_safety_check() {
        let config = ASIConfig::default();
        let core = ASICore::new(config).await.unwrap();
        
        // Safe action
        let safe_action = Action {
            id: Uuid::new_v4(),
            action_type: ActionType::QueryKnowledge,
            target: "test query".to_string(),
            parameters: serde_json::json!({}),
            priority: 5,
        };
        assert!(core.is_action_safe(&safe_action).await);
        
        // Unsafe action (forbidden pattern)
        let unsafe_action = Action {
            id: Uuid::new_v4(),
            action_type: ActionType::ExecuteTask,
            target: "rm -rf /".to_string(),
            parameters: serde_json::json!({}),
            priority: 5,
        };
        assert!(!core.is_action_safe(&unsafe_action).await);
    }
}
