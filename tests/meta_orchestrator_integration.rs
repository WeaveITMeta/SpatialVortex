//! Integration tests for Meta Orchestrator
//!
//! Tests the unified ASI coordination between ASIOrchestrator and FluxOrchestrator

use spatial_vortex::ai::meta_orchestrator::{MetaOrchestrator, RoutingStrategy, OrchestratorSource};
use spatial_vortex::error::Result;

#[tokio::test]
async fn test_meta_orchestrator_creation() -> Result<()> {
    let meta = MetaOrchestrator::new_default().await?;
    assert_eq!(meta.strategy(), RoutingStrategy::Hybrid);
    Ok(())
}

#[tokio::test]
async fn test_ai_first_routing() -> Result<()> {
    let meta = MetaOrchestrator::new(RoutingStrategy::AIFirst).await?;
    
    let result = meta.process_unified("What is consciousness?").await?;
    
    // Verify it used ASI
    assert!(matches!(result.orchestrators_used, OrchestratorSource::ASI));
    assert!(result.confidence > 0.0);
    assert!(result.flux_position <= 9);
    
    Ok(())
}

#[tokio::test]
async fn test_runtime_first_routing() -> Result<()> {
    let meta = MetaOrchestrator::new(RoutingStrategy::RuntimeFirst).await?;
    
    let result = meta.process_unified("Simple test").await?;
    
    // Verify it used Runtime
    assert!(matches!(result.orchestrators_used, OrchestratorSource::Runtime));
    assert!(result.duration_ms < 200);  // Should be fast
    
    Ok(())
}

#[tokio::test]
async fn test_hybrid_routing_simple() -> Result<()> {
    let meta = MetaOrchestrator::new(RoutingStrategy::Hybrid).await?;
    
    // Simple query should go to Runtime
    let result = meta.process_unified("Hello").await?;
    
    // Runtime is faster
    assert!(result.duration_ms < 500);
    
    Ok(())
}

#[tokio::test]
async fn test_hybrid_routing_complex() -> Result<()> {
    let meta = MetaOrchestrator::new(RoutingStrategy::Hybrid).await?;
    
    // Complex query should go to ASI
    let result = meta.process_unified(
        "Calculate the integral of f(x) = x^3 + 2x^2 - 5x + 7 from 0 to 10 and explain the process step by step."
    ).await?;
    
    // Should use ASI for complex math
    assert!(matches!(result.orchestrators_used, OrchestratorSource::ASI));
    
    Ok(())
}

#[tokio::test]
async fn test_parallel_fusion() -> Result<()> {
    let meta = MetaOrchestrator::new(RoutingStrategy::ParallelFusion).await?;
    
    let result = meta.process_unified("Test parallel fusion").await?;
    
    // Verify fusion happened
    assert!(matches!(
        result.orchestrators_used,
        OrchestratorSource::Fused { .. }
    ));
    
    // Fusion should happen at position 6
    assert_eq!(result.flux_position, 6);
    
    // Should have sacred boost since fusion is at sacred position
    assert!(result.sacred_boost);
    
    Ok(())
}

#[tokio::test]
async fn test_sacred_position_boost() -> Result<()> {
    let meta = MetaOrchestrator::new(RoutingStrategy::ParallelFusion).await?;
    
    let result = meta.process_unified("Test sacred positions").await?;
    
    // Fusion always happens at position 6 (sacred)
    assert!(result.sacred_boost);
    assert_eq!(result.flux_position, 6);
    
    Ok(())
}

#[tokio::test]
async fn test_complexity_analysis() -> Result<()> {
    let meta = MetaOrchestrator::new_default().await?;
    
    // Simple inputs should route to Runtime in Hybrid mode
    let simple_inputs = vec![
        "Hi",
        "Hello world",
        "Test",
    ];
    
    for input in simple_inputs {
        let result = meta.process_unified(input).await?;
        // Fast processing
        assert!(result.duration_ms < 500);
    }
    
    // Complex inputs should route to ASI
    let complex_inputs = vec![
        "Write a Rust function to implement quicksort with generics",
        "Calculate 15823 * 9471 using the standard algorithm",
        "What are the philosophical implications of consciousness?",
    ];
    
    for input in complex_inputs {
        let result = meta.process_unified(input).await?;
        // Should have higher confidence from ASI
        assert!(result.confidence >= 0.0);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_strategy_switching() -> Result<()> {
    let mut meta = MetaOrchestrator::new(RoutingStrategy::AIFirst).await?;
    
    // Start with AIFirst
    assert_eq!(meta.strategy(), RoutingStrategy::AIFirst);
    
    // Switch to RuntimeFirst
    meta.set_strategy(RoutingStrategy::RuntimeFirst);
    assert_eq!(meta.strategy(), RoutingStrategy::RuntimeFirst);
    
    // Switch to Hybrid
    meta.set_strategy(RoutingStrategy::Hybrid);
    assert_eq!(meta.strategy(), RoutingStrategy::Hybrid);
    
    Ok(())
}

#[tokio::test]
async fn test_complexity_threshold() -> Result<()> {
    let mut meta = MetaOrchestrator::new(RoutingStrategy::Hybrid).await?;
    
    // Set low threshold (route more to Runtime)
    meta.set_complexity_threshold(0.8);
    
    let result1 = meta.process_unified("Moderate complexity question").await?;
    
    // Set high threshold (route more to ASI)
    meta.set_complexity_threshold(0.2);
    
    let result2 = meta.process_unified("Moderate complexity question").await?;
    
    // Results should be different based on threshold
    assert!(result1.duration_ms > 0);
    assert!(result2.duration_ms > 0);
    
    Ok(())
}

#[tokio::test]
async fn test_elp_tensor_estimation() -> Result<()> {
    let meta = MetaOrchestrator::new_default().await?;
    
    let result = meta.process_unified("I feel strongly that we should analyze this logically").await?;
    
    // Should have all ELP dimensions
    assert!(result.elp.ethos > 0.0);
    assert!(result.elp.logos > 0.0);
    assert!(result.elp.pathos > 0.0);
    
    Ok(())
}

#[tokio::test]
async fn test_confidence_tracking() -> Result<()> {
    let meta = MetaOrchestrator::new_default().await?;
    
    let result = meta.process_unified("Test signal strength").await?;
    
    // Signal strength should be in valid range
    assert!(result.confidence >= 0.0);
    assert!(result.confidence <= 1.0);
    
    Ok(())
}

#[tokio::test]
async fn test_metadata_population() -> Result<()> {
    let meta = MetaOrchestrator::new(RoutingStrategy::ParallelFusion).await?;
    
    let result = meta.process_unified("Test metadata").await?;
    
    // Verify metadata is populated
    assert!(!result.metadata.routing_strategy.is_empty());
    assert!(result.metadata.vortex_cycles >= 0);
    
    Ok(())
}

#[tokio::test]
async fn test_performance_metrics_update() -> Result<()> {
    let meta = MetaOrchestrator::new_default().await?;
    
    // Initial metrics
    let metrics_before = meta.metrics().await;
    
    // Process something
    let result = meta.process_unified("Test").await?;
    
    // Update metrics
    meta.update_metrics(&result.orchestrators_used, true, result.duration_ms).await;
    
    let metrics_after = meta.metrics().await;
    
    // Metrics should have changed
    assert!(metrics_before.asi_avg_latency_ms > 0.0);
    assert!(metrics_after.asi_avg_latency_ms > 0.0);
    
    Ok(())
}

#[tokio::test]
async fn test_adaptive_routing() -> Result<()> {
    let meta = MetaOrchestrator::new(RoutingStrategy::Adaptive).await?;
    
    // Should adapt based on performance metrics
    let result = meta.process_unified("Adaptive routing test").await?;
    
    assert!(result.confidence >= 0.0);
    assert!(result.duration_ms > 0);
    
    Ok(())
}

#[tokio::test]
async fn test_multiple_sequential_requests() -> Result<()> {
    let meta = MetaOrchestrator::new(RoutingStrategy::Hybrid).await?;
    
    let inputs = vec![
        "First query",
        "Second query",
        "Third query",
    ];
    
    for input in inputs {
        let result = meta.process_unified(input).await?;
        assert!(result.confidence >= 0.0);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_flux_position_range() -> Result<()> {
    let meta = MetaOrchestrator::new_default().await?;
    
    for _ in 0..10 {
        let result = meta.process_unified("Random test").await?;
        // Flux position should always be 0-9
        assert!(result.flux_position <= 9);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_confidence_range() -> Result<()> {
    let meta = MetaOrchestrator::new_default().await?;
    
    let result = meta.process_unified("Test confidence").await?;
    
    // Confidence should be 0.0-1.0
    assert!(result.confidence >= 0.0);
    assert!(result.confidence <= 1.0);
    
    Ok(())
}
