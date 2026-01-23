//! Canvas/Workspace API Endpoints
//!
//! Manage code editing workspaces with version history and collaboration

use actix_web::{get, post, put, delete, web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// In-memory canvas storage (replace with database in production)
pub struct CanvasStore {
    workspaces: Mutex<HashMap<String, Canvas>>,
}

impl CanvasStore {
    pub fn new() -> Self {
        Self {
            workspaces: Mutex::new(HashMap::new()),
        }
    }
}

/// Canvas workspace data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Canvas {
    pub id: String,
    pub name: String,
    pub content: String,
    pub language: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub versions: Vec<CanvasVersion>,
}

/// Canvas version history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasVersion {
    pub id: usize,
    pub content: String,
    pub description: String,
    pub timestamp: DateTime<Utc>,
}

/// Create canvas request
#[derive(Debug, Deserialize)]
pub struct CreateCanvasRequest {
    pub name: String,
    pub content: String,
    pub language: Option<String>,
}

/// Update canvas request
#[derive(Debug, Deserialize)]
pub struct UpdateCanvasRequest {
    pub content: String,
    pub description: Option<String>,
}

/// Create a new canvas workspace
#[post("/canvas/create")]
pub async fn create_canvas(
    req: web::Json<CreateCanvasRequest>,
    store: web::Data<CanvasStore>,
) -> Result<HttpResponse> {
    let canvas_id = Uuid::new_v4().to_string();
    let now = Utc::now();
    
    let canvas = Canvas {
        id: canvas_id.clone(),
        name: req.name.clone(),
        content: req.content.clone(),
        language: req.language.clone().unwrap_or_else(|| "typescript".to_string()),
        created_at: now,
        updated_at: now,
        versions: vec![CanvasVersion {
            id: 1,
            content: req.content.clone(),
            description: "Initial version".to_string(),
            timestamp: now,
        }],
    };
    
    let mut workspaces = store.workspaces.lock().unwrap();
    workspaces.insert(canvas_id.clone(), canvas.clone());
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "canvas": canvas,
        "message": "Canvas created successfully"
    })))
}

/// Get canvas by ID
#[get("/canvas/{id}")]
pub async fn get_canvas(
    path: web::Path<String>,
    store: web::Data<CanvasStore>,
) -> Result<HttpResponse> {
    let canvas_id = path.into_inner();
    let workspaces = store.workspaces.lock().unwrap();
    
    match workspaces.get(&canvas_id) {
        Some(canvas) => Ok(HttpResponse::Ok().json(canvas)),
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Canvas not found"
        }))),
    }
}

/// Update canvas content
#[put("/canvas/{id}")]
pub async fn update_canvas(
    path: web::Path<String>,
    req: web::Json<UpdateCanvasRequest>,
    store: web::Data<CanvasStore>,
) -> Result<HttpResponse> {
    let canvas_id = path.into_inner();
    let mut workspaces = store.workspaces.lock().unwrap();
    
    match workspaces.get_mut(&canvas_id) {
        Some(canvas) => {
            let now = Utc::now();
            let version_id = canvas.versions.len() + 1;
            
            // Add new version
            canvas.versions.push(CanvasVersion {
                id: version_id,
                content: req.content.clone(),
                description: req.description.clone().unwrap_or_else(|| "Update".to_string()),
                timestamp: now,
            });
            
            // Update content
            canvas.content = req.content.clone();
            canvas.updated_at = now;
            
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "canvas": canvas.clone(),
                "message": "Canvas updated successfully"
            })))
        }
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Canvas not found"
        }))),
    }
}

/// Get canvas version history
#[get("/canvas/{id}/history")]
pub async fn get_canvas_history(
    path: web::Path<String>,
    store: web::Data<CanvasStore>,
) -> Result<HttpResponse> {
    let canvas_id = path.into_inner();
    let workspaces = store.workspaces.lock().unwrap();
    
    match workspaces.get(&canvas_id) {
        Some(canvas) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "versions": canvas.versions,
            "total": canvas.versions.len()
        }))),
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Canvas not found"
        }))),
    }
}

/// Get diff between two versions
#[get("/canvas/{id}/diff")]
pub async fn get_canvas_diff(
    path: web::Path<String>,
    query: web::Query<DiffQuery>,
    store: web::Data<CanvasStore>,
) -> Result<HttpResponse> {
    let canvas_id = path.into_inner();
    let workspaces = store.workspaces.lock().unwrap();
    
    match workspaces.get(&canvas_id) {
        Some(canvas) => {
            let from_version = canvas.versions.iter()
                .find(|v| v.id == query.from)
                .map(|v| v.content.clone());
            
            let to_version = canvas.versions.iter()
                .find(|v| v.id == query.to)
                .map(|v| v.content.clone());
            
            match (from_version, to_version) {
                (Some(from), Some(to)) => {
                    // Simple line-based diff
                    let from_lines: Vec<&str> = from.lines().collect();
                    let to_lines: Vec<&str> = to.lines().collect();
                    
                    let mut changes = Vec::new();
                    for (i, line) in to_lines.iter().enumerate() {
                        if i >= from_lines.len() || from_lines[i] != *line {
                            changes.push(serde_json::json!({
                                "line": i + 1,
                                "type": if i >= from_lines.len() { "added" } else { "modified" },
                                "content": line
                            }));
                        }
                    }
                    
                    Ok(HttpResponse::Ok().json(serde_json::json!({
                        "from_version": query.from,
                        "to_version": query.to,
                        "changes": changes,
                        "total_changes": changes.len()
                    })))
                }
                _ => Ok(HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "One or both versions not found"
                }))),
            }
        }
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Canvas not found"
        }))),
    }
}

#[derive(Debug, Deserialize)]
pub struct DiffQuery {
    pub from: usize,
    pub to: usize,
}

/// Delete canvas
#[delete("/canvas/{id}")]
pub async fn delete_canvas(
    path: web::Path<String>,
    store: web::Data<CanvasStore>,
) -> Result<HttpResponse> {
    let canvas_id = path.into_inner();
    let mut workspaces = store.workspaces.lock().unwrap();
    
    match workspaces.remove(&canvas_id) {
        Some(_) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "Canvas deleted successfully"
        }))),
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Canvas not found"
        }))),
    }
}

/// List all canvases
#[get("/canvas/list")]
pub async fn list_canvases(
    store: web::Data<CanvasStore>,
) -> Result<HttpResponse> {
    let workspaces = store.workspaces.lock().unwrap();
    
    let canvas_list: Vec<serde_json::Value> = workspaces.values()
        .map(|canvas| serde_json::json!({
            "id": canvas.id,
            "name": canvas.name,
            "language": canvas.language,
            "created_at": canvas.created_at,
            "updated_at": canvas.updated_at,
            "version_count": canvas.versions.len()
        }))
        .collect();
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "canvases": canvas_list,
        "total": canvas_list.len()
    })))
}

/// Configure Canvas routes
pub fn configure_canvas_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(create_canvas)
        .service(get_canvas)
        .service(update_canvas)
        .service(get_canvas_history)
        .service(get_canvas_diff)
        .service(delete_canvas)
        .service(list_canvases);
}
