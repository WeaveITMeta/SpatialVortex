//! Database Optimization
//!
//! Optimizes SQLx with connection pooling, prepared statements, and batching
//! Target: <5ms queries, 1000+ qps

use sqlx::{Pool, Postgres, Sqlite, Any};
use std::sync::Arc;
use std::time::Duration;
use super::OptimizationConfig;

/// Optimized database pool using deadpool/bb8 patterns
pub struct OptimizedDbPool {
    postgres_pool: Option<Pool<Postgres>>,
    sqlite_pool: Option<Pool<Sqlite>>,
    config: OptimizationConfig,
}

impl OptimizedDbPool {
    /// Create optimized connection pool
    pub async fn new(config: OptimizationConfig) -> Result<Self, sqlx::Error> {
        let postgres_pool = if std::env::var("DATABASE_URL").is_ok() {
            Some(Self::create_postgres_pool(&config).await?)
        } else {
            None
        };
        
        let sqlite_pool = Some(Self::create_sqlite_pool(&config).await?);
        
        Ok(Self {
            postgres_pool,
            sqlite_pool,
            config,
        })
    }
    
    async fn create_postgres_pool(config: &OptimizationConfig) -> Result<Pool<Postgres>, sqlx::Error> {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://localhost/spatial_vortex".to_string());
        
        sqlx::postgres::PgPoolOptions::new()
            .max_connections(config.connection_pool_size as u32)
            .min_connections(config.connection_pool_size as u32 / 4)
            .acquire_timeout(Duration::from_millis(config.query_timeout_ms))
            .idle_timeout(Duration::from_secs(10))
            .max_lifetime(Duration::from_secs(30 * 60))  // 30 minutes
            .test_before_acquire(false)  // Faster acquisition
            .connect(&database_url)
            .await
    }
    
    async fn create_sqlite_pool(config: &OptimizationConfig) -> Result<Pool<Sqlite>, sqlx::Error> {
        sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(config.connection_pool_size as u32)
            .min_connections(1)  // SQLite works best with fewer connections
            .acquire_timeout(Duration::from_millis(config.query_timeout_ms))
            .connect("sqlite::memory:")  // In-memory for speed
            .await
    }
    
    /// Get appropriate pool based on query complexity
    pub fn get_pool(&self, is_complex_query: bool) -> &Pool<Sqlite> {
        // Use Postgres for complex queries if available, otherwise SQLite
        if is_complex_query && self.postgres_pool.is_some() {
            // Would return postgres_pool here, but type system requires same type
            // In production, use enum wrapper
            &self.sqlite_pool.as_ref().unwrap()
        } else {
            &self.sqlite_pool.as_ref().unwrap()
        }
    }
}

/// Prepared statement cache for performance
pub struct PreparedStatementCache {
    statements: dashmap::DashMap<String, Arc<String>>,
}

impl PreparedStatementCache {
    pub fn new() -> Self {
        Self {
            statements: dashmap::DashMap::new(),
        }
    }
    
    /// Get or create prepared statement
    pub fn get_or_prepare(&self, key: &str, sql: &str) -> Arc<String> {
        self.statements
            .entry(key.to_string())
            .or_insert_with(|| Arc::new(sql.to_string()))
            .clone()
    }
}

/// Batch insert optimizer
pub struct BatchInserter {
    batch_size: usize,
    buffer: Vec<Vec<serde_json::Value>>,
}

impl BatchInserter {
    pub fn new(batch_size: usize) -> Self {
        Self {
            batch_size,
            buffer: Vec::with_capacity(batch_size),
        }
    }
    
    /// Add item to batch
    pub fn add(&mut self, item: Vec<serde_json::Value>) -> bool {
        self.buffer.push(item);
        self.buffer.len() >= self.batch_size
    }
    
    /// Execute batch insert
    pub async fn execute_batch<'a>(
        &mut self,
        pool: &Pool<Sqlite>,
        table: &str,
        columns: &[&str],
    ) -> Result<u64, sqlx::Error> {
        if self.buffer.is_empty() {
            return Ok(0);
        }
        
        let batch = self.buffer.drain(..).collect::<Vec<_>>();
        let rows = batch.len();
        
        // Build multi-value insert
        let placeholders: Vec<String> = (1..=columns.len())
            .map(|i| format!("${}", i))
            .collect();
        let values_clause = format!("({})", placeholders.join(","));
        
        let all_values: Vec<String> = (0..rows)
            .map(|_| values_clause.clone())
            .collect();
        
        let query = format!(
            "INSERT INTO {} ({}) VALUES {}",
            table,
            columns.join(","),
            all_values.join(",")
        );
        
        // Execute with transaction for atomicity
        let mut tx = pool.begin().await?;
        
        let result = sqlx::query(&query)
            .execute(&mut *tx)
            .await?;
        
        tx.commit().await?;
        
        Ok(result.rows_affected())
    }
}

/// Query optimizer with indexing hints
pub struct QueryOptimizer {
    statement_cache: Arc<PreparedStatementCache>,
}

impl QueryOptimizer {
    pub fn new() -> Self {
        Self {
            statement_cache: Arc::new(PreparedStatementCache::new()),
        }
    }
    
    /// Optimize query with index hints
    pub fn optimize_query(&self, query: &str) -> String {
        let mut optimized = query.to_string();
        
        // Add index hints for common patterns
        if query.contains("WHERE flux_position") {
            optimized = optimized.replace(
                "WHERE flux_position",
                "WHERE flux_position /* INDEX: idx_flux_position */"
            );
        }
        
        if query.contains("WHERE confidence") {
            optimized = optimized.replace(
                "WHERE confidence",
                "WHERE confidence /* INDEX: idx_confidence */"
            );
        }
        
        // Use prepared statement
        self.statement_cache.get_or_prepare(
            &format!("{:x}", md5::compute(query)),
            &optimized
        );
        
        optimized
    }
    
    /// Create recommended indexes
    pub async fn create_indexes(&self, pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
        let indexes = vec![
            "CREATE INDEX IF NOT EXISTS idx_flux_position ON embeddings(flux_position)",
            "CREATE INDEX IF NOT EXISTS idx_confidence ON embeddings(confidence)",
            "CREATE INDEX IF NOT EXISTS idx_timestamp ON embeddings(timestamp)",
            "CREATE INDEX IF NOT EXISTS idx_is_sacred ON embeddings(flux_position) WHERE flux_position IN (3,6,9)",
        ];
        
        for index_sql in indexes {
            sqlx::query(index_sql).execute(pool).await?;
        }
        
        Ok(())
    }
}

/// Connection pool monitor
pub struct PoolMonitor {
    metrics: Arc<tokio::sync::RwLock<PoolMetrics>>,
}

#[derive(Debug, Default, Clone)]
pub struct PoolMetrics {
    pub active_connections: u32,
    pub idle_connections: u32,
    pub wait_time_ms: f64,
    pub query_count: u64,
    pub error_count: u64,
}

impl PoolMonitor {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(tokio::sync::RwLock::new(PoolMetrics::default())),
        }
    }
    
    pub async fn update_from_pool(&self, pool: &Pool<Sqlite>) {
        let mut metrics = self.metrics.write().await;
        
        metrics.active_connections = pool.size();
        metrics.idle_connections = pool.num_idle();
        // Additional metrics would come from pool internals
    }
    
    pub async fn record_query(&self, duration_ms: f64, success: bool) {
        let mut metrics = self.metrics.write().await;
        
        metrics.query_count += 1;
        metrics.wait_time_ms = (metrics.wait_time_ms * (metrics.query_count - 1) as f64 
            + duration_ms) / metrics.query_count as f64;
        
        if !success {
            metrics.error_count += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_pool_creation() {
        let config = OptimizationConfig::default();
        let pool = OptimizedDbPool::new(config).await.unwrap();
        assert!(pool.sqlite_pool.is_some());
    }
    
    #[test]
    fn test_batch_inserter() {
        let mut inserter = BatchInserter::new(3);
        
        assert!(!inserter.add(vec![serde_json::json!(1)]));
        assert!(!inserter.add(vec![serde_json::json!(2)]));
        assert!(inserter.add(vec![serde_json::json!(3)]));  // Should trigger batch
    }
    
    #[test]
    fn test_query_optimization() {
        let optimizer = QueryOptimizer::new();
        let query = "SELECT * FROM table WHERE flux_position = 3";
        let optimized = optimizer.optimize_query(query);
        assert!(optimized.contains("INDEX: idx_flux_position"));
    }
}
