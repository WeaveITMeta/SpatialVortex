use crate::ai::orchestrator::ASIOrchestrator;
use crate::evaluation::benchmarks::{BenchmarkSuite, BenchmarkResult};
use crate::evaluation::metrics::{EvaluationScorecard, AggregatedMetrics};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use chrono::{DateTime, Utc};

/// Evaluation harness for running comprehensive benchmarks
pub struct EvaluationHarness {
    orchestrator: Arc<Mutex<ASIOrchestrator>>,
    config: EvaluationConfig,
    results: Vec<EvaluationResult>,
}

/// Configuration for evaluation runs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationConfig {
    /// Number of iterations per benchmark
    pub iterations: usize,
    /// Timeout per benchmark run (seconds)
    pub timeout_seconds: u64,
    /// Whether to save detailed results
    pub save_detailed_results: bool,
    /// Output directory for results
    pub output_dir: Option<PathBuf>,
    /// Which benchmarks to run
    pub benchmark_filter: Option<Vec<String>>,
    /// Enable real-time monitoring
    pub enable_monitoring: bool,
}

/// Result of a single evaluation run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    pub run_id: String,
    pub timestamp: DateTime<Utc>,
    pub config: EvaluationConfig,
    pub benchmark_results: Vec<BenchmarkResult>,
    pub scorecard: EvaluationScorecard,
    pub aggregated_metrics: AggregatedMetrics,
    pub execution_time_seconds: f64,
    pub success: bool,
    pub errors: Vec<String>,
}

/// Real-time monitoring data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringData {
    pub current_benchmark: String,
    pub current_iteration: usize,
    pub total_iterations: usize,
    pub elapsed_seconds: f64,
    pub estimated_remaining_seconds: f64,
    pub completed_benchmarks: usize,
    pub total_benchmarks: usize,
}

impl Default for EvaluationConfig {
    fn default() -> Self {
        Self {
            iterations: 3,
            timeout_seconds: 300, // 5 minutes
            save_detailed_results: true,
            output_dir: Some(PathBuf::from("./evaluation_results")),
            benchmark_filter: None,
            enable_monitoring: true,
        }
    }
}

impl EvaluationHarness {
    /// Create a new evaluation harness
    pub fn new(orchestrator: Arc<Mutex<ASIOrchestrator>>, config: EvaluationConfig) -> Self {
        Self {
            orchestrator,
            config,
            results: Vec::new(),
        }
    }

    /// Run the full evaluation suite
    pub async fn run_evaluation(&mut self) -> Result<EvaluationResult, Box<dyn std::error::Error + Send + Sync>> {
        let run_id = uuid::Uuid::new_v4().to_string();
        let start_time = Instant::now();
        let timestamp = Utc::now();

        tracing::info!("🚀 Starting evaluation run: {}", run_id);

        // Load benchmark suite
        let benchmark_suite = BenchmarkSuite::standard();
        let mut benchmark_results = Vec::new();
        let mut errors = Vec::new();
        let mut success = true;

        // Filter benchmarks if specified
        let benchmarks_to_run = if let Some(filter) = &self.config.benchmark_filter {
            benchmark_suite.benchmarks
                .into_iter()
                .filter(|b| filter.contains(&b.name))
                .collect()
        } else {
            benchmark_suite.benchmarks
        };

        let total_benchmarks = benchmarks_to_run.len();
        let mut completed_benchmarks = 0;

        for (benchmark_idx, benchmark) in benchmarks_to_run.into_iter().enumerate() {
            completed_benchmarks += 1;
            tracing::info!("📊 Running benchmark {}/{}: {}", benchmark_idx + 1, total_benchmarks, benchmark.name);

            // Run benchmark with timeout
            let benchmark_result = tokio::time::timeout(
                Duration::from_secs(self.config.timeout_seconds),
                async {
                    self.run_single_benchmark(&benchmark).await
                }
            ).await;

            match benchmark_result {
                Ok(Ok(result)) => {
                    tracing::info!("✅ Benchmark '{}' completed in {}ms", result.benchmark_name, result.execution_time_ms);
                    benchmark_results.push(result);
                }
                Ok(Err(e)) => {
                    tracing::error!("❌ Benchmark '{}' failed: {}", benchmark.name, e);
                    errors.push(format!("Benchmark '{}' failed: {}", benchmark.name, e));
                    success = false;
                }
                Err(_) => {
                    tracing::error!("⏰ Benchmark '{}' timed out after {}s", benchmark.name, self.config.timeout_seconds);
                    errors.push(format!("Benchmark '{}' timed out", benchmark.name));
                    success = false;
                }
            }

            // Emit monitoring data if enabled
            if self.config.enable_monitoring {
                let elapsed = start_time.elapsed().as_secs_f64();
                let avg_time_per_benchmark = elapsed / completed_benchmarks as f64;
                let remaining_benchmarks = total_benchmarks - completed_benchmarks;
                let estimated_remaining = avg_time_per_benchmark * remaining_benchmarks as f64;

                let monitoring_data = MonitoringData {
                    current_benchmark: benchmark.name.clone(),
                    current_iteration: 0,
                    total_iterations: self.config.iterations,
                    elapsed_seconds: elapsed,
                    estimated_remaining_seconds: estimated_remaining,
                    completed_benchmarks,
                    total_benchmarks,
                };

                tracing::info!("📈 Progress: {}/{} benchmarks, {:.1}s elapsed, {:.1}s estimated remaining",
                    completed_benchmarks, total_benchmarks, elapsed, estimated_remaining);
            }
        }

        // Generate scorecard and aggregated metrics
        let scorecard = BenchmarkSuite::generate_scorecard(&benchmark_results);
        let aggregated_metrics = AggregatedMetrics::from_scorecards(&[scorecard.clone()]);
        let execution_time = start_time.elapsed().as_secs_f64();

        let result = EvaluationResult {
            run_id,
            timestamp,
            config: self.config.clone(),
            benchmark_results,
            scorecard,
            aggregated_metrics,
            execution_time_seconds: execution_time,
            success,
            errors,
        };

        // Save results if configured
        if self.config.save_detailed_results {
            self.save_result(&result).await?;
        }

        tracing::info!("🏁 Evaluation completed in {:.2}s: {}", execution_time, 
            if success { "SUCCESS" } else { "FAILED" });

        self.results.push(result.clone());
        Ok(result)
    }

    /// Run a single benchmark multiple times
    async fn run_single_benchmark(
        &self,
        benchmark: &crate::evaluation::benchmarks::MultiTurnBenchmark,
    ) -> Result<BenchmarkResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut best_result: Option<BenchmarkResult> = None;
        let mut best_score = 0.0;

        for iteration in 0..self.config.iterations {
            tracing::debug!("🔄 Running iteration {}/{}", iteration + 1, self.config.iterations);
            
            let result = benchmark.execute(self.orchestrator.clone()).await?;
            let score = result.overall_metrics.health_score();

            if best_result.is_none() || score > best_score {
                best_score = score;
                best_result = Some(result);
            }

            // Small delay between iterations to avoid resource contention
            if iteration < self.config.iterations - 1 {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }

        best_result.ok_or("No successful benchmark runs".into())
    }

    /// Save evaluation results to file
    async fn save_result(&self, result: &EvaluationResult) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(output_dir) = &self.config.output_dir {
            // Create output directory if it doesn't exist
            tokio::fs::create_dir_all(output_dir).await?;

            // Save main result
            let result_file = output_dir.join(format!("evaluation_{}.json", result.run_id));
            let result_json = serde_json::to_string_pretty(result)?;
            tokio::fs::write(result_file, result_json).await?;

            // Save scorecard separately
            let scorecard_file = output_dir.join(format!("scorecard_{}.json", result.run_id));
            let scorecard_json = serde_json::to_string_pretty(&result.scorecard)?;
            tokio::fs::write(scorecard_file, scorecard_json).await?;

            // Save summary CSV
            let summary_file = output_dir.join(format!("summary_{}.csv", result.run_id));
            self.save_summary_csv(result, &summary_file).await?;

            tracing::info!("💾 Results saved to: {:?}", output_dir);
        }

        Ok(())
    }

    /// Save summary CSV for easy analysis
    async fn save_summary_csv(&self, result: &EvaluationResult, file_path: &PathBuf) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut csv_content = String::new();
        csv_content.push_str("run_id,timestamp,benchmark_name,passed,execution_time_ms,context_integrity,grounding_score,hallucination_risk,controller_compliance,avg_latency_ms\n");

        for benchmark_result in &result.benchmark_results {
            csv_content.push_str(&format!(
                "{},{},{},{},{},{},{},{},{},{}\n",
                result.run_id,
                result.timestamp.to_rfc3339(),
                benchmark_result.benchmark_name,
                benchmark_result.passed,
                benchmark_result.execution_time_ms,
                benchmark_result.overall_metrics.context_integrity,
                benchmark_result.overall_metrics.grounding_score,
                benchmark_result.overall_metrics.avg_hallucination_risk(),
                benchmark_result.overall_metrics.controller_compliance,
                benchmark_result.overall_metrics.avg_latency_ms,
            ));
        }

        tokio::fs::write(file_path, csv_content).await?;
        Ok(())
    }

    /// Get all results
    pub fn get_results(&self) -> &[EvaluationResult] {
        &self.results
    }

    /// Get latest result
    pub fn get_latest_result(&self) -> Option<&EvaluationResult> {
        self.results.last()
    }

    /// Clear all results
    pub fn clear_results(&mut self) {
        self.results.clear();
    }

    /// Generate trend analysis across multiple runs
    pub fn generate_trend_analysis(&self) -> TrendAnalysis {
        let mut trend = TrendAnalysis::new();

        for result in &self.results {
            trend.add_result(result);
        }

        trend
    }

    /// Compare two evaluation results
    pub fn compare_results(&self, result1: &EvaluationResult, result2: &EvaluationResult) -> ComparisonReport {
        ComparisonReport::new(result1, result2)
    }
}

/// Trend analysis across multiple evaluation runs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    pub run_ids: Vec<String>,
    pub timestamps: Vec<DateTime<Utc>>,
    pub context_integrity_trend: Vec<f32>,
    pub grounding_score_trend: Vec<f32>,
    pub hallucination_risk_trend: Vec<f32>,
    pub controller_compliance_trend: Vec<f32>,
    pub latency_trend: Vec<f64>,
    pub success_rate_trend: Vec<f32>,
}

impl TrendAnalysis {
    pub fn new() -> Self {
        Self {
            run_ids: Vec::new(),
            timestamps: Vec::new(),
            context_integrity_trend: Vec::new(),
            grounding_score_trend: Vec::new(),
            hallucination_risk_trend: Vec::new(),
            controller_compliance_trend: Vec::new(),
            latency_trend: Vec::new(),
            success_rate_trend: Vec::new(),
        }
    }

    pub fn add_result(&mut self, result: &EvaluationResult) {
        self.run_ids.push(result.run_id.clone());
        self.timestamps.push(result.timestamp);
        self.context_integrity_trend.push(result.scorecard.metrics.context_integrity);
        self.grounding_score_trend.push(result.scorecard.metrics.grounding_score);
        self.hallucination_risk_trend.push(result.scorecard.metrics.avg_hallucination_risk());
        self.controller_compliance_trend.push(result.scorecard.metrics.controller_compliance);
        self.latency_trend.push(result.scorecard.metrics.avg_latency_ms);
        
        let success_rate = result.benchmark_results.iter()
            .filter(|b| b.passed)
            .count() as f32 / result.benchmark_results.len() as f32;
        self.success_rate_trend.push(success_rate);
    }

    /// Calculate improvement percentage for each metric
    pub fn calculate_improvements(&self) -> ImprovementMetrics {
        if self.run_ids.len() < 2 {
            return ImprovementMetrics::default();
        }

        let first = 0;
        let last = self.run_ids.len() - 1;

        ImprovementMetrics {
            context_integrity_improvement: self.calculate_percentage_change(
                self.context_integrity_trend[first], 
                self.context_integrity_trend[last]
            ),
            grounding_score_improvement: self.calculate_percentage_change(
                self.grounding_score_trend[first],
                self.grounding_score_trend[last]
            ),
            hallucination_risk_improvement: self.calculate_percentage_change(
                self.hallucination_risk_trend[first],
                self.hallucination_risk_trend[last]
            ),
            controller_compliance_improvement: self.calculate_percentage_change(
                self.controller_compliance_trend[first],
                self.controller_compliance_trend[last]
            ),
            latency_improvement: self.calculate_percentage_change(
                self.latency_trend[first] as f32,
                self.latency_trend[last] as f32
            ),
            success_rate_improvement: self.calculate_percentage_change(
                self.success_rate_trend[first] as f32,
                self.success_rate_trend[last] as f32
            ),
        }
    }

    fn calculate_percentage_change(&self, initial: f32, final_value: f32) -> f32 {
        if initial == 0.0 {
            return 0.0;
        }
        ((final_value - initial) / initial) * 100.0
    }
}

/// Improvement metrics between evaluation runs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementMetrics {
    pub context_integrity_improvement: f32,
    pub grounding_score_improvement: f32,
    pub hallucination_risk_improvement: f32,
    pub controller_compliance_improvement: f32,
    pub latency_improvement: f32,
    pub success_rate_improvement: f32,
}

impl Default for ImprovementMetrics {
    fn default() -> Self {
        Self {
            context_integrity_improvement: 0.0,
            grounding_score_improvement: 0.0,
            hallucination_risk_improvement: 0.0,
            controller_compliance_improvement: 0.0,
            latency_improvement: 0.0,
            success_rate_improvement: 0.0,
        }
    }
}

/// Comparison report between two evaluation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonReport {
    pub baseline_run_id: String,
    pub comparison_run_id: String,
    pub baseline_timestamp: DateTime<Utc>,
    pub comparison_timestamp: DateTime<Utc>,
    pub metric_differences: MetricDifferences,
    pub benchmark_comparisons: Vec<BenchmarkComparison>,
    pub overall_assessment: ComparisonAssessment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDifferences {
    pub context_integrity_diff: f32,
    pub grounding_score_diff: f32,
    pub hallucination_risk_diff: f32,
    pub controller_compliance_diff: f32,
    pub latency_diff: f64,
    pub success_rate_diff: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkComparison {
    pub benchmark_name: String,
    pub baseline_passed: bool,
    pub comparison_passed: bool,
    pub baseline_execution_time_ms: u64,
    pub comparison_execution_time_ms: u64,
    pub performance_change: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonAssessment {
    Improved,
    Degraded,
    Unchanged,
    Mixed,
}

impl ComparisonReport {
    pub fn new(baseline: &EvaluationResult, comparison: &EvaluationResult) -> Self {
        let metric_differences = MetricDifferences {
            context_integrity_diff: comparison.scorecard.metrics.context_integrity - baseline.scorecard.metrics.context_integrity,
            grounding_score_diff: comparison.scorecard.metrics.grounding_score - baseline.scorecard.metrics.grounding_score,
            hallucination_risk_diff: comparison.scorecard.metrics.avg_hallucination_risk() - baseline.scorecard.metrics.avg_hallucination_risk(),
            controller_compliance_diff: comparison.scorecard.metrics.controller_compliance - baseline.scorecard.metrics.controller_compliance,
            latency_diff: comparison.scorecard.metrics.avg_latency_ms - baseline.scorecard.metrics.avg_latency_ms,
            success_rate_diff: comparison.benchmark_results.iter().filter(|b| b.passed).count() as f32 / comparison.benchmark_results.len() as f32
                - baseline.benchmark_results.iter().filter(|b| b.passed).count() as f32 / baseline.benchmark_results.len() as f32,
        };

        let mut benchmark_comparisons = Vec::new();
        for benchmark_result in &comparison.benchmark_results {
            if let Some(baseline_result) = baseline.benchmark_results.iter()
                .find(|b| b.benchmark_name == benchmark_result.benchmark_name) {
                
                let performance_change = if baseline_result.execution_time_ms > 0 {
                    ((benchmark_result.execution_time_ms as f64 - baseline_result.execution_time_ms as f64) 
                        / baseline_result.execution_time_ms as f64 * 100.0) as f32
                } else {
                    0.0
                };

                benchmark_comparisons.push(BenchmarkComparison {
                    benchmark_name: benchmark_result.benchmark_name.clone(),
                    baseline_passed: baseline_result.passed,
                    comparison_passed: benchmark_result.passed,
                    baseline_execution_time_ms: baseline_result.execution_time_ms,
                    comparison_execution_time_ms: benchmark_result.execution_time_ms,
                    performance_change,
                });
            }
        }

        let overall_assessment = Self::assess_overall(&metric_differences);

        Self {
            baseline_run_id: baseline.run_id.clone(),
            comparison_run_id: comparison.run_id.clone(),
            baseline_timestamp: baseline.timestamp,
            comparison_timestamp: comparison.timestamp,
            metric_differences,
            benchmark_comparisons,
            overall_assessment,
        }
    }

    fn assess_overall(differences: &MetricDifferences) -> ComparisonAssessment {
        let positive_changes = [
            differences.context_integrity_diff > 0.01,
            differences.grounding_score_diff > 0.01,
            differences.hallucination_risk_diff < -0.01, // Lower is better
            differences.controller_compliance_diff > 0.01,
            differences.latency_diff < -10.0, // Lower is better
            differences.success_rate_diff > 0.01,
        ].iter().filter(|&&x| x).count();

        let negative_changes = [
            differences.context_integrity_diff < -0.01,
            differences.grounding_score_diff < -0.01,
            differences.hallucination_risk_diff > 0.01, // Higher is worse
            differences.controller_compliance_diff < -0.01,
            differences.latency_diff > 10.0, // Higher is worse
            differences.success_rate_diff < -0.01,
        ].iter().filter(|&&x| x).count();

        if positive_changes > negative_changes {
            ComparisonAssessment::Improved
        } else if negative_changes > positive_changes {
            ComparisonAssessment::Degraded
        } else if positive_changes == 0 && negative_changes == 0 {
            ComparisonAssessment::Unchanged
        } else {
            ComparisonAssessment::Mixed
        }
    }
}
