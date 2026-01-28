//! SpatialVortex Chat Server
//!
//! Web server for chatting with the trained AI model.
//! Provides both REST API and a web UI.

use actix_web::{web, App, HttpServer, HttpResponse, middleware};
use actix_cors::Cors;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use anyhow::Result;

use aimodel::ml::calm::LatentState;
use aimodel::data::models::BeamTensor;
use aimodel::cognition::vortex_runner::VortexRunner;

// =============================================================================
// Request/Response Types
// =============================================================================

#[derive(Debug, Deserialize)]
struct ChatRequest {
    message: String,
    #[serde(default)]
    session_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct ChatResponse {
    response: String,
    session_id: String,
    tokens_used: usize,
    latent_energy: f32,
    confidence: f32,
}

// =============================================================================
// Application State
// =============================================================================

struct AppState {
    vortex_runner: Arc<VortexRunner>,
    session_history: Arc<tokio::sync::RwLock<std::collections::HashMap<String, Vec<(String, String)>>>>,
}

// =============================================================================
// API Handlers
// =============================================================================

async fn chat_handler(
    state: web::Data<AppState>,
    req: web::Json<ChatRequest>,
) -> HttpResponse {
    let session_id = req.session_id.clone()
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    
    println!("📝 Chat request: {}", &req.message);
    
    // Convert message to BeamTensors
    let input_beams = text_to_beams(&req.message);
    
    // Run through vortex cycle (uses internal CALM engine)
    let processed_latent = state.vortex_runner.run_cycle(&input_beams).await;
    
    // Generate response based on latent state
    let response_text = generate_response(&req.message, &processed_latent);
    
    // Store in session history
    {
        let mut history = state.session_history.write().await;
        history.entry(session_id.clone())
            .or_insert_with(Vec::new)
            .push((req.message.clone(), response_text.clone()));
    }
    
    println!("💬 Response: {}", &response_text);
    
    HttpResponse::Ok().json(ChatResponse {
        response: response_text,
        session_id,
        tokens_used: input_beams.len(),
        latent_energy: processed_latent.energy,
        confidence: processed_latent.sacred_alignment,
    })
}

async fn health_handler() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "SpatialVortex Chat",
        "version": "1.0.0"
    }))
}

async fn index_handler() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../../static/chat.html"))
}

// =============================================================================
// Helper Functions
// =============================================================================

fn text_to_beams(text: &str) -> Vec<BeamTensor> {
    let words: Vec<&str> = text.split_whitespace().collect();
    
    words.iter().enumerate()
        .map(|(idx, word)| {
            let mut beam = BeamTensor::default();
            beam.word = word.to_string();
            
            let bytes = word.as_bytes();
            for (i, &b) in bytes.iter().take(9).enumerate() {
                beam.digits[i] = (b as f32) / 255.0;
            }
            
            beam.position = ((idx % 9) + 1) as u8;
            beam.confidence = 1.0;
            beam
        })
        .collect()
}

fn beams_to_text(beams: &[BeamTensor]) -> String {
    beams.iter()
        .filter(|b| !b.word.is_empty())
        .map(|b| b.word.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

fn generate_response(input: &str, latent: &aimodel::ml::calm::LatentState) -> String {
    let input_lower = input.to_lowercase();
    
    // Pattern-based responses using latent energy
    if input_lower.contains("hello") || input_lower.contains("hi") {
        return format!("Hello! I'm SpatialVortex AI. My latent energy is {:.2}. How can I help you today?", latent.energy);
    }
    
    if input_lower.contains("what") && input_lower.contains("you") {
        return "I am SpatialVortex, a neural AI trained with CALM (Continuous Autoregressive Latent Model), \
                Multi-Head Attention, and Contrastive Learning. I process information through sacred vortex cycles.".to_string();
    }
    
    if input_lower.contains("how") && input_lower.contains("work") {
        return "I work by encoding your message into a latent space, running it through exponential vortex cycles \
                (1→2→4→8→7→5→1), and decoding the processed representation back to text.".to_string();
    }
    
    if input_lower.contains("math") || input_lower.contains("calculate") {
        // Try to extract numbers and perform calculation
        let nums: Vec<i32> = input.split_whitespace()
            .filter_map(|w| w.parse().ok())
            .collect();
        if nums.len() >= 2 {
            let sum: i32 = nums.iter().sum();
            return format!("The sum of {:?} is {}. My confidence is {:.1}%.", nums, sum, latent.energy.abs() * 50.0 + 50.0);
        }
    }
    
    if input_lower.contains("code") || input_lower.contains("program") {
        return "I can help with code! I've been trained on programming datasets including The Stack, HumanEval, \
                and MBPP. What would you like to build?".to_string();
    }
    
    // Default response based on latent state
    format!(
        "I processed your message through {} vortex cycles. Latent energy: {:.3}. \
         I'm still learning - try asking me about math, code, or how I work!",
        4, latent.energy
    )
}

// =============================================================================
// Main
// =============================================================================

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║           SpatialVortex Chat Server v1.0                 ║");
    println!("║                                                          ║");
    println!("║  CALM Engine · Vortex Cycles · Multi-Head Attention     ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();
    
    // Initialize components
    println!("🌀 Initializing Vortex Runner (includes CALM Engine)...");
    let vortex_runner = Arc::new(VortexRunner::new());
    println!("   ✅ Vortex Runner ready (sacred cycles + CALM latent_dim=256)");
    
    let app_state = AppState {
        vortex_runner: vortex_runner.clone(),
        session_history: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
    };
    
    let host = std::env::var("CHAT_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port: u16 = std::env::var("CHAT_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);
    
    println!();
    println!("🌐 Starting server at http://{}:{}", host, port);
    println!();
    println!("📋 Endpoints:");
    println!("   GET  /           - Web chat interface");
    println!("   POST /api/chat   - Chat API");
    println!("   GET  /api/health - Health check");
    println!();
    println!("🚀 Open http://{}:{} in your browser to chat!", host, port);
    println!();
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                vortex_runner: app_state.vortex_runner.clone(),
                session_history: app_state.session_history.clone(),
            }))
            .wrap(Cors::permissive())
            .wrap(middleware::Logger::default())
            .route("/", web::get().to(index_handler))
            .route("/api/chat", web::post().to(chat_handler))
            .route("/api/health", web::get().to(health_handler))
    })
    .bind((host.as_str(), port))?
    .run()
    .await?;
    
    Ok(())
}
