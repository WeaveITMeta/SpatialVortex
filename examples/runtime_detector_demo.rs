//! Runtime Weakness Detector Demo
//!
//! Demonstrates autonomous runtime monitoring with auto-trigger RSI:
//! 1. Detector monitors metrics continuously
//! 2. Detects latency spikes, confidence drops, prediction errors
//! 3. Auto-triggers RSI proposals when thresholds exceeded
//! 4. Applies safe improvements automatically

use spatial_vortex::ai::orchestrator::{ASIOrchestrator, ExecutionMode, RSIConfig};
use spatial_vortex::asi::runtime_detector::RuntimeDetectorConfig;
use spatial_vortex::error::Result;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    println!("=== Runtime Weakness Detector Demo ===\n");
    println!("Autonomous monitoring with auto-trigger RSI\n");
    
    // Step 1: Create orchestrator with runtime detector
    println!("📦 Step 1: Initializing ASI with Runtime Detector...");
    let source_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    
    let mut detector_config = RuntimeDetectorConfig::default();
    detector_config.enabled = true;
    detector_config.monitor_interval_ms = 500; // Check every 500ms
    detector_config.window_size = 20; // 10-second window
    detector_config.confidence_low_threshold = 0.7; // Trigger below 70%
    detector_config.latency_spike_threshold_ms = 300.0; // 300ms spike
    detector_config.auto_trigger_rsi = true;
    detector_config.trigger_cooldown_secs = 10; // 10 second cooldown for demo
    
    let orchestrator = ASIOrchestrator::new()
        .await?
        .with_self_modification(source_path)
        .with_runtime_detector(detector_config.clone());
    
    println!("✓ Orchestrator initialized");
    println!("✓ Runtime detector: {}", orchestrator.has_runtime_detector());
    println!("✓ Self-modification: {}", orchestrator.has_self_modification());
    println!();
    
    // Step 2: Configure RSI
    println!("⚙️  Step 2: Configuring RSI...");
    let mut rsi_config = RSIConfig::default();
    rsi_config.enabled = true;
    rsi_config.auto_apply_low_risk = true;
    rsi_config.auto_apply_medium_risk = false;
    orchestrator.enable_rsi(rsi_config).await;
    
    println!("✓ RSI enabled with auto-apply for low-risk improvements");
    println!();
    
    // Step 3: Start autonomous monitoring
    println!("🔄 Step 3: Starting Autonomous Runtime Monitoring...");
    let _monitor_handle = orchestrator.start_runtime_monitoring().await?;
    
    println!("✓ Background monitoring started");
    println!("✓ Monitor interval: {}ms", detector_config.monitor_interval_ms);
    println!("✓ Window size: {} samples", detector_config.window_size);
    println!("✓ Auto-trigger: {}", detector_config.auto_trigger_rsi);
    println!();
    
    // Step 4: Simulate workload with varying performance
    println!("🧠 Step 4: Running Workload (30 seconds)...");
    println!("   Simulating normal operation, then degradation, then recovery\n");
    
    let test_inputs = vec![
        "What is consciousness?",
        "Explain quantum mechanics",
        "How does the brain work?",
        "What is artificial intelligence?",
        "Describe machine learning",
    ];
    
    // Phase 1: Normal operation (10 seconds)
    println!("Phase 1: Normal Operation (0-10s)");
    for i in 0..10 {
        let input = &test_inputs[i % test_inputs.len()];
        let result = orchestrator.process(input, ExecutionMode::Fast).await?;
        
        print!("  [{}s] Processed: confidence={:.2}, latency={}ms", 
            i + 1, result.confidence, result.processing_time_ms);
        
        if let Some(stats) = orchestrator.runtime_stats() {
            print!(" | avg_conf={:.2}, samples={}", 
                stats.avg_confidence, stats.samples_collected);
        }
        println!();
        
        sleep(Duration::from_secs(1)).await;
    }
    println!();
    
    // Phase 2: Performance degradation (10 seconds)
    println!("Phase 2: Performance Degradation (10-20s)");
    println!("   Simulating high latency and low confidence...");
    for i in 0..10 {
        let input = &test_inputs[i % test_inputs.len()];
        
        // Simulate degraded performance by using slower mode
        let result = orchestrator.process(input, ExecutionMode::Thorough).await?;
        
        // Manually record degraded metrics to trigger detector
        if let Some(ref detector) = orchestrator.runtime_detector {
            detector.record_sample(
                800.0 + (i as f32 * 50.0), // Increasing latency
                0.55 - (i as f32 * 0.02),  // Decreasing confidence
                0.15 + (i as f32 * 0.02),  // Increasing error
            );
        }
        
        print!("  [{}s] Degraded: confidence={:.2}, latency={}ms", 
            i + 11, result.confidence, result.processing_time_ms);
        
        if let Some(stats) = orchestrator.runtime_stats() {
            print!(" | avg_conf={:.2}, triggers={}", 
                stats.avg_confidence, stats.rsi_triggers);
        }
        println!();
        
        sleep(Duration::from_secs(1)).await;
    }
    println!();
    
    // Phase 3: Recovery (10 seconds)
    println!("Phase 3: Recovery (20-30s)");
    println!("   Returning to normal operation...");
    for i in 0..10 {
        let input = &test_inputs[i % test_inputs.len()];
        let result = orchestrator.process(input, ExecutionMode::Balanced).await?;
        
        print!("  [{}s] Recovered: confidence={:.2}, latency={}ms", 
            i + 21, result.confidence, result.processing_time_ms);
        
        if let Some(stats) = orchestrator.runtime_stats() {
            print!(" | avg_conf={:.2}, weaknesses={}", 
                stats.avg_confidence, stats.weaknesses_detected);
        }
        println!();
        
        sleep(Duration::from_secs(1)).await;
    }
    println!();
    
    // Step 5: Show runtime statistics
    println!("📊 Step 5: Runtime Monitoring Statistics");
    if let Some(stats) = orchestrator.runtime_stats() {
        println!("  Samples collected: {}", stats.samples_collected);
        println!("  Weaknesses detected: {}", stats.weaknesses_detected);
        println!("  RSI triggers: {}", stats.rsi_triggers);
        println!("  Average latency: {:.0}ms", stats.avg_latency_ms);
        println!("  Average confidence: {:.2}", stats.avg_confidence);
        println!("  Average prediction error: {:.3}", stats.avg_prediction_error);
        
        if let Some(last_trigger) = stats.last_trigger_time {
            println!("  Last trigger: {} seconds ago", 
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() - last_trigger);
        }
    }
    println!();
    
    // Step 6: Show detected weaknesses
    println!("⚠️  Step 6: Detected Weaknesses (Last 10)");
    let weaknesses = orchestrator.get_runtime_weaknesses(10);
    
    if weaknesses.is_empty() {
        println!("  No weaknesses detected (system performing within thresholds)");
    } else {
        for (i, weakness) in weaknesses.iter().enumerate() {
            println!("  {}. {:?}", i + 1, weakness.weakness_type);
            println!("     {}", weakness.description);
            println!("     Severity: {:.2}, Value: {:.2}, Baseline: {:.2}", 
                weakness.severity, weakness.metric_value, weakness.baseline_value);
            println!();
        }
    }
    
    // Step 7: Show RSI proposals
    println!("📋 Step 7: Auto-Generated Proposals");
    let proposals = orchestrator.get_proposals().await;
    
    if proposals.is_empty() {
        println!("  No proposals generated");
    } else {
        for (i, proposal) in proposals.iter().enumerate() {
            println!("  {}. {} (Risk: {:?})", i + 1, proposal.description, proposal.risk_level);
            println!("     Addresses: {}", proposal.weakness_addressed);
            println!("     Status: {:?}", proposal.status);
            println!();
        }
    }
    
    // Step 8: Show RSI statistics
    println!("📈 Step 8: Self-Modification Statistics");
    if let Some(stats) = orchestrator.rsi_stats().await {
        println!("  Proposals generated: {}", stats.proposals_generated);
        println!("  Proposals tested: {}", stats.proposals_tested);
        println!("  Proposals applied: {}", stats.proposals_applied);
        println!("  Proposals rejected: {}", stats.proposals_rejected);
        println!("  Total improvement: {:.1}%", stats.total_improvement * 100.0);
    }
    println!();
    
    // Stop monitoring
    orchestrator.stop_runtime_monitoring();
    println!("🛑 Runtime monitoring stopped");
    println!();
    
    // Summary
    println!("=== Demo Summary ===");
    println!("✓ Runtime detector monitored {} samples", 
        orchestrator.runtime_stats().map(|s| s.samples_collected).unwrap_or(0));
    println!("✓ Detected {} weaknesses", 
        orchestrator.runtime_stats().map(|s| s.weaknesses_detected).unwrap_or(0));
    println!("✓ Triggered RSI {} times", 
        orchestrator.runtime_stats().map(|s| s.rsi_triggers).unwrap_or(0));
    println!("✓ Generated {} improvement proposals", proposals.len());
    println!();
    
    println!("🎯 Autonomous Runtime Monitoring: OPERATIONAL");
    println!("   The system continuously:");
    println!("   1. Monitors latency, confidence, and prediction errors");
    println!("   2. Detects performance degradation in real-time");
    println!("   3. Auto-triggers RSI when thresholds exceeded");
    println!("   4. Proposes and applies improvements autonomously");
    println!("   5. Maintains cooldown to prevent trigger spam");
    println!();
    
    println!("🚀 Full Autonomy Achieved!");
    
    Ok(())
}
