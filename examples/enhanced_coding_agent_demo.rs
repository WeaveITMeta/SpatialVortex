//! Enhanced Coding Agent Demo
//!
//! Demonstrates the full capabilities of the enhanced coding agent with:
//! - Explicit chain-of-thought reasoning
//! - Self-verification with VCP
//! - Two-stage RL training
//! - Sacred geometry routing
//!
//! Run with: cargo run --example enhanced_coding_agent_demo

use spatial_vortex::agents::EnhancedCodingAgent;
use spatial_vortex::ml::training::TwoStageConfig;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("\n🤖 Enhanced Coding Agent Demo\n");
    println!("========================================\n");
    
    // Demo 1: Basic reasoning-enabled task
    demo_basic_reasoning_task().await?;
    
    // Demo 2: Complex algorithm task
    demo_complex_algorithm().await?;
    
    // Demo 3: Safety verification
    demo_safety_verification().await?;
    
    // Demo 4: Training with RL
    demo_rl_training().await?;
    
    // Demo 5: Benchmarking
    demo_benchmarking().await?;
    
    println!("\n✅ All demos completed!\n");
    Ok(())
}

async fn demo_basic_reasoning_task() -> Result<()> {
    println!("═══ Demo 1: Basic Reasoning Task ═══\n");
    
    let agent = EnhancedCodingAgent::new();
    
    let task = "Write a Rust function to calculate the factorial of a number";
    
    println!("Task: {}\n", task);
    println!("Executing with reasoning...\n");
    
    let result = agent.execute_with_reasoning(task).await?;
    
    // Show reasoning trace
    println!("{}", result.reasoning_chain.to_trace());
    
    println!("Generated Code:");
    println!("```rust\n{}\n```\n", result.code);
    
    println!("Verification:");
    println!("  Passed: {}", if result.verification.passed { "✓" } else { "✗" });
    println!("  Confidence: {:.1}%", result.verification.confidence * 100.0);
    println!("  Confidence: {:.1}%", result.verification.confidence * 100.0);
    
    if !result.verification.issues.is_empty() {
        println!("  Issues: {}", result.verification.issues.len());
    }
    
    println!();
    Ok(())
}

async fn demo_complex_algorithm() -> Result<()> {
    println!("\n═══ Demo 2: Complex Algorithm Task ═══\n");
    
    let agent = EnhancedCodingAgent::new();
    
    let task = "Implement a binary search tree with insert, search, and delete operations in Rust";
    
    println!("Task: {}\n", task);
    
    let result = agent.execute_with_reasoning(task).await?;
    
    println!("Reasoning Summary:");
    println!("  Total Steps: {}", result.reasoning_chain.steps.len());
    println!("  Sacred Checkpoints: {}", 
        result.reasoning_chain.steps.iter().filter(|s| s.is_sacred).count()
    );
    println!("  Overall Confidence: {:.1}%", result.reasoning_chain.overall_confidence * 100.0);
    println!("  Vortex Cycle Complete: {}", 
        if result.reasoning_chain.completed_vortex_cycle { "✓" } else { "✗" }
    );
    
    // Show key reasoning steps
    println!("\nKey Reasoning Steps:");
    for (i, step) in result.reasoning_chain.steps.iter().enumerate() {
        if step.is_sacred {
            println!("  {} Step {}: [Pos {}] 🔷 {}", 
                if step.verification_passed { "✓" } else { "✗" },
                i + 1,
                step.flux_position,
                &step.thought[..step.thought.len().min(60)]
            );
        }
    }
    
    println!();
    Ok(())
}

async fn demo_safety_verification() -> Result<()> {
    println!("\n═══ Demo 3: Safety Verification ═══\n");
    
    let agent = EnhancedCodingAgent::new();
    
    // Task with potential safety concerns
    let task = "Write a Rust function to delete all files in a directory";
    
    println!("Task: {}\n", task);
    println!("⚠️  This task has safety implications\n");
    
    let result = agent.execute_with_reasoning(task).await?;
    
    // Find the safety check step (Position 3)
    let safety_step = result.reasoning_chain.steps.iter()
        .find(|s| s.flux_position == 3);
    
    if let Some(step) = safety_step {
        println!("Sacred Position 3 (Ethics/Safety Check):");
        println!("  Thought: {}", step.thought);
        println!("  Confidence: {:.1}%", step.confidence * 100.0);
        println!("  ELP State: E={:.1} L={:.1} P={:.1}", 
            step.elp_state.ethos, 
            step.elp_state.logos, 
            step.elp_state.pathos
        );
        println!("  Ethos-dominant: {}", step.elp_state.ethos > 8.0);
    }
    
    println!("\nSafety verification ensures ethical considerations at Position 3!");
    println!();
    Ok(())
}

async fn demo_rl_training() -> Result<()> {
    println!("\n═══ Demo 4: Two-Stage RL Training ═══\n");
    
    let mut agent = EnhancedCodingAgent::new();
    
    // Enable training
    let config = TwoStageConfig {
        discovery_epsilon: 0.25,
        alignment_epsilon: 0.03,
        max_steps: 8,
        discovery_min_confidence: 0.5,
        alignment_min_confidence: 0.75,
        warmstart_traces: 100,
    };
    
    println!("Enabling RL training...");
    agent.enable_training(config).await?;
    println!("✓ Training enabled with warmstart\n");
    
    // Train on a small set of tasks
    let training_tasks = vec![
        "Write a function to reverse a string",
        "Implement bubble sort",
        "Create a function to check if a number is prime",
        "Write a function to find the greatest common divisor",
        "Implement a stack data structure",
    ];
    
    println!("Training on {} tasks...\n", training_tasks.len());
    
    let stats = agent.train(training_tasks).await?;
    
    println!("Training Stats:");
    println!("  Total Iterations: {}", stats.iterations);
    println!("  Stage: {:?}", stats.stage);
    println!("  Discovery Avg Reward: {:.3}", stats.discovery_avg_reward);
    println!("  Alignment Avg Reward: {:.3}", stats.alignment_avg_reward);
    println!("  Discovery Buffer: {} experiences", stats.discovery_buffer_size);
    println!("  Alignment Buffer: {} experiences", stats.alignment_buffer_size);
    
    println!("\n✨ Agent learns coding patterns through RL!\n");
    Ok(())
}

async fn demo_benchmarking() -> Result<()> {
    println!("\n═══ Demo 5: Performance Benchmarking ═══\n");
    
    let agent = EnhancedCodingAgent::new();
    
    let test_tasks = vec![
        "Write a function to add two numbers",
        "Implement string concatenation",
        "Create a function to find max in array",
    ];
    
    println!("Running benchmark on {} tasks...\n", test_tasks.len());
    
    let benchmark = agent.benchmark(test_tasks).await;
    
    println!("Benchmark Results:");
    println!("  Total Tasks: {}", benchmark.total_tasks);
    println!("  Successes: {}", benchmark.successes);
    println!("  Success Rate: {:.1}%", benchmark.success_rate * 100.0);
    println!("  Avg Confidence: {:.1}%", benchmark.avg_confidence * 100.0);
    println!("  Avg Reasoning Steps: {:.1}", benchmark.avg_reasoning_steps);
    
    // Get learning metrics
    let metrics = agent.get_learning_metrics().await;
    
    println!("\nLearning Metrics:");
    println!("  Total Iterations: {}", metrics.iterations);
    println!("  Success Rate: {:.1}%", metrics.success_rate * 100.0);
    println!("  Avg Confidence: {:.1}%", metrics.avg_confidence * 100.0);
    println!("  Tasks Completed: {}", metrics.tasks_completed);
    println!("  Tasks Failed: {}", metrics.tasks_failed);
    
    // Show last decision explanation
    println!("\nLast Decision Explanation:");
    let explanation = agent.explain_last_decision().await;
    println!("{}", explanation);
    
    println!();
    Ok(())
}
