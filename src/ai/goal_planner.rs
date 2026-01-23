//! Goal Planning System for AGI
//!
//! Enables goal-directed behavior through:
//! - Long-term goal formation with multi-step objectives
//! - Hierarchical Task Network (HTN) planning
//! - Plan execution and monitoring with adaptive replanning
//! - Goal conflict resolution using ELP priorities
//!
//! ## Architecture
//!
//! ```text
//! Goals -> HTN Decomposition -> Plan -> Execution -> Monitoring
//!   ^                                              |
//!   +-------------- Replanning (if needed) <------+
//! ```

use crate::data::models::ELPTensor;
use anyhow::Result;
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, VecDeque};
use std::cmp::Ordering;
use uuid::Uuid;

// ============================================================================
// Core Goal Structures
// ============================================================================

/// A goal represents a desired state or outcome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub id: Uuid,
    pub objective: String,
    pub importance: f32,
    pub elp_profile: ELPTensor,
    pub deadline: Option<DateTime<Utc>>,
    pub subgoals: Vec<Goal>,
    pub success_criteria: Vec<Criterion>,
    pub status: GoalStatus,
    pub vortex_position: u8,
    pub sacred_influence: Option<u8>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoalStatus {
    Pending,
    Active,
    Suspended,
    Achieved,
    Failed,
    Abandoned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Criterion {
    pub description: String,
    pub criterion_type: CriterionType,
    pub satisfaction: f32,
    pub weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CriterionType {
    StateThreshold { key: String, threshold: f64 },
    ActionComplete { action_id: Uuid },
    Temporal { condition: String },
    ELPCondition { min_ethos: f64, min_logos: f64, min_pathos: f64 },
    Custom { predicate: String },
}

// ============================================================================
// HTN Planning Structures
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub name: String,
    pub task_type: TaskType,
    pub preconditions: Vec<Condition>,
    pub effects: Vec<Effect>,
    pub estimated_duration: Duration,
    pub elp_cost: ELPTensor,
    pub priority: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    Primitive { action: String },
    Compound { methods: Vec<Method> },
    Oracle { question_template: String },
    Reasoning { target_elp: ELPTensor },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Method {
    pub name: String,
    pub conditions: Vec<Condition>,
    pub subtasks: Vec<Task>,
    pub success_probability: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub key: String,
    pub value: ConditionValue,
    pub negated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionValue {
    Boolean(bool),
    Numeric(f64),
    Text(String),
    ELP(ELPTensor),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Effect {
    pub key: String,
    pub value: ConditionValue,
    pub probability: f32,
}

// ============================================================================
// Plan Structures
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: Uuid,
    pub goal_id: Uuid,
    pub tasks: Vec<PlannedTask>,
    pub current_index: usize,
    pub status: PlanStatus,
    pub estimated_duration: Duration,
    pub elapsed_time: Duration,
    pub success_probability: f32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedTask {
    pub task: Task,
    pub status: TaskStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub result: Option<TaskResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanStatus {
    Planning,
    Ready,
    Executing,
    Completed,
    Failed,
    Replanning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub success: bool,
    pub output: HashMap<String, String>,
    pub effects: Vec<Effect>,
}

// ============================================================================
// Goal Planner
// ============================================================================

pub struct GoalPlanner {
    pub long_term_goals: Vec<Goal>,
    pub active_plan: Option<Plan>,
    pub executor: PlanExecutor,
    pub arbiter: GoalArbiter,
    pub world_state: HashMap<String, ConditionValue>,
    pub task_library: HashMap<String, Task>,
    pub stats: PlanningStats,
}

#[derive(Debug, Clone, Default)]
pub struct PlanningStats {
    pub goals_created: u64,
    pub goals_achieved: u64,
    pub goals_failed: u64,
    pub plans_created: u64,
    pub plans_succeeded: u64,
    pub replans_triggered: u64,
    pub avg_plan_duration_ms: f64,
}

impl Default for GoalPlanner {
    fn default() -> Self {
        Self::new()
    }
}

impl GoalPlanner {
    pub fn new() -> Self {
        Self {
            long_term_goals: Vec::new(),
            active_plan: None,
            executor: PlanExecutor::new(),
            arbiter: GoalArbiter::new(),
            world_state: HashMap::new(),
            task_library: HashMap::new(),
            stats: PlanningStats::default(),
        }
    }
    
    /// Create a goal from objective with ELP analysis
    pub fn create_goal(&mut self, objective: &str, elp: &ELPTensor) -> Goal {
        let importance = ((elp.ethos + elp.logos + elp.pathos) / 39.0) as f32;
        
        let vortex_position = if elp.ethos > elp.logos && elp.ethos > elp.pathos {
            3
        } else if elp.logos > elp.pathos {
            6
        } else {
            9
        };
        
        let sacred_influence = if matches!(vortex_position, 3 | 6 | 9) {
            Some(vortex_position)
        } else {
            None
        };
        
        let goal = Goal {
            id: Uuid::new_v4(),
            objective: objective.to_string(),
            importance,
            elp_profile: elp.clone(),
            deadline: None,
            subgoals: Vec::new(),
            success_criteria: vec![Criterion {
                description: format!("Achieve: {}", objective),
                criterion_type: CriterionType::Custom { predicate: objective.to_string() },
                satisfaction: 0.0,
                weight: 1.0,
            }],
            status: GoalStatus::Pending,
            vortex_position,
            sacred_influence,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        self.stats.goals_created += 1;
        tracing::info!("Goal created: {} (importance: {:.2})", objective, importance);
        goal
    }
    
    /// Add goal with conflict resolution
    pub fn add_goal(&mut self, goal: Goal) {
        let conflicts = self.arbiter.find_conflicts(&goal, &self.long_term_goals);
        
        if !conflicts.is_empty() {
            tracing::warn!("Goal conflicts detected with {} existing goals", conflicts.len());
            let resolved = self.arbiter.resolve_conflicts(goal.clone(), conflicts);
            self.long_term_goals.push(resolved);
        } else {
            self.long_term_goals.push(goal);
        }
        
        self.long_term_goals.sort_by(|a, b| {
            b.importance.partial_cmp(&a.importance).unwrap_or(Ordering::Equal)
        });
    }
    
    /// Plan for highest priority goal
    pub fn plan_next_goal(&mut self) -> Result<Option<Plan>> {
        let goal = match self.long_term_goals.iter_mut()
            .find(|g| g.status == GoalStatus::Pending) 
        {
            Some(g) => {
                g.status = GoalStatus::Active;
                g.clone()
            },
            None => return Ok(None),
        };
        
        tracing::info!("Planning for goal: {}", goal.objective);
        let plan = self.htn_plan(&goal)?;
        self.stats.plans_created += 1;
        self.active_plan = Some(plan.clone());
        Ok(Some(plan))
    }
    
    /// HTN planning algorithm
    fn htn_plan(&self, goal: &Goal) -> Result<Plan> {
        let mut tasks = Vec::new();
        let mut task_queue: VecDeque<Task> = VecDeque::new();
        
        let root_task = self.goal_to_task(goal);
        task_queue.push_back(root_task);
        
        while let Some(task) = task_queue.pop_front() {
            match &task.task_type {
                TaskType::Primitive { .. } | TaskType::Oracle { .. } | TaskType::Reasoning { .. } => {
                    tasks.push(PlannedTask {
                        task,
                        status: TaskStatus::Pending,
                        started_at: None,
                        completed_at: None,
                        result: None,
                    });
                },
                TaskType::Compound { methods } => {
                    if let Some(method) = self.find_applicable_method(methods) {
                        for subtask in &method.subtasks {
                            task_queue.push_back(subtask.clone());
                        }
                    }
                },
            }
        }
        
        let estimated_duration = tasks.iter()
            .map(|t| t.task.estimated_duration)
            .fold(Duration::zero(), |acc, d| acc + d);
        
        let success_probability: f32 = tasks.iter()
            .map(|t| match &t.task.task_type {
                TaskType::Primitive { .. } => 0.9,
                TaskType::Oracle { .. } => 0.85,
                TaskType::Reasoning { .. } => 0.8,
                TaskType::Compound { methods } => methods.first().map(|m| m.success_probability).unwrap_or(0.5),
            })
            .product();
        
        Ok(Plan {
            id: Uuid::new_v4(),
            goal_id: goal.id,
            tasks,
            current_index: 0,
            status: PlanStatus::Ready,
            estimated_duration,
            elapsed_time: Duration::zero(),
            success_probability,
            created_at: Utc::now(),
        })
    }
    
    fn goal_to_task(&self, goal: &Goal) -> Task {
        let methods = vec![Method {
            name: "reasoning_approach".to_string(),
            conditions: vec![],
            subtasks: vec![
                Task {
                    id: Uuid::new_v4(),
                    name: "analyze_problem".to_string(),
                    task_type: TaskType::Reasoning {
                        target_elp: ELPTensor {
                            ethos: goal.elp_profile.ethos,
                            logos: goal.elp_profile.logos + 2.0,
                            pathos: goal.elp_profile.pathos,
                        },
                    },
                    preconditions: vec![],
                    effects: vec![Effect {
                        key: "problem_analyzed".to_string(),
                        value: ConditionValue::Boolean(true),
                        probability: 0.9,
                    }],
                    estimated_duration: Duration::seconds(5),
                    elp_cost: ELPTensor { ethos: 1.0, logos: 2.0, pathos: 0.5 },
                    priority: 0.9,
                },
                Task {
                    id: Uuid::new_v4(),
                    name: "gather_knowledge".to_string(),
                    task_type: TaskType::Oracle {
                        question_template: format!("What are the key facts about: {}?", goal.objective),
                    },
                    preconditions: vec![Condition {
                        key: "problem_analyzed".to_string(),
                        value: ConditionValue::Boolean(true),
                        negated: false,
                    }],
                    effects: vec![Effect {
                        key: "knowledge_gathered".to_string(),
                        value: ConditionValue::Boolean(true),
                        probability: 0.85,
                    }],
                    estimated_duration: Duration::seconds(10),
                    elp_cost: ELPTensor { ethos: 0.5, logos: 3.0, pathos: 1.0 },
                    priority: 0.85,
                },
                Task {
                    id: Uuid::new_v4(),
                    name: "synthesize_solution".to_string(),
                    task_type: TaskType::Reasoning {
                        target_elp: goal.elp_profile.clone(),
                    },
                    preconditions: vec![Condition {
                        key: "knowledge_gathered".to_string(),
                        value: ConditionValue::Boolean(true),
                        negated: false,
                    }],
                    effects: vec![Effect {
                        key: "solution_found".to_string(),
                        value: ConditionValue::Boolean(true),
                        probability: 0.8,
                    }],
                    estimated_duration: Duration::seconds(15),
                    elp_cost: ELPTensor { ethos: 2.0, logos: 2.0, pathos: 2.0 },
                    priority: 0.95,
                },
            ],
            success_probability: 0.75,
        }];
        
        Task {
            id: Uuid::new_v4(),
            name: format!("achieve_{}", goal.id),
            task_type: TaskType::Compound { methods },
            preconditions: vec![],
            effects: vec![Effect {
                key: format!("goal_{}_achieved", goal.id),
                value: ConditionValue::Boolean(true),
                probability: 0.8,
            }],
            estimated_duration: Duration::seconds(30),
            elp_cost: goal.elp_profile.clone(),
            priority: goal.importance,
        }
    }
    
    fn find_applicable_method<'a>(&self, methods: &'a [Method]) -> Option<&'a Method> {
        methods.iter()
            .filter(|m| self.conditions_satisfied(&m.conditions))
            .max_by(|a, b| a.success_probability.partial_cmp(&b.success_probability).unwrap_or(Ordering::Equal))
    }
    
    fn conditions_satisfied(&self, conditions: &[Condition]) -> bool {
        conditions.iter().all(|c| {
            match self.world_state.get(&c.key) {
                Some(value) => {
                    let matches = match (&c.value, value) {
                        (ConditionValue::Boolean(a), ConditionValue::Boolean(b)) => a == b,
                        (ConditionValue::Numeric(a), ConditionValue::Numeric(b)) => (a - b).abs() < 0.001,
                        (ConditionValue::Text(a), ConditionValue::Text(b)) => a == b,
                        _ => false,
                    };
                    if c.negated { !matches } else { matches }
                },
                None => c.negated,
            }
        })
    }
    
    /// Execute the active plan
    pub async fn execute_plan(&mut self) -> Result<bool> {
        let plan = match &mut self.active_plan {
            Some(p) => p,
            None => return Ok(false),
        };
        
        plan.status = PlanStatus::Executing;
        
        while plan.current_index < plan.tasks.len() {
            let task = &mut plan.tasks[plan.current_index];
            task.status = TaskStatus::Running;
            task.started_at = Some(Utc::now());
            
            let result = self.executor.execute_task(&task.task, &self.world_state).await?;
            
            task.completed_at = Some(Utc::now());
            task.result = Some(result.clone());
            
            if result.success {
                task.status = TaskStatus::Completed;
                for effect in &result.effects {
                    self.world_state.insert(effect.key.clone(), effect.value.clone());
                }
                plan.current_index += 1;
            } else {
                task.status = TaskStatus::Failed;
                plan.status = PlanStatus::Failed;
                return Ok(false);
            }
        }
        
        plan.status = PlanStatus::Completed;
        self.stats.plans_succeeded += 1;
        Ok(true)
    }
    
    /// Mark goal as achieved
    pub fn mark_goal_achieved(&mut self, goal_id: Uuid) {
        if let Some(goal) = self.long_term_goals.iter_mut().find(|g| g.id == goal_id) {
            goal.status = GoalStatus::Achieved;
            goal.updated_at = Utc::now();
            self.stats.goals_achieved += 1;
            tracing::info!("Goal achieved: {}", goal.objective);
        }
    }
    
    /// Trigger replanning
    pub fn replan(&mut self) -> Result<Option<Plan>> {
        self.stats.replans_triggered += 1;
        if let Some(plan) = &self.active_plan {
            let goal_id = plan.goal_id;
            if let Some(goal) = self.long_term_goals.iter().find(|g| g.id == goal_id) {
                let new_plan = self.htn_plan(goal)?;
                self.active_plan = Some(new_plan.clone());
                return Ok(Some(new_plan));
            }
        }
        Ok(None)
    }
}

// ============================================================================
// Plan Executor
// ============================================================================

pub struct PlanExecutor {
    pub execution_history: Vec<TaskResult>,
}

impl Default for PlanExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl PlanExecutor {
    pub fn new() -> Self {
        Self {
            execution_history: Vec::new(),
        }
    }
    
    pub async fn execute_task(
        &mut self,
        task: &Task,
        _world_state: &HashMap<String, ConditionValue>,
    ) -> Result<TaskResult> {
        tracing::debug!("Executing task: {}", task.name);
        
        let result = match &task.task_type {
            TaskType::Primitive { action } => {
                tracing::debug!("Primitive action: {}", action);
                TaskResult {
                    success: true,
                    output: HashMap::from([("action".to_string(), action.clone())]),
                    effects: task.effects.clone(),
                }
            },
            TaskType::Oracle { question_template } => {
                tracing::debug!("Oracle query: {}", question_template);
                TaskResult {
                    success: true,
                    output: HashMap::from([("question".to_string(), question_template.clone())]),
                    effects: task.effects.clone(),
                }
            },
            TaskType::Reasoning { target_elp } => {
                tracing::debug!("Reasoning to ELP: {:?}", target_elp);
                TaskResult {
                    success: true,
                    output: HashMap::from([
                        ("ethos".to_string(), target_elp.ethos.to_string()),
                        ("logos".to_string(), target_elp.logos.to_string()),
                        ("pathos".to_string(), target_elp.pathos.to_string()),
                    ]),
                    effects: task.effects.clone(),
                }
            },
            TaskType::Compound { .. } => {
                TaskResult {
                    success: false,
                    output: HashMap::new(),
                    effects: vec![],
                }
            },
        };
        
        self.execution_history.push(result.clone());
        Ok(result)
    }
}

// ============================================================================
// Goal Arbiter (Conflict Resolution)
// ============================================================================

pub struct GoalArbiter {
    pub conflict_history: Vec<(Uuid, Uuid)>,
}

impl Default for GoalArbiter {
    fn default() -> Self {
        Self::new()
    }
}

impl GoalArbiter {
    pub fn new() -> Self {
        Self {
            conflict_history: Vec::new(),
        }
    }
    
    /// Find goals that conflict with the new goal
    pub fn find_conflicts(&self, new_goal: &Goal, existing: &[Goal]) -> Vec<Goal> {
        existing.iter()
            .filter(|g| self.goals_conflict(new_goal, g))
            .cloned()
            .collect()
    }
    
    /// Check if two goals conflict
    fn goals_conflict(&self, g1: &Goal, g2: &Goal) -> bool {
        // Conflict if same objective but different ELP profiles
        if g1.objective == g2.objective {
            return false; // Same goal, not a conflict
        }
        
        // Check ELP compatibility
        let elp_distance = ((g1.elp_profile.ethos - g2.elp_profile.ethos).powi(2)
            + (g1.elp_profile.logos - g2.elp_profile.logos).powi(2)
            + (g1.elp_profile.pathos - g2.elp_profile.pathos).powi(2)).sqrt();
        
        // High ELP distance with overlapping deadlines = conflict
        if elp_distance > 10.0 {
            if let (Some(d1), Some(d2)) = (&g1.deadline, &g2.deadline) {
                return d1 == d2;
            }
        }
        
        false
    }
    
    /// Resolve conflicts using ELP priorities
    pub fn resolve_conflicts(&mut self, new_goal: Goal, conflicts: Vec<Goal>) -> Goal {
        let mut resolved = new_goal.clone();
        
        for conflict in &conflicts {
            self.conflict_history.push((new_goal.id, conflict.id));
            
            // Adjust importance based on ELP balance
            let ethos_priority = (resolved.elp_profile.ethos / 13.0) as f32;
            let logos_priority = (resolved.elp_profile.logos / 13.0) as f32;
            
            // Ethos (ethics) takes precedence, then Logos (logic)
            resolved.importance = resolved.importance * (0.5 + ethos_priority * 0.3 + logos_priority * 0.2);
        }
        
        tracing::info!("Resolved {} conflicts, adjusted importance to {:.2}", 
            conflicts.len(), resolved.importance);
        
        resolved
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_goal_creation() {
        let mut planner = GoalPlanner::new();
        let elp = ELPTensor { ethos: 8.0, logos: 6.0, pathos: 4.0 };
        
        let goal = planner.create_goal("Learn quantum physics", &elp);
        
        assert_eq!(goal.status, GoalStatus::Pending);
        assert_eq!(goal.vortex_position, 3); // Ethos dominant
        assert!(goal.importance > 0.0);
    }
    
    #[test]
    fn test_htn_planning() {
        let mut planner = GoalPlanner::new();
        let elp = ELPTensor { ethos: 5.0, logos: 8.0, pathos: 3.0 };
        
        let goal = planner.create_goal("Solve math problem", &elp);
        planner.add_goal(goal);
        
        let plan = planner.plan_next_goal().unwrap();
        assert!(plan.is_some());
        
        let plan = plan.unwrap();
        assert!(!plan.tasks.is_empty());
        assert_eq!(plan.status, PlanStatus::Ready);
    }
    
    #[test]
    fn test_conflict_detection() {
        let arbiter = GoalArbiter::new();
        
        let g1 = Goal {
            id: Uuid::new_v4(),
            objective: "Goal A".to_string(),
            importance: 0.8,
            elp_profile: ELPTensor { ethos: 10.0, logos: 2.0, pathos: 1.0 },
            deadline: Some(Utc::now()),
            subgoals: vec![],
            success_criteria: vec![],
            status: GoalStatus::Pending,
            vortex_position: 3,
            sacred_influence: Some(3),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        let g2 = Goal {
            id: Uuid::new_v4(),
            objective: "Goal B".to_string(),
            importance: 0.6,
            elp_profile: ELPTensor { ethos: 1.0, logos: 10.0, pathos: 10.0 },
            deadline: Some(Utc::now()),
            subgoals: vec![],
            success_criteria: vec![],
            status: GoalStatus::Pending,
            vortex_position: 6,
            sacred_influence: Some(6),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        let conflicts = arbiter.find_conflicts(&g1, &[g2]);
        assert!(!conflicts.is_empty());
    }
}
