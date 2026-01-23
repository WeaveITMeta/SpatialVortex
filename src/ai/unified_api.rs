//! Unified API Types
//!
//! Harmonized request/response types for all SpatialVortex endpoints.
//! Provides consistent interface across Meta Orchestrator, ASI, and Flux systems.
//!
//! ## API Versioning
//!
//! - **v1**: Current stable API
//! - **v2**: Future enhancements (planned)
//!
//! ## Example
//!
//! ```no_run
//! use spatial_vortex::ai::unified_api::{UnifiedRequest, UnifiedResponse};
//!
//! let request = UnifiedRequest::builder()
//!     .input("What is consciousness?")
//!     .mode(ExecutionMode::Balanced)
//!     .strategy(RoutingStrategy::Hybrid)
//!     .build();
//! ```

use crate::ai::orchestrator::ExecutionMode;
use crate::ai::meta_orchestrator::RoutingStrategy;
use crate::models::ELPTensor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// API version
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ApiVersion {
    /// Version 1 (current)
    V1,
    /// Version 2 (future)
    V2,
}

impl Default for ApiVersion {
    fn default() -> Self {
        Self::V1
    }
}

/// Unified request for all endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedRequest {
    /// Input text or query
    pub input: String,
    
    /// Execution mode (optional, default: Balanced)
    #[serde(default)]
    pub mode: Option<ExecutionMode>,
    
    /// Routing strategy (optional, default: Hybrid)
    #[serde(default)]
    pub strategy: Option<RoutingStrategy>,
    
    /// Additional context (optional)
    #[serde(default)]
    pub context: Option<Vec<String>>,
    
    /// Filter to sacred positions only (3, 6, 9)
    #[serde(default)]
    pub sacred_only: bool,
    
    /// Minimum confidence threshold (0.0-1.0) - includes signal strength
    #[serde(default)]
    pub min_confidence: Option<f32>,
    
    /// Enable consensus verification
    #[serde(default)]
    pub enable_consensus: bool,
    
    /// Store result in Confidence Lake if high quality
    #[serde(default = "default_true")]
    pub store_if_quality: bool,
    
    /// API version
    #[serde(default)]
    pub api_version: ApiVersion,
    
    /// Client metadata (optional)
    #[serde(default)]
    pub metadata: Option<HashMap<String, String>>,
}

fn default_true() -> bool {
    true
}

impl UnifiedRequest {
    /// Create builder for request
    pub fn builder() -> UnifiedRequestBuilder {
        UnifiedRequestBuilder::default()
    }
    
    /// Validate request
    pub fn validate(&self) -> Result<(), String> {
        if self.input.is_empty() {
            return Err("Input cannot be empty".to_string());
        }
        
        if let Some(min_conf) = self.min_confidence {
            if !(0.0..=1.0).contains(&min_conf) {
                return Err("min_confidence must be between 0.0 and 1.0".to_string());
            }
        }
        
        if let Some(min_signal) = self.min_confidence {
            if !(0.0..=1.0).contains(&min_signal) {
                return Err("min_confidence must be between 0.0 and 1.0".to_string());
            }
        }
        
        Ok(())
    }
}

/// Builder for UnifiedRequest
#[derive(Default)]
pub struct UnifiedRequestBuilder {
    input: Option<String>,
    mode: Option<ExecutionMode>,
    strategy: Option<RoutingStrategy>,
    context: Option<Vec<String>>,
    sacred_only: bool,
    min_confidence: Option<f32>,
    enable_consensus: bool,
    store_if_quality: bool,
    api_version: ApiVersion,
    metadata: Option<HashMap<String, String>>,
}

impl UnifiedRequestBuilder {
    pub fn input(mut self, input: impl Into<String>) -> Self {
        self.input = Some(input.into());
        self
    }
    
    pub fn mode(mut self, mode: ExecutionMode) -> Self {
        self.mode = Some(mode);
        self
    }
    
    pub fn strategy(mut self, strategy: RoutingStrategy) -> Self {
        self.strategy = Some(strategy);
        self
    }
    
    pub fn context(mut self, context: Vec<String>) -> Self {
        self.context = Some(context);
        self
    }
    
    pub fn sacred_only(mut self, sacred: bool) -> Self {
        self.sacred_only = sacred;
        self
    }
    
    pub fn min_confidence(mut self, min: f32) -> Self {
        self.min_confidence = Some(min);
        self
    }
    
    pub fn enable_consensus(mut self, enable: bool) -> Self {
        self.enable_consensus = enable;
        self
    }
    
    pub fn store_if_quality(mut self, store: bool) -> Self {
        self.store_if_quality = store;
        self
    }
    
    pub fn api_version(mut self, version: ApiVersion) -> Self {
        self.api_version = version;
        self
    }
    
    pub fn metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = Some(metadata);
        self
    }
    
    pub fn build(self) -> Result<UnifiedRequest, String> {
        let input = self.input.ok_or("Input is required")?;
        
        let request = UnifiedRequest {
            input,
            mode: self.mode,
            strategy: self.strategy,
            context: self.context,
            sacred_only: self.sacred_only,
            min_confidence: self.min_confidence,
            enable_consensus: self.enable_consensus,
            store_if_quality: self.store_if_quality,
            api_version: self.api_version,
            metadata: self.metadata,
        };
        
        request.validate()?;
        Ok(request)
    }
}

/// Unified response for all endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedResponse {
    /// Result content
    pub result: String,
    
    /// Confidence score and signal strength (0.0-1.0)
    pub confidence: f32,
    
    /// Flux position (0-9)
    pub flux_position: u8,
    
    /// ELP tensor
    pub elp: ELPTensor,
    
    /// Sacred position boost applied
    pub sacred_boost: bool,
    
    /// Processing metadata
    pub metadata: ResponseMetadata,
    
    /// Performance metrics
    pub metrics: ResponseMetrics,
    
    /// API version used
    pub api_version: ApiVersion,
}

/// Response metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    /// Execution mode used
    pub mode: Option<ExecutionMode>,
    
    /// Routing strategy used
    pub strategy: String,
    
    /// Which orchestrator(s) processed this
    pub orchestrators_used: String,
    
    /// Vortex cycles completed
    pub vortex_cycles: u32,
    
    /// Models/experts used
    pub models_used: Vec<String>,
    
    /// Confidence Lake hit
    pub confidence_lake_hit: bool,
    
    /// Consensus achieved
    pub consensus_achieved: bool,
    
    /// Was stored to Confidence Lake
    pub stored_to_lake: bool,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetrics {
    /// Total processing duration (ms)
    pub duration_ms: u64,
    
    /// Inference duration (ms)
    pub inference_ms: Option<u64>,
    
    /// Consensus duration (ms)
    pub consensus_ms: Option<u64>,
    
    /// Lake query duration (ms)
    pub lake_query_ms: Option<u64>,
    
    /// Total tokens used (if applicable)
    pub tokens_used: Option<u32>,
    
    /// CPU usage percentage
    pub cpu_usage: Option<f32>,
    
    /// Memory usage (bytes)
    pub memory_bytes: Option<u64>,
}

/// Batch request for multiple inputs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRequest {
    /// Multiple requests
    pub requests: Vec<UnifiedRequest>,
    
    /// Execute in parallel (default: true)
    #[serde(default = "default_true")]
    pub parallel: bool,
    
    /// Maximum parallelism (default: 10)
    #[serde(default = "default_parallel_limit")]
    pub max_parallel: usize,
}

fn default_parallel_limit() -> usize {
    10
}

/// Batch response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResponse {
    /// Individual responses
    pub responses: Vec<Result<UnifiedResponse, String>>,
    
    /// Total duration (ms)
    pub total_duration_ms: u64,
    
    /// Success count
    pub success_count: usize,
    
    /// Failure count
    pub failure_count: usize,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Service status
    pub status: ServiceStatus,
    
    /// Version information
    pub version: String,
    
    /// Component health
    pub components: HashMap<String, ComponentHealth>,
    
    /// System metrics
    pub metrics: SystemMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServiceStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub status: ServiceStatus,
    pub message: Option<String>,
    pub last_check: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub active_requests: u64,
    pub total_requests: u64,
    pub avg_latency_ms: f64,
    pub error_rate: f32,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f32,
}

/// Error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error type
    pub error_type: String,
    
    /// Error message
    pub message: String,
    
    /// Error details (optional)
    pub details: Option<String>,
    
    /// Flux position if available
    pub flux_position: Option<u8>,
    
    /// Sacred position indicator
    pub sacred_position: bool,
    
    /// Recovery strategy suggested
    pub recovery_strategy: String,
    
    /// Request ID for tracing
    pub request_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_request_builder() {
        let request = UnifiedRequest::builder()
            .input("Test input")
            .mode(ExecutionMode::Balanced)
            .strategy(RoutingStrategy::Hybrid)
            .sacred_only(true)
            .build()
            .unwrap();
        
        assert_eq!(request.input, "Test input");
        assert_eq!(request.mode, Some(ExecutionMode::Balanced));
        assert!(request.sacred_only);
    }
    
    #[test]
    fn test_request_validation() {
        let request = UnifiedRequest::builder()
            .input("")
            .build();
        
        assert!(request.is_err());
    }
    
    #[test]
    fn test_confidence_validation() {
        let request = UnifiedRequest::builder()
            .input("Test")
            .min_confidence(1.5)
            .build();
        
        assert!(request.is_err());
    }
    
    #[test]
    fn test_api_version_default() {
        assert_eq!(ApiVersion::default(), ApiVersion::V1);
    }
}
