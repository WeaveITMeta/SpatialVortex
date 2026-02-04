//! Session Memory Manager
//!
//! Manages conversation sessions with persistent storage

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[cfg(feature = "postgres")]
use sqlx::PgPool;
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
    pub message_count: i32,
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

/// Create session request
#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub title: Option<String>,
    pub user_id: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// Add message request
#[derive(Debug, Deserialize)]
pub struct AddMessageRequest {
    pub role: String,
    pub content: String,
    pub token_count: Option<i32>,
    pub model: Option<String>,
}

/// Search sessions request
#[derive(Debug, Deserialize)]
pub struct SearchSessionsRequest {
    pub query: String,
    pub user_id: Option<String>,
    pub tags: Option<Vec<String>>,
    pub limit: Option<i64>,
}

/// Session Manager
#[cfg(feature = "postgres")]
pub struct SessionManager {
    pool: PgPool,
}

#[cfg(not(feature = "postgres"))]
pub struct SessionManager;

#[cfg(feature = "postgres")]
impl SessionManager {
    /// Create new session manager
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    /// Create new session
    pub async fn create_session(&self, req: CreateSessionRequest) -> Result<ConversationSession> {
        let title = req.title.unwrap_or_else(|| "New Conversation".to_string());
        let tags: Vec<String> = req.tags.unwrap_or_default();
        
        let session = sqlx::query_as!(
            ConversationSession,
            r#"
            INSERT INTO conversation_sessions (title, user_id, tags)
            VALUES ($1, $2, $3)
            RETURNING 
                id, title, summary, user_id, created_at, updated_at, 
                message_count, last_message_at, is_archived, tags
            "#,
            title,
            req.user_id,
            &tags
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to create session")?;
        
        Ok(session)
    }
    
    /// Get session by ID
    pub async fn get_session(&self, session_id: Uuid) -> Result<ConversationSession> {
        let session = sqlx::query_as!(
            ConversationSession,
            r#"
            SELECT id, title, summary, user_id, created_at, updated_at,
                   message_count, last_message_at, is_archived, tags
            FROM conversation_sessions
            WHERE id = $1
            "#,
            session_id
        )
        .fetch_one(&self.pool)
        .await
        .context("Session not found")?;
        
        Ok(session)
    }
    
    /// List sessions for user
    pub async fn list_sessions(
        &self,
        user_id: Option<String>,
        limit: Option<i64>,
        include_archived: bool,
    ) -> Result<Vec<ConversationSession>> {
        let limit = limit.unwrap_or(50).min(100);
        
        let sessions = sqlx::query_as!(
            ConversationSession,
            r#"
            SELECT id, title, summary, user_id, created_at, updated_at,
                   message_count, last_message_at, is_archived, tags
            FROM conversation_sessions
            WHERE ($1::text IS NULL OR user_id = $1)
              AND (is_archived = false OR $2 = true)
            ORDER BY updated_at DESC
            LIMIT $3
            "#,
            user_id,
            include_archived,
            limit
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to list sessions")?;
        
        Ok(sessions)
    }
    
    /// Add message to session
    pub async fn add_message(
        &self,
        session_id: Uuid,
        req: AddMessageRequest,
    ) -> Result<SessionMessage> {
        let message = sqlx::query_as!(
            SessionMessage,
            r#"
            INSERT INTO session_messages (session_id, role, content, token_count, model)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, session_id, role, content, timestamp, token_count, model
            "#,
            session_id,
            req.role,
            req.content,
            req.token_count,
            req.model
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to add message")?;
        
        Ok(message)
    }
    
    /// Get messages for session
    pub async fn get_messages(
        &self,
        session_id: Uuid,
        limit: Option<i64>,
    ) -> Result<Vec<SessionMessage>> {
        let limit = limit.unwrap_or(100).min(500);
        
        let messages = sqlx::query_as!(
            SessionMessage,
            r#"
            SELECT id, session_id, role, content, timestamp, token_count, model
            FROM session_messages
            WHERE session_id = $1
            ORDER BY timestamp ASC
            LIMIT $2
            "#,
            session_id,
            limit
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to get messages")?;
        
        Ok(messages)
    }
    
    /// Update session title
    pub async fn update_title(&self, session_id: Uuid, title: String) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE conversation_sessions
            SET title = $1, updated_at = NOW()
            WHERE id = $2
            "#,
            title,
            session_id
        )
        .execute(&self.pool)
        .await
        .context("Failed to update title")?;
        
        Ok(())
    }
    
    /// Update session summary
    pub async fn update_summary(&self, session_id: Uuid, summary: String) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE conversation_sessions
            SET summary = $1, updated_at = NOW()
            WHERE id = $2
            "#,
            summary,
            session_id
        )
        .execute(&self.pool)
        .await
        .context("Failed to update summary")?;
        
        Ok(())
    }
    
    /// Archive session
    pub async fn archive_session(&self, session_id: Uuid) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE conversation_sessions
            SET is_archived = true, updated_at = NOW()
            WHERE id = $1
            "#,
            session_id
        )
        .execute(&self.pool)
        .await
        .context("Failed to archive session")?;
        
        Ok(())
    }
    
    /// Delete session
    pub async fn delete_session(&self, session_id: Uuid) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM conversation_sessions
            WHERE id = $1
            "#,
            session_id
        )
        .execute(&self.pool)
        .await
        .context("Failed to delete session")?;
        
        Ok(())
    }
    
    /// Search sessions by query
    pub async fn search_sessions(&self, req: SearchSessionsRequest) -> Result<Vec<ConversationSession>> {
        let limit = req.limit.unwrap_or(20).min(100);
        
        let sessions = if req.tags.is_some() {
            // Search with tags filter
            sqlx::query_as!(
                ConversationSession,
                r#"
                SELECT id, title, summary, user_id, created_at, updated_at,
                       message_count, last_message_at, is_archived, tags
                FROM conversation_sessions
                WHERE ($1::text IS NULL OR user_id = $1)
                  AND tags && $2
                  AND (
                    to_tsvector('english', title) @@ plainto_tsquery('english', $3)
                    OR to_tsvector('english', COALESCE(summary, '')) @@ plainto_tsquery('english', $3)
                  )
                ORDER BY updated_at DESC
                LIMIT $4
                "#,
                req.user_id,
                &req.tags.unwrap_or_default(),
                req.query,
                limit
            )
            .fetch_all(&self.pool)
            .await
            .context("Failed to search sessions with tags")?
        } else {
            // Search without tags filter
            sqlx::query_as!(
                ConversationSession,
                r#"
                SELECT id, title, summary, user_id, created_at, updated_at,
                       message_count, last_message_at, is_archived, tags
                FROM conversation_sessions
                WHERE ($1::text IS NULL OR user_id = $1)
                  AND (
                    to_tsvector('english', title) @@ plainto_tsquery('english', $2)
                    OR to_tsvector('english', COALESCE(summary, '')) @@ plainto_tsquery('english', $2)
                  )
                ORDER BY updated_at DESC
                LIMIT $3
                "#,
                req.user_id,
                req.query,
                limit
            )
            .fetch_all(&self.pool)
            .await
            .context("Failed to search sessions")?
        };
        
        Ok(sessions)
    }
    
    /// Get session statistics
    pub async fn get_stats(&self, user_id: Option<String>) -> Result<SessionStats> {
        let stats = sqlx::query_as!(
            SessionStats,
            r#"
            SELECT 
                COUNT(*) as "total_sessions!",
                COUNT(*) FILTER (WHERE is_archived = false) as "active_sessions!",
                COUNT(*) FILTER (WHERE is_archived = true) as "archived_sessions!",
                COALESCE(SUM(message_count), 0) as "total_messages!"
            FROM conversation_sessions
            WHERE $1::text IS NULL OR user_id = $1
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to get stats")?;
        
        Ok(stats)
    }
}

/// Session statistics
#[derive(Debug, Serialize)]
pub struct SessionStats {
    pub total_sessions: i64,
    pub active_sessions: i64,
    pub archived_sessions: i64,
    pub total_messages: i64,
}
