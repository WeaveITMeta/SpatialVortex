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
pub mod generative_arch;
pub mod transitive_flux;
pub mod flux_object_macro;
pub mod flux_compression_sota;
pub mod reasoning_engine;
pub mod unified_inference;
pub mod recursive_chains;
pub mod conceptual_agglomeration;
pub mod geometric_world_model;

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
    jepa_mse_loss, jepa_infonce_loss, QuantumJEPAOptimizer,
};
pub use transitive_flux::{
    TransitiveFluxReasoner, TransitiveRelation,
    CountingMode, GraphPath, ExtractedContext,
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
pub use generative_arch::{
    SubwordTokenizer, SubwordToken,
    SacredDynamicAttention, SacredAttentionConfig,
    FewShotContext, FewShotExample,
    GenerationHead,
    GenerativeVortexEngine, GenerativeConfig,
    SymbolicMathExecutor, MathResult,
    KnowledgeBase, KnowledgeTriple,
    DynamicMoERouter, ExpertType,
    AttributeFocusedAttention, AttributeImplication, ImplicationType,
    TrackedObject, ObjectPattern,
};
pub use flux_object_macro::{
    SubjectFluxFlow, GenericFluxEngine, PhysicsFluxEngine,
    FLUX_VORTEX_CYCLE, FLUX_SACRED_POSITIONS,
    next_vortex_position, is_sacred_position, sacred_magnification,
    run_vortex_flow,
    extract_confidence, pack_confidence,
    extract_ethos, pack_ethos,
    extract_logos, pack_logos,
    extract_pathos, pack_pathos,
    extract_entropy, pack_entropy,
    extract_flow_step, pack_flow_step,
    extract_position, pack_position,
    extract_sacred_boost, pack_sacred_boost,
    extract_subject_id, pack_subject_id,
    extract_flags, pack_flags,
    has_cross_ref, set_cross_ref,
    has_reversal_flag, set_reversal_flag,
    extract_elp_component, pack_adjusted_confidence,
};
pub use reasoning_engine::{
    TemporalStateTracker, TemporalFact, Polarity,
    MultiHopReasoner, ReasoningChain,
    SpanExtractor,
    SymbolicMathEngine, MathOp,
    ComprehensiveReasoner,
};
pub use recursive_chains::{SNOATNode, SNOATChain, ChainPathwayReasoner};
pub use conceptual_agglomeration::{
    ConceptNode, ConceptGraph, ConceptualReasoner,
    ConceptRef, ConceptRelation, ConceptRelationType,
    ResolvedAttributes, ResolvedValue,
    SACRED_DEPTH, CONFIDENCE_DECAY, CONCEPT_LATENT_DIM,
};
pub use geometric_world_model::{
    GeometricWorldModel, WorldStateTensor, WorldEncoder,
    GeometricRelationPredictor, WorldConsistencyObjective,
    CoordinateSpace, EMBEDVEC_THRESHOLD,
};
