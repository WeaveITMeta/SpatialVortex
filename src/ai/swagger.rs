//! Swagger/OpenAPI Documentation
//!
//! Interactive API documentation with Swagger UI

use utoipa::{OpenApi, ToSchema};
use serde::{Deserialize, Serialize};

/// OpenAPI documentation for SpatialVortex API
#[derive(OpenApi)]
#[openapi(
    info(
        title = "SpatialVortex API",
        version = "0.7.0",
        description = "Production API for SpatialVortex - Sacred Geometry & AI Inference (Pure Rust ASI)",
        contact(
            name = "SpatialVortex Team",
            url = "https://github.com/WeaveSolutions/SpatialVortex"
        ),
        license(
            name = "MIT"
        )
    ),
    paths(
        health_check,
        list_subjects,
        reverse_inference,
        forward_inference,
        // Added implemented endpoints
        asi_inference_doc,
        asi_mode_suggestion_doc,
        asi_metrics_doc,
        asi_weights_doc,
        metrics_system_doc,
        metrics_prometheus_doc,
        readiness_doc,
        liveness_doc,
        confidence_lake_status_doc,
        voice_status_doc,
    ),
    components(
        schemas(
            HealthResponse,
            InferenceEngineStats,
            ReverseInferenceRequest,
            ReverseInferenceResponse,
            ForwardInferenceRequest,
            ForwardInferenceResponse,
            SubjectInfo,
        )
    ),
    tags(
        (name = "health", description = "Health check and status endpoints"),
        (name = "inference", description = "AI inference operations"),
        (name = "subjects", description = "Subject management"),
        (name = "flux", description = "Flux matrix operations"),
        (name = "sacred-geometry", description = "Sacred geometry analysis"),
    ),
    servers(
        (url = "http://localhost:8080", description = "Local development server"),
        (url = "https://api.spatialvortex.ai", description = "Production server")
    )
)]
pub struct ApiDoc;

// ============================================================================
// Schema Definitions
// ============================================================================

/// Health check response
#[derive(Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    /// Overall system status
    #[schema(example = "healthy")]
    pub status: String,
    
    /// API version
    #[schema(example = "0.1.0")]
    pub version: String,
    
    /// Inference engine statistics
    pub inference_engine_stats: InferenceEngineStats,
    
    /// Database connection status
    #[schema(example = "healthy")]
    pub database_status: String,
    
    /// Cache connection status
    #[schema(example = "healthy")]
    pub cache_status: String,
}

/// Inference engine statistics
#[derive(Serialize, Deserialize, ToSchema)]
pub struct InferenceEngineStats {
    /// Total flux matrices loaded
    #[schema(example = 5)]
    pub total_matrices: usize,
    
    /// Cached inferences count
    #[schema(example = 42)]
    pub cached_inferences: usize,
    
    /// Available subjects
    pub available_subjects: Vec<String>,
}

/// Request for reverse inference (seeds → meanings)
#[derive(Serialize, Deserialize, ToSchema)]
pub struct ReverseInferenceRequest {
    /// Seed numbers to infer meanings from
    #[schema(example = json!([3, 6, 9]))]
    pub seed_numbers: Vec<u64>,
    
    /// Subject filter: "all", "specific", "category:X"
    #[schema(example = "all")]
    pub subject_filter: String,
    
    /// Include synonyms in results
    #[schema(example = true)]
    pub include_synonyms: Option<bool>,
}

/// Response from reverse inference
#[derive(Serialize, Deserialize, ToSchema)]
pub struct ReverseInferenceResponse {
    /// Unique inference ID
    #[schema(example = "b8ae13f2-b9e1-459f-8aa8-3d2440093421")]
    pub inference_id: String,
    
    /// Inferred meanings
    pub inferred_meanings: Vec<String>,
    
    /// Confidence score (0.0 - 1.0)
    #[schema(example = 0.85)]
    pub confidence_score: f64,
    
    /// Processing time in milliseconds
    #[schema(example = 1)]
    pub processing_time_ms: u64,
    
    /// Moral alignment summary
    #[schema(example = "Constructive: 2, Destructive: 0, Neutral: 1")]
    pub moral_alignment_summary: String,
}

/// Request for forward inference (meanings → seeds)
#[derive(Serialize, Deserialize, ToSchema)]
pub struct ForwardInferenceRequest {
    /// Target meanings to tokenize
    #[schema(example = json!(["love", "truth", "wisdom"]))]
    pub target_meanings: Vec<String>,
    
    /// Subject filter
    #[schema(example = "all")]
    pub subject_filter: String,
    
    /// Maximum results to return
    #[schema(example = 10)]
    pub max_results: Option<usize>,
}

/// Response from forward inference
#[derive(Serialize, Deserialize, ToSchema)]
pub struct ForwardInferenceResponse {
    /// Unique tokenization ID
    pub tokenization_id: String,
    
    /// Matched seed numbers
    pub matched_seeds: Vec<u64>,
    
    /// Processing time in milliseconds
    #[schema(example = 2)]
    pub processing_time_ms: u64,
}

/// Subject information
#[derive(Serialize, Deserialize, ToSchema)]
pub struct SubjectInfo {
    /// Subject name
    #[schema(example = "ethics")]
    pub name: String,
    
    /// Subject description
    #[schema(example = "Moral philosophy and ethical reasoning")]
    pub description: Option<String>,
    
    /// Number of entries
    #[schema(example = 100)]
    pub entry_count: usize,
}

// ============================================================================
// Path Definitions
// ============================================================================

/// Health check endpoint
///
/// Returns the current status of all system components
#[utoipa::path(
    get,
    path = "/api/v1/health",
    tag = "health",
    responses(
        (status = 200, description = "System is healthy", body = HealthResponse),
        (status = 503, description = "System is unhealthy")
    )
)]
#[allow(dead_code)]  // Documentation-only function for OpenAPI schema
async fn health_check() {}

/// List available subjects
///
/// Returns all subjects currently loaded in the system
#[utoipa::path(
    get,
    path = "/api/v1/subjects",
    tag = "subjects",
    responses(
        (status = 200, description = "List of subjects", body = Vec<SubjectInfo>)
    )
)]
#[allow(dead_code)]  // Documentation-only function for OpenAPI schema
async fn list_subjects() {}

/// Reverse inference
///
/// Converts seed numbers to their semantic meanings using sacred geometry
#[utoipa::path(
    post,
    path = "/api/v1/inference/reverse",
    tag = "inference",
    request_body = ReverseInferenceRequest,
    responses(
        (status = 200, description = "Inference completed", body = ReverseInferenceResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Inference failed")
    )
)]
#[allow(dead_code)]  // Documentation-only function for OpenAPI schema
async fn reverse_inference() {}

/// Forward inference
///
/// Converts semantic meanings to seed numbers using sacred geometry
#[utoipa::path(
    post,
    path = "/api/v1/inference/forward",
    tag = "inference",
    request_body = ForwardInferenceRequest,
    responses(
        (status = 200, description = "Inference completed", body = ForwardInferenceResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Inference failed")
    )
)]
#[allow(dead_code)]  // Documentation-only function for OpenAPI schema
async fn forward_inference() {}

// ============================================================================
// Additional Implemented Paths (Doc-only for OpenAPI)
// ============================================================================

/// ASI inference (doc-only)
#[utoipa::path(
    post,
    path = "/api/v1/ml/asi/infer",
    tag = "inference",
    request_body = super::endpoints::ASIInferenceRequest,
    responses(
        (status = 200, description = "ASI inference completed"),
        (status = 500, description = "ASI inference failed")
    )
)]
#[allow(dead_code)]
async fn asi_inference_doc() {}

/// ASI mode suggestion (doc-only)
#[utoipa::path(
    get,
    path = "/api/v1/ml/asi/mode-suggestion",
    tag = "inference",
    params(
        ("text" = String, Query, description = "Input text to analyze"),
    ),
    responses(
        (status = 200, description = "Suggested mode returned")
    )
)]
#[allow(dead_code)]
async fn asi_mode_suggestion_doc() {}

/// ASI metrics (doc-only)
#[utoipa::path(
    get,
    path = "/api/v1/ml/asi/metrics",
    tag = "metrics",
    responses((status = 200, description = "ASI metrics returned"))
)]
#[allow(dead_code)]
async fn asi_metrics_doc() {}

/// ASI weights (doc-only)
#[utoipa::path(
    get,
    path = "/api/v1/ml/asi/weights",
    tag = "metrics",
    responses((status = 200, description = "ASI weights returned"))
)]
#[allow(dead_code)]
async fn asi_weights_doc() {}

/// System metrics (doc-only)
#[utoipa::path(
    get,
    path = "/api/v1/metrics/system",
    tag = "metrics",
    responses((status = 200, description = "System metrics returned"))
)]
#[allow(dead_code)]
async fn metrics_system_doc() {}

/// Prometheus metrics (doc-only)
#[utoipa::path(
    get,
    path = "/api/v1/metrics/prometheus",
    tag = "metrics",
    responses((status = 200, description = "Prometheus metrics returned"))
)]
#[allow(dead_code)]
async fn metrics_prometheus_doc() {}

/// Readiness probe (doc-only)
#[utoipa::path(
    get,
    path = "/api/v1/health/readiness",
    tag = "health",
    responses((status = 200, description = "Service ready"), (status = 503, description = "Degraded"))
)]
#[allow(dead_code)]
async fn readiness_doc() {}

/// Liveness probe (doc-only)
#[utoipa::path(
    get,
    path = "/api/v1/health/liveness",
    tag = "health",
    responses((status = 200, description = "Service alive"))
)]
#[allow(dead_code)]
async fn liveness_doc() {}

/// Confidence Lake status (doc-only)
#[utoipa::path(
    get,
    path = "/api/v1/storage/confidence-lake/status",
    tag = "storage",
    responses((status = 200, description = "Confidence Lake status returned"))
)]
#[allow(dead_code)]
async fn confidence_lake_status_doc() {}

/// Voice pipeline status (doc-only)
#[utoipa::path(
    get,
    path = "/api/v1/voice/status",
    tag = "voice",
    responses((status = 200, description = "Voice pipeline status returned"))
)]
#[allow(dead_code)]
async fn voice_status_doc() {}
