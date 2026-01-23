//! PostgreSQL Vector Store with pgvector Extension
//!
//! Persistent storage for RAG embeddings using PostgreSQL and pgvector.
//! Enables similarity search and persistent knowledge accumulation.

use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::sync::Arc;
use tokio::sync::RwLock;


/// PostgreSQL-backed vector store with pgvector extension
pub struct PostgresVectorStore {
    pool: PgPool,
    dimension: usize,
    cache: Arc<RwLock<Vec<StoredEmbedding>>>,
}

/// Stored embedding with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredEmbedding {
    pub id: i64,
    pub content: String,
    pub embedding: Vec<f32>,
    pub flux_position: i32,
    pub confidence: f32,
    pub sacred_score: f32,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl PostgresVectorStore {
    /// Create new PostgreSQL vector store
    ///
    /// # Arguments
    /// * `database_url` - PostgreSQL connection string (e.g., "postgresql://user:pass@localhost/dbname")
    /// * `dimension` - Embedding dimension (e.g., 384 for all-MiniLM-L6-v2)
    ///
    /// # Example
    /// ```no_run
    /// use spatial_vortex::rag::PostgresVectorStore;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let store = PostgresVectorStore::new(
    ///     "postgresql://localhost/spatial_vortex",
    ///     384
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(database_url: &str, dimension: usize) -> Result<Self> {
        // Create connection pool
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await
            .context("Failed to connect to PostgreSQL")?;
        
        // Initialize schema
        Self::init_schema(&pool, dimension).await?;
        
        Ok(Self {
            pool,
            dimension,
            cache: Arc::new(RwLock::new(Vec::new())),
        })
    }
    
    /// Initialize database schema with pgvector extension
    async fn init_schema(pool: &PgPool, dimension: usize) -> Result<()> {
        // Enable pgvector extension
        sqlx::query("CREATE EXTENSION IF NOT EXISTS vector")
            .execute(pool)
            .await
            .context("Failed to create pgvector extension")?;
        
        // Create embeddings table
        let create_table = format!(
            r#"
            CREATE TABLE IF NOT EXISTS rag_embeddings (
                id BIGSERIAL PRIMARY KEY,
                content TEXT NOT NULL,
                embedding vector({}) NOT NULL,
                flux_position INTEGER NOT NULL,
                confidence REAL NOT NULL,
                sacred_score REAL NOT NULL,
                metadata JSONB,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            )
            "#,
            dimension
        );
        
        sqlx::query(&create_table)
            .execute(pool)
            .await
            .context("Failed to create embeddings table")?;
        
        // Create index for vector similarity search
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS rag_embeddings_vector_idx 
            ON rag_embeddings 
            USING ivfflat (embedding vector_cosine_ops)
            WITH (lists = 100)
            "#
        )
        .execute(pool)
        .await
        .context("Failed to create vector index")?;
        
        // Create indices for filtering
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS rag_embeddings_flux_idx 
            ON rag_embeddings (flux_position)
            "#
        )
        .execute(pool)
        .await
        .ok(); // May already exist
        
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS rag_embeddings_signal_idx 
            ON rag_embeddings (confidence)
            "#
        )
        .execute(pool)
        .await
        .ok(); // May already exist
        
        println!("✅ PostgreSQL vector store initialized (dimension: {})", dimension);
        
        Ok(())
    }
    
    /// Store embedding with metadata
    pub async fn store(
        &self,
        content: &str,
        embedding: Vec<f32>,
        flux_position: i32,
        confidence: f32,
        sacred_score: f32,
        metadata: serde_json::Value,
    ) -> Result<i64> {
        if embedding.len() != self.dimension {
            anyhow::bail!("Embedding dimension mismatch: expected {}, got {}", 
                self.dimension, embedding.len());
        }
        
        // Convert Vec<f32> to pgvector format
        let embedding_str = format!("[{}]", embedding.iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(","));
        
        let result = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO rag_embeddings 
                (content, embedding, flux_position, confidence, sacred_score, metadata)
            VALUES ($1, $2::vector, $3, $4, $5, $6)
            RETURNING id
            "#
        )
        .bind(content)
        .bind(embedding_str)
        .bind(flux_position)
        .bind(confidence)
        .bind(sacred_score)
        .bind(metadata)
        .fetch_one(&self.pool)
        .await
        .context("Failed to store embedding")?;
        
        Ok(result)
    }
    
    /// Search for similar embeddings using cosine similarity
    pub async fn search(
        &self,
        query_embedding: &[f32],
        top_k: usize,
        min_confidence: Option<f32>,
    ) -> Result<Vec<StoredEmbedding>> {
        if query_embedding.len() != self.dimension {
            anyhow::bail!("Query embedding dimension mismatch");
        }
        
        // Convert query to pgvector format
        let query_str = format!("[{}]", query_embedding.iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(","));
        
        let signal_filter = min_confidence.unwrap_or(0.0);
        
        let results = sqlx::query_as::<_, (i64, String, Vec<u8>, i32, f32, f32, serde_json::Value, chrono::DateTime<chrono::Utc>)>(
            r#"
            SELECT id, content, embedding::text::bytea, flux_position, 
                   confidence, sacred_score, metadata, created_at
            FROM rag_embeddings
            WHERE confidence >= $3
            ORDER BY embedding <=> $1::vector
            LIMIT $2
            "#
        )
        .bind(query_str)
        .bind(top_k as i64)
        .bind(signal_filter)
        .fetch_all(&self.pool)
        .await
        .context("Failed to search embeddings")?;
        
        // Parse results
        let mut stored_embeddings = Vec::new();
        for (id, content, embedding_bytes, flux_pos, signal, sacred, metadata, created) in results {
            // Parse embedding from PostgreSQL vector format
            let embedding_str = String::from_utf8_lossy(&embedding_bytes);
            let embedding = self.parse_vector(&embedding_str)?;
            
            stored_embeddings.push(StoredEmbedding {
                id,
                content,
                embedding,
                flux_position: flux_pos,
                confidence: signal,
                sacred_score: sacred,
                metadata,
                created_at: created,
            });
        }
        
        Ok(stored_embeddings)
    }
    
    /// Search with sacred position filtering
    pub async fn search_sacred(
        &self,
        query_embedding: &[f32],
        sacred_positions: &[i32],
        top_k: usize,
    ) -> Result<Vec<StoredEmbedding>> {
        if query_embedding.len() != self.dimension {
            anyhow::bail!("Query embedding dimension mismatch");
        }
        
        let query_str = format!("[{}]", query_embedding.iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(","));
        
        // Convert positions to PostgreSQL array
        let positions_array: Vec<i32> = sacred_positions.to_vec();
        
        let results = sqlx::query_as::<_, (i64, String, Vec<u8>, i32, f32, f32, serde_json::Value, chrono::DateTime<chrono::Utc>)>(
            r#"
            SELECT id, content, embedding::text::bytea, flux_position, 
                   confidence, sacred_score, metadata, created_at
            FROM rag_embeddings
            WHERE flux_position = ANY($3)
            ORDER BY embedding <=> $1::vector
            LIMIT $2
            "#
        )
        .bind(query_str)
        .bind(top_k as i64)
        .bind(&positions_array)
        .fetch_all(&self.pool)
        .await
        .context("Failed to search sacred embeddings")?;
        
        let mut stored_embeddings = Vec::new();
        for (id, content, embedding_bytes, flux_pos, signal, sacred, metadata, created) in results {
            let embedding_str = String::from_utf8_lossy(&embedding_bytes);
            let embedding = self.parse_vector(&embedding_str)?;
            
            stored_embeddings.push(StoredEmbedding {
                id,
                content,
                embedding,
                flux_position: flux_pos,
                confidence: signal,
                sacred_score: sacred,
                metadata,
                created_at: created,
            });
        }
        
        Ok(stored_embeddings)
    }
    
    /// Get statistics about stored embeddings
    pub async fn stats(&self) -> Result<VectorStoreStats> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM rag_embeddings")
            .fetch_one(&self.pool)
            .await?;
        
        let avg_signal: Option<f64> = sqlx::query_scalar(
            "SELECT AVG(confidence) FROM rag_embeddings"
        )
        .fetch_one(&self.pool)
        .await?;
        
        let sacred_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM rag_embeddings WHERE flux_position IN (3, 6, 9)"
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(VectorStoreStats {
            total_embeddings: count as usize,
            avg_confidence: avg_signal.unwrap_or(0.0) as f32,
            sacred_embeddings: sacred_count as usize,
            dimension: self.dimension,
        })
    }
    
    /// Parse PostgreSQL vector format to Vec<f32>
    fn parse_vector(&self, vector_str: &str) -> Result<Vec<f32>> {
        let trimmed = vector_str.trim_matches(|c| c == '[' || c == ']');
        let values: Result<Vec<f32>> = trimmed
            .split(',')
            .map(|s| s.trim().parse::<f32>().context("Failed to parse vector component"))
            .collect();
        
        values
    }
    
    /// Delete old embeddings (cleanup)
    pub async fn cleanup_old(&self, days: i64) -> Result<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM rag_embeddings
            WHERE created_at < NOW() - $1::interval
            "#
        )
        .bind(format!("{} days", days))
        .execute(&self.pool)
        .await?;
        
        Ok(result.rows_affected())
    }
    
    /// Get embedding dimension
    pub fn dimension(&self) -> usize {
        self.dimension
    }
}

/// Statistics about the vector store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreStats {
    pub total_embeddings: usize,
    pub avg_confidence: f32,
    pub sacred_embeddings: usize,
    pub dimension: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    #[ignore] // Requires PostgreSQL with pgvector
    async fn test_postgres_vector_store() {
        let store = PostgresVectorStore::new(
            "postgresql://localhost/spatial_vortex_test",
            384
        ).await.unwrap();
        
        // Store test embedding
        let embedding = vec![0.1; 384];
        let id = store.store(
            "test content",
            embedding.clone(),
            3, // sacred position
            0.8,
            0.9,
            serde_json::json!({"test": true}),
        ).await.unwrap();
        
        assert!(id > 0);
        
        // Search
        let results = store.search(&embedding, 5, Some(0.5)).await.unwrap();
        assert!(!results.is_empty());
        
        // Stats
        let stats = store.stats().await.unwrap();
        assert!(stats.total_embeddings > 0);
    }
}
