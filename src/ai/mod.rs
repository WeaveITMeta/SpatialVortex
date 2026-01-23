//! AI Integration Module
//!
//! Contains AI and API integration components:
//! - ASI Orchestrator (unified intelligence coordinator)
//! - AI consensus mechanisms
//! - AI integration layer
//! - API routing
//! - Chat API with ONNX embeddings
//! - API endpoints
//! - Production server
//! - Additional production endpoints

#[cfg(not(target_arch = "wasm32"))]
pub mod orchestrator;
#[cfg(not(target_arch = "wasm32"))]
pub mod meta_orchestrator;
#[cfg(not(target_arch = "wasm32"))]
pub mod unified_api;
#[cfg(not(target_arch = "wasm32"))]
pub mod consensus;
pub mod integration;
// TEMPORARY: parallel_fusion.rs corrupted (only contains "S")
// pub mod parallel_fusion;
pub mod tools;
pub mod response_quality;
pub mod conversation_history;
pub mod chat_persistence;
pub mod prompt_templates;
pub mod reasoning_chain;
pub mod self_verification;
pub mod audit;
#[cfg(not(target_arch = "wasm32"))]
pub mod router;
#[cfg(not(target_arch = "wasm32"))]
pub mod api;
#[cfg(not(target_arch = "wasm32"))]
pub mod endpoints;
#[cfg(not(target_arch = "wasm32"))]
pub mod chat_api;
#[cfg(not(target_arch = "wasm32"))]
pub mod coding_api;
#[cfg(not(target_arch = "wasm32"))]
pub mod chat_endpoints;
#[cfg(not(target_arch = "wasm32"))]
pub mod chat_history_api;
#[cfg(not(target_arch = "wasm32"))]
pub mod dual_response_api;
#[cfg(not(target_arch = "wasm32"))]
pub mod whisper_api;
#[cfg(not(target_arch = "wasm32"))]
pub mod rag_endpoints;
#[cfg(not(target_arch = "wasm32"))]
pub mod canvas_api;
#[cfg(not(target_arch = "wasm32"))]
pub mod code_executor;
#[cfg(not(target_arch = "wasm32"))]
pub mod code_execution_api;
#[cfg(not(target_arch = "wasm32"))]
pub mod session_memory;
#[cfg(not(target_arch = "wasm32"))]
pub mod session_api;
#[cfg(not(target_arch = "wasm32"))]
pub mod collaboration;
#[cfg(not(target_arch = "wasm32"))]
pub mod monitoring_endpoints;
#[cfg(not(target_arch = "wasm32"))]
pub mod benchmark_api;
#[cfg(not(target_arch = "wasm32"))]
pub mod server;
#[cfg(not(target_arch = "wasm32"))]
pub mod swagger;
#[cfg(not(target_arch = "wasm32"))]
pub mod safety;
#[cfg(not(target_arch = "wasm32"))]
pub mod multi_source_search;
#[cfg(not(target_arch = "wasm32"))]
pub mod task_api;
#[cfg(not(target_arch = "wasm32"))]
pub mod response_processor;
#[cfg(all(not(target_arch = "wasm32"), feature = "color_ml"))]
pub mod color_integration;
#[cfg(all(not(target_arch = "wasm32"), feature = "agents"))]
pub mod consciousness_api;
#[cfg(not(target_arch = "wasm32"))]
pub mod vector_consensus;
/// Consensus storage for Confidence Lake
pub mod consensus_storage;

/// Flux reasoning - AGI core (geometric reasoning substrate)
pub mod flux_reasoning;

/// AGI API endpoint
#[cfg(not(target_arch = "wasm32"))]
pub mod agi_api;

/// Self-evolving AGI loop
#[cfg(not(target_arch = "wasm32"))]
pub mod evolution;

/// Meta-learning system (pattern extraction, matching, acceleration)
pub mod meta_learning;
pub mod meta_learning_matcher;
#[cfg(all(not(target_arch = "wasm32"), feature = "lake"))]
pub mod meta_learning_postgres;
pub mod meta_learner_evolution;
pub mod autonomous_architecture_search;

/// AGI Core Components
pub mod goal_planner;
pub mod causal_reasoning;
pub mod self_improvement;
pub mod curiosity_engine;
#[cfg(not(target_arch = "wasm32"))]
pub mod agi_core;

/// Working Memory System (short-term context retention)
pub mod working_memory;

/// Transfer Learning (cross-domain knowledge application)
pub mod transfer_learning;

/// Reasoning Integration (unified AGI reasoning pipeline)
#[cfg(not(target_arch = "wasm32"))]
pub mod reasoning_integration;

/// Billing and monetization system
#[cfg(not(target_arch = "wasm32"))]
pub mod billing;

/// Production API endpoints with auth and rate limiting
#[cfg(not(target_arch = "wasm32"))]
pub mod production_api;

// REMOVED: EustressEngine integration - will be reimplemented via MCP server
// pub mod eustress_integration;

// Re-exports
#[cfg(not(target_arch = "wasm32"))]
pub use orchestrator::ASIOrchestrator;
#[cfg(not(target_arch = "wasm32"))]
pub use orchestrator::{ReasoningOrchestrationConfig, ReasoningOrchestrationResult, TargetedReasoningResult};
#[cfg(not(target_arch = "wasm32"))]
pub use meta_orchestrator::{MetaOrchestrator, RoutingStrategy, UnifiedResult, OrchestratorSource};
#[cfg(not(target_arch = "wasm32"))]
pub use integration::AIModelIntegration;
#[cfg(not(target_arch = "wasm32"))]
pub use api::*;
#[cfg(not(target_arch = "wasm32"))]
pub use server::*;
#[cfg(not(target_arch = "wasm32"))]
pub use endpoints::*;
#[cfg(not(target_arch = "wasm32"))]
pub use vector_consensus::{ConsensusVectorField, ResponseVector, ProblemSolvingType};
#[cfg(not(target_arch = "wasm32"))]
pub use consensus_storage::ConsensusStoragePolicy;

// Re-export key components for easy access
pub use flux_reasoning::{FluxReasoningChain, FluxThought};
pub use consensus::{AIConsensusEngine, AIProvider};
pub use agi_api::agi_endpoint;
#[cfg(not(target_arch = "wasm32"))]
pub use evolution::EvolutionEngine;

// Re-export meta-learning types
#[cfg(not(target_arch = "wasm32"))]
pub use meta_learning::{
    ReasoningPattern, QuerySignature, TransformationSnapshot,
    AccelerationResult, LearningMetrics, PatternExtractor,
    PatternStorage, InMemoryPatternStorage,
};
#[cfg(not(target_arch = "wasm32"))]
pub use meta_learning_matcher::{
    PatternMatcher, QueryAccelerator, FeedbackCollector,
};
#[cfg(all(not(target_arch = "wasm32"), feature = "lake"))]
pub use meta_learning_postgres::PostgresPatternStorage;

// Re-export AGI core components
pub use goal_planner::{GoalPlanner, Goal, GoalStatus, Plan, PlanStatus};
pub use causal_reasoning::{CausalWorldModel, CausalGraph, CausalNode, CausalValue, Counterfactual, Intervention};
pub use self_improvement::{
    MetaLearner, PerformanceMetrics, ArchitectureConfig, Experiment,
    ExperimentStatus, PerformanceTracker, SearchSpace, SelfImprovementStats,
};
pub use meta_learner_evolution::{
    MetaLearnerEvolution, SacredIntervention, InterventionTiming,
    InterventionRecord, EvolutionStats,
};
pub use autonomous_architecture_search::{
    AutonomousArchitectureSearch, ArchitecturePattern, LayerConfig,
    LayerType, ActivationType, SacredIntegration, PatternMetrics,
    TestResult, SearchStats,
};
pub use curiosity_engine::{CuriosityEngine, KnowledgeGap, Hypothesis};
#[cfg(not(target_arch = "wasm32"))]
pub use agi_core::{AGICore, AGIState, AGIMode, AGIResponse, AGIStats};

// Re-export new AGI components
pub use working_memory::{WorkingMemory, ContextWindow, MemoryItem, MemoryContent, MemorySource, MemorySummary};
pub use transfer_learning::{TransferLearningEngine, Domain, Concept, Principle, Skill, Analogy, TransferResult};
#[cfg(not(target_arch = "wasm32"))]
pub use reasoning_integration::{ReasoningIntegration, ReasoningMode, IntegrationState, IntegratedResponse};
