# Background Training System - Continuous Model Evolution

**Status**: ✅ OPERATIONAL  
**Date**: December 30, 2025  
**Version**: 1.0.0

## Overview

The Background Training System enables **continuous model evolution** by automatically training improved models in the background using the DistributedTrainer and EustressTrainingPipeline. Models are versioned, tested, and hot-swapped when improvements are validated.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                 BACKGROUND TRAINING ARCHITECTURE                         │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌──────────────┐                                                       │
│  │ ASI Runs     │  During inference:                                    │
│  │ Inferences   │  • Processes entities                                 │
│  │              │  • Generates outputs                                  │
│  │              │  • Records performance                                │
│  └──────┬───────┘                                                       │
│         │                                                                │
│         ▼                                                                │
│  ┌──────────────┐                                                       │
│  │ Training     │  EustressTrainingPipeline:                           │
│  │ Pipeline     │  • Converts entities to TrainingSamples              │
│  │              │  • Applies sacred geometry weights                   │
│  │              │  • Generates prompts/completions                     │
│  └──────┬───────┘                                                       │
│         │                                                                │
│         ▼                                                                │
│  ┌──────────────┐                                                       │
│  │ Sample       │  Buffering:                                           │
│  │ Buffer       │  • Accumulates training samples                      │
│  │              │  • Max size: 10,000 samples (default)                │
│  │              │  • Auto-triggers when threshold reached              │
│  └──────┬───────┘                                                       │
│         │                                                                │
│         ▼                                                                │
│  ┌──────────────┐                                                       │
│  │ Training     │  Triggers:                                            │
│  │ Triggers     │  1. Scheduled (periodic, e.g., hourly)               │
│  │              │  2. Manual (user-initiated)                          │
│  │              │  3. RSI-triggered (after improvements)               │
│  │              │  4. Sample threshold (buffer full)                   │
│  └──────┬───────┘                                                       │
│         │                                                                │
│         ▼                                                                │
│  ┌──────────────┐                                                       │
│  │ Distributed  │  Training:                                            │
│  │ Trainer      │  • Data parallelism across GPUs                      │
│  │              │  • Gradient synchronization                          │
│  │              │  • Mixed precision (FP16/BF16)                       │
│  │              │  • Gradient checkpointing                            │
│  └──────┬───────┘                                                       │
│         │                                                                │
│         ▼                                                                │
│  ┌──────────────┐                                                       │
│  │ Model        │  Versioning:                                          │
│  │ Versioning   │  • Each training creates new version                 │
│  │              │  • Metadata: loss, accuracy, improvement             │
│  │              │  • Checkpoints saved to disk                         │
│  │              │  • Keep last N versions (default: 5)                 │
│  └──────┬───────┘                                                       │
│         │                                                                │
│         ▼                                                                │
│  ┌──────────────┐                                                       │
│  │ Hot-Swap     │  Auto-swap if:                                        │
│  │ Decision     │  • Improvement > threshold (default: 2%)             │
│  │              │  • Validation accuracy improved                      │
│  │              │  • Auto-swap enabled in config                       │
│  └──────┬───────┘                                                       │
│         │                                                                │
│         ▼                                                                │
│  ┌──────────────┐                                                       │
│  │ Active       │  Production model:                                    │
│  │ Model        │  • Used for all inferences                           │
│  │              │  • Can be manually swapped                           │
│  │              │  • Rollback to previous version                      │
│  └──────────────┘                                                       │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

## Key Components

### 1. BackgroundTrainingCoordinator
**File**: `src/ml/training/background_trainer.rs`

**Responsibilities**:
- Manage sample buffer
- Schedule training runs
- Coordinate with DistributedTrainer
- Version models
- Hot-swap improved models

**Key Methods**:
- `new()` - Create coordinator
- `add_entities()` - Add training samples
- `start()` - Start background training loop
- `stop()` - Stop training
- `trigger_training()` - Manual trigger
- `stats()` - Get statistics
- `get_model_versions()` - Get version history

### 2. EustressTrainingPipeline
**File**: `src/eustress_bridge/training_pipeline.rs`

**Responsibilities**:
- Convert EustressEntity to TrainingSample
- Apply sacred geometry weights
- Generate prompts and completions
- Calculate confidence scores

### 3. DistributedTrainer
**File**: `src/ml/training/distributed.rs`

**Responsibilities**:
- Multi-GPU training
- Gradient synchronization
- Checkpointing
- Optimization (AdamW)

## Configuration

### BackgroundTrainingConfig

```rust
pub struct BackgroundTrainingConfig {
    /// Enable background training
    pub enabled: bool,  // Default: true
    
    /// Training interval in seconds
    pub training_interval_secs: u64,  // Default: 3600 (1 hour)
    
    /// Minimum samples before training
    pub min_samples_for_training: usize,  // Default: 100
    
    /// Maximum samples per training batch
    pub max_samples_per_batch: usize,  // Default: 10000
    
    /// Number of training epochs
    pub num_epochs: usize,  // Default: 3
    
    /// Learning rate
    pub learning_rate: f32,  // Default: 0.001
    
    /// Batch size
    pub batch_size: usize,  // Default: 32
    
    /// Enable model versioning
    pub enable_versioning: bool,  // Default: true
    
    /// Model checkpoint directory
    pub checkpoint_dir: PathBuf,  // Default: "./checkpoints"
    
    /// Auto-swap improved models
    pub auto_swap_models: bool,  // Default: true
    
    /// Minimum improvement threshold for swap (%)
    pub min_improvement_threshold: f32,  // Default: 2.0
    
    /// Keep last N model versions
    pub keep_last_n_versions: usize,  // Default: 5
}
```

## Usage

### Basic Setup

```rust
use spatial_vortex::ml::training::{
    BackgroundTrainingCoordinator, BackgroundTrainingConfig,
};
use spatial_vortex::eustress_bridge::training_pipeline::EustressTrainingPipeline;
use spatial_vortex::eustress_bridge::flux_dynamics::FluxDynamics;

// Create training pipeline
let flux = FluxDynamics::new();
let pipeline = EustressTrainingPipeline::new(flux);

// Configure background training
let config = BackgroundTrainingConfig {
    enabled: true,
    training_interval_secs: 3600, // Train every hour
    min_samples_for_training: 100,
    auto_swap_models: true,
    min_improvement_threshold: 2.0, // 2% improvement
    ..Default::default()
};

// Create coordinator
let coordinator = BackgroundTrainingCoordinator::new(config, pipeline);

// Start training
let handle = coordinator.start().await;
```

### Integration with ASI Orchestrator

```rust
use spatial_vortex::ai::orchestrator::ASIOrchestrator;

// Create orchestrator with background training
let orchestrator = ASIOrchestrator::new()
    .await?
    .with_background_trainer(coordinator);

// Start background training
orchestrator.start_background_training().await?;

// System now trains continuously in background
```

### Adding Training Samples

```rust
// Samples are automatically added during inference
// Or add manually:
let entities = vec![
    EustressEntity::default(),
    // ... more entities
];

if let Some(ref trainer) = orchestrator.background_trainer {
    trainer.add_entities(entities).await;
}
```

### Manual Training Trigger

```rust
use spatial_vortex::ml::training::TrainingTrigger;

// Trigger training manually
orchestrator.trigger_training(TrainingTrigger::Manual);

// Trigger from RSI
orchestrator.trigger_training(TrainingTrigger::RSITriggered {
    reason: "Performance improvement applied".to_string(),
});
```

### Monitoring

```rust
// Get training statistics
if let Some(stats) = orchestrator.training_stats() {
    println!("Training runs: {}", stats.total_training_runs);
    println!("Samples trained: {}", stats.total_samples_trained);
    println!("Current version: {}", stats.current_model_version);
    println!("Model swaps: {}", stats.model_swaps);
    println!("Best accuracy: {:.2}%", stats.best_validation_accuracy * 100.0);
}

// Get model versions
let versions = orchestrator.get_model_versions();
for version in versions {
    println!("Version {}: acc={:.2}%, loss={:.4}", 
        version.version, 
        version.validation_accuracy * 100.0,
        version.training_loss);
}

// Get current active model
let current = orchestrator.current_model_version();
```

## Training Triggers

### 1. Scheduled Training
Runs automatically at configured intervals.

```rust
BackgroundTrainingConfig {
    training_interval_secs: 3600, // Every hour
    ..Default::default()
}
```

### 2. Manual Trigger
User-initiated training.

```rust
orchestrator.trigger_training(TrainingTrigger::Manual);
```

### 3. RSI-Triggered
Automatically triggered after RSI applies improvements.

```rust
// Automatically triggered in orchestrator when RSI applies changes
TrainingTrigger::RSITriggered { 
    reason: "Applied performance optimization".to_string() 
}
```

### 4. Sample Threshold
Triggered when buffer reaches 2x minimum samples.

```rust
BackgroundTrainingConfig {
    min_samples_for_training: 100,
    // Auto-triggers at 200 samples
    ..Default::default()
}
```

## Model Versioning

### Version Metadata

Each trained model includes:
- **Version number** (incremental)
- **Timestamp** (Unix epoch)
- **Checkpoint path** (file location)
- **Training samples** (count)
- **Training loss** (final loss)
- **Validation accuracy** (0-1)
- **Improvement** (% over previous)
- **Trigger type** (what initiated training)

### Version Management

```rust
// Get all versions
let versions = orchestrator.get_model_versions();

// Manually swap to specific version
if let Some(ref trainer) = orchestrator.background_trainer {
    trainer.swap_to_version(5)?;
}

// Versions are automatically pruned
// Keeps last N versions (default: 5)
```

## Hot-Swapping

### Auto-Swap Conditions

Models are automatically swapped when:
1. `auto_swap_models` is enabled
2. New model validation accuracy > previous best
3. Improvement ≥ `min_improvement_threshold`

### Manual Swap

```rust
// Swap to specific version
if let Some(ref trainer) = orchestrator.background_trainer {
    trainer.swap_to_version(3)?;
}
```

### Rollback

```rust
// Get previous version
let versions = orchestrator.get_model_versions();
let previous = versions[versions.len() - 2].version;

// Swap back
if let Some(ref trainer) = orchestrator.background_trainer {
    trainer.swap_to_version(previous)?;
}
```

## Integration with RSI

The background training system integrates seamlessly with the RSI loop:

1. **RSI applies improvement** → Code change
2. **Training triggered** → Learn from improvement
3. **Model trained** → New version created
4. **Validation** → Accuracy measured
5. **Auto-swap** → If improved
6. **Feedback loop** → Better performance

```rust
// In orchestrator RSI handler
if should_apply {
    match engine_guard.apply_proposal(&proposal).await {
        Ok(_) => {
            tracing::info!("Auto-applied improvement: {}", proposal.description);
            
            // Trigger training to learn from improvement
            if let Some(ref trainer) = self.background_trainer {
                trainer.trigger_training(TrainingTrigger::RSITriggered {
                    reason: proposal.description.clone(),
                });
            }
        }
        Err(e) => {
            tracing::error!("Failed to apply proposal: {}", e);
        }
    }
}
```

## Performance Characteristics

### Training Overhead
- **CPU**: Minimal when idle, high during training
- **Memory**: ~100KB per 1000 samples in buffer
- **Disk**: ~50MB per model checkpoint (varies by model size)

### Training Duration
- **Small model** (100M params): ~5-10 minutes
- **Medium model** (1B params): ~30-60 minutes
- **Large model** (10B params): ~2-4 hours

### Throughput
- **Samples/sec**: 100-1000 (depends on GPU)
- **Epochs**: 2-5 typical
- **Batch size**: 32-128 typical

## Best Practices

### 1. Sample Collection
```rust
// Collect diverse samples
// Include edge cases and failures
// Balance positive and negative examples
```

### 2. Training Frequency
```rust
// Production: Train every 1-4 hours
BackgroundTrainingConfig {
    training_interval_secs: 3600,
    ..Default::default()
}

// Development: Train more frequently
BackgroundTrainingConfig {
    training_interval_secs: 600, // 10 minutes
    ..Default::default()
}
```

### 3. Model Versioning
```rust
// Keep enough versions for rollback
BackgroundTrainingConfig {
    keep_last_n_versions: 10, // Last 10 versions
    ..Default::default()
}
```

### 4. Auto-Swap Threshold
```rust
// Conservative: Require significant improvement
BackgroundTrainingConfig {
    min_improvement_threshold: 5.0, // 5%
    ..Default::default()
}

// Aggressive: Accept small improvements
BackgroundTrainingConfig {
    min_improvement_threshold: 1.0, // 1%
    ..Default::default()
}
```

## Testing

### Unit Tests
**File**: `tests/background_training_test.rs`

Tests cover:
- Coordinator creation
- Sample buffering
- Training triggers
- Start/stop lifecycle
- Statistics tracking
- Orchestrator integration

### Integration Demo
**File**: `examples/background_training_demo.rs`

Demonstrates:
1. Training pipeline setup
2. Orchestrator integration
3. Sample generation
4. Scheduled training
5. Manual triggers
6. RSI triggers
7. Model versioning
8. Statistics monitoring

**Run demo**:
```bash
cargo run --example background_training_demo
```

## Production Deployment

### Recommended Configuration

```rust
BackgroundTrainingConfig {
    enabled: true,
    training_interval_secs: 3600, // 1 hour
    min_samples_for_training: 500, // Enough for meaningful training
    max_samples_per_batch: 10000,
    num_epochs: 3,
    learning_rate: 0.0001, // Conservative
    batch_size: 64,
    enable_versioning: true,
    checkpoint_dir: PathBuf::from("/data/checkpoints"),
    auto_swap_models: true,
    min_improvement_threshold: 2.0, // 2% improvement
    keep_last_n_versions: 10,
}
```

### Monitoring

```rust
// Expose metrics endpoint
async fn training_metrics_handler(orchestrator: Arc<ASIOrchestrator>) -> impl Reply {
    json!({
        "training": orchestrator.training_stats(),
        "versions": orchestrator.get_model_versions(),
        "current_version": orchestrator.current_model_version(),
        "buffer_size": orchestrator.background_trainer
            .as_ref()
            .map(|t| t.sample_buffer_size())
            .unwrap_or(0),
    })
}
```

### Disk Management

```rust
// Periodically clean old checkpoints
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(86400)); // Daily
    loop {
        interval.tick().await;
        
        // Clean checkpoints older than 30 days
        cleanup_old_checkpoints(&checkpoint_dir, 30).await;
    }
});
```

## Future Enhancements

### Phase 2: Advanced Training
- Curriculum learning (easy → hard samples)
- Active learning (select most informative samples)
- Multi-task learning (train on multiple objectives)
- Transfer learning (fine-tune from base models)

### Phase 3: Distributed Coordination
- Multi-node training coordination
- Federated learning across instances
- Model ensemble training
- A/B testing of model versions

### Phase 4: AutoML
- Hyperparameter optimization
- Architecture search
- Learning rate scheduling
- Automatic batch size tuning

## Conclusion

The Background Training System enables **continuous model evolution** by:

✅ **Collecting** training samples automatically  
✅ **Training** models in background (scheduled/triggered)  
✅ **Versioning** all trained models  
✅ **Validating** improvements  
✅ **Hot-swapping** better models  
✅ **Integrating** with RSI for learning from improvements  
✅ **Monitoring** training progress  
✅ **Managing** disk space and versions  

Combined with the RSI loop and runtime detector, this creates a **fully autonomous learning system** that continuously improves itself through both code modifications and model training.

## References

- `src/ml/training/background_trainer.rs` - Core implementation
- `src/eustress_bridge/training_pipeline.rs` - Sample generation
- `src/ml/training/distributed.rs` - Distributed training
- `src/ai/orchestrator.rs` - Orchestrator integration
- `tests/background_training_test.rs` - Test suite
- `examples/background_training_demo.rs` - Demo application
- `docs/AUTONOMOUS_RSI_COMPLETE.md` - Full autonomous system
- `docs/RUNTIME_DETECTOR.md` - Runtime monitoring
