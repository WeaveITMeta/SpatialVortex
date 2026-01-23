//! Code Execution Engine
//!
//! Safely executes code in multiple languages using Docker containers

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::time::timeout;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

/// Supported programming languages
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Python,
    Rust,
    JavaScript,
    TypeScript,
    Go,
    Java,
    Cpp,
    C,
    Ruby,
    PHP,
    Shell,
}

impl Language {
    /// Get file extension for language
    pub fn extension(&self) -> &str {
        match self {
            Language::Python => "py",
            Language::Rust => "rs",
            Language::JavaScript => "js",
            Language::TypeScript => "ts",
            Language::Go => "go",
            Language::Java => "java",
            Language::Cpp => "cpp",
            Language::C => "c",
            Language::Ruby => "rb",
            Language::PHP => "php",
            Language::Shell => "sh",
        }
    }
    
    /// Get Docker image for language
    pub fn docker_image(&self) -> &str {
        match self {
            Language::Python => "python:3.11-slim",
            Language::Rust => "rust:1.75-slim",
            Language::JavaScript | Language::TypeScript => "node:20-slim",
            Language::Go => "golang:1.21-alpine",
            Language::Java => "openjdk:21-slim",
            Language::Cpp | Language::C => "gcc:13-slim",
            Language::Ruby => "ruby:3.2-slim",
            Language::PHP => "php:8.3-cli",
            Language::Shell => "bash:latest",
        }
    }
    
    /// Get execution command for language
    pub fn execute_command(&self, filename: &str) -> Vec<String> {
        match self {
            Language::Python => vec!["python".to_string(), filename.to_string()],
            Language::Rust => vec![
                "sh".to_string(),
                "-c".to_string(),
                format!("rustc {} -o /tmp/output && /tmp/output", filename),
            ],
            Language::JavaScript => vec!["node".to_string(), filename.to_string()],
            Language::TypeScript => vec![
                "sh".to_string(),
                "-c".to_string(),
                format!("npx -y tsx {}", filename),
            ],
            Language::Go => vec!["go".to_string(), "run".to_string(), filename.to_string()],
            Language::Java => {
                let classname = filename.strip_suffix(".java").unwrap_or(filename);
                vec![
                    "sh".to_string(),
                    "-c".to_string(),
                    format!("javac {} && java {}", filename, classname),
                ]
            }
            Language::Cpp => vec![
                "sh".to_string(),
                "-c".to_string(),
                format!("g++ {} -o /tmp/output && /tmp/output", filename),
            ],
            Language::C => vec![
                "sh".to_string(),
                "-c".to_string(),
                format!("gcc {} -o /tmp/output && /tmp/output", filename),
            ],
            Language::Ruby => vec!["ruby".to_string(), filename.to_string()],
            Language::PHP => vec!["php".to_string(), filename.to_string()],
            Language::Shell => vec!["bash".to_string(), filename.to_string()],
        }
    }
}

/// Code execution request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRequest {
    pub code: String,
    pub language: Language,
    pub timeout_ms: Option<u64>,
    pub stdin: Option<String>,
}

/// Code execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub execution_time_ms: u64,
    pub error: Option<String>,
}

/// Code Executor
pub struct CodeExecutor {
    temp_dir: PathBuf,
    default_timeout: Duration,
    use_docker: bool,
}

impl CodeExecutor {
    /// Create new code executor
    pub fn new() -> Self {
        Self {
            temp_dir: std::env::temp_dir().join("spatial_vortex_executor"),
            default_timeout: Duration::from_secs(5),
            use_docker: Self::is_docker_available(),
        }
    }
    
    /// Check if Docker is available
    fn is_docker_available() -> bool {
        Command::new("docker")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
    
    /// Execute code
    pub async fn execute(&self, request: ExecutionRequest) -> Result<ExecutionResult> {
        let start_time = std::time::Instant::now();
        
        // Create temp directory
        fs::create_dir_all(&self.temp_dir)?;
        
        // Generate unique filename
        let execution_id = Uuid::new_v4();
        let filename = format!("code_{}.{}", execution_id, request.language.extension());
        let filepath = self.temp_dir.join(&filename);
        
        // Write code to file
        fs::write(&filepath, &request.code)
            .context("Failed to write code to file")?;
        
        // Determine timeout
        let exec_timeout = request.timeout_ms
            .map(Duration::from_millis)
            .unwrap_or(self.default_timeout);
        
        // Execute code
        let result = if self.use_docker {
            self.execute_in_docker(&request, &filepath, &filename, exec_timeout).await
        } else {
            self.execute_locally(&request, &filepath, exec_timeout).await
        };
        
        // Clean up
        let _ = fs::remove_file(&filepath);
        
        // Add execution time
        let mut final_result = result?;
        final_result.execution_time_ms = start_time.elapsed().as_millis() as u64;
        
        Ok(final_result)
    }
    
    /// Execute in Docker container (safer)
    async fn execute_in_docker(
        &self,
        request: &ExecutionRequest,
        filepath: &PathBuf,
        filename: &str,
        exec_timeout: Duration,
    ) -> Result<ExecutionResult> {
        let mut docker_cmd = Command::new("docker");
        docker_cmd
            .arg("run")
            .arg("--rm")
            .arg("--network=none") // No network access
            .arg("--memory=512m") // 512MB memory limit
            .arg("--cpus=1") // 1 CPU limit
            .arg("-v")
            .arg(format!("{}:/workspace/{}", filepath.display(), filename))
            .arg("-w")
            .arg("/workspace")
            .arg(request.language.docker_image());
        
        // Add execution command
        for arg in request.language.execute_command(filename) {
            docker_cmd.arg(arg);
        }
        
        // Set stdin if provided
        if let Some(_stdin_data) = &request.stdin {
            docker_cmd.stdin(Stdio::piped());
        }
        
        docker_cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
        
        // Execute with timeout
        let output = timeout(exec_timeout, async {
            let mut child = docker_cmd.spawn()
                .context("Failed to spawn Docker process")?;
            
            // Write stdin if provided
            if let Some(stdin_data) = &request.stdin {
                if let Some(mut stdin) = child.stdin.take() {
                    use std::io::Write;
                    stdin.write_all(stdin_data.as_bytes())?;
                }
            }
            
            child.wait_with_output()
                .context("Failed to wait for Docker process")
        })
        .await;
        
        match output {
            Ok(Ok(output)) => Ok(ExecutionResult {
                success: output.status.success(),
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                exit_code: output.status.code(),
                execution_time_ms: 0, // Will be set by caller
                error: None,
            }),
            Ok(Err(e)) => Ok(ExecutionResult {
                success: false,
                stdout: String::new(),
                stderr: String::new(),
                exit_code: None,
                execution_time_ms: 0,
                error: Some(format!("Execution failed: {}", e)),
            }),
            Err(_) => Ok(ExecutionResult {
                success: false,
                stdout: String::new(),
                stderr: "Execution timed out".to_string(),
                exit_code: None,
                execution_time_ms: 0,
                error: Some("Timeout exceeded".to_string()),
            }),
        }
    }
    
    /// Execute locally (fallback, less safe)
    async fn execute_locally(
        &self,
        request: &ExecutionRequest,
        filepath: &PathBuf,
        exec_timeout: Duration,
    ) -> Result<ExecutionResult> {
        let filename = filepath.file_name()
            .and_then(|n| n.to_str())
            .context("Invalid filename")?;
        
        let mut cmd = Command::new("sh");
        cmd.arg("-c");
        
        let exec_cmd = request.language.execute_command(filename).join(" ");
        cmd.arg(format!("cd {} && {}", self.temp_dir.display(), exec_cmd));
        
        if let Some(_stdin_data) = &request.stdin {
            cmd.stdin(Stdio::piped());
        }
        
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
        
        let output = timeout(exec_timeout, async {
            let mut child = cmd.spawn()
                .context("Failed to spawn process")?;
            
            if let Some(stdin_data) = &request.stdin {
                if let Some(mut stdin) = child.stdin.take() {
                    use std::io::Write;
                    stdin.write_all(stdin_data.as_bytes())?;
                }
            }
            
            child.wait_with_output()
                .context("Failed to wait for process")
        })
        .await;
        
        match output {
            Ok(Ok(output)) => Ok(ExecutionResult {
                success: output.status.success(),
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                exit_code: output.status.code(),
                execution_time_ms: 0,
                error: None,
            }),
            Ok(Err(e)) => Ok(ExecutionResult {
                success: false,
                stdout: String::new(),
                stderr: String::new(),
                exit_code: None,
                execution_time_ms: 0,
                error: Some(format!("Execution failed: {}", e)),
            }),
            Err(_) => Ok(ExecutionResult {
                success: false,
                stdout: String::new(),
                stderr: "Execution timed out".to_string(),
                exit_code: None,
                execution_time_ms: 0,
                error: Some("Timeout exceeded".to_string()),
            }),
        }
    }
}

impl Default for CodeExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_python_execution() {
        let executor = CodeExecutor::new();
        let request = ExecutionRequest {
            code: "print('Hello from Python!')".to_string(),
            language: Language::Python,
            timeout_ms: Some(5000),
            stdin: None,
        };
        
        let result = executor.execute(request).await.unwrap();
        assert!(result.success);
        assert!(result.stdout.contains("Hello from Python!"));
    }
    
    #[tokio::test]
    async fn test_javascript_execution() {
        let executor = CodeExecutor::new();
        let request = ExecutionRequest {
            code: "console.log('Hello from JavaScript!')".to_string(),
            language: Language::JavaScript,
            timeout_ms: Some(5000),
            stdin: None,
        };
        
        let result = executor.execute(request).await.unwrap();
        assert!(result.success);
        assert!(result.stdout.contains("Hello from JavaScript!"));
    }
}
