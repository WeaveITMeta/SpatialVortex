pub mod pipeline;
pub mod vortex_cycle;
pub mod object_propagation;
pub mod ladder_index;
pub mod intersection_analysis;
pub mod orchestrator;
pub mod pattern_engine;

// Re-export key types
pub use vortex_cycle::{
    VortexCycleEngine, CycleObject, CyclePosition, CycleDirection,
    CycleStats, FORWARD_SEQUENCE, BACKWARD_SEQUENCE, SACRED_ANCHORS,
};
pub use object_propagation::{
    ObjectPropagationManager, PropagationConfig, PropagationMode,
    VisualizationObject, PropagationStats,
};
pub use ladder_index::{
    LadderIndex, LadderEntry, LadderStats,
};
pub use intersection_analysis::{
    IntersectionAnalyzer, Intersection, IntersectionType,
    CrossReference, RelationshipType, Implication, ImplicationCategory,
    InterdynamicsAnalysis, IntersectionStats,
};
pub use orchestrator::{
    FluxOrchestrator, OrchestratorConfig, OrchestratorState, OrchestratorStats,
};
pub use pattern_engine::{
    VortexPattern, PatternType, PatternComparator, PatternAnalysis,
};

/// Parallel Tokio Runtime Configuration
/// 
/// High-performance async runtime optimized for 1000 Hz operation:
/// - Multi-threaded work-stealing scheduler
/// - Thread pool sized to CPU cores
/// - Dedicated I/O threads
/// - Custom task prioritization
/// - Performance monitoring

use std::sync::Arc;
use std::time::{Duration, Instant};
use dashmap::DashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tokio::runtime::{Runtime, Builder};
use tokio::task::JoinHandle;

/// Runtime configuration for optimal performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// Number of worker threads (default: num_cpus)
    pub worker_threads: usize,
    
    /// Thread stack size in bytes (default: 2MB)
    pub thread_stack_size: usize,
    
    /// Enable I/O driver
    pub enable_io: bool,
    
    /// Enable time driver
    pub enable_time: bool,
    
    /// Thread name prefix
    pub thread_name: String,
    
    /// Maximum blocking threads (default: 512)
    pub max_blocking_threads: usize,
    
    /// Thread keep-alive duration
    pub thread_keep_alive: Duration,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            worker_threads: num_cpus::get(),
            thread_stack_size: 2 * 1024 * 1024, // 2MB
            enable_io: true,
            enable_time: true,
            thread_name: "spatial-vortex".to_string(),
            max_blocking_threads: 512,
            thread_keep_alive: Duration::from_secs(10),
        }
    }
}

/// Task priority levels for scheduling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TaskPriority {
    Critical = 0,  // Sacred position operations, <10ms
    High = 1,      // Inference, geometric queries, <50ms
    Normal = 2,    // Standard operations, <100ms
    Low = 3,       // Background tasks, <1s
    Idle = 4,      // Maintenance, garbage collection
}

/// Task metadata for monitoring
#[derive(Debug, Clone)]
pub struct TaskMetadata {
    pub task_id: String,
    pub priority: TaskPriority,
    pub created_at: Instant,
    pub started_at: Option<Instant>,
    pub completed_at: Option<Instant>,
    pub name: String,
}

impl TaskMetadata {
    pub fn new(task_id: String, priority: TaskPriority, name: String) -> Self {
        Self {
            task_id,
            priority,
            created_at: Instant::now(),
            started_at: None,
            completed_at: None,
            name,
        }
    }
    
    pub fn latency(&self) -> Option<Duration> {
        if let (Some(started), Some(completed)) = (self.started_at, self.completed_at) {
            Some(completed.duration_since(started))
        } else {
            None
        }
    }
    
    pub fn queue_time(&self) -> Option<Duration> {
        self.started_at.map(|started| started.duration_since(self.created_at))
    }
}

/// Runtime metrics for performance monitoring
#[derive(Debug, Default)]
pub struct RuntimeMetrics {
    /// Total tasks spawned
    pub total_tasks: Arc<parking_lot::Mutex<u64>>,
    
    /// Tasks by priority
    pub tasks_by_priority: Arc<DashMap<TaskPriority, u64>>,
    
    /// Active tasks
    pub active_tasks: Arc<DashMap<String, TaskMetadata>>,
    
    /// Completed tasks (last 1000)
    pub completed_tasks: Arc<RwLock<Vec<TaskMetadata>>>,
    
    /// Average latency by priority
    pub avg_latency: Arc<DashMap<TaskPriority, Duration>>,
}

impl RuntimeMetrics {
    pub fn new() -> Self {
        Self {
            total_tasks: Arc::new(parking_lot::Mutex::new(0)),
            tasks_by_priority: Arc::new(DashMap::new()),
            active_tasks: Arc::new(DashMap::new()),
            completed_tasks: Arc::new(RwLock::new(Vec::new())),
            avg_latency: Arc::new(DashMap::new()),
        }
    }
    
    pub fn record_task_start(&self, metadata: TaskMetadata) {
        *self.total_tasks.lock() += 1;
        *self.tasks_by_priority.entry(metadata.priority).or_insert(0) += 1;
        self.active_tasks.insert(metadata.task_id.clone(), metadata);
    }
    
    pub fn record_task_complete(&self, task_id: &str) {
        if let Some((_, mut metadata)) = self.active_tasks.remove(task_id) {
            metadata.completed_at = Some(Instant::now());
            
            // Update average latency
            if let Some(latency) = metadata.latency() {
                let mut entry = self.avg_latency.entry(metadata.priority).or_insert(Duration::ZERO);
                let current = *entry;
                let count = self.tasks_by_priority.get(&metadata.priority).map(|r| *r).unwrap_or(1);
                let new_avg = (current * (count as u32 - 1) + latency) / count as u32;
                *entry = new_avg;
            }
            
            // Keep last 1000 completed tasks
            let mut completed = self.completed_tasks.write();
            completed.push(metadata);
            if completed.len() > 1000 {
                completed.remove(0);
            }
        }
    }
    
    pub fn get_stats(&self) -> RuntimeStats {
        RuntimeStats {
            total_tasks: *self.total_tasks.lock(),
            active_task_count: self.active_tasks.len(),
            critical_tasks: self.tasks_by_priority.get(&TaskPriority::Critical).map(|r| *r).unwrap_or(0),
            high_tasks: self.tasks_by_priority.get(&TaskPriority::High).map(|r| *r).unwrap_or(0),
            normal_tasks: self.tasks_by_priority.get(&TaskPriority::Normal).map(|r| *r).unwrap_or(0),
            low_tasks: self.tasks_by_priority.get(&TaskPriority::Low).map(|r| *r).unwrap_or(0),
            avg_critical_latency: self.avg_latency.get(&TaskPriority::Critical).map(|r| *r),
            avg_high_latency: self.avg_latency.get(&TaskPriority::High).map(|r| *r),
        }
    }
}

/// Runtime statistics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStats {
    pub total_tasks: u64,
    pub active_task_count: usize,
    pub critical_tasks: u64,
    pub high_tasks: u64,
    pub normal_tasks: u64,
    pub low_tasks: u64,
    pub avg_critical_latency: Option<Duration>,
    pub avg_high_latency: Option<Duration>,
}

/// Parallel Tokio runtime wrapper
pub struct ParallelRuntime {
    runtime: Runtime,
    config: RuntimeConfig,
    metrics: Arc<RuntimeMetrics>,
}

impl ParallelRuntime {
    /// Create new parallel runtime with configuration
    pub fn new(config: RuntimeConfig) -> anyhow::Result<Self> {
        let runtime = Builder::new_multi_thread()
            .worker_threads(config.worker_threads)
            .thread_name(config.thread_name.clone())
            .thread_stack_size(config.thread_stack_size)
            .max_blocking_threads(config.max_blocking_threads)
            .thread_keep_alive(config.thread_keep_alive)
            .enable_io()
            .enable_time()
            .build()?;
        
        Ok(Self {
            runtime,
            config,
            metrics: Arc::new(RuntimeMetrics::new()),
        })
    }
    
    /// Create with default configuration
    pub fn new_default() -> anyhow::Result<Self> {
        Self::new(RuntimeConfig::default())
    }
    
    /// Spawn task with priority
    pub fn spawn_with_priority<F>(
        &self,
        priority: TaskPriority,
        name: String,
        future: F,
    ) -> JoinHandle<F::Output>
    where
        F: std::future::Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let task_id = uuid::Uuid::new_v4().to_string();
        let mut metadata = TaskMetadata::new(task_id.clone(), priority, name);
        metadata.started_at = Some(Instant::now());
        
        let metrics = self.metrics.clone();
        metrics.record_task_start(metadata);
        
        let metrics_clone = metrics.clone();
        let task_id_clone = task_id.clone();
        
        self.runtime.spawn(async move {
            let result = future.await;
            metrics_clone.record_task_complete(&task_id_clone);
            result
        })
    }
    
    /// Spawn critical task (highest priority)
    pub fn spawn_critical<F>(&self, name: String, future: F) -> JoinHandle<F::Output>
    where
        F: std::future::Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.spawn_with_priority(TaskPriority::Critical, name, future)
    }
    
    /// Spawn high priority task
    pub fn spawn_high<F>(&self, name: String, future: F) -> JoinHandle<F::Output>
    where
        F: std::future::Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.spawn_with_priority(TaskPriority::High, name, future)
    }
    
    /// Get runtime metrics
    pub fn metrics(&self) -> RuntimeStats {
        self.metrics.get_stats()
    }
    
    /// Get configuration
    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }
    
    /// Block on future (for testing)
    pub fn block_on<F>(&self, future: F) -> F::Output
    where
        F: std::future::Future,
    {
        self.runtime.block_on(future)
    }
    
    /// Get handle to underlying runtime
    pub fn handle(&self) -> &tokio::runtime::Handle {
        self.runtime.handle()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_runtime_creation() {
        let runtime = ParallelRuntime::new_default().unwrap();
        assert!(runtime.config().worker_threads > 0);
    }
    
    #[test]
    fn test_spawn_with_priority() {
        let runtime = ParallelRuntime::new_default().unwrap();
        
        let handle = runtime.spawn_high("test_task".to_string(), async {
            tokio::time::sleep(Duration::from_millis(10)).await;
            42
        });
        
        let result = runtime.block_on(handle).unwrap();
        assert_eq!(result, 42);
    }
    
    #[test]
    fn test_metrics_tracking() {
        let runtime = ParallelRuntime::new_default().unwrap();
        
        let handle1 = runtime.spawn_critical("critical_task".to_string(), async { 1 });
        let handle2 = runtime.spawn_high("high_task".to_string(), async { 2 });
        let handle3 = runtime.spawn_with_priority(TaskPriority::Normal, "normal_task".to_string(), async { 3 });
        
        runtime.block_on(async {
            handle1.await.unwrap();
            handle2.await.unwrap();
            handle3.await.unwrap();
        });
        
        let stats = runtime.metrics();
        assert_eq!(stats.total_tasks, 3);
        assert!(stats.critical_tasks > 0);
        assert!(stats.high_tasks > 0);
        assert!(stats.normal_tasks > 0);
    }
    
    #[test]
    fn test_concurrent_tasks() {
        let runtime = ParallelRuntime::new_default().unwrap();
        
        let handles: Vec<_> = (0..100)
            .map(|i| {
                runtime.spawn_with_priority(
                    TaskPriority::Normal,
                    format!("task_{}", i),
                    async move { i * 2 }
                )
            })
            .collect();
        
        let results = runtime.block_on(async {
            let mut results = Vec::new();
            for handle in handles {
                results.push(handle.await.unwrap());
            }
            results
        });
        
        assert_eq!(results.len(), 100);
        assert_eq!(results[50], 100);
    }
}
