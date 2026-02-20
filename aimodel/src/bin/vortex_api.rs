//! Vortex API — OpenAI-Compatible REST Server
//!
//! Endpoints:
//!   POST /v1/chat/completions  — Chat completions (OpenAI-compatible)
//!   GET  /v1/models            — List available models
//!   GET  /health               — Health check
//!
//! Usage:
//!   cargo run --bin vortex-api --features web
//!   curl -X POST http://localhost:7000/v1/chat/completions \
//!     -H "Content-Type: application/json" \
//!     -d '{"model":"vortex-0.1","messages":[{"role":"user","content":"Hello"}]}'
//!
//! Table of Contents:
//! - OpenAI-compatible request/response types
//! - Actix-web handlers
//! - Engine state management
//! - Server startup

use actix_web::{web, App, HttpServer, HttpResponse, middleware};
use actix_cors::Cors;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use vortex::engine::{VortexEngine, VortexEngineConfig, ChatMessage};

// =============================================================================
// OpenAI-Compatible Request/Response Types
// =============================================================================

/// OpenAI chat completion request
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ChatCompletionRequest {
    /// Model identifier (ignored — always uses vortex)
    #[serde(default)]
    model: Option<String>,
    /// Conversation messages
    messages: Vec<ApiMessage>,
    /// Sampling temperature
    #[serde(default = "default_temperature")]
    temperature: f32,
    /// Maximum tokens to generate (informational — vortex uses cycles)
    #[serde(default)]
    max_tokens: Option<usize>,
    /// Whether to stream (not yet supported)
    #[serde(default)]
    stream: bool,
}

fn default_temperature() -> f32 { 0.7 }

/// OpenAI message format
#[derive(Debug, Deserialize, Serialize, Clone)]
struct ApiMessage {
    role: String,
    content: String,
}

/// OpenAI chat completion response
#[derive(Debug, Serialize)]
struct ChatCompletionResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<Choice>,
    usage: ApiUsage,
    /// Vortex-specific extensions
    #[serde(skip_serializing_if = "Option::is_none")]
    vortex: Option<VortexExtension>,
}

/// A single completion choice
#[derive(Debug, Serialize)]
struct Choice {
    index: usize,
    message: ApiMessage,
    finish_reason: String,
}

/// Token usage
#[derive(Debug, Serialize)]
struct ApiUsage {
    prompt_tokens: usize,
    completion_tokens: usize,
    total_tokens: usize,
}

/// Vortex-specific response extensions
#[derive(Debug, Serialize)]
struct VortexExtension {
    confidence: f32,
    energy: f32,
    sacred_alignment: f32,
    cycles: u64,
    duration_ms: u64,
    reasoning_trace: Vec<ApiReasoningStep>,
    safety: Option<ApiSafety>,
}

/// Reasoning step for API
#[derive(Debug, Serialize)]
struct ApiReasoningStep {
    step: usize,
    position: u8,
    content: String,
    confidence: f32,
    is_sacred: bool,
    step_type: String,
}

/// Safety result for API
#[derive(Debug, Serialize)]
struct ApiSafety {
    passed: bool,
    violations: Vec<String>,
}

/// Model info for /v1/models
#[derive(Debug, Serialize)]
struct ModelList {
    object: String,
    data: Vec<ModelInfo>,
}

/// Individual model info
#[derive(Debug, Serialize)]
struct ModelInfo {
    id: String,
    object: String,
    created: u64,
    owned_by: String,
}

// =============================================================================
// Application State
// =============================================================================

struct AppState {
    engine: Arc<Mutex<VortexEngine>>,
}

// =============================================================================
// Handlers
// =============================================================================

/// POST /v1/chat/completions — OpenAI-compatible chat completions
async fn chat_completions_handler(
    state: web::Data<AppState>,
    req: web::Json<ChatCompletionRequest>,
) -> HttpResponse {
    // Convert API messages to Vortex ChatMessages
    let messages: Vec<ChatMessage> = req.messages.iter().map(|m| {
        match m.role.as_str() {
            "system" => ChatMessage::system(&m.content),
            "assistant" => ChatMessage::assistant(&m.content),
            _ => ChatMessage::user(&m.content),
        }
    }).collect();

    // Run inference
    let response = {
        let mut engine = state.engine.lock().await;
        engine.chat_completions(&messages)
    };

    // Build OpenAI-compatible response
    let created = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let api_response = ChatCompletionResponse {
        id: response.id.clone(),
        object: "chat.completion".to_string(),
        created,
        model: response.model.clone(),
        choices: vec![Choice {
            index: 0,
            message: ApiMessage {
                role: "assistant".to_string(),
                content: response.content.clone(),
            },
            finish_reason: "stop".to_string(),
        }],
        usage: ApiUsage {
            prompt_tokens: response.usage.prompt_tokens,
            completion_tokens: response.usage.completion_tokens,
            total_tokens: response.usage.total_tokens,
        },
        vortex: Some(VortexExtension {
            confidence: response.confidence,
            energy: response.energy,
            sacred_alignment: response.sacred_alignment,
            cycles: response.cycles,
            duration_ms: response.duration_ms,
            reasoning_trace: response.reasoning_trace.iter().map(|s| ApiReasoningStep {
                step: s.step,
                position: s.position,
                content: s.content.clone(),
                confidence: s.confidence,
                is_sacred: s.is_sacred,
                step_type: s.step_type.clone(),
            }).collect(),
            safety: response.safety.map(|s| ApiSafety {
                passed: s.passed,
                violations: s.violations,
            }),
        }),
    };

    HttpResponse::Ok().json(api_response)
}

/// GET /v1/models — List available models
async fn models_handler() -> HttpResponse {
    let created = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    HttpResponse::Ok().json(ModelList {
        object: "list".to_string(),
        data: vec![ModelInfo {
            id: "vortex-0.1".to_string(),
            object: "model".to_string(),
            created,
            owned_by: "spatialvortex".to_string(),
        }],
    })
}

/// GET /health — Health check
async fn health_handler() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "Vortex API",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

// =============================================================================
// Main
// =============================================================================

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let host = std::env::var("VORTEX_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port: u16 = std::env::var("VORTEX_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(7000);

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║              Vortex API — OpenAI-Compatible Server           ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    println!("🌀 Initializing Vortex engine...");

    let config = VortexEngineConfig::default();
    let engine = VortexEngine::with_config(config);

    println!("   ✅ Engine ready");
    println!();
    println!("🌐 Server: http://{}:{}", host, port);
    println!();
    println!("📋 Endpoints:");
    println!("   POST /v1/chat/completions  — Chat (OpenAI-compatible)");
    println!("   GET  /v1/models            — List models");
    println!("   GET  /health               — Health check");
    println!();
    println!("📝 Example:");
    println!("   curl -X POST http://{}:{}/v1/chat/completions \\", host, port);
    println!("     -H 'Content-Type: application/json' \\");
    println!("     -d '{{\"model\":\"vortex-0.1\",\"messages\":[{{\"role\":\"user\",\"content\":\"Hello\"}}]}}'");
    println!();

    let state = web::Data::new(AppState {
        engine: Arc::new(Mutex::new(engine)),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(Cors::permissive())
            .wrap(middleware::Logger::new("%a %r %s %Dms"))
            .route("/v1/chat/completions", web::post().to(chat_completions_handler))
            .route("/v1/models", web::get().to(models_handler))
            .route("/health", web::get().to(health_handler))
    })
    .bind((host.as_str(), port))?
    .run()
    .await?;

    Ok(())
}
