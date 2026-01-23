/// SpatialVortex Benchmark Suite
/// 
/// Comprehensive benchmarks for evaluating SpatialVortex against state-of-the-art AI systems

pub mod knowledge_graph;
pub mod semantic;
pub mod qa;
pub mod reasoning;
pub mod compression;
pub mod custom;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Overall benchmark results
#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkSuite {
    pub metadata: BenchmarkMetadata,
    pub results: HashMap<String, BenchmarkCategory>,
    pub summary: BenchmarkSummary,
}

/// Metadata about the benchmark run
#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkMetadata {
    pub version: String,
    pub timestamp: String,
    pub system: String,
    pub cpu_cores: usize,
    pub total_memory_mb: u64,
}

/// Results for a category of benchmarks
#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkCategory {
    pub name: String,
    pub description: String,
    pub benchmarks: Vec<IndividualBenchmark>,
    pub category_score: f64,
}

/// Single benchmark result
#[derive(Debug, Serialize, Deserialize)]
pub struct IndividualBenchmark {
    pub name: String,
    pub metric: String,
    pub spatialvortex_score: f64,
    pub sota_score: f64,
    pub sota_model: String,
    pub improvement: f64,
    pub passed: bool,
}

/// Summary across all benchmarks
#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkSummary {
    pub total_benchmarks: usize,
    pub passed: usize,
    pub failed: usize,
    pub avg_improvement_vs_sota: f64,
    pub highlights: Vec<String>,
}

/// Run all benchmarks
pub async fn run_all_benchmarks() -> anyhow::Result<BenchmarkSuite> {
    println!("╔════════════════════════════════════════════════════════════════════╗");
    println!("║         SpatialVortex Comprehensive Benchmark Suite               ║");
    println!("╚════════════════════════════════════════════════════════════════════╝\n");
    
    let metadata = collect_metadata();
    println!("System: {} cores, {} MB RAM\n", metadata.cpu_cores, metadata.total_memory_mb);
    
    let mut results = HashMap::new();
    
    // Run each benchmark category
    println!("Running benchmarks...\n");
    
    // 1. Custom SpatialVortex benchmarks
    println!("【1/6】Custom SpatialVortex Benchmarks");
    results.insert("custom".to_string(), custom::run_custom_benchmarks().await?);
    
    // 2. Knowledge Graph
    println!("\n【2/6】Knowledge Graph Benchmarks");
    results.insert("knowledge_graph".to_string(), knowledge_graph::run_kg_benchmarks().await?);
    
    // 3. Semantic Similarity
    println!("\n【3/6】Semantic Similarity Benchmarks");
    results.insert("semantic".to_string(), semantic::run_semantic_benchmarks().await?);
    
    // 4. Question Answering
    println!("\n【4/6】Question Answering Benchmarks");
    results.insert("qa".to_string(), qa::run_qa_benchmarks().await?);
    
    // 5. Reasoning
    println!("\n【5/6】Reasoning Benchmarks");
    results.insert("reasoning".to_string(), reasoning::run_reasoning_benchmarks().await?);
    
    // 6. Compression
    println!("\n【6/6】Compression Benchmarks");
    results.insert("compression".to_string(), compression::run_compression_benchmarks().await?);
    
    let summary = calculate_summary(&results);
    
    Ok(BenchmarkSuite {
        metadata,
        results,
        summary,
    })
}

fn collect_metadata() -> BenchmarkMetadata {
    BenchmarkMetadata {
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        system: std::env::consts::OS.to_string(),
        cpu_cores: num_cpus::get(),
        total_memory_mb: 0, // TODO: Get from sysinfo
    }
}

fn calculate_summary(results: &HashMap<String, BenchmarkCategory>) -> BenchmarkSummary {
    let total_benchmarks: usize = results.values()
        .map(|cat| cat.benchmarks.len())
        .sum();
    
    let passed: usize = results.values()
        .flat_map(|cat| &cat.benchmarks)
        .filter(|b| b.passed)
        .count();
    
    let failed = total_benchmarks - passed;
    
    let avg_improvement: f64 = results.values()
        .flat_map(|cat| &cat.benchmarks)
        .map(|b| b.improvement)
        .sum::<f64>() / total_benchmarks as f64;
    
    let mut highlights = Vec::new();
    
    // Find top improvements
    let mut improvements: Vec<_> = results.values()
        .flat_map(|cat| &cat.benchmarks)
        .collect();
    improvements.sort_by(|a, b| b.improvement.partial_cmp(&a.improvement).unwrap());
    
    for bench in improvements.iter().take(3) {
        if bench.improvement > 0.0 {
            highlights.push(format!(
                "{}: +{:.1}% vs {}",
                bench.name, bench.improvement, bench.sota_model
            ));
        }
    }
    
    BenchmarkSummary {
        total_benchmarks,
        passed,
        failed,
        avg_improvement_vs_sota: avg_improvement,
        highlights,
    }
}

/// Print results
pub fn print_results(suite: &BenchmarkSuite) {
    println!("\n╔════════════════════════════════════════════════════════════════════╗");
    println!("║                     BENCHMARK RESULTS SUMMARY                      ║");
    println!("╚════════════════════════════════════════════════════════════════════╝\n");
    
    println!("Total Benchmarks: {}", suite.summary.total_benchmarks);
    println!("Passed: {} ✅", suite.summary.passed);
    println!("Failed: {} ❌", suite.summary.failed);
    println!("Average Improvement vs SOTA: {:+.1}%\n", suite.summary.avg_improvement_vs_sota);
    
    println!("🌟 Highlights:");
    for highlight in &suite.summary.highlights {
        println!("  • {}", highlight);
    }
    
    println!("\n📊 Per-Category Results:\n");
    println!("┌────────────────────────┬────────────┬──────────────────┐");
    println!("│ Category               │ Score      │ vs SOTA          │");
    println!("├────────────────────────┼────────────┼──────────────────┤");
    
    for (name, category) in &suite.results {
        let avg_improvement: f64 = category.benchmarks.iter()
            .map(|b| b.improvement)
            .sum::<f64>() / category.benchmarks.len() as f64;
        
        println!("│ {:<22} │ {:>10.2} │ {:>14.1}% │",
            name, category.category_score, avg_improvement);
    }
    
    println!("└────────────────────────┴────────────┴──────────────────┘");
}

/// Save results to JSON
pub fn save_results(suite: &BenchmarkSuite, path: &str) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(suite)?;
    std::fs::write(path, json)?;
    println!("\n📄 Results saved to: {}", path);
    Ok(())
}
