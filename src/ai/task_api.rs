//! Task Management API - Multi-step task execution endpoints
//!
//! Provides API endpoints for:
//! - Creating task queues from user instructions
//! - Executing tasks sequentially
//! - Tracking progress across conversation turns
//! - Resuming incomplete tasks

use crate::agents::{TaskManager, Task, TaskProgress, TaskExecutionResult};
use actix_web::{get, post, web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Request to create task queue
#[derive(Debug, Deserialize)]
pub struct CreateTasksRequest {
    pub session_id: String,
    pub instructions: String,
}

/// Response with created task IDs
#[derive(Debug, Serialize)]
pub struct CreateTasksResponse {
    pub session_id: String,
    pub task_ids: Vec<String>,
    pub total_tasks: usize,
    pub tasks: Vec<TaskInfo>,
}

/// Task information for display
#[derive(Debug, Serialize)]
pub struct TaskInfo {
    pub id: String,
    pub instruction: String,
    pub task_type: String,
    pub status: String,
}

/// Request to execute next task
#[derive(Debug, Deserialize)]
pub struct ExecuteTaskRequest {
    pub session_id: String,
}

/// Response with execution result
#[derive(Debug, Serialize)]
pub struct ExecuteTaskResponse {
    pub task_id: String,
    pub instruction: String,
    pub status: String,
    pub result: Option<String>,
    pub error: Option<String>,
    pub progress: ProgressInfo,
}

/// Progress information
#[derive(Debug, Serialize)]
pub struct ProgressInfo {
    pub total: usize,
    pub completed: usize,
    pub failed: usize,
    pub pending: usize,
    pub all_done: bool,
    pub percentage: f32,
}

impl From<TaskProgress> for ProgressInfo {
    fn from(p: TaskProgress) -> Self {
        let percentage = if p.total > 0 {
            (p.completed as f32 / p.total as f32) * 100.0
        } else {
            0.0
        };
        
        Self {
            total: p.total,
            completed: p.completed,
            failed: p.failed,
            pending: p.pending,
            all_done: p.all_done,
            percentage,
        }
    }
}

impl From<TaskExecutionResult> for ExecuteTaskResponse {
    fn from(r: TaskExecutionResult) -> Self {
        Self {
            task_id: r.task_id,
            instruction: r.instruction,
            status: format!("{:?}", r.status),
            result: r.result,
            error: r.error,
            progress: r.progress.into(),
        }
    }
}

impl From<Task> for TaskInfo {
    fn from(t: Task) -> Self {
        Self {
            id: t.id,
            instruction: t.instruction,
            task_type: format!("{:?}", t.task_type),
            status: format!("{:?}", t.status),
        }
    }
}

/// Create task queue from user instructions
/// 
/// POST /api/v1/tasks/create
/// 
/// Parses multiple instructions and creates a task queue.
/// Supports numbered lists, bullet points, and natural language.
#[post("/api/v1/tasks/create")]
pub async fn create_tasks(
    req: web::Json<CreateTasksRequest>,
    task_manager: web::Data<Arc<TaskManager>>,
) -> Result<HttpResponse> {
    // Parse instructions into tasks
    let parsed = task_manager.parse_tasks(&req.instructions);
    
    if parsed.is_empty() {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "No valid tasks found in instructions"
        })));
    }
    
    // Add tasks to session
    let task_ids = task_manager.add_tasks(&req.session_id, parsed).await;
    
    // Get task details for response
    let tasks = task_manager.get_tasks(&req.session_id).await
        .unwrap_or_default()
        .into_iter()
        .map(TaskInfo::from)
        .collect::<Vec<_>>();
    
    Ok(HttpResponse::Ok().json(CreateTasksResponse {
        session_id: req.session_id.clone(),
        task_ids: task_ids.clone(),
        total_tasks: task_ids.len(),
        tasks,
    }))
}

/// Execute next pending task
/// 
/// POST /api/v1/tasks/execute
/// 
/// Executes the next pending task in the queue.
/// Routes to appropriate agent based on task type.
#[post("/api/v1/tasks/execute")]
pub async fn execute_task(
    req: web::Json<ExecuteTaskRequest>,
    task_manager: web::Data<Arc<TaskManager>>,
) -> Result<HttpResponse> {
    match task_manager.execute_next_task(&req.session_id).await {
        Ok(result) => {
            Ok(HttpResponse::Ok().json(ExecuteTaskResponse::from(result)))
        }
        Err(e) => {
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Task execution failed: {}", e)
            })))
        }
    }
}

/// Get task queue progress
/// 
/// GET /api/v1/tasks/progress?session_id=xxx
/// 
/// Returns current progress for a task session.
#[get("/api/v1/tasks/progress")]
pub async fn get_progress(
    query: web::Query<std::collections::HashMap<String, String>>,
    task_manager: web::Data<Arc<TaskManager>>,
) -> Result<HttpResponse> {
    let session_id = query.get("session_id")
        .ok_or_else(|| actix_web::error::ErrorBadRequest("session_id required"))?;
    
    match task_manager.get_progress(session_id).await {
        Some(progress) => {
            Ok(HttpResponse::Ok().json(ProgressInfo::from(progress)))
        }
        None => {
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "Session not found"
            })))
        }
    }
}

/// Get all tasks in session
/// 
/// GET /api/v1/tasks/list?session_id=xxx
/// 
/// Returns all tasks and their current status.
#[get("/api/v1/tasks/list")]
pub async fn list_tasks(
    query: web::Query<std::collections::HashMap<String, String>>,
    task_manager: web::Data<Arc<TaskManager>>,
) -> Result<HttpResponse> {
    let session_id = query.get("session_id")
        .ok_or_else(|| actix_web::error::ErrorBadRequest("session_id required"))?;
    
    match task_manager.get_tasks(session_id).await {
        Some(tasks) => {
            let task_infos: Vec<TaskInfo> = tasks.into_iter()
                .map(TaskInfo::from)
                .collect();
            
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "session_id": session_id,
                "total": task_infos.len(),
                "tasks": task_infos
            })))
        }
        None => {
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "Session not found"
            })))
        }
    }
}

/// Execute all remaining tasks in queue
/// 
/// POST /api/v1/tasks/execute-all
/// 
/// Executes all pending tasks sequentially until complete.
/// Returns results for all tasks.
#[post("/api/v1/tasks/execute-all")]
pub async fn execute_all_tasks(
    req: web::Json<ExecuteTaskRequest>,
    task_manager: web::Data<Arc<TaskManager>>,
) -> Result<HttpResponse> {
    let mut results = Vec::new();
    
    // Execute tasks until no more pending
    loop {
        let progress = task_manager.get_progress(&req.session_id).await;
        
        match progress {
            Some(p) if p.pending > 0 => {
                // Execute next task
                match task_manager.execute_next_task(&req.session_id).await {
                    Ok(result) => {
                        results.push(ExecuteTaskResponse::from(result));
                    }
                    Err(e) => {
                        return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                            "error": format!("Task execution failed: {}", e),
                            "completed_tasks": results.len(),
                            "results": results
                        })));
                    }
                }
            }
            Some(p) if p.all_done => {
                // All tasks complete
                break;
            }
            _ => {
                // No session or no tasks
                return Ok(HttpResponse::NotFound().json(serde_json::json!({
                    "error": "Session not found or no tasks",
                    "results": results
                })));
            }
        }
    }
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "session_id": req.session_id,
        "total_executed": results.len(),
        "results": results
    })))
}

/// Get storage statistics (requires persistence enabled)
/// 
/// GET /api/v1/tasks/stats
/// 
/// Returns statistics about persisted task sessions
#[get("/api/v1/tasks/stats")]
pub async fn get_stats(
    _task_manager: web::Data<Arc<TaskManager>>,
) -> Result<HttpResponse> {
    // Note: This would require exposing the persistence layer
    // For now, return basic in-memory stats
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Storage statistics available when persistence is enabled",
        "feature": "task_persistence"
    })))
}

/// Configure task management routes
pub fn configure_task_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(create_tasks)
       .service(execute_task)
       .service(execute_all_tasks)
       .service(get_progress)
       .service(list_tasks)
       .service(get_stats);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_progress_info_from_task_progress() {
        let progress = TaskProgress {
            total: 5,
            completed: 3,
            failed: 1,
            pending: 1,
            all_done: false,
        };
        
        let info = ProgressInfo::from(progress);
        assert_eq!(info.percentage, 60.0);
        assert_eq!(info.completed, 3);
    }
}
