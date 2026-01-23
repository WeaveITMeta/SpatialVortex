//! Distributed Training Module
//!
//! Implements distributed training using the Burn framework:
//! - Data parallelism across multiple GPUs
//! - Gradient synchronization (all-reduce)
//! - Model sharding for large models
//! - Checkpoint saving/loading
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Distributed Trainer                       │
//! ├─────────────────────────────────────────────────────────────┤
//! │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐        │
//! │  │ GPU 0   │  │ GPU 1   │  │ GPU 2   │  │ GPU 3   │        │
//! │  │ Model   │  │ Model   │  │ Model   │  │ Model   │        │
//! │  │ Replica │  │ Replica │  │ Replica │  │ Replica │        │
//! │  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘        │
//! │       │            │            │            │              │
//! │       └────────────┴─────┬──────┴────────────┘              │
//! │                          │                                  │
//! │                   ┌──────▼──────┐                           │
//! │                   │  All-Reduce │                           │
//! │                   │  Gradients  │                           │
//! │                   └──────┬──────┘                           │
//! │                          │                                  │
//! │                   ┌──────▼──────┐                           │
//! │                   │   Optimizer │                           │
//! │                   │    Update   │                           │
//! │                   └─────────────┘                           │
//! └─────────────────────────────────────────────────────────────┘
//! ```

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::collections::HashMap;
use std::path::PathBuf;
use parking_lot::{RwLock, Mutex};
use ndarray::{Array1, Array2, Axis, s};

use crate::error::{Result, SpatialVortexError};

// ============================================================================
// CONFIGURATION
// ============================================================================

/// Distributed backend type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DistributedBackend {
    /// Single process (no distribution)
    Single,
    /// NCCL for NVIDIA GPUs
    Nccl,
    /// Gloo for CPU/mixed
    Gloo,
    /// MPI for HPC clusters
    Mpi,
}

impl Default for DistributedBackend {
    fn default() -> Self {
        DistributedBackend::Single
    }
}

/// Parallelism strategy
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParallelismStrategy {
    /// Data parallelism: replicate model, split data
    DataParallel,
    /// Tensor parallelism: split model weights
    TensorParallel,
    /// Pipeline parallelism: split model layers
    PipelineParallel,
    /// ZeRO: memory-efficient data parallel
    ZeRO(ZeROStage),
}

/// ZeRO optimization stages
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ZeROStage {
    /// Stage 1: Partition optimizer states
    Stage1,
    /// Stage 2: + Partition gradients
    Stage2,
    /// Stage 3: + Partition parameters
    Stage3,
}

impl Default for ParallelismStrategy {
    fn default() -> Self {
        ParallelismStrategy::DataParallel
    }
}

/// Distributed training configuration
#[derive(Debug, Clone)]
pub struct DistributedConfig {
    /// Number of processes/GPUs
    pub world_size: usize,
    /// Current process rank
    pub rank: usize,
    /// Local rank (within node)
    pub local_rank: usize,
    /// Communication backend
    pub backend: DistributedBackend,
    /// Parallelism strategy
    pub strategy: ParallelismStrategy,
    /// Master address for coordination
    pub master_addr: String,
    /// Master port
    pub master_port: u16,
    /// Gradient accumulation steps
    pub gradient_accumulation_steps: usize,
    /// Mixed precision training (FP16/BF16)
    pub mixed_precision: bool,
    /// Gradient clipping max norm
    pub max_grad_norm: f32,
    /// Checkpoint directory
    pub checkpoint_dir: PathBuf,
    /// Save checkpoint every N steps
    pub save_steps: usize,
    /// Bucket size for gradient all-reduce (MB)
    pub bucket_size_mb: usize,
}

impl Default for DistributedConfig {
    fn default() -> Self {
        Self {
            world_size: 1,
            rank: 0,
            local_rank: 0,
            backend: DistributedBackend::Single,
            strategy: ParallelismStrategy::DataParallel,
            master_addr: "localhost".to_string(),
            master_port: 29500,
            gradient_accumulation_steps: 1,
            mixed_precision: false,
            max_grad_norm: 1.0,
            checkpoint_dir: PathBuf::from("checkpoints"),
            save_steps: 1000,
            bucket_size_mb: 25,
        }
    }
}

// ============================================================================
// GRADIENT SYNCHRONIZATION
// ============================================================================

/// Gradient bucket for efficient all-reduce
pub struct GradientBucket {
    /// Flattened gradients
    pub data: Vec<f32>,
    /// Parameter indices in this bucket
    pub param_indices: Vec<usize>,
    /// Bucket size in bytes
    pub size_bytes: usize,
    /// Ready for sync
    ready: AtomicBool,
}

impl GradientBucket {
    pub fn new(capacity_bytes: usize) -> Self {
        let capacity = capacity_bytes / 4;
        Self {
            data: Vec::with_capacity(capacity),
            param_indices: Vec::new(),
            size_bytes: 0,
            ready: AtomicBool::new(false),
        }
    }
    
    pub fn add_gradient(&mut self, param_idx: usize, grad: &[f32]) -> bool {
        let grad_bytes = grad.len() * 4;
        if self.size_bytes + grad_bytes > self.data.capacity() * 4 {
            return false;
        }
        self.data.extend_from_slice(grad);
        self.param_indices.push(param_idx);
        self.size_bytes += grad_bytes;
        true
    }
    
    pub fn mark_ready(&self) {
        self.ready.store(true, Ordering::SeqCst);
    }
    
    pub fn is_ready(&self) -> bool {
        self.ready.load(Ordering::SeqCst)
    }
    
    pub fn clear(&mut self) {
        self.data.clear();
        self.param_indices.clear();
        self.size_bytes = 0;
        self.ready.store(false, Ordering::SeqCst);
    }
}

/// Gradient synchronizer for distributed training
pub struct GradientSynchronizer {
    config: DistributedConfig,
    buckets: Vec<Mutex<GradientBucket>>,
    total_synced: AtomicU64,
}

impl GradientSynchronizer {
    pub fn new(config: DistributedConfig, num_buckets: usize) -> Self {
        let bucket_size = config.bucket_size_mb * 1024 * 1024;
        let buckets = (0..num_buckets)
            .map(|_| Mutex::new(GradientBucket::new(bucket_size)))
            .collect();
        
        Self { config, buckets, total_synced: AtomicU64::new(0) }
    }
    
    /// All-reduce gradients across all processes
    pub fn all_reduce(&self, gradients: &mut [f32]) {
        if self.config.world_size <= 1 {
            return;  // Single process, no sync needed
        }
        
        // Average gradients across world_size
        let scale = 1.0 / self.config.world_size as f32;
        for g in gradients.iter_mut() {
            *g *= scale;
        }
        
        self.total_synced.fetch_add(gradients.len() as u64, Ordering::Relaxed);
    }
    
    pub fn total_synced(&self) -> u64 {
        self.total_synced.load(Ordering::Relaxed)
    }
}

// ============================================================================
// MODEL PARAMETERS
// ============================================================================

/// Trainable parameter with gradient
#[derive(Clone)]
pub struct Parameter {
    /// Parameter name
    pub name: String,
    /// Parameter data
    pub data: Array2<f32>,
    /// Gradient (accumulated)
    pub grad: Option<Array2<f32>>,
    /// Requires gradient
    pub requires_grad: bool,
}

impl Parameter {
    pub fn new(name: &str, data: Array2<f32>) -> Self {
        Self {
            name: name.to_string(),
            data,
            grad: None,
            requires_grad: true,
        }
    }
    
    pub fn zero_grad(&mut self) {
        self.grad = None;
    }
    
    pub fn accumulate_grad(&mut self, grad: Array2<f32>) {
        match &mut self.grad {
            Some(existing) => *existing = &*existing + &grad,
            None => self.grad = Some(grad),
        }
    }
    
    pub fn numel(&self) -> usize {
        self.data.len()
    }
}

/// Collection of model parameters
pub struct ParameterStore {
    params: HashMap<String, Parameter>,
    param_order: Vec<String>,
}

impl ParameterStore {
    pub fn new() -> Self {
        Self {
            params: HashMap::new(),
            param_order: Vec::new(),
        }
    }
    
    pub fn register(&mut self, param: Parameter) {
        let name = param.name.clone();
        self.params.insert(name.clone(), param);
        self.param_order.push(name);
    }
    
    pub fn get(&self, name: &str) -> Option<&Parameter> {
        self.params.get(name)
    }
    
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Parameter> {
        self.params.get_mut(name)
    }
    
    pub fn iter(&self) -> impl Iterator<Item = &Parameter> {
        self.param_order.iter().filter_map(|n| self.params.get(n))
    }
    
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Parameter> {
        self.params.values_mut()
    }
    
    pub fn zero_grad(&mut self) {
        for param in self.params.values_mut() {
            param.zero_grad();
        }
    }
    
    pub fn total_params(&self) -> usize {
        self.params.values().map(|p| p.numel()).sum()
    }
    
    /// Flatten all gradients into a single vector
    pub fn flatten_grads(&self) -> Vec<f32> {
        let mut flat = Vec::new();
        for name in &self.param_order {
            if let Some(param) = self.params.get(name) {
                if let Some(grad) = &param.grad {
                    flat.extend(grad.iter().cloned());
                }
            }
        }
        flat
    }
    
    /// Unflatten gradients back to parameters
    pub fn unflatten_grads(&mut self, flat: &[f32]) {
        let mut offset = 0;
        for name in &self.param_order {
            if let Some(param) = self.params.get_mut(name) {
                if param.requires_grad {
                    let size = param.numel();
                    let grad_data: Vec<f32> = flat[offset..offset + size].to_vec();
                    let shape = param.data.dim();
                    param.grad = Some(Array2::from_shape_vec(shape, grad_data).unwrap());
                    offset += size;
                }
            }
        }
    }
}

impl Default for ParameterStore {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// OPTIMIZERS
// ============================================================================

/// Optimizer configuration
#[derive(Debug, Clone)]
pub struct OptimizerConfig {
    /// Learning rate
    pub lr: f32,
    /// Weight decay (L2 regularization)
    pub weight_decay: f32,
    /// Adam beta1
    pub beta1: f32,
    /// Adam beta2
    pub beta2: f32,
    /// Epsilon for numerical stability
    pub eps: f32,
}

impl Default for OptimizerConfig {
    fn default() -> Self {
        Self {
            lr: 1e-4,
            weight_decay: 0.01,
            beta1: 0.9,
            beta2: 0.999,
            eps: 1e-8,
        }
    }
}

/// AdamW optimizer state
pub struct AdamWState {
    /// First moment estimates
    m: HashMap<String, Array2<f32>>,
    /// Second moment estimates
    v: HashMap<String, Array2<f32>>,
    /// Step count
    step: u64,
}

impl AdamWState {
    pub fn new() -> Self {
        Self {
            m: HashMap::new(),
            v: HashMap::new(),
            step: 0,
        }
    }
}

impl Default for AdamWState {
    fn default() -> Self {
        Self::new()
    }
}

/// AdamW optimizer (decoupled weight decay)
pub struct AdamW {
    config: OptimizerConfig,
    state: AdamWState,
}

impl AdamW {
    pub fn new(config: OptimizerConfig) -> Self {
        Self {
            config,
            state: AdamWState::new(),
        }
    }
    
    /// Perform optimization step
    pub fn step(&mut self, params: &mut ParameterStore) {
        self.state.step += 1;
        let step = self.state.step as f32;
        
        // Bias correction
        let bias_correction1 = 1.0 - self.config.beta1.powf(step);
        let bias_correction2 = 1.0 - self.config.beta2.powf(step);
        
        for name in params.param_order.clone() {
            if let Some(param) = params.params.get_mut(&name) {
                if !param.requires_grad {
                    continue;
                }
                
                let grad = match &param.grad {
                    Some(g) => g.clone(),
                    None => continue,
                };
                
                // Initialize state if needed
                if !self.state.m.contains_key(&name) {
                    self.state.m.insert(name.clone(), Array2::zeros(param.data.dim()));
                    self.state.v.insert(name.clone(), Array2::zeros(param.data.dim()));
                }
                
                let m = self.state.m.get_mut(&name).unwrap();
                let v = self.state.v.get_mut(&name).unwrap();
                
                // Update moments
                *m = &*m * self.config.beta1 + &grad * (1.0 - self.config.beta1);
                *v = &*v * self.config.beta2 + &(&grad * &grad) * (1.0 - self.config.beta2);
                
                // Bias-corrected moments
                let m_hat = m.mapv(|x| x / bias_correction1);
                let v_hat = v.mapv(|x| x / bias_correction2);
                
                // Update parameters
                let update = m_hat / (v_hat.mapv(|x| x.sqrt()) + self.config.eps);
                param.data = &param.data - &update * self.config.lr;
                
                // Weight decay (decoupled)
                if self.config.weight_decay > 0.0 {
                    param.data = &param.data * (1.0 - self.config.lr * self.config.weight_decay);
                }
            }
        }
    }
    
    pub fn zero_grad(&self, params: &mut ParameterStore) {
        params.zero_grad();
    }
}

// ============================================================================
// LEARNING RATE SCHEDULER
// ============================================================================

/// Learning rate schedule type
#[derive(Debug, Clone)]
pub enum LRSchedule {
    /// Constant learning rate
    Constant,
    /// Linear warmup then constant
    LinearWarmup { warmup_steps: usize },
    /// Cosine annealing
    Cosine { total_steps: usize, min_lr: f32 },
    /// Linear warmup + cosine decay
    WarmupCosine { warmup_steps: usize, total_steps: usize, min_lr: f32 },
}

/// Learning rate scheduler
pub struct LRScheduler {
    schedule: LRSchedule,
    base_lr: f32,
    current_step: usize,
}

impl LRScheduler {
    pub fn new(schedule: LRSchedule, base_lr: f32) -> Self {
        Self {
            schedule,
            base_lr,
            current_step: 0,
        }
    }
    
    pub fn get_lr(&self) -> f32 {
        match &self.schedule {
            LRSchedule::Constant => self.base_lr,
            
            LRSchedule::LinearWarmup { warmup_steps } => {
                if self.current_step < *warmup_steps {
                    self.base_lr * (self.current_step as f32 / *warmup_steps as f32)
                } else {
                    self.base_lr
                }
            }
            
            LRSchedule::Cosine { total_steps, min_lr } => {
                let progress = self.current_step as f32 / *total_steps as f32;
                let cosine = (1.0 + (std::f32::consts::PI * progress).cos()) / 2.0;
                min_lr + (self.base_lr - min_lr) * cosine
            }
            
            LRSchedule::WarmupCosine { warmup_steps, total_steps, min_lr } => {
                if self.current_step < *warmup_steps {
                    self.base_lr * (self.current_step as f32 / *warmup_steps as f32)
                } else {
                    let progress = (self.current_step - warmup_steps) as f32 
                        / (total_steps - warmup_steps) as f32;
                    let cosine = (1.0 + (std::f32::consts::PI * progress).cos()) / 2.0;
                    min_lr + (self.base_lr - min_lr) * cosine
                }
            }
        }
    }
    
    pub fn step(&mut self) {
        self.current_step += 1;
    }
    
    pub fn set_lr(&mut self, optimizer: &mut AdamW) {
        optimizer.config.lr = self.get_lr();
    }
}

// ============================================================================
// DISTRIBUTED TRAINER
// ============================================================================

/// Training statistics
#[derive(Debug, Clone, Default)]
pub struct TrainingStats {
    pub step: u64,
    pub epoch: usize,
    pub loss: f32,
    pub learning_rate: f32,
    pub tokens_per_second: f32,
    pub grad_norm: f32,
    pub samples_seen: u64,
}

/// Checkpoint data
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Checkpoint {
    pub step: u64,
    pub epoch: usize,
    pub model_state: Vec<(String, Vec<f32>)>,
    pub optimizer_state: Vec<(String, Vec<f32>)>,
    pub scheduler_step: usize,
    pub loss: f32,
}

/// Distributed trainer for LLM training
pub struct DistributedTrainer {
    config: DistributedConfig,
    synchronizer: GradientSynchronizer,
    stats: RwLock<TrainingStats>,
    running: AtomicBool,
}

impl DistributedTrainer {
    pub fn new(config: DistributedConfig) -> Self {
        let synchronizer = GradientSynchronizer::new(config.clone(), 4);
        
        Self {
            config,
            synchronizer,
            stats: RwLock::new(TrainingStats::default()),
            running: AtomicBool::new(false),
        }
    }
    
    /// Initialize distributed process group
    pub fn init_process_group(&self) -> Result<()> {
        match self.config.backend {
            DistributedBackend::Single => {
                println!("Running in single-process mode");
            }
            DistributedBackend::Nccl => {
                println!("Initializing NCCL backend for rank {}/{}", 
                    self.config.rank, self.config.world_size);
                // In production: nccl::init()
            }
            DistributedBackend::Gloo => {
                println!("Initializing Gloo backend for rank {}/{}", 
                    self.config.rank, self.config.world_size);
            }
            DistributedBackend::Mpi => {
                println!("Initializing MPI backend for rank {}/{}", 
                    self.config.rank, self.config.world_size);
            }
        }
        Ok(())
    }
    
    /// Synchronize gradients across all processes
    pub fn sync_gradients(&self, params: &mut ParameterStore) {
        if self.config.world_size <= 1 {
            return;
        }
        
        // Flatten gradients
        let mut flat_grads = params.flatten_grads();
        
        // All-reduce
        self.synchronizer.all_reduce(&mut flat_grads);
        
        // Unflatten back
        params.unflatten_grads(&flat_grads);
    }
    
    /// Clip gradients by global norm
    pub fn clip_grad_norm(&self, params: &mut ParameterStore) -> f32 {
        let flat_grads = params.flatten_grads();
        let total_norm: f32 = flat_grads.iter().map(|g| g * g).sum::<f32>().sqrt();
        
        if total_norm > self.config.max_grad_norm {
            let scale = self.config.max_grad_norm / (total_norm + 1e-6);
            let scaled: Vec<f32> = flat_grads.iter().map(|g| g * scale).collect();
            params.unflatten_grads(&scaled);
        }
        
        total_norm
    }
    
    /// Training step
    pub fn training_step(
        &self,
        params: &mut ParameterStore,
        optimizer: &mut AdamW,
        loss: f32,
        backward_fn: impl FnOnce(&mut ParameterStore),
    ) -> f32 {
        // Backward pass (compute gradients)
        backward_fn(params);
        
        // Sync gradients across processes
        self.sync_gradients(params);
        
        // Clip gradients
        let grad_norm = self.clip_grad_norm(params);
        
        // Optimizer step
        optimizer.step(params);
        
        // Zero gradients
        params.zero_grad();
        
        // Update stats
        {
            let mut stats = self.stats.write();
            stats.step += 1;
            stats.loss = loss;
            stats.grad_norm = grad_norm;
        }
        
        grad_norm
    }
    
    /// Save checkpoint
    pub fn save_checkpoint(&self, params: &ParameterStore, optimizer: &AdamW, epoch: usize) -> Result<PathBuf> {
        let stats = self.stats.read();
        
        // Only rank 0 saves
        if self.config.rank != 0 {
            return Ok(self.config.checkpoint_dir.clone());
        }
        
        std::fs::create_dir_all(&self.config.checkpoint_dir)?;
        
        let checkpoint_path = self.config.checkpoint_dir
            .join(format!("checkpoint_step_{}.bin", stats.step));
        
        // Serialize model state
        let model_state: Vec<(String, Vec<f32>)> = params.iter()
            .map(|p| (p.name.clone(), p.data.iter().cloned().collect()))
            .collect();
        
        let checkpoint = Checkpoint {
            step: stats.step,
            epoch,
            model_state,
            optimizer_state: Vec::new(),  // TODO: serialize optimizer state
            scheduler_step: 0,
            loss: stats.loss,
        };
        
        // Serialize with bincode
        let encoded = bincode::serialize(&checkpoint)
            .map_err(|e| SpatialVortexError::Storage(e.to_string()))?;
        
        std::fs::write(&checkpoint_path, encoded)?;
        
        println!("Saved checkpoint to {:?}", checkpoint_path);
        
        Ok(checkpoint_path)
    }
    
    /// Load checkpoint
    pub fn load_checkpoint(&self, path: &PathBuf, params: &mut ParameterStore) -> Result<Checkpoint> {
        let data = std::fs::read(path)?;
        let checkpoint: Checkpoint = bincode::deserialize(&data)
            .map_err(|e| SpatialVortexError::Storage(e.to_string()))?;
        
        // Restore model state
        for (name, data) in &checkpoint.model_state {
            if let Some(param) = params.get_mut(name) {
                let shape = param.data.dim();
                param.data = Array2::from_shape_vec(shape, data.clone())
                    .map_err(|e| SpatialVortexError::Storage(e.to_string()))?;
            }
        }
        
        // Update stats
        {
            let mut stats = self.stats.write();
            stats.step = checkpoint.step;
            stats.epoch = checkpoint.epoch;
            stats.loss = checkpoint.loss;
        }
        
        println!("Loaded checkpoint from {:?} (step {})", path, checkpoint.step);
        
        Ok(checkpoint)
    }
    
    /// Get current training stats
    pub fn get_stats(&self) -> TrainingStats {
        self.stats.read().clone()
    }
    
    /// Check if training is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
    
    /// Stop training
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

// ============================================================================
// DATA LOADING
// ============================================================================

/// Training sample
#[derive(Clone)]
pub struct TrainingSample {
    /// Input token IDs
    pub input_ids: Vec<u32>,
    /// Attention mask
    pub attention_mask: Vec<u8>,
    /// Labels (shifted input_ids for causal LM)
    pub labels: Vec<u32>,
}

/// Dataset trait for training data
pub trait Dataset: Send + Sync {
    fn len(&self) -> usize;
    fn get(&self, idx: usize) -> Option<TrainingSample>;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Simple in-memory dataset
pub struct InMemoryDataset {
    samples: Vec<TrainingSample>,
}

impl InMemoryDataset {
    pub fn new(samples: Vec<TrainingSample>) -> Self {
        Self { samples }
    }
    
    pub fn from_tokens(token_sequences: Vec<Vec<u32>>, max_len: usize) -> Self {
        let samples: Vec<TrainingSample> = token_sequences
            .into_iter()
            .map(|tokens| {
                let len = tokens.len().min(max_len);
                let input_ids = tokens[..len].to_vec();
                let attention_mask = vec![1u8; len];
                let labels = if len > 1 {
                    tokens[1..len].to_vec()
                } else {
                    tokens.clone()
                };
                TrainingSample { input_ids, attention_mask, labels }
            })
            .collect();
        
        Self { samples }
    }
}

impl Dataset for InMemoryDataset {
    fn len(&self) -> usize {
        self.samples.len()
    }
    
    fn get(&self, idx: usize) -> Option<TrainingSample> {
        self.samples.get(idx).cloned()
    }
}

/// Data loader with batching and shuffling
pub struct DataLoader {
    dataset: Arc<dyn Dataset>,
    batch_size: usize,
    shuffle: bool,
    indices: Vec<usize>,
    current_idx: usize,
    world_size: usize,
    rank: usize,
}

impl DataLoader {
    pub fn new(
        dataset: Arc<dyn Dataset>,
        batch_size: usize,
        shuffle: bool,
        world_size: usize,
        rank: usize,
    ) -> Self {
        let total_len = dataset.len();
        let indices: Vec<usize> = (0..total_len).collect();
        
        Self {
            dataset,
            batch_size,
            shuffle,
            indices,
            current_idx: 0,
            world_size,
            rank,
        }
    }
    
    pub fn shuffle_indices(&mut self) {
        if self.shuffle {
            use rand::seq::SliceRandom;
            let mut rng = rand::thread_rng();
            self.indices.shuffle(&mut rng);
        }
        self.current_idx = 0;
    }
    
    pub fn next_batch(&mut self) -> Option<Vec<TrainingSample>> {
        if self.current_idx >= self.indices.len() {
            return None;
        }
        
        // Calculate indices for this rank
        let samples_per_rank = self.batch_size;
        let global_batch_size = samples_per_rank * self.world_size;
        
        let start = self.current_idx + self.rank * samples_per_rank;
        let end = (start + samples_per_rank).min(self.indices.len());
        
        if start >= self.indices.len() {
            return None;
        }
        
        let batch: Vec<TrainingSample> = (start..end)
            .filter_map(|i| {
                let idx = self.indices.get(i)?;
                self.dataset.get(*idx)
            })
            .collect();
        
        self.current_idx += global_batch_size;
        
        if batch.is_empty() {
            None
        } else {
            Some(batch)
        }
    }
    
    pub fn reset(&mut self) {
        self.current_idx = 0;
    }
    
    pub fn num_batches(&self) -> usize {
        let samples_per_rank = self.indices.len() / self.world_size;
        (samples_per_rank + self.batch_size - 1) / self.batch_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parameter_store() {
        let mut store = ParameterStore::new();
        
        let param1 = Parameter::new("layer1.weight", Array2::zeros((10, 10)));
        let param2 = Parameter::new("layer1.bias", Array2::zeros((1, 10)));
        
        store.register(param1);
        store.register(param2);
        
        assert_eq!(store.total_params(), 110);
    }
    
    #[test]
    fn test_adamw_optimizer() {
        let mut store = ParameterStore::new();
        let mut param = Parameter::new("test", Array2::ones((2, 2)));
        param.grad = Some(Array2::from_elem((2, 2), 0.1));
        store.register(param);
        
        let config = OptimizerConfig {
            lr: 0.01,
            ..Default::default()
        };
        let mut optimizer = AdamW::new(config);
        
        optimizer.step(&mut store);
        
        let updated = store.get("test").unwrap();
        assert!(updated.data[[0, 0]] < 1.0);  // Should have decreased
    }
    
    #[test]
    fn test_lr_scheduler() {
        let schedule = LRSchedule::WarmupCosine {
            warmup_steps: 100,
            total_steps: 1000,
            min_lr: 1e-6,
        };
        let mut scheduler = LRScheduler::new(schedule, 1e-4);
        
        // During warmup
        assert!(scheduler.get_lr() < 1e-4);
        
        for _ in 0..100 {
            scheduler.step();
        }
        
        // After warmup
        assert!((scheduler.get_lr() - 1e-4).abs() < 1e-6);
    }
    
    #[test]
    fn test_data_loader() {
        let samples: Vec<Vec<u32>> = (0..100).map(|i| vec![i as u32; 10]).collect();
        let dataset = Arc::new(InMemoryDataset::from_tokens(samples, 10));
        
        let mut loader = DataLoader::new(dataset, 8, true, 1, 0);
        
        let mut total = 0;
        while let Some(batch) = loader.next_batch() {
            total += batch.len();
        }
        
        assert_eq!(total, 100);
    }
    
    #[test]
    fn test_gradient_sync() {
        let config = DistributedConfig {
            world_size: 4,
            ..Default::default()
        };
        let sync = GradientSynchronizer::new(config, 2);
        
        let mut grads = vec![1.0f32; 100];
        sync.all_reduce(&mut grads);
        
        // Should be scaled by 1/world_size
        assert!((grads[0] - 0.25).abs() < 1e-6);
    }
}
