//! World Interface - Sensors and Actuators for Real-World Interaction
//!
//! This module provides the ASI with the ability to perceive and act upon
//! the real world through a unified interface.
//!
//! # Sensors (Perception)
//! - FileSystemSensor: Watch directories for changes
//! - TimeSensor: Temporal awareness
//! - NetworkSensor: Monitor network state
//! - ProcessSensor: Track running processes
//!
//! # Actuators (Action)
//! - ShellActuator: Execute shell commands
//! - FileSystemActuator: Read/write files
//! - NetworkActuator: Make HTTP requests
//! - VMActuator: Control VirtualBox VMs

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc, Timelike};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use uuid::Uuid;

// ============================================================================
// Core Traits
// ============================================================================

/// Observation from a sensor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    pub id: Uuid,
    pub source: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub importance: f32,
    pub metadata: HashMap<String, String>,
}

impl Observation {
    pub fn new(source: &str, content: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            source: source.to_string(),
            content: content.to_string(),
            timestamp: Utc::now(),
            importance: 0.5,
            metadata: HashMap::new(),
        }
    }
    
    pub fn with_importance(mut self, importance: f32) -> Self {
        self.importance = importance;
        self
    }
    
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

/// Result of an action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub id: Uuid,
    pub action_type: String,
    pub target: String,
    pub outcome: String,
    pub success: bool,
    pub timestamp: DateTime<Utc>,
    pub duration_ms: u64,
    pub goal_id: Option<Uuid>,
    pub metadata: HashMap<String, String>,
}

impl ActionResult {
    pub fn success(action_type: &str, target: &str, outcome: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            action_type: action_type.to_string(),
            target: target.to_string(),
            outcome: outcome.to_string(),
            success: true,
            timestamp: Utc::now(),
            duration_ms: 0,
            goal_id: None,
            metadata: HashMap::new(),
        }
    }
    
    pub fn failure(action_type: &str, target: &str, error: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            action_type: action_type.to_string(),
            target: target.to_string(),
            outcome: error.to_string(),
            success: false,
            timestamp: Utc::now(),
            duration_ms: 0,
            goal_id: None,
            metadata: HashMap::new(),
        }
    }
}

/// Action to execute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub id: Uuid,
    pub action_type: String,
    pub target: String,
    pub parameters: serde_json::Value,
}

/// Sensor trait for perception
#[async_trait]
pub trait Sensor: Send + Sync {
    /// Get sensor name
    fn name(&self) -> &str;
    
    /// Perceive environment and return observations
    async fn perceive(&self) -> Result<Vec<Observation>>;
    
    /// Check if sensor is healthy
    fn is_healthy(&self) -> bool {
        true
    }
}

/// Actuator trait for action
#[async_trait]
pub trait Actuator: Send + Sync {
    /// Get actuator name
    fn name(&self) -> &str;
    
    /// Check if this actuator can handle the action
    fn can_handle(&self, action: &Action) -> bool;
    
    /// Execute an action
    async fn act(&self, action: Action) -> Result<ActionResult>;
    
    /// Check if actuator is healthy
    fn is_healthy(&self) -> bool {
        true
    }
}

// ============================================================================
// Sensors
// ============================================================================

/// Time sensor - provides temporal awareness
pub struct TimeSensor {
    name: String,
    last_check: std::sync::Mutex<DateTime<Utc>>,
}

impl TimeSensor {
    pub fn new() -> Self {
        Self {
            name: "time_sensor".to_string(),
            last_check: std::sync::Mutex::new(Utc::now()),
        }
    }
}

impl Default for TimeSensor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Sensor for TimeSensor {
    fn name(&self) -> &str {
        &self.name
    }
    
    async fn perceive(&self) -> Result<Vec<Observation>> {
        let now = Utc::now();
        let mut last = self.last_check.lock().unwrap();
        let elapsed = now.signed_duration_since(*last);
        *last = now;
        
        let mut observations = Vec::new();
        
        // Basic time observation
        observations.push(
            Observation::new("time", &format!("Current time: {}", now.format("%Y-%m-%d %H:%M:%S UTC")))
                .with_importance(0.3)
        );
        
        // Check for significant time events
        let hour = now.hour();
        if hour == 0 {
            observations.push(
                Observation::new("time", "New day started")
                    .with_importance(0.7)
            );
        }
        
        // Report elapsed time if significant
        if elapsed.num_minutes() > 5 {
            observations.push(
                Observation::new("time", &format!("{} minutes elapsed since last check", elapsed.num_minutes()))
                    .with_importance(0.5)
            );
        }
        
        Ok(observations)
    }
}

/// File system sensor - watches directories for changes
pub struct FileSystemSensor {
    name: String,
    watch_paths: Vec<PathBuf>,
    last_state: std::sync::Mutex<HashMap<PathBuf, std::time::SystemTime>>,
}

impl FileSystemSensor {
    pub fn new(paths: Vec<PathBuf>) -> Self {
        Self {
            name: "filesystem_sensor".to_string(),
            watch_paths: paths,
            last_state: std::sync::Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl Sensor for FileSystemSensor {
    fn name(&self) -> &str {
        &self.name
    }
    
    async fn perceive(&self) -> Result<Vec<Observation>> {
        let mut observations = Vec::new();
        let mut last_state = self.last_state.lock().unwrap();
        
        for path in &self.watch_paths {
            if path.exists() {
                if let Ok(metadata) = std::fs::metadata(path) {
                    if let Ok(modified) = metadata.modified() {
                        let prev = last_state.get(path);
                        
                        if prev.map(|p| *p != modified).unwrap_or(true) {
                            observations.push(
                                Observation::new(
                                    "filesystem",
                                    &format!("File changed: {}", path.display())
                                )
                                .with_importance(0.6)
                                .with_metadata("path", &path.to_string_lossy())
                            );
                            
                            last_state.insert(path.clone(), modified);
                        }
                    }
                }
            }
        }
        
        Ok(observations)
    }
}

/// Process sensor - monitors running processes
pub struct ProcessSensor {
    name: String,
    watch_processes: Vec<String>,
}

impl ProcessSensor {
    pub fn new(processes: Vec<String>) -> Self {
        Self {
            name: "process_sensor".to_string(),
            watch_processes: processes,
        }
    }
}

#[async_trait]
impl Sensor for ProcessSensor {
    fn name(&self) -> &str {
        &self.name
    }
    
    async fn perceive(&self) -> Result<Vec<Observation>> {
        let mut observations = Vec::new();
        
        // Get running processes (platform-specific)
        #[cfg(target_os = "windows")]
        {
            let output = Command::new("tasklist")
                .output();
            
            if let Ok(output) = output {
                let stdout = String::from_utf8_lossy(&output.stdout);
                
                for process in &self.watch_processes {
                    if stdout.contains(process) {
                        observations.push(
                            Observation::new(
                                "process",
                                &format!("Process running: {}", process)
                            )
                            .with_importance(0.4)
                        );
                    }
                }
            }
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            let output = Command::new("ps")
                .args(["-e"])
                .output();
            
            if let Ok(output) = output {
                let stdout = String::from_utf8_lossy(&output.stdout);
                
                for process in &self.watch_processes {
                    if stdout.contains(process) {
                        observations.push(
                            Observation::new(
                                "process",
                                &format!("Process running: {}", process)
                            )
                            .with_importance(0.4)
                        );
                    }
                }
            }
        }
        
        Ok(observations)
    }
}

// ============================================================================
// Actuators
// ============================================================================

/// Shell actuator - executes shell commands
pub struct ShellActuator {
    name: String,
    working_dir: PathBuf,
    timeout_seconds: u64,
}

impl ShellActuator {
    pub fn new(working_dir: PathBuf) -> Self {
        Self {
            name: "shell_actuator".to_string(),
            working_dir,
            timeout_seconds: 30,
        }
    }
    
    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = seconds;
        self
    }
}

#[async_trait]
impl Actuator for ShellActuator {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn can_handle(&self, action: &Action) -> bool {
        action.action_type == "shell" || action.action_type == "execute"
    }
    
    async fn act(&self, action: Action) -> Result<ActionResult> {
        let start = std::time::Instant::now();
        
        let command = action.target.clone();
        
        #[cfg(target_os = "windows")]
        let output = Command::new("cmd")
            .args(["/C", &command])
            .current_dir(&self.working_dir)
            .output();
        
        #[cfg(not(target_os = "windows"))]
        let output = Command::new("sh")
            .args(["-c", &command])
            .current_dir(&self.working_dir)
            .output();
        
        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                
                let success = output.status.success();
                let outcome = if success {
                    stdout.to_string()
                } else {
                    format!("Error: {}", stderr)
                };
                
                Ok(ActionResult {
                    id: Uuid::new_v4(),
                    action_type: "shell".to_string(),
                    target: command,
                    outcome,
                    success,
                    timestamp: Utc::now(),
                    duration_ms: start.elapsed().as_millis() as u64,
                    goal_id: None,
                    metadata: HashMap::new(),
                })
            }
            Err(e) => {
                Ok(ActionResult::failure("shell", &command, &e.to_string()))
            }
        }
    }
}

/// File system actuator - read/write files
pub struct FileSystemActuator {
    name: String,
    allowed_paths: Vec<PathBuf>,
}

impl FileSystemActuator {
    pub fn new(allowed_paths: Vec<PathBuf>) -> Self {
        Self {
            name: "filesystem_actuator".to_string(),
            allowed_paths,
        }
    }
    
    fn is_path_allowed(&self, path: &PathBuf) -> bool {
        self.allowed_paths.iter().any(|allowed| path.starts_with(allowed))
    }
}

#[async_trait]
impl Actuator for FileSystemActuator {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn can_handle(&self, action: &Action) -> bool {
        matches!(action.action_type.as_str(), "read_file" | "write_file" | "delete_file" | "list_dir")
    }
    
    async fn act(&self, action: Action) -> Result<ActionResult> {
        let path = PathBuf::from(&action.target);
        
        if !self.is_path_allowed(&path) {
            return Ok(ActionResult::failure(
                &action.action_type,
                &action.target,
                "Path not in allowed list"
            ));
        }
        
        match action.action_type.as_str() {
            "read_file" => {
                match std::fs::read_to_string(&path) {
                    Ok(content) => Ok(ActionResult::success("read_file", &action.target, &content)),
                    Err(e) => Ok(ActionResult::failure("read_file", &action.target, &e.to_string())),
                }
            }
            "write_file" => {
                let content = action.parameters["content"].as_str().unwrap_or("");
                match std::fs::write(&path, content) {
                    Ok(()) => Ok(ActionResult::success("write_file", &action.target, "File written")),
                    Err(e) => Ok(ActionResult::failure("write_file", &action.target, &e.to_string())),
                }
            }
            "list_dir" => {
                match std::fs::read_dir(&path) {
                    Ok(entries) => {
                        let files: Vec<String> = entries
                            .filter_map(|e| e.ok())
                            .map(|e| e.path().to_string_lossy().to_string())
                            .collect();
                        Ok(ActionResult::success("list_dir", &action.target, &files.join("\n")))
                    }
                    Err(e) => Ok(ActionResult::failure("list_dir", &action.target, &e.to_string())),
                }
            }
            _ => Ok(ActionResult::failure(&action.action_type, &action.target, "Unknown action")),
        }
    }
}

/// HTTP actuator - make network requests
pub struct HttpActuator {
    name: String,
    client: reqwest::Client,
    allowed_domains: Vec<String>,
}

impl HttpActuator {
    pub fn new(allowed_domains: Vec<String>) -> Self {
        Self {
            name: "http_actuator".to_string(),
            client: reqwest::Client::new(),
            allowed_domains,
        }
    }
    
    fn is_domain_allowed(&self, url: &str) -> bool {
        if self.allowed_domains.is_empty() {
            return true;  // No restrictions
        }
        self.allowed_domains.iter().any(|d| url.contains(d))
    }
}

#[async_trait]
impl Actuator for HttpActuator {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn can_handle(&self, action: &Action) -> bool {
        matches!(action.action_type.as_str(), "http_get" | "http_post")
    }
    
    async fn act(&self, action: Action) -> Result<ActionResult> {
        if !self.is_domain_allowed(&action.target) {
            return Ok(ActionResult::failure(
                &action.action_type,
                &action.target,
                "Domain not allowed"
            ));
        }
        
        let start = std::time::Instant::now();
        
        let result = match action.action_type.as_str() {
            "http_get" => {
                self.client.get(&action.target).send().await
            }
            "http_post" => {
                let body = action.parameters["body"].as_str().unwrap_or("");
                self.client.post(&action.target).body(body.to_string()).send().await
            }
            _ => return Ok(ActionResult::failure(&action.action_type, &action.target, "Unknown HTTP method")),
        };
        
        match result {
            Ok(response) => {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                
                Ok(ActionResult {
                    id: Uuid::new_v4(),
                    action_type: action.action_type,
                    target: action.target,
                    outcome: body,
                    success: status.is_success(),
                    timestamp: Utc::now(),
                    duration_ms: start.elapsed().as_millis() as u64,
                    goal_id: None,
                    metadata: HashMap::from([("status".to_string(), status.to_string())]),
                })
            }
            Err(e) => {
                Ok(ActionResult::failure(&action.action_type, &action.target, &e.to_string()))
            }
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
    async fn test_time_sensor() {
        let sensor = TimeSensor::new();
        let observations = sensor.perceive().await.unwrap();
        
        assert!(!observations.is_empty());
        assert!(observations[0].content.contains("Current time"));
    }
    
    #[tokio::test]
    async fn test_observation_builder() {
        let obs = Observation::new("test", "test content")
            .with_importance(0.9)
            .with_metadata("key", "value");
        
        assert_eq!(obs.importance, 0.9);
        assert_eq!(obs.metadata.get("key"), Some(&"value".to_string()));
    }
    
    #[tokio::test]
    async fn test_action_result() {
        let success = ActionResult::success("test", "target", "outcome");
        assert!(success.success);
        
        let failure = ActionResult::failure("test", "target", "error");
        assert!(!failure.success);
    }
}
