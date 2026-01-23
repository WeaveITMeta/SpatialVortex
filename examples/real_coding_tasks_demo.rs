//! Real Coding Tasks Demo
//!
//! Tests the Enhanced Coding Agent with actual programming challenges:
//! 1. Algorithm implementation (binary search)
//! 2. Data structure (linked list with cycle detection)
//! 3. String manipulation (palindrome checker)
//! 4. Async programming (concurrent task executor)
//! 5. Error handling (robust file processor)

use spatial_vortex::{
    agents::coding_agent_enhanced::EnhancedCodingAgent,
    ml::training::two_stage_rl::TwoStageConfig,
};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("\n🎯 ═══════════════════════════════════════════════════════════");
    println!("   REAL CODING TASKS - ENHANCED AGENT TEST");
    println!("   Testing with production-level programming challenges");
    println!("🎯 ═══════════════════════════════════════════════════════════\n");

    let mut agent = EnhancedCodingAgent::new();
    
    // Enable training for continuous improvement
    agent.enable_training(TwoStageConfig::default()).await?;
    
    let tasks = vec![
        (
            "Task 1: Binary Search",
            "Implement binary search in Rust. Return Ok(index) if found, Err if not found. \
             Include tests for edge cases (empty array, single element, not found).",
            "Binary search with O(log n) complexity and comprehensive error handling"
        ),
        (
            "Task 2: Linked List Cycle Detection",
            "Implement Floyd's cycle detection algorithm (tortoise and hare) for a singly \
             linked list in Rust. Return true if cycle exists, false otherwise.",
            "Cycle detection using two-pointer technique with O(n) time, O(1) space"
        ),
        (
            "Task 3: Palindrome Validator",
            "Create a function that checks if a string is a palindrome, ignoring spaces, \
             punctuation, and case. Handle Unicode correctly.",
            "Robust palindrome checker with Unicode normalization"
        ),
        (
            "Task 4: Async Task Executor",
            "Implement a simple async task executor that can run multiple async tasks \
             concurrently with tokio. Include timeout handling and error recovery.",
            "Concurrent task executor with proper async/await and error handling"
        ),
        (
            "Task 5: Robust File Processor",
            "Create a function that reads a file line by line, processes each line with \
             a callback, and handles errors gracefully (file not found, permission denied, etc.).",
            "File processor with comprehensive error handling and resource cleanup"
        ),
    ];

    let mut results = Vec::new();
    
    for (i, (name, task, expected)) in tasks.iter().enumerate() {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📝 {} (Task {}/{})", name, i + 1, tasks.len());
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        
        println!("📋 Requirements:");
        println!("   {}\n", task);
        
        println!("🔍 Expected Outcome:");
        println!("   {}\n", expected);
        
        println!("🤔 Reasoning...\n");
        
        // Solve with reasoning chain
        let result = agent.execute_with_reasoning(task).await?;
        
        // Display reasoning chain
        println!("🧠 Reasoning Chain ({} steps):", result.reasoning_chain.steps.len());
        println!("   Overall Confidence: {:.1}%", result.confidence * 100.0);
        println!("   Vortex Cycle: {}", 
            if result.reasoning_chain.completed_vortex_cycle { "✅" } else { "⚠️" });
        
        // Show key reasoning steps
        let sacred_steps: Vec<_> = result.reasoning_chain.steps.iter()
            .filter(|s| s.is_sacred)
            .collect();
        
        println!("\n   Sacred Checkpoints ({}):", sacred_steps.len());
        for step in sacred_steps {
            println!("   🔷 Position {}: [Conf {:.0}%] [Signal {:.0}%]",
                step.flux_position,
                step.confidence * 100.0,
                step.confidence * 100.0
            );
            println!("      \"{}\"", &step.thought[..step.thought.len().min(60)]);
        }
        
        // Verification results
        println!("\n🔍 Verification:");
        println!("   Passed: {}", if result.verification.passed { "✅" } else { "⚠️" });
        println!("   Confidence: {:.1}%", result.verification.confidence * 100.0);
        println!("   Issues: {}", result.verification.issues.len());
        
        if !result.verification.issues.is_empty() {
            println!("\n   Issues Found:");
            for (i, issue) in result.verification.issues.iter().take(3).enumerate() {
                println!("   {}. {:?}", i + 1, issue);
            }
        }
        
        // Code quality
        println!("\n💻 Generated Code:");
        println!("   Language: {:?}", result.language);
        println!("   Lines: {}", result.code.lines().count());
        println!("   Has Tests: {}", result.code.contains("#[test]") || result.code.contains("assert"));
        println!("   Error Handling: {}", result.code.contains("Result") || result.code.contains("?"));
        
        // Show code snippet (first 15 lines)
        println!("\n   Code Preview:");
        for (i, line) in result.code.lines().take(15).enumerate() {
            println!("   {:3} | {}", i + 1, line);
        }
        if result.code.lines().count() > 15 {
            println!("   ... ({} more lines)", result.code.lines().count() - 15);
        }
        
        // Execution result (if available)
        if let Some(execution) = &result.execution {
            println!("\n🔧 Execution:");
            println!("   Success: {}", if execution.success { "✅" } else { "❌" });
            if !execution.stdout.is_empty() {
                println!("   Output: {}", execution.stdout.lines().next().unwrap_or(""));
            }
            if !execution.stderr.is_empty() {
                println!("   Errors: {}", execution.stderr);
            }
        }
        
        // Score this result
        let score = calculate_task_score(&result);
        println!("\n📊 Task Score: {:.1}/10.0", score);
        
        results.push((name.to_string(), score, result));
        
        println!("\n");
    }
    
    // Summary
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 OVERALL SUMMARY");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    let total_score: f32 = results.iter().map(|(_, score, _)| score).sum();
    let avg_score = total_score / results.len() as f32;
    
    let avg_confidence: f32 = results.iter()
        .map(|(_, _, r)| r.confidence)
        .sum::<f32>() / results.len() as f32;
    
    let vortex_complete = results.iter()
        .filter(|(_, _, r)| r.reasoning_chain.completed_vortex_cycle)
        .count();
    
    let verification_passed = results.iter()
        .filter(|(_, _, r)| r.verification.passed)
        .count();
    
    println!("Tasks Completed: {}/{}", results.len(), tasks.len());
    println!("Average Score: {:.1}/10.0", avg_score);
    println!("Average Confidence: {:.1}%", avg_confidence * 100.0);
    println!("Vortex Cycles Complete: {}/{} ({:.0}%)", 
        vortex_complete, results.len(), 
        (vortex_complete as f32 / results.len() as f32) * 100.0);
    println!("Verification Passed: {}/{} ({:.0}%)", 
        verification_passed, results.len(),
        (verification_passed as f32 / results.len() as f32) * 100.0);
    
    println!("\n📈 Individual Results:");
    for (name, score, result) in &results {
        let status = if result.verification.passed { "✅" } else { "⚠️" };
        let cycle = if result.reasoning_chain.completed_vortex_cycle { "🔄" } else { "  " };
        println!("   {} {} {:<40} Score: {:.1}/10.0  Conf: {:.0}%",
            status, cycle, name, score, result.confidence * 100.0);
    }
    
    // Quality assessment
    println!("\n🎯 Quality Assessment:");
    if avg_score >= 8.0 {
        println!("   ⭐ EXCELLENT - Production ready code generation!");
    } else if avg_score >= 7.0 {
        println!("   ✅ GOOD - High quality with minor improvements needed");
    } else if avg_score >= 6.0 {
        println!("   ⚠️ ACCEPTABLE - Functional but needs refinement");
    } else {
        println!("   ❌ NEEDS IMPROVEMENT - Further training required");
    }
    
    // Training statistics
    println!("\n🎓 Training Statistics:");
    let metrics = agent.get_learning_metrics().await;
    println!("   Total Iterations: {}", metrics.iterations);
    println!("   Successful: {}", metrics.tasks_completed);
    println!("   Failed: {}", metrics.tasks_failed);
    println!("   Success Rate: {:.1}%",
        (metrics.tasks_completed as f32 / metrics.iterations.max(1) as f32) * 100.0);
    
    println!("\n✨ Real coding tasks demo complete! ✨\n");
    
    Ok(())
}

fn calculate_task_score(result: &spatial_vortex::agents::coding_agent_enhanced::ReasoningTaskResult) -> f32 {
    let mut score = 0.0;
    
    // Base score for completion
    score += 2.0;
    
    // Confidence (max 2.5 points)
    score += result.confidence * 2.5;
    
    // Verification passed (1.5 points)
    if result.verification.passed {
        score += 1.5;
    } else {
        score += result.verification.confidence * 1.5;
    }
    
    // Vortex cycle complete (1.0 points)
    if result.reasoning_chain.completed_vortex_cycle {
        score += 1.0;
    } else {
        score += 0.5;
    }
    
    // Code quality indicators (max 2.0 points)
    let mut quality = 0.0;
    if result.code.contains("Result") || result.code.contains("Option") {
        quality += 0.5; // Error handling
    }
    if result.code.contains("#[test]") || result.code.contains("assert") {
        quality += 0.5; // Tests
    }
    if result.code.lines().count() >= 20 {
        quality += 0.5; // Substantial code
    }
    if result.code.contains("///") || result.code.contains("//!") {
        quality += 0.5; // Documentation
    }
    score += quality;
    
    // Execution success (1.0 point)
    if let Some(execution) = &result.execution {
        if execution.success {
            score += 1.0;
        } else {
            score += 0.3;
        }
    } else {
        score += 0.5; // Partial credit if no execution
    }
    
    score.min(10.0)
}
