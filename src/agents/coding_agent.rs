//! Multi-language coding agent with SpatialVortex integration

use crate::agents::error::{AgentError, Result};
use crate::agents::language::{Language, LanguageDetector};
use crate::agents::executor::{CodeExecutor, ExecutionResult};
use crate::agents::llm_bridge::{LLMBridge, LLMConfig};
use crate::agents::prompts::PromptBuilder;
use crate::agents::prompt_template::{PromptTemplate, Difficulty};
use crate::agents::symbolica_bridge::{SymbolicaMath, is_math_task, extract_math_expression};
use crate::core::sacred_geometry::flux_matrix::FluxMatrixEngine;
use crate::data::models::ELPTensor;

/// Configuration for the coding agent
#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub max_correction_attempts: usize,
    pub enable_self_correction: bool,
    pub enable_flux_routing: bool,
    pub use_enhanced_prompts: bool,  // NEW: Use PromptTemplate system
    pub memory_limit: String,
    pub cpu_limit: String,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_correction_attempts: 3,
            enable_self_correction: true,
            enable_flux_routing: true,
            use_enhanced_prompts: true,  // NEW: Default to enhanced prompts
            memory_limit: "256m".to_string(),
            cpu_limit: "0.5".to_string(),
        }
    }
}

/// Multi-language coding agent
pub struct CodingAgent {
    config: AgentConfig,
    detector: LanguageDetector,
    executor: CodeExecutor,
    flux_engine: FluxMatrixEngine,
    llm_bridge: Option<LLMBridge>,
    prompt_builder: PromptBuilder,
    symbolica: SymbolicaMath,
}

impl CodingAgent {
    /// Create new coding agent with default configuration
    pub fn new() -> Self {
        Self::with_config(AgentConfig::default())
    }
    
    /// Create new coding agent with custom configuration
    pub fn with_config(config: AgentConfig) -> Self {
        Self {
            config,
            detector: LanguageDetector::new(),
            executor: CodeExecutor::new(),
            flux_engine: FluxMatrixEngine::new(),
            llm_bridge: None,
            prompt_builder: PromptBuilder::new(),
            symbolica: SymbolicaMath::default(),
        }
    }
    
    /// Create agent with LLM backend
    pub fn with_llm(config: AgentConfig, llm_config: LLMConfig) -> Result<Self> {
        let llm_bridge = LLMBridge::new(llm_config)?;
        Ok(Self {
            config,
            detector: LanguageDetector::new(),
            executor: CodeExecutor::new(),
            flux_engine: FluxMatrixEngine::new(),
            llm_bridge: Some(llm_bridge),
            prompt_builder: PromptBuilder::new(),
            symbolica: SymbolicaMath::default(),
        })
    }
    
    /// Execute a coding task (async version)
    pub async fn execute_task(&self, task: &str) -> Result<TaskResult> {
        // 1. Check if this is a math task that can be solved symbolically
        if is_math_task(task) {
            if let Ok(result) = self.handle_math_task(task).await {
                return Ok(result);
            }
            // If symbolic solving fails, fall through to LLM generation
        }
        
        // 2. Detect language
        let language = self.detector.detect(task)?;
        
        // 3. Route based on flux position (if enabled)
        let flux_position = if self.config.enable_flux_routing {
            self.determine_flux_position(task)
        } else {
            None
        };
        
        // 4. Generate code via LLM
        let code = self.generate_code(task, language, flux_position).await?;
        
        // 5. Execute with self-correction loop
        if self.config.enable_self_correction {
            self.execute_with_correction(&code, language, task).await
        } else {
            let result = self.executor.execute(&code, language)?;
            Ok(TaskResult {
                code,
                execution: Some(result),
                language,
                flux_position,
                attempts: 1,
            })
        }
    }
    
    /// Execute code with self-correction loop
    async fn execute_with_correction(
        &self,
        initial_code: &str,
        language: Language,
        task: &str,
    ) -> Result<TaskResult> {
        let mut code = initial_code.to_string();
        let mut attempts = 0;
        
        loop {
            attempts += 1;
            
            let result = self.executor.execute(&code, language)?;
            
            if result.success {
                return Ok(TaskResult {
                    code,
                    execution: Some(result),
                    language,
                    flux_position: None,
                    attempts,
                });
            }
            
            if attempts >= self.config.max_correction_attempts {
                return Err(AgentError::MaxAttemptsExceeded);
            }
            
            // Generate corrected code via LLM
            code = self.correct_code(&code, &result.stderr, task, language).await?;
        }
    }
    
    /// Determine flux position based on task characteristics
    fn determine_flux_position(&self, task: &str) -> Option<u8> {
        let task_lower = task.to_lowercase();
        
        // Use flux engine to analyze task semantics
        // For now, use keyword matching weighted by flux analysis
        let mut scores = vec![(3, 0.0), (6, 0.0), (9, 0.0)];
        
        // Position 9 (Logos) - Pure logic/math
        if task_lower.contains("algorithm") || task_lower.contains("math") 
            || task_lower.contains("solve") || task_lower.contains("equation") {
            scores[2].1 += 1.0;
        }
        
        // Position 3 (Ethos) - Architecture/Design
        if task_lower.contains("design") || task_lower.contains("architecture")
            || task_lower.contains("pattern") || task_lower.contains("structure") {
            scores[0].1 += 1.0;
        }
        
        // Position 6 (Pathos) - UI/UX
        if task_lower.contains("ui") || task_lower.contains("ux")
            || task_lower.contains("interface") || task_lower.contains("visual") {
            scores[1].1 += 1.0;
        }
        
        // Use flux_engine to compute position affinities
        // TODO: Integrate with actual FluxMatrixEngine analysis
        let _ = &self.flux_engine; // Mark as used
        
        // Return position with highest score
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        if scores[0].1 > 0.0 {
            Some(scores[0].0)
        } else {
            None
        }
    }
    
    /// Generate code for task via LLM
    async fn generate_code(
        &self,
        task: &str,
        language: Language,
        flux_position: Option<u8>,
    ) -> Result<String> {
        // Check if LLM is available
        let llm = self.llm_bridge.as_ref()
            .ok_or_else(|| AgentError::GenerationError(
                "LLM backend not configured. Use CodingAgent::with_llm()".to_string()
            ))?;
        
        // Build prompt - use enhanced template if enabled
        let prompt = if self.config.use_enhanced_prompts {
            self.build_enhanced_prompt(task, language, flux_position)
        } else {
            self.prompt_builder.build_generation_prompt(
                task,
                language,
                flux_position,
            )
        };
        
        // Generate code via LLM
        llm.generate_code(&prompt, language).await
    }
    
    /// Build enhanced prompt using PromptTemplate system
    fn build_enhanced_prompt(
        &self,
        task: &str,
        language: Language,
        flux_position: Option<u8>,
    ) -> String {
        // Detect difficulty from task complexity
        let difficulty = self.detect_difficulty(task);
        
        // Start with basic template
        let mut template = PromptTemplate::new(task.to_string(), language)
            .with_difficulty(difficulty);
        
        // Add sacred position if routing enabled
        if let Some(pos) = flux_position {
            template = template.with_sacred_position(pos);
        }
        
        // Detect and add algorithm hints based on task keywords
        if let Some(hint) = self.detect_algorithm_hint(task) {
            template = template.with_algorithm_hint(hint);
        }
        
        // Build and return
        template.build()
    }
    
    /// Detect difficulty from task description
    fn detect_difficulty(&self, task: &str) -> Difficulty {
        let task_lower = task.to_lowercase();
        
        // Hard indicators
        if task_lower.contains("dynamic programming") 
            || task_lower.contains("edit distance")
            || task_lower.contains("regular expression")
            || task_lower.contains("optimal")
            || task_lower.contains("minimize")
            || task_lower.contains("maximize") {
            return Difficulty::Hard;
        }
        
        // Medium indicators
        if task_lower.contains("two pointers")
            || task_lower.contains("sliding window")
            || task_lower.contains("binary search")
            || task_lower.contains("graph")
            || task_lower.contains("tree") {
            return Difficulty::Medium;
        }
        
        // Default to easy
        Difficulty::Easy
    }
    
    /// Detect algorithm hint from task description
    fn detect_algorithm_hint(&self, task: &str) -> Option<String> {
        let task_lower = task.to_lowercase();
        
        // Hash table patterns
        if task_lower.contains("sum") && task_lower.contains("target") {
            return Some("Use a hash table (dictionary/map) to store seen elements. Check if complement exists for O(n) time.".to_string());
        }
        
        // Two pointers
        if task_lower.contains("triplet") || task_lower.contains("three sum") {
            return Some("Sort the array, then use two pointers approach. For each element, find pairs that sum to target.".to_string());
        }
        
        // Sliding window
        if task_lower.contains("longest substring") || task_lower.contains("maximum subarray") {
            return Some("Use sliding window technique with two pointers to track the current window.".to_string());
        }
        
        // Dynamic programming - OPTIMIZED: Concise to avoid timeout
        if task_lower.contains("edit distance") {
            return Some("DP table: dp[i][j] = s1[i]==s2[j] ? dp[i-1][j-1] : 1+min(dp[i-1][j], dp[i][j-1], dp[i-1][j-1]). Base: dp[0][j]=j, dp[i][0]=i".to_string());
        }
        
        // Stack
        if task_lower.contains("parentheses") || task_lower.contains("valid") {
            return Some("Use a stack to track opening brackets. Push on open, pop and match on close.".to_string());
        }
        
        None
    }
    
    /// Correct code based on error via LLM
    async fn correct_code(
        &self,
        code: &str,
        error: &str,
        task: &str,
        language: Language,
    ) -> Result<String> {
        // Check if LLM is available
        let llm = self.llm_bridge.as_ref()
            .ok_or_else(|| AgentError::GenerationError(
                "LLM backend not configured. Use CodingAgent::with_llm()".to_string()
            ))?;
        
        // Build correction prompt
        let prompt = self.prompt_builder.build_correction_prompt(
            code,
            error,
            task,
            language,
        );
        
        // Generate corrected code via LLM
        llm.correct_code(&prompt).await
    }
    
    /// Handle mathematical task using Symbolica
    async fn handle_math_task(&self, task: &str) -> Result<TaskResult> {
        // Detect language for code generation
        let language = self.detector.detect(task).unwrap_or(Language::Python);
        
        // Extract mathematical expression
        let expr = extract_math_expression(task)
            .ok_or_else(|| AgentError::SymbolicError(
                "Could not extract mathematical expression".to_string()
            ))?;
        
        // Determine operation type
        let symbolic_result = if task.to_lowercase().contains("solve") {
            self.symbolica.solve_equation(&expr)?
        } else if task.to_lowercase().contains("simplify") {
            self.symbolica.simplify(&expr)?
        } else if task.to_lowercase().contains("differentiate") || task.to_lowercase().contains("derivative") {
            let var = self.extract_variable(task).unwrap_or("x".to_string());
            self.symbolica.differentiate(&expr, &var)?
        } else if task.to_lowercase().contains("integrate") || task.to_lowercase().contains("integral") {
            let var = self.extract_variable(task).unwrap_or("x".to_string());
            self.symbolica.integrate(&expr, &var)?
        } else if task.to_lowercase().contains("factor") {
            self.symbolica.factor(&expr)?
        } else if task.to_lowercase().contains("expand") {
            self.symbolica.expand(&expr)?
        } else {
            return Err(AgentError::SymbolicError(
                "Unknown mathematical operation".to_string()
            ));
        };
        
        // Convert symbolic result to code
        let code = self.symbolica.to_code(&symbolic_result, language)?;
        
        // Execute the generated code
        let execution = self.executor.execute(&code, language).ok();
        
        Ok(TaskResult {
            code,
            execution,
            language,
            flux_position: Some(9), // Math tasks always route to Logos
            attempts: 1,
        })
    }
    
    /// Extract variable name from task description
    fn extract_variable(&self, task: &str) -> Option<String> {
        // Look for "with respect to x" or "d/dx"
        if let Some(pos) = task.find("with respect to") {
            let after = &task[pos + 16..].trim();
            return after.chars().next().map(|c| c.to_string());
        }
        if let Some(pos) = task.find("d/d") {
            let after = &task[pos + 3..];
            return after.chars().next().map(|c| c.to_string());
        }
        Some("x".to_string())
    }
    
    /// Analyze code quality using ELP tensors
    pub fn analyze_code_quality(&self, _code: &str) -> ELPTensor {
        // TODO: Implement actual analysis
        // For now, return default balanced ELP
        ELPTensor {
            ethos: 0.33,   // Architecture quality
            logos: 0.33,   // Logic correctness
            pathos: 0.34,  // Readability/UX
        }
    }
}

impl Default for CodingAgent {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a coding task
#[derive(Debug)]
pub struct TaskResult {
    pub code: String,
    pub execution: Option<ExecutionResult>,
    pub language: Language,
    pub flux_position: Option<u8>,
    pub attempts: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_agent_creation() {
        let agent = CodingAgent::new();
        assert_eq!(agent.config.max_correction_attempts, 3);
    }
    
    #[test]
    fn test_flux_position_detection() {
        let agent = CodingAgent::new();
        
        assert_eq!(agent.determine_flux_position("solve this equation"), Some(9));
        assert_eq!(agent.determine_flux_position("design a clean architecture"), Some(3));
        assert_eq!(agent.determine_flux_position("create a beautiful UI"), Some(6));
        assert_eq!(agent.determine_flux_position("write a function"), None);
    }
    
    #[test]
    fn test_code_quality_analysis() {
        let agent = CodingAgent::new();
        let code = "fn main() { println!(\"Hello\"); }";
        let elp = agent.analyze_code_quality(code);
        
        // Should return balanced ELP
        assert!((elp.ethos + elp.logos + elp.pathos - 1.0).abs() < 0.01);
    }
}
