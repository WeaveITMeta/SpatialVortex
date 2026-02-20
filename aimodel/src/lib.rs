//! # Vortex — Distilled SpatialVortex AGI/ASI Seed
//!
//! Sacred-geometry-centric AI framework with continuous learning through vortex dynamics,
//! recursive self-improvement (RSI), and multi-expert inference.
//!
//! ## Quick Start
//!
//! ### CLI — Interactive Chat
//!
//! ```bash
//! # Interactive REPL
//! cargo run --bin vortex-cli
//!
//! # Single prompt
//! cargo run --bin vortex-cli -- --prompt "What is sacred geometry?" --json
//!
//! # Piped input
//! echo "Hello" | cargo run --bin vortex-cli
//! ```
//!
//! ### REST API — OpenAI-Compatible Server
//!
//! ```bash
//! cargo run --bin vortex-api --features web
//!
//! # Then use any OpenAI client:
//! curl -X POST http://127.0.0.1:7000/v1/chat/completions \
//!   -H 'Content-Type: application/json' \
//!   -d '{"model":"vortex-0.1","messages":[{"role":"user","content":"Hello"}]}'
//! ```
//!
//! ### Library Usage
//!
//! ```rust,no_run
//! use vortex::{VortexEngine, VortexEngineConfig};
//!
//! // Create engine and chat
//! let mut engine = VortexEngine::new();
//! let response = engine.chat("What is sacred geometry?");
//! println!("{} (confidence: {:.0}%)", response.content, response.confidence * 100.0);
//! ```
//!
//! ## Architecture
//!
//! ### Core — Sacred Geometry
//! - [`FluxMatrixEngine`] — Vortex cycles (1→2→4→8→7→5→1), 3-6-9 sacred anchors
//! - [`VortexRunner`] — Continuous exponential learning with u64::MAX cycle reset
//! - [`ExhaustivePathwayOptimizer`] — Exact O(n!) enumeration with entropic objective
//!
//! ### ML — Machine Learning
//! - [`CALMEngine`] — Continuous Autoregressive Language Model (semantic encoder/decoder)
//! - [`UnifiedInferenceEngine`] — 3-pass iterative refinement with MoE routing
//! - [`UnifiedKnowledgePipeline`] — RETRIEVE → EXTRACT → EMBED → REASON → SCORE
//! - [`DynamicRSI`] — Runtime self-improving inference strategy per dataset
//! - [`SacredMoEModel`] — Mixture of Experts with geometric routing
//! - [`TransitiveFluxReasoner`] — Transitive reasoning via vortex flux matrix
//! - [`GenerativeVortexEngine`] — Generative architecture with BPE + CALM
//! - [`SpectralSphereOptimizer`] — μP-aligned optimizer (SSO)
//! - [`VortexContextPreserver`] — Hallucination detection (VCP)
//! - [`EnergyBasedReasoningModel`] — Energy-based reasoning (EBRM)
//!
//! ### Cognition — Thinking & Memory
//! - [`ThinkingEngine`] — Reasoning loop with sacred geometry
//! - [`VortexRunner`] — Continuous learning with vortex dynamics
//! - [`RAGEngine`] — Retrieval-augmented generation
//! - [`MemoryStore`] — RocksDB-backed persistent memory
//! - [`Constitution`] — Ethical constraints and truth checking
//!
//! ### Data — Benchmarks & Datasets
//! - [`RealBenchmarkEvaluator`] — Full eval harness (MMLU, GSM8K, ARC, HellaSwag, TruthfulQA, HumanEval)
//! - [`HFDatasetLoader`] — HuggingFace dataset streaming
//! - [`Attributes`] — Dynamic attribute system (replaces ELP tensors)
//!
//! ### Engine — Unified Inference
//! - [`VortexEngine`] — Single entry point wiring all subsystems (CLI + API)
//! - [`ChatMessage`] / [`ChatRole`] — OpenAI-compatible message types
//! - [`ChatResponse`] — Structured response with reasoning trace, confidence, and safety
//!
//! ### Serving — High-Performance API
//! - [`ContinuousBatchScheduler`] — 1200+ RPS continuous batching
//! - [`MoEGate`] — Expert routing for inference
//! - [`MCPServer`] — Model Context Protocol server
//!
//! ## Feature Flags
//!
//! | Feature | Description |
//! |---------|-------------|
//! | `burn-cpu` | Burn ML framework with CPU backend (default) |
//! | `burn-gpu` | Burn with PyTorch GPU backend |
//! | `burn-wgpu` | Burn with WebGPU backend |
//! | `gpu` | Alias for `burn-wgpu` |
//! | `onnx` | ONNX Runtime inference |
//! | `embeddings` | EmbedVec vector persistence |
//! | `web-learning` | Web crawler for knowledge acquisition (default) |
//! | `web` | Actix-web chat API server |
//! | `storage` | RocksDB hot-path storage |
//! | `transport` | WebTransport/QUIC networking |
//! | `bevy_viz` | Bevy 3D visualization |

// ============================================================================
// Module declarations
// ============================================================================

pub mod error;
pub mod data;
pub mod core;
pub mod ml;
pub mod ai;
pub mod storage;
pub mod visualization;
pub mod transport;
pub mod cognition;
pub mod serving;
pub mod engine;

// ============================================================================
// Top-level re-exports — Error types
// ============================================================================

pub use error::{Result, VortexError};

// ============================================================================
// Data — Models, attributes, benchmarks, datasets
// ============================================================================

pub use data::attributes::{Attributes, AttributeValue, AttributeAccessor};
pub use data::models::{BeamTensor, FluxMatrix};
pub use data::hf_datasets::{
    HFDatasetLoader, DatasetLoaderConfig, DatasetInfo, DatasetCategory,
};
pub use data::real_benchmarks::{
    RealBenchmarkEvaluator, RealBenchmarkResult, RealBenchmarkQuestion,
    load_mmlu, load_gsm8k, load_arc, load_hellaswag, load_truthfulqa, load_humaneval,
};

// ============================================================================
// Core — Sacred geometry engines
// ============================================================================

pub use core::sacred_geometry::{
    FluxMatrixEngine,
    GeometricInferenceEngine,
    VortexPositioningEngine,
    PatternCoherenceTracker,
};

// ============================================================================
// ML — Machine learning components
// ============================================================================

// CALM — Continuous Autoregressive Language Model
pub use ml::{CALMEngine, CALMConfig};

// Inference engines
pub use ml::unified_inference::{UnifiedInferenceEngine, UnifiedConfig};
pub use ml::unified_knowledge_pipeline::{UnifiedKnowledgePipeline, PipelineConfig, PipelineStats};
pub use ml::dynamic_rsi::{DynamicRSI, InferenceStrategy, InferenceObservation};

// Sacred geometry ML
pub use ml::{
    SacredMoEConfig, SacredMoEModel, SacredMoELayer,
    SacredExpert, ExpertSpecialization,
    GeometricRouter, MoEOutput,
    SacredObservation, ControlSignal, ReasoningSignal,
    PHI, PHI_INV, SACRED_POSITIONS, VORTEX_CYCLE,
};

// Generative architecture
pub use ml::{
    GenerativeVortexEngine, GenerativeConfig,
    SubwordTokenizer, SubwordToken,
    KnowledgeBase, KnowledgeTriple,
    AttributeFocusedAttention,
};

// Optimizers and training
pub use ml::{
    SpectralSphereOptimizer, SSOConfig,
    VortexContextPreserver,
    EnergyBasedReasoningModel,
};

// Pathway search
pub use ml::{
    ExhaustivePathwayOptimizer, PathwayConfig, ScoredPathway,
    CompoundingModel, StackedResult,
};

// Reasoning engines
pub use ml::{
    TransitiveFluxReasoner, TransitiveRelation, CountingMode,
    ComprehensiveReasoner,
};
pub use ml::{
    LogicTensorNetwork, NeuralTheoremProver, ProofResult,
};

// Knowledge acquisition
pub use ml::rag_search::{RAGSearchEngine, RAGSearchConfig};
pub use ml::{
    ConsciousnessLearner, ConsciousnessConfig,
    DuckDuckGoScraper, WebScraperConfig,
    WebCrawler, CrawlerConfig,
};

// Swarm intelligence
pub use ml::{SacredSwarm, SacredSwarmConfig, SwarmAgent, SwarmTask};

// JEPA
pub use ml::{JEPAConfig, JEPAPredictor, QuantumJEPAOptimizer};

// ============================================================================
// AI — Orchestration
// ============================================================================

pub use ai::{
    AIConsensusEngine,
    FluxReasoningChain,
};

// ============================================================================
// Cognition — Thinking, memory, reasoning
// ============================================================================

pub use cognition::{
    ThinkingEngine, ThinkingConfig, ThoughtChain,
    VortexRunner, VortexState,
    ToolRegistry, ToolCall, ToolResult,
    RAGEngine, RAGConfig,
    MemoryStore,
    Constitution, ConstitutionalGuard, TruthChecker,
};

// ============================================================================
// Serving — High-performance API
// ============================================================================

pub use serving::{
    ContinuousBatchScheduler, BatchConfig,
    MoEGate, ExpertType,
    MCPServer, MCPServerConfig,
};
#[cfg(feature = "web")]
pub use serving::configure_chat_routes;

// ============================================================================
// Storage — Persistence
// ============================================================================

pub use storage::{
    SacredEmbedding, SacredEmbeddingIndex,
    UnifiedStore, UnifiedStoreConfig,
};

// ============================================================================
// Engine — Unified inference entry point
// ============================================================================

pub use engine::{
    VortexEngine, VortexEngineConfig,
    ChatMessage, ChatRole, ChatResponse,
    ReasoningStep, Usage, SafetyResult,
};
