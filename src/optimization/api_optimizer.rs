//! API Server Optimization
//!
//! Optimizes Actix-Web for higher throughput and lower latency
//! Target: 1000+ req/sec with p95 < 50ms

use actix_web::{middleware, web, App, HttpServer};
use std::time::Duration;
use super::OptimizationConfig;

/// Optimized Actix-Web server configuration
pub struct OptimizedApiServer {
    config: OptimizationConfig,
}

impl OptimizedApiServer {
    pub fn new(config: OptimizationConfig) -> Self {
        Self { config }
    }
    
    /// Create optimized HTTP server
    pub fn create_server<F>(&self, app_config: F) -> std::io::Result<actix_web::dev::Server>
    where
        F: Fn(&mut web::ServiceConfig) + Send + Sync + Clone + 'static,
    {
        // Set worker threads via environment
        std::env::set_var("ACTIX_WORKERS", self.config.worker_threads.to_string());
        
        let server = HttpServer::new(move || {
            let mut app = App::new()
                // Compression middleware for reduced network I/O
                .wrap(middleware::Compress::default())
                // Logging with minimal overhead
                .wrap(middleware::Logger::new("%r %s %D"))
                // Connection keep-alive
                .wrap(
                    middleware::DefaultHeaders::new()
                        .add(("Connection", "keep-alive"))
                        .add(("Keep-Alive", "timeout=5, max=1000"))
                );
            
            // Apply user configuration
            app = app.configure(app_config.clone());
            
            app
        })
        // Optimize connection settings
        .keep_alive(Duration::from_secs(75))
        .client_request_timeout(Duration::from_secs(30))
        .client_disconnect_timeout(Duration::from_secs(5))
        // Increased backlog for high concurrency
        .backlog(8192)
        // Worker configuration
        .workers(self.config.worker_threads)
        // Bind to all interfaces
        .bind("0.0.0.0:8080")?
        .run();
        
        Ok(server)
    }
}

/// JSON serialization optimization with simd-json
pub mod json_optimizer {
    use serde::{Serialize, Deserialize};
    use actix_web::{HttpResponse, Result};
    
    /// Fast JSON serialization using pre-allocated buffers
    pub fn fast_json_response<T: Serialize>(data: &T, buffer_size: usize) -> Result<HttpResponse> {
        let mut buffer = Vec::with_capacity(buffer_size);
        
        // Use simd-json for faster serialization if available
        #[cfg(feature = "simd-json")]
        {
            simd_json::to_writer(&mut buffer, data)?;
        }
        
        #[cfg(not(feature = "simd-json"))]
        {
            serde_json::to_writer(&mut buffer, data)?;
        }
        
        Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(buffer))
    }
    
    /// Batch JSON deserialization
    pub async fn batch_deserialize<T: for<'de> Deserialize<'de>>(
        payloads: Vec<bytes::Bytes>
    ) -> Result<Vec<T>> {
        use rayon::prelude::*;
        
        let results: Vec<T> = payloads
            .into_par_iter()
            .map(|bytes| {
                serde_json::from_slice(&bytes)
                    .map_err(|e| actix_web::error::ErrorBadRequest(e))
            })
            .collect::<Result<Vec<_>>>()?;
        
        Ok(results)
    }
}

/// Connection pool optimization
pub mod connection_pool {
    use std::sync::Arc;
    use tokio::sync::Semaphore;
    
    /// Rate limiter to prevent overload
    pub struct RateLimiter {
        semaphore: Arc<Semaphore>,
        requests_per_sec: u32,
    }
    
    impl RateLimiter {
        pub fn new(requests_per_sec: u32) -> Self {
            Self {
                semaphore: Arc::new(Semaphore::new(requests_per_sec as usize)),
                requests_per_sec,
            }
        }
        
        pub async fn acquire(&self) {
            let _permit = self.semaphore.acquire().await.unwrap();
            // Hold permit for 1/rps seconds
            tokio::time::sleep(
                std::time::Duration::from_millis(1000 / self.requests_per_sec as u64)
            ).await;
        }
    }
}

/// Middleware for request batching
pub mod batch_middleware {
    use actix_web::{dev::ServiceRequest, Error};
    use std::collections::VecDeque;
    use tokio::sync::Mutex;
    use std::sync::Arc;
    
    pub struct BatchCollector<T> {
        queue: Arc<Mutex<VecDeque<T>>>,
        batch_size: usize,
        timeout_ms: u64,
    }
    
    impl<T: Send + 'static> BatchCollector<T> {
        pub fn new(batch_size: usize, timeout_ms: u64) -> Self {
            Self {
                queue: Arc::new(Mutex::new(VecDeque::new())),
                batch_size,
                timeout_ms,
            }
        }
        
        pub async fn add(&self, item: T) -> Option<Vec<T>> {
            let mut queue = self.queue.lock().await;
            queue.push_back(item);
            
            if queue.len() >= self.batch_size {
                let batch: Vec<T> = queue.drain(..).collect();
                Some(batch)
            } else {
                None
            }
        }
        
        pub async fn flush(&self) -> Vec<T> {
            let mut queue = self.queue.lock().await;
            queue.drain(..).collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_optimized_config() {
        let config = OptimizationConfig::default();
        assert!(config.worker_threads > 0);
        assert_eq!(config.worker_threads, num_cpus::get() * 2);
    }
    
    #[actix_web::test]
    async fn test_rate_limiter() {
        let limiter = connection_pool::RateLimiter::new(100);
        let start = std::time::Instant::now();
        
        for _ in 0..10 {
            limiter.acquire().await;
        }
        
        let elapsed = start.elapsed();
        assert!(elapsed.as_millis() >= 90); // Should take ~100ms for 10 requests at 100/sec
    }
}
