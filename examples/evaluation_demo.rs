use spatial_vortex::ai::orchestrator::ASIOrchestrator;
use spatial_vortex::evaluation::{EvaluationHarness, EvaluationConfig};
use spatial_vortex::ml::inference::production_engine::ProductionEngine;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 Starting ASI Orchestrator Evaluation Demo");

    // Create ASI Orchestrator
    let orchestrator = Arc::new(Mutex::new(
        ASIOrchestrator::new().await?
    ));

    // Set up evaluation configuration
    let config = EvaluationConfig {
        iterations: 2,
        timeout_seconds: 180, // 3 minutes
        save_detailed_results: true,
        output_dir: Some(PathBuf::from("./evaluation_results")),
        benchmark_filter: None,
        enable_monitoring: true,
    };

    println!("📊 Configuration:");
    println!("  - Iterations per benchmark: {}", config.iterations);
    println!("  - Timeout: {}s", config.timeout_seconds);
    println!("  - Output directory: {:?}", config.output_dir);

    // Create evaluation harness
    let mut harness = EvaluationHarness::new(orchestrator, config);

    // Run evaluation
    println!("\n🔬 Running evaluation suite...");
    let result = harness.run_evaluation().await?;

    // Display results
    println!("\n📈 Evaluation Results:");
    println!("  Run ID: {}", result.run_id);
    println!("  Success: {}", result.success);
    println!("  Execution time: {:.2}s", result.execution_time_seconds);
    println!("  Benchmarks completed: {}/{}", 
        result.benchmark_results.len(), 
        result.benchmark_results.iter().filter(|b| b.passed).count()
    );

    println!("\n📊 Overall Metrics:");
    println!("  Context Integrity: {:.3}", result.scorecard.metrics.context_integrity);
    println!("  Grounding Score: {:.3}", result.scorecard.metrics.grounding_score);
    println!("  Hallucination Risk: {:.3}", result.scorecard.metrics.avg_hallucination_risk());
    println!("  Controller Compliance: {:.3}", result.scorecard.metrics.controller_compliance);
    println!("  Average Latency: {:.1}ms", result.scorecard.metrics.avg_latency_ms);

    println!("\n🎯 KPI Summary:");
    println!("  Overall Health: {:.3}", result.scorecard.kpi_summary.overall_health);
    println!("  Risk Level: {:?}", result.scorecard.kpi_summary.risk_level);
    println!("  Performance Grade: {:?}", result.scorecard.kpi_summary.performance_grade);

    if !result.scorecard.kpi_summary.strengths.is_empty() {
        println!("\n💪 Strengths:");
        for strength in &result.scorecard.kpi_summary.strengths {
            println!("  - {}", strength);
        }
    }

    if !result.scorecard.kpi_summary.improvements.is_empty() {
        println!("\n🔧 Areas for Improvement:");
        for improvement in &result.scorecard.kpi_summary.improvements {
            println!("  - {}", improvement);
        }
    }

    if !result.scorecard.recommendations.is_empty() {
        println!("\n💡 Recommendations:");
        for recommendation in &result.scorecard.recommendations {
            println!("  - {}", recommendation);
        }
    }

    // Display benchmark details
    println!("\n📋 Benchmark Details:");
    for benchmark_result in &result.benchmark_results {
        println!("  {} - {} ({}ms)", 
            benchmark_result.benchmark_name,
            if benchmark_result.passed { "✅ PASSED" } else { "❌ FAILED" },
            benchmark_result.execution_time_ms
        );
        
        if !benchmark_result.errors.is_empty() {
            for error in &benchmark_result.errors {
                println!("    Error: {}", error);
            }
        }

        // Show turn-level details
        for turn_result in &benchmark_result.turn_results {
            if !turn_result.issues.is_empty() {
                println!("    Turn {}: {} issues", turn_result.turn_index, turn_result.issues.len());
                for issue in &turn_result.issues {
                    println!("      - {}", issue);
                }
            }
        }
    }

    // Generate trend analysis if we have multiple runs
    if harness.get_results().len() > 1 {
        println!("\n📈 Trend Analysis:");
        let trend = harness.generate_trend_analysis();
        let improvements = trend.calculate_improvements();

        println!("  Context Integrity: {:+.1}%", improvements.context_integrity_improvement);
        println!("  Grounding Score: {:+.1}%", improvements.grounding_score_improvement);
        println!("  Hallucination Risk: {:+.1}%", improvements.hallucination_risk_improvement);
        println!("  Controller Compliance: {:+.1}%", improvements.controller_compliance_improvement);
        println!("  Latency: {:+.1}%", improvements.latency_improvement);
        println!("  Success Rate: {:+.1}%", improvements.success_rate_improvement);
    }

    // Compare with previous run if available
    let results = harness.get_results();
    if results.len() >= 2 {
        let latest = &results[results.len() - 1];
        let previous = &results[results.len() - 2];

        println!("\n🔄 Comparison with Previous Run:");
        let comparison = harness.compare_results(previous, latest);
        
        println!("  Overall Assessment: {:?}", comparison.overall_assessment);
        println!("  Context Integrity: {:+.3}", comparison.metric_differences.context_integrity_diff);
        println!("  Grounding Score: {:+.3}", comparison.metric_differences.grounding_score_diff);
        println!("  Hallucination Risk: {:+.3}", comparison.metric_differences.hallucination_risk_diff);
        println!("  Controller Compliance: {:+.3}", comparison.metric_differences.controller_compliance_diff);
        println!("  Latency: {:+.1}ms", comparison.metric_differences.latency_diff);
        println!("  Success Rate: {:+.1}%", comparison.metric_differences.success_rate_diff * 100.0);
    }

    if !result.success {
        println!("\n❌ Evaluation completed with errors:");
        for error in &result.errors {
            println!("  - {}", error);
        }
    } else {
        println!("\n✅ Evaluation completed successfully!");
    }

    Ok(())
}
