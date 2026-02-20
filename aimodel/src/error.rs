//! Error types for Vortex

use thiserror::Error;

pub type Result<T> = std::result::Result<T, VortexError>;

#[derive(Error, Debug)]
pub enum VortexError {
    #[error("Invalid flux matrix configuration: {0}")]
    InvalidFluxMatrix(String),

    #[error("Inference engine error: {0}")]
    InferenceEngine(String),

    #[error("AI integration error: {0}")]
    AIIntegration(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Processing error: {0}")]
    Processing(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Sacred position validation failed at position {position}: {reason}")]
    SacredPositionError { position: u8, reason: String },

    #[error("Signal strength too low: {signal:.2} < {threshold:.2}")]
    WeakSignalError { signal: f32, threshold: f32 },

    #[error("Vortex cycle error: {0}")]
    VortexCycleError(String),

    #[error("Hallucination detected: {0}")]
    HallucinationDetected(String),

    #[error("Energy threshold not met: {energy:.2} < {threshold:.2}")]
    EnergyThresholdError { energy: f32, threshold: f32 },
}
