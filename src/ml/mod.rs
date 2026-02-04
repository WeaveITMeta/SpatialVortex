//! Machine Learning Module (2026 Distilled)
//!
//! Minimal, non-redundant ML stack:
//! - **ProductionEngine** - Primary inference with CALM
//! - **Burn** - Training framework with tch-rs backend
//! - **VortexModel** - Unified transformer with sacred geometry
//! - **VCP** - Vortex Context Preserver (hallucination detection)
//! - **EBRM** - Energy-Based Reasoning Model

pub mod inference;
pub mod hallucinations;
pub mod training;
pub mod vortex_model;
pub mod ebrm;

// Re-export inference
pub use inference::{
    TokenizerWrapper,
    TokenizedInput,
    ProductionEngine,
    ProductionConfig,
    ProductionStats,
};

// Re-export hallucinations (VCP - sacred core)
pub use hallucinations::{
    SignalSubspace,
    HallucinationDetector,
    VortexContextPreserver,
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

// Meta learning module
pub mod meta_learning;
pub use meta_learning::MetaLearner;
