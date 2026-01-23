//! Multi-language coding agent with SpatialVortex integration
//!
//! This module implements an AI coding agent capable of generating code in 24+
//! programming languages with geometric-semantic reasoning via SpatialVortex.

pub mod coding_agent;
pub mod coding_agent_enhanced;
pub mod thinking_agent;
pub mod task_manager;
pub mod task_persistence;
pub mod first_principles;
pub mod language;
pub mod executor;
pub mod error;
pub mod llm_bridge;
pub mod prompts;
pub mod symbolica_bridge;
pub mod knowledge;
pub mod prompt_template;
pub mod self_optimization;
pub mod improvements;

pub use coding_agent::{CodingAgent, AgentConfig, TaskResult};
pub use coding_agent_enhanced::{
    EnhancedCodingAgent, ReasoningTaskResult, LearningMetrics, BenchmarkResult
};
pub use thinking_agent::{ThinkingAgent, ThinkingResult};
pub use language::{Language, LanguageDetector};
pub use executor::{CodeExecutor, ExecutionResult};
pub use error::{AgentError, Result};
pub use llm_bridge::{LLMBridge, LLMBackend, LLMConfig};
pub use prompts::{PromptBuilder, Example};
pub use knowledge::{CodeKnowledge, CodeExample, KnowledgeStats};
pub use prompt_template::{PromptTemplate, PromptTemplateLibrary, Difficulty};
pub use self_optimization::{
    SelfOptimizationAgent, BottleneckDetector, BottleneckPrediction, 
    OptimizationAction, MetricsSnapshot
};
pub use task_manager::{
    TaskManager, Task, TaskStatus, TaskType, TaskSession, TaskProgress, TaskExecutionResult
};
pub use first_principles::{
    FirstPrinciplesReasoner, FirstPrinciplesResult, TruthAssessment, 
    DeceptionType, LogicalOperation, ReasoningStep
};
