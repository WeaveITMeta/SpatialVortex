//! Machine Learning Module
//!
//! Contains all ML and AI functionality:
//! - Inference engines
//! - Training infrastructure
//! - Backend selection (Burn/Candle)

pub mod inference;
pub mod backend;
pub mod enhancement;
pub mod hallucinations;
pub mod training;
pub mod rl_gradient_optimizer;
pub mod meta_learning;
pub mod vortex_model;  // Unified Vortex Model - the crown jewel
pub mod ebrm;  // Energy-Based Reasoning Model (inspired by Logical Intelligence)
pub mod exhaustive_pathway;  // Exhaustive n! pathway enumeration with stacked federated inference

// Re-export inference
pub use inference::{
    InferenceEngine,
    OnnxInferenceEngine,
    TokenizerWrapper,
    TokenizedInput,
    ASIIntegrationEngine,
    PositionalEncoding,
    SelfAttention,
    MultiHeadAttention,
    FeedForwardNetwork,
    TransformerBlock,
    ActivationFunction,
    DynamicPositionalEncoding,
    ConfidenceContextManager,
    ContextStats,
};

// Re-export training
pub use training::{
    Trainer,
    TrainingConfig,
    TrainingMetrics,
    LossFunction,
    Optimizer,
    OptimizerType,
    Trainable,
};

// Re-export backend
pub use backend::{
    BackendSelector,
    BackendType,
    BackendError,
    BackendInfo,
};

// Re-export hallucinations
pub use hallucinations::{
    SignalSubspace,
    HallucinationDetector,
    VortexContextPreserver,
};

// Re-export enhancement
pub use enhancement::*;

// Re-export exhaustive pathway optimizer
pub use exhaustive_pathway::{
    ExhaustivePathwayOptimizer,
    PathwayConfig,
    ScoredPath,
    StackedResult,
    StackStats,
    EBRMSentenceRefiner,
    CompoundingModel,
};

// Re-export EBRM (Energy-Based Reasoning Model)
pub use ebrm::{
    EnergyBasedReasoningModel,
    TraceEnergy,
    FailureLocation,
    ConstraintType,
    PositionEnergy,
    ChannelBalanceConfig,
    LatentSpaceEditor,
    RefinementResult,
    BackwardConditioner,
    ConditionedResult,
};

// Re-export Vortex Model - the unified AI architecture
pub use vortex_model::{
    VortexModel,
    VortexModelConfig,
    VortexTransformerLayer,
    RMSNorm,
    SwiGLUFFN,
    VCPState,
    GenerationStats as VortexGenerationStats,
};

// Re-export RoPE and GQA from inference
pub use inference::{
    RotaryPositionEmbedding,
    RoPEConfig,
    ExtendedRoPE,
    GroupedQueryAttention,
    GQAConfig,
    GQAKVCache,
};

// Re-export pre-training infrastructure
pub use training::{
    PretrainingObjective,
    PretrainingConfig,
    MaskedLanguageModel,
    CausalLanguageModel,
    Pretrainer,
    PretrainableModel,
    CheckpointStrategy,
    CheckpointConfig,
    CheckpointManager,
    GradScaler,
    PrecisionMode,
};
