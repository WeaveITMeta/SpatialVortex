//! Meta Orchestrator - Unified ASI Coordinator
//!
//! Coordinates both ASIOrchestrator (AI/ML) and FluxOrchestrator (runtime/cycles)
//! into a single unified intelligence system with smart routing and result fusion.
//!
//! ## Architecture
//!
//! ```text
//! Input → [Complexity Analysis] → Route to:
//!         ├─ ASI Orchestrator (complex queries, AI needed)
//!         ├─ Flux Orchestrator (cycles, real-time)
//!         └─ Parallel Fusion (both, fuse at sacred positions)
//! ```
//!
//! ## Routing Strategies
//!
//! - **AIFirst**: Always use ASI (high accuracy, higher latency)
//! - **RuntimeFirst**: Always use Flux (fast, geometric only)
//! - **Hybrid**: Route based on complexity analysis
//! - **ParallelFusion**: Run both, fuse at sacred position 6
//!
//! ## Example
//!
//! ```no_run
//! use spatial_vortex::ai::meta_orchestrator::{MetaOrchestrator, RoutingStrategy};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let meta = MetaOrchestrator::new(RoutingStrategy::Hybrid).await?;
//!     
//!     let result = meta.process_unified("What is consciousness?").await?;
//!     println!("Result: {}", result.content);
//!     println!("Confidence: {:.2}", result.confidence);
//!     println!("Used: {:?}", result.orchestrators_used);
//!     
//!     Ok(())
//! }
//! ```

use crate::error::{Result, ErrorContext};
use crate::ai::orchestrator::{ASIOrchestrator, ExecutionMode};
use crate::processing::runtime::orchestrator::FluxOrchestrator;
use crate::models::ELPTensor;
use crate::processing::runtime::vortex_cycle::CycleDirection;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Routing strategy for meta orchestrator
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RoutingStrategy {
    /// Always use ASI Orchestrator (high accuracy, ~300-500ms)
    AIFirst,
    
    /// Always use Flux Orchestrator (fast, geometric only, ~50ms)
    RuntimeFirst,
    
    /// Route based on complexity (simple → Runtime, complex → ASI)
    Hybrid,
    
    /// Run both in parallel, fuse results at sacred position (Position 6: Harmonic Balance)
    ParallelFusion,
    
    /// Adaptive routing based on past performance
    Adaptive,
}

/// Which orchestrator(s) were used
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrchestratorSource {
    ASI,
    Runtime,
    Fused { asi_weight: f32, runtime_weight: f32 },
}

/// Unified result from meta orchestrator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedResult {
    /// Final response content
    pub content: String,
    
    /// Confidence score (0.0-1.0) and signal strength (for VCP)
    pub confidence: f32,
    
    /// Flux position (0-9)
    pub flux_position: u8,
    
    /// ELP tensor
    pub elp: ELPTensor,
    
    /// Which orchestrator(s) produced this result
    pub orchestrators_used: OrchestratorSource,
    
    /// Sacred position intervention applied
    pub sacred_boost: bool,
    
    /// Processing duration in milliseconds
    pub duration_ms: u64,
    
    /// Metadata
    pub metadata: UnifiedMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedMetadata {
    pub asi_mode: Option<ExecutionMode>,
    pub vortex_cycles: u32,
    pub confidence_lake_hit: bool,
    pub routing_strategy: String,
}

/// Complexity analysis result
#[derive(Debug, Clone)]
struct ComplexityAnalysis {
    score: f32,  // 0.0-1.0, higher = more complex
    requires_ai: bool,
    
    // Reserved for future adaptive routing enhancements
    #[allow(dead_code)]
    word_count: usize,
    #[allow(dead_code)]
    has_question: bool,
    #[allow(dead_code)]
    has_code: bool,
    #[allow(dead_code)]
    has_math: bool,
}

/// Performance metrics for adaptive routing
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub asi_success_rate: f32,
    pub runtime_success_rate: f32,
    pub asi_avg_latency_ms: f64,
    pub runtime_avg_latency_ms: f64,
}

/// Meta Orchestrator - Unified ASI
pub struct MetaOrchestrator {
    /// ASI Orchestrator (AI/ML)
    asi_orchestrator: Arc<RwLock<ASIOrchestrator>>,
    
    /// Flux Orchestrator (runtime cycles)
    flux_orchestrator: Arc<FluxOrchestrator>,
    
    /// Current routing strategy
    routing_strategy: RoutingStrategy,
    
    /// Performance metrics for adaptive routing
    performance: Arc<RwLock<PerformanceMetrics>>,
    
    /// Complexity threshold for hybrid mode (0.0-1.0)
    complexity_threshold: f32,
}

impl MetaOrchestrator {
    /// Create new meta orchestrator
    pub async fn new(routing_strategy: RoutingStrategy) -> Result<Self> {
        let asi_orchestrator = Arc::new(RwLock::new(ASIOrchestrator::new().await?));
        let flux_orchestrator = Arc::new(FluxOrchestrator::new_default());
        
        // Start flux orchestrator background loop
        flux_orchestrator.start().await?;
        
        Ok(Self {
            asi_orchestrator,
            flux_orchestrator,
            routing_strategy,
            performance: Arc::new(RwLock::new(PerformanceMetrics {
                asi_success_rate: 0.95,
                runtime_success_rate: 0.85,
                asi_avg_latency_ms: 300.0,
                runtime_avg_latency_ms: 50.0,
            })),
            complexity_threshold: 0.5,  // Default: route to ASI if complexity > 0.5
        })
    }
    
    /// Create with default hybrid routing
    pub async fn new_default() -> Result<Self> {
        Self::new(RoutingStrategy::Hybrid).await
    }
    
    /// Set routing strategy
    pub fn set_strategy(&mut self, strategy: RoutingStrategy) {
        self.routing_strategy = strategy;
    }
    
    /// Set complexity threshold for hybrid routing
    pub fn set_complexity_threshold(&mut self, threshold: f32) {
        self.complexity_threshold = threshold.clamp(0.0, 1.0);
    }
    
    /// Process input with unified orchestration
    pub async fn process_unified(&self, input: &str) -> Result<UnifiedResult> {
        use tracing::info;
        
        let start = std::time::Instant::now();
        
        info!(
            strategy = ?self.routing_strategy,
            input_len = input.len(),
            "Processing request with meta orchestrator"
        );
        
        let result = match self.routing_strategy {
            RoutingStrategy::AIFirst => {
                self.process_with_asi(input, ExecutionMode::Balanced, start).await
            }
            
            RoutingStrategy::RuntimeFirst => {
                self.process_with_runtime(input, start).await
            }
            
            RoutingStrategy::Hybrid => {
                let complexity = self.analyze_complexity(input);
                
                if complexity.requires_ai || complexity.score > self.complexity_threshold {
                    // Complex query → ASI
                    let mode = if complexity.score > 0.8 {
                        ExecutionMode::Thorough
                    } else {
                        ExecutionMode::Balanced
                    };
                    self.process_with_asi(input, mode, start).await
                } else {
                    // Simple query → Runtime
                    self.process_with_runtime(input, start).await
                }
            }
            
            RoutingStrategy::ParallelFusion => {
                self.process_parallel_fusion(input, start).await
            }
            
            RoutingStrategy::Adaptive => {
                self.process_adaptive(input, start).await
            }
        };
        
        match &result {
            Ok(unified) => {
                info!(
                    strategy = ?self.routing_strategy,
                    source = ?unified.orchestrators_used,
                    confidence = unified.confidence,
                    flux_position = unified.flux_position,
                    sacred = unified.sacred_boost,
                    duration_ms = unified.duration_ms,
                    "Request completed successfully"
                );
            }
            Err(e) => {
                tracing::error!(
                    strategy = ?self.routing_strategy,
                    error = ?e,
                    "Request failed"
                );
            }
        }
        
        result
    }
    
    /// Process with ASI Orchestrator only
    async fn process_with_asi(
        &self,
        input: &str,
        mode: ExecutionMode,
        start: std::time::Instant,
    ) -> Result<UnifiedResult> {
        let mut asi = self.asi_orchestrator.write().await;
        let result = asi.process(input, mode).await.map_err(|e| {
            e.with_context(
                ErrorContext::new()
                    .with_component("MetaOrchestrator")
                    .with_operation("process_with_asi")
            )
        })?;
        
        let duration = start.elapsed().as_millis() as u64;
        
        Ok(UnifiedResult {
            content: result.result,
            confidence: result.confidence,
            flux_position: result.flux_position,
            elp: result.elp,
            orchestrators_used: OrchestratorSource::ASI,
            sacred_boost: result.is_sacred, // Use is_sacred instead of sacred_intervention
            duration_ms: duration,
            metadata: UnifiedMetadata {
                asi_mode: Some(mode),
                vortex_cycles: 0,
                confidence_lake_hit: false, // ASIOutput doesn't track this
                routing_strategy: format!("{:?}", self.routing_strategy),
            },
        })
    }
    
    /// Process with Flux Orchestrator only
    async fn process_with_runtime(
        &self,
        input: &str,
        start: std::time::Instant,
    ) -> Result<UnifiedResult> {
        // Add input to flux orchestrator as a cycle object
        let id = format!("meta_{}", uuid::Uuid::new_v4());
        let position = self.calculate_flux_position(input);
        let elp = self.estimate_elp(input);
        
        self.flux_orchestrator
            .add_object(id.clone(), position, elp.clone(), CycleDirection::Forward)
            .await?;
        
        // Wait for a few cycles to propagate
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        
        // Get state
        let state = self.flux_orchestrator.get_state().await;
        
        let duration = start.elapsed().as_millis() as u64;
        
        Ok(UnifiedResult {
            content: format!("Geometric result: {}", input),
            confidence: state.coherence as f32,
            flux_position: position,
            elp,
            orchestrators_used: OrchestratorSource::Runtime,
            sacred_boost: [3, 6, 9].contains(&position),
            duration_ms: duration,
            metadata: UnifiedMetadata {
                asi_mode: None,
                vortex_cycles: state.total_cycles as u32,
                confidence_lake_hit: false,
                routing_strategy: format!("{:?}", self.routing_strategy),
            },
        })
    }
    
    /// Process with both orchestrators in parallel, fuse at sacred position
    async fn process_parallel_fusion(
        &self,
        input: &str,
        start: std::time::Instant,
    ) -> Result<UnifiedResult> {
        // Run both in parallel
        let (asi_result, runtime_result) = tokio::join!(
            self.process_with_asi(input, ExecutionMode::Fast, start),
            self.process_with_runtime(input, start)
        );
        
        let asi_result = asi_result?;
        let runtime_result = runtime_result?;
        
        // Fuse at sacred position 6 (Harmonic Balance)
        let fusion_position = 6u8;
        
        // Calculate weights based on sacred position proximity
        let asi_weight = if asi_result.flux_position == fusion_position { 1.5 } else { 1.0 };
        let runtime_weight = if runtime_result.flux_position == fusion_position { 1.5 } else { 1.0 };
        
        let total_weight = asi_weight + runtime_weight;
        let normalized_asi = asi_weight / total_weight;
        let normalized_runtime = runtime_weight / total_weight;
        
        // Fuse results
        let fused_confidence = asi_result.confidence * normalized_asi 
                             + runtime_result.confidence * normalized_runtime;
        
        let fused_elp = ELPTensor {
            ethos: asi_result.elp.ethos * normalized_asi as f64 
                 + runtime_result.elp.ethos * normalized_runtime as f64,
            logos: asi_result.elp.logos * normalized_asi as f64 
                 + runtime_result.elp.logos * normalized_runtime as f64,
            pathos: asi_result.elp.pathos * normalized_asi as f64 
                  + runtime_result.elp.pathos * normalized_runtime as f64,
        };
        
        let duration = start.elapsed().as_millis() as u64;
        
        Ok(UnifiedResult {
            content: asi_result.content,  // Prefer ASI content
            confidence: fused_confidence,
            flux_position: fusion_position,
            elp: fused_elp,
            orchestrators_used: OrchestratorSource::Fused {
                asi_weight: normalized_asi,
                runtime_weight: normalized_runtime,
            },
            sacred_boost: true,  // Fusion happens at sacred position
            duration_ms: duration,
            metadata: UnifiedMetadata {
                asi_mode: asi_result.metadata.asi_mode,
                vortex_cycles: runtime_result.metadata.vortex_cycles,
                confidence_lake_hit: asi_result.metadata.confidence_lake_hit,
                routing_strategy: "ParallelFusion".to_string(),
            },
        })
    }
    
    /// Adaptive routing based on past performance
    async fn process_adaptive(
        &self,
        input: &str,
        start: std::time::Instant,
    ) -> Result<UnifiedResult> {
        let perf = self.performance.read().await;
        let complexity = self.analyze_complexity(input);
        
        // Calculate expected value for each option
        let asi_ev = perf.asi_success_rate / ((perf.asi_avg_latency_ms as f32) / 100.0_f32);
        let runtime_ev = perf.runtime_success_rate / ((perf.runtime_avg_latency_ms as f32) / 100.0_f32);
        
        drop(perf);
        
        // Choose based on expected value + complexity
        if complexity.requires_ai || asi_ev > runtime_ev * 1.2 {
            self.process_with_asi(input, ExecutionMode::Balanced, start).await
        } else {
            self.process_with_runtime(input, start).await
        }
    }
    
    /// Analyze input complexity
    fn analyze_complexity(&self, input: &str) -> ComplexityAnalysis {
        let words: Vec<&str> = input.split_whitespace().collect();
        let word_count = words.len();
        
        let has_question = input.contains('?');
        let has_code = input.contains("fn ") || input.contains("def ") || input.contains("```");
        let has_math = input.contains('+') || input.contains('=') || input.contains("calculate");
        
        // Complexity score based on multiple factors
        let mut score = 0.0;
        
        // Length complexity
        score += (word_count as f32 / 100.0).min(0.3);
        
        // Question complexity
        if has_question { score += 0.2; }
        
        // Code complexity
        if has_code { score += 0.3; }
        
        // Math complexity
        if has_math { score += 0.2; }
        
        // Multi-sentence complexity
        let sentences = input.matches('.').count();
        score += (sentences as f32 * 0.1).min(0.3);
        
        let requires_ai = has_code || has_math || word_count > 50;
        
        ComplexityAnalysis {
            score: score.min(1.0),
            requires_ai,
            word_count,
            has_question,
            has_code,
            has_math,
        }
    }
    
    /// Calculate flux position from input
    fn calculate_flux_position(&self, input: &str) -> u8 {
        // Simple hash-based position (0-9)
        let hash = input.bytes().fold(0u64, |acc, b| acc.wrapping_add(b as u64));
        (hash % 10) as u8
    }
    
    /// Estimate ELP tensor from input
    fn estimate_elp(&self, input: &str) -> ELPTensor {
        let words = input.to_lowercase();
        
        // Simple heuristic estimation
        let ethos = if words.contains("should") || words.contains("must") { 6.0 } else { 5.0 };
        let logos = if words.contains("calculate") || words.contains("analyze") { 7.0 } else { 5.0 };
        let pathos = if words.contains("feel") || words.contains("believe") { 6.0 } else { 5.0 };
        
        ELPTensor { ethos, logos, pathos }
    }
    
    /// Update performance metrics (called after each request)
    pub async fn update_metrics(&self, source: &OrchestratorSource, success: bool, latency_ms: u64) {
        let mut perf = self.performance.write().await;
        
        match source {
            OrchestratorSource::ASI => {
                // Exponential moving average
                let alpha_f32 = 0.1_f32;
                let alpha_f64 = 0.1_f64;
                perf.asi_success_rate = perf.asi_success_rate * (1.0_f32 - alpha_f32) 
                                      + (if success { 1.0_f32 } else { 0.0_f32 }) * alpha_f32;
                perf.asi_avg_latency_ms = perf.asi_avg_latency_ms * (1.0_f64 - alpha_f64) 
                                        + latency_ms as f64 * alpha_f64;
            }
            OrchestratorSource::Runtime => {
                let alpha_f32 = 0.1_f32;
                let alpha_f64 = 0.1_f64;
                perf.runtime_success_rate = perf.runtime_success_rate * (1.0_f32 - alpha_f32) 
                                          + (if success { 1.0_f32 } else { 0.0_f32 }) * alpha_f32;
                perf.runtime_avg_latency_ms = perf.runtime_avg_latency_ms * (1.0_f64 - alpha_f64) 
                                            + latency_ms as f64 * alpha_f64;
            }
            OrchestratorSource::Fused { .. } => {
                // Update both
                let alpha_f32 = 0.1_f32;
                let alpha_f64 = 0.1_f64;
                let success_val = if success { 1.0_f32 } else { 0.0_f32 };
                perf.asi_success_rate = perf.asi_success_rate * (1.0_f32 - alpha_f32) + success_val * alpha_f32;
                perf.runtime_success_rate = perf.runtime_success_rate * (1.0_f32 - alpha_f32) + success_val * alpha_f32;
                perf.asi_avg_latency_ms = perf.asi_avg_latency_ms * (1.0_f64 - alpha_f64) + latency_ms as f64 * alpha_f64;
                perf.runtime_avg_latency_ms = perf.runtime_avg_latency_ms * (1.0_f64 - alpha_f64) + latency_ms as f64 * alpha_f64;
            }
        }
    }
    
    /// Get current routing strategy
    pub fn strategy(&self) -> RoutingStrategy {
        self.routing_strategy
    }
    
    /// Get performance metrics
    pub async fn metrics(&self) -> PerformanceMetrics {
        self.performance.read().await.clone()
    }
}

impl Drop for MetaOrchestrator {
    fn drop(&mut self) {
        // Stop flux orchestrator when dropped
        let flux = Arc::clone(&self.flux_orchestrator);
        tokio::spawn(async move {
            flux.stop().await;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_meta_orchestrator_creation() {
        let meta = MetaOrchestrator::new_default().await;
        assert!(meta.is_ok());
    }
    
    #[tokio::test]
    async fn test_complexity_analysis() {
        let meta = MetaOrchestrator::new_default().await.unwrap();
        
        let simple = meta.analyze_complexity("Hello");
        assert!(simple.score < 0.3);
        
        let complex = meta.analyze_complexity(
            "Calculate the derivative of f(x) = x^2 + 3x using the power rule and explain why."
        );
        assert!(complex.score > 0.5);
        assert!(complex.requires_ai);
    }
    
    #[tokio::test]
    async fn test_routing_strategies() {
        let mut meta = MetaOrchestrator::new_default().await.unwrap();
        
        // Test strategy switching
        meta.set_strategy(RoutingStrategy::AIFirst);
        assert_eq!(meta.strategy(), RoutingStrategy::AIFirst);
        
        meta.set_strategy(RoutingStrategy::RuntimeFirst);
        assert_eq!(meta.strategy(), RoutingStrategy::RuntimeFirst);
    }
}
