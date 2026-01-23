//! Additional production API endpoints
//! ONNX, ASI, and Confidence Lake integration

use actix_web::{web, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::models::ELPValues;
use crate::ai::orchestrator::{ASIOrchestrator, ExecutionMode};
use crate::ai::{session_api, collaboration};

// ============================================================================
// ONNX & ASI Integration Endpoints
// ============================================================================

/// Request for ONNX embedding generation
#[derive(Debug, Deserialize)]
pub struct OnnxEmbedRequest {
    pub text: String,
    pub include_sacred_geometry: Option<bool>,
}

/// Suggest execution mode for ASI based on input complexity
///
/// # Endpoint
/// GET /api/v1/ml/asi/mode-suggestion?text=...
pub async fn asi_mode_suggestion(
    query: web::Query<ASIModeSuggestionQuery>,
    orchestrator: web::Data<Arc<Mutex<ASIOrchestrator>>>,
) -> ActixResult<HttpResponse> {
    let asi = orchestrator.lock().await;
    let mode = asi.suggest_mode(&query.text);
    let suggested_mode = match mode {
        ExecutionMode::Fast => "fast",
        ExecutionMode::Balanced => "balanced",
        ExecutionMode::Thorough => "thorough",
        ExecutionMode::Reasoning => "reasoning",
    };
    Ok(HttpResponse::Ok().json(ASIModeSuggestionResponse { suggested_mode: suggested_mode.to_string() }))
}

#[derive(Debug, Deserialize)]
pub struct ASIModeSuggestionQuery {
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct ASIModeSuggestionResponse {
    pub suggested_mode: String,
}

/// Response with ONNX embedding
#[derive(Debug, Serialize)]
pub struct OnnxEmbedResponse {
    pub text: String,
    pub embedding: Vec<f32>,
    pub embedding_dim: usize,
    pub confidence: Option<f32>,
    pub elp_channels: Option<ELPValues>,
}

/// Request for ASI inference
#[derive(Debug, Deserialize)]
pub struct ASIInferenceRequest {
    pub text: String,
    /// Execution mode: "fast", "balanced", or "thorough"
    pub mode: Option<String>,
}

/// Response from ASI inference
#[derive(Debug, Serialize)]
pub struct ASIInferenceResultResponse {
    pub text: String,
    pub flux_position: u8,
    pub position_archetype: String,
    pub elp_values: ELPValues,
    pub confidence: f64,
    pub lake_worthy: bool,
    pub interpretation: String,
}

/// ONNX embedding endpoint - Generate embeddings with optional sacred geometry
///
/// # Endpoint
/// POST /api/v1/ml/embed
///
/// # Request Body
/// ```json
/// {
///   "text": "Hello world",
///   "include_sacred_geometry": true
/// }
/// ```
///
/// # Response
/// ```json
/// {
///   "text": "Hello world",
///   "embedding": [0.1, 0.2, ...],
///   "embedding_dim": 384,
///   "confidence": 0.75,
///   "elp_channels": {"ethos": 6.5, "logos": 7.0, "pathos": 5.5}
/// }
/// ```
#[cfg(feature = "onnx")]
pub async fn onnx_embed(
    req: web::Json<OnnxEmbedRequest>,
) -> ActixResult<HttpResponse> {
    // TODO: Load ONNX engine from app state
    // For production, integrate with shared ONNX engine
    // use crate::ml::inference::onnx_runtime::OnnxInferenceEngine;
    
    let include_geometry = req.include_sacred_geometry.unwrap_or(true);
    
    let response = if include_geometry {
        // With sacred geometry transformation
        OnnxEmbedResponse {
            text: req.text.clone(),
            embedding: vec![0.0; 384], // TODO: Real embedding
            embedding_dim: 384,
            confidence: Some(0.75),
            elp_channels: Some(ELPValues {
                ethos: 6.5,
                logos: 7.0,
                pathos: 5.5,
            }),
        }
    } else {
        // Just embedding
        OnnxEmbedResponse {
            text: req.text.clone(),
            embedding: vec![0.0; 384], // TODO: Real embedding
            embedding_dim: 384,
            confidence: None,
            elp_channels: None,
        }
    };
    
    Ok(HttpResponse::Ok().json(response))
}

#[cfg(not(feature = "onnx"))]
pub async fn onnx_embed(
    _req: web::Json<OnnxEmbedRequest>,
) -> ActixResult<HttpResponse> {
    Ok(HttpResponse::ServiceUnavailable().json(serde_json::json!({
        "error": "ONNX feature not enabled",
        "message": "Compile with --features onnx to enable ML inference"
    })))
}

/// Helper function to get position archetype name
fn get_position_archetype(position: u8, is_sacred: bool) -> String {
    match position {
        0 => "Void/Center".to_string(),
        1 => "Object/Beginning".to_string(),
        2 => "Forces/Polarity".to_string(),
        3 if is_sacred => "⭐ Creative Trinity (Sacred)".to_string(),
        3 => "Law/Structure".to_string(),
        4 => "Value/Worth".to_string(),
        5 => "Unit/Measure".to_string(),
        6 if is_sacred => "⭐ Harmonic Balance (Sacred)".to_string(),
        6 => "Anti-Matter/Negation".to_string(),
        7 => "Assembly/Construction".to_string(),
        8 => "Constraints/Limits".to_string(),
        9 if is_sacred => "⭐ Divine Completion (Sacred)".to_string(),
        9 => "Material/Manifestation".to_string(),
        _ => "Unknown".to_string(),
    }
}

/// Helper function to generate interpretation
fn generate_interpretation(
    confidence: f32,
    is_sacred: bool,
    flux_position: u8,
) -> String {
    let quality = if confidence >= 0.7 {
        "very high"
    } else if confidence >= 0.5 {
        "high"
    } else {
        "moderate"
    };
    
    let sacred_note = if is_sacred {
        format!(" Sacred position {} detected - enhanced by geometric intelligence.", flux_position)
    } else {
        String::new()
    };
    
    format!(
        "ASI analysis complete with {} confidence ({:.1}%).{}",
        quality,
        confidence * 100.0,
        sacred_note
    )
}

/// ASI inference endpoint - Full sacred geometry pipeline
///
/// # Endpoint
/// POST /api/v1/ml/asi/infer
///
/// # Request Body
/// ```json
/// {
///   "text": "Truth and justice prevail",
///   "mode": "balanced"
/// }
/// ```
///
/// # Response
/// ```json
/// {
///   "text": "Truth and justice prevail",
///   "flux_position": 9,
///   "position_archetype": "⭐ Divine Completion (Sacred)",
///   "elp_values": {"ethos": 8.5, "logos": 7.5, "pathos": 5.0},
///   "confidence": 0.90,
///   "lake_worthy": true,
///   "interpretation": "ASI analysis complete with very high confidence..."
/// }
/// ```
pub async fn asi_inference(
    req: web::Json<ASIInferenceRequest>,
    orchestrator: web::Data<Arc<Mutex<ASIOrchestrator>>>,
) -> ActixResult<HttpResponse> {
    let dev = std::env::var("DEVELOPMENT_MODE").unwrap_or_default() == "true";
    let mode = if dev {
        ExecutionMode::Fast
    } else {
        match req.mode.as_deref() {
            Some("fast") => ExecutionMode::Fast,
            Some("thorough") => ExecutionMode::Thorough,
            _ => ExecutionMode::Balanced,
        }
    };
    
    // Get orchestrator instance
    let mut asi = orchestrator.lock().await;
    
    // Run ASI inference
    let result = asi.process(&req.text, mode)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(
            format!("ASI inference failed: {}", e)
        ))?;
    
    // Map to API response format
    let response = ASIInferenceResultResponse {
        text: req.text.clone(),
        flux_position: result.flux_position,
        position_archetype: get_position_archetype(result.flux_position, result.is_sacred),
        elp_values: ELPValues {
            ethos: result.elp.ethos as f32,
            logos: result.elp.logos as f32,
            pathos: result.elp.pathos as f32,
        },
        confidence: result.confidence as f64,
        lake_worthy: result.confidence >= 0.6, // Confidence Lake threshold
        interpretation: generate_interpretation(
            result.confidence,
            result.is_sacred,
            result.flux_position,
        ),
    };
    
    Ok(HttpResponse::Ok().json(response))
}

/// ParallelFusion v0.8.4 unified process endpoint
///
/// # Endpoint
/// POST /api/v1/process
///
/// # Request Body
/// ```json
/// {
///   "input": "What is the meaning of life?",
///   "mode": "balanced",
///   "timeout_ms": 30000
/// }
/// ```
///
/// # Response
/// ```json
/// {
///   "result": "Answer with 97-99% accuracy",
///   "confidence": 0.98,
///   "flux_position": 6,
///   "elp": { "ethos": 7.5, "logos": 8.2, "pathos": 6.3 },
///   "confidence": 0.95,
///   "sacred_boost": true,
///   "metadata": {
///     "mode": "balanced",
///     "strategy": "Ensemble",
///     "orchestrators_used": "Fusion",
///     "models_used": ["ASI", "Runtime"],
///     "both_succeeded": true
///   },
///   "metrics": {
///     "duration_ms": 350,
///     "processing_time_ms": 345
///   }
/// }
/// ```
// TEMPORARY: ParallelFusion disabled
/*
pub async fn parallel_fusion_process(
    req: web::Json<crate::ai::unified_api::UnifiedRequest>,
    state: web::Data<crate::ai::api::AppState>,
) -> ActixResult<HttpResponse> {
    use crate::ai::unified_api::{UnifiedResponse, ResponseMetadata, ResponseMetrics};
    use std::time::Instant;
    
    let start = Instant::now();
    
    // Validate request
    if let Err(e) = req.validate() {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "ValidationError",
            "message": e,
        })));
    }
    
    // Process with ParallelFusion
    let fusion = state.parallel_fusion.read().await;
    
    match fusion.process(&req.input).await {
        Ok(result) => {
            let duration = start.elapsed();
            
            // Convert to UnifiedResponse
            let response = UnifiedResponse {
                result: result.content,
                confidence: result.confidence,
                flux_position: result.flux_position,
                elp: result.elp,
                confidence: result.confidence,
                sacred_boost: result.sacred_boost,
                metadata: ResponseMetadata {
                    mode: Some(fusion.get_config().await.asi_mode),
                    strategy: format!("{:?}", result.metadata.algorithm),
                    orchestrators_used: "Fusion".to_string(),
                    vortex_cycles: 0,
                    models_used: vec!["ASI".to_string(), "Runtime".to_string()],
                    confidence_lake_hit: false,
                    consensus_achieved: result.metadata.both_succeeded,
                    stored_to_lake: false,
                },
                metrics: ResponseMetrics {
                    duration_ms: duration.as_millis() as u64,
                    inference_ms: Some(result.duration_ms),
                    consensus_ms: None,
                    lake_query_ms: None,
                    tokens_used: None,
                    cpu_usage: None,
                    memory_bytes: None,
                },
                api_version: crate::ai::unified_api::ApiVersion::V1,
            };
            
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => {
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "FusionError",
                "message": format!("ParallelFusion failed: {}", e),
            })))
        }
    }
}
*/

/// Confidence Lake status endpoint
///
/// # Endpoint
/// GET /api/v1/storage/confidence-lake/status
///
/// # Response
/// ```json
/// {
///   "status": "ready",
///   "encryption_enabled": true,
///   "total_entries": 42,
///   "signal_threshold": 0.6,
///   "used_space_mb": 12.5,
///   "available_space_mb": 87.5
/// }
/// ```
#[cfg(feature = "lake")]
pub async fn confidence_lake_status() -> ActixResult<HttpResponse> {
    // TODO: Integrate with actual Confidence Lake from app state
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "ready",
        "encryption_enabled": true,
        "total_entries": 0,
        "signal_threshold": 0.6,
        "used_space_mb": 0.0,
        "available_space_mb": 100.0,
    })))
}

#[cfg(not(feature = "lake"))]
pub async fn confidence_lake_status() -> ActixResult<HttpResponse> {
    Ok(HttpResponse::ServiceUnavailable().json(serde_json::json!({
        "error": "Confidence Lake feature not enabled",
        "message": "Compile with --features lake to enable encrypted storage"
    })))
}

/// Voice pipeline status endpoint
///
/// # Endpoint  
/// GET /api/v1/voice/status
///
/// # Response
/// ```json
/// {
///   "status": "ready",
///   "sample_rate": 44100,
///   "fft_enabled": true,
///   "elp_mapping_enabled": true
/// }
/// ```
#[cfg(feature = "voice")]
pub async fn voice_pipeline_status() -> ActixResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "ready",
        "sample_rate": 44100,
        "fft_enabled": true,
        "elp_mapping_enabled": true,
    })))
}

#[cfg(not(feature = "voice"))]
pub async fn voice_pipeline_status() -> ActixResult<HttpResponse> {
    Ok(HttpResponse::ServiceUnavailable().json(serde_json::json!({
        "error": "Voice feature not enabled",
        "message": "Compile with --features voice to enable voice processing"
    })))
}

/// Get ASI performance metrics (Phase 3)
///
/// # Endpoint
/// GET /api/v1/ml/asi/metrics
///
/// # Response
/// ```json
/// {
///   "total_inferences": 1250,
///   "fast_mode_avg_time": 45.2,
///   "balanced_mode_avg_time": 125.8,
///   "thorough_mode_avg_time": 287.3,
///   "avg_confidence": 0.87,
///   "consensus_rate": 342
/// }
/// ```
pub async fn asi_metrics(
    orchestrator: web::Data<Arc<Mutex<ASIOrchestrator>>>,
) -> ActixResult<HttpResponse> {
    let asi = orchestrator.lock().await;
    let metrics = asi.get_metrics();
    Ok(HttpResponse::Ok().json(metrics))
}

/// Get ASI adaptive weights (Phase 3)
///
/// # Endpoint
/// GET /api/v1/ml/asi/weights
///
/// # Response
/// ```json
/// {
///   "geometric_weight": 0.32,
///   "ml_weight": 0.51,
///   "consensus_weight": 0.17,
///   "learning_rate": 0.01
/// }
/// ```
pub async fn asi_weights(
    orchestrator: web::Data<Arc<Mutex<ASIOrchestrator>>>,
) -> ActixResult<HttpResponse> {
    let asi = orchestrator.lock().await;
    let weights = asi.get_weights().await;
    Ok(HttpResponse::Ok().json(weights))
}

/// Configure production API routes (NO SCOPE - add to existing /api/v1)
///
/// Sets up all ML, storage, voice, chat, RAG, and monitoring endpoints with shared app state.
/// The ASI orchestrator is automatically injected via actix-web's Data extractor.
/// 
/// **IMPORTANT**: This function does NOT create a new scope. It adds routes to
/// the existing configuration, which should already be inside /api/v1 scope.
pub fn configure_production_routes(cfg: &mut web::ServiceConfig) {
    tracing::info!("🚀 Configuring production API routes...");
    cfg
        // ML endpoints
        .route("/ml/embed", web::post().to(onnx_embed))
        .route("/ml/asi/infer", web::post().to(asi_inference))
        // Session Memory routes
        .service(session_api::create_session)
        .service(session_api::get_session)
        .service(session_api::list_sessions)
        .service(session_api::add_message)
        .service(session_api::get_messages)
        .service(session_api::update_title)
        .service(session_api::update_summary)
        .service(session_api::archive_session)
        .service(session_api::delete_session)
        .service(session_api::search_sessions)
        .service(session_api::get_stats)
        // Collaboration routes
        .service(collaboration::join_session)
        .service(collaboration::leave_session)
        .service(collaboration::update_cursor)
        .service(collaboration::get_session)
        .service(collaboration::list_sessions)
        .route("/ml/asi/metrics", web::get().to(asi_metrics))
        .route("/ml/asi/weights", web::get().to(asi_weights))
        // TEMPORARY: ParallelFusion disabled
        // .route("/process", web::post().to(parallel_fusion_process))
        // Storage endpoints
        .route("/storage/confidence-lake/status", web::get().to(confidence_lake_status))
        // Voice endpoints
        .route("/voice/status", web::get().to(voice_pipeline_status))
        .service(super::whisper_api::transcribe_audio);
    
    // Configure extended chat endpoints
    tracing::info!("🗨️  Configuring chat management endpoints...");
    super::chat_endpoints::configure_chat_routes(cfg);
    
    // Configure RAG endpoints
    #[cfg(feature = "rag")]
    {
        tracing::info!("📚 Configuring RAG (Retrieval-Augmented Generation) endpoints...");
        super::rag_endpoints::configure_rag_routes(cfg);
    }
    
    // Configure Canvas/Workspace endpoints
    tracing::info!("🎨 Configuring Canvas/Workspace endpoints...");
    super::canvas_api::configure_canvas_routes(cfg);
    
    // Configure Code Execution endpoints
    tracing::info!("💻 Configuring Code Execution endpoints...");
    super::code_execution_api::configure_code_execution_routes(cfg);
    
    // Configure Session Memory endpoints
    tracing::info!("🧠 Configuring Session Memory endpoints...");
    super::session_api::configure_session_routes(cfg);
    
    // Configure monitoring endpoints  
    tracing::info!("📊 Configuring monitoring and observability endpoints...");
    super::monitoring_endpoints::configure_monitoring_routes(cfg);
    
    // Configure AGI endpoints
    tracing::info!("🧠 Configuring AGI (Flux Reasoning) endpoints...");
    super::agi_api::configure_agi_routes(cfg);
    
    tracing::info!("✅ All production routes configured (36+ endpoints)");
}
