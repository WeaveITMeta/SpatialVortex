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
pub mod ebrm_alignment;
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
pub mod betr_selector;
pub mod integration_test;
pub mod jepa;
pub mod neuro_symbolic;
pub mod task_specific_moe;
pub mod sacred_moe;
pub mod sacred_swarm;
pub mod generative_arch;
pub mod transitive_flux;
pub mod stacked_flux;
pub mod flux_object_macro;
pub mod flux_compression_sota;
pub mod reasoning_engine;
pub mod unified_inference;
pub mod recursive_chains;
pub mod conceptual_agglomeration;
pub mod geometric_world_model;
pub mod world_knowledge;
pub mod consciousness_learner;
pub mod web_knowledge;
pub mod web_quality_filter;
pub mod web_crawler;
pub mod calm_web_integration;
pub mod benchmark_weighted_retrieval;
pub mod rag_search;
pub mod test_time_compute;
pub use betr_selector::{
    BETRConfig, BenchmarkEmbeddingStore, BETRDocumentScorer, BETRDataSelector,
    BenchmarkEmbedding, ScoredDocument, create_betr_selector,
};
pub use task_specific_moe::{
    TaskSpecificSacredMoE, TaskSpecificMoEConfig, TaskExpertCluster,
    TaskCategory, ClusterTrainingStats, BenchmarkRouter,
};
pub use test_time_compute::{TTCWrapper, TTCConfig};
pub mod unified_knowledge_pipeline;
pub mod sacred_attention;
pub mod dynamic_rsi;
pub mod rl_actor_critic;
pub mod writing_gate;
pub mod structured_prediction;
pub mod secure_aggregation;
pub mod learning_to_rank;
pub mod pillar_integration;

pub use hallucinations::{VortexContextPreserver, HallucinationResult};
pub use ebrm_alignment::{
    AlignedEBRM, EnergyAlignmentConfig, GoldenPath, ContrastivePair,
    AlignmentStats, BenchmarkQA, AlignedTraceEnergy,
};
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
    SacredObservation, ControlSignal, ReasoningSignal,
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
pub use consciousness_learner::{
    ConsciousnessLearner, ConsciousnessConfig,
    SubjectNode, DynamicVortex, VortexStats,
    LearningStats, FailedQuestion, KnowledgeGapAnalysis,
    AttributeValue, OperationLog, OperationType,
    ExtractedFact,
};
pub use web_knowledge::{
    DuckDuckGoScraper, WebScraperConfig, SearchResult, ScraperStats,
    WebKnowledgeExtractor, WebKnowledge,
    BatchWebLearner, BatchLearnerStats,
};
pub use web_quality_filter::{
    BluWerpFilter, GopherQualityFilter, RepetitionFilter, SemanticQualityScorer,
    QualityFilterConfig, FilterResult, FilterFailure,
};
pub use web_crawler::{
    WebCrawler, CrawlerConfig, CrawledPage, CrawlerStats,
};
pub use benchmark_weighted_retrieval::{
    BenchmarkWeightedRetriever, BenchmarkWeightedRetrievalConfig, BenchmarkWeightedDocument,
    BenchmarkContext, RetrievalStats, BenchmarkAwareRAG,
};
pub use calm_web_integration::{
    CALMSemanticStore, SemanticFact, CALMWebConfig, SemanticStoreStats,
    CALMWebLearner, CALMWebLearnerStats,
};
// TODO: Re-enable when fast_knowledge_acquisition module is ready
// pub use fast_knowledge_acquisition::{
//     FastKnowledgeAcquisition, FastKnowledgeConfig, AcquisitionStats,
// };
pub use sacred_attention::{
    SacredAttentionPipeline, PipelineStats as SacredPipelineStats,
    KeywordExtractionHeader, Position3Output,
    RelationVerificationHeader, Position6Output,
    KnowledgeIntegrationHeader, Position9Output,
    ExtractedEntity, EntityType, ExtractedRelation,
    VerifiedFact, VerificationResult, IntegratedEmbedding, IntegrationStats,
    SACRED_POSITION_3, SACRED_POSITION_6, SACRED_POSITION_9,
    VORTEX_CYCLE as SACRED_VORTEX_CYCLE,
};
pub use unified_knowledge_pipeline::{
    UnifiedKnowledgePipeline, PipelineConfig, PipelineStats,
    ExtractedFact as PipelineFact, RetrievalResult, SemanticEmbedding,
};
pub use rl_actor_critic::{
    ActorCriticPolicy, ActorCriticStats, QTable, QTableStats,
    DiscreteAction, EvidenceEvent, EvidenceType,
    StateTransition, TransitionReward, RewardWeights,
    PathCurator, CurationMode, HumanFeedbackEncoder,
    RLMetrics, RLMetricsSummary,
};
pub use writing_gate::{
    WritingGate, GatePolicy, TraitProposal, ProposalVerdict,
    VerdictDecision, GateMetrics,
};
pub use structured_prediction::{
    TraitDependencyGraph, DependencyEdge, PropagationRule,
    UnaryPotential, PairwisePotential,
    BeliefPropagation, BPResult,
    CascadeEngine, CascadeResult, CascadeStats,
    ConsistencyChecker, ConsistencyResult, ConsistencyViolation,
};
pub use secure_aggregation::{
    DifferentialPrivacy, NoiseMechanism, SecretSharing, SecretShare,
    SecureAggregator, AggregationResult, AggregationStats,
    PrivacyBudget, FederatedProtocol, FederatedRoundResult, FederatedStats,
};
pub use learning_to_rank::{
    RankableItem, ItemType, LambdaRank, LambdaRankStats,
    SlidingIndex, IndexMode, IndexStats,
    RankTrainer, RankTrainerStats, RankMetrics,
};
pub use pillar_integration::{
    JEPAPathwayIntegration, InferenceMode, ScoredPath,
    GatedProposalPipeline, ProposalOrigin, GatedProposalResult,
    InferenceEvidence, ProvenanceTrace, ProvenanceStep,
};
