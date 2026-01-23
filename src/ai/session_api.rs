//! Session Memory API Endpoints

use actix_web::{get, post, put, delete, web, HttpResponse, Result};
use serde::Deserialize;
use uuid::Uuid;

use super::session_memory::SessionStore;

/// Request structs
#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub title: Option<String>,
    pub user_id: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// Create new session
#[post("/sessions/create")]
pub async fn create_session(
    req: web::Json<CreateSessionRequest>,
    store: web::Data<SessionStore>,
) -> Result<HttpResponse> {
    let session = store.create_session(req.title.clone(), req.user_id.clone(), req.tags.clone());
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "session": session
    })))
}

/// Get session by ID
#[get("/sessions/{id}")]
pub async fn get_session(
    path: web::Path<Uuid>,
    store: web::Data<SessionStore>,
) -> Result<HttpResponse> {
    let session_id = path.into_inner();
    
    match store.get_session(session_id) {
        Some(session) => Ok(HttpResponse::Ok().json(session)),
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Session not found"
        }))),
    }
}

/// List sessions
#[get("/sessions/list")]
pub async fn list_sessions(
    query: web::Query<ListSessionsQuery>,
    store: web::Data<SessionStore>,
) -> Result<HttpResponse> {
    let sessions = store.list_sessions(
        query.user_id.clone(),
        query.include_archived.unwrap_or(false),
    );
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "sessions": sessions,
        "total": sessions.len()
    })))
}

#[derive(Debug, Deserialize)]
pub struct ListSessionsQuery {
    pub user_id: Option<String>,
    pub limit: Option<i64>,
    pub include_archived: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct AddMessageRequest {
    pub role: String,
    pub content: String,
    pub token_count: Option<i32>,
    pub model: Option<String>,
}

/// Add message to session
#[post("/sessions/{id}/messages")]
pub async fn add_message(
    path: web::Path<Uuid>,
    req: web::Json<AddMessageRequest>,
    store: web::Data<SessionStore>,
) -> Result<HttpResponse> {
    let session_id = path.into_inner();
    
    match store.add_message(
        session_id,
        req.role.clone(),
        req.content.clone(),
        req.token_count,
        req.model.clone(),
    ) {
        Some(message) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": message
        }))),
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "success": false,
            "error": "Session not found"
        }))),
    }
}

/// Get messages for session
#[get("/sessions/{id}/messages")]
pub async fn get_messages(
    path: web::Path<Uuid>,
    store: web::Data<SessionStore>,
) -> Result<HttpResponse> {
    let session_id = path.into_inner();
    let messages = store.get_messages(session_id);
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "messages": messages,
        "total": messages.len()
    })))
}

#[derive(Debug, Deserialize)]
pub struct UpdateTitleRequest {
    pub title: String,
}

/// Update session title
#[put("/sessions/{id}/title")]
pub async fn update_title(
    path: web::Path<Uuid>,
    req: web::Json<UpdateTitleRequest>,
    store: web::Data<SessionStore>,
) -> Result<HttpResponse> {
    let session_id = path.into_inner();
    
    if store.update_title(session_id, req.title.clone()) {
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "Title updated"
        })))
    } else {
        Ok(HttpResponse::NotFound().json(serde_json::json!({
            "success": false,
            "error": "Session not found"
        })))
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateSummaryRequest {
    pub summary: String,
}

/// Update session summary
#[put("/sessions/{id}/summary")]
pub async fn update_summary(
    path: web::Path<Uuid>,
    req: web::Json<UpdateSummaryRequest>,
    store: web::Data<SessionStore>,
) -> Result<HttpResponse> {
    let session_id = path.into_inner();
    
    if store.update_summary(session_id, req.summary.clone()) {
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "Summary updated"
        })))
    } else {
        Ok(HttpResponse::NotFound().json(serde_json::json!({
            "success": false,
            "error": "Session not found"
        })))
    }
}

/// Archive session
#[put("/sessions/{id}/archive")]
pub async fn archive_session(
    path: web::Path<Uuid>,
    store: web::Data<SessionStore>,
) -> Result<HttpResponse> {
    let session_id = path.into_inner();
    
    if store.archive_session(session_id) {
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "Session archived"
        })))
    } else {
        Ok(HttpResponse::NotFound().json(serde_json::json!({
            "success": false,
            "error": "Session not found"
        })))
    }
}

/// Delete session
#[delete("/sessions/{id}")]
pub async fn delete_session(
    path: web::Path<Uuid>,
    store: web::Data<SessionStore>,
) -> Result<HttpResponse> {
    let session_id = path.into_inner();
    store.delete_session(session_id);
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Session deleted"
    })))
}

#[derive(Debug, Deserialize)]
pub struct SearchSessionsRequest {
    pub query: String,
    pub user_id: Option<String>,
}

/// Search sessions
#[post("/sessions/search")]
pub async fn search_sessions(
    req: web::Json<SearchSessionsRequest>,
    store: web::Data<SessionStore>,
) -> Result<HttpResponse> {
    let sessions = store.search_sessions(&req.query, req.user_id.clone());
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "sessions": sessions,
        "total": sessions.len()
    })))
}

#[derive(Debug, Deserialize)]
pub struct StatsQuery {
    pub user_id: Option<String>,
}

/// Get session statistics
#[get("/sessions/stats")]
pub async fn get_stats(
    query: web::Query<StatsQuery>,
    store: web::Data<SessionStore>,
) -> Result<HttpResponse> {
    let stats = store.get_stats(query.user_id.clone());
    Ok(HttpResponse::Ok().json(stats))
}

/// Configure session routes
pub fn configure_session_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(create_session)
        .service(get_session)
        .service(list_sessions)
        .service(add_message)
        .service(get_messages)
        .service(update_title)
        .service(update_summary)
        .service(archive_session)
        .service(delete_session)
        .service(search_sessions)
        .service(get_stats);
}
