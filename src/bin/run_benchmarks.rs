//! Benchmark Runner Binary
//!
//! Runs the context preservation benchmarks and generates a report.
//!
//! Usage:
//!   cargo run --bin run_benchmarks --release

use spatial_vortex::benchmarks::{BenchmarkRunner, BenchmarkConfig};

fn main() {
    println!();
    println!("Starting SpatialVortex Context Preservation Benchmarks...");
    println!();
    
    // Configure benchmarks
    let config = BenchmarkConfig {
        max_steps: vec![5, 10, 20, 50, 100, 200],
        trials_per_config: 20,
        ..Default::default()
    };
    
    let runner = BenchmarkRunner::new(config);
    
    // Run benchmarks
    println!("Running {} configurations with {} trials each...", 
        runner.config.max_steps.len(),
        runner.config.trials_per_config);
    println!();
    
    let results = runner.run_benchmarks();
    
    // Generate and print report
    let report = runner.generate_report(&results);
    println!("{}", report);
    
    // Save results to JSON
    let json_path = "benchmark_results.json";
    if let Ok(json) = serde_json::to_string_pretty(&results) {
        if std::fs::write(json_path, &json).is_ok() {
            println!("Results saved to: {}", json_path);
        }
    }
    
    // Print key metrics
    if let Some(comparison) = &results.comparison {
        println!();
        println!("═══════════════════════════════════════════════════════════════════");
        println!("  KEY RESULT: {:.1}% improvement over linear baseline",
            comparison.improvement_percentage);
        println!("  Statistical significance: p = {:.4} ({})",
            comparison.p_value,
            if comparison.is_significant { "SIGNIFICANT" } else { "not significant" });
        println!("═══════════════════════════════════════════════════════════════════");
        println!();
    }
}
