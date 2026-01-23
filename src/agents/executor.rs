//! Code execution with Docker sandboxing

use crate::agents::error::{AgentError, Result};
use crate::agents::language::Language;
use std::process::Command;
use std::time::Duration;
use serde::Serialize;

/// Result of code execution
#[derive(Debug, Clone, Serialize)]
pub struct ExecutionResult {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
    pub exit_code: Option<i32>,
}

/// Code executor with Docker sandboxing
pub struct CodeExecutor {
    memory_limit: String,
    cpu_limit: String,
    timeout: Duration,
}

impl CodeExecutor {
    pub fn new() -> Self {
        Self {
            memory_limit: "256m".to_string(),
            cpu_limit: "0.5".to_string(),
            timeout: Duration::from_secs(30),
        }
    }
    
    /// Execute code in sandboxed Docker container
    pub fn execute(&self, code: &str, language: Language) -> Result<ExecutionResult> {
        match language {
            Language::Python => self.execute_python(code),
            Language::Elixir => self.execute_elixir(code),
            Language::TypeScript => self.execute_typescript(code),
            Language::Nim => self.execute_nim(code),
            Language::Haxe => self.execute_haxe(code),
            Language::Go => self.execute_go(code),
            Language::Rust => self.execute_rust(code),
            Language::Ruby => self.execute_ruby(code),
            Language::Kotlin => self.execute_kotlin(code),
            Language::JavaScript => self.execute_javascript(code),
            Language::Swift => self.execute_swift(code),
            Language::Zig => self.execute_zig(code),
            Language::Cpp => self.execute_cpp(code),
            Language::C => self.execute_c(code),
            Language::Java => self.execute_java(code),
            _ => Err(AgentError::UnsupportedLanguage(language.name().to_string())),
        }
    }
    
    fn execute_python(&self, code: &str) -> Result<ExecutionResult> {
        self.run_in_docker("python:3.11-slim", vec!["python", "-c", code])
    }
    
    fn execute_elixir(&self, code: &str) -> Result<ExecutionResult> {
        self.run_in_docker("elixir:1.15-alpine", vec!["elixir", "-e", code])
    }
    
    fn execute_typescript(&self, code: &str) -> Result<ExecutionResult> {
        let cmd = format!("npm install -g ts-node typescript >/dev/null 2>&1 && echo '{}' | ts-node", code);
        self.run_in_docker("node:20-alpine", vec!["sh", "-c", &cmd])
    }
    
    fn execute_nim(&self, code: &str) -> Result<ExecutionResult> {
        let cmd = format!("echo '{}' > /tmp/code.nim && nim compile --run /tmp/code.nim 2>&1", code);
        self.run_in_docker("nimlang/nim:2.0.0-alpine", vec!["sh", "-c", &cmd])
    }
    
    fn execute_haxe(&self, code: &str) -> Result<ExecutionResult> {
        let cmd = format!(
            "echo '{}' > Main.hx && haxe -python out.py -main Main 2>&1 && python out.py",
            code
        );
        self.run_in_docker("haxe:4.3-alpine", vec!["sh", "-c", &cmd])
    }
    
    fn execute_go(&self, code: &str) -> Result<ExecutionResult> {
        let cmd = format!("echo '{}' > /tmp/main.go && go run /tmp/main.go 2>&1", code);
        self.run_in_docker("golang:1.21-alpine", vec!["sh", "-c", &cmd])
    }
    
    fn execute_rust(&self, code: &str) -> Result<ExecutionResult> {
        let cmd = format!(
            "echo '{}' > /tmp/main.rs && rustc /tmp/main.rs -o /tmp/main 2>&1 && /tmp/main",
            code
        );
        self.run_in_docker("rust:1.75-alpine", vec!["sh", "-c", &cmd])
    }
    
    fn execute_ruby(&self, code: &str) -> Result<ExecutionResult> {
        self.run_in_docker("ruby:3.2-alpine", vec!["ruby", "-e", code])
    }
    
    fn execute_kotlin(&self, code: &str) -> Result<ExecutionResult> {
        let cmd = format!(
            "echo '{}' > Main.kt && kotlinc Main.kt -include-runtime -d out.jar 2>&1 && java -jar out.jar",
            code
        );
        self.run_in_docker("zenika/kotlin:1.9-jdk17", vec!["sh", "-c", &cmd])
    }
    
    fn execute_javascript(&self, code: &str) -> Result<ExecutionResult> {
        self.run_in_docker("node:20-alpine", vec!["node", "-e", code])
    }
    
    fn execute_swift(&self, code: &str) -> Result<ExecutionResult> {
        let cmd = format!("echo '{}' > /tmp/main.swift && swift /tmp/main.swift 2>&1", code);
        self.run_in_docker("swift:5.9", vec!["sh", "-c", &cmd])
    }
    
    fn execute_zig(&self, code: &str) -> Result<ExecutionResult> {
        let cmd = format!("echo '{}' > /tmp/main.zig && zig run /tmp/main.zig 2>&1", code);
        self.run_in_docker("euantorano/zig:0.11.0", vec!["sh", "-c", &cmd])
    }
    
    fn execute_cpp(&self, code: &str) -> Result<ExecutionResult> {
        let cmd = format!(
            "echo '{}' > /tmp/main.cpp && g++ /tmp/main.cpp -o /tmp/main 2>&1 && /tmp/main",
            code
        );
        self.run_in_docker("gcc:13-alpine", vec!["sh", "-c", &cmd])
    }
    
    fn execute_c(&self, code: &str) -> Result<ExecutionResult> {
        let cmd = format!(
            "echo '{}' > /tmp/main.c && gcc /tmp/main.c -o /tmp/main 2>&1 && /tmp/main",
            code
        );
        self.run_in_docker("gcc:13-alpine", vec!["sh", "-c", &cmd])
    }
    
    fn execute_java(&self, code: &str) -> Result<ExecutionResult> {
        let cmd = format!(
            "echo '{}' > Main.java && javac Main.java 2>&1 && java Main",
            code
        );
        self.run_in_docker("openjdk:21-slim", vec!["sh", "-c", &cmd])
    }
    
    /// Generic Docker execution with security constraints
    fn run_in_docker(&self, image: &str, cmd: Vec<&str>) -> Result<ExecutionResult> {
        let mut docker_cmd = Command::new("docker");
        
        docker_cmd
            .arg("run")
            .arg("--rm")
            .arg("--network=none")                          // No network access
            .arg(format!("--memory={}", self.memory_limit)) // Memory limit
            .arg(format!("--cpus={}", self.cpu_limit))      // CPU limit
            .arg("--pids-limit=100")                        // Process limit
            .arg("--read-only")                             // Read-only filesystem
            .arg("--tmpfs=/tmp:rw,noexec,nosuid,size=50m")  // Temp space
            .arg(format!("--stop-timeout={}", self.timeout.as_secs())) // Timeout
            .arg(image);
        
        for arg in cmd {
            docker_cmd.arg(arg);
        }
        
        let output = docker_cmd
            .output()
            .map_err(|e| AgentError::DockerError(format!("Failed to execute Docker: {}", e)))?;
        
        Ok(ExecutionResult {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            success: output.status.success(),
            exit_code: output.status.code(),
        })
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
    
    #[test]
    #[ignore] // Requires Docker
    fn test_python_execution() {
        let executor = CodeExecutor::new();
        let result = executor.execute("print('Hello, World!')", Language::Python).unwrap();
        assert!(result.success);
        assert!(result.stdout.contains("Hello, World!"));
    }
    
    #[test]
    #[ignore] // Requires Docker
    fn test_rust_execution() {
        let executor = CodeExecutor::new();
        let code = r#"fn main() { println!("Hello from Rust!"); }"#;
        let result = executor.execute(code, Language::Rust).unwrap();
        assert!(result.success);
        assert!(result.stdout.contains("Hello from Rust!"));
    }
}
