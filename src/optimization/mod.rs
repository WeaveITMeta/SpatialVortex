//! Performance Optimization Module
//!
//! Addresses identified bottlenecks in API, voice pipeline, inference, and database layers
//! to improve throughput from 200-500 req/sec to target 1000+ req/sec.

pub mod api_optimizer;
pub mod voice_optimizer;
pub mod inference_optimizer;
pub mod db_optimizer;
pub mod batch_processor;
pub mod cache_layer;
pub mod config_optimizer;

use std::sync::Arc;
use tokio::sync::RwLock;

/// Performance metrics tracking
#[derive(Debug, Default, Clone)]
pub struct PerformanceMetrics {
    pub api_throughput: f64,
    pub voice_latency_ms: f64,
    pub inference_latency_ms: f64,
    pub db_query_time_ms: f64,
    pub cache_hit_rate: f64,
    pub total_requests: u64,
    pub errors: u64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
}

/// Global performance monitor
pub struct PerformanceMonitor {
    metrics: Arc<RwLock<PerformanceMetrics>>,
    start_time: std::time::Instant,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
            start_time: std::time::Instant::now(),
        }
    }
    
    pub async fn update_metric<F>(&self, updater: F) 
    where
        F: FnOnce(&mut PerformanceMetrics)
    {
        let mut metrics = self.metrics.write().await;
        updater(&mut metrics);
    }
    
    pub async fn get_metrics(&self) -> PerformanceMetrics {
        self.metrics.read().await.clone()
    }
    
    pub fn elapsed_secs(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }
}

/// Optimization configuration
#[derive(Debug, Clone)]
pub struct OptimizationConfig {
    // API Optimization
    pub worker_threads: usize,
    pub max_connections: usize,
    pub enable_compression: bool,
    pub json_buffer_size: usize,
    
    // Voice Optimization  
    pub audio_buffer_size: usize,
    pub enable_simd: bool,
    pub whisper_batch_size: usize,
    pub use_gpu_acceleration: bool,
    
    // Inference Optimization
    pub onnx_session_pool_size: usize,
    pub enable_tensorrt: bool,
    pub batch_inference_size: usize,
    pub model_cache_size: usize,
    
    // Database Optimization
    pub connection_pool_size: usize,
    pub enable_prepared_statements: bool,
    pub batch_insert_size: usize,
    pub query_timeout_ms: u64,
    
    // Cache Configuration
    pub enable_redis_cache: bool,
    pub cache_ttl_secs: u64,
    pub max_cache_size_mb: usize,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        let num_cpus = num_cpus::get();
        
        Self {
            // API: 2x cores for workers as recommended
            worker_threads: num_cpus * 2,
            max_connections: 10000,
            enable_compression: true,
            json_buffer_size: 8192,
            
            // Voice: Larger buffers to prevent crackles
            audio_buffer_size: 1024,  // Increased from 512
            enable_simd: true,
            whisper_batch_size: 4,
            use_gpu_acceleration: false,  // Enable if GPU available
            
            // Inference: Pool sessions for reuse
            onnx_session_pool_size: num_cpus,
            enable_tensorrt: false,
            batch_inference_size: 32,
            model_cache_size: 5,
            
            // Database: Connection pooling
            connection_pool_size: num_cpus * 4,
            enable_prepared_statements: true,
            batch_insert_size: 1000,
            query_timeout_ms: 5000,
            
            // Cache: In-memory caching
            enable_redis_cache: false,
            cache_ttl_secs: 300,
            max_cache_size_mb: 512,
        }
    }
}

/// Load optimization config from environment
impl OptimizationConfig {
    pub fn from_env() -> Self {
        let mut config = Self::default();
        
        // Override from env vars
        if let Ok(workers) = std::env::var("ACTIX_WORKERS") {
            if let Ok(n) = workers.parse() {
                config.worker_threads = n;
            }
        }
        
        if let Ok(buffer) = std::env::var("AUDIO_BUFFER_SIZE") {
            if let Ok(n) = buffer.parse() {
                config.audio_buffer_size = n;
            }
        }
        
        if let Ok(pool) = std::env::var("DB_POOL_SIZE") {
            if let Ok(n) = pool.parse() {
                config.connection_pool_size = n;
            }
        }
        
        if std::env::var("USE_GPU").unwrap_or_default() == "true" {
            config.use_gpu_acceleration = true;
            config.enable_tensorrt = true;
        }
        
        config
    }
}
