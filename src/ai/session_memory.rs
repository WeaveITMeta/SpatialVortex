//! In-Memory Session Storage
//! Simple implementation for immediate use, can be upgraded to PostgreSQL later

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

/// Conversation session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationSession {
    pub id: Uuid,
    pub title: String,
    pub summary: Option<String>,
    pub user_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub message_count: usize,
    pub last_message_at: Option<DateTime<Utc>>,
    pub is_archived: bool,
    pub tags: Vec<String>,
}

/// Session message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMessage {
    pub id: Uuid,
    pub session_id: Uuid,
    pub role: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub token_count: Option<i32>,
    pub model: Option<String>,
}

/// In-memory session store
pub struct SessionStore {
    sessions: Mutex<HashMap<Uuid, ConversationSession>>,
    messages: Mutex<HashMap<Uuid, Vec<SessionMessage>>>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
            messages: Mutex::new(HashMap::new()),
        }
    }
    
    /// Create new session
    pub fn create_session(&self, title: Option<String>, user_id: Option<String>, tags: Option<Vec<String>>) -> ConversationSession {
        let session = ConversationSession {
            id: Uuid::new_v4(),
            title: title.unwrap_or_else(|| "New Conversation".to_string()),
            summary: None,
            user_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            message_count: 0,
            last_message_at: None,
            is_archived: false,
            tags: tags.unwrap_or_default(),
        };
        
        let mut sessions = self.sessions.lock().unwrap();
        let mut messages = self.messages.lock().unwrap();
        
        sessions.insert(session.id, session.clone());
        messages.insert(session.id, Vec::new());
        
        session
    }
    
    /// Get session
    pub fn get_session(&self, id: Uuid) -> Option<ConversationSession> {
        let sessions = self.sessions.lock().unwrap();
        sessions.get(&id).cloned()
    }
    
    /// List sessions
    pub fn list_sessions(&self, user_id: Option<String>, include_archived: bool) -> Vec<ConversationSession> {
        let sessions = self.sessions.lock().unwrap();
        let mut result: Vec<ConversationSession> = sessions
            .values()
            .filter(|s| {
                (user_id.is_none() || s.user_id == user_id) &&
                (include_archived || !s.is_archived)
            })
            .cloned()
            .collect();
        
        result.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        result
    }
    
    /// Add message
    pub fn add_message(&self, session_id: Uuid, role: String, content: String, token_count: Option<i32>, model: Option<String>) -> Option<SessionMessage> {
        let message = SessionMessage {
            id: Uuid::new_v4(),
            session_id,
            role,
            content: content.clone(),
            timestamp: Utc::now(),
            token_count,
            model,
        };
        
        let mut messages = self.messages.lock().unwrap();
        let mut sessions = self.sessions.lock().unwrap();
        
        // Add message
        if let Some(session_messages) = messages.get_mut(&session_id) {
            session_messages.push(message.clone());
            
            // Update session
            if let Some(session) = sessions.get_mut(&session_id) {
                session.message_count = session_messages.len();
                session.last_message_at = Some(message.timestamp);
                session.updated_at = Utc::now();
                
                // Auto-generate title from first user message
                if session.title == "New Conversation" && session.message_count == 1 && message.role == "user" {
                    session.title = content.chars().take(100).collect();
                }
            }
            
            Some(message)
        } else {
            None
        }
    }
    
    /// Get messages
    pub fn get_messages(&self, session_id: Uuid) -> Vec<SessionMessage> {
        let messages = self.messages.lock().unwrap();
        messages.get(&session_id).cloned().unwrap_or_default()
    }
    
    /// Update title
    pub fn update_title(&self, session_id: Uuid, title: String) -> bool {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get_mut(&session_id) {
            session.title = title;
            session.updated_at = Utc::now();
            true
        } else {
            false
        }
    }
    
    /// Update summary
    pub fn update_summary(&self, session_id: Uuid, summary: String) -> bool {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get_mut(&session_id) {
            session.summary = Some(summary);
            session.updated_at = Utc::now();
            true
        } else {
            false
        }
    }
    
    /// Archive session
    pub fn archive_session(&self, session_id: Uuid) -> bool {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get_mut(&session_id) {
            session.is_archived = true;
            session.updated_at = Utc::now();
            true
        } else {
            false
        }
    }
    
    /// Delete session
    pub fn delete_session(&self, session_id: Uuid) -> bool {
        let mut sessions = self.sessions.lock().unwrap();
        let mut messages = self.messages.lock().unwrap();
        
        sessions.remove(&session_id);
        messages.remove(&session_id);
        
        true
    }
    
    /// Search sessions
    pub fn search_sessions(&self, query: &str, user_id: Option<String>) -> Vec<ConversationSession> {
        let sessions = self.sessions.lock().unwrap();
        let query_lower = query.to_lowercase();
        
        let mut result: Vec<ConversationSession> = sessions
            .values()
            .filter(|s| {
                (user_id.is_none() || s.user_id == user_id) &&
                (s.title.to_lowercase().contains(&query_lower) ||
                 s.summary.as_ref().map_or(false, |sum| sum.to_lowercase().contains(&query_lower)))
            })
            .cloned()
            .collect();
        
        result.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        result
    }
    
    /// Get stats
    pub fn get_stats(&self, user_id: Option<String>) -> SessionStats {
        let sessions = self.sessions.lock().unwrap();
        let messages = self.messages.lock().unwrap();
        
        let filtered: Vec<&ConversationSession> = sessions
            .values()
            .filter(|s| user_id.is_none() || s.user_id == user_id)
            .collect();
        
        let total_sessions = filtered.len();
        let active_sessions = filtered.iter().filter(|s| !s.is_archived).count();
        let archived_sessions = filtered.iter().filter(|s| s.is_archived).count();
        let total_messages: usize = filtered
            .iter()
            .filter_map(|s| messages.get(&s.id))
            .map(|m| m.len())
            .sum();
        
        SessionStats {
            total_sessions,
            active_sessions,
            archived_sessions,
            total_messages,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SessionStats {
    pub total_sessions: usize,
    pub active_sessions: usize,
    pub archived_sessions: usize,
    pub total_messages: usize,
}
