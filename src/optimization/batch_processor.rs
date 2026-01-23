//! Batch Processing Optimization
//!
//! Batches requests across all layers for maximum throughput
//! Target: 10x throughput improvement via batching

use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use std::time::{Duration, Instant};
use std::collections::VecDeque;

/// Generic batch processor for any request type
pub struct BatchProcessor<T, R> {
    batch_size: usize,
    timeout: Duration,
    pending: Arc<RwLock<VecDeque<PendingItem<T, R>>>>,
    processor: Arc<dyn BatchHandler<T, R> + Send + Sync>,
}

struct PendingItem<T, R> {
    request: T,
    response_tx: tokio::sync::oneshot::Sender<Result<R, String>>,
    submitted_at: Instant,
}

/// Trait for batch processing handlers
pub trait BatchHandler<T, R>: Send + Sync {
    fn process_batch(&self, batch: Vec<T>) -> Vec<Result<R, String>>;
}

impl<T, R> BatchProcessor<T, R>
where
    T: Send + 'static,
    R: Send + 'static,
{
    pub fn new(
        batch_size: usize,
        timeout_ms: u64,
        processor: Arc<dyn BatchHandler<T, R> + Send + Sync>,
    ) -> Self {
        let pending = Arc::new(RwLock::new(VecDeque::new()));
        
        let batch_processor = Self {
            batch_size,
            timeout: Duration::from_millis(timeout_ms),
            pending: pending.clone(),
            processor: processor.clone(),
        };
        
        // Start background processor
        let pending_clone = pending.clone();
        let timeout = batch_processor.timeout;
        let batch_size = batch_processor.batch_size;
        
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(timeout).await;
                
                let batch = {
                    let mut pending = pending_clone.write().await;
                    
                    if pending.is_empty() {
                        continue;
                    }
                    
                    // Take up to batch_size items
                    let take_count = pending.len().min(batch_size);
                    pending.drain(..take_count).collect::<Vec<_>>()
                };
                
                if !batch.is_empty() {
                    Self::process_batch_internal(batch, processor.as_ref());
                }
            }
        });
        
        batch_processor
    }
    
    /// Submit request for batch processing
    pub async fn submit(&self, request: T) -> Result<R, String> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        
        let item = PendingItem {
            request,
            response_tx: tx,
            submitted_at: Instant::now(),
        };
        
        // Check if batch should trigger immediately
        let should_process = {
            let mut pending = self.pending.write().await;
            pending.push_back(item);
            pending.len() >= self.batch_size
        };
        
        if should_process {
            let batch = {
                let mut pending = self.pending.write().await;
                pending.drain(..self.batch_size).collect::<Vec<_>>()
            };
            
            Self::process_batch_internal(batch, self.processor.as_ref());
        }
        
        // Wait for response
        rx.await.map_err(|_| "Batch processor error".to_string())
    }
    
    fn process_batch_internal(
        batch: Vec<PendingItem<T, R>>,
        processor: &dyn BatchHandler<T, R>,
    ) {
        let requests: Vec<T> = batch.iter()
            .map(|item| item.request.clone())
            .collect();
        
        let results = processor.process_batch(requests);
        
        // Send results back
        for (item, result) in batch.into_iter().zip(results) {
            let _ = item.response_tx.send(result);
        }
    }
    
    /// Get pending count
    pub async fn pending_count(&self) -> usize {
        self.pending.read().await.len()
    }
}

/// Batch processor for API requests
pub struct ApiBatchProcessor;

impl BatchHandler<serde_json::Value, serde_json::Value> for ApiBatchProcessor {
    fn process_batch(&self, batch: Vec<serde_json::Value>) -> Vec<Result<serde_json::Value, String>> {
        // Process API requests in batch
        batch.into_iter()
            .map(|request| {
                // Simulate processing
                Ok(serde_json::json!({
                    "processed": true,
                    "request": request
                }))
            })
            .collect()
    }
}

/// Batch processor for voice pipeline
pub struct VoiceBatchProcessor {
    fft_size: usize,
}

impl VoiceBatchProcessor {
    pub fn new(fft_size: usize) -> Self {
        Self { fft_size }
    }
}

impl BatchHandler<Vec<f32>, String> for VoiceBatchProcessor {
    fn process_batch(&self, batch: Vec<Vec<f32>>) -> Vec<Result<String, String>> {
        use rayon::prelude::*;
        
        // Process audio samples in parallel
        batch.into_par_iter()
            .map(|audio| {
                // Simulate STT processing
                Ok(format!("Transcribed {} samples", audio.len()))
            })
            .collect()
    }
}

// Database batch processor requires sqlx feature
#[cfg(feature = "lake")]
pub struct DbBatchProcessor {
    pool: Arc<sqlx::SqlitePool>,
}

#[cfg(feature = "lake")]
impl DbBatchProcessor {
    pub fn new(pool: Arc<sqlx::SqlitePool>) -> Self {
        Self { pool }
    }
}

#[cfg(feature = "lake")]
impl BatchHandler<DbOperation, DbResult> for DbBatchProcessor {
    fn process_batch(&self, batch: Vec<DbOperation>) -> Vec<Result<DbResult, String>> {
        let mut inserts = Vec::new();
        let mut selects = Vec::new();
        let mut updates = Vec::new();
        
        for op in batch {
            match op {
                DbOperation::Insert(data) => inserts.push(data),
                DbOperation::Select(query) => selects.push(query),
                DbOperation::Update(data) => updates.push(data),
            }
        }
        
        let mut results = Vec::new();
        
        if !inserts.is_empty() {
            for _data in inserts {
                results.push(Ok(DbResult::Inserted(1)));
            }
        }
        
        if !selects.is_empty() {
            for _query in selects {
                results.push(Ok(DbResult::Selected(vec![serde_json::json!({})])));
            }
        }
        
        if !updates.is_empty() {
            for _data in updates {
                results.push(Ok(DbResult::Updated(1)));
            }
        }
        
        results
    }
}

#[cfg(feature = "lake")]
#[derive(Clone)]
pub enum DbOperation {
    Insert(serde_json::Value),
    Select(String),
    Update(serde_json::Value),
}

#[cfg(feature = "lake")]
#[derive(Debug)]
pub enum DbResult {
    Inserted(u64),
    Selected(Vec<serde_json::Value>),
    Updated(u64),
}

/// Adaptive batch size optimizer
pub struct AdaptiveBatchOptimizer {
    min_batch: usize,
    max_batch: usize,
    current_batch: Arc<RwLock<usize>>,
    latency_target_ms: u64,
    recent_latencies: Arc<RwLock<VecDeque<u64>>>,
}

impl AdaptiveBatchOptimizer {
    pub fn new(min: usize, max: usize, target_ms: u64) -> Self {
        Self {
            min_batch: min,
            max_batch: max,
            current_batch: Arc::new(RwLock::new(min)),
            latency_target_ms: target_ms,
            recent_latencies: Arc::new(RwLock::new(VecDeque::with_capacity(100))),
        }
    }
    
    /// Record latency and adjust batch size
    pub async fn record_latency(&self, latency_ms: u64) {
        let mut latencies = self.recent_latencies.write().await;
        
        if latencies.len() >= 100 {
            latencies.pop_front();
        }
        latencies.push_back(latency_ms);
        
        // Calculate average
        let avg_latency = latencies.iter().sum::<u64>() / latencies.len() as u64;
        
        // Adjust batch size
        let mut current = self.current_batch.write().await;
        
        if avg_latency > self.latency_target_ms && *current > self.min_batch {
            // Reduce batch size
            *current = (*current * 9 / 10).max(self.min_batch);
        } else if avg_latency < self.latency_target_ms * 8 / 10 && *current < self.max_batch {
            // Increase batch size
            *current = (*current * 11 / 10).min(self.max_batch);
        }
    }
    
    /// Get current optimal batch size
    pub async fn get_batch_size(&self) -> usize {
        *self.current_batch.read().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    struct TestProcessor;
    
    impl BatchHandler<i32, i32> for TestProcessor {
        fn process_batch(&self, batch: Vec<i32>) -> Vec<Result<i32, String>> {
            batch.into_iter().map(|x| Ok(x * 2)).collect()
        }
    }
    
    #[tokio::test]
    async fn test_batch_processor() {
        let processor = Arc::new(TestProcessor);
        let batch_processor = BatchProcessor::new(3, 100, processor);
        
        // Submit requests
        let mut handles = Vec::new();
        for i in 0..3 {
            let bp = batch_processor.clone();
            handles.push(tokio::spawn(async move {
                bp.submit(i).await
            }));
        }
        
        // Wait for results
        let results: Vec<_> = futures::future::join_all(handles).await;
        
        for (i, result) in results.into_iter().enumerate() {
            let value = result.unwrap().unwrap();
            assert_eq!(value, i as i32 * 2);
        }
    }
    
    #[tokio::test]
    async fn test_adaptive_optimizer() {
        let optimizer = AdaptiveBatchOptimizer::new(10, 100, 50);
        
        // Record fast latencies
        for _ in 0..10 {
            optimizer.record_latency(30).await;
        }
        
        // Batch size should increase
        let size1 = optimizer.get_batch_size().await;
        assert!(size1 > 10);
        
        // Record slow latencies
        for _ in 0..10 {
            optimizer.record_latency(100).await;
        }
        
        // Batch size should decrease
        let size2 = optimizer.get_batch_size().await;
        assert!(size2 < size1);
    }
}
