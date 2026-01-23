use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Collaboration message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CollaborationMessage {
    /// User joins a session
    Join {
        session_id: String,
        user_id: String,
        username: String,
    },
    /// User leaves a session
    Leave {
        session_id: String,
        user_id: String,
    },
    /// Cursor position update
    CursorMove {
        session_id: String,
        user_id: String,
        position: CursorPosition,
    },
    /// Text edit operation
    TextEdit {
        session_id: String,
        user_id: String,
        operation: TextOperation,
    },
    /// Chat message in session
    ChatMessage {
        session_id: String,
        user_id: String,
        message: String,
        timestamp: i64,
    },
    /// Canvas update
    CanvasUpdate {
        session_id: String,
        user_id: String,
        content: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPosition {
    pub x: f64,
    pub y: f64,
    pub line: Option<u32>,
    pub column: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextOperation {
    pub operation_type: String, // "insert" or "delete"
    pub position: u32,
    pub text: Option<String>,
    pub length: Option<u32>,
}

/// Active user in a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveUser {
    pub user_id: String,
    pub username: String,
    pub color: String,
    pub cursor: Option<CursorPosition>,
    pub last_seen: i64,
}

/// Collaboration session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub session_id: String,
    pub active_users: HashMap<String, ActiveUser>,
    pub created_at: i64,
    pub last_activity: i64,
}

/// In-memory collaboration store
pub struct CollaborationStore {
    sessions: Arc<RwLock<HashMap<String, SessionState>>>,
    user_colors: Vec<String>,
}

impl CollaborationStore {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            user_colors: vec![
                "#ef4444".to_string(), // red
                "#f59e0b".to_string(), // orange
                "#eab308".to_string(), // yellow
                "#22c55e".to_string(), // green
                "#06b6d4".to_string(), // cyan
                "#3b82f6".to_string(), // blue
                "#8b5cf6".to_string(), // purple
                "#ec4899".to_string(), // pink
            ],
        }
    }
    
    /// Join a collaboration session
    pub async fn join_session(
        &self,
        session_id: String,
        user_id: String,
        username: String,
    ) -> SessionState {
        let mut sessions = self.sessions.write().await;
        
        let session = sessions.entry(session_id.clone()).or_insert_with(|| {
            SessionState {
                session_id: session_id.clone(),
                active_users: HashMap::new(),
                created_at: chrono::Utc::now().timestamp(),
                last_activity: chrono::Utc::now().timestamp(),
            }
        });
        
        // Assign color based on user count
        let color_index = session.active_users.len() % self.user_colors.len();
        let color = self.user_colors[color_index].clone();
        
        session.active_users.insert(
            user_id.clone(),
            ActiveUser {
                user_id: user_id.clone(),
                username,
                color,
                cursor: None,
                last_seen: chrono::Utc::now().timestamp(),
            },
        );
        
        session.last_activity = chrono::Utc::now().timestamp();
        session.clone()
    }
    
    /// Leave a collaboration session
    pub async fn leave_session(&self, session_id: &str, user_id: &str) -> Option<SessionState> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.get_mut(session_id) {
            session.active_users.remove(user_id);
            session.last_activity = chrono::Utc::now().timestamp();
            
            // Remove empty sessions
            if session.active_users.is_empty() {
                sessions.remove(session_id);
                return None;
            }
            
            return Some(session.clone());
        }
        
        None
    }
    
    /// Update cursor position
    pub async fn update_cursor(
        &self,
        session_id: &str,
        user_id: &str,
        position: CursorPosition,
    ) -> Option<SessionState> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.get_mut(session_id) {
            if let Some(user) = session.active_users.get_mut(user_id) {
                user.cursor = Some(position);
                user.last_seen = chrono::Utc::now().timestamp();
            }
            session.last_activity = chrono::Utc::now().timestamp();
            return Some(session.clone());
        }
        
        None
    }
    
    /// Get session state
    pub async fn get_session(&self, session_id: &str) -> Option<SessionState> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).cloned()
    }
    
    /// List all active sessions
    pub async fn list_sessions(&self) -> Vec<SessionState> {
        let sessions = self.sessions.read().await;
        sessions.values().cloned().collect()
    }
    
    /// Clean up inactive sessions (older than 1 hour)
    pub async fn cleanup_inactive(&self) {
        let mut sessions = self.sessions.write().await;
        let now = chrono::Utc::now().timestamp();
        let timeout = 3600; // 1 hour
        
        sessions.retain(|_, session| {
            now - session.last_activity < timeout
        });
    }
}

/// API: Join session
#[actix_web::post("/collaboration/join")]
pub async fn join_session(
    req: web::Json<serde_json::Value>,
    store: web::Data<CollaborationStore>,
) -> Result<HttpResponse> {
    let session_id = req["session_id"].as_str().unwrap_or("default").to_string();
    let user_id = req["user_id"]
        .as_str()
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    let username = req["username"].as_str().unwrap_or("Anonymous").to_string();
    
    let session = store.join_session(session_id, user_id.clone(), username).await;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "session": session,
        "user_id": user_id,
    })))
}

/// API: Leave session
#[actix_web::post("/collaboration/leave")]
pub async fn leave_session(
    req: web::Json<serde_json::Value>,
    store: web::Data<CollaborationStore>,
) -> Result<HttpResponse> {
    let session_id = req["session_id"].as_str().unwrap_or("default");
    let user_id = req["user_id"].as_str().unwrap_or("");
    
    let session = store.leave_session(session_id, user_id).await;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "session": session,
    })))
}

/// API: Update cursor
#[actix_web::post("/collaboration/cursor")]
pub async fn update_cursor(
    req: web::Json<serde_json::Value>,
    store: web::Data<CollaborationStore>,
) -> Result<HttpResponse> {
    let session_id = req["session_id"].as_str().unwrap_or("default");
    let user_id = req["user_id"].as_str().unwrap_or("");
    
    let position = CursorPosition {
        x: req["position"]["x"].as_f64().unwrap_or(0.0),
        y: req["position"]["y"].as_f64().unwrap_or(0.0),
        line: req["position"]["line"].as_u64().map(|v| v as u32),
        column: req["position"]["column"].as_u64().map(|v| v as u32),
    };
    
    let session = store.update_cursor(session_id, user_id, position).await;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "session": session,
    })))
}

/// API: Get session state
#[actix_web::get("/collaboration/session/{session_id}")]
pub async fn get_session(
    session_id: web::Path<String>,
    store: web::Data<CollaborationStore>,
) -> Result<HttpResponse> {
    let session = store.get_session(&session_id).await;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": session.is_some(),
        "session": session,
    })))
}

/// API: List sessions
#[actix_web::get("/collaboration/sessions")]
pub async fn list_sessions(
    store: web::Data<CollaborationStore>,
) -> Result<HttpResponse> {
    let sessions = store.list_sessions().await;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "sessions": sessions,
        "count": sessions.len(),
    })))
}
