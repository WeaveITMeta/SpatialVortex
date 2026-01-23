//! Consciousness API Endpoints for Frontend Integration
//!
//! Provides REST API endpoints for the consciousness simulation system,
//! including thinking, analytics, streaming, and memory palace features.

use actix_web::{web, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::consciousness::ConsciousnessSimulator;

#[cfg(feature = "persistence")]
use crate::consciousness::MemoryPalace;

/// Shared consciousness simulator state
pub type ConsciousnessState = Arc<RwLock<ConsciousnessSimulator>>;

// ============================================================================
// Request/Response Models
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ThinkRequest {
    /// Question or prompt for the consciousness to process
    pub question: String,
    
    /// Optional session ID for conversation continuity
    pub session_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ThinkResponse {
    /// Generated answer
    pub answer: String,
    
    /// Integrated Information (Φ)
    pub phi: f64,
    
    /// Current mental state
    pub mental_state: String,
    
    /// Confidence level (0.0-1.0)
    pub confidence: f64,
    
    /// ELP balance (Ethos, Logos, Pathos)
    pub elp_balance: ELPBalance,
    
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct ELPBalance {
    pub ethos: f64,
    pub logos: f64,
    pub pathos: f64,
}

#[derive(Debug, Deserialize)]
pub struct EnableLearningRequest {
    /// Configuration for background learning
    pub config: Option<BackgroundLearningConfigDTO>,
}

#[derive(Debug, Deserialize)]
pub struct BackgroundLearningConfigDTO {
    /// Learning interval in seconds
    pub learning_interval_secs: Option<u64>,
    
    /// Enable RAG ingestion
    pub enable_rag_ingestion: Option<bool>,
    
    /// Enable Confidence Lake review
    pub enable_lake_review: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub features: HealthFeatures,
}

#[derive(Debug, Serialize)]
pub struct HealthFeatures {
    pub consciousness: bool,
    pub background_learning: bool,
    pub streaming: bool,
    pub persistence: bool,
    pub postgres_rag: bool,
}

// ============================================================================
// API Endpoints
// ============================================================================

/// Health check endpoint
///
/// GET /api/v1/consciousness/health
#[actix_web::get("/health")]
pub async fn health() -> ActixResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(HealthResponse {
        status: "healthy".to_string(),
        version: "1.6.0".to_string(),
        features: HealthFeatures {
            consciousness: true,
            background_learning: cfg!(feature = "agents"),
            streaming: cfg!(feature = "transport"),
            persistence: cfg!(feature = "persistence"),
            postgres_rag: cfg!(feature = "postgres"),
        },
    }))
}

/// Process a thought/question through the consciousness simulator
///
/// POST /api/v1/consciousness/think
#[actix_web::post("/think")]
pub async fn think(
    req: web::Json<ThinkRequest>,
    sim: web::Data<ConsciousnessState>,
) -> ActixResult<HttpResponse> {
    let start_time = std::time::Instant::now();
    
    let simulator = sim.write().await;
    
    let response = simulator
        .generate_response(&req.question, None)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    let processing_time = start_time.elapsed();
    
    Ok(HttpResponse::Ok().json(ThinkResponse {
        answer: response.answer,
        phi: response.phi,
        mental_state: response.mental_state.to_string(),
        confidence: response.confidence,
        elp_balance: ELPBalance {
            ethos: response.ethos_weight,
            logos: response.logos_weight,
            pathos: response.pathos_weight,
        },
        processing_time_ms: processing_time.as_millis() as u64,
    }))
}

/// Get current analytics snapshot
///
/// GET /api/v1/consciousness/analytics
#[actix_web::get("/analytics")]
pub async fn get_analytics(
    sim: web::Data<ConsciousnessState>,
) -> ActixResult<HttpResponse> {
    let simulator = sim.read().await;
    
    let snapshot = simulator.get_analytics_snapshot().await;
    
    Ok(HttpResponse::Ok().json(snapshot))
}

/// Get session information
///
/// GET /api/v1/consciousness/session
#[actix_web::get("/session")]
pub async fn get_session_info(
    sim: web::Data<ConsciousnessState>,
) -> ActixResult<HttpResponse> {
    let simulator = sim.read().await;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "session_id": simulator.session_id(),
        "session_start": simulator.session_id(), // TODO: Add timestamp
        "consciousness_active": true,
    })))
}

/// Get background learning statistics
///
/// GET /api/v1/consciousness/learning-stats
#[actix_web::get("/learning-stats")]
pub async fn get_learning_stats(
    sim: web::Data<ConsciousnessState>,
) -> ActixResult<HttpResponse> {
    let simulator = sim.read().await;
    
    let stats = simulator.learning_stats().await;
    
    Ok(HttpResponse::Ok().json(stats))
}

/// Check if background learning is active
///
/// GET /api/v1/consciousness/learning-active
#[actix_web::get("/learning-active")]
pub async fn is_learning_active(
    sim: web::Data<ConsciousnessState>,
) -> ActixResult<HttpResponse> {
    let simulator = sim.read().await;
    
    let active = simulator.is_learning_active().await;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "active": active
    })))
}

/// Enable background learning
///
/// POST /api/v1/consciousness/enable-learning
#[actix_web::post("/enable-learning")]
pub async fn enable_learning(
    req: web::Json<EnableLearningRequest>,
    sim: web::Data<ConsciousnessState>,
) -> ActixResult<HttpResponse> {
    let mut simulator = sim.write().await;
    
    // Apply configuration if provided
    if let Some(_config) = &req.config {
        // TODO: Apply custom configuration
        // For now, use defaults
    }
    
    simulator
        .enable_background_learning()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "enabled",
        "message": "Background learning started"
    })))
}

/// Start background learning (if already enabled)
///
/// POST /api/v1/consciousness/start-learning
#[actix_web::post("/start-learning")]
pub async fn start_learning(
    sim: web::Data<ConsciousnessState>,
) -> ActixResult<HttpResponse> {
    let simulator = sim.read().await;
    
    simulator
        .start_background_learning()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "started",
        "message": "Background learning resumed"
    })))
}

/// Stop background learning
///
/// POST /api/v1/consciousness/stop-learning
#[actix_web::post("/stop-learning")]
pub async fn stop_learning(
    sim: web::Data<ConsciousnessState>,
) -> ActixResult<HttpResponse> {
    let simulator = sim.read().await;
    
    simulator.stop_background_learning().await;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "stopped",
        "message": "Background learning stopped"
    })))
}

/// Save consciousness state (Memory Palace)
///
/// POST /api/v1/consciousness/save-state
#[actix_web::post("/save-state")]
pub async fn save_state(
    _sim: web::Data<ConsciousnessState>,
) -> ActixResult<HttpResponse> {
    #[cfg(feature = "persistence")]
    {
        use std::path::Path;
        
        let simulator = _sim.read().await;
        let palace = MemoryPalace::new(Path::new("consciousness_state.json"));
        
        let stats = simulator.learning_stats().await.unwrap_or_default();
        
        palace
            .save_state(
                simulator.session_id().to_string(),
                simulator.meta_monitor(),
                simulator.predictor(),
                simulator.phi_calculator(),
                &stats,
            )
            .await
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
        
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "status": "saved",
            "message": "Consciousness state saved successfully",
            "file": "consciousness_state.json"
        })))
    }
    
    #[cfg(not(feature = "persistence"))]
    {
        Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Persistence feature not enabled",
            "message": "Rebuild with --features persistence"
        })))
    }
}

/// Load consciousness state (Memory Palace)
///
/// POST /api/v1/consciousness/load-state
#[actix_web::post("/load-state")]
pub async fn load_state(
    _sim: web::Data<ConsciousnessState>,
) -> ActixResult<HttpResponse> {
    #[cfg(feature = "persistence")]
    {
        use std::path::Path;
        
        let palace = MemoryPalace::new(Path::new("consciousness_state.json"));
        
        let state = palace
            .load_state()
            .await
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
        
        if let Some(state) = state {
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "status": "loaded",
                "message": "Consciousness state loaded successfully",
                "state": {
                    "version": state.version,
                    "session_id": state.session_id,
                    "phi": state.phi_state.current_phi,
                    "patterns": state.metacognitive_state.pattern_count,
                    "accuracy": state.predictive_state.accuracy,
                }
            })))
        } else {
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "status": "not_found",
                "message": "No previous state found"
            })))
        }
    }
    
    #[cfg(not(feature = "persistence"))]
    {
        Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Persistence feature not enabled",
            "message": "Rebuild with --features persistence"
        })))
    }
}

// ============================================================================
// Configuration
// ============================================================================

/// Configure consciousness API routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1/consciousness")
            .service(health)
            .service(think)
            .service(get_analytics)
            .service(get_session_info)
            .service(get_learning_stats)
            .service(is_learning_active)
            .service(enable_learning)
            .service(start_learning)
            .service(stop_learning)
            .service(save_state)
            .service(load_state),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_health_response_serialization() {
        let health_response = HealthResponse {
            status: "healthy".to_string(),
            version: "1.6.0".to_string(),
            features: HealthFeatures {
                consciousness: true,
                background_learning: true,
                streaming: true,
                persistence: true,
                postgres_rag: true,
            },
        };
        
        let json = serde_json::to_string(&health_response).unwrap();
        assert!(json.contains("healthy"));
        assert!(json.contains("1.6.0"));
    }
}
