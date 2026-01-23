//! RSI Loop Demo - First Real Self-Improvement Cycle
//!
//! Demonstrates the complete RSI (Recursive Self-Improvement) loop:
//! 1. ASI runs inferences and collects metrics
//! 2. PerformanceTracker detects weaknesses
//! 3. SelfModificationEngine proposes improvements
//! 4. Proposals are tested and applied automatically (if safe)
//! 5. System improves itself recursively

use spatial_vortex::ai::orchestrator::{ASIOrchestrator, ExecutionMode, RSIConfig};
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
    
    println!("=== RSI Loop Demo: First Real Self-Improvement Cycle ===\n");
    
    // Step 1: Create ASI Orchestrator with self-modification capability
    println!("📦 Step 1: Initializing ASI Orchestrator with Self-Modification Engine...");
    let source_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    
    let orchestrator = ASIOrchestrator::new()
        .await?
        .with_self_modification(source_path);
    
    println!("✓ Orchestrator initialized");
    println!("✓ Self-modification engine: {}", orchestrator.has_self_modification());
    println!();
    
    // Step 2: Configure RSI
    println!("⚙️  Step 2: Configuring RSI Parameters...");
    let mut rsi_config = RSIConfig {
        enabled: true,
        min_confidence_threshold: 0.75,  // Trigger if confidence < 75%
        max_thorough_time_ms: 800.0,     // Trigger if thorough mode > 800ms
        max_error_rate: 0.05,             // Trigger if error rate > 5%
        max_weaknesses_per_cycle: 3,     // Address top 3 weaknesses
        auto_apply_low_risk: true,       // Auto-apply low-risk improvements
        auto_apply_medium_risk: false,   // Require approval for medium-risk
        min_inferences_before_rsi: 10,   // Need 10+ inferences before RSI
        cycle_interval_secs: 60,          // Run RSI every 60 seconds
    };
    
    orchestrator.enable_rsi(rsi_config.clone()).await;
    
    println!("✓ RSI enabled: {}", rsi_config.enabled);
    println!("✓ Confidence threshold: {:.2}", rsi_config.min_confidence_threshold);
    println!("✓ Max thorough time: {:.0}ms", rsi_config.max_thorough_time_ms);
    println!("✓ Auto-apply low risk: {}", rsi_config.auto_apply_low_risk);
    println!();
    
    // Step 3: Run inferences to generate metrics
    println!("🧠 Step 3: Running Inferences to Generate Performance Metrics...");
    
    let test_inputs = vec![
        "What is the meaning of life?",
        "Explain quantum entanglement",
        "How does consciousness emerge?",
        "What is the nature of time?",
        "Describe the heat death of the universe",
        "What is the relationship between mind and matter?",
        "How do neural networks learn?",
        "What is the halting problem?",
        "Explain Gödel's incompleteness theorems",
        "What is the hard problem of consciousness?",
        "How does evolution create complexity?",
        "What is the arrow of time?",
        "Describe the measurement problem in quantum mechanics",
        "What is emergence in complex systems?",
        "How does language shape thought?",
    ];
    
    for (i, input) in test_inputs.iter().enumerate() {
        print!("  [{}/{}] Processing: {}... ", i + 1, test_inputs.len(), 
            &input[..input.len().min(40)]);
        
        let mode = match i % 3 {
            0 => ExecutionMode::Fast,
            1 => ExecutionMode::Balanced,
            _ => ExecutionMode::Thorough,
        };
        
        let result = orchestrator.process(input, mode).await?;
        println!("✓ (confidence: {:.2}, time: {}ms)", 
            result.confidence, result.processing_time_ms);
        
        // Small delay between inferences
        sleep(Duration::from_millis(100)).await;
    }
    
    println!();
    
    // Step 4: Check current metrics
    println!("📊 Step 4: Analyzing Performance Metrics...");
    let metrics = orchestrator.get_metrics();
    println!("  Total inferences: {}", metrics.total_inferences);
    println!("  Average confidence: {:.2}", metrics.avg_confidence);
    println!("  Fast mode avg: {:.0}ms", metrics.fast_mode_avg_time);
    println!("  Balanced mode avg: {:.0}ms", metrics.balanced_mode_avg_time);
    println!("  Thorough mode avg: {:.0}ms", metrics.thorough_mode_avg_time);
    println!();
    
    // Step 5: Run RSI Cycle
    println!("🔄 Step 5: Running RSI Cycle (Auto-Trigger Self-Improvement)...");
    println!("  Detecting weaknesses from metrics...");
    
    let rsi_result = orchestrator.rsi_cycle().await?;
    
    println!();
    println!("=== RSI Cycle Results ===");
    println!("Weaknesses Detected: {}", rsi_result.weaknesses_detected.len());
    for (i, weakness) in rsi_result.weaknesses_detected.iter().enumerate() {
        println!("  {}. {} (severity: {:.2})", i + 1, weakness.description, weakness.severity);
        println!("     Type: {}, Value: {:.2}", weakness.weakness_type, weakness.metric_value);
    }
    println!();
    
    println!("Proposals Generated: {}", rsi_result.proposals_generated);
    println!("Proposals Tested: {}", rsi_result.proposals_tested);
    println!("Proposals Applied: {}", rsi_result.proposals_applied);
    println!("Proposals Rejected: {}", rsi_result.proposals_rejected);
    println!("Proposals Pending Approval: {}", rsi_result.proposals_pending_approval);
    println!();
    
    if !rsi_result.improvements.is_empty() {
        println!("✅ Improvements Applied:");
        for improvement in &rsi_result.improvements {
            println!("  • {}", improvement);
        }
        println!();
    }
    
    if !rsi_result.errors.is_empty() {
        println!("⚠️  Errors:");
        for error in &rsi_result.errors {
            println!("  • {}", error);
        }
        println!();
    }
    
    // Step 6: Show all proposals
    println!("📋 Step 6: All Improvement Proposals...");
    let proposals = orchestrator.get_proposals().await;
    
    if proposals.is_empty() {
        println!("  No proposals generated (system performing within thresholds)");
    } else {
        for (i, proposal) in proposals.iter().enumerate() {
            println!("  {}. {} (Risk: {:?})", i + 1, proposal.description, proposal.risk_level);
            println!("     Addresses: {}", proposal.weakness_addressed);
            println!("     Expected improvement: {:.1}%", proposal.expected_improvement * 100.0);
            println!("     Status: {:?}", proposal.status);
            println!("     Patches: {}", proposal.patches.len());
            println!();
        }
    }
    
    // Step 7: Show RSI statistics
    println!("📈 Step 7: Self-Modification Statistics...");
    if let Some(stats) = orchestrator.rsi_stats().await {
        println!("  Proposals generated: {}", stats.proposals_generated);
        println!("  Proposals tested: {}", stats.proposals_tested);
        println!("  Proposals applied: {}", stats.proposals_applied);
        println!("  Proposals rejected: {}", stats.proposals_rejected);
        println!("  Rollbacks: {}", stats.rollbacks);
        println!("  Total improvement: {:.1}%", stats.total_improvement * 100.0);
    } else {
        println!("  No statistics available");
    }
    println!();
    
    // Step 8: Demonstrate manual approval (if any pending)
    if rsi_result.proposals_pending_approval > 0 {
        println!("🔐 Step 8: Manual Approval Demo...");
        let pending_proposals: Vec<_> = proposals.iter()
            .filter(|p| matches!(p.status, spatial_vortex::asi::self_modification::ProposalStatus::Proposed))
            .collect();
        
        if let Some(proposal) = pending_proposals.first() {
            println!("  Pending proposal: {}", proposal.description);
            println!("  Risk level: {:?}", proposal.risk_level);
            println!("  Would require manual approval via:");
            println!("    orchestrator.approve_proposal({}).await", proposal.id);
        }
        println!();
    }
    
    // Summary
    println!("=== RSI Loop Summary ===");
    println!("✓ Metrics collected from {} inferences", metrics.total_inferences);
    println!("✓ {} weaknesses detected", rsi_result.weaknesses_detected.len());
    println!("✓ {} improvement proposals generated", rsi_result.proposals_generated);
    println!("✓ {} proposals auto-applied", rsi_result.proposals_applied);
    println!("✓ {} proposals pending manual review", rsi_result.proposals_pending_approval);
    println!();
    
    println!("🎯 RSI Loop Status: OPERATIONAL");
    println!("   The system can now autonomously:");
    println!("   1. Monitor its own performance");
    println!("   2. Detect weaknesses and bottlenecks");
    println!("   3. Propose code improvements");
    println!("   4. Test improvements in sandbox");
    println!("   5. Apply safe improvements automatically");
    println!("   6. Request approval for risky changes");
    println!("   7. Rollback if needed");
    println!();
    
    println!("🚀 First Real Self-Improvement Cycle: COMPLETE");
    
    Ok(())
}
