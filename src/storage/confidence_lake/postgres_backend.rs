//! PostgreSQL backend for Confidence Lake using sqlx
//!
//! Provides persistent storage with advanced querying capabilities,
//! combining AES-GCM-SIV encryption with PostgreSQL for metadata and indexing.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
#[cfg(feature = "postgres")]
use sqlx::{postgres::{PgPool, PgPoolOptions}, Row};

#[cfg(feature = "lake")]
use super::encryption::SecureStorage;

/// Flux Matrix entry for Confidence Lake
///
/// Represents a high-value moment captured from processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredFluxMatrix {
    /// Unique identifier (timestamp in milliseconds)
    pub id: i64,
    /// Confidence (0.0-1.0, must be >= 0.6)
    /// CONSOLIDATED: Replaces both confidence and confidence
    pub confidence: f64,
    /// Flux matrix position (0-9)
    pub flux_position: u8,
    /// Whether this is a sacred position (3, 6, 9)
    pub is_sacred: bool,
    /// ELP channel values
    pub ethos: f64,
    pub logos: f64,
    pub pathos: f64,
    /// Processing mode used
    pub mode: String,
    /// Processing time in milliseconds
    pub processing_time_ms: u32,
    /// Encrypted data blob
    pub data: Vec<u8>,
}

/// PostgreSQL-backed Confidence Lake with encryption
pub struct PostgresConfidenceLake {
    /// Database connection pool
    pool: PgPool,
    /// Optional encryption for secure storage
    #[cfg(feature = "lake")]
    encryption: Option<SecureStorage>,
}

impl PostgresConfidenceLake {
    /// Creates a new PostgreSQL-backed Confidence Lake from DATABASE_URL env var
    ///
    /// Loads connection string from `DATABASE_URL` environment variable.
    /// Uses dotenv to load from .env file if present.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use spatial_vortex::storage::confidence_lake::PostgresConfidenceLake;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// // Reads from DATABASE_URL in .env file
    /// let lake = PostgresConfidenceLake::from_env().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn from_env() -> Result<Self> {
        // Load .env file if present
        let _ = dotenv::dotenv();
        
        let connection_string = std::env::var("DATABASE_URL")
            .context("DATABASE_URL environment variable not set")?;
        
        Self::new(&connection_string).await
    }

    /// Creates a new PostgreSQL-backed Confidence Lake with explicit connection string
    ///
    /// # Arguments
    ///
    /// * `connection_string` - PostgreSQL connection string
    ///   (e.g., "postgresql://user:pass@localhost/spatialvortex")
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use spatial_vortex::storage::confidence_lake::PostgresConfidenceLake;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let lake = PostgresConfidenceLake::new("postgresql://localhost/confidence").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(connection_string: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(connection_string)
            .await
            .context("Failed to connect to PostgreSQL database")?;
        
        let lake = Self {
            pool,
            #[cfg(feature = "lake")]
            encryption: None,
        };
        
        lake.initialize_schema().await?;
        Ok(lake)
    }
    
    /// Initializes the database schema
    async fn initialize_schema(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS flux_matrices (
                id BIGINT PRIMARY KEY,
                confidence DOUBLE PRECISION NOT NULL,
                flux_position INTEGER NOT NULL,
                is_sacred BOOLEAN NOT NULL,
                ethos DOUBLE PRECISION NOT NULL,
                logos DOUBLE PRECISION NOT NULL,
                pathos DOUBLE PRECISION NOT NULL,
                mode TEXT NOT NULL,
                processing_time_ms INTEGER NOT NULL,
                data BYTEA NOT NULL,
                created_at TIMESTAMP DEFAULT NOW()
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .context("Failed to create flux_matrices table")?;
        
        // Create indexes for efficient querying
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_confidence ON flux_matrices(confidence DESC)"
        )
        .execute(&self.pool)
        .await?;
        
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_sacred ON flux_matrices(is_sacred, flux_position)"
        )
        .execute(&self.pool)
        .await?;
        
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_created_at ON flux_matrices(created_at DESC)"
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    /// Enables encryption with a provided key
    #[cfg(feature = "lake")]
    pub fn enable_encryption(&mut self, key: &[u8; 32]) {
        self.encryption = Some(SecureStorage::new(key));
    }
    
    /// Stores a flux matrix (high-value moment) in the lake
    ///
    /// # Arguments
    ///
    /// * `output` - ASI output to store
    ///
    /// # Returns
    ///
    /// * `Result<i64>` - ID of stored flux matrix
    pub async fn store_flux_matrix(&self, output: &crate::ai::orchestrator::ASIOutput) -> Result<i64> {
        if output.confidence < 0.6 {
            anyhow::bail!("Confidence too low for Confidence Lake: {:.2}", output.confidence);
        }
        
        // Serialize the output
        let data = serde_json::to_vec(output)?;
        
        // Encrypt if enabled
        #[cfg(feature = "lake")]
        let encrypted_data = if let Some(ref encryption) = self.encryption {
            encryption.encrypt(&data)?
        } else {
            data
        };
        
        #[cfg(not(feature = "lake"))]
        let encrypted_data = data;
        
        let id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        
        sqlx::query(
            r#"
            INSERT INTO flux_matrices (
                id, confidence, flux_position, is_sacred,
                ethos, logos, pathos, mode, processing_time_ms, data
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(id)
        .bind(output.confidence as f64)
        .bind(output.flux_position as i32)
        .bind(output.is_sacred)
        .bind(output.elp.ethos)
        .bind(output.elp.logos)
        .bind(output.elp.pathos)
        .bind(format!("{:?}", output.mode))
        .bind(output.processing_time_ms as i32)
        .bind(encrypted_data)
        .execute(&self.pool)
        .await
        .context("Failed to store flux matrix")?;
        
        Ok(id)
    }
    
    /// Retrieves a flux matrix by ID
    pub async fn retrieve_flux_matrix(&self, id: i64) -> Result<StoredFluxMatrix> {
        let row = sqlx::query(
            r#"
            SELECT id, confidence, flux_position, is_sacred,
                   ethos, logos, pathos, mode, processing_time_ms, data
            FROM flux_matrices
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .context("Flux matrix not found")?;
        
        let encrypted_data: Vec<u8> = row.get("data");
        
        // Decrypt if enabled
        #[cfg(feature = "lake")]
        let data = if let Some(ref encryption) = self.encryption {
            encryption.decrypt(&encrypted_data)?
        } else {
            encrypted_data
        };
        
        #[cfg(not(feature = "lake"))]
        let data = encrypted_data;
        
        Ok(StoredFluxMatrix {
            id: row.get("id"),
            confidence: row.get("confidence"),
            flux_position: row.get::<i32, _>("flux_position") as u8,
            is_sacred: row.get("is_sacred"),
            ethos: row.get("ethos"),
            logos: row.get("logos"),
            pathos: row.get("pathos"),
            mode: row.get("mode"),
            processing_time_ms: row.get::<i32, _>("processing_time_ms") as u32,
            data,
        })
    }
    
    /// Queries flux matrices by minimum confidence
    pub async fn query_by_confidence(&self, min_confidence: f64) -> Result<Vec<StoredFluxMatrix>> {
        let rows = sqlx::query(
            r#"
            SELECT id, confidence, flux_position, is_sacred,
                   ethos, logos, pathos, mode, processing_time_ms, data
            FROM flux_matrices
            WHERE confidence >= $1
            ORDER BY confidence DESC
            "#,
        )
        .bind(min_confidence)
        .fetch_all(&self.pool)
        .await?;
        
        let mut flux_matrices = Vec::new();
        for row in rows {
            let encrypted_data: Vec<u8> = row.get("data");
            
            #[cfg(feature = "lake")]
            let data = if let Some(ref encryption) = self.encryption {
                encryption.decrypt(&encrypted_data)?
            } else {
                encrypted_data
            };
            
            #[cfg(not(feature = "lake"))]
            let data = encrypted_data;
            
            flux_matrices.push(StoredFluxMatrix {
                id: row.get("id"),
                confidence: row.get("confidence"),
                flux_position: row.get::<i32, _>("flux_position") as u8,
                is_sacred: row.get("is_sacred"),
                ethos: row.get("ethos"),
                logos: row.get("logos"),
                pathos: row.get("pathos"),
                mode: row.get("mode"),
                processing_time_ms: row.get::<i32, _>("processing_time_ms") as u32,
                data,
            });
        }
        
        Ok(flux_matrices)
    }
    
    /// Queries sacred position flux matrices (3, 6, 9)
    pub async fn query_sacred_flux_matrices(&self) -> Result<Vec<StoredFluxMatrix>> {
        let rows = sqlx::query(
            r#"
            SELECT id, confidence, flux_position, is_sacred,
                   ethos, logos, pathos, mode, processing_time_ms, data
            FROM flux_matrices
            WHERE is_sacred = true
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        
        let mut flux_matrices = Vec::new();
        for row in rows {
            let encrypted_data: Vec<u8> = row.get("data");
            
            #[cfg(feature = "lake")]
            let data = if let Some(ref encryption) = self.encryption {
                encryption.decrypt(&encrypted_data)?
            } else {
                encrypted_data
            };
            
            #[cfg(not(feature = "lake"))]
            let data = encrypted_data;
            
            flux_matrices.push(StoredFluxMatrix {
                id: row.get("id"),
                confidence: row.get("confidence"),
                flux_position: row.get::<i32, _>("flux_position") as u8,
                is_sacred: row.get("is_sacred"),
                ethos: row.get("ethos"),
                logos: row.get("logos"),
                pathos: row.get("pathos"),
                mode: row.get("mode"),
                processing_time_ms: row.get::<i32, _>("processing_time_ms") as u32,
                data,
            });
        }
        
        Ok(flux_matrices)
    }
    
    /// Gets statistics about the lake
    pub async fn get_stats(&self) -> Result<LakeStats> {
        let row = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as total_flux_matrices,
                AVG(confidence) as avg_confidence,
                MAX(confidence) as max_confidence,
                MIN(confidence) as min_confidence,
                COUNT(CASE WHEN is_sacred = true THEN 1 END) as sacred_count
            FROM flux_matrices
            "#,
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(LakeStats {
            total_flux_matrices: row.get::<i32, _>("total_flux_matrices") as usize,
            avg_confidence: row.get("avg_confidence"),
            max_confidence: row.get("max_confidence"),
            min_confidence: row.get("min_confidence"),
            sacred_count: row.get::<i32, _>("sacred_count") as usize,
        })
    }
    
    /// Prunes old entries beyond retention limit
    pub async fn prune(&self, keep_count: usize) -> Result<usize> {
        let result = sqlx::query(
            r#"
            DELETE FROM flux_matrices
            WHERE id NOT IN (
                SELECT id FROM flux_matrices
                ORDER BY created_at DESC
                LIMIT $1
            )
            "#,
        )
        .bind(keep_count as i32)
        .execute(&self.pool)
        .await?;
        
        Ok(result.rows_affected() as usize)
    }
}

/// Statistics about the Confidence Lake
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LakeStats {
    pub total_flux_matrices: usize,
    pub avg_confidence: f64,
    pub max_confidence: f64,
    pub min_confidence: f64,
    pub sacred_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::orchestrator::{ASIOutput, ExecutionMode};
    use crate::models::ELPTensor;
    
    // NOTE: These tests require a running PostgreSQL instance
    // Set DATABASE_URL environment variable to run tests:
    // export DATABASE_URL="postgresql://localhost/test_confidence"
    
    #[tokio::test]
    #[ignore] // Requires PostgreSQL instance
    async fn test_postgres_lake_creation() {
        let url = std::env::var("DATABASE_URL").unwrap_or("postgresql://localhost/test".to_string());
        let lake = PostgresConfidenceLake::new(&url).await.unwrap();
        let stats = lake.get_stats().await.unwrap();
        
        assert_eq!(stats.total_flux_matrices, 0);
    }
    
    #[tokio::test]
    #[ignore] // Requires PostgreSQL instance
    async fn test_store_and_retrieve_flux_matrix() {
        let url = std::env::var("DATABASE_URL").unwrap_or("postgresql://localhost/test".to_string());
        let lake = PostgresConfidenceLake::new(&url).await.unwrap();
        
        let output = ASIOutput {
            result: "Test result".to_string(),
            elp: ELPTensor {
                ethos: 5.0,
                logos: 7.0,
                pathos: 6.0,
            },
            flux_position: 6,
            confidence: 0.85,
            is_sacred: true,
            mode: ExecutionMode::Balanced,
            consensus_used: false,
            processing_time_ms: 150,
        };
        
        let id = lake.store_flux_matrix(&output).await.unwrap();
        assert!(id > 0);
        
        let flux_matrix = lake.retrieve_flux_matrix(id).await.unwrap();
        assert_eq!(flux_matrix.confidence, 0.85);
        assert_eq!(flux_matrix.flux_position, 6);
        assert!(flux_matrix.is_sacred);
    }
    
    #[tokio::test]
    #[ignore] // Requires PostgreSQL instance
    async fn test_reject_low_confidence() {
        let url = std::env::var("DATABASE_URL").unwrap_or("postgresql://localhost/test".to_string());
        let lake = PostgresConfidenceLake::new(&url).await.unwrap();
        
        let output = ASIOutput {
            result: "Low signal".to_string(),
            elp: ELPTensor {
                ethos: 3.0,
                logos: 3.0,
                pathos: 3.0,
            },
            flux_position: 1,
            confidence: 0.5, // Below threshold
            is_sacred: false,
            mode: ExecutionMode::Fast,
            consensus_used: false,
            processing_time_ms: 50,
        };
        
        let result = lake.store_flux_matrix(&output).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    #[ignore] // Requires PostgreSQL instance
    async fn test_query_sacred_flux_matrices() {
        let url = std::env::var("DATABASE_URL").unwrap_or("postgresql://localhost/test".to_string());
        let lake = PostgresConfidenceLake::new(&url).await.unwrap();
        
        // Store sacred flux matrix
        let sacred_output = ASIOutput {
            result: "Sacred".to_string(),
            elp: ELPTensor {
                ethos: 6.0,
                logos: 6.0,
                pathos: 6.0,
            },
            flux_position: 9,
            confidence: 0.9,
            is_sacred: true,
            mode: ExecutionMode::Thorough,
            consensus_used: true,
            processing_time_ms: 300,
        };
        
        lake.store_flux_matrix(&sacred_output).await.unwrap();
        
        // Store non-sacred flux matrix
        let normal_output = ASIOutput {
            result: "Normal".to_string(),
            elp: ELPTensor {
                ethos: 4.0,
                logos: 5.0,
                pathos: 4.0,
            },
            flux_position: 2,
            confidence: 0.7,
            is_sacred: false,
            mode: ExecutionMode::Fast,
            consensus_used: false,
            processing_time_ms: 100,
        };
        
        lake.store_flux_matrix(&normal_output).await.unwrap();
        
        let sacred_flux_matrices = lake.query_sacred_flux_matrices().await.unwrap();
        assert_eq!(sacred_flux_matrices.len(), 1);
        assert_eq!(sacred_flux_matrices[0].flux_position, 9);
    }
}
