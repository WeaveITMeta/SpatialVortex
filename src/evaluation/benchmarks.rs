use crate::ai::orchestrator::{ASIOrchestrator, SessionId};
use crate::evaluation::metrics::{EvaluationMetrics, EvaluationScorecard, SessionMetrics};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

/// Multi-turn conversation benchmark
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiTurnBenchmark {
    pub name: String,
    pub description: String,
    pub turns: Vec<BenchmarkTurn>,
    pub expected_behaviors: Vec<ExpectedBehavior>,
}

/// Single turn in a benchmark
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkTurn {
    pub user_input: String,
    pub expected_context_integrity: f32,
    pub expected_grounding_score: f32,
    pub max_hallucination_risk: f32,
    pub expected_controller_intervention: bool,
}

/// Expected behavior patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedBehavior {
    pub behavior_type: BehaviorType,
    pub description: String,
    pub trigger_turns: Vec<usize>,
}

/// Types of expected behaviors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BehaviorType {
    ContextCompression,
    CheckpointIntervention,
    HallucinationMitigation,
    RAGGrounding,
    ConsensusBuilding,
}

/// Grounding benchmark for RAG evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundingBenchmark {
    pub name: String,
    pub queries: Vec<GroundingQuery>,
    pub knowledge_base: Vec<String>,
}

/// Query with expected grounding results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundingQuery {
    pub query: String,
    pub expected_retrieval_count: usize,
    pub expected_min_similarity: f32,
    pub expected_factual_accuracy: f32,
}

/// Benchmark execution results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub benchmark_name: String,
    pub session_id: String,
    pub execution_time_ms: u64,
    pub turn_results: Vec<TurnResult>,
    pub overall_metrics: EvaluationMetrics,
    pub passed: bool,
    pub errors: Vec<String>,
}

/// Results for a single turn
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnResult {
    pub turn_index: usize,
    pub user_input: String,
    pub generated_response: String,
    pub actual_context_integrity: f32,
    pub actual_grounding_score: f32,
    pub actual_hallucination_risk: f32,
    pub controller_intervention: bool,
    pub latency_ms: u64,
    pub checkpoint_hit: bool,
    pub passed: bool,
    pub issues: Vec<String>,
}

impl MultiTurnBenchmark {
    /// Create a basic multi-turn integrity benchmark
    pub fn basic_integrity() -> Self {
        Self {
            name: "Basic Multi-Turn Integrity".to_string(),
            description: "Tests context preservation and integrity across multiple turns".to_string(),
            turns: vec![
                BenchmarkTurn {
                    user_input: "I'm planning a trip to Japan. Can you help me with an itinerary?".to_string(),
                    expected_context_integrity: 0.9,
                    expected_grounding_score: 0.7,
                    max_hallucination_risk: 0.2,
                    expected_controller_intervention: false,
                },
                BenchmarkTurn {
                    user_input: "Great! What about the cherry blossom season?".to_string(),
                    expected_context_integrity: 0.85,
                    expected_grounding_score: 0.75,
                    max_hallucination_risk: 0.25,
                    expected_controller_intervention: false,
                },
                BenchmarkTurn {
                    user_input: "Actually, I changed my mind. Let's go to Italy instead.".to_string(),
                    expected_context_integrity: 0.8,
                    expected_grounding_score: 0.7,
                    max_hallucination_risk: 0.3,
                    expected_controller_intervention: true, // Should trigger context shift handling
                },
                BenchmarkTurn {
                    user_input: "What was I originally planning to visit?".to_string(),
                    expected_context_integrity: 0.75,
                    expected_grounding_score: 0.6,
                    max_hallucination_risk: 0.2,
                    expected_controller_intervention: false,
                },
            ],
            expected_behaviors: vec![
                ExpectedBehavior {
                    behavior_type: BehaviorType::ContextCompression,
                    description: "Context should be compressed after multiple turns".to_string(),
                    trigger_turns: vec![2, 3],
                },
                ExpectedBehavior {
                    behavior_type: BehaviorType::CheckpointIntervention,
                    description: "Controller should intervene on topic shift".to_string(),
                    trigger_turns: vec![2],
                },
            ],
        }
    }

    /// Create a hallucination stress test
    pub fn hallucination_stress() -> Self {
        Self {
            name: "Hallucination Stress Test".to_string(),
            description: "Tests hallucination detection with misleading queries".to_string(),
            turns: vec![
                BenchmarkTurn {
                    user_input: "Tell me about the famous purple elephants of Antarctica.".to_string(),
                    expected_context_integrity: 0.7,
                    expected_grounding_score: 0.3,
                    max_hallucination_risk: 0.6,
                    expected_controller_intervention: true,
                },
                BenchmarkTurn {
                    user_input: "What did they eat?".to_string(),
                    expected_context_integrity: 0.6,
                    expected_grounding_score: 0.2,
                    max_hallucination_risk: 0.7,
                    expected_controller_intervention: true,
                },
                BenchmarkTurn {
                    user_input: "Actually, that was fictional. Tell me about real Antarctic wildlife.".to_string(),
                    expected_context_integrity: 0.8,
                    expected_grounding_score: 0.8,
                    max_hallucination_risk: 0.2,
                    expected_controller_intervention: false,
                },
            ],
            expected_behaviors: vec![
                ExpectedBehavior {
                    behavior_type: BehaviorType::HallucinationMitigation,
                    description: "VCP should detect and mitigate hallucinations".to_string(),
                    trigger_turns: vec![0, 1],
                },
            ],
        }
    }

    /// Execute the benchmark
    pub async fn execute(
        &self,
        orchestrator: Arc<Mutex<ASIOrchestrator>>,
    ) -> Result<BenchmarkResult, Box<dyn std::error::Error + Send + Sync>> {
        let session_id = uuid::Uuid::new_v4().to_string();
        let start_time = Instant::now();
        let mut turn_results = Vec::new();
        let mut errors = Vec::new();

        for (turn_index, turn) in self.turns.iter().enumerate() {
            let turn_start = Instant::now();
            
            // Execute the turn
            let asi = orchestrator.lock().await;
            let result = asi.process_controlled(&session_id, &turn.user_input, 256).await;
            drop(asi);

            let turn_latency = turn_start.elapsed().as_millis() as u64;

            match result {
                Ok(output) => {
                    // Get session state for metrics
                    let asi = orchestrator.lock().await;
                    let session_state = asi.get_session_state(&session_id).await.unwrap_or_else(|_| {
                        // Create dummy state if not found
                        Arc::new(tokio::sync::RwLock::new(
                            crate::ai::orchestrator::CognitiveControlState::new(session_id.clone())
                        ))
                    });
                    drop(asi);

                    let state = session_state.read().await;
                    let actual_hallucination_risk = state.last_vcp_risk.unwrap_or(0.0);
                    let checkpoint_hit = matches!(output.flux_position, 3 | 6 | 9);
                    let controller_intervention = matches!(output.flux_position, 3 | 6 | 9) && actual_hallucination_risk > 0.3;

                    // Calculate actual metrics (simplified for now)
                    let actual_context_integrity = self.calculate_context_integrity(&output.result);
                    let actual_grounding_score = self.calculate_grounding_score(&output.result);

                    let turn_passed = self.evaluate_turn_pass(
                        turn,
                        actual_context_integrity,
                        actual_grounding_score,
                        actual_hallucination_risk,
                        controller_intervention,
                    );

                    let mut issues = Vec::new();
                    if actual_context_integrity < turn.expected_context_integrity {
                        issues.push(format!(
                            "Context integrity {} below expected {}",
                            actual_context_integrity, turn.expected_context_integrity
                        ));
                    }
                    if actual_grounding_score < turn.expected_grounding_score {
                        issues.push(format!(
                            "Grounding score {} below expected {}",
                            actual_grounding_score, turn.expected_grounding_score
                        ));
                    }
                    if actual_hallucination_risk > turn.max_hallucination_risk {
                        issues.push(format!(
                            "Hallucination risk {} above threshold {}",
                            actual_hallucination_risk, turn.max_hallucination_risk
                        ));
                    }
                    if turn.expected_controller_intervention && !controller_intervention {
                        issues.push("Expected controller intervention but none occurred".to_string());
                    }

                    turn_results.push(TurnResult {
                        turn_index,
                        user_input: turn.user_input.clone(),
                        generated_response: output.result,
                        actual_context_integrity,
                        actual_grounding_score,
                        actual_hallucination_risk,
                        controller_intervention,
                        latency_ms: turn_latency,
                        checkpoint_hit,
                        passed: turn_passed,
                        issues,
                    });
                }
                Err(e) => {
                    errors.push(format!("Turn {} failed: {}", turn_index, e));
                    turn_results.push(TurnResult {
                        turn_index,
                        user_input: turn.user_input.clone(),
                        generated_response: String::new(),
                        actual_context_integrity: 0.0,
                        actual_grounding_score: 0.0,
                        actual_hallucination_risk: 1.0,
                        controller_intervention: false,
                        latency_ms: turn_latency,
                        checkpoint_hit: false,
                        passed: false,
                        issues: vec![format!("Execution error: {}", e)],
                    });
                }
            }
        }

        let execution_time = start_time.elapsed().as_millis() as u64;
        let overall_metrics = self.calculate_overall_metrics(&turn_results);
        let passed = self.evaluate_benchmark_pass(&turn_results, &errors);

        Ok(BenchmarkResult {
            benchmark_name: self.name.clone(),
            session_id,
            execution_time_ms: execution_time,
            turn_results,
            overall_metrics,
            passed,
            errors,
        })
    }

    fn calculate_context_integrity(&self, response: &str) -> f32 {
        // Simplified context integrity calculation
        // In a real implementation, this would compare against compressed context
        let response_length = response.len();
        let unique_words = response.split_whitespace().collect::<std::collections::HashSet<_>>().len();
        
        if response_length == 0 {
            return 0.0;
        }
        
        // Basic coherence score based on word diversity
        let diversity_ratio = unique_words as f32 / response.split_whitespace().count() as f32;
        (diversity_ratio * 0.7 + 0.3).min(1.0)
    }

    fn calculate_grounding_score(&self, response: &str) -> f32 {
        // Simplified grounding score
        // In a real implementation, this would check against retrieved documents
        let factual_indicators = ["according to", "research shows", "studies indicate", "data suggests"];
        let factual_count = factual_indicators.iter()
            .map(|indicator| response.matches(indicator).count())
            .sum::<usize>();
        
        let sentences = response.split(['.', '!', '?']).count();
        if sentences == 0 {
            return 0.0;
        }
        
        (factual_count as f32 / sentences as f32).min(1.0)
    }

    fn evaluate_turn_pass(
        &self,
        turn: &BenchmarkTurn,
        actual_context_integrity: f32,
        actual_grounding_score: f32,
        actual_hallucination_risk: f32,
        controller_intervention: bool,
    ) -> bool {
        actual_context_integrity >= turn.expected_context_integrity
            && actual_grounding_score >= turn.expected_grounding_score
            && actual_hallucination_risk <= turn.max_hallucination_risk
            && (turn.expected_controller_intervention == controller_intervention)
    }

    fn calculate_overall_metrics(&self, turn_results: &[TurnResult]) -> EvaluationMetrics {
        let turn_count = turn_results.len() as u64;
        if turn_count == 0 {
            return EvaluationMetrics::new();
        }

        let context_integrity_sum: f32 = turn_results.iter().map(|t| t.actual_context_integrity).sum();
        let grounding_score_sum: f32 = turn_results.iter().map(|t| t.actual_grounding_score).sum();
        let hallucination_risks: Vec<f32> = turn_results.iter().map(|t| t.actual_hallucination_risk).collect();
        let controller_interventions: u64 = turn_results.iter().map(|t| t.controller_intervention as u64).sum();
        let latency_sum: u64 = turn_results.iter().map(|t| t.latency_ms).sum();
        let checkpoint_hits: u64 = turn_results.iter().map(|t| t.checkpoint_hit as u64).sum();

        let session_metrics = SessionMetrics {
            turn_count,
            compression_efficiency: 0.8, // Simplified
            memory_usage_ratio: 0.6,     // Simplified
            checkpoint_hit_rate: checkpoint_hits as f32 / turn_count as f32,
        };

        EvaluationMetrics {
            context_integrity: context_integrity_sum / turn_count as f32,
            grounding_score: grounding_score_sum / turn_count as f32,
            hallucination_risk_trend: hallucination_risks,
            controller_compliance: controller_interventions as f32 / turn_count as f32,
            avg_latency_ms: latency_sum as f64 / turn_count as f64,
            cost_per_request: 1.0, // Simplified
            session_metrics,
        }
    }

    fn evaluate_benchmark_pass(&self, turn_results: &[TurnResult], errors: &[String]) -> bool {
        let passed_turns = turn_results.iter().filter(|t| t.passed).count();
        let total_turns = turn_results.len();
        
        // Pass if at least 80% of turns pass and no critical errors
        (passed_turns as f32 / total_turns as f32) >= 0.8 && errors.is_empty()
    }
}

impl GroundingBenchmark {
    /// Create a basic grounding benchmark
    pub fn basic() -> Self {
        Self {
            name: "Basic RAG Grounding".to_string(),
            queries: vec![
                GroundingQuery {
                    query: "What is the capital of France?".to_string(),
                    expected_retrieval_count: 3,
                    expected_min_similarity: 0.7,
                    expected_factual_accuracy: 0.9,
                },
                GroundingQuery {
                    query: "Who wrote Romeo and Juliet?".to_string(),
                    expected_retrieval_count: 2,
                    expected_min_similarity: 0.8,
                    expected_factual_accuracy: 0.95,
                },
                GroundingQuery {
                    query: "What is the speed of light?".to_string(),
                    expected_retrieval_count: 3,
                    expected_min_similarity: 0.75,
                    expected_factual_accuracy: 0.9,
                },
            ],
            knowledge_base: vec![
                "Paris is the capital city of France, located in the northern part of the country.".to_string(),
                "William Shakespeare wrote the tragedy Romeo and Juliet around 1594.".to_string(),
                "The speed of light in vacuum is approximately 299,792,458 meters per second.".to_string(),
                "France is a country in Western Europe with Paris as its capital city.".to_string(),
                "Shakespeare is considered the greatest writer in the English language.".to_string(),
                "Light travels at different speeds through different media.".to_string(),
            ],
        }
    }
}

/// Benchmark suite containing multiple benchmarks
pub struct BenchmarkSuite {
    pub benchmarks: Vec<MultiTurnBenchmark>,
    pub grounding_benchmarks: Vec<GroundingBenchmark>,
}

impl BenchmarkSuite {
    pub fn standard() -> Self {
        Self {
            benchmarks: vec![
                MultiTurnBenchmark::basic_integrity(),
                MultiTurnBenchmark::hallucination_stress(),
            ],
            grounding_benchmarks: vec![
                GroundingBenchmark::basic(),
            ],
        }
    }

    /// Execute all benchmarks
    pub async fn execute_all(
        &self,
        orchestrator: Arc<Mutex<ASIOrchestrator>>,
    ) -> Vec<BenchmarkResult> {
        let mut results = Vec::new();

        for benchmark in &self.benchmarks {
            match benchmark.execute(orchestrator.clone()).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    eprintln!("Benchmark {} failed: {}", benchmark.name, e);
                }
            }
        }

        results
    }

    /// Generate scorecard from benchmark results
    pub fn generate_scorecard(results: &[BenchmarkResult]) -> EvaluationScorecard {
        let session_id = "benchmark_suite".to_string();
        let mut scorecard = EvaluationScorecard::new(session_id);

        if results.is_empty() {
            return scorecard;
        }

        // Aggregate metrics from all benchmarks
        let total_context_integrity: f32 = results.iter().map(|r| r.overall_metrics.context_integrity).sum();
        let total_grounding_score: f32 = results.iter().map(|r| r.overall_metrics.grounding_score).sum();
        let total_controller_compliance: f32 = results.iter().map(|r| r.overall_metrics.controller_compliance).sum();
        let total_latency: f64 = results.iter().map(|r| r.overall_metrics.avg_latency_ms).sum();

        let count = results.len() as f32;

        scorecard.metrics.context_integrity = total_context_integrity / count;
        scorecard.metrics.grounding_score = total_grounding_score / count;
        scorecard.metrics.controller_compliance = total_controller_compliance / count;
        scorecard.metrics.avg_latency_ms = total_latency / count as f64;

        // Collect all hallucination risks
        for result in results {
            scorecard.metrics.hallucination_risk_trend.extend(result.overall_metrics.hallucination_risk_trend.clone());
        }

        // Update KPI summary
        scorecard.kpi_summary = crate::evaluation::metrics::KPISummary::from_metrics(&scorecard.metrics);
        scorecard.generate_recommendations();

        scorecard
    }
}
