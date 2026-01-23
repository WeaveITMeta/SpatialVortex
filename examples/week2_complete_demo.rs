//! Week 2 Complete Demo
//!
//! Demonstrates all Week 2 enhancements:
//! - Enhanced error handling with context
//! - Prometheus metrics tracking
//! - Structured logging
//! - Unified API types

use spatial_vortex::{
    ai::{
        meta_orchestrator::{MetaOrchestrator, RoutingStrategy},
        unified_api::{UnifiedRequest, ExecutionMode},
    },
    error::{ErrorContext, RecoveryStrategy},
    monitoring::{VORTEX_METRICS, init_logging, LogConfig},
};
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize structured logging (development mode)
    init_logging(LogConfig::development())?;
    
    info!("Starting Week 2 Complete Demo");
    
    // Create meta orchestrator
    let meta = MetaOrchestrator::new(RoutingStrategy::Hybrid).await?;
    
    // Demonstrate unified API request building
    let request = UnifiedRequest::builder()
        .input("What is the meaning of sacred geometry in AI?")
        .mode(ExecutionMode::Balanced)
        .strategy(RoutingStrategy::Hybrid)
        .sacred_only(false)
        .min_confidence(0.6)
        .build()
        .expect("Failed to build request");
    
    info!(
        request_len = request.input.len(),
        mode = ?request.mode,
        strategy = ?request.strategy,
        "Built unified request"
    );
    
    // Process request with full observability
    match meta.process_unified(&request.input).await {
        Ok(result) => {
            // Record successful metrics
            VORTEX_METRICS.record_meta_request(
                &format!("{:?}", meta.strategy()),
                &format!("{:?}", result.orchestrators_used),
                result.duration_ms as f64 / 1000.0,
                true,
            );
            
            // Record flux position
            VORTEX_METRICS.record_flux_position(result.flux_position);
            
            // Record signal strength
            VORTEX_METRICS.record_confidence(
                result.confidence,
                &format!("{:?}", result.orchestrators_used),
            );
            
            // If sacred position, record sacred hit
            if result.sacred_boost {
                VORTEX_METRICS.record_sacred_hit(
                    result.flux_position,
                    0.15, // Example boost value
                );
            }
            
            // Log success with structured data
            info!(
                result_len = result.content.len(),
                confidence = result.confidence,
                flux_position = result.flux_position,
                sacred_boost = result.sacred_boost,
                confidence = result.confidence,
                duration_ms = result.duration_ms,
                source = ?result.orchestrators_used,
                "Request completed successfully"
            );
            
            println!("\n========== RESULT ==========");
            println!("Content: {}", result.content);
            println!("Confidence: {:.2}", result.confidence);
            println!("Flux Position: {}", result.flux_position);
            println!("Sacred Boost: {}", result.sacred_boost);
            println!("Confidence: {:.2}", result.confidence);
            println!("Duration: {}ms", result.duration_ms);
            println!("Source: {:?}", result.orchestrators_used);
            println!("============================\n");
        }
        
        Err(e) => {
            // Demonstrate error context handling
            error!(
                error = ?e,
                error_type = std::any::type_name_of_val(&e),
                recovery_strategy = ?e.recovery_strategy(),
                sacred_position = e.is_at_sacred_position(),
                flux_position = ?e.flux_position(),
                "Request failed"
            );
            
            // Record error metrics
            VORTEX_METRICS.record_error(
                "processing",
                "meta_orchestrator",
                &format!("{:?}", e.recovery_strategy()),
            );
            
            // Demonstrate recovery strategy
            match e.recovery_strategy() {
                RecoveryStrategy::Retry => {
                    println!("🔄 Error is retryable, attempting retry...");
                    // Could retry here
                }
                RecoveryStrategy::Fallback => {
                    println!("⚡ Using fallback strategy...");
                    // Could use fallback here
                }
                RecoveryStrategy::Propagate => {
                    println!("❌ Critical error, propagating...");
                    return Err(e.into());
                }
                RecoveryStrategy::Ignore => {
                    println!("ℹ️ Non-critical error, continuing...");
                }
            }
        }
    }
    
    // Demonstrate error context creation
    println!("\n========== ERROR CONTEXT DEMO ==========");
    
    let context = ErrorContext::new()
        .with_flux_position(6)
        .with_confidence(0.75)
        .with_component("MetaOrchestrator")
        .with_operation("process_unified");
    
    println!("Error Context: {}", context);
    println!("Sacred Position: {}", context.sacred_position);
    println!("=========================================\n");
    
    // Demonstrate metrics access
    println!("========== PERFORMANCE METRICS ==========");
    let perf_metrics = meta.metrics().await;
    println!("ASI Success Rate: {:.1}%", perf_metrics.asi_success_rate * 100.0);
    println!("ASI Avg Latency: {:.0}ms", perf_metrics.asi_avg_latency_ms);
    println!("Runtime Success Rate: {:.1}%", perf_metrics.runtime_success_rate * 100.0);
    println!("Runtime Avg Latency: {:.0}ms", perf_metrics.runtime_avg_latency_ms);
    println!("==========================================\n");
    
    // Test different routing strategies
    println!("========== TESTING ROUTING STRATEGIES ==========");
    
    let strategies = vec![
        ("AIFirst", RoutingStrategy::AIFirst),
        ("RuntimeFirst", RoutingStrategy::RuntimeFirst),
        ("ParallelFusion", RoutingStrategy::ParallelFusion),
    ];
    
    for (name, strategy) in strategies {
        let meta_strategy = MetaOrchestrator::new(strategy).await?;
        
        info!(
            strategy = name,
            "Testing routing strategy"
        );
        
        match meta_strategy.process_unified("Quick test").await {
            Ok(result) => {
                println!("✅ {} - {}ms, confidence: {:.2}", 
                    name, result.duration_ms, result.confidence);
                
                // Record routing decision
                VORTEX_METRICS.record_routing(
                    name,
                    &format!("{:?}", result.orchestrators_used),
                );
            }
            Err(e) => {
                println!("❌ {} - Error: {}", name, e);
            }
        }
    }
    
    println!("=================================================\n");
    
    info!("Week 2 Complete Demo finished successfully");
    
    Ok(())
}
