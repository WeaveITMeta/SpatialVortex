//! Code Execution API Endpoints
//!
//! Execute code in multiple programming languages safely

use actix_web::{post, web, HttpResponse, Result};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::code_executor::{CodeExecutor, ExecutionRequest};

/// Shared code executor state
pub struct ExecutorState {
    executor: Arc<Mutex<CodeExecutor>>,
}

impl ExecutorState {
    pub fn new() -> Self {
        Self {
            executor: Arc::new(Mutex::new(CodeExecutor::new())),
        }
    }
}

/// Execute code endpoint
#[post("/code/execute")]
pub async fn execute_code(
    req: web::Json<ExecutionRequest>,
    state: web::Data<ExecutorState>,
) -> Result<HttpResponse> {
    let executor = state.executor.lock().await;
    
    match executor.execute(req.into_inner()).await {
        Ok(result) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "success": result.success,
            "stdout": result.stdout,
            "stderr": result.stderr,
            "exit_code": result.exit_code,
            "execution_time_ms": result.execution_time_ms,
            "error": result.error,
        }))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": format!("Execution failed: {}", e)
        }))),
    }
}

/// Get supported languages
#[post("/code/languages")]
pub async fn get_languages() -> Result<HttpResponse> {
    let languages = vec![
        serde_json::json!({
            "name": "Python",
            "value": "python",
            "extension": "py",
            "example": "print('Hello, World!')"
        }),
        serde_json::json!({
            "name": "Rust",
            "value": "rust",
            "extension": "rs",
            "example": "fn main() {\n    println!(\"Hello, World!\");\n}"
        }),
        serde_json::json!({
            "name": "JavaScript",
            "value": "javascript",
            "extension": "js",
            "example": "console.log('Hello, World!');"
        }),
        serde_json::json!({
            "name": "TypeScript",
            "value": "typescript",
            "extension": "ts",
            "example": "console.log('Hello, World!');"
        }),
        serde_json::json!({
            "name": "Go",
            "value": "go",
            "extension": "go",
            "example": "package main\nimport \"fmt\"\nfunc main() {\n    fmt.Println(\"Hello, World!\")\n}"
        }),
        serde_json::json!({
            "name": "Java",
            "value": "java",
            "extension": "java",
            "example": "public class Main {\n    public static void main(String[] args) {\n        System.out.println(\"Hello, World!\");\n    }\n}"
        }),
        serde_json::json!({
            "name": "C++",
            "value": "cpp",
            "extension": "cpp",
            "example": "#include <iostream>\nint main() {\n    std::cout << \"Hello, World!\" << std::endl;\n    return 0;\n}"
        }),
        serde_json::json!({
            "name": "C",
            "value": "c",
            "extension": "c",
            "example": "#include <stdio.h>\nint main() {\n    printf(\"Hello, World!\\n\");\n    return 0;\n}"
        }),
        serde_json::json!({
            "name": "Ruby",
            "value": "ruby",
            "extension": "rb",
            "example": "puts 'Hello, World!'"
        }),
        serde_json::json!({
            "name": "PHP",
            "value": "php",
            "extension": "php",
            "example": "<?php\necho 'Hello, World!';\n?>"
        }),
        serde_json::json!({
            "name": "Shell",
            "value": "shell",
            "extension": "sh",
            "example": "echo 'Hello, World!'"
        }),
    ];
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "languages": languages,
        "total": languages.len()
    })))
}

/// Check Docker availability
#[post("/code/status")]
pub async fn check_status() -> Result<HttpResponse> {
    let docker_available = std::process::Command::new("docker")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "docker_available": docker_available,
        "execution_mode": if docker_available { "docker" } else { "local" },
        "security_level": if docker_available { "high" } else { "medium" },
        "message": if docker_available {
            "Docker is available. Code will execute in isolated containers."
        } else {
            "Docker is not available. Code will execute locally (less secure)."
        }
    })))
}

/// Configure code execution routes
pub fn configure_code_execution_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(execute_code)
        .service(get_languages)
        .service(check_status);
}
