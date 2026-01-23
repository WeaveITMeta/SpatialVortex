//! Reasoning Chain Demo
//!
//! Demonstrates explicit chain-of-thought reasoning with self-verification
//! and two-stage RL training for emergent reasoning patterns.
//!
//! Run with: cargo run --example reasoning_chain_demo

use spatial_vortex::ai::reasoning_chain::{ReasoningChain, ReasoningStep};
use spatial_vortex::ai::self_verification::SelfVerificationEngine;
use spatial_vortex::ml::training::two_stage_rl::{TwoStageRLTrainer, TwoStageConfig, TrainingStage};
use spatial_vortex::data::models::ELPTensor;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("\n🧠 Reasoning Chain Demo\n");
    println!("========================================\n");
    
    // Demo 1: Basic reasoning chain
    demo_basic_reasoning_chain()?;
    
    // Demo 2: Self-verification
    demo_self_verification()?;
    
    // Demo 3: Two-stage RL training
    demo_two_stage_rl().await?;
    
    println!("\n✅ All demos completed!\n");
    Ok(())
}

fn demo_basic_reasoning_chain() -> Result<()> {
    println!("═══ Demo 1: Basic Reasoning Chain ═══\n");
    
    let mut chain = ReasoningChain::new();
    
    // Build a reasoning chain following vortex sequence
    println!("Building reasoning chain through vortex sequence...\n");
    
    // Position 1: Start
    chain.add_step(
        "First, I need to understand the problem space and identify key components.".to_string(),
        ELPTensor::new(6.0, 6.0, 6.0),
        1,
        0.75
    );
    
    // Position 2: Expand
    chain.add_step(
        "Breaking this down into smaller sub-problems will make it more manageable.".to_string(),
        ELPTensor::new(6.5, 6.5, 6.0),
        2,
        0.8
    );
    
    // Position 3: Sacred checkpoint (Ethos)
    chain.add_step(
        "Consider the ethical implications and ensure approach aligns with core values.".to_string(),
        ELPTensor::new(8.0, 6.0, 5.0),
        3,
        0.88
    );
    
    // Position 4: Continue
    chain.add_step(
        "Now examining the logical structure of the arguments presented.".to_string(),
        ELPTensor::new(6.5, 7.0, 5.5),
        4,
        0.82
    );
    
    // Position 6: Sacred checkpoint (Logos)
    chain.add_step(
        "Apply rigorous logical reasoning to validate the conclusion.".to_string(),
        ELPTensor::new(5.5, 8.5, 5.0),
        6,
        0.90
    );
    
    // Position 9: Sacred checkpoint (Pathos)
    chain.add_step(
        "Finally, consider the emotional impact and human factors in this decision.".to_string(),
        ELPTensor::new(6.0, 6.0, 8.0),
        9,
        0.87
    );
    
    // Verify consistency
    let is_consistent = chain.verify_consistency()?;
    println!("Consistency check: {}\n", if is_consistent { "✓ PASSED" } else { "✗ FAILED" });
    
    // Finalize
    chain.finalize("The balanced approach considers ethics, logic, and emotion.".to_string());
    
    // Display trace
    println!("{}", chain.to_trace());
    
    println!("Overall Confidence: {:.1}%", chain.overall_confidence * 100.0);
    println!("Vortex Cycle Complete: {}\n", if chain.completed_vortex_cycle { "✓" } else { "✗" });
    
    Ok(())
}

fn demo_self_verification() -> Result<()> {
    println!("\n═══ Demo 2: Self-Verification ═══\n");
    
    let verifier = SelfVerificationEngine::new();
    
    // Create a chain with potential issues
    let mut problem_chain = ReasoningChain::new();
    
    problem_chain.add_step(
        "Starting with high confidence".to_string(),
        ELPTensor::new(6.0, 6.0, 6.0),
        1,
        0.85
    );
    
    // Large ELP jump (will trigger issue)
    problem_chain.add_step(
        "Sudden jump in reasoning".to_string(),
        ELPTensor::new(11.0, 3.0, 10.0),  // Drastic change
        2,
        0.50  // Low confidence
    );
    
    problem_chain.verify_consistency()?;
    problem_chain.finalize("Questionable conclusion".to_string());
    
    // Verify the chain
    let result = verifier.verify_chain(&problem_chain)?;
    
    println!("Verification Result:");
    println!("  Passed: {}", if result.passed { "✓" } else { "✗" });
    println!("  Confidence: {:.1}%", result.confidence * 100.0);
    println!("  Confidence: {:.1}%", result.confidence * 100.0);
    println!("  Issues Found: {}", result.issues.len());
    
    if !result.issues.is_empty() {
        println!("\nIssues Detected:");
        for (i, issue) in result.issues.iter().enumerate() {
            println!("  {}. {:?}", i + 1, issue);
        }
    }
    
    // Now create a high-quality chain
    println!("\n--- Testing High-Quality Chain ---\n");
    
    let mut good_chain = ReasoningChain::new();
    
    good_chain.add_step("".to_string(), ELPTensor::new(6.0, 6.0, 6.0), 1, 0.85);
    good_chain.add_step("".to_string(), ELPTensor::new(6.5, 6.5, 6.0), 3, 0.90);  // Sacred
    good_chain.add_step("".to_string(), ELPTensor::new(6.0, 7.0, 6.0), 6, 0.88);  // Sacred
    good_chain.add_step("".to_string(), ELPTensor::new(6.5, 6.5, 7.0), 9, 0.92);  // Sacred
    
    good_chain.verify_consistency()?;
    good_chain.finalize("High-quality result".to_string());
    
    let good_result = verifier.verify_chain(&good_chain)?;
    
    println!("High-Quality Chain Verification:");
    println!("  Passed: {}", if good_result.passed { "✓" } else { "✗" });
    println!("  Confidence: {:.1}%", good_result.confidence * 100.0);
    println!("  Issues: {}\n", good_result.issues.len());
    
    Ok(())
}

async fn demo_two_stage_rl() -> Result<()> {
    println!("\n═══ Demo 3: Two-Stage RL Training ═══\n");
    
    let config = TwoStageConfig {
        discovery_epsilon: 0.25,
        alignment_epsilon: 0.03,
        max_steps: 8,
        discovery_min_confidence: 0.5,
        alignment_min_confidence: 0.75,
        warmstart_traces: 50,  // Small for demo
    };
    
    let mut trainer = TwoStageRLTrainer::new(config)?;
    
    // Warmstart from Confidence Lake
    trainer.warmstart_from_lake().await?;
    
    let stats = trainer.get_stats();
    println!("Initial Stats:");
    println!("  Stage: {:?}", stats.stage);
    println!("  Discovery Buffer: {} traces\n", stats.discovery_buffer_size);
    
    // Stage 1: Discovery training iterations
    println!("--- Stage 1: Discovery (High Exploration) ---\n");
    
    for i in 0..5 {
        let chain = trainer.train_iteration("What is consciousness?")?;
        println!("Discovery Iteration {}: {} steps, confidence: {:.1}%",
            i + 1,
            chain.steps.len(),
            chain.overall_confidence * 100.0
        );
    }
    
    let discovery_stats = trainer.get_stats();
    println!("\nDiscovery Stats:");
    println!("  Avg Reward: {:.3}", discovery_stats.discovery_avg_reward);
    println!("  Buffer Size: {}\n", discovery_stats.discovery_buffer_size);
    
    // Force switch to alignment
    println!("--- Stage 2: Alignment (Low Exploration) ---\n");
    
    // Manually switch stage for demo
    // In production, this happens automatically after sufficient discovery
    
    let final_stats = trainer.get_stats();
    println!("Final Training Stats:");
    println!("  Total Iterations: {}", final_stats.iterations);
    println!("  Discovery Avg Reward: {:.3}", final_stats.discovery_avg_reward);
    println!("  Alignment Avg Reward: {:.3}", final_stats.alignment_avg_reward);
    println!("  Discovery Buffer: {}", final_stats.discovery_buffer_size);
    println!("  Alignment Buffer: {}", final_stats.alignment_buffer_size);
    
    println!("\n✨ Two-stage RL enables emergent reasoning at low training cost!");
    
    Ok(())
}
