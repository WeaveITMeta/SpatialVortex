//! ML Training Module
//!
//! Contains training infrastructure:
//! - Trainers and optimizers
//! - Loss functions
//! - Sacred gradient descent
//! - Vortex SGD

pub mod trainer;
pub mod vortex_sgd;
pub mod sacred_gradients;
pub mod loss_functions;
pub mod color_loss;
pub mod aspect_color_trainer;
pub mod two_stage_rl;
pub mod distributed;  // Distributed training with data parallelism
pub mod burn_model;   // Burn-based model definitions for training
// REMOVED: pub mod eustress_adapter;  // EustressEngine → DistributedTrainer adapter - will be reimplemented via MCP server
pub mod pretraining;  // Pre-training infrastructure (MLM, CLM)
pub mod gradient_checkpointing;  // Memory-efficient training
// REMOVED: pub mod background_trainer;  // Eustress-dependent - will be reimplemented via MCP server

// Re-export training types
pub use trainer::{
    Trainer,
    TrainingConfig,
    TrainingMetrics,
    LossFunction,
    Optimizer,
    OptimizerType,
    Trainable,
};

pub use vortex_sgd::VortexSGD;
pub use sacred_gradients::SacredGradientField;
pub use loss_functions::*;
pub use color_loss::{ColorLossFunction, ColorLossCombination};
pub use aspect_color_trainer::{
    ColorDatasetGenerator,
    ColorDatasetConfig,
    AspectColorModelTrainer,
};
pub use two_stage_rl::{
    TwoStageRLTrainer,
    TrainingStage,
    TwoStageConfig,
    TrainingStats as TwoStageStats,
};

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

// REMOVED: EustressEngine training adapter exports - will be reimplemented via MCP server
// pub use eustress_adapter::{
//     EustressTrainingAdapter,
//     EustressTrainingConfig,
//     EustressTrainingStats,
//     TokenizedBatch,
//     SimpleTokenizer,
// };

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

// REMOVED: Background training exports - Eustress-dependent, will be reimplemented via MCP server
// pub use background_trainer::{
//     BackgroundTrainingCoordinator,
//     BackgroundTrainingConfig,
//     TrainingTrigger,
//     ModelVersion,
//     TrainingStats as BackgroundTrainingStats,
// };
