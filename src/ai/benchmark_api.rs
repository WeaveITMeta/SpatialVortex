//! Benchmark API Endpoints
//!
//! Provides REST API for running benchmarks similar to chat interface pattern
//! Integrates with Meta Orchestrator for high-accuracy results

use actix_web::{web, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::error::Result;
use crate::ai::meta_orchestrator::{MetaOrchestrator, RoutingStrategy};
use crate::models::ELPTensor;

/// Benchmark request payload
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BenchmarkRequest {
    /// Benchmark type (MMLU, HumanEval, GeometricReasoning, etc.)
    pub benchmark_type: String,
    
    /// Input query/question
    pub query: String,
    
    /// Expected answer (for validation)
    pub expected_answer: Option<String>,
    
    /// Routing strategy override
    pub strategy: Option<RoutingStrategy>,
    
    /// Context/additional data
    pub context: Option<String>,
    
    /// Benchmark ID for tracking
    pub benchmark_id: Option<String>,
}

/// Benchmark response
#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkResponse {
    /// Generated answer
    pub answer: String,
    
    /// Confidence score and signal strength (0.0-1.0)
    pub confidence: f32,
    
    /// Flux position (0-9)
    pub flux_position: u8,
    
    /// ELP tensor
    pub elp: ELPTensor,
    
    /// Whether answer is correct (if expected_answer provided)
    pub is_correct: Option<bool>,
    
    /// Which orchestrator(s) were used
    pub orchestrators_used: String,
    
    /// Sacred position boost applied
    pub sacred_boost: bool,
    
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    
    /// Benchmark metadata
    pub metadata: BenchmarkMetadata,
}

/// Benchmark metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkMetadata {
    pub benchmark_type: String,
    pub benchmark_id: String,
    pub routing_strategy: String,
    pub timestamp: String,
}

/// Batch benchmark request (for running multiple benchmarks)
#[derive(Debug, Deserialize)]
pub struct BatchBenchmarkRequest {
    pub benchmarks: Vec<BenchmarkRequest>,
    pub parallel: Option<bool>, // Run in parallel if true
}

/// Batch benchmark response
#[derive(Debug, Serialize)]
pub struct BatchBenchmarkResponse {
    pub results: Vec<BenchmarkResponse>,
    pub summary: BatchSummary,
}

/// Summary of batch results
#[derive(Debug, Serialize)]
pub struct BatchSummary {
    pub total: usize,
    pub correct: usize,
    pub incorrect: usize,
    pub accuracy: f64,
    pub avg_confidence: f64,
    pub avg_processing_time_ms: f64,
    pub total_time_ms: u64,
}

/// Single benchmark endpoint
///
/// # Example
/// ```
/// POST /api/v1/benchmark
/// {
///   "benchmark_type": "GeometricReasoning",
///   "query": "What position represents harmonic balance?",
///   "expected_answer": "6",
///   "strategy": "ParallelFusion"
/// }
/// ```
pub async fn run_single_benchmark(
    meta: web::Data<Arc<RwLock<MetaOrchestrator>>>,
    req: web::Json<BenchmarkRequest>,
) -> ActixResult<HttpResponse> {
    let start = std::time::Instant::now();
    
    // Override strategy if provided (before acquiring read lock)
    if let Some(strategy) = req.strategy {
        let mut meta_mut = meta.write().await;
        meta_mut.set_strategy(strategy);
    }
    
    // Get Meta Orchestrator for processing
    let meta_orchestrator = meta.read().await;
    
    // Process via Meta Orchestrator
    let result = match meta_orchestrator.process_unified(&req.query).await {
        Ok(r) => r,
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Benchmark processing failed: {}", e),
            })));
        }
    };
    
    let processing_time_ms = start.elapsed().as_millis() as u64;
    
    // Check correctness if expected answer provided
    let is_correct = req.expected_answer.as_ref().map(|expected| {
        result.content.trim().eq_ignore_ascii_case(expected.trim())
    });
    
    // Update metrics
    if let Some(correct) = is_correct {
        meta_orchestrator.update_metrics(
            &result.orchestrators_used,
            correct,
            processing_time_ms
        ).await;
    }
    
    let response = BenchmarkResponse {
        answer: result.content,
        confidence: result.confidence,
        flux_position: result.flux_position,
        elp: result.elp,
        is_correct,
        orchestrators_used: format!("{:?}", result.orchestrators_used),
        sacred_boost: result.sacred_boost,
        processing_time_ms,
        metadata: BenchmarkMetadata {
            benchmark_type: req.benchmark_type.clone(),
            benchmark_id: req.benchmark_id.clone()
                .unwrap_or_else(|| format!("bench_{}", chrono::Utc::now().timestamp())),
            routing_strategy: format!("{:?}", meta_orchestrator.strategy()),
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
    };
    
    Ok(HttpResponse::Ok().json(response))
}

/// Batch benchmark endpoint (optimized for running test suites)
///
/// # Example
/// ```
/// POST /api/v1/benchmark/batch
/// {
///   "benchmarks": [
///     {"benchmark_type": "GeometricReasoning", "query": "...", "expected_answer": "..."},
///     {"benchmark_type": "MMLU", "query": "...", "expected_answer": "..."}
///   ],
///   "parallel": true
/// }
/// ```
pub async fn run_batch_benchmarks(
    meta: web::Data<Arc<RwLock<MetaOrchestrator>>>,
    req: web::Json<BatchBenchmarkRequest>,
) -> ActixResult<HttpResponse> {
    let batch_start = std::time::Instant::now();
    let parallel = req.parallel.unwrap_or(false);
    
    let results = if parallel {
        // Run benchmarks in parallel
        let mut futures = Vec::new();
        
        for bench_req in &req.benchmarks {
            let meta_clone = meta.clone();
            let bench_req_clone = bench_req.clone();
            
            futures.push(tokio::spawn(async move {
                process_single_benchmark(meta_clone, bench_req_clone).await
            }));
        }
        
        let mut results = Vec::new();
        for future in futures {
            match future.await {
                Ok(Ok(result)) => results.push(result),
                Ok(Err(e)) => {
                    eprintln!("Benchmark error: {:?}", e);
                    continue;
                }
                Err(e) => {
                    eprintln!("Task join error: {:?}", e);
                    continue;
                }
            }
        }
        results
    } else {
        // Run benchmarks sequentially
        let mut results = Vec::new();
        for bench_req in &req.benchmarks {
            match process_single_benchmark(meta.clone(), bench_req.clone()).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    eprintln!("Benchmark error: {:?}", e);
                    continue;
                }
            }
        }
        results
    };
    
    let total_time_ms = batch_start.elapsed().as_millis() as u64;
    
    // Calculate summary
    let total = results.len();
    let correct = results.iter().filter(|r| r.is_correct == Some(true)).count();
    let incorrect = results.iter().filter(|r| r.is_correct == Some(false)).count();
    let accuracy = if total > 0 {
        correct as f64 / total as f64
    } else {
        0.0
    };
    
    let avg_confidence = if total > 0 {
        results.iter().map(|r| r.confidence as f64).sum::<f64>() / total as f64
    } else {
        0.0
    };
    
    let avg_processing_time_ms = if total > 0 {
        results.iter().map(|r| r.processing_time_ms as f64).sum::<f64>() / total as f64
    } else {
        0.0
    };
    
    let response = BatchBenchmarkResponse {
        results,
        summary: BatchSummary {
            total,
            correct,
            incorrect,
            accuracy,
            avg_confidence,
            avg_processing_time_ms,
            total_time_ms,
        },
    };
    
    Ok(HttpResponse::Ok().json(response))
}

/// Helper to process a single benchmark
async fn process_single_benchmark(
    meta: web::Data<Arc<RwLock<MetaOrchestrator>>>,
    req: BenchmarkRequest,
) -> Result<BenchmarkResponse> {
    let start = std::time::Instant::now();
    
    let meta_orchestrator = meta.read().await;
    
    let result = meta_orchestrator.process_unified(&req.query).await?;
    
    let processing_time_ms = start.elapsed().as_millis() as u64;
    
    let is_correct = req.expected_answer.as_ref().map(|expected| {
        result.content.trim().eq_ignore_ascii_case(expected.trim())
    });
    
    if let Some(correct) = is_correct {
        meta_orchestrator.update_metrics(
            &result.orchestrators_used,
            correct,
            processing_time_ms
        ).await;
    }
    
    Ok(BenchmarkResponse {
        answer: result.content,
        confidence: result.confidence,
        flux_position: result.flux_position,
        elp: result.elp,
        is_correct,
        orchestrators_used: format!("{:?}", result.orchestrators_used),
        sacred_boost: result.sacred_boost,
        processing_time_ms,
        metadata: BenchmarkMetadata {
            benchmark_type: req.benchmark_type.clone(),
            benchmark_id: req.benchmark_id
                .unwrap_or_else(|| format!("bench_{}", chrono::Utc::now().timestamp())),
            routing_strategy: format!("{:?}", meta_orchestrator.strategy()),
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
    })
}

/// Configure benchmark routes
pub fn configure_benchmark_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/api/v1/benchmark")
            .route(web::post().to(run_single_benchmark))
    )
    .service(
        web::resource("/api/v1/benchmark/batch")
            .route(web::post().to(run_batch_benchmarks))
    );
}
