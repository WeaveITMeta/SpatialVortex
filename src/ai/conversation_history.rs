//! Conversation History Management
//!
//! Provides comprehensive conversation context tracking for multi-turn interactions.
//! Maintains session state, message history, and contextual awareness across requests.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};

use super::chat_persistence::ChatPersistence;

/// Session timeout in hours
const SESSION_TIMEOUT_HOURS: i64 = 24;

/// Base context window in characters (soft limit, can extend with high confidence)
const BASE_CONTEXT_CHARS: usize = 4000;

/// High confidence threshold for unlimited context retention
const HIGH_CONFIDENCE_THRESHOLD: f32 = 0.7;

/// Medium confidence threshold for sacred position retention
const MEDIUM_CONFIDENCE_THRESHOLD: f32 = 0.6;

/// A single message in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    /// Message role (user, assistant, system)
    pub role: MessageRole,
    
    /// Message content
    pub content: String,
    
    /// Timestamp when message was created
    pub timestamp: DateTime<Utc>,
    
    /// Optional metadata (code blocks, confidence, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<MessageMetadata>,
}

/// Message role in conversation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

/// Optional metadata for messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageMetadata {
    /// Code blocks if this was a code generation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_blocks: Option<Vec<String>>,
    
    /// Confidence score (0.0-1.0) - AI's certainty about this response
    /// 
    /// This is the ONLY metric used for retention decisions.
    /// High confidence (≥0.7) = keep indefinitely
    /// Medium confidence (≥0.6) at sacred positions = keep
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f32>,
    
    /// Programming language
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

/// A conversation session with full history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationSession {
    /// Unique session identifier
    pub session_id: String,
    
    /// User identifier
    pub user_id: String,
    
    /// Message history
    pub messages: Vec<ConversationMessage>,
    
    /// When session was created
    pub created_at: DateTime<Utc>,
    
    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,
    
    /// Session-specific context/notes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_summary: Option<String>,
}

impl ConversationSession {
    /// Create new conversation session
    pub fn new(session_id: String, user_id: String) -> Self {
        let now = Utc::now();
        Self {
            session_id,
            user_id,
            messages: Vec::new(),
            created_at: now,
            last_activity: now,
            context_summary: None,
        }
    }
    
    /// Add a message to the conversation with confidence-based retention
    pub fn add_message(&mut self, role: MessageRole, content: String, metadata: Option<MessageMetadata>) {
        let message = ConversationMessage {
            role,
            content,
            timestamp: Utc::now(),
            metadata,
        };
        
        self.messages.push(message);
        self.last_activity = Utc::now();
        
        // Prune low-confidence messages at sacred checkpoints (positions 3, 6, 9)
        // This preserves high-confidence content indefinitely (like VortexContextPreserver)
        self.prune_low_confidence_messages();
    }
    
    /// Get recent context for LLM with dynamic confidence-based extension
    /// 
    /// Uses confidence as the ONLY metric with sacred geometry:
    /// - Base window (4000 chars soft limit)
    /// - High-confidence messages (≥0.7) extend context indefinitely
    /// - Sacred checkpoints (3, 6, 9) with medium confidence (≥0.6) preserved
    pub fn get_context_window(&self, base_chars: usize) -> Vec<ConversationMessage> {
        let mut context = Vec::new();
        let mut char_count = 0;
        
        // Start from most recent and work backwards
        for (idx, message) in self.messages.iter().rev().enumerate() {
            let msg_chars = message.content.len() + 50; // +50 for role/metadata overhead
            
            // Get confidence from metadata (default 0.5 for user messages)
            let confidence = message.metadata.as_ref()
                .and_then(|m| m.confidence)
                .unwrap_or(if message.role == MessageRole::User { 0.5 } else { 0.5 });
            
            // Check if at sacred checkpoint (reverse indexing: 3, 6, 9 from end)
            let at_sacred_position = (idx + 1) % 3 == 0;
            
            // Dynamic context extension logic based on confidence
            let should_include = if confidence >= HIGH_CONFIDENCE_THRESHOLD {
                // High confidence (≥0.7): always include (unlimited context)
                true
            } else if at_sacred_position && confidence >= MEDIUM_CONFIDENCE_THRESHOLD {
                // Sacred position with medium confidence (≥0.6): include
                true
            } else if char_count + msg_chars <= base_chars {
                // Within base window: include
                true
            } else {
                // Low confidence beyond base window: skip
                false
            };
            
            if should_include {
                context.push(message.clone());
                char_count += msg_chars;
            }
        }
        
        // Reverse to get chronological order
        context.reverse();
        context
    }
    
    /// Prune low-confidence messages at sacred checkpoints
    /// 
    /// Implements 3-6-9 sacred geometry pruning based on confidence scores
    fn prune_low_confidence_messages(&mut self) {
        if self.messages.len() < 10 {
            return; // Don't prune short conversations
        }
        
        // Keep messages that are:
        // 1. High confidence (≥0.7) - unlimited retention
        // 2. At sacred positions (3, 6, 9...) with medium confidence (≥0.6)
        // 3. Recent (last 20 messages always kept)
        
        let messages_to_keep: Vec<ConversationMessage> = self.messages.iter()
            .enumerate()
            .filter(|(idx, msg)| {
                let is_recent = *idx >= self.messages.len().saturating_sub(20);
                let confidence = msg.metadata.as_ref()
                    .and_then(|m| m.confidence)
                    .unwrap_or(0.5);
                
                let at_sacred_position = (*idx + 1) % 3 == 0;
                
                is_recent 
                    || confidence >= HIGH_CONFIDENCE_THRESHOLD
                    || (at_sacred_position && confidence >= MEDIUM_CONFIDENCE_THRESHOLD)
            })
            .map(|(_, msg)| msg.clone())
            .collect();
        
        self.messages = messages_to_keep;
    }
    
    /// Check if session is expired
    pub fn is_expired(&self) -> bool {
        let now = Utc::now();
        let timeout = Duration::hours(SESSION_TIMEOUT_HOURS);
        now.signed_duration_since(self.last_activity) > timeout
    }
    
    /// Get conversation summary for context
    pub fn get_summary(&self) -> String {
        if let Some(summary) = &self.context_summary {
            return summary.clone();
        }
        
        // Generate basic summary from message count
        format!("Conversation with {} messages", self.messages.len())
    }
    
    /// Update context summary (for long conversations)
    pub fn update_summary(&mut self, summary: String) {
        self.context_summary = Some(summary);
    }
}

/// Conversation history manager (thread-safe)
pub struct ConversationHistory {
    /// Active sessions by session_id
    sessions: Arc<RwLock<HashMap<String, ConversationSession>>>,
    /// Persistence layer for saving/loading sessions
    persistence: Option<Arc<ChatPersistence>>,
}

impl ConversationHistory {
    /// Create new conversation history manager
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            persistence: None,
        }
    }
    
    /// Create conversation history with persistence enabled
    pub fn with_persistence(storage_dir: &str) -> Self {
        let persistence = match ChatPersistence::new(storage_dir) {
            Ok(p) => Some(Arc::new(p)),
            Err(e) => {
                eprintln!("⚠️  Failed to initialize chat persistence: {}", e);
                eprintln!("⚠️  Chat history will not be saved to disk");
                None
            }
        };
        
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            persistence,
        }
    }
    
    /// Load all saved sessions from disk on startup
    pub async fn load_saved_sessions(&self) -> usize {
        if let Some(persistence) = &self.persistence {
            match persistence.load_all_sessions().await {
                Ok(saved_sessions) => {
                    let mut sessions = self.sessions.write().await;
                    let count = saved_sessions.len();
                    
                    for (session_id, session) in saved_sessions {
                        sessions.insert(session_id, session);
                    }
                    
                    if count > 0 {
                        println!("✅ Loaded {} chat session(s) from disk", count);
                    }
                    
                    count
                }
                Err(e) => {
                    eprintln!("⚠️  Failed to load saved chat sessions: {}", e);
                    0
                }
            }
        } else {
            0
        }
    }
    
    /// Save a session to disk (if persistence enabled)
    async fn save_session_to_disk(&self, session_id: &str, session: &ConversationSession) {
        if let Some(persistence) = &self.persistence {
            if let Err(e) = persistence.save_session(session_id, session).await {
                eprintln!("⚠️  Failed to save chat session {}: {}", session_id, e);
            }
        }
    }
    
    /// Get or create a session
    pub async fn get_or_create_session(&self, session_id: &str, user_id: &str) -> ConversationSession {
        let mut sessions = self.sessions.write().await;
        
        // Clean up expired sessions first
        sessions.retain(|_, session| !session.is_expired());
        
        // Get existing or create new
        sessions.entry(session_id.to_string())
            .or_insert_with(|| ConversationSession::new(session_id.to_string(), user_id.to_string()))
            .clone()
    }
    
    /// Add message to session
    pub async fn add_message(
        &self,
        session_id: &str,
        role: MessageRole,
        content: String,
        metadata: Option<MessageMetadata>,
    ) {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.get_mut(session_id) {
            session.add_message(role, content, metadata);
            
            // Save to disk after each message (if persistence enabled)
            let session_clone = session.clone();
            drop(sessions); // Release lock before async operation
            
            self.save_session_to_disk(session_id, &session_clone).await;
        }
    }
    
    /// Get conversation context for LLM prompt with dynamic confidence-based extension
    pub async fn get_context(&self, session_id: &str) -> Vec<ConversationMessage> {
        let sessions = self.sessions.read().await;
        
        sessions.get(session_id)
            .map(|session| session.get_context_window(BASE_CONTEXT_CHARS))
            .unwrap_or_default()
    }
    
    /// Get full session
    pub async fn get_session(&self, session_id: &str) -> Option<ConversationSession> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).cloned()
    }
    
    /// Delete a session
    pub async fn delete_session(&self, session_id: &str) {
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id);
    }
    
    /// Get all sessions for a user
    pub async fn get_user_sessions(&self, user_id: &str) -> Vec<ConversationSession> {
        let sessions = self.sessions.read().await;
        
        sessions.values()
            .filter(|s| s.user_id == user_id)
            .cloned()
            .collect()
    }
    
    /// Build context-aware prompt from history
    pub async fn build_contextual_prompt(&self, session_id: &str, current_message: &str) -> String {
        let context = self.get_context(session_id).await;
        
        if context.is_empty() {
            return current_message.to_string();
        }
        
        let mut prompt = String::from("Previous conversation:\n\n");
        
        for msg in context {
            let role_label = match msg.role {
                MessageRole::User => "User",
                MessageRole::Assistant => "Assistant",
                MessageRole::System => "System",
            };
            
            prompt.push_str(&format!("{}: {}\n\n", role_label, msg.content));
        }
        
        prompt.push_str(&format!("User: {}\n\nAssistant:", current_message));
        
        prompt
    }
    
    /// Get statistics
    pub async fn get_stats(&self) -> HistoryStats {
        let sessions = self.sessions.read().await;
        
        let total_sessions = sessions.len();
        let total_messages: usize = sessions.values().map(|s| s.messages.len()).sum();
        let active_sessions = sessions.values().filter(|s| !s.is_expired()).count();
        
        HistoryStats {
            total_sessions,
            active_sessions,
            total_messages,
        }
    }
}

impl Default for ConversationHistory {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for conversation history
#[derive(Debug, Clone, Serialize)]
pub struct HistoryStats {
    pub total_sessions: usize,
    pub active_sessions: usize,
    pub total_messages: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_session_creation() {
        let history = ConversationHistory::new();
        let session = history.get_or_create_session("test-123", "user-1").await;
        
        assert_eq!(session.session_id, "test-123");
        assert_eq!(session.user_id, "user-1");
        assert_eq!(session.messages.len(), 0);
    }
    
    #[tokio::test]
    async fn test_add_messages() {
        let history = ConversationHistory::new();
        
        history.add_message(
            "test-123",
            MessageRole::User,
            "Hello".to_string(),
            None,
        ).await;
        
        history.add_message(
            "test-123",
            MessageRole::Assistant,
            "Hi there!".to_string(),
            None,
        ).await;
        
        let session = history.get_session("test-123").await.unwrap();
        assert_eq!(session.messages.len(), 2);
    }
    
    #[tokio::test]
    async fn test_context_window() {
        let mut session = ConversationSession::new("test".to_string(), "user".to_string());
        
        // Add many messages
        for i in 0..10 {
            session.add_message(
                MessageRole::User,
                format!("Message {}", i),
                None,
            );
        }
        
        // Context window should be limited
        let context = session.get_context_window(200);
        assert!(context.len() < 10);
    }
    
    #[tokio::test]
    async fn test_contextual_prompt() {
        let history = ConversationHistory::new();
        
        history.add_message(
            "test-123",
            MessageRole::User,
            "Write a function".to_string(),
            None,
        ).await;
        
        history.add_message(
            "test-123",
            MessageRole::Assistant,
            "Here's the code...".to_string(),
            None,
        ).await;
        
        let prompt = history.build_contextual_prompt("test-123", "Make it better").await;
        
        assert!(prompt.contains("Previous conversation"));
        assert!(prompt.contains("Write a function"));
        assert!(prompt.contains("Make it better"));
    }
}
