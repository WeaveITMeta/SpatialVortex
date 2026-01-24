//! Mixture of Experts (MoE) Gate - Bottleneck Resolution
//!
//! Adaptive expert selection for routing requests to optimal processing paths.
//! Adapted from SpatialVortex metrics and self_optimization modules.
//!
//! Key features:
//! - Dynamic expert weighting based on confidence
//! - Automatic bottleneck detection and rerouting
//! - Sacred geometry integration for expert selection

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Expert types available in the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExpertType {
    /// Geometric reasoning (sacred geometry)
    Geometric,
    /// Machine learning inference
    ML,
    /// Retrieval augmented generation
    RAG,
    /// Heuristic rules
    Heuristic,
    /// Multi-model consensus
    Consensus,
}

impl ExpertType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ExpertType::Geometric => "geometric",
            ExpertType::ML => "ml",
            ExpertType::RAG => "rag",
            ExpertType::Heuristic => "heuristic",
            ExpertType::Consensus => "consensus",
        }
    }
}

/// An expert in the MoE system
#[derive(Debug, Clone)]
pub struct Expert {
    pub expert_type: ExpertType,
    pub weight: f32,
    pub confidence: f32,
    pub latency_ms: f32,
    pub error_rate: f32,
    pub invocation_count: u64,
}

impl Expert {
    pub fn new(expert_type: ExpertType) -> Self {
        Self {
            expert_type,
            weight: 1.0,
            confidence: 0.5,
            latency_ms: 0.0,
            error_rate: 0.0,
            invocation_count: 0,
        }
    }

    /// Calculate effective score for routing
    pub fn effective_score(&self) -> f32 {
        // Higher confidence, lower latency, lower error = better
        let latency_penalty = (self.latency_ms / 100.0).min(1.0);
        let error_penalty = self.error_rate;
        
        self.weight * self.confidence * (1.0 - latency_penalty) * (1.0 - error_penalty)
    }

    /// Update statistics after invocation
    pub fn record_invocation(&mut self, confidence: f32, latency_ms: f32, success: bool) {
        self.invocation_count += 1;
        
        // Exponential moving average
        let alpha = 0.1;
        self.confidence = self.confidence * (1.0 - alpha) + confidence * alpha;
        self.latency_ms = self.latency_ms * (1.0 - alpha) + latency_ms * alpha;
        
        let error = if success { 0.0 } else { 1.0 };
        self.error_rate = self.error_rate * (1.0 - alpha) + error * alpha;
    }
}

/// MoE configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoEConfig {
    /// Number of experts to select per query
    pub top_k: usize,
    /// Minimum confidence threshold
    pub min_confidence: f32,
    /// Enable load balancing
    pub load_balance: bool,
    /// Sacred position boost factor
    pub sacred_boost: f32,
    /// Error threshold for expert demotion
    pub error_threshold: f32,
}

impl Default for MoEConfig {
    fn default() -> Self {
        Self {
            top_k: 2,
            min_confidence: 0.3,
            load_balance: true,
            sacred_boost: 1.2,
            error_threshold: 0.1,
        }
    }
}

impl MoEConfig {
    pub fn new() -> Self { Self::default() }
    pub fn with_top_k(mut self, k: usize) -> Self { self.top_k = k; self }
}

/// MoE Gate for expert routing
pub struct MoEGate {
    config: MoEConfig,
    experts: HashMap<ExpertType, Expert>,
    selection_history: Vec<(ExpertType, f32)>,
}

impl MoEGate {
    pub fn new(config: MoEConfig) -> Self {
        let mut experts = HashMap::new();
        
        // Initialize all expert types
        for expert_type in [
            ExpertType::Geometric,
            ExpertType::ML,
            ExpertType::RAG,
            ExpertType::Heuristic,
            ExpertType::Consensus,
        ] {
            experts.insert(expert_type, Expert::new(expert_type));
        }
        
        Self {
            config,
            experts,
            selection_history: Vec::new(),
        }
    }

    /// Select top-k experts for a query
    pub fn select_experts(&mut self, query_features: &QueryFeatures) -> Vec<(ExpertType, f32)> {
        let mut scores: Vec<(ExpertType, f32)> = self.experts
            .iter()
            .map(|(expert_type, expert)| {
                let mut score = expert.effective_score();
                
                // Apply query-specific boosts
                score *= self.query_boost(*expert_type, query_features);
                
                // Sacred position boost
                if query_features.is_sacred_position {
                    if *expert_type == ExpertType::Geometric {
                        score *= self.config.sacred_boost;
                    }
                }
                
                // Load balancing penalty for overused experts
                if self.config.load_balance {
                    let usage_ratio = expert.invocation_count as f32 / 
                        (self.total_invocations() as f32 + 1.0);
                    if usage_ratio > 0.5 {
                        score *= 0.8; // Penalize overused experts
                    }
                }
                
                (*expert_type, score)
            })
            .filter(|(_, score)| *score >= self.config.min_confidence)
            .collect();

        // Sort by score descending
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take top-k
        let selected: Vec<(ExpertType, f32)> = scores.into_iter()
            .take(self.config.top_k)
            .collect();

        // Record selection
        for (expert_type, score) in &selected {
            self.selection_history.push((*expert_type, *score));
        }

        selected
    }

    /// Query-specific boost based on features
    fn query_boost(&self, expert_type: ExpertType, features: &QueryFeatures) -> f32 {
        match expert_type {
            ExpertType::Geometric => {
                if features.has_numeric_pattern { 1.3 }
                else if features.is_sacred_position { 1.5 }
                else { 1.0 }
            }
            ExpertType::ML => {
                if features.requires_inference { 1.4 }
                else { 1.0 }
            }
            ExpertType::RAG => {
                if features.requires_knowledge { 1.5 }
                else { 0.8 }
            }
            ExpertType::Heuristic => {
                if features.is_simple_query { 1.3 }
                else { 0.7 }
            }
            ExpertType::Consensus => {
                if features.requires_high_confidence { 1.4 }
                else { 0.9 }
            }
        }
    }

    /// Record expert result
    pub fn record_result(
        &mut self,
        expert_type: ExpertType,
        confidence: f32,
        latency_ms: f32,
        success: bool,
    ) {
        if let Some(expert) = self.experts.get_mut(&expert_type) {
            expert.record_invocation(confidence, latency_ms, success);
        }
    }

    /// Update expert weight manually
    pub fn set_weight(&mut self, expert_type: ExpertType, weight: f32) {
        if let Some(expert) = self.experts.get_mut(&expert_type) {
            expert.weight = weight.clamp(0.0, 2.0);
        }
    }

    /// Get expert statistics
    pub fn get_expert(&self, expert_type: ExpertType) -> Option<&Expert> {
        self.experts.get(&expert_type)
    }

    /// Get all expert statistics
    pub fn all_experts(&self) -> Vec<&Expert> {
        self.experts.values().collect()
    }

    /// Total invocations across all experts
    fn total_invocations(&self) -> u64 {
        self.experts.values().map(|e| e.invocation_count).sum()
    }

    /// Detect bottleneck expert (high error rate or latency)
    pub fn detect_bottleneck(&self) -> Option<ExpertType> {
        for (expert_type, expert) in &self.experts {
            if expert.error_rate > self.config.error_threshold {
                return Some(*expert_type);
            }
            if expert.latency_ms > 200.0 && expert.invocation_count > 10 {
                return Some(*expert_type);
            }
        }
        None
    }

    /// Get routing statistics
    pub fn stats(&self) -> MoEStats {
        let total = self.total_invocations();
        let mut expert_usage: HashMap<ExpertType, f32> = HashMap::new();
        
        for (expert_type, expert) in &self.experts {
            let usage = if total > 0 {
                expert.invocation_count as f32 / total as f32
            } else {
                0.0
            };
            expert_usage.insert(*expert_type, usage);
        }

        let avg_confidence = self.experts.values()
            .map(|e| e.confidence)
            .sum::<f32>() / self.experts.len() as f32;

        MoEStats {
            total_invocations: total,
            expert_usage,
            avg_confidence,
            bottleneck: self.detect_bottleneck(),
        }
    }
}

/// Query features for expert routing
#[derive(Debug, Clone, Default)]
pub struct QueryFeatures {
    pub has_numeric_pattern: bool,
    pub is_sacred_position: bool,
    pub requires_inference: bool,
    pub requires_knowledge: bool,
    pub is_simple_query: bool,
    pub requires_high_confidence: bool,
}

impl QueryFeatures {
    pub fn new() -> Self { Self::default() }
    
    pub fn with_sacred(mut self) -> Self {
        self.is_sacred_position = true;
        self
    }
    
    pub fn with_knowledge(mut self) -> Self {
        self.requires_knowledge = true;
        self
    }
    
    pub fn with_inference(mut self) -> Self {
        self.requires_inference = true;
        self
    }
}

/// MoE statistics
#[derive(Debug, Clone)]
pub struct MoEStats {
    pub total_invocations: u64,
    pub expert_usage: HashMap<ExpertType, f32>,
    pub avg_confidence: f32,
    pub bottleneck: Option<ExpertType>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_moe_gate() {
        let config = MoEConfig::new().with_top_k(2);
        let mut gate = MoEGate::new(config);

        let features = QueryFeatures::new().with_knowledge();
        let selected = gate.select_experts(&features);

        assert!(!selected.is_empty());
        assert!(selected.len() <= 2);
    }

    #[test]
    fn test_expert_scoring() {
        let mut expert = Expert::new(ExpertType::ML);
        expert.confidence = 0.9;
        expert.latency_ms = 10.0;
        expert.error_rate = 0.01;

        let score = expert.effective_score();
        assert!(score > 0.5);
    }

    #[test]
    fn test_record_result() {
        let mut gate = MoEGate::new(MoEConfig::default());

        gate.record_result(ExpertType::Geometric, 0.95, 5.0, true);
        gate.record_result(ExpertType::Geometric, 0.90, 6.0, true);

        let expert = gate.get_expert(ExpertType::Geometric).unwrap();
        assert_eq!(expert.invocation_count, 2);
        assert!(expert.confidence > 0.5);
    }

    #[test]
    fn test_sacred_boost() {
        let mut gate = MoEGate::new(MoEConfig::default());

        // Without sacred position
        let features_normal = QueryFeatures::new();
        let selected_normal = gate.select_experts(&features_normal);

        // With sacred position
        let features_sacred = QueryFeatures::new().with_sacred();
        let selected_sacred = gate.select_experts(&features_sacred);

        // Geometric should be boosted for sacred queries
        let geo_score_normal = selected_normal.iter()
            .find(|(t, _)| *t == ExpertType::Geometric)
            .map(|(_, s)| *s)
            .unwrap_or(0.0);

        let geo_score_sacred = selected_sacred.iter()
            .find(|(t, _)| *t == ExpertType::Geometric)
            .map(|(_, s)| *s)
            .unwrap_or(0.0);

        assert!(geo_score_sacred >= geo_score_normal);
    }

    #[test]
    fn test_bottleneck_detection() {
        let mut gate = MoEGate::new(MoEConfig::default());

        // Simulate high error rate
        for _ in 0..20 {
            gate.record_result(ExpertType::ML, 0.3, 50.0, false);
        }

        let bottleneck = gate.detect_bottleneck();
        assert!(bottleneck.is_some());
        assert_eq!(bottleneck.unwrap(), ExpertType::ML);
    }
}
