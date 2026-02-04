//! PostgreSQL Pattern Storage Backend
//!
//! Production-ready pattern storage using PostgreSQL for:
//! - Persistent pattern storage
//! - Fast similarity search
//! - Concurrent access
//! - ACID guarantees

use super::meta_learning::{
    ReasoningPattern, QuerySignature, TransformationSnapshot, LearningMetrics, PatternStorage,
};
use anyhow::{Context, Result};
#[cfg(feature = "postgres")]
use sqlx::postgres::{PgPool, PgPoolOptions};
#[cfg(feature = "postgres")]
use sqlx::Row;
use uuid::Uuid;

/// PostgreSQL-backed pattern storage
pub struct PostgresPatternStorage {
    pool: PgPool,
}

impl PostgresPatternStorage {
    /// Create new PostgreSQL storage with connection pool
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await
            .context("Failed to connect to PostgreSQL")?;
        
        Ok(Self { pool })
    }
    
    /// Initialize database schema
    pub async fn init_schema(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS reasoning_patterns (
                pattern_id UUID PRIMARY KEY,
                created_at TIMESTAMPTZ NOT NULL,
                updated_at TIMESTAMPTZ NOT NULL,
                
                -- Query signature
                domain TEXT NOT NULL,
                complexity REAL NOT NULL,
                keywords TEXT[] NOT NULL,
                elp_dominant CHAR(1) NOT NULL,
                
                -- ELP profile
                ethos DOUBLE PRECISION NOT NULL,
                logos DOUBLE PRECISION NOT NULL,
                pathos DOUBLE PRECISION NOT NULL,
                
                -- Solution pathway
                entropy_type TEXT NOT NULL,
                vortex_path INTEGER[] NOT NULL,
                sacred_influences INTEGER[] NOT NULL,
                oracle_questions TEXT[] NOT NULL,
                transformations JSONB NOT NULL,
                
                -- Effectiveness metrics
                success_rate REAL NOT NULL,
                avg_steps INTEGER NOT NULL,
                confidence_achieved REAL NOT NULL,
                reuse_count INTEGER NOT NULL DEFAULT 0,
                
                -- Quality signals
                confidence REAL NOT NULL,
                efficiency_score REAL NOT NULL
            );
            
            -- Indexes for fast lookup
            CREATE INDEX IF NOT EXISTS idx_patterns_domain 
                ON reasoning_patterns(domain);
            CREATE INDEX IF NOT EXISTS idx_patterns_success 
                ON reasoning_patterns(success_rate DESC);
            CREATE INDEX IF NOT EXISTS idx_patterns_signal 
                ON reasoning_patterns(confidence DESC);
            CREATE INDEX IF NOT EXISTS idx_patterns_elp_dominant 
                ON reasoning_patterns(elp_dominant);
            CREATE INDEX IF NOT EXISTS idx_patterns_domain_elp 
                ON reasoning_patterns(domain, elp_dominant);
            "#
        )
        .execute(&self.pool)
        .await
        .context("Failed to create schema")?;
        
        tracing::info!("✅ PostgreSQL schema initialized");
        Ok(())
    }
}

#[async_trait::async_trait]
impl PatternStorage for PostgresPatternStorage {
    async fn store(&self, pattern: ReasoningPattern) -> Result<()> {
        // Convert vortex_path Vec<u8> to Vec<i32> for PostgreSQL
        let vortex_path: Vec<i32> = pattern.vortex_path.iter().map(|&x| x as i32).collect();
        let sacred_influences: Vec<i32> = pattern.sacred_influences.iter().map(|&x| x as i32).collect();
        
        // Serialize transformations to JSON
        let transformations_json = serde_json::to_value(&pattern.key_transformations)
            .context("Failed to serialize transformations")?;
        
        sqlx::query(
            r#"
            INSERT INTO reasoning_patterns (
                pattern_id, created_at, updated_at,
                domain, complexity, keywords, elp_dominant,
                ethos, logos, pathos,
                entropy_type, vortex_path, sacred_influences, oracle_questions, transformations,
                success_rate, avg_steps, confidence_achieved, reuse_count,
                confidence, efficiency_score
            ) VALUES (
                $1, $2, $3,
                $4, $5, $6, $7,
                $8, $9, $10,
                $11, $12, $13, $14, $15,
                $16, $17, $18, $19,
                $20, $21
            )
            ON CONFLICT (pattern_id) DO UPDATE SET
                updated_at = $3,
                success_rate = $16,
                avg_steps = $17,
                reuse_count = $19,
                efficiency_score = $21
            "#
        )
        .bind(pattern.pattern_id)
        .bind(pattern.created_at)
        .bind(pattern.updated_at)
        .bind(&pattern.query_signature.domain)
        .bind(pattern.query_signature.complexity)
        .bind(&pattern.query_signature.keywords)
        .bind(pattern.query_signature.elp_dominant.to_string())
        .bind(pattern.elp_profile.ethos)
        .bind(pattern.elp_profile.logos)
        .bind(pattern.elp_profile.pathos)
        .bind(format!("{:?}", pattern.entropy_type))
        .bind(&vortex_path)
        .bind(&sacred_influences)
        .bind(&pattern.oracle_questions)
        .bind(transformations_json)
        .bind(pattern.success_rate)
        .bind(pattern.avg_steps as i32)
        .bind(pattern.confidence_achieved)
        .bind(pattern.reuse_count as i32)
        .bind(pattern.confidence)
        .bind(pattern.efficiency_score)
        .execute(&self.pool)
        .await
        .context("Failed to store pattern")?;
        
        tracing::debug!("💾 Pattern {} stored in PostgreSQL", pattern.pattern_id);
        Ok(())
    }
    
    async fn find_similar(
        &self,
        signature: &QuerySignature,
        limit: usize,
    ) -> Result<Vec<ReasoningPattern>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                pattern_id, created_at, updated_at,
                domain, complexity, keywords, elp_dominant,
                ethos, logos, pathos,
                entropy_type, vortex_path, sacred_influences, oracle_questions, transformations,
                success_rate, avg_steps, confidence_achieved, reuse_count,
                confidence, efficiency_score
            FROM reasoning_patterns
            WHERE domain = $1 
              AND success_rate >= 0.5
            ORDER BY success_rate DESC, confidence DESC
            LIMIT $2
            "#
        )
        .bind(&signature.domain)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .context("Failed to query patterns")?;
        
        let mut patterns = Vec::new();
        
        for row in rows {
            // Parse entropy type
            let entropy_type_str: String = row.try_get("entropy_type")?;
            let entropy_type = match entropy_type_str.as_str() {
                "MissingFacts" => crate::ai::flux_reasoning::EntropyType::MissingFacts,
                "UnclearCausality" => crate::ai::flux_reasoning::EntropyType::UnclearCausality,
                "MultiplePathways" => crate::ai::flux_reasoning::EntropyType::MultiplePathways,
                "EthicalAmbiguity" => crate::ai::flux_reasoning::EntropyType::EthicalAmbiguity,
                _ => crate::ai::flux_reasoning::EntropyType::Low,
            };
            
            // Convert i32 arrays back to u8
            let vortex_path_i32: Vec<i32> = row.try_get("vortex_path")?;
            let vortex_path: Vec<u8> = vortex_path_i32.iter().map(|&x| x as u8).collect();
            
            let sacred_i32: Vec<i32> = row.try_get("sacred_influences")?;
            let sacred_influences: Vec<u8> = sacred_i32.iter().map(|&x| x as u8).collect();
            
            // Deserialize transformations
            let transformations_json: serde_json::Value = row.try_get("transformations")?;
            let key_transformations: Vec<TransformationSnapshot> = 
                serde_json::from_value(transformations_json)
                    .context("Failed to deserialize transformations")?;
            
            // Parse ELP dominant
            let elp_dominant_str: String = row.try_get("elp_dominant")?;
            let elp_dominant = elp_dominant_str.chars().next().unwrap_or('L');
            
            patterns.push(ReasoningPattern {
                pattern_id: row.try_get("pattern_id")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
                query_signature: QuerySignature {
                    domain: row.try_get("domain")?,
                    complexity: row.try_get("complexity")?,
                    keywords: row.try_get("keywords")?,
                    elp_dominant,
                },
                elp_profile: crate::data::models::ELPTensor {
                    ethos: row.try_get("ethos")?,
                    logos: row.try_get("logos")?,
                    pathos: row.try_get("pathos")?,
                },
                entropy_type,
                vortex_path,
                sacred_influences,
                oracle_questions: row.try_get("oracle_questions")?,
                key_transformations,
                success_rate: row.try_get("success_rate")?,
                avg_steps: row.try_get::<i32, _>("avg_steps")? as usize,
                confidence_achieved: row.try_get("confidence_achieved")?,
                reuse_count: row.try_get::<i32, _>("reuse_count")? as u32,
                confidence: row.try_get("confidence")?,
                efficiency_score: row.try_get("efficiency_score")?,
            });
        }
        
        tracing::debug!("🔍 Found {} similar patterns for domain: {}", 
            patterns.len(), signature.domain);
        
        Ok(patterns)
    }
    
    async fn update_metrics(
        &self,
        pattern_id: Uuid,
        success: bool,
        actual_steps: usize,
    ) -> Result<()> {
        // Use exponential moving average with alpha = 0.1
        let alpha = 0.1_f32;
        
        sqlx::query(
            r#"
            UPDATE reasoning_patterns
            SET 
                updated_at = NOW(),
                reuse_count = reuse_count + 1,
                success_rate = CASE 
                    WHEN $2 THEN success_rate * $3 + $3
                    ELSE success_rate * $3
                END,
                avg_steps = CAST((avg_steps * $4 + $5 * $3) AS INTEGER)
            WHERE pattern_id = $1
            "#
        )
        .bind(pattern_id)
        .bind(success)
        .bind(alpha)
        .bind(1.0 - alpha)
        .bind(actual_steps as i32)
        .execute(&self.pool)
        .await
        .context("Failed to update pattern metrics")?;
        
        tracing::debug!("📊 Pattern {} metrics updated (success: {})", pattern_id, success);
        Ok(())
    }
    
    async fn prune_ineffective(&self, min_success_rate: f32) -> Result<usize> {
        let result = sqlx::query(
            r#"
            DELETE FROM reasoning_patterns
            WHERE success_rate < $1
            "#
        )
        .bind(min_success_rate)
        .execute(&self.pool)
        .await
        .context("Failed to prune patterns")?;
        
        let pruned = result.rows_affected() as usize;
        
        if pruned > 0 {
            tracing::info!("🧹 Pruned {} patterns with success < {:.1}%", 
                pruned, min_success_rate * 100.0);
        }
        
        Ok(pruned)
    }
    
    async fn get_metrics(&self) -> Result<LearningMetrics> {
        let row = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as total,
                AVG(reuse_count) as avg_reuse,
                AVG(success_rate) as avg_success
            FROM reasoning_patterns
            "#
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to get metrics")?;
        
        let total: i64 = row.try_get("total")?;
        
        if total == 0 {
            return Ok(LearningMetrics::default());
        }
        
        let avg_reuse: f64 = row.try_get("avg_reuse").unwrap_or(0.0);
        let avg_success: f64 = row.try_get("avg_success").unwrap_or(0.0);
        
        Ok(LearningMetrics {
            patterns_extracted: total as u64,
            patterns_active: total as u64,
            patterns_pruned: 0, // Would need separate tracking
            avg_reuse_count: avg_reuse as f32,
            avg_success_rate: avg_success as f32,
            acceleration_rate: 0.0, // Tracked externally
            avg_speedup: 0.0, // Tracked externally
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    #[ignore] // Requires PostgreSQL connection
    async fn test_postgres_storage() {
        // Set DATABASE_URL environment variable for testing
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://localhost/spatialvortex_test".to_string());
        
        let storage = PostgresPatternStorage::new(&database_url).await.unwrap();
        storage.init_schema().await.unwrap();
        
        // Test will use actual database
        // Add pattern creation and retrieval tests here
    }
}
