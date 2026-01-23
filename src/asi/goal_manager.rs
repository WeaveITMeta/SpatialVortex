//! Goal Manager - Autonomous Goal Selection and Pursuit
//!
//! Manages the ASI's goals, including:
//! - Goal creation from observations and curiosity
//! - Priority-based goal selection
//! - Progress tracking
//! - Goal completion and archival

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use uuid::Uuid;

// ============================================================================
// Goal Types
// ============================================================================

/// An autonomous goal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomousGoal {
    pub id: Uuid,
    pub objective: String,
    pub priority: GoalPriority,
    pub importance: f32,
    pub created_at: DateTime<Utc>,
    pub deadline: Option<DateTime<Utc>>,
    pub status: GoalStatus,
    pub progress: f32,  // 0.0 - 1.0
}

/// Goal priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum GoalPriority {
    Critical = 4,
    High = 3,
    Medium = 2,
    Low = 1,
    Background = 0,
}

/// Goal status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoalStatus {
    Pending,
    Active,
    Paused,
    Completed,
    Failed,
    Abandoned,
}

// ============================================================================
// Goal Manager
// ============================================================================

/// Manages autonomous goals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalManager {
    /// Active goals (sorted by priority)
    pub goals: Vec<AutonomousGoal>,
    
    /// Currently active goal
    pub current_goal_id: Option<Uuid>,
    
    /// Completed goals (for learning)
    pub completed_goals: VecDeque<AutonomousGoal>,
    
    /// Maximum completed goals to retain
    max_completed: usize,
    
    /// Statistics
    pub stats: GoalStats,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GoalStats {
    pub total_created: u64,
    pub total_completed: u64,
    pub total_failed: u64,
    pub total_abandoned: u64,
    pub avg_completion_time_hours: f64,
}

impl GoalManager {
    pub fn new() -> Self {
        Self {
            goals: Vec::new(),
            current_goal_id: None,
            completed_goals: VecDeque::with_capacity(100),
            max_completed: 100,
            stats: GoalStats::default(),
        }
    }
    
    /// Add a new goal
    pub fn add_goal(&mut self, goal: AutonomousGoal) {
        self.goals.push(goal);
        self.stats.total_created += 1;
        
        // Sort by priority (highest first)
        self.goals.sort_by(|a, b| b.priority.cmp(&a.priority));
    }
    
    /// Create and add a goal from text
    pub fn create_goal(&mut self, objective: &str, priority: GoalPriority) -> Uuid {
        let goal = AutonomousGoal {
            id: Uuid::new_v4(),
            objective: objective.to_string(),
            priority,
            importance: match priority {
                GoalPriority::Critical => 1.0,
                GoalPriority::High => 0.8,
                GoalPriority::Medium => 0.5,
                GoalPriority::Low => 0.3,
                GoalPriority::Background => 0.1,
            },
            created_at: Utc::now(),
            deadline: None,
            status: GoalStatus::Pending,
            progress: 0.0,
        };
        
        let id = goal.id;
        self.add_goal(goal);
        id
    }
    
    /// Get current active goal
    pub fn get_current_goal(&self) -> Option<AutonomousGoal> {
        self.current_goal_id.and_then(|id| {
            self.goals.iter().find(|g| g.id == id).cloned()
        })
    }
    
    /// Activate the highest priority pending goal
    pub fn activate_next_goal(&mut self) -> Option<Uuid> {
        // Find highest priority pending goal
        if let Some(goal) = self.goals.iter_mut()
            .filter(|g| g.status == GoalStatus::Pending)
            .max_by(|a, b| a.priority.cmp(&b.priority))
        {
            goal.status = GoalStatus::Active;
            self.current_goal_id = Some(goal.id);
            return Some(goal.id);
        }
        
        None
    }
    
    /// Update progress on a goal
    pub fn update_progress(&mut self, goal_id: Option<Uuid>, delta: f32) {
        let id = goal_id.or(self.current_goal_id);
        
        if let Some(id) = id {
            if let Some(goal) = self.goals.iter_mut().find(|g| g.id == id) {
                goal.progress = (goal.progress + delta).min(1.0);
                
                // Check for completion
                if goal.progress >= 1.0 {
                    self.complete_goal(id);
                }
            }
        }
    }
    
    /// Mark a goal as completed
    pub fn complete_goal(&mut self, goal_id: Uuid) {
        if let Some(idx) = self.goals.iter().position(|g| g.id == goal_id) {
            let mut goal = self.goals.remove(idx);
            goal.status = GoalStatus::Completed;
            goal.progress = 1.0;
            
            // Archive
            self.completed_goals.push_front(goal);
            while self.completed_goals.len() > self.max_completed {
                self.completed_goals.pop_back();
            }
            
            self.stats.total_completed += 1;
            
            // Clear current if this was it
            if self.current_goal_id == Some(goal_id) {
                self.current_goal_id = None;
            }
        }
    }
    
    /// Mark a goal as failed
    pub fn fail_goal(&mut self, goal_id: Uuid, reason: &str) {
        if let Some(idx) = self.goals.iter().position(|g| g.id == goal_id) {
            let mut goal = self.goals.remove(idx);
            goal.status = GoalStatus::Failed;
            
            tracing::warn!("Goal failed: {} - {}", goal.objective, reason);
            
            self.completed_goals.push_front(goal);
            while self.completed_goals.len() > self.max_completed {
                self.completed_goals.pop_back();
            }
            
            self.stats.total_failed += 1;
            
            if self.current_goal_id == Some(goal_id) {
                self.current_goal_id = None;
            }
        }
    }
    
    /// Abandon a goal
    pub fn abandon_goal(&mut self, goal_id: Uuid) {
        if let Some(idx) = self.goals.iter().position(|g| g.id == goal_id) {
            let mut goal = self.goals.remove(idx);
            goal.status = GoalStatus::Abandoned;
            
            self.completed_goals.push_front(goal);
            self.stats.total_abandoned += 1;
            
            if self.current_goal_id == Some(goal_id) {
                self.current_goal_id = None;
            }
        }
    }
    
    /// Pause current goal
    pub fn pause_current(&mut self) {
        if let Some(id) = self.current_goal_id {
            if let Some(goal) = self.goals.iter_mut().find(|g| g.id == id) {
                goal.status = GoalStatus::Paused;
            }
            self.current_goal_id = None;
        }
    }
    
    /// Get all pending goals
    pub fn pending_goals(&self) -> Vec<&AutonomousGoal> {
        self.goals.iter()
            .filter(|g| g.status == GoalStatus::Pending)
            .collect()
    }
    
    /// Get goal by ID
    pub fn get_goal(&self, id: Uuid) -> Option<&AutonomousGoal> {
        self.goals.iter().find(|g| g.id == id)
    }
    
    /// Check if any goals are active
    pub fn has_active_goal(&self) -> bool {
        self.current_goal_id.is_some()
    }
    
    /// Get success rate
    pub fn success_rate(&self) -> f32 {
        let total = self.stats.total_completed + self.stats.total_failed;
        if total == 0 {
            0.5
        } else {
            self.stats.total_completed as f32 / total as f32
        }
    }
}

impl Default for GoalManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_goal_creation() {
        let mut manager = GoalManager::new();
        
        let id = manager.create_goal("Test objective", GoalPriority::High);
        
        assert!(manager.get_goal(id).is_some());
        assert_eq!(manager.goals.len(), 1);
    }
    
    #[test]
    fn test_goal_priority_sorting() {
        let mut manager = GoalManager::new();
        
        manager.create_goal("Low priority", GoalPriority::Low);
        manager.create_goal("High priority", GoalPriority::High);
        manager.create_goal("Medium priority", GoalPriority::Medium);
        
        // Should be sorted by priority
        assert_eq!(manager.goals[0].priority, GoalPriority::High);
        assert_eq!(manager.goals[1].priority, GoalPriority::Medium);
        assert_eq!(manager.goals[2].priority, GoalPriority::Low);
    }
    
    #[test]
    fn test_goal_activation() {
        let mut manager = GoalManager::new();
        
        manager.create_goal("Goal 1", GoalPriority::Low);
        manager.create_goal("Goal 2", GoalPriority::High);
        
        let activated = manager.activate_next_goal();
        
        assert!(activated.is_some());
        let current = manager.get_current_goal().unwrap();
        assert_eq!(current.priority, GoalPriority::High);
    }
    
    #[test]
    fn test_goal_completion() {
        let mut manager = GoalManager::new();
        
        let id = manager.create_goal("Test", GoalPriority::Medium);
        manager.activate_next_goal();
        
        manager.update_progress(Some(id), 1.0);
        
        assert!(manager.current_goal_id.is_none());
        assert_eq!(manager.stats.total_completed, 1);
        assert_eq!(manager.completed_goals.len(), 1);
    }
}
