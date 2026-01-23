//! Chat History Persistence
//!
//! Stores and retrieves conversation history to/from disk for session recovery.

use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use anyhow::{Context, Result};

use super::conversation_history::ConversationSession;

/// Chat history persistence manager
pub struct ChatPersistence {
    storage_dir: PathBuf,
}

/// Serializable session snapshot for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSnapshot {
    pub session_id: String,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub session: ConversationSession,
}

impl ChatPersistence {
    /// Create new persistence manager with storage directory
    pub fn new<P: AsRef<Path>>(storage_dir: P) -> Result<Self> {
        let dir = storage_dir.as_ref().to_path_buf();
        
        // Create directory if it doesn't exist
        if !dir.exists() {
            fs::create_dir_all(&dir)
                .context("Failed to create chat storage directory")?;
        }
        
        Ok(Self {
            storage_dir: dir,
        })
    }
    
    /// Save session to disk
    pub async fn save_session(&self, session_id: &str, session: &ConversationSession) -> Result<()> {
        let snapshot = ChatSnapshot {
            session_id: session_id.to_string(),
            user_id: session.user_id.clone(),
            created_at: session.created_at,
            updated_at: Utc::now(),
            session: session.clone(),
        };
        
        let path = self.session_path(session_id);
        let json = serde_json::to_string_pretty(&snapshot)
            .context("Failed to serialize chat session")?;
        
        fs::write(&path, json)
            .context("Failed to write chat session file")?;
        
        Ok(())
    }
    
    /// Load session from disk
    pub async fn load_session(&self, session_id: &str) -> Result<ConversationSession> {
        let path = self.session_path(session_id);
        
        let json = fs::read_to_string(&path)
            .context("Failed to read chat session file")?;
        
        let snapshot: ChatSnapshot = serde_json::from_str(&json)
            .context("Failed to deserialize chat session")?;
        
        Ok(snapshot.session)
    }
    
    /// Delete session from disk
    pub async fn delete_session(&self, session_id: &str) -> Result<()> {
        let path = self.session_path(session_id);
        
        if path.exists() {
            fs::remove_file(&path)
                .context("Failed to delete chat session file")?;
        }
        
        Ok(())
    }
    
    /// Check if session exists on disk
    pub async fn session_exists(&self, session_id: &str) -> bool {
        self.session_path(session_id).exists()
    }
    
    /// Load all sessions from disk
    pub async fn load_all_sessions(&self) -> Result<HashMap<String, ConversationSession>> {
        let mut sessions = HashMap::new();
        
        if !self.storage_dir.exists() {
            return Ok(sessions);
        }
        
        let entries = fs::read_dir(&self.storage_dir)
            .context("Failed to read chat storage directory")?;
        
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                // Extract session ID from filename
                if let Some(session_id) = path.file_stem().and_then(|s| s.to_str()) {
                    match self.load_session(session_id).await {
                        Ok(session) => {
                            sessions.insert(session_id.to_string(), session);
                        }
                        Err(e) => {
                            eprintln!("Failed to load chat session {}: {}", session_id, e);
                            // Continue loading other sessions
                        }
                    }
                }
            }
        }
        
        Ok(sessions)
    }
    
    /// List all session IDs
    pub async fn list_sessions(&self) -> Result<Vec<String>> {
        let mut session_ids = Vec::new();
        
        if !self.storage_dir.exists() {
            return Ok(session_ids);
        }
        
        let entries = fs::read_dir(&self.storage_dir)
            .context("Failed to read chat storage directory")?;
        
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(session_id) = path.file_stem().and_then(|s| s.to_str()) {
                    session_ids.push(session_id.to_string());
                }
            }
        }
        
        Ok(session_ids)
    }
    
    /// List sessions for a specific user
    pub async fn list_user_sessions(&self, user_id: &str) -> Result<Vec<String>> {
        let mut user_sessions = Vec::new();
        
        let all_sessions = self.load_all_sessions().await?;
        
        for (session_id, session) in all_sessions {
            if session.user_id == user_id {
                user_sessions.push(session_id);
            }
        }
        
        Ok(user_sessions)
    }
    
    /// Archive old sessions (move to archive directory)
    pub async fn archive_old_sessions(&self, days_old: i64) -> Result<usize> {
        let cutoff = Utc::now() - chrono::Duration::days(days_old);
        let mut archived_count = 0;
        
        let sessions = self.load_all_sessions().await?;
        
        for (session_id, session) in sessions {
            if session.last_activity < cutoff {
                // Create archive directory
                let archive_dir = self.storage_dir.join("archive");
                if !archive_dir.exists() {
                    fs::create_dir_all(&archive_dir)
                        .context("Failed to create archive directory")?;
                }
                
                // Move session to archive
                let from = self.session_path(&session_id);
                let to = archive_dir.join(format!("{}.json", session_id));
                
                fs::rename(from, to)
                    .context("Failed to move session to archive")?;
                
                archived_count += 1;
            }
        }
        
        Ok(archived_count)
    }
    
    /// Get storage statistics
    pub async fn get_stats(&self) -> Result<StorageStats> {
        let sessions = self.load_all_sessions().await?;
        
        let mut total_messages = 0;
        let mut total_size = 0;
        
        for session in sessions.values() {
            total_messages += session.messages.len();
        }
        
        // Calculate directory size
        if self.storage_dir.exists() {
            if let Ok(entries) = fs::read_dir(&self.storage_dir) {
                for entry in entries.flatten() {
                    if let Ok(metadata) = entry.metadata() {
                        total_size += metadata.len();
                    }
                }
            }
        }
        
        Ok(StorageStats {
            total_sessions: sessions.len(),
            total_messages,
            storage_size_bytes: total_size,
            storage_dir: self.storage_dir.display().to_string(),
        })
    }
    
    /// Get path for session file
    fn session_path(&self, session_id: &str) -> PathBuf {
        self.storage_dir.join(format!("{}.json", session_id))
    }
}

/// Storage statistics
#[derive(Debug, Serialize)]
pub struct StorageStats {
    pub total_sessions: usize,
    pub total_messages: usize,
    pub storage_size_bytes: u64,
    pub storage_dir: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::conversation_history::{MessageRole, MessageMetadata};
    
    #[tokio::test]
    async fn test_save_and_load() {
        let temp_dir = std::env::temp_dir().join("test_chat_persistence");
        let _ = fs::remove_dir_all(&temp_dir); // Clean up
        
        let persistence = ChatPersistence::new(&temp_dir).unwrap();
        
        // Create test session
        let mut session = ConversationSession::new(
            "test_session".to_string(),
            "test_user".to_string()
        );
        session.add_message(
            MessageRole::User,
            "Hello".to_string(),
            None
        );
        session.add_message(
            MessageRole::Assistant,
            "Hi there!".to_string(),
            Some(MessageMetadata {
                code_blocks: None,
                confidence: Some(0.85),
                language: None,
            })
        );
        
        // Save
        persistence.save_session("test_session", &session).await.unwrap();
        
        // Load
        let loaded = persistence.load_session("test_session").await.unwrap();
        assert_eq!(loaded.session_id, "test_session");
        assert_eq!(loaded.user_id, "test_user");
        assert_eq!(loaded.messages.len(), 2);
        
        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }
    
    #[tokio::test]
    async fn test_list_sessions() {
        let temp_dir = std::env::temp_dir().join("test_chat_list");
        let _ = fs::remove_dir_all(&temp_dir);
        
        let persistence = ChatPersistence::new(&temp_dir).unwrap();
        
        // Create multiple sessions
        for i in 0..3 {
            let session = ConversationSession::new(
                format!("session_{}", i),
                "test_user".to_string()
            );
            persistence.save_session(&format!("session_{}", i), &session).await.unwrap();
        }
        
        // List
        let sessions = persistence.list_sessions().await.unwrap();
        assert_eq!(sessions.len(), 3);
        
        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
