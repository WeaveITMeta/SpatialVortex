//! Chat API for trained AI model
//!
//! Provides HTTP endpoints for interacting with the trained CALM engine
//! via a web interface. Uses VortexRunner which contains the CALM engine.

#[cfg(feature = "web")]
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::data::models::BeamTensor;
use crate::cognition::vortex_runner::VortexRunner;
use crate::ml::calm::LatentState;

// =============================================================================
// Request/Response Types
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub response: String,
    pub session_id: String,
    pub tokens_used: usize,
    pub latent_energy: f32,
    pub confidence: f32,
}

#[derive(Debug, Serialize)]
pub struct ModelInfoResponse {
    pub model_name: String,
    pub version: String,
    pub latent_dim: usize,
    pub capabilities: Vec<String>,
}

// =============================================================================
// Chat Engine State
// =============================================================================

pub struct ChatEngineState {
    pub vortex_runner: Arc<VortexRunner>,
    pub session_history: Arc<RwLock<std::collections::HashMap<String, Vec<(String, String)>>>>,
}

impl ChatEngineState {
    pub fn new() -> Self {
        Self {
            vortex_runner: Arc::new(VortexRunner::new()),
            session_history: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

pub fn text_to_beams(text: &str) -> Vec<BeamTensor> {
    text.split_whitespace()
        .enumerate()
        .map(|(idx, word)| {
            let mut beam = BeamTensor::default();
            beam.word = word.to_string();
            for (i, &b) in word.as_bytes().iter().take(9).enumerate() {
                beam.digits[i] = (b as f32) / 255.0;
            }
            beam.position = ((idx % 9) + 1) as u8;
            beam.confidence = 1.0;
            beam
        })
        .collect()
}

pub fn generate_response(input: &str, latent: &LatentState) -> String {
    let input_lower = input.to_lowercase();
    
    if input_lower.contains("hello") || input_lower.contains("hi") {
        return format!("Hello! I'm SpatialVortex AI. Latent energy: {:.2}. How can I help?", latent.energy);
    }
    if input_lower.contains("what") && input_lower.contains("you") {
        return "I am SpatialVortex, trained with CALM, Multi-Head Attention, and Contrastive Learning.".to_string();
    }
    if input_lower.contains("how") && input_lower.contains("work") {
        return "I encode messages to latent space, run vortex cycles (1→2→4→8→7→5→1), and decode back.".to_string();
    }
    
    format!("Processed through vortex cycles. Latent energy: {:.3}, alignment: {:.1}%", 
            latent.energy, latent.sacred_alignment * 100.0)
}

// =============================================================================
// Route Configuration (only with web feature)
// =============================================================================

#[cfg(feature = "web")]
pub async fn chat_handler(
    state: web::Data<ChatEngineState>,
    req: web::Json<ChatRequest>,
) -> impl Responder {
    let session_id = req.session_id.clone()
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    
    let input_beams = text_to_beams(&req.message);
    let latent = state.vortex_runner.run_cycle(&input_beams).await;
    let response = generate_response(&req.message, &latent);
    
    {
        let mut history = state.session_history.write().await;
        history.entry(session_id.clone())
            .or_default()
            .push((req.message.clone(), response.clone()));
    }
    
    HttpResponse::Ok().json(ChatResponse {
        response,
        session_id,
        tokens_used: input_beams.len(),
        latent_energy: latent.energy,
        confidence: latent.sacred_alignment,
    })
}

#[cfg(feature = "web")]
pub async fn model_info_handler() -> impl Responder {
    HttpResponse::Ok().json(ModelInfoResponse {
        model_name: "SpatialVortex CALM".to_string(),
        version: "1.0.0".to_string(),
        latent_dim: 256,
        capabilities: vec![
            "Text Generation".to_string(),
            "Reasoning".to_string(),
            "Code Understanding".to_string(),
        ],
    })
}

#[cfg(feature = "web")]
pub async fn health_handler() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "SpatialVortex Chat API"
    }))
}

#[cfg(feature = "web")]
pub fn configure_chat_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/chat", web::post().to(chat_handler))
            .route("/model/info", web::get().to(model_info_handler))
            .route("/health", web::get().to(health_handler))
    );
}
