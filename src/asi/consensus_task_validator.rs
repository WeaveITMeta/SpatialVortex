//! Consensus Task Validator
//!
//! Validates AI-generated tasks using multiple LLMs to ensure quality and prevent
//! holes in training data. Uses consensus engine to verify tasks are realistic,
//! well-formed, and suitable for training.

use std::sync::Arc;
use serde::{Deserialize, Serialize};

use crate::ai::consensus::{AIConsensusEngine, ModelResponse, AIProvider, ConsensusResult};
use crate::asi::task_pattern_tracker::TaskCategory;
use crate::error::Result;

/// Task validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskValidationResult {
    /// Is task valid for training
    pub is_valid: bool,
    
    /// Consensus score (0-1)
    pub consensus_score: f32,
    
    /// Number of models that agreed
    pub models_agreed: usize,
    
    /// Total models consulted
    pub total_models: usize,
    
    /// Validation feedback from models
    pub feedback: Vec<String>,
    
    /// Quality scores by dimension
    pub quality_scores: QualityScores,
}

/// Quality scores for different dimensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityScores {
    /// How realistic is the task (0-1)
    pub realism: f32,
    
    /// How specific/detailed is the task (0-1)
    pub specificity: f32,
    
    /// How diverse from previous tasks (0-1)
    pub diversity: f32,
    
    /// How appropriate is difficulty (0-1)
    pub difficulty_appropriate: f32,
    
    /// Overall quality (average of above)
    pub overall: f32,
}

/// Consensus Task Validator
pub struct ConsensusTaskValidator {
    /// Consensus engine
    consensus_engine: Arc<AIConsensusEngine>,
    
    /// External LLM endpoint
    external_llm_endpoint: String,
    
    /// Models to use for validation
    validation_models: Vec<String>,
    
    /// Minimum consensus score to accept
    min_consensus_score: f32,
}

impl ConsensusTaskValidator {
    /// Create new consensus task validator
    pub fn new(
        consensus_engine: Arc<AIConsensusEngine>,
        external_llm_endpoint: String,
        validation_models: Vec<String>,
        min_consensus_score: f32,
    ) -> Self {
        Self {
            consensus_engine,
            external_llm_endpoint,
            validation_models,
            min_consensus_score,
        }
    }
    
    /// Validate a task using consensus from multiple LLMs
    pub async fn validate_task(
        &self,
        category: &TaskCategory,
        description: &str,
        requirements: &[String],
        difficulty: u8,
    ) -> Result<TaskValidationResult> {
        // Build validation prompt
        let prompt = self.build_validation_prompt(category, description, requirements, difficulty);
        
        // Get responses from multiple models
        let mut responses = Vec::new();
        
        for model in &self.validation_models {
            match self.get_validation_response(&prompt, model).await {
                Ok(response) => responses.push(response),
                Err(e) => {
                    tracing::warn!("Failed to get validation from {}: {}", model, e);
                }
            }
        }
        
        if responses.is_empty() {
            return Err(crate::error::SpatialVortexError::AIIntegration(
                "No validation responses received".to_string()
            ));
        }
        
        // Reach consensus
        let consensus = self.consensus_engine.reach_consensus(responses.clone())?;
        
        // Parse validation results
        let validation_result = self.parse_validation_consensus(&consensus, responses.len())?;
        
        Ok(validation_result)
    }
    
    /// Build validation prompt for LLMs
    fn build_validation_prompt(
        &self,
        category: &TaskCategory,
        description: &str,
        requirements: &[String],
        difficulty: u8,
    ) -> String {
        format!(
            r#"Validate this AI training task for quality and realism.

Task Category: {}
Description: {}
Requirements:
{}
Difficulty: {}/10

Evaluate the task on these dimensions (score each 0-10):
1. REALISM: Is this a realistic task that would occur in practice?
2. SPECIFICITY: Is the task specific and detailed enough?
3. DIFFICULTY_MATCH: Does the difficulty rating match the task complexity?
4. TRAINING_VALUE: Would this task be valuable for AI training?

Respond in this exact format:
REALISM: [score 0-10]
SPECIFICITY: [score 0-10]
DIFFICULTY_MATCH: [score 0-10]
TRAINING_VALUE: [score 0-10]
VALID: [YES or NO]
FEEDBACK: [brief explanation]

Be critical - only approve high-quality tasks suitable for training."#,
            category.as_str(),
            description,
            requirements.iter()
                .map(|r| format!("  - {}", r))
                .collect::<Vec<_>>()
                .join("\n"),
            difficulty
        )
    }
    
    /// Get validation response from a model
    async fn get_validation_response(&self, prompt: &str, model: &str) -> Result<ModelResponse> {
        let client = reqwest::Client::new();
        
        let request_body = serde_json::json!({
            "model": model,
            "prompt": prompt,
            "stream": false,
        });
        
        let start = std::time::Instant::now();
        
        let response = client
            .post(format!("{}/api/generate", self.external_llm_endpoint))
            .json(&request_body)
            .send()
            .await
            .map_err(|e| crate::error::SpatialVortexError::AIIntegration(e.to_string()))?;
        
        let latency_ms = start.elapsed().as_millis() as u64;
        
        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| crate::error::SpatialVortexError::AIIntegration(e.to_string()))?;
        
        let response_text = response_json["response"].as_str()
            .ok_or_else(|| crate::error::SpatialVortexError::AIIntegration(
                "No response field".to_string()
            ))?
            .to_string();
        
        // Parse confidence from response
        let confidence = self.extract_confidence(&response_text);
        
        Ok(ModelResponse {
            provider: AIProvider::Ollama,
            model_name: model.to_string(),
            response_text,
            confidence,
            latency_ms,
            tokens_used: 0, // Not provided by Ollama
        })
    }
    
    /// Extract confidence from validation response
    fn extract_confidence(&self, response: &str) -> f64 {
        // Look for VALID: YES/NO
        if response.contains("VALID: YES") || response.contains("VALID:YES") {
            0.8
        } else if response.contains("VALID: NO") || response.contains("VALID:NO") {
            0.2
        } else {
            0.5
        }
    }
    
    /// Parse consensus result into validation result
    fn parse_validation_consensus(
        &self,
        consensus: &ConsensusResult,
        total_models: usize,
    ) -> Result<TaskValidationResult> {
        let response = &consensus.final_response;
        
        // Extract scores
        let realism = self.extract_score(response, "REALISM") / 10.0;
        let specificity = self.extract_score(response, "SPECIFICITY") / 10.0;
        let difficulty_match = self.extract_score(response, "DIFFICULTY_MATCH") / 10.0;
        let training_value = self.extract_score(response, "TRAINING_VALUE") / 10.0;
        
        let overall = (realism + specificity + difficulty_match + training_value) / 4.0;
        
        // Extract validity
        let is_valid = response.contains("VALID: YES") || response.contains("VALID:YES");
        
        // Extract feedback
        let feedback = self.extract_feedback(response);
        
        // Calculate models agreed (based on consensus agreement score)
        let models_agreed = (consensus.agreement_score * total_models as f64) as usize;
        
        Ok(TaskValidationResult {
            is_valid: is_valid && consensus.confidence >= self.min_consensus_score as f64,
            consensus_score: consensus.confidence as f32,
            models_agreed,
            total_models,
            feedback: vec![feedback],
            quality_scores: QualityScores {
                realism,
                specificity,
                diversity: 0.8, // TODO: Calculate based on task history
                difficulty_appropriate: difficulty_match,
                overall,
            },
        })
    }
    
    /// Extract score from response
    fn extract_score(&self, response: &str, field: &str) -> f32 {
        let pattern = format!("{}: ", field);
        
        if let Some(start) = response.find(&pattern) {
            let after = &response[start + pattern.len()..];
            if let Some(end) = after.find('\n') {
                let score_str = &after[..end].trim();
                if let Ok(score) = score_str.parse::<f32>() {
                    return score.clamp(0.0, 10.0);
                }
            }
        }
        
        5.0 // Default middle score
    }
    
    /// Extract feedback from response
    fn extract_feedback(&self, response: &str) -> String {
        if let Some(start) = response.find("FEEDBACK: ") {
            let after = &response[start + 10..];
            if let Some(end) = after.find('\n') {
                return after[..end].trim().to_string();
            }
            return after.trim().to_string();
        }
        
        "No feedback provided".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::consensus::ConsensusStrategy;
    
    #[test]
    fn test_extract_score() {
        let validator = create_test_validator();
        
        let response = "REALISM: 8\nSPECIFICITY: 7\n";
        
        assert_eq!(validator.extract_score(response, "REALISM"), 8.0);
        assert_eq!(validator.extract_score(response, "SPECIFICITY"), 7.0);
        assert_eq!(validator.extract_score(response, "MISSING"), 5.0);
    }
    
    #[test]
    fn test_extract_confidence() {
        let validator = create_test_validator();
        
        assert_eq!(validator.extract_confidence("VALID: YES"), 0.8);
        assert_eq!(validator.extract_confidence("VALID: NO"), 0.2);
        assert_eq!(validator.extract_confidence("UNCLEAR"), 0.5);
    }
    
    fn create_test_validator() -> ConsensusTaskValidator {
        let consensus_engine = Arc::new(AIConsensusEngine::new(
            ConsensusStrategy::WeightedConfidence,
            2,
            30
        ));
        
        ConsensusTaskValidator::new(
            consensus_engine,
            "http://localhost:11434".to_string(),
            vec!["llama3".to_string()],
            0.7,
        )
    }
}
