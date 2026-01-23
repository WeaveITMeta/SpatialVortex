//! Production API Endpoints
//!
//! Monetizable API endpoints with:
//! - Authentication via API keys
//! - Rate limiting per tier
//! - Usage tracking and billing
//! - Comprehensive error handling
//! - OpenAPI documentation

use actix_web::{web, HttpRequest, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::ai::billing::{BillingEngine, ApiKey, ApiTier, RateLimitResult, extract_api_key, billing_error_response};
use crate::ai::orchestrator::ASIOrchestrator;
use crate::ai::flux_reasoning::FluxReasoningChain;
use crate::ai::causal_reasoning::{CausalWorldModel, CausalValue};
use crate::ai::goal_planner::GoalPlanner;
use crate::data::models::ELPTensor;

// ============================================================================
// Shared State
// ============================================================================

/// Production API state
pub struct ProductionApiState {
    pub billing: Arc<BillingEngine>,
    pub orchestrator: Arc<Mutex<ASIOrchestrator>>,
    pub causal_model: Arc<Mutex<CausalWorldModel>>,
    pub goal_planner: Arc<Mutex<GoalPlanner>>,
}

impl ProductionApiState {
    pub async fn new() -> anyhow::Result<Self> {
        Ok(Self {
            billing: Arc::new(BillingEngine::new(None)),
            orchestrator: Arc::new(Mutex::new(ASIOrchestrator::new().await?)),
            causal_model: Arc::new(Mutex::new(CausalWorldModel::new())),
            goal_planner: Arc::new(Mutex::new(GoalPlanner::new())),
        })
    }
}

// ============================================================================
// Request/Response Types
// ============================================================================

/// Reasoning request
#[derive(Debug, Deserialize)]
pub struct ReasonRequest {
    pub query: String,
    #[serde(default = "default_max_steps")]
    pub max_steps: usize,
    #[serde(default)]
    pub include_trace: bool,
}

fn default_max_steps() -> usize { 20 }

/// Reasoning response
#[derive(Debug, Serialize)]
pub struct ReasonResponse {
    pub id: String,
    pub answer: String,
    pub confidence: f32,
    pub reasoning_steps: usize,
    pub tokens_used: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace: Option<Vec<ReasoningStep>>,
    pub usage: UsageInfo,
}

#[derive(Debug, Serialize)]
pub struct ReasoningStep {
    pub step: usize,
    pub position: u8,
    pub entropy: f32,
    pub certainty: f32,
    pub insight: String,
}

#[derive(Debug, Serialize)]
pub struct UsageInfo {
    pub tokens: u32,
    pub reasoning_steps: u32,
    pub cost_cents: u32,
    pub remaining_requests_minute: u32,
    pub remaining_requests_day: u32,
}

/// Causal query request
#[derive(Debug, Deserialize)]
pub struct CausalRequest {
    pub cause: String,
    pub effect: String,
    #[serde(default = "default_strength")]
    pub strength: f32,
}

fn default_strength() -> f32 { 0.8 }

/// Counterfactual request
#[derive(Debug, Deserialize)]
pub struct CounterfactualRequest {
    pub description: String,
    pub variable: String,
    pub value: f64,
    pub query: String,
}

/// Goal request
#[derive(Debug, Deserialize)]
pub struct GoalRequest {
    pub objective: String,
    #[serde(default)]
    pub ethos: f64,
    #[serde(default)]
    pub logos: f64,
    #[serde(default)]
    pub pathos: f64,
}

/// API key request
#[derive(Debug, Deserialize)]
pub struct CreateApiKeyRequest {
    pub email: String,
    pub tier: String,
    #[serde(default)]
    pub organization: Option<String>,
}

/// Standard API response wrapper
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
    pub request_id: String,
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            request_id: Uuid::new_v4().to_string(),
        }
    }
    
    pub fn error(code: &str, message: &str) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            error: Some(ApiError {
                code: code.to_string(),
                message: message.to_string(),
            }),
            request_id: Uuid::new_v4().to_string(),
        }
    }
}

// ============================================================================
// Authentication Helper
// ============================================================================

fn authenticate(req: &HttpRequest, billing: &BillingEngine) -> Result<ApiKey, HttpResponse> {
    // Extract API key from headers
    let raw_key = req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer ").map(|k| k.to_string()))
        .or_else(|| {
            req.headers()
                .get("X-API-Key")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string())
        })
        .ok_or_else(|| billing_error_response(401, "Missing API key", "missing_api_key"))?;
    
    let api_key = billing.validate_key(&raw_key)
        .ok_or_else(|| billing_error_response(401, "Invalid API key", "invalid_api_key"))?;
    
    Ok(api_key)
}

fn check_rate_limit(billing: &BillingEngine, api_key: &ApiKey) -> Result<(u32, u32), HttpResponse> {
    match billing.check_rate_limit(api_key) {
        RateLimitResult::Allowed { remaining_minute, remaining_day } => {
            Ok((remaining_minute, remaining_day))
        },
        RateLimitResult::ExceededMinute { limit, retry_after_seconds } => {
            Err(HttpResponse::TooManyRequests()
                .insert_header(("Retry-After", retry_after_seconds.to_string()))
                .insert_header(("X-RateLimit-Limit", limit.to_string()))
                .json(ApiResponse::<()>::error(
                    "rate_limit_exceeded",
                    &format!("Rate limit exceeded. Retry after {} seconds.", retry_after_seconds)
                )))
        },
        RateLimitResult::ExceededDaily { limit, retry_after_seconds } => {
            Err(HttpResponse::TooManyRequests()
                .insert_header(("Retry-After", retry_after_seconds.to_string()))
                .insert_header(("X-RateLimit-Limit", limit.to_string()))
                .json(ApiResponse::<()>::error(
                    "daily_limit_exceeded",
                    "Daily request limit exceeded. Upgrade your plan for more requests."
                )))
        },
    }
}

// ============================================================================
// API Endpoints
// ============================================================================

/// POST /api/v1/reason
/// 
/// Main reasoning endpoint - the core AGI capability
#[utoipa::path(
    post,
    path = "/api/v1/reason",
    request_body = ReasonRequest,
    responses(
        (status = 200, description = "Reasoning completed", body = ReasonResponse),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limit exceeded"),
    ),
    security(("api_key" = []))
)]
pub async fn reason_endpoint(
    req: HttpRequest,
    body: web::Json<ReasonRequest>,
    state: web::Data<ProductionApiState>,
) -> ActixResult<HttpResponse> {
    let start = std::time::Instant::now();
    
    // Authenticate
    let api_key = match authenticate(&req, &state.billing) {
        Ok(key) => key,
        Err(resp) => return Ok(resp),
    };
    
    // Check rate limit
    let (remaining_minute, remaining_day) = match check_rate_limit(&state.billing, &api_key) {
        Ok(limits) => limits,
        Err(resp) => return Ok(resp),
    };
    
    // Enforce tier limits
    let max_steps = body.max_steps.min(api_key.tier.max_reasoning_steps());
    
    // Execute reasoning
    let mut chain = FluxReasoningChain::new(&body.query);
    let result = chain.reason(max_steps).await;
    
    let final_thought = match result {
        Ok(thought) => thought,
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(
                ApiResponse::<()>::error("reasoning_failed", &e.to_string())
            ));
        }
    };
    
    // Synthesize answer
    let answer = match chain.synthesize_final_answer().await {
        Ok(a) => a,
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(
                ApiResponse::<()>::error("synthesis_failed", &e.to_string())
            ));
        }
    };
    
    // Calculate tokens (rough estimate)
    let tokens_used = (body.query.len() + answer.len()) as u32 / 4;
    let reasoning_steps = chain.thoughts.len() as u32;
    
    // Record usage
    let usage_record = state.billing.record_usage(
        &api_key,
        "/api/v1/reason",
        tokens_used,
        reasoning_steps,
        start.elapsed().as_millis() as u64,
        true,
    );
    
    // Build trace if requested
    let trace = if body.include_trace {
        Some(chain.thoughts.iter().enumerate().map(|(i, t)| {
            ReasoningStep {
                step: i,
                position: t.vortex_position,
                entropy: t.entropy,
                certainty: t.certainty,
                insight: t.reasoning_trace.clone(),
            }
        }).collect())
    } else {
        None
    };
    
    let response = ReasonResponse {
        id: Uuid::new_v4().to_string(),
        answer,
        confidence: final_thought.certainty,
        reasoning_steps: chain.thoughts.len(),
        tokens_used,
        trace,
        usage: UsageInfo {
            tokens: tokens_used,
            reasoning_steps,
            cost_cents: usage_record.cost_cents,
            remaining_requests_minute: remaining_minute,
            remaining_requests_day: remaining_day,
        },
    };
    
    Ok(HttpResponse::Ok()
        .insert_header(("X-RateLimit-Remaining-Minute", remaining_minute.to_string()))
        .insert_header(("X-RateLimit-Remaining-Day", remaining_day.to_string()))
        .json(ApiResponse::success(response)))
}

/// POST /api/v1/causal/learn
/// 
/// Learn a causal relationship
pub async fn causal_learn_endpoint(
    req: HttpRequest,
    body: web::Json<CausalRequest>,
    state: web::Data<ProductionApiState>,
) -> ActixResult<HttpResponse> {
    let api_key = match authenticate(&req, &state.billing) {
        Ok(key) => key,
        Err(resp) => return Ok(resp),
    };
    
    let (remaining_minute, remaining_day) = match check_rate_limit(&state.billing, &api_key) {
        Ok(limits) => limits,
        Err(resp) => return Ok(resp),
    };
    
    let elp = ELPTensor { ethos: 5.0, logos: 8.0, pathos: 4.0 };
    
    let mut causal_model = state.causal_model.lock().await;
    causal_model.learn_from_observation(&body.cause, &body.effect, body.strength, &elp);
    
    state.billing.record_usage(&api_key, "/api/v1/causal/learn", 50, 1, 10, true);
    
    Ok(HttpResponse::Ok()
        .insert_header(("X-RateLimit-Remaining-Minute", remaining_minute.to_string()))
        .json(ApiResponse::success(serde_json::json!({
            "learned": true,
            "cause": body.cause,
            "effect": body.effect,
            "strength": body.strength,
        }))))
}

/// POST /api/v1/causal/counterfactual
/// 
/// Ask a counterfactual question
pub async fn counterfactual_endpoint(
    req: HttpRequest,
    body: web::Json<CounterfactualRequest>,
    state: web::Data<ProductionApiState>,
) -> ActixResult<HttpResponse> {
    let api_key = match authenticate(&req, &state.billing) {
        Ok(key) => key,
        Err(resp) => return Ok(resp),
    };
    
    let (remaining_minute, remaining_day) = match check_rate_limit(&state.billing, &api_key) {
        Ok(limits) => limits,
        Err(resp) => return Ok(resp),
    };
    
    let mut causal_model = state.causal_model.lock().await;
    
    let result = causal_model.ask_counterfactual(
        &body.description,
        &body.variable,
        CausalValue::Numeric(body.value),
        &body.query,
    );
    
    match result {
        Ok(cf) => {
            state.billing.record_usage(&api_key, "/api/v1/causal/counterfactual", 100, 5, 50, true);
            
            Ok(HttpResponse::Ok()
                .insert_header(("X-RateLimit-Remaining-Minute", remaining_minute.to_string()))
                .json(ApiResponse::success(serde_json::json!({
                    "description": cf.description,
                    "confidence": cf.confidence,
                    "counterfactual_value": cf.counterfactual_value,
                }))))
        },
        Err(e) => {
            Ok(HttpResponse::BadRequest().json(
                ApiResponse::<()>::error("counterfactual_failed", &e.to_string())
            ))
        }
    }
}

/// POST /api/v1/goal/create
/// 
/// Create a new goal
pub async fn create_goal_endpoint(
    req: HttpRequest,
    body: web::Json<GoalRequest>,
    state: web::Data<ProductionApiState>,
) -> ActixResult<HttpResponse> {
    let api_key = match authenticate(&req, &state.billing) {
        Ok(key) => key,
        Err(resp) => return Ok(resp),
    };
    
    let (remaining_minute, remaining_day) = match check_rate_limit(&state.billing, &api_key) {
        Ok(limits) => limits,
        Err(resp) => return Ok(resp),
    };
    
    let elp = ELPTensor {
        ethos: if body.ethos > 0.0 { body.ethos } else { 5.0 },
        logos: if body.logos > 0.0 { body.logos } else { 5.0 },
        pathos: if body.pathos > 0.0 { body.pathos } else { 5.0 },
    };
    
    let mut planner = state.goal_planner.lock().await;
    let goal = planner.create_goal(&body.objective, &elp);
    let goal_id = goal.id;
    planner.add_goal(goal.clone());
    
    state.billing.record_usage(&api_key, "/api/v1/goal/create", 50, 2, 20, true);
    
    Ok(HttpResponse::Ok()
        .insert_header(("X-RateLimit-Remaining-Minute", remaining_minute.to_string()))
        .json(ApiResponse::success(serde_json::json!({
            "goal_id": goal_id.to_string(),
            "objective": goal.objective,
            "importance": goal.importance,
            "vortex_position": goal.vortex_position,
            "status": format!("{:?}", goal.status),
        }))))
}

/// GET /api/v1/usage
/// 
/// Get usage statistics
pub async fn usage_endpoint(
    req: HttpRequest,
    state: web::Data<ProductionApiState>,
) -> ActixResult<HttpResponse> {
    let api_key = match authenticate(&req, &state.billing) {
        Ok(key) => key,
        Err(resp) => return Ok(resp),
    };
    
    let summary = state.billing.get_billing_summary(api_key.id, api_key.tier);
    
    Ok(HttpResponse::Ok().json(ApiResponse::success(summary)))
}

/// POST /api/v1/keys
/// 
/// Create a new API key (admin endpoint)
pub async fn create_api_key_endpoint(
    body: web::Json<CreateApiKeyRequest>,
    state: web::Data<ProductionApiState>,
) -> ActixResult<HttpResponse> {
    let tier = match body.tier.to_lowercase().as_str() {
        "free" => ApiTier::Free,
        "developer" => ApiTier::Developer,
        "professional" => ApiTier::Professional,
        "enterprise" => ApiTier::Enterprise,
        _ => return Ok(HttpResponse::BadRequest().json(
            ApiResponse::<()>::error("invalid_tier", "Tier must be: free, developer, professional, or enterprise")
        )),
    };
    
    let metadata = crate::ai::billing::ApiKeyMetadata {
        organization: body.organization.clone(),
        ..Default::default()
    };
    
    let (raw_key, api_key) = state.billing.generate_api_key(
        tier,
        &Uuid::new_v4().to_string(),
        &body.email,
        Some(metadata),
    );
    
    Ok(HttpResponse::Created().json(ApiResponse::success(serde_json::json!({
        "api_key": raw_key,
        "key_id": api_key.id.to_string(),
        "tier": format!("{:?}", api_key.tier),
        "features": api_key.tier.features(),
        "rate_limits": {
            "requests_per_minute": api_key.tier.rate_limit_rpm(),
            "requests_per_day": api_key.tier.rate_limit_rpd(),
            "max_reasoning_steps": api_key.tier.max_reasoning_steps(),
            "max_tokens": api_key.tier.max_tokens(),
        },
        "pricing": {
            "monthly_base_cents": api_key.tier.monthly_price_cents(),
            "cost_per_1k_tokens_cents": api_key.tier.cost_per_1k_tokens(),
            "cost_per_reasoning_step_cents": api_key.tier.cost_per_reasoning_step(),
        },
        "warning": "Save this API key! It will not be shown again.",
    }))))
}

/// GET /api/v1/health
/// 
/// Health check endpoint (no auth required)
pub async fn health_endpoint() -> ActixResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION"),
        "service": "SpatialVortex AGI API",
        "capabilities": [
            "reasoning",
            "causal_inference",
            "goal_planning",
            "transfer_learning",
            "meta_learning",
        ],
    })))
}

/// GET /api/v1/pricing
/// 
/// Get pricing information (no auth required)
pub async fn pricing_endpoint() -> ActixResult<HttpResponse> {
    let tiers = vec![
        (ApiTier::Free, "Free"),
        (ApiTier::Developer, "Developer"),
        (ApiTier::Professional, "Professional"),
        (ApiTier::Enterprise, "Enterprise"),
    ];
    
    let pricing: Vec<_> = tiers.iter().map(|(tier, name)| {
        serde_json::json!({
            "name": name,
            "monthly_price_usd": tier.monthly_price_cents() as f64 / 100.0,
            "features": tier.features(),
            "limits": {
                "requests_per_minute": tier.rate_limit_rpm(),
                "requests_per_day": tier.rate_limit_rpd(),
                "max_reasoning_steps": tier.max_reasoning_steps(),
                "max_tokens": tier.max_tokens(),
            },
            "usage_pricing": {
                "per_1k_tokens_usd": tier.cost_per_1k_tokens() as f64 / 100.0,
                "per_reasoning_step_usd": tier.cost_per_reasoning_step() as f64 / 100.0,
            },
        })
    }).collect();
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "currency": "USD",
        "tiers": pricing,
    })))
}

// ============================================================================
// Route Configuration
// ============================================================================

/// Configure production API routes
pub fn configure_production_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            // Public endpoints
            .route("/health", web::get().to(health_endpoint))
            .route("/pricing", web::get().to(pricing_endpoint))
            // Admin endpoints
            .route("/keys", web::post().to(create_api_key_endpoint))
            // Authenticated endpoints
            .route("/reason", web::post().to(reason_endpoint))
            .route("/causal/learn", web::post().to(causal_learn_endpoint))
            .route("/causal/counterfactual", web::post().to(counterfactual_endpoint))
            .route("/goal/create", web::post().to(create_goal_endpoint))
            .route("/usage", web::get().to(usage_endpoint))
    );
}
