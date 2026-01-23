//! ASI Bootstrap - Initialize and Configure the Autonomous System
//!
//! This module provides factory functions to create a fully-configured
//! ASICore with sensors, actuators, and default goals.
//!
//! # Example
//!
//! ```no_run
//! use spatial_vortex::asi::bootstrap::ASIBuilder;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let asi = ASIBuilder::new()
//!         .with_file_watching(vec!["./src".into()])
//!         .with_shell_access("./")
//!         .with_http_access(vec!["localhost".into()])
//!         .with_default_goals()
//!         .build()
//!         .await?;
//!     
//!     // Start autonomous loop
//!     asi.start().await?;
//!     Ok(())
//! }
//! ```

use super::core::{ASICore, ASIConfig, SafetyConfig};
use super::world_interface::{
    TimeSensor, FileSystemSensor, ProcessSensor,
    ShellActuator, FileSystemActuator, HttpActuator,
};
use super::goal_manager::GoalPriority;

use anyhow::Result;
use std::path::PathBuf;

// ============================================================================
// Builder Pattern for ASI Configuration
// ============================================================================

/// Builder for creating a configured ASICore
pub struct ASIBuilder {
    config: ASIConfig,
    
    // Sensors
    enable_time_sensor: bool,
    file_watch_paths: Vec<PathBuf>,
    process_watch_list: Vec<String>,
    
    // Actuators
    shell_working_dir: Option<PathBuf>,
    shell_timeout: u64,
    fs_allowed_paths: Vec<PathBuf>,
    http_allowed_domains: Vec<String>,
    
    // Goals
    initial_goals: Vec<(String, GoalPriority)>,
}

impl ASIBuilder {
    /// Create a new builder with default configuration
    pub fn new() -> Self {
        Self {
            config: ASIConfig::default(),
            enable_time_sensor: true,
            file_watch_paths: Vec::new(),
            process_watch_list: Vec::new(),
            shell_working_dir: None,
            shell_timeout: 30,
            fs_allowed_paths: Vec::new(),
            http_allowed_domains: Vec::new(),
            initial_goals: Vec::new(),
        }
    }
    
    /// Set the storage path for persistent identity
    pub fn with_storage_path(mut self, path: PathBuf) -> Self {
        self.config.storage_path = path;
        self
    }
    
    /// Set the loop interval in milliseconds
    pub fn with_loop_interval(mut self, ms: u64) -> Self {
        self.config.loop_interval_ms = ms;
        self
    }
    
    /// Enable/disable autonomous goal pursuit
    pub fn with_autonomous_goals(mut self, enabled: bool) -> Self {
        self.config.autonomous_goals = enabled;
        self
    }
    
    /// Enable/disable self-modification (dangerous!)
    pub fn with_self_modification(mut self, enabled: bool) -> Self {
        self.config.self_modification = enabled;
        self
    }
    
    /// Configure safety settings
    pub fn with_safety(mut self, safety: SafetyConfig) -> Self {
        self.config.safety = safety;
        self
    }
    
    // ========================================================================
    // Sensor Configuration
    // ========================================================================
    
    /// Enable time sensor (enabled by default)
    pub fn with_time_sensor(mut self, enabled: bool) -> Self {
        self.enable_time_sensor = enabled;
        self
    }
    
    /// Add file system watching for specified paths
    pub fn with_file_watching(mut self, paths: Vec<PathBuf>) -> Self {
        self.file_watch_paths = paths;
        self
    }
    
    /// Add process monitoring for specified process names
    pub fn with_process_monitoring(mut self, processes: Vec<String>) -> Self {
        self.process_watch_list = processes;
        self
    }
    
    // ========================================================================
    // Actuator Configuration
    // ========================================================================
    
    /// Enable shell command execution
    pub fn with_shell_access(mut self, working_dir: &str) -> Self {
        self.shell_working_dir = Some(PathBuf::from(working_dir));
        self
    }
    
    /// Set shell command timeout
    pub fn with_shell_timeout(mut self, seconds: u64) -> Self {
        self.shell_timeout = seconds;
        self
    }
    
    /// Enable file system access for specified paths
    pub fn with_filesystem_access(mut self, paths: Vec<PathBuf>) -> Self {
        self.fs_allowed_paths = paths;
        self
    }
    
    /// Enable HTTP access for specified domains
    pub fn with_http_access(mut self, domains: Vec<String>) -> Self {
        self.http_allowed_domains = domains;
        self
    }
    
    // ========================================================================
    // Goal Configuration
    // ========================================================================
    
    /// Add an initial goal
    pub fn with_goal(mut self, objective: &str, priority: GoalPriority) -> Self {
        self.initial_goals.push((objective.to_string(), priority));
        self
    }
    
    /// Add default goals for a helpful ASI
    pub fn with_default_goals(mut self) -> Self {
        self.initial_goals.extend(vec![
            ("Monitor system health and report anomalies".to_string(), GoalPriority::Background),
            ("Learn from interactions to improve responses".to_string(), GoalPriority::Background),
            ("Identify opportunities to help the user".to_string(), GoalPriority::Low),
        ]);
        self
    }
    
    // ========================================================================
    // Build
    // ========================================================================
    
    /// Build the configured ASICore
    pub async fn build(self) -> Result<ASICore> {
        let mut asi = ASICore::new(self.config).await?;
        
        // Add sensors
        if self.enable_time_sensor {
            asi.add_sensor(Box::new(TimeSensor::new()));
        }
        
        if !self.file_watch_paths.is_empty() {
            asi.add_sensor(Box::new(FileSystemSensor::new(self.file_watch_paths)));
        }
        
        if !self.process_watch_list.is_empty() {
            asi.add_sensor(Box::new(ProcessSensor::new(self.process_watch_list)));
        }
        
        // Add actuators
        if let Some(working_dir) = self.shell_working_dir {
            asi.add_actuator(Box::new(
                ShellActuator::new(working_dir).with_timeout(self.shell_timeout)
            ));
        }
        
        if !self.fs_allowed_paths.is_empty() {
            asi.add_actuator(Box::new(FileSystemActuator::new(self.fs_allowed_paths)));
        }
        
        if !self.http_allowed_domains.is_empty() {
            asi.add_actuator(Box::new(HttpActuator::new(self.http_allowed_domains)));
        }
        
        // Add initial goals
        {
            let mut gm = asi.goal_manager.write().await;
            for (objective, priority) in self.initial_goals {
                gm.create_goal(&objective, priority);
            }
        }
        
        Ok(asi)
    }
}

impl Default for ASIBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Preset Configurations
// ============================================================================

/// Create a minimal ASI for testing (no external access)
pub async fn create_minimal_asi() -> Result<ASICore> {
    ASIBuilder::new()
        .with_time_sensor(true)
        .with_autonomous_goals(false)
        .build()
        .await
}

/// Create a development ASI with local access
pub async fn create_dev_asi(project_path: &str) -> Result<ASICore> {
    ASIBuilder::new()
        .with_storage_path(PathBuf::from(project_path).join(".asi_data"))
        .with_file_watching(vec![PathBuf::from(project_path).join("src")])
        .with_shell_access(project_path)
        .with_filesystem_access(vec![PathBuf::from(project_path)])
        .with_process_monitoring(vec!["cargo".to_string(), "rust-analyzer".to_string()])
        .with_default_goals()
        .with_goal("Assist with code development", GoalPriority::Medium)
        .build()
        .await
}

/// Create a full ASI with network access (use with caution)
pub async fn create_full_asi(
    project_path: &str,
    allowed_domains: Vec<String>,
) -> Result<ASICore> {
    ASIBuilder::new()
        .with_storage_path(PathBuf::from(project_path).join(".asi_data"))
        .with_loop_interval(500)  // Faster loop
        .with_file_watching(vec![PathBuf::from(project_path)])
        .with_shell_access(project_path)
        .with_filesystem_access(vec![PathBuf::from(project_path)])
        .with_http_access(allowed_domains)
        .with_autonomous_goals(true)
        .with_default_goals()
        .build()
        .await
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_minimal_asi() {
        let asi = create_minimal_asi().await;
        assert!(asi.is_ok());
    }
    
    #[tokio::test]
    async fn test_builder_pattern() {
        let asi = ASIBuilder::new()
            .with_loop_interval(2000)
            .with_time_sensor(true)
            .with_goal("Test goal", GoalPriority::High)
            .build()
            .await;
        
        assert!(asi.is_ok());
    }
}
