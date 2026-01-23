use std::task::{Context, Poll};
use std::time::Instant;
use actix_web::dev::{Service, Transform, ServiceRequest, ServiceResponse};
use actix_web::Error;
use futures::future::{LocalBoxFuture, Ready};
use futures::FutureExt;
use once_cell::sync::Lazy;
use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use prometheus::{Registry, IntCounter, IntCounterVec, IntGaugeVec, HistogramVec, Encoder, TextEncoder};

pub static REGISTRY: Lazy<Registry> = Lazy::new(Registry::new);
static TOTAL_REQUESTS: Lazy<AtomicU64> = Lazy::new(|| AtomicU64::new(0));
static TOTAL_ERRORS: Lazy<AtomicU64> = Lazy::new(|| AtomicU64::new(0));
pub static REQUESTS_TOTAL: Lazy<IntCounterVec> = Lazy::new(|| {
    let c = IntCounterVec::new(
        prometheus::Opts::new("http_requests_total", "Total HTTP requests"),
        &["method", "path", "status"],
    ).unwrap();
    REGISTRY.register(Box::new(c.clone())).ok();
    c
});

// VCP overflow risk occurrences
pub static VCP_OVERFLOW_RISK_TOTAL: Lazy<IntCounterVec> = Lazy::new(|| {
    let c = IntCounterVec::new(
        prometheus::Opts::new("vcp_overflow_risk_total", "Overflow risk escalations in VCP processing"),
        &["risk"],
    ).unwrap();
    REGISTRY.register(Box::new(c.clone())).ok();
    c
});

// MoE expert-level instrumentation
pub static ASI_EXPERT_SELECTED: Lazy<IntCounterVec> = Lazy::new(|| {
    let c = IntCounterVec::new(
        prometheus::Opts::new("asi_expert_selected_total", "Times an expert contributed to final result"),
        &["expert", "reason"],
    ).unwrap();
    REGISTRY.register(Box::new(c.clone())).ok();
    // Pre-create common label series to ensure visibility in tests and metrics readers
    let _ = c.with_label_values(&["geometric", "only"]);
    let _ = c.with_label_values(&["ml", "no_moe"]);
    let _ = c.with_label_values(&["moe", "baseline_kept"]);
    let _ = c.with_label_values(&["geometric", "higher_confidence"]);
    let _ = c.with_label_values(&["ml", "higher_confidence"]);
    let _ = c.with_label_values(&["geometric", "moe_weight"]);
    let _ = c.with_label_values(&["ml", "moe_weight"]);
    let _ = c.with_label_values(&["rag", "selected_by_moe"]);
    let _ = c.with_label_values(&["heuristic", "selected_by_moe"]);
    let _ = c.with_label_values(&["consensus", "selected_by_moe"]);
    c
});

pub static ASI_EXPERT_DURATION: Lazy<HistogramVec> = Lazy::new(|| {
    let h = HistogramVec::new(
        prometheus::HistogramOpts::new(
            "asi_expert_duration_seconds",
            "Expert execution duration in seconds"
        ).buckets(vec![0.001, 0.005, 0.01, 0.02, 0.05, 0.1, 0.25, 0.5]),
        &["expert"],
    ).unwrap();
    REGISTRY.register(Box::new(h.clone())).ok();
    h
});

pub static CACHE_HITS: Lazy<IntCounterVec> = Lazy::new(|| {
    let c = IntCounterVec::new(
        prometheus::Opts::new("cache_hits_total", "Cache hits total"),
        &["category"],
    ).unwrap();
    REGISTRY.register(Box::new(c.clone())).ok();
    c
});

pub static CACHE_MISSES: Lazy<IntCounterVec> = Lazy::new(|| {
    let c = IntCounterVec::new(
        prometheus::Opts::new("cache_misses_total", "Cache misses total"),
        &["category"],
    ).unwrap();
    REGISTRY.register(Box::new(c.clone())).ok();
    c
});

pub static CACHE_STORES: Lazy<IntCounterVec> = Lazy::new(|| {
    let c = IntCounterVec::new(
        prometheus::Opts::new("cache_stores_total", "Cache store operations total"),
        &["category"],
    ).unwrap();
    REGISTRY.register(Box::new(c.clone())).ok();
    c
});

// ASI-specific metrics
pub static ASI_INFER_TOTAL: Lazy<IntCounterVec> = Lazy::new(|| {
    let c = IntCounterVec::new(
        prometheus::Opts::new("asi_inferences_total", "Total ASI inferences"),
        &["mode"],
    ).unwrap();
    REGISTRY.register(Box::new(c.clone())).ok();
    c
});

pub static ASI_CONSENSUS_TOTAL: Lazy<IntCounter> = Lazy::new(|| {
    let c = IntCounter::new("asi_consensus_total", "ASI consensus triggers").unwrap();
    REGISTRY.register(Box::new(c.clone())).ok();
    c
});

pub static ASI_SACRED_HITS: Lazy<IntCounterVec> = Lazy::new(|| {
    let c = IntCounterVec::new(
        prometheus::Opts::new("asi_sacred_hits_total", "Sacred position occurrences during ASI"),
        &["position"],
    ).unwrap();
    REGISTRY.register(Box::new(c.clone())).ok();
    c
});

pub static ASI_INFERENCE_DURATION: Lazy<HistogramVec> = Lazy::new(|| {
    let h = HistogramVec::new(
        prometheus::HistogramOpts::new(
            "asi_inference_duration_seconds",
            "ASI inference duration in seconds"
        ).buckets(vec![0.01, 0.02, 0.05, 0.1, 0.25, 0.5, 1.0]),
        &["mode"],
    ).unwrap();
    REGISTRY.register(Box::new(h.clone())).ok();
    h
});

pub static INFLIGHT: Lazy<IntGaugeVec> = Lazy::new(|| {
    let g = IntGaugeVec::new(
        prometheus::Opts::new("http_inflight", "In-flight HTTP requests"),
        &["path"],
    ).unwrap();
    REGISTRY.register(Box::new(g.clone())).ok();
    g
});

pub static REQ_DURATION: Lazy<HistogramVec> = Lazy::new(|| {
    let h = HistogramVec::new(
        prometheus::HistogramOpts::new("http_request_duration_seconds", "HTTP request duration in seconds")
            .buckets(vec![0.005, 0.01, 0.02, 0.05, 0.1, 0.25, 0.5, 1.0, 2.0]),
        &["method", "path"],
    ).unwrap();
    REGISTRY.register(Box::new(h.clone())).ok();
    h
});

pub fn metrics_text() -> String {
    let metric_families = REGISTRY.gather();
    let mut buffer = Vec::new();
    let encoder = TextEncoder::new();
    encoder.encode(&metric_families, &mut buffer).ok();
    String::from_utf8(buffer).unwrap_or_default()
}

pub fn api_percentiles_all_routes(limit: usize) -> serde_json::Value {
    // Collect (path, counts snapshot)
    let mut items: Vec<(String, Vec<u64>)> = Vec::new();
    for entry in ROUTE_LAT.iter() {
        items.push((entry.key().clone(), entry.snapshot()));
    }
    // Rank by total count desc
    items.sort_by_key(|(_, counts)| std::cmp::Reverse(counts.iter().sum::<u64>()));
    let take_n = items.into_iter().take(limit);
    let routes: Vec<serde_json::Value> = take_n
        .map(|(path, counts)| {
            let (p50, p95, p99, total) = percentiles_from_counts(&counts);
            serde_json::json!({
                "path": path,
                "count": total,
                "p50_s": p50,
                "p95_s": p95,
                "p99_s": p99,
            })
        })
        .collect();
    serde_json::json!({ "routes": routes })
}

pub struct MetricsMiddleware;

impl<S, B> Transform<S, ServiceRequest> for MetricsMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = MetricsService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        futures::future::ready(Ok(MetricsService { service }))
    }
}

pub struct MetricsService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for MetricsService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let method = req.method().as_str().to_owned();
        let path = req.path().to_owned();
        INFLIGHT.with_label_values(&[path.as_str()]).inc();
        let start = Instant::now();

        let fut = self.service.call(req);
        async move {
            let res = fut.await;
            let elapsed = start.elapsed().as_secs_f64();
            REQ_DURATION.with_label_values(&[method.as_str(), path.as_str()]).observe(elapsed);
            record_latency(&path, elapsed);

            match &res {
                Ok(resp) => {
                    let status = resp.response().status().as_u16().to_string();
                    REQUESTS_TOTAL.with_label_values(&[method.as_str(), path.as_str(), status.as_str()]).inc();
                    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
                    if let Ok(code) = status.parse::<u16>() { if code >= 400 { TOTAL_ERRORS.fetch_add(1, Ordering::Relaxed); } }
                }
                Err(_) => {
                    REQUESTS_TOTAL.with_label_values(&[method.as_str(), path.as_str(), "error"]).inc();
                    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
                    TOTAL_ERRORS.fetch_add(1, Ordering::Relaxed);
                }
            }
            INFLIGHT.with_label_values(&[path.as_str()]).dec();
            res
        }
        .boxed_local()
    }
}

// =============================
// Latency Aggregator (Percentiles)
// =============================

// Buckets mirror histogram buckets for consistency (seconds)
static BUCKETS: &[f64] = &[0.005, 0.01, 0.02, 0.05, 0.1, 0.25, 0.5, 1.0, 2.0, 5.0];

struct BucketCounts {
    counts: Vec<AtomicU64>,
}

impl BucketCounts {
    fn new() -> Self { Self { counts: BUCKETS.iter().map(|_| AtomicU64::new(0)).collect() } }
    fn observe(&self, v: f64) {
        let idx = BUCKETS.iter().position(|b| v <= *b).unwrap_or(BUCKETS.len()-1);
        self.counts[idx].fetch_add(1, Ordering::Relaxed);
    }
    fn snapshot(&self) -> Vec<u64> { self.counts.iter().map(|c| c.load(Ordering::Relaxed)).collect() }
}

static GLOBAL_LAT: Lazy<BucketCounts> = Lazy::new(BucketCounts::new);
static ROUTE_LAT: Lazy<DashMap<String, BucketCounts>> = Lazy::new(DashMap::new);

pub fn record_latency(path: &str, seconds: f64) {
    GLOBAL_LAT.observe(seconds);
    let entry = ROUTE_LAT.entry(path.to_string()).or_insert_with(BucketCounts::new);
    entry.observe(seconds);
}

fn percentiles_from_counts(counts: &[u64]) -> (f64, f64, f64, u64) {
    let total: u64 = counts.iter().sum();
    if total == 0 { return (0.0, 0.0, 0.0, 0); }
    let p50_idx = ((total as f64) * 0.50).ceil() as u64;
    let p95_idx = ((total as f64) * 0.95).ceil() as u64;
    let p99_idx = ((total as f64) * 0.99).ceil() as u64;
    let mut acc = 0u64;
    let mut p50 = BUCKETS[0];
    let mut p95 = BUCKETS[0];
    let mut p99 = BUCKETS[0];
    for (i, c) in counts.iter().enumerate() {
        acc += *c;
        if acc >= p50_idx && p50 == BUCKETS[0] { p50 = BUCKETS[i]; }
        if acc >= p95_idx && p95 == BUCKETS[0] { p95 = BUCKETS[i]; }
        if acc >= p99_idx && p99 == BUCKETS[0] { p99 = BUCKETS[i]; break; }
    }
    (p50, p95, p99, total)
}

pub fn api_percentiles_overall() -> serde_json::Value {
    let snapshot = GLOBAL_LAT.snapshot();
    let (p50, p95, p99, total) = percentiles_from_counts(&snapshot);
    serde_json::json!({
        "count": total,
        "p50_s": p50,
        "p95_s": p95,
        "p99_s": p99,
    })
}

pub fn api_percentiles_for_route(path: &str) -> serde_json::Value {
    if let Some(entry) = ROUTE_LAT.get(path) {
        let snapshot = entry.snapshot();
        let (p50, p95, p99, total) = percentiles_from_counts(&snapshot);
        serde_json::json!({
            "path": path,
            "count": total,
            "p50_s": p50,
            "p95_s": p95,
            "p99_s": p99,
        })
    } else {
        serde_json::json!({"path": path, "count": 0, "p50_s": 0.0, "p95_s": 0.0, "p99_s": 0.0})
    }
}

pub fn api_error_rate_summary() -> serde_json::Value {
    let total = TOTAL_REQUESTS.load(Ordering::Relaxed);
    let errors = TOTAL_ERRORS.load(Ordering::Relaxed);
    let error_rate = if total > 0 { errors as f64 / total as f64 } else { 0.0 };
    serde_json::json!({
        "total": total,
        "errors": errors,
        "error_rate": error_rate,
    })
}
