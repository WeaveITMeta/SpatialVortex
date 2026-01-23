use thiserror::Error;
use std::fmt;

pub type Result<T> = std::result::Result<T, SpatialVortexError>;

/// Error context for better debugging and recovery
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub flux_position: Option<u8>,
    pub sacred_position: bool,
    pub confidence: Option<f32>,
    pub operation: Option<String>,
    pub component: Option<String>,
}

impl ErrorContext {
    pub fn new() -> Self {
        Self {
            flux_position: None,
            sacred_position: false,
            confidence: None,
            operation: None,
            component: None,
        }
    }
    
    pub fn with_flux_position(mut self, position: u8) -> Self {
        self.flux_position = Some(position);
        self.sacred_position = [3, 6, 9].contains(&position);
        self
    }
    
    pub fn with_confidence(mut self, strength: f32) -> Self {
        self.confidence = Some(strength);
        self
    }
    
    pub fn with_operation(mut self, op: impl Into<String>) -> Self {
        self.operation = Some(op.into());
        self
    }
    
    pub fn with_component(mut self, comp: impl Into<String>) -> Self {
        self.component = Some(comp.into());
        self
    }
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        
        if let Some(comp) = &self.component {
            parts.push(format!("component={}", comp));
        }
        if let Some(op) = &self.operation {
            parts.push(format!("operation={}", op));
        }
        if let Some(pos) = self.flux_position {
            let sacred = if self.sacred_position { " (sacred)" } else { "" };
            parts.push(format!("position={}{}", pos, sacred));
        }
        if let Some(signal) = self.confidence {
            parts.push(format!("signal={:.2}", signal));
        }
        
        if parts.is_empty() {
            write!(f, "[no context]")
        } else {
            write!(f, "[{}]", parts.join(", "))
        }
    }
}

#[derive(Error, Debug)]
pub enum SpatialVortexError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Redis error: {0}")]
    #[cfg(not(target_arch = "wasm32"))]
    Redis(#[from] redis::RedisError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("HTTP client error: {0}")]
    #[cfg(not(target_arch = "wasm32"))]
    HttpClient(#[from] reqwest::Error),

    #[error("Invalid flux matrix configuration: {0}")]
    InvalidFluxMatrix(String),

    #[error("Inference engine error: {0}")]
    InferenceEngine(String),

    #[error("Subject matrix not found: {0}")]
    SubjectMatrixNotFound(String),

    #[error("AI model integration error: {0}")]
    AIIntegration(String),

    #[error("AI provider error: {0}")]
    AIProviderError(String),

    #[error("Seed number invalid: {0}")]
    InvalidSeedNumber(String),

    #[error("Node index out of bounds: {0}")]
    NodeIndexOutOfBounds(usize),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Processing error: {0}")]
    Processing(String),
    
    #[error("Storage error: {0}")]
    Storage(String),
    
    // Orchestration errors
    #[error("Meta orchestration failed: {message} {context}")]
    MetaOrchestration { message: String, context: ErrorContext },
    
    #[error("ASI orchestration failed: {message} {context}")]
    ASIOrchestration { message: String, context: ErrorContext },
    
    #[error("Flux orchestration failed: {message} {context}")]
    FluxOrchestration { message: String, context: ErrorContext },
    
    // Sacred geometry errors
    #[error("Sacred position validation failed at position {position}: {reason}")]
    SacredPositionError { position: u8, reason: String },
    
    #[error("Signal strength too low: {signal:.2} < {threshold:.2} {context}")]
    WeakSignalError { signal: f32, threshold: f32, context: ErrorContext },
    
    #[error("Vortex cycle error: {message} {context}")]
    VortexCycleError { message: String, context: ErrorContext },
    
    // Confidence Lake errors
    #[error("Confidence Lake error: {0}")]
    ConfidenceLake(String),
    
    #[error("Lake query failed: {message} {context}")]
    LakeQuery { message: String, context: ErrorContext },
    
    // Training errors
    #[error("Training error: {message} {context}")]
    Training { message: String, context: ErrorContext },
    
    #[error("Learning rate invalid: {rate} (must be 0.0-1.0)")]
    InvalidLearningRate { rate: f64 },
    
    // Routing errors
    #[error("Routing failed: {strategy} strategy could not process input {context}")]
    RoutingError { strategy: String, context: ErrorContext },
    
    // Complexity analysis errors
    #[error("Complexity analysis failed: {0}")]
    ComplexityAnalysis(String),
    
    // Fusion errors
    #[error("Result fusion failed at position {position}: {reason}")]
    FusionError { position: u8, reason: String },
    
    #[error("Tract inference error: {0}")]
    #[cfg(feature = "tract")]
    TractError(String),
    
    #[error("ndarray shape error: {0}")]
    #[cfg(feature = "tract")]
    NdarrayShapeError(String),
    
    // Generic with context
    #[error("{message} {context}")]
    WithContext { message: String, context: ErrorContext },
}

// Error recovery strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryStrategy {
    /// Retry the operation
    Retry,
    /// Use fallback/default value
    Fallback,
    /// Propagate error to caller
    Propagate,
    /// Ignore and continue
    Ignore,
}

impl SpatialVortexError {
    /// Get recommended recovery strategy for this error
    pub fn recovery_strategy(&self) -> RecoveryStrategy {
        match self {
            // Transient errors - retry
            SpatialVortexError::HttpClient(_) => RecoveryStrategy::Retry,
            SpatialVortexError::Redis(_) => RecoveryStrategy::Retry,
            
            // Low signal - fallback to simpler method
            SpatialVortexError::WeakSignalError { .. } => RecoveryStrategy::Fallback,
            
            // Routing failures - try alternative route
            SpatialVortexError::RoutingError { .. } => RecoveryStrategy::Fallback,
            
            // Sacred position errors - propagate (important)
            SpatialVortexError::SacredPositionError { .. } => RecoveryStrategy::Propagate,
            
            // Training errors - ignore and continue
            SpatialVortexError::Training { .. } => RecoveryStrategy::Ignore,
            
            // Most others - propagate
            _ => RecoveryStrategy::Propagate,
        }
    }
    
    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(self.recovery_strategy(), RecoveryStrategy::Retry)
    }
    
    /// Check if error occurred at sacred position
    pub fn is_at_sacred_position(&self) -> bool {
        match self {
            SpatialVortexError::SacredPositionError { .. } => true,
            SpatialVortexError::MetaOrchestration { context, .. } => context.sacred_position,
            SpatialVortexError::ASIOrchestration { context, .. } => context.sacred_position,
            SpatialVortexError::FluxOrchestration { context, .. } => context.sacred_position,
            SpatialVortexError::WeakSignalError { context, .. } => context.sacred_position,
            SpatialVortexError::VortexCycleError { context, .. } => context.sacred_position,
            SpatialVortexError::WithContext { context, .. } => context.sacred_position,
            _ => false,
        }
    }
    
    /// Get flux position if available
    pub fn flux_position(&self) -> Option<u8> {
        match self {
            SpatialVortexError::SacredPositionError { position, .. } => Some(*position),
            SpatialVortexError::FusionError { position, .. } => Some(*position),
            SpatialVortexError::MetaOrchestration { context, .. } => context.flux_position,
            SpatialVortexError::ASIOrchestration { context, .. } => context.flux_position,
            SpatialVortexError::FluxOrchestration { context, .. } => context.flux_position,
            SpatialVortexError::WeakSignalError { context, .. } => context.flux_position,
            SpatialVortexError::VortexCycleError { context, .. } => context.flux_position,
            SpatialVortexError::WithContext { context, .. } => context.flux_position,
            _ => None,
        }
    }
    
    /// Add context to any error
    pub fn with_context(self, context: ErrorContext) -> Self {
        match self {
            // If already has context, merge
            SpatialVortexError::MetaOrchestration { message, .. } => {
                SpatialVortexError::MetaOrchestration { message, context }
            }
            SpatialVortexError::ASIOrchestration { message, .. } => {
                SpatialVortexError::ASIOrchestration { message, context }
            }
            SpatialVortexError::FluxOrchestration { message, .. } => {
                SpatialVortexError::FluxOrchestration { message, context }
            }
            // Wrap others
            other => SpatialVortexError::WithContext {
                message: other.to_string(),
                context,
            },
        }
    }
}

// Tract error conversions
#[cfg(feature = "tract")]
impl From<tract_onnx::prelude::TractError> for SpatialVortexError {
    fn from(err: tract_onnx::prelude::TractError) -> Self {
        SpatialVortexError::TractError(err.to_string())
    }
}

#[cfg(feature = "tract")]
impl From<ndarray::ShapeError> for SpatialVortexError {
    fn from(err: ndarray::ShapeError) -> Self {
        SpatialVortexError::NdarrayShapeError(err.to_string())
    }
}
