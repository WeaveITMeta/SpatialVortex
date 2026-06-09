//! Vortex API — OpenAI-Compatible REST Server with API Keys + Usage Billing
//!
//! Endpoints:
//!   POST /v1/chat/completions  — Chat completions (OpenAI-compatible, requires API key)
//!   GET  /v1/models            — List available models
//!   GET  /v1/usage             — Usage + cost for the calling API key
//!   GET  /health               — Health check (no auth)
//!   POST /admin/keys           — Create a new API key (admin only)
//!   GET  /admin/keys           — List keys + usage (admin only)
//!   DELETE /admin/keys/{key}   — Revoke a key (admin only)
//!
//! Auth:
//!   Authorization: Bearer sk-vortex-...      (developer keys)
//!   Authorization: Bearer <VORTEX_ADMIN_KEY> (admin endpoints)
//!
//! Environment:
//!   VORTEX_HOST                 bind host (default 127.0.0.1)
//!   VORTEX_PORT                 bind port (default 7000)
//!   VORTEX_ADMIN_KEY            admin bearer token (auto-generated + persisted if unset)
//!   VORTEX_REQUIRE_AUTH         "false" to disable key checks (default true)
//!   VORTEX_KEYS_PATH            key/usage store path (default ./vortex_keys.json)
//!   VORTEX_PRICE_INPUT_PER_M    USD per 1M input tokens  (default 0.10)
//!   VORTEX_PRICE_OUTPUT_PER_M   USD per 1M output tokens (default 0.40)
//!
//! Usage:
//!   cargo run --bin vortex-api --features web
//!   curl -X POST http://localhost:7000/v1/chat/completions \
//!     -H "Authorization: Bearer sk-vortex-..." \
//!     -H "Content-Type: application/json" \
//!     -d '{"model":"vortex-0.1","messages":[{"role":"user","content":"Hello"}]}'
//!
//! Table of Contents:
//! - OpenAI-compatible request/response types
//! - API key store (file-backed) + pricing
//! - Actix-web handlers (chat, models, usage, admin)
//! - Server startup

use actix_web::{web, App, HttpServer, HttpRequest, HttpResponse, middleware};
use actix_cors::Cors;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
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
    /// Cost of this request in USD at the server's configured rates
    cost_usd: f64,
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

/// OpenAI-style error body
fn api_error(status: actix_web::http::StatusCode, code: &str, message: &str) -> HttpResponse {
    HttpResponse::build(status).json(serde_json::json!({
        "error": { "type": code, "message": message }
    }))
}

// =============================================================================
// API Key Store + Pricing
// =============================================================================

/// Per-key accumulated usage
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct KeyUsage {
    requests: u64,
    prompt_tokens: u64,
    completion_tokens: u64,
}

/// A developer API key record
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiKeyRecord {
    /// Human label ("alice-dev", "ci-bot")
    label: String,
    /// RFC3339 creation timestamp
    created: String,
    /// Revoked keys are kept for usage history but rejected at auth
    #[serde(default)]
    revoked: bool,
    #[serde(default)]
    usage: KeyUsage,
}

/// File-backed key + usage store. Plaintext keys: this store is for
/// self-hosted deployments — keep the file readable only by the service user.
#[derive(Debug, Default, Serialize, Deserialize)]
struct KeyStore {
    /// Admin bearer token (auto-generated on first boot if env unset)
    admin_key: String,
    /// key -> record
    keys: HashMap<String, ApiKeyRecord>,
}

impl KeyStore {
    fn load_or_create(path: &PathBuf, admin_from_env: Option<String>) -> Self {
        let mut store: KeyStore = std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        if let Some(admin) = admin_from_env {
            store.admin_key = admin;
        } else if store.admin_key.is_empty() {
            store.admin_key = generate_key("sk-vortex-admin");
        }
        store
    }

    fn save(&self, path: &PathBuf) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            // Best-effort persistence; an unwritable store should not kill serving
            if let Err(e) = std::fs::write(path, json) {
                eprintln!("[WARN] could not persist key store to {:?}: {}", path, e);
            }
        }
    }
}

/// Generate a key with 128 bits of randomness
fn generate_key(prefix: &str) -> String {
    let raw = uuid::Uuid::new_v4().simple().to_string();
    format!("{}-{}", prefix, raw)
}

/// Pricing in USD per single token (converted from per-million env vars)
#[derive(Debug, Clone, Copy)]
struct Pricing {
    input_per_token: f64,
    output_per_token: f64,
}

impl Pricing {
    fn from_env() -> Self {
        let per_m = |var: &str, default: f64| -> f64 {
            std::env::var(var).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
        };
        Pricing {
            input_per_token: per_m("VORTEX_PRICE_INPUT_PER_M", 0.10) / 1_000_000.0,
            output_per_token: per_m("VORTEX_PRICE_OUTPUT_PER_M", 0.40) / 1_000_000.0,
        }
    }

    fn cost(&self, prompt_tokens: u64, completion_tokens: u64) -> f64 {
        prompt_tokens as f64 * self.input_per_token
            + completion_tokens as f64 * self.output_per_token
    }
}

// =============================================================================
// Application State + Auth
// =============================================================================

struct AppState {
    engine: Arc<Mutex<VortexEngine>>,
    keys: Arc<Mutex<KeyStore>>,
    keys_path: PathBuf,
    pricing: Pricing,
    require_auth: bool,
}

/// Extract the bearer token from an Authorization header
fn bearer_token(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("Authorization")?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .map(|s| s.trim().to_string())
}

/// Outcome of developer-key authentication
enum AuthResult {
    /// Authenticated with this key (None when auth is disabled)
    Ok(Option<String>),
    Denied(HttpResponse),
}

/// Authenticate a developer request; admin key is also accepted for /v1 routes
async fn authenticate(state: &AppState, req: &HttpRequest) -> AuthResult {
    if !state.require_auth {
        return AuthResult::Ok(None);
    }
    let Some(token) = bearer_token(req) else {
        return AuthResult::Denied(api_error(
            actix_web::http::StatusCode::UNAUTHORIZED,
            "invalid_request_error",
            "Missing Authorization: Bearer <api-key> header",
        ));
    };
    let store = state.keys.lock().await;
    if token == store.admin_key {
        return AuthResult::Ok(None); // admin usage is not metered
    }
    match store.keys.get(&token) {
        Some(rec) if !rec.revoked => AuthResult::Ok(Some(token)),
        Some(_) => AuthResult::Denied(api_error(
            actix_web::http::StatusCode::UNAUTHORIZED,
            "invalid_api_key",
            "This API key has been revoked",
        )),
        None => AuthResult::Denied(api_error(
            actix_web::http::StatusCode::UNAUTHORIZED,
            "invalid_api_key",
            "Unknown API key",
        )),
    }
}

/// Require the admin bearer token
async fn require_admin(state: &AppState, req: &HttpRequest) -> Option<HttpResponse> {
    let token = bearer_token(req);
    let store = state.keys.lock().await;
    if token.as_deref() == Some(store.admin_key.as_str()) {
        None
    } else {
        Some(api_error(
            actix_web::http::StatusCode::FORBIDDEN,
            "admin_required",
            "Admin key required for this endpoint",
        ))
    }
}

// =============================================================================
// Handlers
// =============================================================================

/// POST /v1/chat/completions — OpenAI-compatible chat completions
async fn chat_completions_handler(
    state: web::Data<AppState>,
    http_req: HttpRequest,
    req: web::Json<ChatCompletionRequest>,
) -> HttpResponse {
    let metered_key = match authenticate(&state, &http_req).await {
        AuthResult::Ok(key) => key,
        AuthResult::Denied(resp) => return resp,
    };

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

    let prompt_tokens = response.usage.prompt_tokens as u64;
    let completion_tokens = response.usage.completion_tokens as u64;
    let cost_usd = state.pricing.cost(prompt_tokens, completion_tokens);

    // Meter usage against the calling key
    if let Some(key) = metered_key {
        let mut store = state.keys.lock().await;
        if let Some(rec) = store.keys.get_mut(&key) {
            rec.usage.requests += 1;
            rec.usage.prompt_tokens += prompt_tokens;
            rec.usage.completion_tokens += completion_tokens;
        }
        store.save(&state.keys_path);
    }

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
            cost_usd,
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

/// GET /v1/usage — usage + cost for the calling key
async fn usage_handler(state: web::Data<AppState>, http_req: HttpRequest) -> HttpResponse {
    let key = match authenticate(&state, &http_req).await {
        AuthResult::Ok(Some(key)) => key,
        AuthResult::Ok(None) => {
            return api_error(
                actix_web::http::StatusCode::BAD_REQUEST,
                "invalid_request_error",
                "Usage reporting requires a developer API key (not admin / open mode)",
            );
        }
        AuthResult::Denied(resp) => return resp,
    };
    let store = state.keys.lock().await;
    let Some(rec) = store.keys.get(&key) else {
        return api_error(
            actix_web::http::StatusCode::NOT_FOUND,
            "not_found",
            "Key not found",
        );
    };
    let cost = state.pricing.cost(rec.usage.prompt_tokens, rec.usage.completion_tokens);
    HttpResponse::Ok().json(serde_json::json!({
        "label": rec.label,
        "created": rec.created,
        "requests": rec.usage.requests,
        "prompt_tokens": rec.usage.prompt_tokens,
        "completion_tokens": rec.usage.completion_tokens,
        "total_tokens": rec.usage.prompt_tokens + rec.usage.completion_tokens,
        "cost_usd": cost,
        "pricing": {
            "input_per_million_usd": state.pricing.input_per_token * 1_000_000.0,
            "output_per_million_usd": state.pricing.output_per_token * 1_000_000.0,
        },
    }))
}

#[derive(Debug, Deserialize)]
struct CreateKeyRequest {
    /// Human label for the key ("alice-dev")
    label: String,
}

/// POST /admin/keys — create a developer key
async fn create_key_handler(
    state: web::Data<AppState>,
    http_req: HttpRequest,
    req: web::Json<CreateKeyRequest>,
) -> HttpResponse {
    if let Some(denied) = require_admin(&state, &http_req).await {
        return denied;
    }
    let key = generate_key("sk-vortex");
    let record = ApiKeyRecord {
        label: req.label.clone(),
        created: chrono::Utc::now().to_rfc3339(),
        revoked: false,
        usage: KeyUsage::default(),
    };
    let mut store = state.keys.lock().await;
    store.keys.insert(key.clone(), record);
    store.save(&state.keys_path);
    HttpResponse::Ok().json(serde_json::json!({
        "api_key": key,
        "label": req.label,
        "note": "Store this key now — it is shown in full only here and in the server-side store file.",
    }))
}

/// GET /admin/keys — list keys (redacted) with usage + cost
async fn list_keys_handler(state: web::Data<AppState>, http_req: HttpRequest) -> HttpResponse {
    if let Some(denied) = require_admin(&state, &http_req).await {
        return denied;
    }
    let store = state.keys.lock().await;
    let keys: Vec<serde_json::Value> = store.keys.iter().map(|(key, rec)| {
        let redacted = format!("{}...{}", &key[..key.len().min(13)], &key[key.len().saturating_sub(4)..]);
        serde_json::json!({
            "key": redacted,
            "label": rec.label,
            "created": rec.created,
            "revoked": rec.revoked,
            "requests": rec.usage.requests,
            "prompt_tokens": rec.usage.prompt_tokens,
            "completion_tokens": rec.usage.completion_tokens,
            "cost_usd": state.pricing.cost(rec.usage.prompt_tokens, rec.usage.completion_tokens),
        })
    }).collect();
    HttpResponse::Ok().json(serde_json::json!({ "keys": keys }))
}

/// DELETE /admin/keys/{key} — revoke a key (kept for usage history)
async fn revoke_key_handler(
    state: web::Data<AppState>,
    http_req: HttpRequest,
    path: web::Path<String>,
) -> HttpResponse {
    if let Some(denied) = require_admin(&state, &http_req).await {
        return denied;
    }
    let key = path.into_inner();
    let mut store = state.keys.lock().await;
    match store.keys.get_mut(&key) {
        Some(rec) => {
            rec.revoked = true;
            store.save(&state.keys_path);
            HttpResponse::Ok().json(serde_json::json!({ "revoked": true }))
        }
        None => api_error(
            actix_web::http::StatusCode::NOT_FOUND,
            "not_found",
            "Key not found",
        ),
    }
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
    let require_auth = std::env::var("VORTEX_REQUIRE_AUTH")
        .map(|v| v.to_lowercase() != "false")
        .unwrap_or(true);
    let keys_path = PathBuf::from(
        std::env::var("VORTEX_KEYS_PATH").unwrap_or_else(|_| "vortex_keys.json".to_string()),
    );
    let pricing = Pricing::from_env();

    let admin_from_env = std::env::var("VORTEX_ADMIN_KEY").ok();
    let admin_was_generated = admin_from_env.is_none();
    let key_store = KeyStore::load_or_create(&keys_path, admin_from_env);
    key_store.save(&keys_path);

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║              Vortex API — OpenAI-Compatible Server           ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    // VORTEX_LIGHTWEIGHT=true skips HF dataset downloads + web learning at
    // boot — seconds instead of minutes, at the cost of knowledge coverage.
    let lightweight = std::env::var("VORTEX_LIGHTWEIGHT")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false);
    println!(
        "🌀 Initializing Vortex engine ({} mode)...",
        if lightweight { "lightweight" } else { "full-knowledge" }
    );

    let config = if lightweight {
        VortexEngineConfig::lightweight()
    } else {
        VortexEngineConfig::default()
    };
    let engine = VortexEngine::with_config(config);

    println!("   ✅ Engine ready");
    println!();
    println!("🌐 Server: http://{}:{}", host, port);
    println!("🔐 Auth: {}", if require_auth { "API keys required" } else { "OPEN (VORTEX_REQUIRE_AUTH=false)" });
    if admin_was_generated {
        println!("🔑 Admin key (auto-generated, persisted in {:?}):", keys_path);
        println!("   {}", key_store.admin_key);
    }
    println!("💲 Pricing: ${:.2}/M input, ${:.2}/M output tokens",
        pricing.input_per_token * 1_000_000.0,
        pricing.output_per_token * 1_000_000.0);
    println!();
    println!("📋 Endpoints:");
    println!("   POST   /v1/chat/completions  — Chat (OpenAI-compatible)");
    println!("   GET    /v1/models            — List models");
    println!("   GET    /v1/usage             — Usage + cost for your key");
    println!("   GET    /health               — Health check");
    println!("   POST   /admin/keys           — Create API key (admin)");
    println!("   GET    /admin/keys           — List keys (admin)");
    println!("   DELETE /admin/keys/{{key}}     — Revoke key (admin)");
    println!();
    println!("📝 Create a developer key:");
    println!("   curl -X POST http://{}:{}/admin/keys \\", host, port);
    println!("     -H 'Authorization: Bearer <admin-key>' \\");
    println!("     -H 'Content-Type: application/json' -d '{{\"label\":\"my-app\"}}'");
    println!();

    let state = web::Data::new(AppState {
        engine: Arc::new(Mutex::new(engine)),
        keys: Arc::new(Mutex::new(key_store)),
        keys_path,
        pricing,
        require_auth,
    });

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(Cors::permissive())
            .wrap(middleware::Logger::new("%a %r %s %Dms"))
            .route("/v1/chat/completions", web::post().to(chat_completions_handler))
            .route("/v1/models", web::get().to(models_handler))
            .route("/v1/usage", web::get().to(usage_handler))
            .route("/health", web::get().to(health_handler))
            .route("/admin/keys", web::post().to(create_key_handler))
            .route("/admin/keys", web::get().to(list_keys_handler))
            .route("/admin/keys/{key}", web::delete().to(revoke_key_handler))
    })
    .bind((host.as_str(), port))?
    .run()
    .await?;

    Ok(())
}
