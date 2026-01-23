//! Consciousness Simulation Module
//!
//! Implements multiple theories of consciousness:
//! - Global Workspace Theory (GWT) - attention and broadcasting
//! - Meta-Cognition - self-awareness and introspection
//! - Predictive Processing - learning from prediction errors
//! - Integrated Information Theory (IIT) - Φ measurement

pub mod global_workspace;
pub mod cognitive_module;
pub mod attention;
pub mod thought;
pub mod consciousness_simulator;
pub mod meta_cognition;
pub mod predictive_processing;
pub mod integrated_information;
pub mod analytics;
pub mod streaming;
pub mod background_learner;
pub mod memory_palace;
pub mod subject_graph;
pub mod workspace_flux;
pub mod dream_module;
// REMOVED: pub mod eustress_cognitive_module; - will be reimplemented via MCP server

pub use global_workspace::GlobalWorkspace;
pub use cognitive_module::{CognitiveModule, ModuleResponse, AttentionScore};
pub use attention::AttentionMechanism;
pub use thought::{Thought, ThoughtPriority};
pub use dream_module::DreamModule;
pub use consciousness_simulator::ConsciousnessSimulator;
pub use meta_cognition::{MetaCognitiveMonitor, ThinkingPattern, SelfAwarenessMetrics, MentalState};
pub use predictive_processing::{PredictiveProcessor, SurpriseSignal, PredictionResult};
pub use integrated_information::IntegratedInformationCalculator;
pub use analytics::{
    AnalyticsSnapshot, ConsciousnessMetrics, MetaCognitiveMetrics, PredictiveMetrics,
    ELPMetrics, SacredGeometryMetrics, SessionStats, StreamingEvent, ThoughtMetrics,
    WordLevelInsights, SelectionAnalysisResult, PatternInfo, AnalyticsBroadcaster
};
pub use streaming::{ConsciousnessStreamingServer, WordTracker, EventFilter, create_word_insights};
pub use background_learner::{BackgroundLearner, LearningStats, BackgroundLearningConfig};
pub use memory_palace::{MemoryPalace, ConsciousnessState, PredictiveState, MetaCognitiveState, PhiState};
pub use subject_graph::{SubjectDefinition as FluxSubjectDefinition, SubjectState, SubjectId, SubjectGraph, FluxEdge};
pub use workspace_flux::{FluxWorkspace, FluxMetrics, encode_text_to_workspace};
