//! Task Persistence - Durable storage for task sessions
//!
//! Provides functionality to persist task sessions to disk, enabling:
//! - Recovery after server restarts
//! - Long-term task history
//! - Audit trail of completed tasks
//! - Session resumption

use super::{TaskSession, TaskProgress};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Serializable session snapshot for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSnapshot {
    pub session_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub session: TaskSession,
}

/// Task persistence manager
pub struct TaskPersistence {
    storage_dir: PathBuf,
}

impl TaskPersistence {
    /// Create new persistence manager
    pub fn new<P: AsRef<Path>>(storage_dir: P) -> Result<Self> {
        let storage_dir = storage_dir.as_ref().to_path_buf();
        
        // Create directory if it doesn't exist
        if !storage_dir.exists() {
            fs::create_dir_all(&storage_dir)
                .context("Failed to create task storage directory")?;
        }
        
        Ok(Self { storage_dir })
    }
    
    /// Save session to disk
    pub async fn save_session(&self, session_id: &str, session: &TaskSession) -> Result<()> {
        let snapshot = SessionSnapshot {
            session_id: session_id.to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            session: session.clone(),
        };
        
        let path = self.session_path(session_id);
        let json = serde_json::to_string_pretty(&snapshot)
            .context("Failed to serialize session")?;
        
        fs::write(&path, json)
            .context("Failed to write session file")?;
        
        Ok(())
    }
    
    /// Load session from disk
    pub async fn load_session(&self, session_id: &str) -> Result<TaskSession> {
        let path = self.session_path(session_id);
        
        let json = fs::read_to_string(&path)
            .context("Failed to read session file")?;
        
        let snapshot: SessionSnapshot = serde_json::from_str(&json)
            .context("Failed to deserialize session")?;
        
        Ok(snapshot.session)
    }
    
    /// Delete session from disk
    pub async fn delete_session(&self, session_id: &str) -> Result<()> {
        let path = self.session_path(session_id);
        
        if path.exists() {
            fs::remove_file(&path)
                .context("Failed to delete session file")?;
        }
        
        Ok(())
    }
    
    /// Load all sessions from disk
    pub async fn load_all_sessions(&self) -> Result<HashMap<String, TaskSession>> {
        let mut sessions = HashMap::new();
        
        if !self.storage_dir.exists() {
            return Ok(sessions);
        }
        
        let entries = fs::read_dir(&self.storage_dir)
            .context("Failed to read storage directory")?;
        
        for entry in entries {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();
            
            // Only process .json files
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            
            // Extract session ID from filename
            if let Some(session_id) = path.file_stem().and_then(|s| s.to_str()) {
                match self.load_session(session_id).await {
                    Ok(session) => {
                        sessions.insert(session_id.to_string(), session);
                    }
                    Err(e) => {
                        eprintln!("Failed to load session {}: {}", session_id, e);
                        // Continue loading other sessions
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
            .context("Failed to read storage directory")?;
        
        for entry in entries {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(session_id) = path.file_stem().and_then(|s| s.to_str()) {
                    session_ids.push(session_id.to_string());
                }
            }
        }
        
        Ok(session_ids)
    }
    
    /// Get session progress without loading full session
    pub async fn get_progress(&self, session_id: &str) -> Result<TaskProgress> {
        let session = self.load_session(session_id).await?;
        Ok(session.get_progress())
    }
    
    /// Archive old completed sessions
    pub async fn archive_completed(&self, older_than_days: u64) -> Result<usize> {
        let cutoff = Utc::now() - chrono::Duration::days(older_than_days as i64);
        let mut archived = 0;
        
        let sessions = self.load_all_sessions().await?;
        
        for (session_id, session) in sessions {
            let progress = session.get_progress();
            
            // Archive if all tasks are done and last update was before cutoff
            if progress.all_done {
                if let Some(last_task) = session.tasks.last() {
                    if let Some(completed_at) = last_task.completed_at {
                        if completed_at < cutoff {
                            // Move to archive directory
                            let archive_dir = self.storage_dir.join("archive");
                            fs::create_dir_all(&archive_dir)
                                .context("Failed to create archive directory")?;
                            
                            let from = self.session_path(&session_id);
                            let to = archive_dir.join(format!("{}.json", session_id));
                            
                            fs::rename(from, to)
                                .context("Failed to move session to archive")?;
                            
                            archived += 1;
                        }
                    }
                }
            }
        }
        
        Ok(archived)
    }
    
    /// Get path for session file
    fn session_path(&self, session_id: &str) -> PathBuf {
        self.storage_dir.join(format!("{}.json", session_id))
    }
    
    /// Get storage statistics
    pub async fn stats(&self) -> Result<StorageStats> {
        let sessions = self.load_all_sessions().await?;
        
        let mut total_tasks = 0;
        let mut completed_tasks = 0;
        let mut failed_tasks = 0;
        let mut pending_tasks = 0;
        let mut active_sessions = 0;
        let mut completed_sessions = 0;
        
        for session in sessions.values() {
            let progress = session.get_progress();
            
            total_tasks += progress.total;
            completed_tasks += progress.completed;
            failed_tasks += progress.failed;
            pending_tasks += progress.pending;
            
            if progress.all_done {
                completed_sessions += 1;
            } else {
                active_sessions += 1;
            }
        }
        
        Ok(StorageStats {
            total_sessions: sessions.len(),
            active_sessions,
            completed_sessions,
            total_tasks,
            completed_tasks,
            failed_tasks,
            pending_tasks,
        })
    }
}

/// Storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub total_sessions: usize,
    pub active_sessions: usize,
    pub completed_sessions: usize,
    pub total_tasks: usize,
    pub completed_tasks: usize,
    pub failed_tasks: usize,
    pub pending_tasks: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::{Task, TaskType, TaskStatus};
    use tempfile::tempdir;
    
    #[tokio::test]
    async fn test_save_and_load_session() {
        let dir = tempdir().unwrap();
        let persistence = TaskPersistence::new(dir.path()).unwrap();
        
        let mut session = TaskSession::new("test_session".to_string());
        session.add_task("Test task".to_string(), TaskType::General);
        
        // Save
        persistence.save_session("test_session", &session).await.unwrap();
        
        // Load
        let loaded = persistence.load_session("test_session").await.unwrap();
        assert_eq!(loaded.session_id, "test_session");
        assert_eq!(loaded.tasks.len(), 1);
    }
    
    #[tokio::test]
    async fn test_load_all_sessions() {
        let dir = tempdir().unwrap();
        let persistence = TaskPersistence::new(dir.path()).unwrap();
        
        // Create multiple sessions
        for i in 0..3 {
            let mut session = TaskSession::new(format!("session_{}", i));
            session.add_task("Task".to_string(), TaskType::General);
            persistence.save_session(&format!("session_{}", i), &session).await.unwrap();
        }
        
        // Load all
        let sessions = persistence.load_all_sessions().await.unwrap();
        assert_eq!(sessions.len(), 3);
    }
    
    #[tokio::test]
    async fn test_storage_stats() {
        let dir = tempdir().unwrap();
        let persistence = TaskPersistence::new(dir.path()).unwrap();
        
        let mut session = TaskSession::new("test".to_string());
        session.add_task("Task 1".to_string(), TaskType::Coding);
        session.add_task("Task 2".to_string(), TaskType::Analysis);
        
        persistence.save_session("test", &session).await.unwrap();
        
        let stats = persistence.stats().await.unwrap();
        assert_eq!(stats.total_sessions, 1);
        assert_eq!(stats.total_tasks, 2);
        assert_eq!(stats.pending_tasks, 2);
    }
}
