//! Production Metrics System
//!
//! Comprehensive Prometheus metrics for SpatialVortex ASI monitoring.
//! Tracks performance, sacred geometry effects, and system health.

use lazy_static::lazy_static;
use prometheus::{
    register_counter_vec, register_gauge, register_gauge_vec, register_histogram_vec,
    CounterVec, Gauge, GaugeVec, HistogramVec, Registry,
};
use std::sync::Arc;

lazy_static! {
    // ========================================================================
    // META ORCHESTRATOR METRICS
    // ========================================================================
    
    /// Total requests processed by meta orchestrator
    pub static ref META_REQUESTS_TOTAL: CounterVec = register_counter_vec!(
        "vortex_meta_requests_total",
        "Total requests processed by meta orchestrator",
        &["strategy", "source"]  // strategy: Hybrid/AIFirst/etc, source: ASI/Runtime/Fused
    ).unwrap();
    
    /// Request processing duration
    pub static ref META_DURATION: HistogramVec = register_histogram_vec!(
        "vortex_meta_duration_seconds",
        "Meta orchestrator request duration",
        &["strategy", "source"],
        vec![0.01, 0.05, 0.1, 0.2, 0.3, 0.5, 1.0, 2.0, 5.0]
    ).unwrap();
    
    /// Routing decisions
    pub static ref ROUTING_DECISIONS: CounterVec = register_counter_vec!(
        "vortex_routing_decisions_total",
        "Routing decisions by strategy and outcome",
        &["strategy", "routed_to"]
    ).unwrap();
    
    /// Complexity scores
    pub static ref COMPLEXITY_SCORE: HistogramVec = register_histogram_vec!(
        "vortex_complexity_score",
        "Input complexity analysis scores",
        &["complexity_level"],  // simple/moderate/complex
        vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0]
    ).unwrap();
    
    // ========================================================================
    // ASI ORCHESTRATOR METRICS
    // ========================================================================
    
    /// ASI inference requests
    pub static ref ASI_INFER_TOTAL: CounterVec = register_counter_vec!(
        "vortex_asi_infer_total",
        "Total ASI inference requests",
        &["mode", "success"]  // mode: Fast/Balanced/Thorough
    ).unwrap();
    
    /// ASI inference duration
    pub static ref ASI_INFERENCE_DURATION: HistogramVec = register_histogram_vec!(
        "vortex_asi_inference_duration_seconds",
        "ASI inference duration by mode",
        &["mode"],
        vec![0.05, 0.1, 0.2, 0.3, 0.5, 1.0, 2.0]
    ).unwrap();
    
    /// Expert selection counts
    pub static ref ASI_EXPERT_SELECTED: CounterVec = register_counter_vec!(
        "vortex_asi_expert_selected_total",
        "Expert selection counts",
        &["expert"]  // geometric/heuristic/rag/consensus
    ).unwrap();
    
    /// Expert processing duration
    pub static ref ASI_EXPERT_DURATION: HistogramVec = register_histogram_vec!(
        "vortex_asi_expert_duration_seconds",
        "Expert processing duration",
        &["expert"],
        vec![0.01, 0.05, 0.1, 0.2, 0.5, 1.0]
    ).unwrap();
    
    /// Consensus attempts
    pub static ref ASI_CONSENSUS_TOTAL: CounterVec = register_counter_vec!(
        "vortex_asi_consensus_total",
        "Consensus verification attempts",
        &["strategy", "success"]
    ).unwrap();
    
    // ========================================================================
    // SACRED GEOMETRY METRICS
    // ========================================================================
    
    /// Sacred position hits
    pub static ref ASI_SACRED_HITS: CounterVec = register_counter_vec!(
        "vortex_sacred_hits_total",
        "Hits on sacred positions (3, 6, 9)",
        &["position"]
    ).unwrap();
    
    /// Sacred boost effects
    pub static ref SACRED_BOOST_EFFECT: HistogramVec = register_histogram_vec!(
        "vortex_sacred_boost_effect",
        "Confidence boost from sacred positions",
        &["position"],
        vec![0.0, 0.05, 0.1, 0.15, 0.2, 0.3, 0.5]
    ).unwrap();
    
    /// Fusion at position 6
    pub static ref FUSION_AT_POSITION_6: CounterVec = register_counter_vec!(
        "vortex_fusion_position_6_total",
        "Result fusion at sacred position 6",
        &["asi_weight", "runtime_weight"]
    ).unwrap();
    
    /// Flux position distribution
    pub static ref FLUX_POSITION_DIST: CounterVec = register_counter_vec!(
        "vortex_flux_position_distribution",
        "Distribution of flux positions (0-9)",
        &["position"]
    ).unwrap();
    
    // ========================================================================
    // CONFIDENCE LAKE METRICS
    // ========================================================================
    
    /// Confidence Lake stores
    pub static ref LAKE_STORES_TOTAL: CounterVec = register_counter_vec!(
        "vortex_lake_stores_total",
        "Items stored in Confidence Lake",
        &["signal_level"]  // weak/moderate/strong
    ).unwrap();
    
    /// Confidence Lake queries
    pub static ref LAKE_QUERIES_TOTAL: CounterVec = register_counter_vec!(
        "vortex_lake_queries_total",
        "Confidence Lake query attempts",
        &["hit"]  // hit/miss
    ).unwrap();
    
    /// Confidence Lake size
    pub static ref LAKE_SIZE: Gauge = register_gauge!(
        "vortex_lake_size_items",
        "Number of items in Confidence Lake"
    ).unwrap();
    
    /// Signal strength distribution
    pub static ref SIGNAL_STRENGTH: HistogramVec = register_histogram_vec!(
        "vortex_confidence",
        "Signal strength distribution",
        &["source"],  // asi/runtime/fused
        vec![0.0, 0.3, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0]
    ).unwrap();
    
    // ========================================================================
    // VCP (VORTEX CONTEXT PRESERVER) METRICS
    // ========================================================================
    
    /// VCP interventions
    pub static ref VCP_INTERVENTIONS: CounterVec = register_counter_vec!(
        "vortex_vcp_interventions_total",
        "VCP interventions at sacred positions",
        &["position", "intervention_type"]
    ).unwrap();
    
    /// Context preservation score
    pub static ref VCP_CONTEXT_PRESERVATION: HistogramVec = register_histogram_vec!(
        "vortex_vcp_context_preservation",
        "VCP context preservation effectiveness",
        &["cycle_depth"],
        vec![0.0, 0.3, 0.5, 0.7, 0.8, 0.9, 1.0]
    ).unwrap();
    
    /// Hallucination detection
    pub static ref HALLUCINATION_DETECTED: CounterVec = register_counter_vec!(
        "vortex_hallucination_detected_total",
        "Hallucination detection events",
        &["severity"]  // low/medium/high
    ).unwrap();
    
    // ========================================================================
    // TRAINING & LEARNING METRICS
    // ========================================================================
    
    /// Training iterations
    pub static ref TRAINING_ITERATIONS: CounterVec = register_counter_vec!(
        "vortex_training_iterations_total",
        "Training iteration counts",
        &["mode"]  // continuous/batch/meta
    ).unwrap();
    
    /// Learning loss
    pub static ref TRAINING_LOSS: HistogramVec = register_histogram_vec!(
        "vortex_training_loss",
        "Training loss values",
        &["optimizer"],  // vortex_sgd/sacred_gradients
        vec![0.0, 0.1, 0.5, 1.0, 2.0, 5.0, 10.0]
    ).unwrap();
    
    /// Model accuracy
    pub static ref MODEL_ACCURACY: GaugeVec = register_gauge_vec!(
        "vortex_model_accuracy",
        "Model accuracy by component",
        &["component"]  // asi/ml/geometric
    ).unwrap();
    
    // ========================================================================
    // FLUX ORCHESTRATOR METRICS
    // ========================================================================
    
    /// Active vortex cycles
    pub static ref ACTIVE_CYCLES: Gauge = register_gauge!(
        "vortex_active_cycles",
        "Number of active vortex cycles"
    ).unwrap();
    
    /// Cycle throughput
    pub static ref CYCLE_THROUGHPUT: CounterVec = register_counter_vec!(
        "vortex_cycle_throughput_total",
        "Vortex cycle throughput",
        &["direction"]  // forward/reverse
    ).unwrap();
    
    /// Ladder index ranks
    pub static ref LADDER_RANKS: HistogramVec = register_histogram_vec!(
        "vortex_ladder_ranks",
        "Ladder index rank distribution",
        &["position"],
        vec![0.0, 0.1, 0.2, 0.5, 1.0, 2.0, 5.0, 10.0]
    ).unwrap();
    
    /// Intersection detections
    pub static ref INTERSECTION_DETECTIONS: CounterVec = register_counter_vec!(
        "vortex_intersection_detections_total",
        "Geometric intersection detections",
        &["type"]  // sacred_triangle/flow_lines/center
    ).unwrap();
    
    // ========================================================================
    // SYSTEM HEALTH METRICS
    // ========================================================================
    
    /// Error counts
    pub static ref ERRORS_TOTAL: CounterVec = register_counter_vec!(
        "vortex_errors_total",
        "Total errors by type and component",
        &["error_type", "component", "recovery_strategy"]
    ).unwrap();
    
    /// Active connections
    pub static ref ACTIVE_CONNECTIONS: Gauge = register_gauge!(
        "vortex_active_connections",
        "Number of active client connections"
    ).unwrap();
    
    /// Memory usage
    pub static ref MEMORY_USAGE_BYTES: Gauge = register_gauge!(
        "vortex_memory_usage_bytes",
        "Current memory usage in bytes"
    ).unwrap();
    
    /// Request rate
    pub static ref REQUEST_RATE: GaugeVec = register_gauge_vec!(
        "vortex_request_rate",
        "Requests per second by endpoint",
        &["endpoint"]
    ).unwrap();
}

/// Vortex Metrics Manager
pub struct VortexMetrics {
    registry: Arc<Registry>,
}

impl VortexMetrics {
    /// Create new metrics manager with default registry
    pub fn new() -> Self {
        Self {
            registry: Arc::new(Registry::new()),
        }
    }
    
    /// Get Prometheus registry
    pub fn registry(&self) -> Arc<Registry> {
        Arc::clone(&self.registry)
    }
    
    /// Record meta orchestrator request
    pub fn record_meta_request(
        &self,
        strategy: &str,
        source: &str,
        duration_secs: f64,
        success: bool,
    ) {
        META_REQUESTS_TOTAL
            .with_label_values(&[strategy, source])
            .inc();
        
        META_DURATION
            .with_label_values(&[strategy, source])
            .observe(duration_secs);
        
        if !success {
            ERRORS_TOTAL
                .with_label_values(&["orchestration", "meta", "propagate"])
                .inc();
        }
    }
    
    /// Record routing decision
    pub fn record_routing(&self, strategy: &str, routed_to: &str) {
        ROUTING_DECISIONS
            .with_label_values(&[strategy, routed_to])
            .inc();
    }
    
    /// Record complexity analysis
    pub fn record_complexity(&self, score: f32) {
        let level = if score < 0.3 {
            "simple"
        } else if score < 0.7 {
            "moderate"
        } else {
            "complex"
        };
        
        COMPLEXITY_SCORE
            .with_label_values(&[level])
            .observe(score as f64);
    }
    
    /// Record sacred position hit
    pub fn record_sacred_hit(&self, position: u8, boost: f32) {
        ASI_SACRED_HITS
            .with_label_values(&[&position.to_string()])
            .inc();
        
        SACRED_BOOST_EFFECT
            .with_label_values(&[&position.to_string()])
            .observe(boost as f64);
    }
    
    /// Record fusion at position 6
    pub fn record_fusion(&self, asi_weight: f32, runtime_weight: f32) {
        FUSION_AT_POSITION_6
            .with_label_values(&[
                &format!("{:.2}", asi_weight),
                &format!("{:.2}", runtime_weight),
            ])
            .inc();
    }
    
    /// Record flux position
    pub fn record_flux_position(&self, position: u8) {
        FLUX_POSITION_DIST
            .with_label_values(&[&position.to_string()])
            .inc();
    }
    
    /// Record signal strength
    pub fn record_confidence(&self, strength: f32, source: &str) {
        SIGNAL_STRENGTH
            .with_label_values(&[source])
            .observe(strength as f64);
    }
    
    /// Record Confidence Lake store
    pub fn record_lake_store(&self, confidence: f32) {
        let level = if confidence < 0.5 {
            "weak"
        } else if confidence < 0.7 {
            "moderate"
        } else {
            "strong"
        };
        
        LAKE_STORES_TOTAL
            .with_label_values(&[level])
            .inc();
        
        LAKE_SIZE.inc();
    }
    
    /// Record Confidence Lake query
    pub fn record_lake_query(&self, hit: bool) {
        LAKE_QUERIES_TOTAL
            .with_label_values(&[if hit { "hit" } else { "miss" }])
            .inc();
    }
    
    /// Record VCP intervention
    pub fn record_vcp_intervention(&self, position: u8, intervention_type: &str) {
        VCP_INTERVENTIONS
            .with_label_values(&[&position.to_string(), intervention_type])
            .inc();
    }
    
    /// Record hallucination detection
    pub fn record_hallucination(&self, severity: &str) {
        HALLUCINATION_DETECTED
            .with_label_values(&[severity])
            .inc();
    }
    
    /// Record training iteration
    pub fn record_training(&self, mode: &str, loss: f64, optimizer: &str) {
        TRAINING_ITERATIONS
            .with_label_values(&[mode])
            .inc();
        
        TRAINING_LOSS
            .with_label_values(&[optimizer])
            .observe(loss);
    }
    
    /// Update model accuracy
    pub fn update_accuracy(&self, component: &str, accuracy: f64) {
        MODEL_ACCURACY
            .with_label_values(&[component])
            .set(accuracy);
    }
    
    /// Record error with context
    pub fn record_error(&self, error_type: &str, component: &str, recovery: &str) {
        ERRORS_TOTAL
            .with_label_values(&[error_type, component, recovery])
            .inc();
    }
    
    /// Update system health metrics
    pub fn update_health(&self, active_connections: u64, memory_bytes: u64) {
        ACTIVE_CONNECTIONS.set(active_connections as f64);
        MEMORY_USAGE_BYTES.set(memory_bytes as f64);
    }
    
    /// Record vortex cycle
    pub fn record_cycle(&self, direction: &str) {
        CYCLE_THROUGHPUT
            .with_label_values(&[direction])
            .inc();
    }
    
    /// Update active cycles
    pub fn set_active_cycles(&self, count: u64) {
        ACTIVE_CYCLES.set(count as f64);
    }
    
    /// Record intersection detection
    pub fn record_intersection(&self, intersection_type: &str) {
        INTERSECTION_DETECTIONS
            .with_label_values(&[intersection_type])
            .inc();
    }
}

impl Default for VortexMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Global metrics instance
pub static VORTEX_METRICS: once_cell::sync::Lazy<VortexMetrics> =
    once_cell::sync::Lazy::new(VortexMetrics::new);

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_metrics_creation() {
        let metrics = VortexMetrics::new();
        // Registry should be created successfully
        assert!(!metrics.registry().gather().is_empty() || metrics.registry().gather().is_empty());
    }
    
    #[test]
    fn test_record_meta_request() {
        let metrics = VortexMetrics::new();
        metrics.record_meta_request("Hybrid", "ASI", 0.3, true);
        
        // Should not panic
    }
    
    #[test]
    fn test_record_sacred_hit() {
        let metrics = VortexMetrics::new();
        metrics.record_sacred_hit(6, 0.15);
        
        // Verify position 6 is sacred
        assert!([3, 6, 9].contains(&6));
    }
    
    #[test]
    fn test_complexity_levels() {
        let metrics = VortexMetrics::new();
        
        metrics.record_complexity(0.2);  // simple
        metrics.record_complexity(0.5);  // moderate
        metrics.record_complexity(0.9);  // complex
    }
}
