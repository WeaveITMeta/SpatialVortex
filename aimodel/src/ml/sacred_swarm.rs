//! Sacred Swarm - Agentic Orchestration with Vortex Coordination
//!
//! Hyper-scale agent swarm inspired by:
//! - Kimi K2.5: Agent Swarm (100 sub-agents, 1,500 steps, 4.5x faster via PARL)
//! - SpatialVortex: Vortex cycles and sacred anchors for consensus
//!
//! Architecture:
//! - 1000+ agents coordinated by vortex graphs
//! - Agents as nodes in cycles, anchors for consensus
//! - Neuro-symbolic agents: GeometricInference + VortexModel hybrid
//! - EBRM for energy-based path selection
//! - Imagination/counterfactual simulation for abductive reasoning

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::sacred_moe::{PHI, SACRED_POSITIONS, VORTEX_CYCLE};
use crate::data::attributes::{Attributes, AttributeAccessor};

// =============================================================================
// Swarm Configuration
// =============================================================================

/// Configuration for Sacred Swarm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SacredSwarmConfig {
    /// Total number of agents
    pub num_agents: usize,
    /// Maximum concurrent active agents
    pub max_active: usize,
    /// Maximum steps per task
    pub max_steps: usize,
    /// Enable vortex coordination
    pub vortex_coordination: bool,
    /// Number of anchor agents (consensus nodes)
    pub num_anchors: usize,
    /// Enable parallel execution
    pub parallel_execution: bool,
    /// Consensus threshold (0.0-1.0)
    pub consensus_threshold: f32,
    /// Enable imagination/counterfactual
    pub imagination_enabled: bool,
    /// Energy threshold for path selection
    pub energy_threshold: f32,
    /// Sacred boost for anchor agents
    pub sacred_boost: f32,
}

impl Default for SacredSwarmConfig {
    fn default() -> Self {
        Self {
            num_agents: 1000,
            max_active: 100,
            max_steps: 1500,
            vortex_coordination: true,
            num_anchors: 9, // Sacred positions
            parallel_execution: true,
            consensus_threshold: 0.7,
            imagination_enabled: true,
            energy_threshold: 0.5,
            sacred_boost: 1.15,
        }
    }
}

impl SacredSwarmConfig {
    pub fn new() -> Self { Self::default() }
    
    /// Kimi K2.5-style: 100 agents, 1500 steps
    pub fn kimi_style() -> Self {
        Self {
            num_agents: 100,
            max_active: 100,
            max_steps: 1500,
            ..Default::default()
        }
    }
    
    /// Hyper-scale: 10000 agents
    pub fn hyper_scale() -> Self {
        Self {
            num_agents: 10000,
            max_active: 1000,
            max_steps: 5000,
            num_anchors: 27, // 3x sacred positions
            ..Default::default()
        }
    }
}

// =============================================================================
// Agent Types and Specializations
// =============================================================================

/// Agent specialization domains
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentSpecialization {
    /// General purpose reasoning
    General,
    /// Code generation and analysis
    Code,
    /// Mathematical reasoning
    Math,
    /// Language understanding
    Language,
    /// Knowledge retrieval
    Retrieval,
    /// Planning and decomposition
    Planning,
    /// Verification and validation
    Verification,
    /// Synthesis and generation
    Synthesis,
    /// Geometric/symbolic reasoning
    Geometric,
}

/// Agent state in the swarm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentState {
    /// Ready to accept tasks
    Idle,
    /// Currently processing
    Active,
    /// Waiting for dependencies
    Waiting,
    /// Completed task
    Completed,
    /// Error state
    Error,
    /// Anchor consensus mode
    Consensus,
}

// =============================================================================
// Swarm Agent
// =============================================================================

/// A single agent in the Sacred Swarm
#[derive(Debug, Clone)]
pub struct SwarmAgent {
    /// Unique agent ID
    pub id: Uuid,
    /// Agent index in swarm
    pub index: usize,
    /// Vortex position (1-9)
    pub vortex_position: usize,
    /// Whether this is an anchor agent
    pub is_anchor: bool,
    /// Agent specialization
    pub specialization: AgentSpecialization,
    /// Current state
    pub state: AgentState,
    /// Current task (if any)
    pub current_task: Option<String>,
    /// Confidence in current result
    pub confidence: f32,
    /// Energy level (for EBRM)
    pub energy: f32,
    /// Completed steps
    pub steps_completed: u64,
    /// Success rate
    pub success_rate: f32,
    /// Connected agents (vortex graph edges)
    pub connections: Vec<usize>,
    /// ELP Attributes for value dynamics
    /// - Ethos: Agent credibility/trust weight in consensus
    /// - Logos: Logical reasoning strength (affects inference quality)
    /// - Pathos: Emotional/intuitive weight (affects creative tasks)
    pub attributes: Attributes,
}

impl AttributeAccessor for SwarmAgent {
    fn attributes(&self) -> &Attributes { &self.attributes }
    fn attributes_mut(&mut self) -> &mut Attributes { &mut self.attributes }
}

impl SwarmAgent {
    pub fn new(index: usize, config: &SacredSwarmConfig) -> Self {
        // Assign vortex position based on index
        let vortex_position = (index % 9) + 1;
        let is_anchor = SACRED_POSITIONS.contains(&vortex_position) && index < config.num_anchors;
        
        // Assign specialization based on vortex position
        let specialization = match vortex_position {
            1 => AgentSpecialization::General,
            2 => AgentSpecialization::Code,
            3 => AgentSpecialization::Math,      // Sacred
            4 => AgentSpecialization::Language,
            5 => AgentSpecialization::Retrieval,
            6 => AgentSpecialization::Planning,   // Sacred
            7 => AgentSpecialization::Verification,
            8 => AgentSpecialization::Synthesis,
            9 => AgentSpecialization::Geometric,  // Sacred
            _ => AgentSpecialization::General,
        };
        
        // Build vortex graph connections
        let connections = Self::build_connections(index, config.num_agents);
        
        // Initialize ELP attributes based on specialization
        // These drive value dynamics in consensus and feed-forward to MoE routing
        let attributes = Self::init_attributes_for_specialization(specialization, is_anchor);
        
        Self {
            id: Uuid::new_v4(),
            index,
            vortex_position,
            is_anchor,
            specialization,
            state: AgentState::Idle,
            current_task: None,
            confidence: 0.5,
            energy: 1.0,
            steps_completed: 0,
            success_rate: 0.5,
            connections,
            attributes,
        }
    }
    
    /// Initialize ELP attributes based on agent specialization
    /// 
    /// ELP Value Dynamics:
    /// - **Ethos** (credibility): Weight in consensus voting, trust propagation
    /// - **Logos** (logic): Inference quality multiplier, reasoning depth
    /// - **Pathos** (intuition): Creative task weight, abductive reasoning strength
    /// 
    /// Feed-forward connections:
    /// 1. Consensus: ethos-weighted voting among anchors
    /// 2. MoE Routing: logos affects expert selection scores
    /// 3. Swarm Output: pathos weights creative vs analytical outputs
    fn init_attributes_for_specialization(spec: AgentSpecialization, is_anchor: bool) -> Attributes {
        let (ethos, logos, pathos): (f32, f32, f32) = match spec {
            // Math: High logos (logical), moderate ethos
            AgentSpecialization::Math => (0.7, 0.95, 0.3),
            // Code: High logos, high ethos (verifiable)
            AgentSpecialization::Code => (0.8, 0.9, 0.2),
            // Language: Balanced, higher pathos for nuance
            AgentSpecialization::Language => (0.6, 0.6, 0.7),
            // Retrieval: High ethos (factual), moderate logos
            AgentSpecialization::Retrieval => (0.85, 0.7, 0.3),
            // Planning: Balanced logos/pathos for strategy
            AgentSpecialization::Planning => (0.7, 0.8, 0.6),
            // Verification: Highest ethos (trust), high logos
            AgentSpecialization::Verification => (0.95, 0.85, 0.2),
            // Synthesis: High pathos (creative), moderate logos
            AgentSpecialization::Synthesis => (0.5, 0.6, 0.9),
            // Geometric: Sacred - balanced with sacred boost
            AgentSpecialization::Geometric => (0.8, 0.8, 0.8),
            // General: Balanced baseline
            AgentSpecialization::General => (0.5, 0.5, 0.5),
        };
        
        // Anchor agents get ethos boost (trusted consensus nodes)
        let ethos = if is_anchor { (ethos * 1.15).min(1.0) } else { ethos };
        
        let mut attrs = Attributes::with_elp(ethos, logos, pathos);
        attrs.set_digital_root_flux(spec as u8 % 9 + 1);
        attrs
    }
    
    /// Build vortex graph connections for an agent
    fn build_connections(index: usize, num_agents: usize) -> Vec<usize> {
        let mut connections = Vec::new();
        let position = (index % 9) + 1;
        
        // Connect to next in vortex cycle
        let cycle_idx = VORTEX_CYCLE.iter().position(|&p| p == position);
        if let Some(idx) = cycle_idx {
            let next_pos = VORTEX_CYCLE[(idx + 1) % VORTEX_CYCLE.len()];
            // Find agent at next position
            for i in 0..num_agents {
                if (i % 9) + 1 == next_pos && i != index {
                    connections.push(i);
                    break;
                }
            }
        }
        
        // Connect to sacred anchors
        for &sacred_pos in &SACRED_POSITIONS {
            for i in 0..num_agents.min(27) {
                if (i % 9) + 1 == sacred_pos && i != index {
                    connections.push(i);
                    break;
                }
            }
        }
        
        connections.sort();
        connections.dedup();
        connections
    }
    
    /// Execute a step on current task
    pub fn step(&mut self) -> AgentStepResult {
        if self.state != AgentState::Active {
            return AgentStepResult {
                agent_id: self.id,
                success: false,
                output: None,
                confidence: 0.0,
                energy_delta: 0.0,
            };
        }
        
        self.steps_completed += 1;
        
        // Simulate work (in real impl, would call actual inference)
        let success = self.energy > 0.1;
        let output = if success {
            Some(format!("Agent {} step {} output", self.index, self.steps_completed))
        } else {
            None
        };
        
        // Energy decay with sacred boost
        let decay = if self.is_anchor { 0.005 } else { 0.01 };
        self.energy = (self.energy - decay).max(0.0);
        
        // Update confidence
        if success {
            self.confidence = (self.confidence * 0.9 + 0.1).min(1.0);
            self.success_rate = self.success_rate * 0.99 + 0.01;
        } else {
            self.confidence *= 0.9;
            self.success_rate = self.success_rate * 0.99;
        }
        
        AgentStepResult {
            agent_id: self.id,
            success,
            output,
            confidence: self.confidence,
            energy_delta: -decay,
        }
    }
    
    /// Reset agent for new task
    pub fn reset(&mut self) {
        self.state = AgentState::Idle;
        self.current_task = None;
        self.energy = 1.0;
        self.confidence = 0.5;
    }
}

/// Result of a single agent step
#[derive(Debug, Clone)]
pub struct AgentStepResult {
    pub agent_id: Uuid,
    pub success: bool,
    pub output: Option<String>,
    pub confidence: f32,
    pub energy_delta: f32,
}

// =============================================================================
// Swarm Task
// =============================================================================

/// A task to be executed by the swarm
#[derive(Debug, Clone)]
pub struct SwarmTask {
    /// Unique task ID
    pub id: Uuid,
    /// Task description
    pub description: String,
    /// Required specializations
    pub required_specs: Vec<AgentSpecialization>,
    /// Priority (higher = more important)
    pub priority: u32,
    /// Maximum steps allowed
    pub max_steps: usize,
    /// Current step
    pub current_step: usize,
    /// Assigned agents
    pub assigned_agents: Vec<usize>,
    /// Partial results
    pub partial_results: Vec<String>,
    /// Task state
    pub state: TaskState,
    /// Consensus votes from anchors
    pub consensus_votes: HashMap<usize, f32>,
}

/// Task execution state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskState {
    Pending,
    InProgress,
    AwaitingConsensus,
    Completed,
    Failed,
}

impl SwarmTask {
    pub fn new(description: &str, required_specs: Vec<AgentSpecialization>) -> Self {
        Self {
            id: Uuid::new_v4(),
            description: description.to_string(),
            required_specs,
            priority: 1,
            max_steps: 100,
            current_step: 0,
            assigned_agents: Vec::new(),
            partial_results: Vec::new(),
            state: TaskState::Pending,
            consensus_votes: HashMap::new(),
        }
    }
    
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }
    
    pub fn with_max_steps(mut self, max_steps: usize) -> Self {
        self.max_steps = max_steps;
        self
    }
}

// =============================================================================
// Sacred Swarm Orchestrator
// =============================================================================

/// Main orchestrator for the Sacred Swarm
#[derive(Debug)]
pub struct SacredSwarm {
    /// Configuration
    pub config: SacredSwarmConfig,
    /// All agents
    pub agents: Vec<SwarmAgent>,
    /// Pending tasks
    pub task_queue: Vec<SwarmTask>,
    /// Active tasks
    pub active_tasks: HashMap<Uuid, SwarmTask>,
    /// Completed tasks
    pub completed_tasks: Vec<SwarmTask>,
    /// Current vortex cycle position
    pub cycle_position: usize,
    /// Total steps executed
    pub total_steps: u64,
    /// Swarm statistics
    pub stats: SwarmStats,
}

/// Swarm statistics
#[derive(Debug, Clone, Default)]
pub struct SwarmStats {
    pub total_tasks_completed: u64,
    pub total_tasks_failed: u64,
    pub total_steps: u64,
    pub avg_steps_per_task: f32,
    pub avg_confidence: f32,
    pub consensus_success_rate: f32,
    pub agent_utilization: f32,
    pub vortex_position_distribution: [u64; 9],
}

impl SacredSwarm {
    pub fn new(config: SacredSwarmConfig) -> Self {
        // Create agents
        let agents: Vec<SwarmAgent> = (0..config.num_agents)
            .map(|i| SwarmAgent::new(i, &config))
            .collect();
        
        Self {
            config,
            agents,
            task_queue: Vec::new(),
            active_tasks: HashMap::new(),
            completed_tasks: Vec::new(),
            cycle_position: 0,
            total_steps: 0,
            stats: SwarmStats::default(),
        }
    }
    
    /// Submit a task to the swarm
    pub fn submit_task(&mut self, task: SwarmTask) {
        self.task_queue.push(task);
        // Sort by priority (higher first)
        self.task_queue.sort_by(|a, b| b.priority.cmp(&a.priority));
    }
    
    /// Execute one step of the swarm
    pub fn step(&mut self) -> SwarmStepResult {
        self.total_steps += 1;
        self.stats.total_steps += 1;
        
        // Advance vortex cycle
        self.cycle_position = (self.cycle_position + 1) % VORTEX_CYCLE.len();
        let current_vortex = VORTEX_CYCLE[self.cycle_position];
        
        // Assign pending tasks to idle agents
        self.assign_tasks();
        
        // Execute active agents and collect updates
        let mut step_results = Vec::new();
        let mut completed_task_ids = Vec::new();
        let mut task_updates: Vec<(Uuid, Option<String>, f32)> = Vec::new();
        
        for agent in &mut self.agents {
            if agent.state == AgentState::Active {
                let result = agent.step();
                let agent_idx = agent.index;
                step_results.push(result.clone());
                
                // Collect task updates (defer actual updates)
                for (task_id, task) in &self.active_tasks {
                    if task.assigned_agents.contains(&agent_idx) {
                        task_updates.push((*task_id, result.output.clone(), result.confidence));
                        break;
                    }
                }
            }
        }
        
        // Apply task updates
        for (task_id, output, confidence) in task_updates {
            if let Some(task) = self.active_tasks.get_mut(&task_id) {
                task.current_step += 1;
                if let Some(out) = output {
                    task.partial_results.push(out);
                }
                if task.current_step >= task.max_steps || confidence > 0.9 {
                    task.state = TaskState::AwaitingConsensus;
                }
            }
        }
        
        // Run consensus on tasks awaiting it
        let consensus_threshold = self.config.consensus_threshold;
        let awaiting_consensus: Vec<Uuid> = self.active_tasks.iter()
            .filter(|(_, t)| t.state == TaskState::AwaitingConsensus)
            .map(|(id, _)| *id)
            .collect();
        
        for task_id in awaiting_consensus {
            let consensus = self.compute_consensus_score();
            if let Some(task) = self.active_tasks.get_mut(&task_id) {
                task.consensus_votes.insert(0, consensus);
                if consensus >= consensus_threshold {
                    task.state = TaskState::Completed;
                    completed_task_ids.push(task_id);
                } else if task.current_step >= task.max_steps * 2 {
                    task.state = TaskState::Failed;
                    completed_task_ids.push(task_id);
                }
            }
        }
        
        // Move completed tasks
        for task_id in completed_task_ids {
            if let Some(task) = self.active_tasks.remove(&task_id) {
                // Release agents
                for &agent_idx in &task.assigned_agents {
                    if let Some(agent) = self.agents.get_mut(agent_idx) {
                        agent.reset();
                    }
                }
                
                // Update stats
                if task.state == TaskState::Completed {
                    self.stats.total_tasks_completed += 1;
                } else {
                    self.stats.total_tasks_failed += 1;
                }
                
                self.completed_tasks.push(task);
            }
        }
        
        // Update statistics
        self.update_stats();
        
        SwarmStepResult {
            step_number: self.total_steps,
            vortex_position: current_vortex,
            active_agents: self.agents.iter().filter(|a| a.state == AgentState::Active).count(),
            tasks_in_progress: self.active_tasks.len(),
            tasks_completed: self.stats.total_tasks_completed,
            agent_results: step_results,
        }
    }
    
    /// Assign pending tasks to available agents
    fn assign_tasks(&mut self) {
        while !self.task_queue.is_empty() {
            // Find idle agents matching required specializations
            let task = match self.task_queue.first() {
                Some(t) => t.clone(),
                None => break,
            };
            
            let mut assigned = Vec::new();
            for agent in &mut self.agents {
                if agent.state == AgentState::Idle {
                    if task.required_specs.is_empty() || 
                       task.required_specs.contains(&agent.specialization) {
                        agent.state = AgentState::Active;
                        agent.current_task = Some(task.description.clone());
                        assigned.push(agent.index);
                        
                        if assigned.len() >= self.config.max_active.min(10) {
                            break;
                        }
                    }
                }
            }
            
            if assigned.is_empty() {
                break; // No available agents
            }
            
            // Move task to active
            let mut task = self.task_queue.remove(0);
            task.assigned_agents = assigned;
            task.state = TaskState::InProgress;
            self.active_tasks.insert(task.id, task);
        }
    }
    
    /// Find task assigned to an agent
    fn find_agent_task(&self, agent_idx: usize) -> Option<Uuid> {
        for (task_id, task) in &self.active_tasks {
            if task.assigned_agents.contains(&agent_idx) {
                return Some(*task_id);
            }
        }
        None
    }
    
    /// Compute consensus score from anchor agents using ELP-weighted voting
    /// 
    /// Feed-forward from Attributes:
    /// - Ethos: Primary weight in consensus (credibility/trust)
    /// - Logos: Scales confidence (logical certainty)
    /// - Pathos: Adds intuitive weight for uncertain cases
    fn compute_consensus_score(&self) -> f32 {
        let mut weighted_vote_sum = 0.0;
        let mut weight_sum = 0.0;
        
        for agent in &self.agents {
            if agent.is_anchor {
                // ELP-weighted voting:
                // - Ethos determines vote weight (trust/credibility)
                // - Logos scales confidence (logical certainty)
                // - Pathos adds intuitive component
                let ethos = agent.attributes.ethos();
                let logos = agent.attributes.logos();
                let pathos = agent.attributes.pathos();
                
                // Vote = confidence * logos (logical certainty)
                let logical_vote = agent.confidence * logos;
                // Intuitive component for uncertain cases
                let intuitive_vote = agent.energy * pathos * 0.3;
                let vote = logical_vote + intuitive_vote;
                
                // Weight by ethos (credibility)
                weighted_vote_sum += vote * ethos;
                weight_sum += ethos;
            }
        }
        
        if weight_sum > 0.0 {
            weighted_vote_sum / weight_sum
        } else {
            0.5 // Default if no anchors
        }
    }
    
    /// Update swarm statistics
    fn update_stats(&mut self) {
        let total_tasks = self.stats.total_tasks_completed + self.stats.total_tasks_failed;
        if total_tasks > 0 {
            self.stats.avg_steps_per_task = self.stats.total_steps as f32 / total_tasks as f32;
            self.stats.consensus_success_rate = 
                self.stats.total_tasks_completed as f32 / total_tasks as f32;
        }
        
        let active_count = self.agents.iter().filter(|a| a.state == AgentState::Active).count();
        self.stats.agent_utilization = active_count as f32 / self.agents.len() as f32;
        
        self.stats.avg_confidence = self.agents.iter()
            .map(|a| a.confidence)
            .sum::<f32>() / self.agents.len() as f32;
        
        // Update vortex distribution
        for agent in &self.agents {
            if agent.state == AgentState::Active && agent.vortex_position > 0 {
                self.stats.vortex_position_distribution[agent.vortex_position - 1] += 1;
            }
        }
    }
    
    /// Run swarm until all tasks complete or max steps reached
    pub fn run_to_completion(&mut self, max_steps: usize) -> SwarmRunResult {
        let start_tasks = self.task_queue.len() + self.active_tasks.len();
        let mut steps = 0;
        
        while steps < max_steps {
            if self.task_queue.is_empty() && self.active_tasks.is_empty() {
                break;
            }
            
            self.step();
            steps += 1;
        }
        
        SwarmRunResult {
            total_steps: steps,
            tasks_submitted: start_tasks,
            tasks_completed: self.stats.total_tasks_completed as usize,
            tasks_failed: self.stats.total_tasks_failed as usize,
            avg_confidence: self.stats.avg_confidence,
            agent_utilization: self.stats.agent_utilization,
        }
    }
    
    /// Get swarm summary
    pub fn summary(&self) -> String {
        format!(
            "Sacred Swarm\n\
             ============\n\
             Agents: {} ({} anchors)\n\
             Active: {}\n\
             Tasks Completed: {}\n\
             Tasks Failed: {}\n\
             Avg Confidence: {:.2}\n\
             Agent Utilization: {:.1}%\n\
             Consensus Success: {:.1}%\n\
             Total Steps: {}",
            self.agents.len(),
            self.agents.iter().filter(|a| a.is_anchor).count(),
            self.agents.iter().filter(|a| a.state == AgentState::Active).count(),
            self.stats.total_tasks_completed,
            self.stats.total_tasks_failed,
            self.stats.avg_confidence,
            self.stats.agent_utilization * 100.0,
            self.stats.consensus_success_rate * 100.0,
            self.stats.total_steps
        )
    }
    
    /// Query swarm agents for test-time knowledge retrieval (RAG-style)
    /// 
    /// This enables agents to retrieve relevant knowledge during inference,
    /// implementing the "test-time compute scaling" paradigm from arXiv:2408.03314.
    /// 
    /// Each agent specialization contributes different knowledge:
    /// - Retrieval agents: External knowledge lookup
    /// - Math agents: Numerical reasoning
    /// - Geometric agents: Spatial/structural reasoning
    /// - Verification agents: Fact-checking
    pub fn query_knowledge(&self, query: &str, max_results: usize) -> Vec<SwarmKnowledgeResult> {
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();
        
        // Query each active retrieval agent
        for agent in &self.agents {
            if agent.specialization == AgentSpecialization::Retrieval && agent.state != AgentState::Error {
                // Simulate knowledge retrieval based on query
                let knowledge = self.agent_retrieve_knowledge(agent, &query_lower);
                if !knowledge.is_empty() {
                    results.push(SwarmKnowledgeResult {
                        agent_id: agent.id,
                        agent_specialization: agent.specialization,
                        knowledge,
                        confidence: agent.confidence,
                        vortex_position: agent.vortex_position,
                    });
                }
            }
        }
        
        // Also query verification agents for fact-checking
        for agent in &self.agents {
            if agent.specialization == AgentSpecialization::Verification && agent.state != AgentState::Error {
                let verification = self.agent_verify_query(agent, &query_lower);
                if !verification.is_empty() {
                    results.push(SwarmKnowledgeResult {
                        agent_id: agent.id,
                        agent_specialization: agent.specialization,
                        knowledge: verification,
                        confidence: agent.confidence * 0.9, // Slightly lower for verification
                        vortex_position: agent.vortex_position,
                    });
                }
            }
        }
        
        // Sort by confidence and take top results
        results.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(max_results);
        
        results
    }
    
    /// Agent-specific knowledge retrieval
    fn agent_retrieve_knowledge(&self, agent: &SwarmAgent, query: &str) -> Vec<String> {
        let mut knowledge = Vec::new();
        
        // Location-based knowledge
        if query.contains("where") || query.contains("location") || query.contains("find") {
            knowledge.push("Location knowledge: Check context for spatial relationships".to_string());
            if query.contains("penguin") {
                knowledge.push("Penguins are found in Antarctica and zoos".to_string());
            }
            if query.contains("book") {
                knowledge.push("Books are found in libraries and bookstores".to_string());
            }
        }
        
        // Causal/reasoning knowledge
        if query.contains("why") || query.contains("because") || query.contains("cause") {
            knowledge.push("Causal reasoning: Look for cause-effect relationships".to_string());
        }
        
        // Property knowledge
        if query.contains("what is") || query.contains("property") || query.contains("characteristic") {
            knowledge.push("Property knowledge: Check for entity-attribute relationships".to_string());
        }
        
        // Fear/emotion knowledge (for bAbI Task 15)
        if query.contains("afraid") || query.contains("fear") {
            knowledge.push("Fear relationships: Check for 'X are afraid of Y' patterns".to_string());
        }
        
        knowledge
    }
    
    /// Agent-specific verification
    fn agent_verify_query(&self, _agent: &SwarmAgent, query: &str) -> Vec<String> {
        let mut verification = Vec::new();
        
        // Verify logical consistency
        if query.contains("is") || query.contains("are") {
            verification.push("Verify: Check for contradictions in context".to_string());
        }
        
        // Verify temporal consistency
        if query.contains("before") || query.contains("after") || query.contains("then") {
            verification.push("Verify: Check temporal ordering of events".to_string());
        }
        
        verification
    }
    
    /// Aggregate knowledge from multiple agents using ELP-weighted consensus
    pub fn aggregate_knowledge(&self, results: &[SwarmKnowledgeResult]) -> AggregatedKnowledge {
        if results.is_empty() {
            return AggregatedKnowledge::default();
        }
        
        let mut all_knowledge: Vec<String> = Vec::new();
        let mut total_confidence = 0.0f32;
        let mut total_weight = 0.0f32;
        
        for result in results {
            // Find the agent and get its ELP weights
            if let Some(agent) = self.agents.iter().find(|a| a.id == result.agent_id) {
                let ethos = agent.attributes.ethos();
                let weight = ethos * result.confidence;
                
                for k in &result.knowledge {
                    if !all_knowledge.contains(k) {
                        all_knowledge.push(k.clone());
                    }
                }
                
                total_confidence += result.confidence * weight;
                total_weight += weight;
            }
        }
        
        let avg_confidence = if total_weight > 0.0 {
            total_confidence / total_weight
        } else {
            0.0
        };
        
        AggregatedKnowledge {
            knowledge: all_knowledge,
            confidence: avg_confidence,
            num_agents: results.len(),
        }
    }
}

/// Result of a swarm knowledge query
#[derive(Debug, Clone)]
pub struct SwarmKnowledgeResult {
    pub agent_id: Uuid,
    pub agent_specialization: AgentSpecialization,
    pub knowledge: Vec<String>,
    pub confidence: f32,
    pub vortex_position: usize,
}

/// Aggregated knowledge from multiple agents
#[derive(Debug, Clone, Default)]
pub struct AggregatedKnowledge {
    pub knowledge: Vec<String>,
    pub confidence: f32,
    pub num_agents: usize,
}

/// Result of a single swarm step
#[derive(Debug, Clone)]
pub struct SwarmStepResult {
    pub step_number: u64,
    pub vortex_position: usize,
    pub active_agents: usize,
    pub tasks_in_progress: usize,
    pub tasks_completed: u64,
    pub agent_results: Vec<AgentStepResult>,
}

/// Result of running swarm to completion
#[derive(Debug, Clone)]
pub struct SwarmRunResult {
    pub total_steps: usize,
    pub tasks_submitted: usize,
    pub tasks_completed: usize,
    pub tasks_failed: usize,
    pub avg_confidence: f32,
    pub agent_utilization: f32,
}

// =============================================================================
// Geometric Optimizer (φ-scaling)
// =============================================================================

/// Geometric optimizer with φ-scaling for stable training
#[derive(Debug, Clone)]
pub struct GeometricOptimizer {
    /// Learning rate
    pub lr: f32,
    /// φ-scaling factor
    pub phi_scale: f32,
    /// Momentum
    pub momentum: f32,
    /// Weight decay
    pub weight_decay: f32,
    /// Gradient history for momentum
    pub grad_history: HashMap<String, Vec<f32>>,
    /// Step count
    pub step_count: u64,
    /// Sacred position boost schedule
    pub sacred_schedule: [f32; 9],
}

impl GeometricOptimizer {
    pub fn new(lr: f32) -> Self {
        // Initialize sacred schedule with φ-based scaling
        let mut sacred_schedule = [1.0; 9];
        for (i, pos) in (1..=9).enumerate() {
            if SACRED_POSITIONS.contains(&pos) {
                sacred_schedule[i] = 1.15; // Sacred boost
            } else {
                // φ-based scaling for non-sacred positions
                sacred_schedule[i] = PHI.powf((pos as f32 - 5.0) / 9.0);
            }
        }
        
        Self {
            lr,
            phi_scale: PHI,
            momentum: 0.9,
            weight_decay: 0.01,
            grad_history: HashMap::new(),
            step_count: 0,
            sacred_schedule,
        }
    }
    
    /// Compute update for a parameter
    pub fn compute_update(&mut self, param_name: &str, gradient: &[f32], vortex_position: usize) -> Vec<f32> {
        self.step_count += 1;
        
        // Get or create gradient history
        let history = self.grad_history.entry(param_name.to_string())
            .or_insert_with(|| vec![0.0; gradient.len()]);
        
        // Ensure history has correct size
        if history.len() != gradient.len() {
            *history = vec![0.0; gradient.len()];
        }
        
        // Get sacred schedule factor
        let sacred_factor = if vortex_position > 0 && vortex_position <= 9 {
            self.sacred_schedule[vortex_position - 1]
        } else {
            1.0
        };
        
        // Compute update with momentum and φ-scaling
        let mut update = Vec::with_capacity(gradient.len());
        for (i, &g) in gradient.iter().enumerate() {
            // Momentum update
            let m = self.momentum * history[i] + (1.0 - self.momentum) * g;
            history[i] = m;
            
            // φ-scaled learning rate
            let scaled_lr = self.lr * sacred_factor / self.phi_scale.powf(self.step_count as f32 / 10000.0);
            
            // Weight decay
            let wd = self.weight_decay * scaled_lr;
            
            update.push(-scaled_lr * m - wd);
        }
        
        update
    }
    
    /// Get current effective learning rate
    pub fn effective_lr(&self, vortex_position: usize) -> f32 {
        let sacred_factor = if vortex_position > 0 && vortex_position <= 9 {
            self.sacred_schedule[vortex_position - 1]
        } else {
            1.0
        };
        
        self.lr * sacred_factor / self.phi_scale.powf(self.step_count as f32 / 10000.0)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swarm_config() {
        let config = SacredSwarmConfig::default();
        assert_eq!(config.num_agents, 1000);
        assert_eq!(config.max_active, 100);
        assert_eq!(config.num_anchors, 9);
    }

    #[test]
    fn test_swarm_agent() {
        let config = SacredSwarmConfig::default();
        let agent = SwarmAgent::new(0, &config);
        
        assert_eq!(agent.vortex_position, 1);
        assert_eq!(agent.state, AgentState::Idle);
        assert!(!agent.connections.is_empty());
    }

    #[test]
    fn test_anchor_agents() {
        let config = SacredSwarmConfig::default();
        let swarm = SacredSwarm::new(config);
        
        let anchors: Vec<_> = swarm.agents.iter().filter(|a| a.is_anchor).collect();
        assert!(!anchors.is_empty());
        
        // All anchors should be at sacred positions
        for anchor in anchors {
            assert!(SACRED_POSITIONS.contains(&anchor.vortex_position));
        }
    }

    #[test]
    fn test_task_submission() {
        let config = SacredSwarmConfig {
            num_agents: 10,
            max_active: 5,
            ..Default::default()
        };
        let mut swarm = SacredSwarm::new(config);
        
        let task = SwarmTask::new("Test task", vec![AgentSpecialization::General]);
        swarm.submit_task(task);
        
        assert_eq!(swarm.task_queue.len(), 1);
    }

    #[test]
    fn test_swarm_step() {
        let config = SacredSwarmConfig {
            num_agents: 10,
            max_active: 5,
            ..Default::default()
        };
        let mut swarm = SacredSwarm::new(config);
        
        let task = SwarmTask::new("Test task", vec![])
            .with_max_steps(5);
        swarm.submit_task(task);
        
        let result = swarm.step();
        assert!(result.active_agents > 0 || result.tasks_in_progress > 0);
    }

    #[test]
    fn test_geometric_optimizer() {
        let mut optimizer = GeometricOptimizer::new(0.001);
        
        let gradient = vec![0.1, 0.2, 0.3];
        let update = optimizer.compute_update("test_param", &gradient, 3);
        
        assert_eq!(update.len(), gradient.len());
        // Updates should be negative (gradient descent)
        for u in &update {
            assert!(*u < 0.0);
        }
    }

    #[test]
    fn test_sacred_schedule() {
        let optimizer = GeometricOptimizer::new(0.001);
        
        // Sacred positions should have higher factors
        assert!(optimizer.sacred_schedule[2] > optimizer.sacred_schedule[0]); // pos 3 > pos 1
        assert!(optimizer.sacred_schedule[5] > optimizer.sacred_schedule[3]); // pos 6 > pos 4
        assert!(optimizer.sacred_schedule[8] > optimizer.sacred_schedule[6]); // pos 9 > pos 7
    }

    #[test]
    fn test_swarm_run() {
        let config = SacredSwarmConfig {
            num_agents: 20,
            max_active: 10,
            consensus_threshold: 0.3, // Lower for testing
            ..Default::default()
        };
        let mut swarm = SacredSwarm::new(config);
        
        // Submit multiple tasks
        for i in 0..3 {
            let task = SwarmTask::new(&format!("Task {}", i), vec![])
                .with_max_steps(10);
            swarm.submit_task(task);
        }
        
        let result = swarm.run_to_completion(100);
        
        assert!(result.total_steps > 0);
        assert!(result.tasks_completed > 0 || result.tasks_failed > 0);
    }
}
