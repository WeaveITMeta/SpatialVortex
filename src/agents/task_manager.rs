//! Task Manager - Persistent Multi-Step Task Execution
//!
//! Enables Vortex to:
//! - Accept multiple instructions as separate tasks
//! - Work through them sequentially
//! - Track completion status per task
//! - Persist progress across conversation turns
//! - Resume incomplete tasks

use crate::agents::error::{Result, AgentError};
use crate::agents::coding_agent_enhanced::EnhancedCodingAgent;
use crate::agents::thinking_agent::ThinkingAgent;
use crate::agents::task_persistence::TaskPersistence;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use std::path::Path;

/// Task status tracking
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

/// A single task to be executed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub instruction: String,
    pub status: TaskStatus,
    pub result: Option<String>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub confidence: Option<f32>,
    pub task_type: TaskType,
}

/// Type of task for routing to appropriate agent
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskType {
    Coding,      // Generate or fix code
    Analysis,    // Analyze or explain something
    Research,    // Answer questions, RAG-based
    Reasoning,   // Deep thinking, multi-step
    General,     // Default fallback
}

/// Session-based task list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSession {
    pub session_id: String,
    pub tasks: Vec<Task>,
    pub current_task_index: usize,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TaskSession {
    pub fn new(session_id: String) -> Self {
        let now = Utc::now();
        Self {
            session_id,
            tasks: Vec::new(),
            current_task_index: 0,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Add a new task to the session
    pub fn add_task(&mut self, instruction: String, task_type: TaskType) -> String {
        let task_id = format!("task_{}_{}", self.tasks.len() + 1, Utc::now().timestamp());
        let task = Task {
            id: task_id.clone(),
            instruction,
            status: TaskStatus::Pending,
            result: None,
            error: None,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            confidence: None,
            task_type,
        };
        
        self.tasks.push(task);
        self.updated_at = Utc::now();
        task_id
    }
    
    /// Get current task to work on
    pub fn get_current_task(&mut self) -> Option<&mut Task> {
        if self.current_task_index < self.tasks.len() {
            Some(&mut self.tasks[self.current_task_index])
        } else {
            None
        }
    }
    
    /// Mark current task as complete and move to next
    pub fn complete_current_task(&mut self, result: String, confidence: f32) {
        if let Some(task) = self.get_current_task() {
            task.status = TaskStatus::Completed;
            task.result = Some(result);
            task.completed_at = Some(Utc::now());
            task.confidence = Some(confidence);
        }
        self.current_task_index += 1;
        self.updated_at = Utc::now();
    }
    
    /// Mark current task as failed
    pub fn fail_current_task(&mut self, error: String) {
        if let Some(task) = self.get_current_task() {
            task.status = TaskStatus::Failed;
            task.error = Some(error);
            task.completed_at = Some(Utc::now());
        }
        self.current_task_index += 1;
        self.updated_at = Utc::now();
    }
    
    /// Get progress summary
    pub fn get_progress(&self) -> TaskProgress {
        let total = self.tasks.len();
        let completed = self.tasks.iter().filter(|t| t.status == TaskStatus::Completed).count();
        let failed = self.tasks.iter().filter(|t| t.status == TaskStatus::Failed).count();
        let pending = self.tasks.iter().filter(|t| t.status == TaskStatus::Pending).count();
        
        TaskProgress {
            total,
            completed,
            failed,
            pending,
            all_done: completed + failed == total,
        }
    }
    
    /// Check if all tasks are complete
    pub fn is_complete(&self) -> bool {
        self.tasks.iter().all(|t| {
            t.status == TaskStatus::Completed || t.status == TaskStatus::Failed
        })
    }
}

/// Progress summary
#[derive(Debug, Clone, Serialize)]
pub struct TaskProgress {
    pub total: usize,
    pub completed: usize,
    pub failed: usize,
    pub pending: usize,
    pub all_done: bool,
}

/// Task Manager - manages multiple task sessions
pub struct TaskManager {
    sessions: Arc<RwLock<HashMap<String, TaskSession>>>,
    coding_agent: Arc<RwLock<EnhancedCodingAgent>>,
    thinking_agent: Arc<ThinkingAgent>,
    persistence: Option<Arc<TaskPersistence>>,
    auto_save: bool,
}

impl TaskManager {
    /// Create new TaskManager without persistence
    pub fn new(coding_agent: Arc<RwLock<EnhancedCodingAgent>>, thinking_agent: Arc<ThinkingAgent>) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            coding_agent,
            thinking_agent,
            persistence: None,
            auto_save: false,
        }
    }
    
    /// Create TaskManager with persistence enabled
    pub fn with_persistence<P: AsRef<Path>>(
        coding_agent: Arc<RwLock<EnhancedCodingAgent>>,
        thinking_agent: Arc<ThinkingAgent>,
        storage_dir: P,
        auto_save: bool,
    ) -> anyhow::Result<Self> {
        let persistence = TaskPersistence::new(storage_dir)?;
        
        Ok(Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            coding_agent,
            thinking_agent,
            persistence: Some(Arc::new(persistence)),
            auto_save,
        })
    }
    
    /// Load all persisted sessions on startup
    pub async fn restore_sessions(&self) -> anyhow::Result<usize> {
        if let Some(persistence) = &self.persistence {
            let loaded_sessions = persistence.load_all_sessions().await?;
            let count = loaded_sessions.len();
            
            let mut sessions = self.sessions.write().await;
            *sessions = loaded_sessions;
            
            println!("📦 Restored {} task sessions from disk", count);
            Ok(count)
        } else {
            Ok(0)
        }
    }
    
    /// Save a specific session to disk
    async fn save_session_if_enabled(&self, session_id: &str) -> anyhow::Result<()> {
        if self.auto_save {
            if let Some(persistence) = &self.persistence {
                let sessions = self.sessions.read().await;
                if let Some(session) = sessions.get(session_id) {
                    persistence.save_session(session_id, session).await?;
                }
            }
        }
        Ok(())
    }
    
    /// Parse user input into multiple tasks
    pub fn parse_tasks(&self, input: &str) -> Vec<(String, TaskType)> {
        let mut tasks = Vec::new();
        
        // Check for numbered lists (1. 2. 3.)
        if input.contains("1.") || input.contains("2.") {
            let lines: Vec<&str> = input.lines().collect();
            for line in lines {
                let trimmed = line.trim();
                // Match patterns like "1. ", "2. ", etc.
                if let Some(num_pos) = trimmed.find(|c: char| c.is_numeric()) {
                    if trimmed.chars().nth(num_pos + 1) == Some('.') {
                        if let Some(instruction) = trimmed.get(num_pos + 2..) {
                            let task_type = self.detect_task_type(instruction);
                            tasks.push((instruction.trim().to_string(), task_type));
                        }
                    }
                }
            }
        }
        
        // Check for bullet lists (- * •)
        if tasks.is_empty() && (input.contains("\n-") || input.contains("\n*") || input.contains("\n•")) {
            let lines: Vec<&str> = input.lines().collect();
            for line in lines {
                let trimmed = line.trim();
                if trimmed.starts_with('-') || trimmed.starts_with('*') || trimmed.starts_with('•') {
                    let instruction = trimmed.trim_start_matches(&['-', '*', '•'][..]).trim();
                    if !instruction.is_empty() {
                        let task_type = self.detect_task_type(instruction);
                        tasks.push((instruction.to_string(), task_type));
                    }
                }
            }
        }
        
        // Check for explicit task markers
        if tasks.is_empty() && (input.to_lowercase().contains("task 1") || input.to_lowercase().contains("first,")) {
            let parts: Vec<&str> = input.split(&[',', ';', '\n'][..]).collect();
            for part in parts {
                let trimmed = part.trim();
                if !trimmed.is_empty() && trimmed.len() > 5 {
                    let task_type = self.detect_task_type(trimmed);
                    tasks.push((trimmed.to_string(), task_type));
                }
            }
        }
        
        // Fallback: single task
        if tasks.is_empty() {
            tasks.push((input.to_string(), self.detect_task_type(input)));
        }
        
        tasks
    }
    
    /// Detect task type from instruction
    fn detect_task_type(&self, instruction: &str) -> TaskType {
        let lower = instruction.to_lowercase();
        
        if lower.contains("write code") || lower.contains("implement") || 
           lower.contains("function") || lower.contains("class") ||
           lower.contains("fix bug") || lower.contains("debug") {
            TaskType::Coding
        } else if lower.contains("analyze") || lower.contains("explain") ||
                  lower.contains("what is") || lower.contains("describe") {
            TaskType::Analysis
        } else if lower.contains("research") || lower.contains("find") ||
                  lower.contains("look up") || lower.contains("search") {
            TaskType::Research
        } else if lower.contains("think") || lower.contains("reason") ||
                  lower.contains("solve") || lower.contains("plan") {
            TaskType::Reasoning
        } else {
            TaskType::General
        }
    }
    
    /// Create or get task session
    pub async fn get_or_create_session(&self, session_id: &str) -> TaskSession {
        let mut sessions = self.sessions.write().await;
        sessions.entry(session_id.to_string())
            .or_insert_with(|| TaskSession::new(session_id.to_string()))
            .clone()
    }
    
    /// Add tasks to session
    pub async fn add_tasks(&self, session_id: &str, instructions: Vec<(String, TaskType)>) -> Vec<String> {
        let task_ids = {
            let mut sessions = self.sessions.write().await;
            let session = sessions.entry(session_id.to_string())
                .or_insert_with(|| TaskSession::new(session_id.to_string()));
            
            instructions.iter()
                .map(|(instr, task_type)| session.add_task(instr.clone(), task_type.clone()))
                .collect()
        };
        
        // Auto-save after adding tasks
        let _ = self.save_session_if_enabled(session_id).await;
        
        task_ids
    }
    
    /// Execute next pending task
    pub async fn execute_next_task(&self, session_id: &str) -> Result<TaskExecutionResult> {
        let mut sessions = self.sessions.write().await;
        let session = sessions.get_mut(session_id)
            .ok_or_else(|| AgentError::GenerationError("Session not found".to_string()))?;
        
        let task = session.get_current_task()
            .ok_or_else(|| AgentError::GenerationError("No tasks remaining".to_string()))?;
        
        task.status = TaskStatus::InProgress;
        task.started_at = Some(Utc::now());
        
        let task_id = task.id.clone();
        let instruction = task.instruction.clone();
        let task_type = task.task_type.clone();
        
        // Drop write lock before async operations
        drop(sessions);
        
        // Execute task based on type
        let result = match task_type {
            TaskType::Coding => {
                let agent = self.coding_agent.read().await;
                agent.execute_with_reasoning(&instruction).await
                    .map(|r| (r.code, r.reasoning_chain.overall_confidence))
            }
            TaskType::Reasoning | TaskType::Analysis | TaskType::Research | TaskType::General => {
                self.thinking_agent.think_and_respond(&instruction, None, None).await
                    .map(|r| (r.answer, r.confidence))
            }
        };
        
        // Re-acquire lock to update status
        let execution_result = {
            let mut sessions = self.sessions.write().await;
            let session = sessions.get_mut(session_id).unwrap();
            
            match result {
                Ok((output, confidence)) => {
                    session.complete_current_task(output.clone(), confidence);
                    TaskExecutionResult {
                        task_id,
                        instruction,
                        status: TaskStatus::Completed,
                        result: Some(output),
                        error: None,
                        progress: session.get_progress(),
                    }
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    session.fail_current_task(error_msg.clone());
                    TaskExecutionResult {
                        task_id,
                        instruction,
                        status: TaskStatus::Failed,
                        result: None,
                        error: Some(error_msg),
                        progress: session.get_progress(),
                    }
                }
            }
        };
        
        // Auto-save after task execution
        let _ = self.save_session_if_enabled(session_id).await;
        
        Ok(execution_result)
    }
    
    /// Get session progress
    pub async fn get_progress(&self, session_id: &str) -> Option<TaskProgress> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).map(|s| s.get_progress())
    }
    
    /// Get all tasks in session
    pub async fn get_tasks(&self, session_id: &str) -> Option<Vec<Task>> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).map(|s| s.tasks.clone())
    }
}

/// Result of executing a task
#[derive(Debug, Clone, Serialize)]
pub struct TaskExecutionResult {
    pub task_id: String,
    pub instruction: String,
    pub status: TaskStatus,
    pub result: Option<String>,
    pub error: Option<String>,
    pub progress: TaskProgress,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_numbered_tasks() {
        let agent = Arc::new(RwLock::new(EnhancedCodingAgent::new()));
        let thinking = Arc::new(ThinkingAgent::new());
        let manager = TaskManager::new(agent, thinking);
        
        let input = "1. Write a function\n2. Test it\n3. Document it";
        let tasks = manager.parse_tasks(input);
        
        assert_eq!(tasks.len(), 3);
        assert!(tasks[0].0.contains("function"));
        assert!(tasks[1].0.contains("Test"));
    }
    
    #[test]
    fn test_task_type_detection() {
        let agent = Arc::new(RwLock::new(EnhancedCodingAgent::new()));
        let thinking = Arc::new(ThinkingAgent::new());
        let manager = TaskManager::new(agent, thinking);
        
        assert_eq!(manager.detect_task_type("Write code for sorting"), TaskType::Coding);
        assert_eq!(manager.detect_task_type("Analyze the data"), TaskType::Analysis);
        assert_eq!(manager.detect_task_type("Research quantum computing"), TaskType::Research);
    }
    
    #[tokio::test]
    async fn test_task_session() {
        let mut session = TaskSession::new("test-123".to_string());
        
        session.add_task("Task 1".to_string(), TaskType::General);
        session.add_task("Task 2".to_string(), TaskType::General);
        
        assert_eq!(session.tasks.len(), 2);
        assert!(!session.is_complete());
        
        session.complete_current_task("Result 1".to_string(), 0.9);
        assert_eq!(session.current_task_index, 1);
        
        let progress = session.get_progress();
        assert_eq!(progress.completed, 1);
        assert_eq!(progress.pending, 1);
    }
}
