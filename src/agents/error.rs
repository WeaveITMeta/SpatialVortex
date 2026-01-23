//! Error types for the coding agent

use std::fmt;

pub type Result<T> = std::result::Result<T, AgentError>;

#[derive(Debug)]
pub enum AgentError {
    /// Language detection failed
    LanguageDetectionFailed(String),
    
    /// Code execution failed
    ExecutionFailed(String),
    
    /// Docker error
    DockerError(String),
    
    /// Code compilation error
    CompilationError(String),
    
    /// Timeout error
    Timeout(String),
    
    /// Unsupported language
    UnsupportedLanguage(String),
    
    /// LLM generation error
    GenerationError(String),
    
    /// Symbolic math error
    SymbolicError(String),
    
    /// SpatialVortex integration error
    FluxError(String),
    
    /// I/O error
    IoError(std::io::Error),
    
    /// Max attempts exceeded
    MaxAttemptsExceeded,
}

impl fmt::Display for AgentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AgentError::LanguageDetectionFailed(msg) => {
                write!(f, "Language detection failed: {}", msg)
            }
            AgentError::ExecutionFailed(msg) => {
                write!(f, "Code execution failed: {}", msg)
            }
            AgentError::DockerError(msg) => {
                write!(f, "Docker error: {}", msg)
            }
            AgentError::CompilationError(msg) => {
                write!(f, "Compilation error: {}", msg)
            }
            AgentError::Timeout(msg) => {
                write!(f, "Timeout: {}", msg)
            }
            AgentError::UnsupportedLanguage(lang) => {
                write!(f, "Unsupported language: {}", lang)
            }
            AgentError::GenerationError(msg) => {
                write!(f, "LLM generation error: {}", msg)
            }
            AgentError::SymbolicError(msg) => {
                write!(f, "Symbolic math error: {}", msg)
            }
            AgentError::FluxError(msg) => {
                write!(f, "SpatialVortex flux error: {}", msg)
            }
            AgentError::IoError(e) => {
                write!(f, "I/O error: {}", e)
            }
            AgentError::MaxAttemptsExceeded => {
                write!(f, "Maximum correction attempts exceeded")
            }
        }
    }
}

impl std::error::Error for AgentError {}

impl From<std::io::Error> for AgentError {
    fn from(err: std::io::Error) -> Self {
        AgentError::IoError(err)
    }
}
