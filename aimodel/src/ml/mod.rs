//! Machine Learning Module
//!
//! - **VCP** - Vortex Context Preserver (hallucination detection)
//! - **EBRM** - Energy-Based Reasoning Model
//! - **SSO** - Spectral Sphere Optimizer (μP-aligned)
//! - **CALM** - Continuous Autoregressive Language Models
//! - **BurnSSO** - Burn-native SSO with tensor operations
//! - **BurnCALM** - Burn-native CALM autoencoder
//! - **backends** - GPU acceleration (tch/wgpu)

pub mod hallucinations;
pub mod ebrm;
pub mod training;
pub mod calm;
pub mod burn_calm;
pub mod vortex_discovery;
pub mod backends;
pub mod pathway;
pub mod huggingface;
pub mod continuous_learning;
pub mod gpu_trainer;
pub mod rag_search;
pub mod integration_test;
pub mod jepa;
pub mod neuro_symbolic;
pub mod sacred_moe;
pub mod sacred_swarm;

pub use hallucinations::{VortexContextPreserver, HallucinationResult};
pub use ebrm::{EnergyBasedReasoningModel, TraceEnergy};
pub use training::{SpectralSphereOptimizer, SSOConfig, SpectralScaler, BurnSSO, SSOState, AdaptiveSSO};
pub use calm::{CALMEngine, CALMConfig};
pub use burn_calm::{BurnCALM, BurnCALMConfig, LatentEnergyScorer};
pub use vortex_discovery::{VortexDiscovery, DiscoveryConfig};
pub use backends::backend_info;
pub use pathway::{ExhaustivePathwayOptimizer, PathwayConfig, ScoredPathway, CompoundingModel, StackedResult};
pub use huggingface::{HFModelLoader, HFModelConfig, RSIState};
pub use continuous_learning::{
    ContinuousTrainer, ContinuousLearningConfig,
    RSIEpochScheduler, RSISignal, TrainingRecommendation,
    SyntheticDataGenerator, SyntheticExample,
    TrainingBatch, AdaptiveLearningRate,
    EpochResult, TrainingSessionResult,
};
pub use jepa::{
    JEPAConfig, JEPAPredictor, JEPATrainer, JEPAStats,
    HierarchicalDeductionEngine, DeductionRules, DeductionStep,
    jepa_mse_loss, jepa_infonce_loss,
};
pub use neuro_symbolic::{
    LogicTensorNetwork, RuleEmbedding,
    NeuralTheoremProver, ProofResult, ProofStep,
    EnergyPathScorer, PathNode, PathScore,
    HybridInferenceEngine, HybridConfig, HybridStats, HybridInferenceResult,
    ImaginationEngine, WorldState, CounterfactualResult,
};
pub use sacred_moe::{
    SacredMoEConfig, SacredMoEModel, SacredMoELayer,
    SacredExpert, ExpertSpecialization,
    GeometricRouter, RouterOutput,
    MultiHeadLatentAttention, MoEOutput, ModelStats,
    PHI, PHI_INV, SACRED_POSITIONS, VORTEX_CYCLE,
};
pub use sacred_swarm::{
    SacredSwarmConfig, SacredSwarm, SwarmAgent, SwarmTask,
    AgentSpecialization as SwarmAgentSpecialization, AgentState, TaskState,
    SwarmStepResult, SwarmRunResult, SwarmStats,
    GeometricOptimizer,
};
