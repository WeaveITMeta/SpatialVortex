//! Spatial Database - PostgreSQL persistence layer
//! 
//! Provides async CRUD operations for FluxMatrix storage and retrieval
//! with connection pooling and JSON serialization.

use crate::error::{Result, SpatialVortexError};
use crate::models::*;
use deadpool_postgres::{Config, Pool, Runtime};
use serde_json;
use tokio_postgres::NoTls;
use uuid::Uuid;

/// Spatial Database with connection pooling
#[derive(Clone)]
pub struct SpatialDatabase {
    pool: Pool,
}

impl SpatialDatabase {
    /// Create new database connection pool from DATABASE_URL environment variable
    /// 
    /// Loads connection string from `DATABASE_URL` in .env file.
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// use spatial_vortex::storage::SpatialDatabase;
    /// 
    /// # async fn example() -> anyhow::Result<()> {
    /// // Reads from DATABASE_URL in .env file
    /// let db = SpatialDatabase::from_env().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn from_env() -> Result<Self> {
        // Load .env file if present
        let _ = dotenv::dotenv();
        
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| SpatialVortexError::Database(
                "DATABASE_URL environment variable not set".to_string()
            ))?;
        
        Self::new(&database_url).await
    }

    /// Create new database connection pool with explicit connection string
    /// 
    /// # Arguments
    /// 
    /// * `database_url` - PostgreSQL connection string
    ///   (e.g., "postgresql://user:pass@localhost:5432/spatial_vortex")
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// use spatial_vortex::storage::SpatialDatabase;
    /// 
    /// # async fn example() -> anyhow::Result<()> {
    /// let db = SpatialDatabase::new("postgresql://localhost/spatial_vortex").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(database_url: &str) -> Result<Self> {
        // Use the provided database URL directly
        let mut cfg = Config::new();
        cfg.url = Some(database_url.to_string());
        
        let pool = cfg
            .create_pool(Some(Runtime::Tokio1), NoTls)
            .map_err(|e| SpatialVortexError::Database(format!("Failed to create pool: {}", e)))?;
        
        Ok(Self { pool })
    }

    /// Initialize database schema (tables, indexes)
    pub async fn initialize_schema(&self) -> Result<()> {
        let client = self.pool.get().await
            .map_err(|e| SpatialVortexError::Database(e.to_string()))?;
        
        // Create flux_matrices table
        client.execute(
            r#"
            CREATE TABLE IF NOT EXISTS flux_matrices (
                id UUID PRIMARY KEY,
                subject TEXT NOT NULL,
                data JSONB NOT NULL,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            )
            "#,
            &[],
        ).await.map_err(|e| SpatialVortexError::Database(e.to_string()))?;
        
        // Create index on subject for fast lookups
        client.execute(
            r#"
            CREATE INDEX IF NOT EXISTS idx_flux_matrices_subject 
            ON flux_matrices(subject)
            "#,
            &[],
        ).await.map_err(|e| SpatialVortexError::Database(e.to_string()))?;
        
        // Create inference_log table
        client.execute(
            r#"
            CREATE TABLE IF NOT EXISTS inference_log (
                id SERIAL PRIMARY KEY,
                matrix_id UUID REFERENCES flux_matrices(id),
                input_seeds JSONB NOT NULL,
                output_meanings JSONB NOT NULL,
                confidence REAL NOT NULL,
                processing_time_ms INTEGER NOT NULL,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            )
            "#,
            &[],
        ).await.map_err(|e| SpatialVortexError::Database(e.to_string()))?;
        
        Ok(())
    }

    /// Store or update a FluxMatrix
    pub async fn store_matrix(&self, matrix: &FluxMatrix) -> Result<()> {
        let client = self.pool.get().await
            .map_err(|e| SpatialVortexError::Database(e.to_string()))?;
        
        let matrix_json = serde_json::to_value(matrix)?;
        
        // Upsert: insert or update if exists
        client.execute(
            r#"
            INSERT INTO flux_matrices (id, subject, data, created_at, updated_at)
            VALUES ($1, $2, $3, NOW(), NOW())
            ON CONFLICT (id) 
            DO UPDATE SET 
                subject = EXCLUDED.subject,
                data = EXCLUDED.data,
                updated_at = NOW()
            "#,
            &[&matrix.id, &matrix.subject, &matrix_json],
        ).await.map_err(|e| SpatialVortexError::Database(e.to_string()))?;
        
        Ok(())
    }

    /// Retrieve FluxMatrix by subject name
    pub async fn get_matrix_by_subject(&self, subject: &str) -> Result<Option<FluxMatrix>> {
        let client = self.pool.get().await
            .map_err(|e| SpatialVortexError::Database(e.to_string()))?;
        
        let row = client.query_opt(
            "SELECT data FROM flux_matrices WHERE subject = $1 ORDER BY updated_at DESC LIMIT 1",
            &[&subject],
        ).await.map_err(|e| SpatialVortexError::Database(e.to_string()))?;
        
        match row {
            Some(row) => {
                let json: serde_json::Value = row.get(0);
                let matrix: FluxMatrix = serde_json::from_value(json)?;
                Ok(Some(matrix))
            },
            None => Ok(None),
        }
    }

    /// Retrieve FluxMatrix by ID
    pub async fn get_matrix_by_id(&self, matrix_id: Uuid) -> Result<Option<FluxMatrix>> {
        let client = self.pool.get().await
            .map_err(|e| SpatialVortexError::Database(e.to_string()))?;
        
        let row = client.query_opt(
            "SELECT data FROM flux_matrices WHERE id = $1",
            &[&matrix_id],
        ).await.map_err(|e| SpatialVortexError::Database(e.to_string()))?;
        
        match row {
            Some(row) => {
                let json: serde_json::Value = row.get(0);
                let matrix: FluxMatrix = serde_json::from_value(json)?;
                Ok(Some(matrix))
            },
            None => Ok(None),
        }
    }

    /// Delete FluxMatrix by ID
    pub async fn delete_matrix(&self, matrix_id: Uuid) -> Result<bool> {
        let client = self.pool.get().await
            .map_err(|e| SpatialVortexError::Database(e.to_string()))?;
        
        let rows_affected = client.execute(
            "DELETE FROM flux_matrices WHERE id = $1",
            &[&matrix_id],
        ).await.map_err(|e| SpatialVortexError::Database(e.to_string()))?;
        
        Ok(rows_affected > 0)
    }

    /// Get all stored subject names
    pub async fn get_all_subjects(&self) -> Result<Vec<String>> {
        let client = self.pool.get().await
            .map_err(|e| SpatialVortexError::Database(e.to_string()))?;
        
        let rows = client.query(
            "SELECT DISTINCT subject FROM flux_matrices ORDER BY subject",
            &[],
        ).await.map_err(|e| SpatialVortexError::Database(e.to_string()))?;
        
        Ok(rows.iter().map(|row| row.get(0)).collect())
    }

    /// Log an inference operation
    pub async fn log_inference(
        &self,
        matrix_id: Uuid,
        input_seeds: &[i16],
        output_meanings: &[String],
        confidence: f32,
        processing_time_ms: i32,
    ) -> Result<()> {
        let client = self.pool.get().await
            .map_err(|e| SpatialVortexError::Database(e.to_string()))?;
        
        let seeds_json = serde_json::to_value(input_seeds)?;
        let meanings_json = serde_json::to_value(output_meanings)?;
        
        client.execute(
            r#"
            INSERT INTO inference_log (matrix_id, input_seeds, output_meanings, confidence, processing_time_ms)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            &[&matrix_id, &seeds_json, &meanings_json, &confidence, &processing_time_ms],
        ).await.map_err(|e| SpatialVortexError::Database(e.to_string()))?;
        
        Ok(())
    }

    /// Get database statistics
    pub async fn get_statistics(&self) -> Result<DatabaseStatistics> {
        let client = self.pool.get().await
            .map_err(|e| SpatialVortexError::Database(e.to_string()))?;
        
        let row = client.query_one(
            r#"
            SELECT 
                (SELECT COUNT(*) FROM flux_matrices) as total_matrices,
                (SELECT COUNT(*) FROM inference_log) as total_inferences,
                (SELECT SUM(jsonb_array_length(data->'nodes')) FROM flux_matrices) as total_nodes
            "#,
            &[],
        ).await.map_err(|e| SpatialVortexError::Database(e.to_string()))?;
        
        Ok(DatabaseStatistics {
            total_matrices: row.get::<_, i64>(0) as usize,
            total_inferences: row.get::<_, i64>(1) as usize,
            total_associations: row.get::<_, Option<i64>>(2).unwrap_or(0) as usize,
        })
    }

    /// Check database connection health
    pub async fn health_check(&self) -> Result<()> {
        let client = self.pool.get().await
            .map_err(|e| SpatialVortexError::Database(format!("Connection failed: {}", e)))?;
        
        client.execute("SELECT 1", &[]).await
            .map_err(|e| SpatialVortexError::Database(format!("Query failed: {}", e)))?;
        
        Ok(())
    }
    /// Get database connection pool
    pub fn get_pool(&self) -> &Pool {
        &self.pool
    }
}

/// Database statistics
#[derive(Debug, Clone)]
pub struct DatabaseStatistics {
    pub total_matrices: usize,
    pub total_associations: usize,
    pub total_inferences: usize,
}

// ============================================================
// ASI-Specific Types
// ============================================================

/// ONNX Model metadata
#[derive(Debug, Clone)]
pub struct OnnxModel {
    pub id: Uuid,
    pub name: String,
    pub model_path: String,
    pub tokenizer_path: String,
    pub embedding_dim: i32,
    pub status: String,
}

/// ASI Inference record
#[derive(Debug, Clone)]
pub struct ASIInference {
    pub id: Uuid,
    pub input_text: String,
    pub beam_tensor_id: Option<Uuid>,
    pub onnx_model_id: Option<Uuid>,
    pub flux_position: i32,
    pub archetype: Option<String>,
    pub confidence: f32,
    pub hallucination_detected: bool,
    pub vortex_intervention: bool,
    pub lake_worthy: bool,
    pub processing_time_ms: i32,
}

/// Sacred intervention record
#[derive(Debug, Clone)]
pub struct SacredIntervention {
    pub inference_id: Uuid,
    pub position: i32,
    pub signal_before: f32,
    pub signal_after: f32,
    pub confidence_boost: f32,
    pub intervention_type: String,
}

// ============================================================
// ASI-Specific Methods
// ============================================================

impl SpatialDatabase {
    /// Store ONNX model metadata
    pub async fn store_onnx_model(&self, model: &OnnxModel) -> Result<()> {
        let client = self.pool.get().await
            .map_err(|e| SpatialVortexError::Database(e.to_string()))?;
        
        client.execute(
            r#"
            INSERT INTO onnx_models (id, name, model_path, tokenizer_path, embedding_dim, status, loaded_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW())
            ON CONFLICT (name) 
            DO UPDATE SET 
                model_path = EXCLUDED.model_path,
                tokenizer_path = EXCLUDED.tokenizer_path,
                embedding_dim = EXCLUDED.embedding_dim,
                status = EXCLUDED.status,
                loaded_at = NOW(),
                updated_at = NOW()
            "#,
            &[&model.id, &model.name, &model.model_path, &model.tokenizer_path, &model.embedding_dim, &model.status],
        ).await.map_err(|e| SpatialVortexError::Database(e.to_string()))?;
        
        Ok(())
    }
    
    /// Get ONNX model by name
    pub async fn get_onnx_model(&self, name: &str) -> Result<Option<OnnxModel>> {
        let client = self.pool.get().await
            .map_err(|e| SpatialVortexError::Database(e.to_string()))?;
        
        let row = client.query_opt(
            "SELECT id, name, model_path, tokenizer_path, embedding_dim, status FROM onnx_models WHERE name = $1",
            &[&name],
        ).await.map_err(|e| SpatialVortexError::Database(e.to_string()))?;
        
        match row {
            Some(row) => Ok(Some(OnnxModel {
                id: row.get(0),
                name: row.get(1),
                model_path: row.get(2),
                tokenizer_path: row.get(3),
                embedding_dim: row.get(4),
                status: row.get(5),
            })),
            None => Ok(None),
        }
    }
    
    /// Log ASI inference
    pub async fn log_asi_inference(&self, inference: &ASIInference) -> Result<Uuid> {
        let client = self.pool.get().await
            .map_err(|e| SpatialVortexError::Database(e.to_string()))?;
        
        let row = client.query_one(
            r#"
            INSERT INTO asi_inferences (
                id, input_text, beam_tensor_id, onnx_model_id, flux_position,
                archetype, confidence, hallucination_detected,
                vortex_intervention, lake_worthy, processing_time_ms
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id
            "#,
            &[
                &inference.id,
                &inference.input_text,
                &inference.beam_tensor_id,
                &inference.onnx_model_id,
                &inference.flux_position,
                &inference.archetype,
                &inference.confidence,
                &inference.hallucination_detected,
                &inference.vortex_intervention,
                &inference.lake_worthy,
                &inference.processing_time_ms,
            ],
        ).await.map_err(|e| SpatialVortexError::Database(e.to_string()))?;
        
        Ok(row.get(0))
    }
    
    /// Log sacred position intervention
    pub async fn log_sacred_intervention(&self, intervention: &SacredIntervention) -> Result<()> {
        let client = self.pool.get().await
            .map_err(|e| SpatialVortexError::Database(e.to_string()))?;
        
        client.execute(
            r#"
            INSERT INTO sacred_interventions (
                inference_id, position, signal_before, signal_after,
                confidence_boost, intervention_type
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            &[
                &intervention.inference_id,
                &intervention.position,
                &intervention.signal_before,
                &intervention.signal_after,
                &intervention.confidence_boost,
                &intervention.intervention_type,
            ],
        ).await.map_err(|e| SpatialVortexError::Database(e.to_string()))?;
        
        Ok(())
    }
    
    /// Get ASI performance metrics
    pub async fn get_asi_metrics(&self) -> Result<ASIMetrics> {
        let client = self.pool.get().await
            .map_err(|e| SpatialVortexError::Database(e.to_string()))?;
        
        let row = client.query_one(
            r#"
            SELECT 
                COUNT(*) as total_inferences,
                AVG(confidence) as avg_confidence,
                SUM(CASE WHEN hallucination_detected THEN 1 ELSE 0 END) as hallucination_count,
                SUM(CASE WHEN vortex_intervention THEN 1 ELSE 0 END) as intervention_count,
                SUM(CASE WHEN lake_worthy THEN 1 ELSE 0 END) as lake_worthy_count
            FROM asi_inferences
            WHERE created_at >= NOW() - INTERVAL '24 hours'
            "#,
            &[],
        ).await.map_err(|e| SpatialVortexError::Database(e.to_string()))?;
        
        Ok(ASIMetrics {
            total_inferences: row.get::<_, i64>(0) as usize,
            avg_confidence: row.get::<_, Option<f64>>(1).unwrap_or(0.0) as f32,
            hallucination_count: row.get::<_, i64>(2) as usize,
            intervention_count: row.get::<_, i64>(3) as usize,
            lake_worthy_count: row.get::<_, i64>(4) as usize,
        })
    }
    
    /// Run ASI migration
    pub async fn run_asi_migration(&self) -> Result<()> {
        let migration_sql = include_str!("../../migrations/001_asi_tables.sql");
        let client = self.pool.get().await
            .map_err(|e| SpatialVortexError::Database(e.to_string()))?;
        
        client.batch_execute(migration_sql).await
            .map_err(|e| SpatialVortexError::Database(format!("Migration failed: {}", e)))?;
        
        Ok(())
    }
}

/// ASI performance metrics
#[derive(Debug, Clone)]
pub struct ASIMetrics {
    pub total_inferences: usize,
    pub avg_confidence: f32,
    pub hallucination_count: usize,
    pub intervention_count: usize,
    pub lake_worthy_count: usize,
}
