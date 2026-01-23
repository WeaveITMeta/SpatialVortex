//! Complex Scenarios Demo
//!
//! Tests the enhanced coding agent with challenging, real-world scenarios:
//! 1. Multi-threaded race condition debugging
//! 2. Database transaction with rollback
//! 3. WebSocket server with connection pooling
//! 4. Machine learning pipeline with validation
//! 5. Distributed cache with consistency guarantees
//! 6. API gateway with rate limiting and circuit breaker
//! 7. Event sourcing with CQRS pattern
//! 8. Cryptographic key rotation system
//! 9. Real-time data streaming with backpressure
//! 10. Microservice orchestration with saga pattern

use spatial_vortex::agents::coding_agent_enhanced::EnhancedCodingAgent;
use spatial_vortex::ml::training::two_stage_rl::TwoStageConfig;
use anyhow::Result;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    println!("\n🌟 ═══════════════════════════════════════════════════════════");
    println!("   COMPLEX SCENARIOS - ADVANCED TESTING");
    println!("   Real-world challenges requiring deep reasoning");
    println!("🌟 ═══════════════════════════════════════════════════════════\n");

    let mut agent = EnhancedCodingAgent::new();
    agent.enable_training(TwoStageConfig::default()).await?;
    
    let scenarios = vec![
        (
            "🧵 Multi-threaded Race Condition",
            "Debug and fix a race condition in a multi-threaded counter. The counter should \
             support concurrent increments from multiple threads without data races. Use atomic \
             operations or mutexes appropriately. Include a test that spawns 1000 threads.",
            vec!["concurrency", "atomics", "testing", "race conditions"]
        ),
        (
            "🗄️ Database Transaction Rollback",
            "Implement a database transaction wrapper that automatically rolls back on errors. \
             Support nested transactions, savepoints, and proper resource cleanup. Handle \
             connection pool exhaustion gracefully.",
            vec!["databases", "error handling", "resource management", "transactions"]
        ),
        (
            "🔌 WebSocket Server with Pooling",
            "Create a WebSocket server that maintains a connection pool, broadcasts messages \
             to subscribed clients, and handles disconnections gracefully. Implement heartbeat \
             mechanism and reconnection logic.",
            vec!["networking", "async", "protocols", "state management"]
        ),
        (
            "🤖 ML Pipeline with Validation",
            "Build a machine learning inference pipeline with input validation, model versioning, \
             A/B testing support, and result caching. Handle model loading failures and provide \
             fallback predictions.",
            vec!["ML", "validation", "caching", "versioning"]
        ),
        (
            "📦 Distributed Cache Consistency",
            "Implement a distributed cache client with eventual consistency guarantees. Support \
             cache invalidation across nodes, handle network partitions, and implement read-through \
             and write-through patterns.",
            vec!["distributed systems", "consistency", "networking", "caching"]
        ),
        (
            "🚪 API Gateway with Circuit Breaker",
            "Design an API gateway that implements rate limiting per client, circuit breaker \
             pattern for failing services, and request/response transformation. Include metrics \
             collection and health checks.",
            vec!["APIs", "resilience", "rate limiting", "monitoring"]
        ),
        (
            "📝 Event Sourcing with CQRS",
            "Implement an event sourcing system with CQRS pattern. Support event replay, \
             snapshot creation, and separate read/write models. Handle eventual consistency \
             and event versioning.",
            vec!["architecture", "event sourcing", "CQRS", "consistency"]
        ),
        (
            "🔐 Cryptographic Key Rotation",
            "Create a system for automatic cryptographic key rotation. Support zero-downtime \
             rotation, key versioning, and backward compatibility for decryption. Implement \
             secure key storage and access controls.",
            vec!["security", "cryptography", "key management", "zero-downtime"]
        ),
        (
            "🌊 Real-time Streaming with Backpressure",
            "Build a real-time data streaming system with backpressure handling. Support \
             buffering, flow control, and dynamic batching. Handle slow consumers without \
             blocking fast producers.",
            vec!["streaming", "backpressure", "async", "performance"]
        ),
        (
            "🎭 Microservice Saga Pattern",
            "Implement a saga orchestrator for distributed transactions across microservices. \
             Support compensation actions, timeout handling, and partial failure recovery. \
             Include idempotency and retry logic.",
            vec!["microservices", "distributed transactions", "saga", "resilience"]
        ),
    ];

    println!("📚 Test Suite: {} complex scenarios", scenarios.len());
    println!("   Each scenario tests multiple advanced concepts\n");
    
    let mut results = Vec::new();
    let start_time = Instant::now();
    
    for (i, (name, description, tags)) in scenarios.iter().enumerate() {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("🎯 Scenario {}/{}: {}", i + 1, scenarios.len(), name);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        
        println!("📋 Challenge:");
        println!("   {}\n", description);
        
        println!("🏷️  Tags: {}\n", tags.join(", "));
        
        let scenario_start = Instant::now();
        
        // Solve with reasoning
        println!("🧠 Reasoning...");
        let result = agent.execute_with_reasoning(description).await?;
        
        let scenario_time = scenario_start.elapsed();
        
        // Analyze reasoning chain
        println!("\n📊 Reasoning Analysis:");
        println!("   Steps: {}", result.reasoning_chain.steps.len());
        println!("   Confidence: {:.1}%", result.confidence * 100.0);
        println!("   Vortex Cycle: {}", 
            if result.reasoning_chain.completed_vortex_cycle { "✅" } else { "⚠️" });
        println!("   Time: {:.1}s", scenario_time.as_secs_f32());
        
        // Sacred checkpoints
        let sacred_steps: Vec<_> = result.reasoning_chain.steps.iter()
            .enumerate()
            .filter(|(_, s)| s.is_sacred)
            .collect();
        
        if !sacred_steps.is_empty() {
            println!("\n   Sacred Checkpoints:");
            for (idx, step) in sacred_steps {
                println!("   🔷 Step {} [Pos {}]: Conf {:.0}%",
                    idx + 1,
                    step.flux_position,
                    step.confidence * 100.0
                );
            }
        }
        
        // Verification
        println!("\n🔍 Verification:");
        println!("   Status: {}", if result.verification.passed { "✅ PASSED" } else { "⚠️ ISSUES" });
        println!("   Confidence: {:.1}%", result.verification.confidence * 100.0);
        println!("   Issues: {}", result.verification.issues.len());
        
        if result.verification.issues.len() > 0 {
            println!("\n   Top Issues:");
            for (i, issue) in result.verification.issues.iter().take(2).enumerate() {
                println!("   {}. {:?}", i + 1, issue);
            }
        }
        
        // Code analysis
        println!("\n💻 Code Analysis:");
        let lines = result.code.lines().count();
        let has_tests = result.code.contains("#[test]") || result.code.contains("assert");
        let has_async = result.code.contains("async") || result.code.contains(".await");
        let has_errors = result.code.contains("Result<") || result.code.contains("?");
        let has_docs = result.code.contains("///") || result.code.contains("//!");
        
        println!("   Lines of Code: {}", lines);
        println!("   Tests: {}", if has_tests { "✅" } else { "❌" });
        println!("   Async/Await: {}", if has_async { "✅" } else { "○" });
        println!("   Error Handling: {}", if has_errors { "✅" } else { "❌" });
        println!("   Documentation: {}", if has_docs { "✅" } else { "○" });
        
        // Complexity score
        let complexity = calculate_complexity(&result.code, tags);
        println!("   Complexity Score: {:.1}/10.0", complexity);
        
        // Overall assessment
        let assessment = assess_solution(&result, complexity, scenario_time);
        println!("\n🎯 Assessment: {:.1}/10.0", assessment);
        
        if assessment >= 8.5 {
            println!("   ⭐ EXCELLENT - Production-ready solution");
        } else if assessment >= 7.0 {
            println!("   ✅ GOOD - Solid implementation");
        } else if assessment >= 5.5 {
            println!("   ⚠️  ACCEPTABLE - Needs refinement");
        } else {
            println!("   ❌ INSUFFICIENT - Requires significant work");
        }
        
        results.push((name.to_string(), assessment, result));
        println!();
    }
    
    let total_time = start_time.elapsed();
    
    // Final summary
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📈 FINAL SUMMARY");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    let avg_score: f32 = results.iter().map(|(_, s, _)| s).sum::<f32>() / results.len() as f32;
    let avg_confidence: f32 = results.iter()
        .map(|(_, _, r)| r.confidence)
        .sum::<f32>() / results.len() as f32;
    let vortex_complete = results.iter()
        .filter(|(_, _, r)| r.reasoning_chain.completed_vortex_cycle)
        .count();
    let verification_passed = results.iter()
        .filter(|(_, _, r)| r.verification.passed)
        .count();
    
    println!("⏱️  Performance:");
    println!("   Total Time: {:.1}s", total_time.as_secs_f32());
    println!("   Avg per Scenario: {:.1}s", total_time.as_secs_f32() / results.len() as f32);
    
    println!("\n📊 Quality Metrics:");
    println!("   Average Score: {:.1}/10.0", avg_score);
    println!("   Average Confidence: {:.1}%", avg_confidence * 100.0);
    println!("   Vortex Cycles: {}/{} ({:.0}%)", 
        vortex_complete, results.len(),
        (vortex_complete as f32 / results.len() as f32) * 100.0);
    println!("   Verification Passed: {}/{} ({:.0}%)",
        verification_passed, results.len(),
        (verification_passed as f32 / results.len() as f32) * 100.0);
    
    println!("\n🏆 Top Performers:");
    let mut sorted_results = results.clone();
    sorted_results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    
    for (i, (name, score, _)) in sorted_results.iter().take(5).enumerate() {
        let medal = match i {
            0 => "🥇",
            1 => "🥈",
            2 => "🥉",
            _ => "  ",
        };
        println!("   {} {:<45} {:.1}/10.0", medal, name, score);
    }
    
    println!("\n🎯 System Capability Assessment:");
    
    if avg_score >= 8.0 {
        println!("   ⭐ EXPERT LEVEL");
        println!("   • Handles complex, production-level scenarios");
        println!("   • Demonstrates deep reasoning and understanding");
        println!("   • Ready for challenging real-world applications");
    } else if avg_score >= 7.0 {
        println!("   ✅ ADVANCED LEVEL");
        println!("   • Competent with complex scenarios");
        println!("   • Good reasoning and code quality");
        println!("   • Suitable for most production use cases");
    } else if avg_score >= 6.0 {
        println!("   ⚠️  INTERMEDIATE LEVEL");
        println!("   • Handles basic complexity");
        println!("   • Needs improvement for production");
        println!("   • Continue training recommended");
    } else {
        println!("   📚 LEARNING LEVEL");
        println!("   • Still developing capabilities");
        println!("   • Requires significant training");
        println!("   • Not ready for complex production scenarios");
    }
    
    // Training stats
    println!("\n🎓 Training Impact:");
    let metrics = agent.get_learning_metrics().await;
    println!("   Total Iterations: {}", metrics.iterations);
    println!("   Success Rate: {:.1}%",
        (metrics.tasks_completed as f32 / metrics.iterations.max(1) as f32) * 100.0);
    
    println!("\n✨ Complex scenarios testing complete! ✨\n");
    
    Ok(())
}

fn calculate_complexity(code: &str, tags: &[&str]) -> f32 {
    let mut score: f32 = 5.0;
    
    // Base complexity from code features
    if code.contains("async") { score += 0.5; }
    if code.contains("Arc<") || code.contains("Mutex<") { score += 0.8; }
    if code.contains("tokio::") { score += 0.6; }
    if code.contains("match") { score += 0.3; }
    if code.lines().count() > 50 { score += 0.5; }
    if code.lines().count() > 100 { score += 0.8; }
    
    // Complexity from scenario tags
    for tag in tags {
        match *tag {
            "distributed systems" | "microservices" => score += 1.0,
            "concurrency" | "async" => score += 0.8,
            "cryptography" | "security" => score += 0.7,
            "ML" | "streaming" => score += 0.6,
            _ => score += 0.2,
        }
    }
    
    score.min(10.0)
}

fn assess_solution(
    result: &spatial_vortex::agents::coding_agent_enhanced::ReasoningTaskResult,
    complexity: f32,
    time: std::time::Duration,
) -> f32 {
    let mut score = 0.0;
    
    // Confidence (30%)
    score += result.confidence * 3.0;
    
    // Verification (20%)
    if result.verification.passed {
        score += 2.0;
    } else {
        score += result.verification.confidence * 2.0;
    }
    
    // Vortex cycle (10%)
    if result.reasoning_chain.completed_vortex_cycle {
        score += 1.0;
    } else {
        score += 0.4;
    }
    
    // Code quality (25%)
    let mut quality = 0.0;
    if result.code.contains("Result<") { quality += 0.6; }
    if result.code.contains("#[test]") { quality += 0.5; }
    if result.code.contains("///") { quality += 0.4; }
    if result.code.contains("async") { quality += 0.5; }
    score += quality;
    
    // Complexity handling (10%)
    score += (complexity / 10.0) * 1.0;
    
    // Time bonus/penalty (5%)
    if time.as_secs() < 10 {
        score += 0.5;
    } else if time.as_secs() > 30 {
        score -= 0.3;
    }
    
    score.clamp(0.0, 10.0)
}
