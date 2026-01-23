//! AI-Powered Task Generator
//!
//! Generates realistic, diverse training tasks using AI models instead of static templates.
//! Uses both our own ProductionEngine and external LLMs for maximum diversity.

use std::sync::Arc;
use serde::{Deserialize, Serialize};
use rand::Rng;

use crate::asi::task_pattern_tracker::{TaskCategory, ErrorType};
use crate::ml::inference::production_engine::ProductionEngine;
use crate::ai::consensus::{AIConsensusEngine, ModelResponse, AIProvider};
use crate::error::Result;

/// Task generation strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskGenerationStrategy {
    /// Use our own ProductionEngine
    OwnModel,
    
    /// Use external LLM (Ollama, OpenAI, etc.)
    ExternalLLM(String),
    
    /// Hybrid: Mix of own model and external
    Hybrid,
    
    /// Consensus validation: Generate with multiple LLMs and validate via consensus
    ConsensusValidation,
    
    /// Fallback to templates only if AI fails
    TemplateFallback,
}

/// Generated task specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedTask {
    /// Task category
    pub category: TaskCategory,
    
    /// Task description (AI-generated)
    pub description: String,
    
    /// Detailed requirements (AI-generated)
    pub requirements: Vec<String>,
    
    /// Expected difficulty (1-10, AI-estimated)
    pub difficulty: u8,
    
    /// Potential failure modes (AI-predicted)
    pub failure_modes: Vec<PredictedFailureMode>,
    
    /// Success criteria (AI-defined)
    pub success_criteria: Vec<String>,
    
    /// Test cases (AI-generated)
    pub test_cases: Vec<String>,
    
    /// Generation metadata
    pub metadata: TaskMetadata,
}

/// Predicted failure mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictedFailureMode {
    pub error_type: ErrorType,
    pub probability: f32,
    pub description: String,
    pub mitigation: String,
}

/// Task generation metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMetadata {
    /// Which strategy generated this task
    pub strategy: TaskGenerationStrategy,
    
    /// Generation timestamp
    pub generated_at: u64,
    
    /// Complexity score (0-1)
    pub complexity: f32,
    
    /// Diversity score (how unique vs other tasks)
    pub diversity: f32,
    
    /// Consensus validation score (0-1, if validated)
    pub consensus_score: Option<f32>,
    
    /// Number of models that agreed (if consensus used)
    pub models_agreed: Option<usize>,
    
    /// Quality validated by consensus
    pub quality_validated: bool,
}

/// AI Task Generator
pub struct AITaskGenerator {
    /// Our production engine (optional)
    production_engine: Option<Arc<tokio::sync::RwLock<ProductionEngine>>>,
    
    /// External LLM endpoint (optional)
    external_llm_endpoint: Option<String>,
    
    /// Consensus engine for validation (optional)
    consensus_engine: Option<Arc<AIConsensusEngine>>,
    
    /// External LLM models for consensus
    consensus_models: Vec<String>,
    
    /// Generation strategy
    strategy: TaskGenerationStrategy,
    
    /// Previously generated tasks (for diversity)
    task_history: Arc<tokio::sync::RwLock<Vec<GeneratedTask>>>,
    
    /// Max history size
    max_history: usize,
    
    /// Minimum consensus score to accept task (0-1)
    min_consensus_score: f32,
}

impl AITaskGenerator {
    /// Create new AI task generator
    pub fn new(strategy: TaskGenerationStrategy) -> Self {
        Self {
            production_engine: None,
            external_llm_endpoint: None,
            consensus_engine: None,
            consensus_models: vec![
                "llama3".to_string(),
                "mistral".to_string(),
                "codellama".to_string(),
            ],
            strategy,
            task_history: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            max_history: 100,
            min_consensus_score: 0.7, // 70% agreement required
        }
    }
    
    /// Set production engine
    pub fn with_production_engine(mut self, engine: Arc<tokio::sync::RwLock<ProductionEngine>>) -> Self {
        self.production_engine = Some(engine);
        self
    }
    
    /// Set external LLM endpoint
    pub fn with_external_llm(mut self, endpoint: String) -> Self {
        self.external_llm_endpoint = Some(endpoint);
        self
    }
    
    /// Set consensus engine for validation
    pub fn with_consensus_engine(mut self, engine: Arc<AIConsensusEngine>) -> Self {
        self.consensus_engine = Some(engine);
        self
    }
    
    /// Set consensus models
    pub fn with_consensus_models(mut self, models: Vec<String>) -> Self {
        self.consensus_models = models;
        self
    }
    
    /// Set minimum consensus score
    pub fn with_min_consensus_score(mut self, score: f32) -> Self {
        self.min_consensus_score = score.clamp(0.0, 1.0);
        self
    }
    
    /// Generate a task using AI
    pub async fn generate_task(&self, category: &TaskCategory, iteration: usize) -> Result<GeneratedTask> {
        match &self.strategy {
            TaskGenerationStrategy::OwnModel => {
                self.generate_with_own_model(category, iteration).await
            }
            TaskGenerationStrategy::ExternalLLM(model) => {
                self.generate_with_external_llm(category, iteration, model).await
            }
            TaskGenerationStrategy::Hybrid => {
                // Alternate between own model and external
                if iteration % 2 == 0 {
                    self.generate_with_own_model(category, iteration).await
                } else if let Some(ref endpoint) = self.external_llm_endpoint {
                    self.generate_with_external_llm(category, iteration, endpoint).await
                } else {
                    self.generate_with_own_model(category, iteration).await
                }
            }
            TaskGenerationStrategy::ConsensusValidation => {
                self.generate_with_own_model(category, iteration).await
            }
            TaskGenerationStrategy::TemplateFallback => {
                // Try AI first, fall back to template
                match self.generate_with_own_model(category, iteration).await {
                    Ok(task) => Ok(task),
                    Err(_) => self.generate_from_template(category, iteration),
                }
            }
        }
    }
    
    /// Generate task using our own ProductionEngine
    async fn generate_with_own_model(&self, category: &TaskCategory, iteration: usize) -> Result<GeneratedTask> {
        let engine = self.production_engine.as_ref()
            .ok_or_else(|| crate::error::SpatialVortexError::AIIntegration(
                format!("Category not found: {:?}", category)
            ))?;
        
        // Get previous tasks for diversity
        let history = self.task_history.read().await;
        let previous_descriptions: Vec<String> = history.iter()
            .filter(|t| t.category == *category)
            .map(|t| t.description.clone())
            .collect();
        drop(history);
        
        // Build prompt for task generation
        let prompt = self.build_task_generation_prompt(category, &previous_descriptions, iteration);
        
        // Generate using ProductionEngine
        let engine_lock = engine.read().await;
        let response = engine_lock.generate(prompt.as_str(), 512);
        drop(engine_lock);
        
        // Parse response into GeneratedTask
        let task = self.parse_task_response(category, &response, iteration)?;
        
        // Add to history
        let mut history = self.task_history.write().await;
        history.push(task.clone());
        if history.len() > self.max_history {
            history.remove(0);
        }
        drop(history);
        
        Ok(task)
    }
    
    /// Generate task using external LLM
    async fn generate_with_external_llm(
        &self,
        category: &TaskCategory,
        iteration: usize,
        model: &str,
    ) -> Result<GeneratedTask> {
        // Get previous tasks for diversity
        let history = self.task_history.read().await;
        let previous_descriptions: Vec<String> = history.iter()
            .filter(|t| t.category == *category)
            .map(|t| t.description.clone())
            .collect();
        drop(history);
        
        // Build prompt
        let prompt = self.build_task_generation_prompt(category, &previous_descriptions, iteration);
        
        // Call external LLM (Ollama, OpenAI, etc.)
        let response = self.call_external_llm(&prompt, model).await?;
        
        // Parse response
        let task = self.parse_task_response(category, &response, iteration)?;
        
        // Add to history
        let mut history = self.task_history.write().await;
        history.push(task.clone());
        if history.len() > self.max_history {
            history.remove(0);
        }
        drop(history);
        
        Ok(task)
    }
    
    /// Build prompt for task generation
    fn build_task_generation_prompt(
        &self,
        category: &TaskCategory,
        previous_tasks: &[String],
        iteration: usize,
    ) -> String {
        let category_str = category.as_str();
        
        let diversity_note = if !previous_tasks.is_empty() {
            format!(
                "\n\nPrevious tasks (avoid duplicating these):\n{}",
                previous_tasks.iter()
                    .take(5)
                    .map(|t| format!("- {}", t))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        } else {
            String::new()
        };
        
        let difficulty_level = match iteration {
            0..=2 => "beginner (difficulty 3-5)",
            3..=5 => "intermediate (difficulty 5-7)",
            _ => "advanced (difficulty 7-10)",
        };
        
        format!(
            r#"Generate a realistic {} task for AI training at {} level.

Task Category: {}
Iteration: {}

Requirements:
1. Create a specific, realistic task description
2. List 3-5 detailed requirements
3. Estimate difficulty (1-10)
4. Predict 2-4 potential failure modes with probabilities
5. Define 3-5 success criteria
6. Provide 2-3 test cases

Format your response as JSON:
{{
  "description": "specific task description",
  "requirements": ["req1", "req2", "req3"],
  "difficulty": 7,
  "failure_modes": [
    {{
      "error_type": "SyntaxError",
      "probability": 0.4,
      "description": "what could go wrong",
      "mitigation": "how to prevent it"
    }}
  ],
  "success_criteria": ["criterion1", "criterion2"],
  "test_cases": ["test1", "test2"]
}}{}

Make this task realistic, specific, and challenging. Focus on real-world scenarios."#,
            category_str,
            difficulty_level,
            category_str,
            iteration,
            diversity_note
        )
    }
    
    /// Parse LLM response into GeneratedTask
    fn parse_task_response(
        &self,
        category: &TaskCategory,
        response: &str,
        iteration: usize,
    ) -> Result<GeneratedTask> {
        // Try to parse as JSON first
        if let Ok(parsed) = self.parse_json_response(response) {
            return Ok(self.create_task_from_parsed(category, parsed, iteration));
        }
        
        // Fallback: Extract from text
        self.parse_text_response(category, response, iteration)
    }
    
    /// Parse JSON response
    fn parse_json_response(&self, response: &str) -> Result<serde_json::Value> {
        // Find JSON block in response
        let json_start = response.find('{').unwrap_or(0);
        let json_end = response.rfind('}').map(|i| i + 1).unwrap_or(response.len());
        let json_str = &response[json_start..json_end];
        
        serde_json::from_str(json_str)
            .map_err(|e| crate::error::SpatialVortexError::AIIntegration(e.to_string()))
    }
    
    /// Create GeneratedTask from parsed JSON
    fn create_task_from_parsed(
        &self,
        category: &TaskCategory,
        parsed: serde_json::Value,
        iteration: usize,
    ) -> GeneratedTask {
        let description = parsed["description"].as_str()
            .unwrap_or("Generated task")
            .to_string();
        
        let requirements = parsed["requirements"].as_array()
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect())
            .unwrap_or_else(Vec::new);
        
        let difficulty = parsed["difficulty"].as_u64()
            .unwrap_or(5) as u8;
        
        let failure_modes = parsed["failure_modes"].as_array()
            .map(|arr| arr.iter()
                .filter_map(|v| self.parse_failure_mode(v))
                .collect())
            .unwrap_or_else(|| self.default_failure_modes(category));
        
        let success_criteria = parsed["success_criteria"].as_array()
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect())
            .unwrap_or_else(Vec::new);
        
        let test_cases = parsed["test_cases"].as_array()
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect())
            .unwrap_or_else(Vec::new);
        
        GeneratedTask {
            category: category.clone(),
            description,
            requirements,
            difficulty,
            failure_modes,
            success_criteria,
            test_cases,
            metadata: TaskMetadata {
                strategy: self.strategy.clone(),
                generated_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                complexity: difficulty as f32 / 10.0,
                diversity: 0.8, // TODO: Calculate based on similarity to history
                consensus_score: None,
                models_agreed: None,
                quality_validated: false,
            },
        }
    }
    
    /// Generate task with consensus validation
    fn generate_with_consensus_validation<'a>(
        &'a self,
        category: &'a TaskCategory,
        iteration: usize,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<GeneratedTask>> + Send + 'a>> {
        Box::pin(async move {
        use crate::asi::consensus_task_validator::ConsensusTaskValidator;
        
        // First, generate task using hybrid approach
        let mut task = if iteration % 2 == 0 {
            self.generate_with_own_model(category, iteration).await?
        } else if let Some(ref endpoint) = self.external_llm_endpoint {
            self.generate_with_external_llm(category, iteration, endpoint).await?
        } else {
            self.generate_with_own_model(category, iteration).await?
        };
        
        // Then validate with consensus
        if let Some(ref consensus_engine) = self.consensus_engine {
            if let Some(ref endpoint) = self.external_llm_endpoint {
                let validator = ConsensusTaskValidator::new(
                    consensus_engine.clone(),
                    endpoint.clone(),
                    self.consensus_models.clone(),
                    self.min_consensus_score,
                );
                
                let validation = validator.validate_task(
                    category,
                    &task.description,
                    &task.requirements,
                    task.difficulty,
                ).await?;
                
                // Update task metadata with validation results
                task.metadata.consensus_score = Some(validation.consensus_score);
                task.metadata.models_agreed = Some(validation.models_agreed);
                task.metadata.quality_validated = validation.is_valid;
                
                // Reject task if validation failed
                if !validation.is_valid {
                    tracing::warn!(
                        "Task rejected by consensus validation (score: {:.2}, agreed: {}/{})",
                        validation.consensus_score,
                        validation.models_agreed,
                        validation.total_models
                    );
                    
                    // Try again with different approach
                    return self.generate_with_consensus_validation(category, iteration + 1).await;
                }
                
                tracing::info!(
                    "Task validated by consensus (score: {:.2}, quality: {:.2})",
                    validation.consensus_score,
                    validation.quality_scores.overall
                );
            }
        }
        
        Ok(task)
        })
    }
    
    /// Parse failure mode from JSON
    fn parse_failure_mode(&self, value: &serde_json::Value) -> Option<PredictedFailureMode> {
        let error_type_str = value["error_type"].as_str()?;
        let error_type = ErrorType::from_message(error_type_str);
        
        Some(PredictedFailureMode {
            error_type,
            probability: value["probability"].as_f64()? as f32,
            description: value["description"].as_str()?.to_string(),
            mitigation: value["mitigation"].as_str()?.to_string(),
        })
    }
    
    /// Parse text response (fallback)
    fn parse_text_response(
        &self,
        category: &TaskCategory,
        response: &str,
        iteration: usize,
    ) -> Result<GeneratedTask> {
        // Extract description (first line or first sentence)
        let description = response.lines()
            .next()
            .unwrap_or("AI-generated task")
            .trim()
            .to_string();
        
        let difficulty = 5 + (iteration as u8 % 5);
        
        Ok(GeneratedTask {
            category: category.clone(),
            description,
            requirements: vec![
                "Complete the task successfully".to_string(),
                "Follow best practices".to_string(),
            ],
            difficulty,
            failure_modes: self.default_failure_modes(category),
            success_criteria: vec![
                "Task completed".to_string(),
                "No errors".to_string(),
            ],
            test_cases: vec![
                "Basic functionality test".to_string(),
            ],
            metadata: TaskMetadata {
                strategy: self.strategy.clone(),
                generated_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                complexity: difficulty as f32 / 10.0,
                diversity: 0.5,
                consensus_score: None,
                models_agreed: None,
                quality_validated: false,
            },
        })
    }
    
    /// Default failure modes for category
    fn default_failure_modes(&self, category: &TaskCategory) -> Vec<PredictedFailureMode> {
        match category {
            TaskCategory::CodeGeneration => vec![
                PredictedFailureMode {
                    error_type: ErrorType::SyntaxError,
                    probability: 0.4,
                    description: "Generated code has syntax errors".to_string(),
                    mitigation: "Add syntax validation step".to_string(),
                },
                PredictedFailureMode {
                    error_type: ErrorType::CompilationError,
                    probability: 0.3,
                    description: "Type mismatches or missing imports".to_string(),
                    mitigation: "Improve type inference".to_string(),
                },
            ],
            TaskCategory::BugFix => vec![
                PredictedFailureMode {
                    error_type: ErrorType::LogicError,
                    probability: 0.5,
                    description: "Fix introduces new bugs".to_string(),
                    mitigation: "Add regression tests".to_string(),
                },
            ],
            TaskCategory::Testing => vec![
                PredictedFailureMode {
                    error_type: ErrorType::TimeoutError,
                    probability: 0.3,
                    description: "Tests take too long".to_string(),
                    mitigation: "Optimize test execution".to_string(),
                },
            ],
            _ => vec![],
        }
    }
    
    /// Call external LLM (Ollama, OpenAI, etc.)
    async fn call_external_llm(&self, prompt: &str, model: &str) -> Result<String> {
        // Try Ollama first
        if let Some(ref endpoint) = self.external_llm_endpoint {
            return self.call_ollama(endpoint, model, prompt).await;
        }
        
        Err(crate::error::SpatialVortexError::AIIntegration(
            "No templates available".to_string()
        ))
    }
    
    /// Call Ollama API
    async fn call_ollama(&self, endpoint: &str, model: &str, prompt: &str) -> Result<String> {
        let client = reqwest::Client::new();
        
        let request_body = serde_json::json!({
            "model": model,
            "prompt": prompt,
            "stream": false,
        });
        
        let response = client
            .post(format!("{}/api/generate", endpoint))
            .json(&request_body)
            .send()
            .await
            .map_err(|e| crate::error::SpatialVortexError::AIIntegration(e.to_string()))?;
        
        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| crate::error::SpatialVortexError::AIIntegration(e.to_string()))?;
        
        response_json["response"].as_str()
            .ok_or_else(|| crate::error::SpatialVortexError::AIIntegration(
                "Failed to parse task from response".to_string()
            ))
            .map(String::from)
    }
    
    /// Generate from template (fallback)
    fn generate_from_template(&self, category: &TaskCategory, iteration: usize) -> Result<GeneratedTask> {
        let mut rng = rand::thread_rng();
        let difficulty = 3 + (iteration as u8 % 7);
        
        let (description, requirements, failure_modes, success_criteria, test_cases) = match category {
            TaskCategory::CodeGeneration => (
                format!("Generate a REST API endpoint for resource #{}", iteration),
                vec![
                    "Implement CRUD operations".to_string(),
                    "Add input validation".to_string(),
                    "Include error handling".to_string(),
                ],
                vec![
                    PredictedFailureMode {
                        error_type: ErrorType::SyntaxError,
                        probability: 0.4,
                        description: "Syntax errors in generated code".to_string(),
                        mitigation: "Add syntax validation".to_string(),
                    },
                ],
                vec!["Valid syntax".to_string(), "Compiles successfully".to_string()],
                vec!["Test basic CRUD".to_string()],
            ),
            TaskCategory::BugFix => (
                format!("Fix bug in parser module #{}", iteration),
                vec!["Identify root cause".to_string(), "Fix without regressions".to_string()],
                vec![
                    PredictedFailureMode {
                        error_type: ErrorType::LogicError,
                        probability: 0.5,
                        description: "Fix introduces new bug".to_string(),
                        mitigation: "Add regression tests".to_string(),
                    },
                ],
                vec!["Bug fixed".to_string(), "All tests pass".to_string()],
                vec!["Test original bug".to_string(), "Test edge cases".to_string()],
            ),
            _ => (
                format!("{} task #{}", category.as_str(), iteration),
                vec!["Complete task".to_string()],
                vec![],
                vec!["Success".to_string()],
                vec!["Basic test".to_string()],
            ),
        };
        
        Ok(GeneratedTask {
            category: category.clone(),
            description,
            requirements,
            difficulty,
            failure_modes,
            success_criteria,
            test_cases,
            metadata: TaskMetadata {
                strategy: TaskGenerationStrategy::TemplateFallback,
                generated_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                complexity: difficulty as f32 / 10.0,
                diversity: 0.3, // Templates have low diversity
                consensus_score: None,
                models_agreed: None,
                quality_validated: false,
            },
        })
    }
    
    /// Get task generation statistics
    pub async fn get_stats(&self) -> TaskGenerationStats {
        let history = self.task_history.read().await;
        
        let total = history.len();
        let by_strategy: std::collections::HashMap<String, usize> = history.iter()
            .fold(std::collections::HashMap::new(), |mut acc, task| {
                let strategy_name = format!("{:?}", task.metadata.strategy);
                *acc.entry(strategy_name).or_insert(0) += 1;
                acc
            });
        
        let avg_diversity = if total > 0 {
            history.iter().map(|t| t.metadata.diversity).sum::<f32>() / total as f32
        } else {
            0.0
        };
        
        let avg_complexity = if total > 0 {
            history.iter().map(|t| t.metadata.complexity).sum::<f32>() / total as f32
        } else {
            0.0
        };
        
        TaskGenerationStats {
            total_generated: total,
            by_strategy,
            avg_diversity,
            avg_complexity,
        }
    }
}

/// Task generation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskGenerationStats {
    pub total_generated: usize,
    pub by_strategy: std::collections::HashMap<String, usize>,
    pub avg_diversity: f32,
    pub avg_complexity: f32,
}
