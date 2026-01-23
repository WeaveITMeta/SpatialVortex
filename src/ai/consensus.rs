//! AI Model Consensus System
//!
//! Aggregates responses from multiple AI models to reach consensus
//! Supports voting, weighted averaging, and confidence-based selection

use crate::error::{Result, SpatialVortexError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// AI model provider
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AIProvider {
    OpenAI,
    Anthropic,
    XAI,         // Grok
    Google,      // Gemini
    Meta,        // Llama
    Mistral,
    Ollama,      // Local Ollama models
}

/// Response from a single AI model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelResponse {
    pub provider: AIProvider,
    pub model_name: String,
    pub response_text: String,
    pub confidence: f64,
    pub latency_ms: u64,
    pub tokens_used: u32,
}

/// Consensus strategy for aggregating responses
#[derive(Debug, Clone, PartialEq)]
pub enum ConsensusStrategy {
    /// Simple majority voting
    MajorityVote,
    
    /// Weighted by confidence scores
    WeightedConfidence,
    
    /// Best single response (highest confidence)
    BestResponse,
    
    /// Average/merge all responses
    Ensemble,
    
    /// Custom weighted voting
    CustomWeights(HashMap<AIProvider, f64>),
}

/// Consensus result from multiple models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusResult {
    pub final_response: String,
    pub confidence: f64,
    pub model_responses: Vec<ModelResponse>,
    pub strategy_used: String,
    pub agreement_score: f64,  // 0.0-1.0, how much models agreed
    pub voting_breakdown: HashMap<String, usize>,
}

/// AI Consensus Engine
pub struct AIConsensusEngine {
    strategy: ConsensusStrategy,
    min_models: usize,
    #[allow(dead_code)]  // Reserved for model management
    models: Vec<String>,
    #[allow(dead_code)]  // Reserved for weighted consensus
    weights: Vec<f32>,
    #[allow(dead_code)]  // Reserved for timeout enforcement
    timeout_seconds: u64,
}

impl Default for AIConsensusEngine {
    fn default() -> Self {
        Self::new(ConsensusStrategy::WeightedConfidence, 3, 30)
    }
}

impl AIConsensusEngine {
    /// Create new consensus engine
    pub fn new(strategy: ConsensusStrategy, min_models: usize, timeout_seconds: u64) -> Self {
        Self {
            strategy,
            min_models,
            models: vec![],
            weights: vec![],
            timeout_seconds,
        }
    }
    
    /// Reach consensus from multiple model responses
    pub fn reach_consensus(&self, responses: Vec<ModelResponse>) -> Result<ConsensusResult> {
        if responses.len() < self.min_models {
            return Err(SpatialVortexError::InvalidInput(format!(
                "Need at least {} model responses, got {}",
                self.min_models,
                responses.len()
            )));
        }
        
        match &self.strategy {
            ConsensusStrategy::MajorityVote => self.majority_vote(responses),
            ConsensusStrategy::WeightedConfidence => self.weighted_confidence(responses),
            ConsensusStrategy::BestResponse => self.best_response(responses),
            ConsensusStrategy::Ensemble => self.ensemble(responses),
            ConsensusStrategy::CustomWeights(weights) => self.custom_weights(responses, weights),
        }
    }
    
    /// Simple majority voting
    fn majority_vote(&self, responses: Vec<ModelResponse>) -> Result<ConsensusResult> {
        let mut vote_counts: HashMap<String, usize> = HashMap::new();
        
        // Count votes for each unique response
        for response in &responses {
            let normalized = self.normalize_response(&response.response_text);
            *vote_counts.entry(normalized).or_insert(0) += 1;
        }
        
        // Find response with most votes
        let (winner, votes) = vote_counts
            .iter()
            .max_by_key(|(_, &count)| count)
            .ok_or_else(|| SpatialVortexError::InvalidInput("No votes".to_string()))?;
        
        let agreement_score = *votes as f64 / responses.len() as f64;
        
        Ok(ConsensusResult {
            final_response: winner.clone(),
            confidence: agreement_score,
            model_responses: responses,
            strategy_used: "MajorityVote".to_string(),
            agreement_score,
            voting_breakdown: vote_counts,
        })
    }
    
    /// Weighted by confidence scores
    fn weighted_confidence(&self, responses: Vec<ModelResponse>) -> Result<ConsensusResult> {
        let mut weighted_responses: HashMap<String, f64> = HashMap::new();
        let mut total_confidence = 0.0;
        
        // Weight each response by its confidence
        for response in &responses {
            let normalized = self.normalize_response(&response.response_text);
            *weighted_responses.entry(normalized).or_insert(0.0) += response.confidence;
            total_confidence += response.confidence;
        }
        
        // Find highest weighted response
        let (winner, weight) = weighted_responses
            .iter()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .ok_or_else(|| SpatialVortexError::InvalidInput("No responses".to_string()))?;
        
        let confidence = weight / total_confidence;
        let agreement_score = self.calculate_agreement(&responses);
        
        Ok(ConsensusResult {
            final_response: winner.clone(),
            confidence,
            model_responses: responses,
            strategy_used: "WeightedConfidence".to_string(),
            agreement_score,
            voting_breakdown: weighted_responses
                .into_iter()
                .map(|(k, v)| (k, v as usize))
                .collect(),
        })
    }
    
    /// Best single response (highest confidence)
    fn best_response(&self, responses: Vec<ModelResponse>) -> Result<ConsensusResult> {
        let best = responses
            .iter()
            .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap())
            .ok_or_else(|| SpatialVortexError::InvalidInput("No responses".to_string()))?;
        
        let agreement_score = self.calculate_agreement(&responses);
        
        Ok(ConsensusResult {
            final_response: best.response_text.clone(),
            confidence: best.confidence,
            model_responses: responses,
            strategy_used: "BestResponse".to_string(),
            agreement_score,
            voting_breakdown: HashMap::new(),
        })
    }
    
    /// Ensemble: merge all responses
    fn ensemble(&self, responses: Vec<ModelResponse>) -> Result<ConsensusResult> {
        // Combine all responses with separator
        let combined = responses
            .iter()
            .map(|r| format!("[{:?}] {}", r.provider, r.response_text))
            .collect::<Vec<_>>()
            .join("\n---\n");
        
        let avg_confidence = responses.iter().map(|r| r.confidence).sum::<f64>() / responses.len() as f64;
        let agreement_score = self.calculate_agreement(&responses);
        
        Ok(ConsensusResult {
            final_response: combined,
            confidence: avg_confidence,
            model_responses: responses,
            strategy_used: "Ensemble".to_string(),
            agreement_score,
            voting_breakdown: HashMap::new(),
        })
    }
    
    /// Custom weighted voting
    fn custom_weights(
        &self,
        responses: Vec<ModelResponse>,
        weights: &HashMap<AIProvider, f64>,
    ) -> Result<ConsensusResult> {
        let mut weighted_responses: HashMap<String, f64> = HashMap::new();
        let mut total_weight = 0.0;
        
        for response in &responses {
            let weight = weights.get(&response.provider).copied().unwrap_or(1.0);
            let normalized = self.normalize_response(&response.response_text);
            *weighted_responses.entry(normalized).or_insert(0.0) += weight * response.confidence;
            total_weight += weight;
        }
        
        let (winner, weight) = weighted_responses
            .iter()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .ok_or_else(|| SpatialVortexError::InvalidInput("No responses".to_string()))?;
        
        let confidence = weight / total_weight;
        let agreement_score = self.calculate_agreement(&responses);
        
        Ok(ConsensusResult {
            final_response: winner.clone(),
            confidence,
            model_responses: responses,
            strategy_used: "CustomWeights".to_string(),
            agreement_score,
            voting_breakdown: weighted_responses
                .into_iter()
                .map(|(k, v)| (k, v as usize))
                .collect(),
        })
    }
    
    /// Normalize response text for comparison
    fn normalize_response(&self, text: &str) -> String {
        text.trim()
            .to_lowercase()
            .lines()
            .map(|l| l.trim())
            .collect::<Vec<_>>()
            .join(" ")
    }
    
    /// Calculate agreement score between responses
    fn calculate_agreement(&self, responses: &[ModelResponse]) -> f64 {
        if responses.len() < 2 {
            return 1.0;
        }
        
        let mut similarity_sum = 0.0;
        let mut comparisons = 0;
        
        for i in 0..responses.len() {
            for j in (i + 1)..responses.len() {
                let sim = self.text_similarity(
                    &responses[i].response_text,
                    &responses[j].response_text,
                );
                similarity_sum += sim;
                comparisons += 1;
            }
        }
        
        if comparisons == 0 {
            return 1.0;
        }
        
        similarity_sum / comparisons as f64
    }
    
    /// Calculate text similarity (simple Jaccard similarity on words)
    fn text_similarity(&self, text1: &str, text2: &str) -> f64 {
        let words1: std::collections::HashSet<_> = text1
            .split_whitespace()
            .map(|w| w.to_lowercase())
            .collect();
        
        let words2: std::collections::HashSet<_> = text2
            .split_whitespace()
            .map(|w| w.to_lowercase())
            .collect();
        
        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();
        
        if union == 0 {
            return 0.0;
        }
        
        intersection as f64 / union as f64
    }
}

/// Configuration for Ollama queries
#[derive(Debug, Clone)]
pub struct OllamaConfig {
    pub url: String,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: usize,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:11434".to_string(),
            model: "mixtral:8x7b".to_string(),  // Using your installed Mixtral (26GB, high quality)
            temperature: 0.7,
            max_tokens: 2000,
        }
    }
}

/// Query a local Ollama model
pub async fn query_ollama(
    prompt: &str,
    config: Option<OllamaConfig>,
) -> Result<ModelResponse> {
    use serde::{Deserialize, Serialize};
    
    let config = config.unwrap_or_default();
    
    #[derive(Serialize)]
    struct OllamaRequest {
        model: String,
        prompt: String,
        stream: bool,
        options: OllamaOptions,
    }
    
    #[derive(Serialize)]
    struct OllamaOptions {
        temperature: f32,
        num_predict: usize,
    }
    
    #[derive(Deserialize)]
    struct OllamaResponse {
        response: String,
        #[allow(dead_code)]
        done: bool,
        #[allow(dead_code)]
        total_duration: Option<u64>,
    }
    
    let start = std::time::Instant::now();
    
    let request = OllamaRequest {
        model: config.model.clone(),
        prompt: prompt.to_string(),
        stream: false,
        options: OllamaOptions {
            temperature: config.temperature,
            num_predict: config.max_tokens,
        },
    };
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| SpatialVortexError::InvalidInput(format!("Failed to create HTTP client: {}", e)))?;
    
    let response = client
        .post(format!("{}/api/generate", config.url))
        .json(&request)
        .send()
        .await
        .map_err(|e| SpatialVortexError::InvalidInput(format!("Ollama request failed: {}", e)))?;
    
    if !response.status().is_success() {
        return Err(SpatialVortexError::InvalidInput(format!(
            "Ollama returned error: {}",
            response.status()
        )));
    }
    
    let result: OllamaResponse = response
        .json()
        .await
        .map_err(|e| SpatialVortexError::InvalidInput(format!("Failed to parse Ollama response: {}", e)))?;
    
    let latency = start.elapsed().as_millis() as u64;
    let tokens = result.response.split_whitespace().count() as u32;
    
    // Calculate confidence based on response quality (simple heuristic)
    let confidence = if result.response.len() > 50 && !result.response.contains("error") {
        0.85
    } else if result.response.len() > 20 {
        0.70
    } else {
        0.50
    };
    
    Ok(ModelResponse {
        provider: AIProvider::Ollama,
        model_name: config.model,
        response_text: result.response,
        confidence,
        latency_ms: latency,
        tokens_used: tokens,
    })
}

/// Query multiple Ollama models in parallel for consensus
/// 
/// # Example
/// ```no_run
/// let models = vec![
///     "llama3.2:latest",
///     "mixtral:8x7b", 
///     "codellama:13b",
/// ];
/// let responses = query_multiple_ollama("What is AGI?", models, None).await?;
/// let consensus = engine.reach_consensus(responses)?;
/// ```
pub async fn query_multiple_ollama(
    prompt: &str,
    models: Vec<&str>,
    base_config: Option<OllamaConfig>,
) -> Result<Vec<ModelResponse>> {
    use futures::future::join_all;
    
    let base_config = base_config.unwrap_or_default();
    
    let tasks: Vec<_> = models
        .into_iter()
        .map(|model| {
            let prompt = prompt.to_string();
            let config = OllamaConfig {
                url: base_config.url.clone(),
                model: model.to_string(),
                temperature: base_config.temperature,
                max_tokens: base_config.max_tokens,
            };
            
            async move {
                match query_ollama(&prompt, Some(config)).await {
                    Ok(response) => Some(response),
                    Err(e) => {
                        eprintln!("Failed to query model {}: {}", model, e);
                        None
                    }
                }
            }
        })
        .collect();
    
    let results = join_all(tasks).await;
    
    // Filter out failed queries
    let successful_responses: Vec<ModelResponse> = results
        .into_iter()
        .filter_map(|r| r)
        .collect();
    
    if successful_responses.is_empty() {
        return Err(SpatialVortexError::InvalidInput(
            "All Ollama model queries failed".to_string()
        ));
    }
    
    Ok(successful_responses)
}

/// Helper to call multiple AI models in parallel
pub async fn call_multiple_models(
    prompt: &str,
    providers: Vec<AIProvider>,
) -> Vec<ModelResponse> {
    use futures::future::join_all;
    
    let tasks: Vec<_> = providers
        .into_iter()
        .map(|provider| {
            let prompt = prompt.to_string();
            async move {
                match provider {
                    AIProvider::Ollama => {
                        // Try to query Ollama
                        match query_ollama(&prompt, None).await {
                            Ok(response) => response,
                            Err(e) => {
                                eprintln!("Ollama query failed: {}", e);
                                // Return mock response on failure
                                ModelResponse {
                                    provider: AIProvider::Ollama,
                                    model_name: "ollama-unavailable".to_string(),
                                    response_text: format!("Ollama unavailable: {}", e),
                                    confidence: 0.0,
                                    latency_ms: 0,
                                    tokens_used: 0,
                                }
                            }
                        }
                    }
                    _ => {
                        // TODO: Implement other providers
                        // For now, return mock responses
                        ModelResponse {
                            provider: provider.clone(),
                            model_name: format!("{:?}-model", provider),
                            response_text: format!("Response from {:?}", provider),
                            confidence: 0.85,
                            latency_ms: 500,
                            tokens_used: 100,
                        }
                    }
                }
            }
        })
        .collect();
    
    join_all(tasks).await
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_responses() -> Vec<ModelResponse> {
        vec![
            ModelResponse {
                provider: AIProvider::OpenAI,
                model_name: "gpt-4".to_string(),
                response_text: "Answer A".to_string(),
                confidence: 0.9,
                latency_ms: 500,
                tokens_used: 100,
            },
            ModelResponse {
                provider: AIProvider::Anthropic,
                model_name: "claude-3".to_string(),
                response_text: "Answer A".to_string(),
                confidence: 0.85,
                latency_ms: 600,
                tokens_used: 120,
            },
            ModelResponse {
                provider: AIProvider::XAI,
                model_name: "grok-2".to_string(),
                response_text: "Answer B".to_string(),
                confidence: 0.8,
                latency_ms: 400,
                tokens_used: 90,
            },
        ]
    }
    
    #[test]
    fn test_majority_vote() {
        let engine = AIConsensusEngine::new(ConsensusStrategy::MajorityVote, 2, 30);
        let responses = create_test_responses();
        
        let result = engine.reach_consensus(responses).unwrap();
        
        // Final response is normalized (lowercase)
        assert!(result.final_response.contains("answer a"));
        assert!(result.agreement_score > 0.5);
    }
    
    #[test]
    fn test_weighted_confidence() {
        let engine = AIConsensusEngine::new(ConsensusStrategy::WeightedConfidence, 2, 30);
        let responses = create_test_responses();
        
        let result = engine.reach_consensus(responses).unwrap();
        
        assert!(result.confidence > 0.0);
        assert_eq!(result.strategy_used, "WeightedConfidence");
    }
    
    #[test]
    fn test_best_response() {
        let engine = AIConsensusEngine::new(ConsensusStrategy::BestResponse, 2, 30);
        let responses = create_test_responses();
        
        let result = engine.reach_consensus(responses).unwrap();
        
        // Should pick OpenAI with highest confidence (0.9)
        assert!(result.final_response.contains("Answer A"));
        assert_eq!(result.confidence, 0.9);
    }
    
    #[test]
    fn test_min_models() {
        let engine = AIConsensusEngine::new(ConsensusStrategy::MajorityVote, 5, 30);
        let responses = create_test_responses();  // Only 3 responses
        
        let result = engine.reach_consensus(responses);
        
        assert!(result.is_err());
    }
    
    #[test]
    fn test_text_similarity() {
        let engine = AIConsensusEngine::default();
        
        let sim1 = engine.text_similarity("hello world", "hello world");
        assert_eq!(sim1, 1.0);
        
        let sim2 = engine.text_similarity("hello world", "goodbye world");
        assert!(sim2 > 0.0 && sim2 < 1.0);
        
        let sim3 = engine.text_similarity("hello world", "foo bar");
        assert_eq!(sim3, 0.0);
    }
}
