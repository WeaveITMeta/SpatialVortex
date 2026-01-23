//! Universal Data Flow Pipeline
//!
//! A 5-layer architecture for processing any data modality through intelligent routing,
//! contextual processing, and AI-powered reasoning.
//!
//! ## Architecture Overview
//!
//! ```text
//! Raw Input (any modality)
//!     ↓
//! [Input Data Layer]          → Accept everything, unify access
//!     ↓
//! [Inference Layer]           → Route, map, contextualize instantly
//!     ↓
//! [Processing Layer]          → Transform, categorize, activate modalities
//!     ↓
//! [Intelligence Layer]        → Reason, learn, hypothesize, deduce
//!     ↓
//! [Output Layer]              → Generate best possible result (any modality)
//!     ↓
//! Knowledge Update (feedback loop)
//! ```
//!
//! ## Layer Responsibilities
//!
//! - **Input Layer**: Universal connectors, format detection, authentication, streaming
//! - **Inference Layer**: Intelligent routing, field mapping, modality detection, context
//! - **Processing Layer**: Modality-specific transforms, embeddings, vectorization
//! - **Intelligence Layer**: VortexModel reasoning, learning, hypothesis generation
//! - **Output Layer**: Multi-modal generation, visualization, structured export

pub mod input_layer;
pub mod inference_layer;
pub mod processing_layer;
pub mod intelligence_layer;
pub mod output_layer;
pub mod data_types;

// Re-export main types
pub use input_layer::{
    InputLayer,
    DataSource,
    DataSourceType,
    AuthType,
    UpdateMode,
    AnonymizationMode,
    RawInput,
    InputConfig,
};

pub use inference_layer::{
    InferenceLayer,
    RoutingDecision,
    FieldMapping,
    ModalityType,
    ContextualMetadata,
    InferenceConfig,
};

pub use processing_layer::{
    ProcessingLayer,
    ProcessedData,
    ModalityPipeline,
    ProcessingConfig,
    EmbeddingResult,
};

pub use intelligence_layer::{
    IntelligenceLayer,
    ReasoningResult,
    Hypothesis,
    LearningUpdate,
    IntelligenceConfig,
};

pub use output_layer::{
    OutputLayer,
    OutputResult,
    OutputFormat,
    OutputConfig,
};

pub use data_types::{
    UniversalData,
    DataEnvelope,
    ProcessingStage,
    PipelineMetrics,
};

use std::sync::Arc;
use parking_lot::RwLock;

/// The complete unified pipeline orchestrating all 5 layers
pub struct UniversalPipeline {
    /// Input data layer - accepts any data source
    input: Arc<InputLayer>,
    /// Inference layer - routes and contextualizes
    inference: Arc<InferenceLayer>,
    /// Processing layer - transforms by modality
    processing: Arc<ProcessingLayer>,
    /// Intelligence layer - reasons and learns
    intelligence: Arc<IntelligenceLayer>,
    /// Output layer - generates results
    output: Arc<OutputLayer>,
    /// Pipeline metrics
    metrics: RwLock<PipelineMetrics>,
    /// Knowledge feedback enabled
    feedback_enabled: bool,
}

impl UniversalPipeline {
    /// Create a new universal pipeline with default configuration
    pub fn new() -> Self {
        Self {
            input: Arc::new(InputLayer::new(InputConfig::default())),
            inference: Arc::new(InferenceLayer::new(InferenceConfig::default())),
            processing: Arc::new(ProcessingLayer::new(ProcessingConfig::default())),
            intelligence: Arc::new(IntelligenceLayer::new(IntelligenceConfig::default())),
            output: Arc::new(OutputLayer::new(OutputConfig::default())),
            metrics: RwLock::new(PipelineMetrics::default()),
            feedback_enabled: true,
        }
    }
    
    /// Process data through all 5 layers
    pub async fn process(&self, input: RawInput) -> Result<OutputResult, PipelineError> {
        let start = std::time::Instant::now();
        
        // Layer 1: Input - Accept and normalize
        let envelope = self.input.ingest(input).await?;
        
        // Layer 2: Inference - Route and contextualize
        let routed = self.inference.route(envelope).await?;
        
        // Layer 3: Processing - Transform by modality
        let processed = self.processing.transform(routed).await?;
        
        // Layer 4: Intelligence - Reason and learn
        let reasoned = self.intelligence.reason(processed).await?;
        
        // Layer 5: Output - Generate result
        let result = self.output.generate(reasoned).await?;
        
        // Update metrics
        {
            let mut metrics = self.metrics.write();
            metrics.total_processed += 1;
            metrics.total_time_ms += start.elapsed().as_millis() as u64;
        }
        
        // Feedback loop - update knowledge
        if self.feedback_enabled {
            self.intelligence.update_knowledge(&result).await;
        }
        
        Ok(result)
    }
    
    /// Get pipeline metrics
    pub fn get_metrics(&self) -> PipelineMetrics {
        self.metrics.read().clone()
    }
    
    /// Enable/disable knowledge feedback loop
    pub fn set_feedback(&mut self, enabled: bool) {
        self.feedback_enabled = enabled;
    }
}

impl Default for UniversalPipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Pipeline error types
#[derive(Debug, Clone)]
pub enum PipelineError {
    /// Input layer error
    InputError(String),
    /// Inference/routing error
    InferenceError(String),
    /// Processing error
    ProcessingError(String),
    /// Intelligence/reasoning error
    IntelligenceError(String),
    /// Output generation error
    OutputError(String),
    /// Configuration error
    ConfigError(String),
}

impl std::fmt::Display for PipelineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PipelineError::InputError(msg) => write!(f, "Input error: {}", msg),
            PipelineError::InferenceError(msg) => write!(f, "Inference error: {}", msg),
            PipelineError::ProcessingError(msg) => write!(f, "Processing error: {}", msg),
            PipelineError::IntelligenceError(msg) => write!(f, "Intelligence error: {}", msg),
            PipelineError::OutputError(msg) => write!(f, "Output error: {}", msg),
            PipelineError::ConfigError(msg) => write!(f, "Config error: {}", msg),
        }
    }
}

impl std::error::Error for PipelineError {}
