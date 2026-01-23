/// SpatialVortex Benchmark Runner
/// 
/// Executes all benchmarks and generates comprehensive report

use spatialvortex_benchmarks::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Run all benchmarks
    let results = run_all_benchmarks().await?;
    
    // Print results
    print_results(&results);
    
    // Save to JSON
    let output_path = "benchmark_results.json";
    save_results(&results, output_path)?;
    
    println!("\n✅ Benchmark suite completed!");
    println!("   Total: {} benchmarks", results.summary.total_benchmarks);
    println!("   Passed: {} ✅", results.summary.passed);
    println!("   Failed: {} ❌", results.summary.failed);
    println!("   Avg vs SOTA: {:+.1}%", results.summary.avg_improvement_vs_sota);
    
    Ok(())
}
