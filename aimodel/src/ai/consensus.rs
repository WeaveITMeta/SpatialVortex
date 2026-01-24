//! AI Consensus Engine - Multi-LLM fusion

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct AIConsensusEngine {
    providers: Vec<String>,
    weights: HashMap<String, f32>,
    #[allow(dead_code)]
    sacred_boost: f32,
}

impl Default for AIConsensusEngine {
    fn default() -> Self {
        Self { providers: Vec::new(), weights: HashMap::new(), sacred_boost: 1.15 }
    }
}

impl AIConsensusEngine {
    pub fn new() -> Self { Self::default() }

    pub fn add_provider(&mut self, name: &str, weight: f32) {
        self.providers.push(name.to_string());
        self.weights.insert(name.to_string(), weight);
    }

    pub fn fuse(&self, responses: &[ProviderResponse]) -> ConsensusResult {
        if responses.is_empty() { return ConsensusResult::default(); }

        let mut total_weight = 0.0;
        let mut weighted_conf = 0.0;
        let mut best = (String::new(), 0.0f32);

        for r in responses {
            let w = self.weights.get(&r.provider).copied().unwrap_or(1.0);
            total_weight += w;
            weighted_conf += r.confidence * w;
            if r.confidence * w > best.1 {
                best = (r.content.clone(), r.confidence * w);
            }
        }

        ConsensusResult {
            content: best.0,
            confidence: if total_weight > 0.0 { weighted_conf / total_weight } else { 0.0 },
            provider_count: responses.len(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderResponse {
    pub provider: String,
    pub content: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConsensusResult {
    pub content: String,
    pub confidence: f32,
    pub provider_count: usize,
}
