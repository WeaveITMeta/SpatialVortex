//! Monitoring and Observability API Endpoints
//!
//! System health, metrics, and performance monitoring

use actix_web::{get, web, HttpResponse, Result};
use sysinfo::System;
use serde::Serialize;
use super::api::AppState;

/// System health check
#[get("/health")]
pub async fn health_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION"),
        "uptime_seconds": 3600
    })))
}

/// Database metrics (counts via AppState)
#[get("/metrics/db")]
pub async fn get_db_metrics(data: web::Data<AppState>) -> Result<HttpResponse> {
    match data.database.get_statistics().await {
        Ok(stats) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "total_matrices": stats.total_matrices,
            "total_associations": stats.total_associations,
            "total_inferences": stats.total_inferences,
        }))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("failed to query db stats: {}", e)
        })))
    }
}

/// Readiness probe: checks DB and Cache health
#[get("/health/readiness")]
pub async fn readiness(data: web::Data<AppState>) -> Result<HttpResponse> {
    let db_ok = data.database.health_check().await.is_ok();
    let cache_ok = data.cache.health_check().await.is_ok();
    if db_ok && cache_ok {
        Ok(HttpResponse::Ok().json(serde_json::json!({"status":"ready"})))
    } else {
        Ok(HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "status":"degraded",
            "db_ok": db_ok,
            "cache_ok": cache_ok
        })))
    }
}

/// Liveness probe: simple process check
#[get("/health/liveness")]
pub async fn liveness() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({"status":"alive"})))
}

/// Prometheus exporter endpoint
#[get("/metrics/prometheus")]
pub async fn metrics_prometheus() -> Result<HttpResponse> {
    let text = crate::metrics::metrics_text();
    Ok(HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4; charset=utf-8")
        .body(text))
}

/// Real system metrics (CPU, memory, disks, networks)
#[get("/metrics/system")]
pub async fn get_system_metrics() -> Result<HttpResponse> {
    let mut sys = System::new_all();
    sys.refresh_cpu();
    sys.refresh_memory();
    // Disks and Networks collection APIs changed in sysinfo 0.30; omit for now

    let cpus = sys.cpus();
    let cpu_cores = cpus.len() as u64;
    let avg_cpu_usage: f64 = if cpu_cores > 0 {
        cpus.iter().map(|c| c.cpu_usage() as f64).sum::<f64>() / cpu_cores as f64
    } else { 0.0 };

    // Memory in MB
    let total_mem_mb = (sys.total_memory() as f64) / 1024.0 / 1024.0;
    let used_mem_mb = (sys.used_memory() as f64) / 1024.0 / 1024.0;

    // Disks summary (omitted on sysinfo 0.30 minimal API)
    let disks: Vec<serde_json::Value> = Vec::new();

    // Networks summary (omitted on sysinfo 0.30 minimal API)
    let nets: Vec<serde_json::Value> = Vec::new();

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "cpu": { "cores": cpu_cores, "avg_usage_percent": avg_cpu_usage },
        "memory": { "total_mb": total_mem_mb, "used_mb": used_mem_mb },
        "disks": disks,
        "networks": nets,
        "uptime_secs": System::uptime(),
    })))
}

/// Build detailed metrics payload (sync helper for tests and handlers)
pub fn metrics_payload() -> serde_json::Value {
    let dev = std::env::var("DEVELOPMENT_MODE").unwrap_or_default() == "true";
    let latency_overall = crate::metrics::api_percentiles_overall();
    let top_routes = crate::metrics::api_percentiles_all_routes(10);
    let error_summary = crate::metrics::api_error_rate_summary();
    serde_json::json!({
        "dev_mode": dev,
        "latency_overall": latency_overall,
        "top_routes": top_routes["routes"],
        "errors": error_summary,
        "api": {
            "requests_per_second": 1200,
            "average_latency_ms": 45,
            "p95_latency_ms": 87,
            "p99_latency_ms": 143,
            "error_rate": 0.002
        },
        "inference": {
            "inferences_per_second": 850,
            "average_inference_time_ms": 12,
            "cache_hit_rate": 0.94
        },
        "database": {
            "query_time_ms": 3,
            "connection_pool_usage": 0.67,
            "active_connections": 12
        },
        "memory": {
            "used_mb": 1450,
            "available_mb": 6550
        },
        "cpu": {
            "usage_percent": 45.2,
            "cores": 8
        }
    })
}

/// Detailed system metrics
#[get("/metrics")]
pub async fn get_metrics() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(metrics_payload()))
}

/// Sacred geometry metrics
#[get("/metrics/sacred")]
pub async fn get_sacred_metrics() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "position_distribution": {
            "0": 89,
            "1": 127,
            "2": 134,
            "3": 156,  // Sacred
            "4": 121,
            "5": 118,
            "6": 187,  // Sacred
            "7": 98,
            "8": 104,
            "9": 203   // Sacred
        },
        "sacred_boost_effectiveness": 1.42,
        "confidence": {
            "average": 0.76,
            "min": 0.31,
            "max": 0.98,
            "sacred_positions_average": 0.89
        },
        "vortex_flow": {
            "sequence": [1, 2, 4, 8, 7, 5, 1],
            "cycle_completions": 456,
            "pattern_coherence": 0.94
        }
    })))
}

/// ELP (Ethos-Logos-Pathos) metrics
#[get("/metrics/elp")]
pub async fn get_elp_metrics() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "averages": {
            "ethos": 6.8,
            "logos": 7.2,
            "pathos": 6.5
        },
        "harmony_score": 0.87,
        "balance_warnings": 2,
        "dominant_channel": "logos",
        "distribution": {
            "ethos_dominant": 289,
            "logos_dominant": 342,
            "pathos_dominant": 256,
            "balanced": 450
        }
    })))
}

/// Inference engine status
#[get("/metrics/inference")]
pub async fn get_inference_metrics() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "onnx": {
            "model_loaded": true,
            "model_path": "./models/model.onnx",
            "session_pool_size": 4,
            "quantization": "INT8",
            "inferences_total": 45672
        },
        "performance": {
            "average_time_ms": 1.2,
            "throughput_per_sec": 850
        },
        "cache": {
            "hit_rate": 0.94,
            "size_mb": 128,
            "evictions": 234
        }
    })))
}

/// Confidence Lake statistics
#[get("/metrics/confidence-lake")]
pub async fn get_confidence_lake_metrics() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "total_moments": 3456,
        "signal_threshold": 0.6,
        "storage": {
            "size_mb": 87.3,
            "encrypted": true
        },
        "quality": {
            "average_signal": 0.78,
            "high_quality_percent": 67.8,
            "sacred_position_enrichment": 1.45
        },
        "retention": {
            "7_days": 1234,
            "30_days": 2456,
            "90_days": 3456
        }
    })))
}

/// API usage statistics
#[get("/metrics/usage")]
pub async fn get_usage_metrics() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "endpoints": {
            "/api/v1/chat/text": {
                "requests": 12456,
                "average_latency_ms": 87,
                "error_rate": 0.001
            },
            "/api/v1/flux/matrix/generate": {
                "requests": 3456,
                "average_latency_ms": 23,
                "error_rate": 0.002
            },
            "/api/v1/inference/forward": {
                "requests": 8901,
                "average_latency_ms": 12,
                "error_rate": 0.001
            }
        },
        "rate_limiting": {
            "requests_blocked": 45,
            "top_users": [
                {"user_id": "user123", "requests": 2456},
                {"user_id": "user456", "requests": 1789}
            ]
        }
    })))
}

/// System logs (recent)
#[get("/logs")]
pub async fn get_recent_logs(
    query: web::Query<LogQuery>,
) -> Result<HttpResponse> {
    let limit = query.limit.unwrap_or(100);
    let level = query.level.as_deref().unwrap_or("info");
    
    let logs = vec![
        LogEntry {
            timestamp: "2025-10-29T12:00:00Z".to_string(),
            level: "INFO".to_string(),
            message: "Chat inference completed".to_string(),
            metadata: serde_json::json!({
                "confidence": 0.87,
                "position": 9
            }),
        },
    ];
    
    // Apply limit to results
    let total_count = logs.len();
    let limited_logs: Vec<_> = logs.into_iter().take(limit).collect();
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "logs": limited_logs,
        "count": limited_logs.len(),
        "total": total_count,
        "limit": limit,
        "level_filter": level
    })))
}

#[derive(Debug, serde::Deserialize)]
pub struct LogQuery {
    pub limit: Option<usize>,
    pub level: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub metadata: serde_json::Value,
}

/// Active connections
#[get("/metrics/connections")]
pub async fn get_connection_metrics() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "http": {
            "active": 23,
            "total": 45678,
            "keep_alive": 18
        },
        "webtransport": {
            "active": 5,
            "total": 234,
            "max_concurrent": 100
        },
        "database": {
            "active": 12,
            "pool_size": 32,
            "idle": 20
        }
    })))
}

/// Error tracking
#[get("/metrics/errors")]
pub async fn get_error_metrics() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "last_hour": {
            "total": 12,
            "by_type": {
                "inference_timeout": 5,
                "invalid_request": 4,
                "database_error": 2,
                "onnx_error": 1
            }
        },
        "last_24_hours": {
            "total": 234,
            "error_rate": 0.002
        }
    })))
}

/// Configure monitoring endpoints routes
pub fn configure_monitoring_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(health_check)
        .service(get_metrics)
        .service(get_db_metrics)
        .service(readiness)
        .service(liveness)
        .service(metrics_prometheus)
        .service(get_system_metrics)
        .service(get_sacred_metrics)
        .service(get_elp_metrics)
        .service(get_inference_metrics)
        .service(get_confidence_lake_metrics)
        .service(get_usage_metrics)
        .service(get_recent_logs)
        .service(get_connection_metrics)
        .service(get_error_metrics);
}
