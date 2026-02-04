//! ML Training Module (2026 Distilled)
//!
//! Minimal, non-redundant training stack:
//! - **Burn** - Primary ML framework with tch-rs backend
//! - **SSO** - Spectral Sphere Optimizer (to be implemented)
//! - **Distributed** - Multi-GPU training with ZeRO
//! - **Pretraining** - MLM/CLM infrastructure

pub mod two_stage_rl;
pub mod loss_functions;
pub mod distributed;
pub mod burn_model;
pub mod pretraining;
pub mod gradient_checkpointing;

pub use two_stage_rl::TwoStageRLTrainer;

// Loss functions
pub use loss_functions::*;

// Distributed training exports
pub use distributed::{
    DistributedTrainer,
    DistributedConfig,
    DistributedBackend,
    ParallelismStrategy,
    ZeROStage,
    GradientSynchronizer,
    GradientBucket,
    Parameter,
    ParameterStore,
    AdamW,
    AdamWState,
    OptimizerConfig,
    LRScheduler,
    LRSchedule,
    TrainingStats as DistributedStats,
    Checkpoint,
    TrainingSample,
    Dataset,
    InMemoryDataset,
    DataLoader,
};

// Burn model exports
pub use burn_model::{
    ModelConfig,
    RoPE,
    TransformerLayer,
    SpatialVortexModel,
};

// Pre-training exports
pub use pretraining::{
    PretrainingObjective,
    PretrainingConfig,
    MaskedLanguageModel,
    CausalLanguageModel,
    LearningRateScheduler,
    PretrainingDataLoader,
    PretrainingBatch,
    PretrainingMetrics,
    Pretrainer,
    PretrainableModel,
    MASK_TOKEN_ID,
    PAD_TOKEN_ID,
    BOS_TOKEN_ID,
    EOS_TOKEN_ID,
};

// Gradient checkpointing exports
pub use gradient_checkpointing::{
    CheckpointStrategy,
    CheckpointConfig,
    CheckpointManager,
    CheckpointStats,
    MixedPrecisionConfig,
    PrecisionMode,
    GradScaler,
};
