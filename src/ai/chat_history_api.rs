//! Chat History API Endpoints
//!
//! Endpoints for retrieving, listing, and managing chat sessions

use actix_web::{get, post, delete, web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::coding_api::CodingAgentState;

/// List all chat sessions for a user
/// 
/// GET /api/v1/chat/sessions?user_id=xxx
#[get("/chat/sessions")]
pub async fn list_user_sessions(
    query: web::Query<std::collections::HashMap<String, String>>,
    state: web::Data<Arc<Mutex<CodingAgentState>>>,
) -> Result<HttpResponse> {
    let user_id = query.get("user_id")
        .ok_or_else(|| actix_web::error::ErrorBadRequest("user_id required"))?;
    
    let state_lock = state.lock().await;
    let sessions = state_lock.history.get_user_sessions(user_id).await;
    
    // Convert to response format
    let session_summaries: Vec<SessionSummary> = sessions.iter()
        .map(|s| SessionSummary {
            session_id: s.session_id.clone(),
            user_id: s.user_id.clone(),
            created_at: s.created_at.to_rfc3339(),
            last_activity: s.last_activity.to_rfc3339(),
            message_count: s.messages.len(),
            preview: s.messages.first()
                .map(|m| {
                    let preview_len = m.content.len().min(100);
                    m.content[..preview_len].to_string()
                })
                .unwrap_or_else(|| "Empty conversation".to_string()),
        })
        .collect();
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "user_id": user_id,
        "total_sessions": session_summaries.len(),
        "sessions": session_summaries
    })))
}

/// Get full chat history for a session
/// 
/// GET /api/v1/chat/history/{session_id}
#[get("/chat/history/{session_id}")]
pub async fn get_session_history(
    session_id: web::Path<String>,
    state: web::Data<Arc<Mutex<CodingAgentState>>>,
) -> Result<HttpResponse> {
    let state_lock = state.lock().await;
    
    match state_lock.history.get_session(&session_id).await {
        Some(session) => {
            Ok(HttpResponse::Ok().json(SessionHistoryResponse {
                session_id: session.session_id.clone(),
                user_id: session.user_id.clone(),
                created_at: session.created_at.to_rfc3339(),
                last_activity: session.last_activity.to_rfc3339(),
                message_count: session.messages.len(),
                messages: session.messages.iter().map(|m| MessageResponse {
                    role: format!("{:?}", m.role).to_lowercase(),
                    content: m.content.clone(),
                    timestamp: m.timestamp.to_rfc3339(),
                    confidence: m.metadata.as_ref().and_then(|meta| meta.confidence),
                    language: m.metadata.as_ref().and_then(|meta| meta.language.clone()),
                }).collect(),
            }))
        }
        None => {
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "Session not found",
                "session_id": session_id.as_str()
            })))
        }
    }
}

/// Delete a chat session
/// 
/// DELETE /api/v1/chat/sessions/{session_id}
#[delete("/chat/sessions/{session_id}")]
pub async fn delete_session(
    session_id: web::Path<String>,
    state: web::Data<Arc<Mutex<CodingAgentState>>>,
) -> Result<HttpResponse> {
    let state_lock = state.lock().await;
    
    // Delete from memory
    state_lock.history.delete_session(&session_id).await;
    
    // Delete from disk (if persistence enabled)
    // This happens automatically through the persistence layer
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "session_id": session_id.as_str(),
        "message": "Session deleted"
    })))
}

/// Get session statistics
/// 
/// GET /api/v1/chat/stats
#[get("/chat/stats")]
pub async fn get_chat_stats(
    state: web::Data<Arc<Mutex<CodingAgentState>>>,
) -> Result<HttpResponse> {
    let state_lock = state.lock().await;
    let stats = state_lock.history.get_stats().await;
    
    Ok(HttpResponse::Ok().json(stats))
}

/// Continue an existing session
/// 
/// POST /api/v1/chat/continue
/// 
/// Body: { "session_id": "xxx", "message": "..." }
#[post("/chat/continue")]
pub async fn continue_session(
    req: web::Json<ContinueSessionRequest>,
    state: web::Data<Arc<Mutex<CodingAgentState>>>,
) -> Result<HttpResponse> {
    // Check if session exists
    let session_exists = {
        let state_lock = state.lock().await;
        state_lock.history.get_session(&req.session_id).await.is_some()
    };
    
    if !session_exists {
        return Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Session not found",
            "session_id": &req.session_id,
            "suggestion": "Create a new session or check the session_id"
        })));
    }
    
    // Create request with existing session_id to continue conversation
    let request_body = serde_json::json!({
        "message": req.message,
        "user_id": req.user_id.clone().unwrap_or_else(|| "default".to_string()),
        "session_id": req.session_id,
    });
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Use POST /api/v1/chat/unified with this body to continue the session",
        "request_body": request_body,
        "note": "Session exists and will preserve history automatically"
    })))
}

/// Session summary for listing
#[derive(Debug, Serialize)]
struct SessionSummary {
    session_id: String,
    user_id: String,
    created_at: String,
    last_activity: String,
    message_count: usize,
    preview: String,
}

/// Full session history response
#[derive(Debug, Serialize)]
struct SessionHistoryResponse {
    session_id: String,
    user_id: String,
    created_at: String,
    last_activity: String,
    message_count: usize,
    messages: Vec<MessageResponse>,
}

/// Message in history response
#[derive(Debug, Serialize)]
struct MessageResponse {
    role: String,
    content: String,
    timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    confidence: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    language: Option<String>,
}

/// Continue session request
#[derive(Debug, Deserialize)]
pub struct ContinueSessionRequest {
    pub session_id: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
}
