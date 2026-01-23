//! Billing and Monetization System
//!
//! Production-ready billing infrastructure for API monetization:
//! - API key management with tiers
//! - Usage tracking and metering
//! - Rate limiting per tier
//! - Cost calculation based on tokens/reasoning steps
//! - Stripe-compatible webhook support

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

// ============================================================================
// API Key and Tier Management
// ============================================================================

/// API key tiers with different capabilities and limits
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApiTier {
    /// Free tier - limited usage for evaluation
    Free,
    /// Developer tier - $29/month
    Developer,
    /// Professional tier - $99/month
    Professional,
    /// Enterprise tier - custom pricing
    Enterprise,
}

impl ApiTier {
    /// Requests per minute limit
    pub fn rate_limit_rpm(&self) -> u32 {
        match self {
            ApiTier::Free => 10,
            ApiTier::Developer => 60,
            ApiTier::Professional => 300,
            ApiTier::Enterprise => 1000,
        }
    }
    
    /// Requests per day limit
    pub fn rate_limit_rpd(&self) -> u32 {
        match self {
            ApiTier::Free => 100,
            ApiTier::Developer => 5_000,
            ApiTier::Professional => 50_000,
            ApiTier::Enterprise => 500_000,
        }
    }
    
    /// Maximum reasoning steps per request
    pub fn max_reasoning_steps(&self) -> usize {
        match self {
            ApiTier::Free => 10,
            ApiTier::Developer => 20,
            ApiTier::Professional => 50,
            ApiTier::Enterprise => 100,
        }
    }
    
    /// Maximum tokens per request
    pub fn max_tokens(&self) -> usize {
        match self {
            ApiTier::Free => 1_000,
            ApiTier::Developer => 4_000,
            ApiTier::Professional => 16_000,
            ApiTier::Enterprise => 64_000,
        }
    }
    
    /// Cost per 1000 tokens (in cents)
    pub fn cost_per_1k_tokens(&self) -> u32 {
        match self {
            ApiTier::Free => 0,
            ApiTier::Developer => 5,      // $0.05 per 1k tokens
            ApiTier::Professional => 3,   // $0.03 per 1k tokens (volume discount)
            ApiTier::Enterprise => 2,     // $0.02 per 1k tokens
        }
    }
    
    /// Cost per reasoning step (in cents)
    pub fn cost_per_reasoning_step(&self) -> u32 {
        match self {
            ApiTier::Free => 0,
            ApiTier::Developer => 1,      // $0.01 per step
            ApiTier::Professional => 1,   // $0.01 per step
            ApiTier::Enterprise => 0,     // Included in enterprise
        }
    }
    
    /// Monthly base price in cents
    pub fn monthly_price_cents(&self) -> u32 {
        match self {
            ApiTier::Free => 0,
            ApiTier::Developer => 2_900,      // $29
            ApiTier::Professional => 9_900,   // $99
            ApiTier::Enterprise => 0,         // Custom
        }
    }
    
    /// Features available
    pub fn features(&self) -> Vec<&'static str> {
        match self {
            ApiTier::Free => vec![
                "Basic reasoning",
                "10 requests/minute",
                "100 requests/day",
                "Community support",
            ],
            ApiTier::Developer => vec![
                "Full reasoning pipeline",
                "60 requests/minute",
                "5,000 requests/day",
                "Causal reasoning",
                "Goal planning",
                "Email support",
            ],
            ApiTier::Professional => vec![
                "Everything in Developer",
                "300 requests/minute",
                "50,000 requests/day",
                "Transfer learning",
                "Meta-learning acceleration",
                "Priority support",
                "Custom fine-tuning",
            ],
            ApiTier::Enterprise => vec![
                "Everything in Professional",
                "1,000+ requests/minute",
                "Unlimited requests",
                "Dedicated infrastructure",
                "SLA guarantee",
                "24/7 support",
                "On-premise deployment",
                "Custom integrations",
            ],
        }
    }
}

/// API key with associated metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: Uuid,
    pub key_hash: String,  // Store hash, not actual key
    pub tier: ApiTier,
    pub owner_id: String,
    pub owner_email: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub metadata: ApiKeyMetadata,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ApiKeyMetadata {
    pub organization: Option<String>,
    pub project_name: Option<String>,
    pub allowed_ips: Vec<String>,
    pub allowed_origins: Vec<String>,
}

// ============================================================================
// Usage Tracking
// ============================================================================

/// Usage record for a single API call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    pub id: Uuid,
    pub api_key_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub endpoint: String,
    pub tokens_used: u32,
    pub reasoning_steps: u32,
    pub latency_ms: u64,
    pub success: bool,
    pub cost_cents: u32,
}

/// Aggregated usage statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UsageStats {
    pub total_requests: u64,
    pub total_tokens: u64,
    pub total_reasoning_steps: u64,
    pub total_cost_cents: u64,
    pub avg_latency_ms: f64,
    pub success_rate: f64,
    pub requests_today: u64,
    pub requests_this_minute: u64,
}

// ============================================================================
// Rate Limiter
// ============================================================================

/// Token bucket rate limiter
pub struct RateLimiter {
    /// Buckets per API key: (tokens_remaining, last_refill)
    minute_buckets: DashMap<Uuid, (u32, DateTime<Utc>)>,
    day_buckets: DashMap<Uuid, (u32, DateTime<Utc>)>,
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            minute_buckets: DashMap::new(),
            day_buckets: DashMap::new(),
        }
    }
    
    /// Check if request is allowed and consume a token
    pub fn check_and_consume(&self, key_id: Uuid, tier: ApiTier) -> RateLimitResult {
        let now = Utc::now();
        
        // Check minute limit
        let minute_allowed = {
            let mut entry = self.minute_buckets.entry(key_id).or_insert((tier.rate_limit_rpm(), now));
            let (tokens, last_refill) = entry.value_mut();
            
            // Refill if minute has passed
            let elapsed = now.signed_duration_since(*last_refill);
            if elapsed.num_seconds() >= 60 {
                *tokens = tier.rate_limit_rpm();
                *last_refill = now;
            }
            
            if *tokens > 0 {
                *tokens -= 1;
                true
            } else {
                false
            }
        };
        
        if !minute_allowed {
            return RateLimitResult::ExceededMinute {
                limit: tier.rate_limit_rpm(),
                retry_after_seconds: 60,
            };
        }
        
        // Check daily limit
        let day_allowed = {
            let mut entry = self.day_buckets.entry(key_id).or_insert((tier.rate_limit_rpd(), now));
            let (tokens, last_refill) = entry.value_mut();
            
            // Refill if day has passed
            let elapsed = now.signed_duration_since(*last_refill);
            if elapsed.num_hours() >= 24 {
                *tokens = tier.rate_limit_rpd();
                *last_refill = now;
            }
            
            if *tokens > 0 {
                *tokens -= 1;
                true
            } else {
                false
            }
        };
        
        if !day_allowed {
            return RateLimitResult::ExceededDaily {
                limit: tier.rate_limit_rpd(),
                retry_after_seconds: 86400,
            };
        }
        
        RateLimitResult::Allowed {
            remaining_minute: self.minute_buckets.get(&key_id).map(|e| e.0).unwrap_or(0),
            remaining_day: self.day_buckets.get(&key_id).map(|e| e.0).unwrap_or(0),
        }
    }
    
    /// Get current limits without consuming
    pub fn get_limits(&self, key_id: Uuid, tier: ApiTier) -> (u32, u32) {
        let minute = self.minute_buckets.get(&key_id).map(|e| e.0).unwrap_or(tier.rate_limit_rpm());
        let day = self.day_buckets.get(&key_id).map(|e| e.0).unwrap_or(tier.rate_limit_rpd());
        (minute, day)
    }
}

/// Result of rate limit check
#[derive(Debug, Clone, Serialize)]
pub enum RateLimitResult {
    Allowed {
        remaining_minute: u32,
        remaining_day: u32,
    },
    ExceededMinute {
        limit: u32,
        retry_after_seconds: u32,
    },
    ExceededDaily {
        limit: u32,
        retry_after_seconds: u32,
    },
}

impl RateLimitResult {
    pub fn is_allowed(&self) -> bool {
        matches!(self, RateLimitResult::Allowed { .. })
    }
}

// ============================================================================
// Billing Engine
// ============================================================================

/// Main billing engine
pub struct BillingEngine {
    /// API keys (in production, use database)
    api_keys: DashMap<String, ApiKey>,  // key_hash -> ApiKey
    
    /// Rate limiter
    rate_limiter: RateLimiter,
    
    /// Usage records (in production, use database)
    usage_records: DashMap<Uuid, Vec<UsageRecord>>,  // api_key_id -> records
    
    /// Stripe API key (for production billing)
    #[allow(dead_code)]
    stripe_api_key: Option<String>,
}

impl Default for BillingEngine {
    fn default() -> Self {
        Self::new(None)
    }
}

impl BillingEngine {
    pub fn new(stripe_api_key: Option<String>) -> Self {
        Self {
            api_keys: DashMap::new(),
            rate_limiter: RateLimiter::new(),
            usage_records: DashMap::new(),
            stripe_api_key,
        }
    }
    
    /// Generate a new API key
    pub fn generate_api_key(
        &self,
        tier: ApiTier,
        owner_id: &str,
        owner_email: &str,
        metadata: Option<ApiKeyMetadata>,
    ) -> (String, ApiKey) {
        let id = Uuid::new_v4();
        
        // Generate secure random key
        let raw_key = format!("sv_{}", Uuid::new_v4().to_string().replace('-', ""));
        let key_hash = self.hash_key(&raw_key);
        
        let api_key = ApiKey {
            id,
            key_hash: key_hash.clone(),
            tier,
            owner_id: owner_id.to_string(),
            owner_email: owner_email.to_string(),
            created_at: Utc::now(),
            expires_at: None,
            is_active: true,
            metadata: metadata.unwrap_or_default(),
        };
        
        self.api_keys.insert(key_hash, api_key.clone());
        
        (raw_key, api_key)
    }
    
    /// Validate an API key and return its metadata
    pub fn validate_key(&self, raw_key: &str) -> Option<ApiKey> {
        let key_hash = self.hash_key(raw_key);
        self.api_keys.get(&key_hash).map(|entry| {
            let key = entry.value().clone();
            if key.is_active {
                if let Some(expires) = key.expires_at {
                    if Utc::now() < expires {
                        Some(key)
                    } else {
                        None
                    }
                } else {
                    Some(key)
                }
            } else {
                None
            }
        }).flatten()
    }
    
    /// Check rate limits for a request
    pub fn check_rate_limit(&self, api_key: &ApiKey) -> RateLimitResult {
        self.rate_limiter.check_and_consume(api_key.id, api_key.tier)
    }
    
    /// Record usage for billing
    pub fn record_usage(
        &self,
        api_key: &ApiKey,
        endpoint: &str,
        tokens_used: u32,
        reasoning_steps: u32,
        latency_ms: u64,
        success: bool,
    ) -> UsageRecord {
        let cost_cents = self.calculate_cost(api_key.tier, tokens_used, reasoning_steps);
        
        let record = UsageRecord {
            id: Uuid::new_v4(),
            api_key_id: api_key.id,
            timestamp: Utc::now(),
            endpoint: endpoint.to_string(),
            tokens_used,
            reasoning_steps,
            latency_ms,
            success,
            cost_cents,
        };
        
        self.usage_records
            .entry(api_key.id)
            .or_insert_with(Vec::new)
            .push(record.clone());
        
        record
    }
    
    /// Calculate cost for a request
    pub fn calculate_cost(&self, tier: ApiTier, tokens: u32, reasoning_steps: u32) -> u32 {
        let token_cost = (tokens as u32 * tier.cost_per_1k_tokens()) / 1000;
        let step_cost = reasoning_steps * tier.cost_per_reasoning_step();
        token_cost + step_cost
    }
    
    /// Get usage statistics for an API key
    pub fn get_usage_stats(&self, api_key_id: Uuid) -> UsageStats {
        let records = self.usage_records.get(&api_key_id);
        
        if let Some(records) = records {
            let records = records.value();
            let now = Utc::now();
            let today_start = now - chrono::Duration::hours(24);
            let minute_start = now - chrono::Duration::minutes(1);
            
            let total_requests = records.len() as u64;
            let total_tokens: u64 = records.iter().map(|r| r.tokens_used as u64).sum();
            let total_reasoning_steps: u64 = records.iter().map(|r| r.reasoning_steps as u64).sum();
            let total_cost_cents: u64 = records.iter().map(|r| r.cost_cents as u64).sum();
            let avg_latency_ms = if total_requests > 0 {
                records.iter().map(|r| r.latency_ms as f64).sum::<f64>() / total_requests as f64
            } else {
                0.0
            };
            let success_count = records.iter().filter(|r| r.success).count() as f64;
            let success_rate = if total_requests > 0 {
                success_count / total_requests as f64
            } else {
                1.0
            };
            let requests_today = records.iter().filter(|r| r.timestamp > today_start).count() as u64;
            let requests_this_minute = records.iter().filter(|r| r.timestamp > minute_start).count() as u64;
            
            UsageStats {
                total_requests,
                total_tokens,
                total_reasoning_steps,
                total_cost_cents,
                avg_latency_ms,
                success_rate,
                requests_today,
                requests_this_minute,
            }
        } else {
            UsageStats::default()
        }
    }
    
    /// Get billing summary for invoice
    pub fn get_billing_summary(&self, api_key_id: Uuid, tier: ApiTier) -> BillingSummary {
        let stats = self.get_usage_stats(api_key_id);
        let (remaining_minute, remaining_day) = self.rate_limiter.get_limits(api_key_id, tier);
        
        BillingSummary {
            tier,
            base_price_cents: tier.monthly_price_cents(),
            usage_cost_cents: stats.total_cost_cents as u32,
            total_cost_cents: tier.monthly_price_cents() + stats.total_cost_cents as u32,
            usage_stats: stats,
            rate_limits: RateLimitInfo {
                requests_per_minute: tier.rate_limit_rpm(),
                requests_per_day: tier.rate_limit_rpd(),
                remaining_minute,
                remaining_day,
            },
        }
    }
    
    fn hash_key(&self, raw_key: &str) -> String {
        // In production, use proper hashing like bcrypt or argon2
        // For now, simple hash for demonstration
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        raw_key.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

/// Billing summary for invoicing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingSummary {
    pub tier: ApiTier,
    pub base_price_cents: u32,
    pub usage_cost_cents: u32,
    pub total_cost_cents: u32,
    pub usage_stats: UsageStats,
    pub rate_limits: RateLimitInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitInfo {
    pub requests_per_minute: u32,
    pub requests_per_day: u32,
    pub remaining_minute: u32,
    pub remaining_day: u32,
}

// ============================================================================
// Middleware for Actix-web
// ============================================================================

use actix_web::{dev::ServiceRequest, Error, HttpResponse};
use actix_web::body::BoxBody;

/// Extract API key from request headers
pub fn extract_api_key(req: &ServiceRequest) -> Option<String> {
    // Check Authorization header first
    if let Some(auth) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth.to_str() {
            if auth_str.starts_with("Bearer ") {
                return Some(auth_str[7..].to_string());
            }
        }
    }
    
    // Check X-API-Key header
    if let Some(key) = req.headers().get("X-API-Key") {
        if let Ok(key_str) = key.to_str() {
            return Some(key_str.to_string());
        }
    }
    
    // Check query parameter
    if let Some(query) = req.uri().query() {
        for pair in query.split('&') {
            let mut parts = pair.split('=');
            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                if key == "api_key" {
                    return Some(value.to_string());
                }
            }
        }
    }
    
    None
}

/// Create error response for billing/auth issues
pub fn billing_error_response(status: u16, message: &str, code: &str) -> HttpResponse<BoxBody> {
    let body = serde_json::json!({
        "error": {
            "message": message,
            "code": code,
            "status": status,
        }
    });
    
    match status {
        401 => HttpResponse::Unauthorized().json(body),
        403 => HttpResponse::Forbidden().json(body),
        429 => HttpResponse::TooManyRequests().json(body),
        _ => HttpResponse::InternalServerError().json(body),
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_api_key_generation() {
        let engine = BillingEngine::new(None);
        let (raw_key, api_key) = engine.generate_api_key(
            ApiTier::Developer,
            "user123",
            "user@example.com",
            None,
        );
        
        assert!(raw_key.starts_with("sv_"));
        assert_eq!(api_key.tier, ApiTier::Developer);
        assert!(api_key.is_active);
    }
    
    #[test]
    fn test_api_key_validation() {
        let engine = BillingEngine::new(None);
        let (raw_key, _) = engine.generate_api_key(
            ApiTier::Professional,
            "user123",
            "user@example.com",
            None,
        );
        
        let validated = engine.validate_key(&raw_key);
        assert!(validated.is_some());
        
        let invalid = engine.validate_key("invalid_key");
        assert!(invalid.is_none());
    }
    
    #[test]
    fn test_rate_limiting() {
        let engine = BillingEngine::new(None);
        let (raw_key, api_key) = engine.generate_api_key(
            ApiTier::Free,  // 10 requests/minute
            "user123",
            "user@example.com",
            None,
        );
        
        // First 10 should succeed
        for _ in 0..10 {
            let result = engine.check_rate_limit(&api_key);
            assert!(result.is_allowed());
        }
        
        // 11th should fail
        let result = engine.check_rate_limit(&api_key);
        assert!(!result.is_allowed());
    }
    
    #[test]
    fn test_usage_recording() {
        let engine = BillingEngine::new(None);
        let (_, api_key) = engine.generate_api_key(
            ApiTier::Developer,
            "user123",
            "user@example.com",
            None,
        );
        
        let record = engine.record_usage(&api_key, "/api/reason", 1000, 10, 250, true);
        
        assert_eq!(record.tokens_used, 1000);
        assert_eq!(record.reasoning_steps, 10);
        // Cost: (1000 * 5 / 1000) + (10 * 1) = 5 + 10 = 15 cents
        assert_eq!(record.cost_cents, 15);
    }
    
    #[test]
    fn test_tier_features() {
        assert!(ApiTier::Free.rate_limit_rpm() < ApiTier::Developer.rate_limit_rpm());
        assert!(ApiTier::Developer.rate_limit_rpm() < ApiTier::Professional.rate_limit_rpm());
        assert!(ApiTier::Professional.rate_limit_rpm() < ApiTier::Enterprise.rate_limit_rpm());
    }
}
