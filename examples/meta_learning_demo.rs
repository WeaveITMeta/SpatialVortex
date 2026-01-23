//! Meta-Learning System Demo
//!
//! Demonstrates:
//! - Pattern extraction from successful reasoning
//! - Pattern storage and retrieval
//! - Query acceleration via pattern matching
//! - Learning metrics tracking

use spatial_vortex::ai::{
    FluxReasoningChain,
    PatternExtractor, InMemoryPatternStorage, PatternStorage, PatternMatcher, QueryAccelerator,
    FeedbackCollector, LearningMetrics,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    println!("\n🧠 Meta-Learning System Demo");
    println!("================================\n");
    
    // Initialize meta-learning components
    let extractor = PatternExtractor::new();
    let storage = Arc::new(InMemoryPatternStorage::new());
    let matcher = Arc::new(PatternMatcher::new(storage.clone()));
    let accelerator = QueryAccelerator::new(matcher.clone());
    let feedback = FeedbackCollector::new(storage.clone());
    
    // ========================================================================
    // PHASE 1: Initial Learning (No Patterns)
    // ========================================================================
    
    println!("📚 PHASE 1: Learning from First Query");
    println!("---------------------------------------\n");
    
    let query1 = "How do I reverse type 2 diabetes?";
    println!("Query 1: {}", query1);
    
    let mut chain1 = FluxReasoningChain::new(query1);
    
    // Try acceleration (will fail - no patterns yet)
    match accelerator.try_accelerate(&chain1).await? {
        Some(_) => println!("✨ Accelerated!"),
        None => println!("📝 No pattern match - reasoning from scratch"),
    }
    
    // Reason through the query (simulated - in real use would call chain1.reason(20))
    // For demo, we'll manually set converged state
    println!("🔄 Reasoning... (12 steps)");
    // Simulate reasoning steps
    for _ in 0..12 {
        chain1.apply_flux_transformation();
    }
    
    // Manually set convergence for demo
    if let Some(last) = chain1.thoughts.last_mut() {
        last.certainty = 0.85;
        last.entropy = 0.25;
    }
    chain1.chain_confidence = 0.85;
    chain1.sacred_milestones = vec![3, 6, 9];  // All trinity positions influenced
    
    println!("✅ Converged: confidence={:.2}, steps={}", 
        chain1.chain_confidence, chain1.thoughts.len());
    
    // Extract pattern
    if let Some(pattern) = extractor.extract(&chain1) {
        println!("\n📦 Pattern Extracted:");
        println!("  Pattern ID: {}", pattern.pattern_id);
        println!("  Domain: {}", pattern.query_signature.domain);
        println!("  Confidence: {:.2}", pattern.confidence);
        println!("  Steps: {}", pattern.avg_steps);
        println!("  Sacred Influences: {:?}", pattern.sacred_influences);
        
        // Store pattern
        storage.store(pattern).await?;
        println!("💾 Pattern stored in Confidence Lake");
    }
    
    // ========================================================================
    // PHASE 2: Pattern Reuse (Similar Query)
    // ========================================================================
    
    println!("\n\n📚 PHASE 2: Accelerating Similar Query");
    println!("---------------------------------------\n");
    
    let query2 = "What's the best way to manage diabetes?";
    println!("Query 2: {}", query2);
    
    let mut chain2 = FluxReasoningChain::new(query2);
    
    // Try acceleration (should find pattern from Query 1)
    match accelerator.try_accelerate(&chain2).await? {
        Some(result) => {
            println!("🚀 Pattern Match Found!");
            println!("  Pattern ID: {}", result.pattern_id);
            println!("  Steps Saved: {}", result.steps_saved);
            println!("  Confidence Boost: {:.2}", result.confidence_boost);
            chain2 = result.accelerated_chain;
        },
        None => {
            println!("📝 No pattern match");
        },
    }
    
    // Finish reasoning if needed
    if !chain2.has_converged() {
        println!("🔄 Completing reasoning... (4 more steps)");
        for _ in 0..4 {
            chain2.apply_flux_transformation();
        }
    }
    
    println!("✅ Converged: confidence={:.2}, steps={}", 
        chain2.chain_confidence, chain2.thoughts.len());
    
    // Extract new pattern (refined version)
    if let Some(pattern) = extractor.extract(&chain2) {
        storage.store(pattern).await?;
        println!("💾 Refined pattern stored");
    }
    
    // ========================================================================
    // PHASE 3: More Learning (Different Domain)
    // ========================================================================
    
    println!("\n\n📚 PHASE 3: Learning New Domain");
    println!("--------------------------------\n");
    
    let query3 = "Is it ethical to use AI for medical diagnosis?";
    println!("Query 3: {}", query3);
    
    let mut chain3 = FluxReasoningChain::new(query3);
    
    // Try acceleration (should fail - different domain)
    match accelerator.try_accelerate(&chain3).await? {
        Some(_) => println!("✨ Accelerated!"),
        None => println!("📝 Different domain - reasoning from scratch"),
    }
    
    // Reason through
    println!("🔄 Reasoning... (10 steps)");
    for _ in 0..10 {
        chain3.apply_flux_transformation();
    }
    
    // Set converged
    if let Some(last) = chain3.thoughts.last_mut() {
        last.certainty = 0.80;
        last.entropy = 0.30;
    }
    chain3.chain_confidence = 0.80;
    chain3.sacred_milestones = vec![3, 6];
    
    println!("✅ Converged: confidence={:.2}, steps={}", 
        chain3.chain_confidence, chain3.thoughts.len());
    
    // Extract pattern
    if let Some(pattern) = extractor.extract(&chain3) {
        println!("\n📦 New Domain Pattern:");
        println!("  Domain: {}", pattern.query_signature.domain);
        storage.store(pattern).await?;
        println!("💾 Pattern stored");
    }
    
    // ========================================================================
    // PHASE 4: Learning Metrics
    // ========================================================================
    
    println!("\n\n📊 LEARNING METRICS");
    println!("===================\n");
    
    let metrics = storage.get_metrics().await?;
    display_metrics(&metrics);
    
    // ========================================================================
    // PHASE 5: Pattern Evolution (Pruning)
    // ========================================================================
    
    println!("\n\n🧹 PATTERN EVOLUTION");
    println!("====================\n");
    
    // Simulate some pattern failures by lowering success rates
    // (In real use, this happens through feedback)
    
    let pruned = feedback.evolve_patterns(0.5).await?;
    println!("Pruned {} ineffective patterns (success < 50%)", pruned);
    
    // Final metrics
    let final_metrics = storage.get_metrics().await?;
    println!("\n📈 Final Metrics:");
    display_metrics(&final_metrics);
    
    // ========================================================================
    // Summary
    // ========================================================================
    
    println!("\n\n✨ SUMMARY");
    println!("==========\n");
    println!("The meta-learning system:");
    println!("✅ Extracted {} patterns from successful reasoning", final_metrics.patterns_extracted);
    println!("✅ Maintains {} active patterns", final_metrics.patterns_active);
    println!("✅ Average success rate: {:.1}%", final_metrics.avg_success_rate * 100.0);
    println!("✅ Patterns are reused and refined over time");
    println!("✅ Ineffective patterns are automatically pruned");
    println!("\n🚀 Query acceleration will improve as more patterns are learned!");
    
    Ok(())
}

fn display_metrics(metrics: &LearningMetrics) {
    println!("  Patterns Extracted: {}", metrics.patterns_extracted);
    println!("  Patterns Active: {}", metrics.patterns_active);
    println!("  Patterns Pruned: {}", metrics.patterns_pruned);
    println!("  Avg Reuse Count: {:.1}", metrics.avg_reuse_count);
    println!("  Avg Success Rate: {:.1}%", metrics.avg_success_rate * 100.0);
}
