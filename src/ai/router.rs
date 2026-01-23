//! AI Router - Dynamic subject generation with consensus or Grok 4 fallback
//!
//! Provides intelligent routing for:
//! - Dynamic subject matrix generation
//! - Multi-AI consensus for subject creation
//! - Grok 4 API fallback
//! - Flux matrix instruction set execution

use crate::error::{Result, SpatialVortexError};
use crate::core::sacred_geometry::flux_matrix::FluxMatrixEngine;
use crate::processing::lock_free_flux::LockFreeFluxMatrix;
use crate::models::{FluxNode, SemanticIndex, SemanticAssociation, NodeAttributes, NodeState, NodeDynamics};
use crate::ai::prompt_templates::CHAT_SYSTEM_PROMPT;
use chrono::Utc;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct AIRouter {
    _flux_engine: FluxMatrixEngine,  // Reserved for future flux matrix integration
    /// Active matrix instances by subject name
    matrices: Arc<DashMap<String, Arc<LockFreeFluxMatrix>>>,
    grok_api_key: Option<String>,
    grok_endpoint: String,
    consensus_providers: Vec<AIProvider>,
    use_consensus: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIProvider {
    Grok4,
    OpenAI,
    Anthropic,
    Gemini,
    Llama,
    Mistral,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubjectCreationRequest {
    pub subject_name: String,
    pub description: String,
    pub use_consensus: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FluxMatrixInstruction {
    pub operation: MatrixOperation,
    pub position: u8,
    pub concept: String,
    pub positive_associations: Vec<String>,
    pub negative_associations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MatrixOperation {
    Define,      // Define position concept
    Associate,   // Add associations
    Connect,     // Connect positions
    Validate,    // Validate coherence
}

impl AIRouter {
    /// Create new AI router with optional Grok 4 API key
    pub fn new(grok_api_key: Option<String>, use_consensus: bool) -> Self {
        Self {
            _flux_engine: FluxMatrixEngine::new(),
            matrices: Arc::new(DashMap::new()),
            grok_api_key,
            grok_endpoint: "https://api.x.ai/v1".to_string(),
            consensus_providers: vec![
                AIProvider::Grok4,
                AIProvider::OpenAI,
                AIProvider::Anthropic,
            ],
            use_consensus,
        }
    }
    
    /// Enable or disable consensus mode
    pub fn set_consensus_mode(&mut self, enabled: bool) {
        self.use_consensus = enabled;
    }
    
    /// Get matrix for a subject (if it exists)
    pub fn get_matrix(&self, subject: &str) -> Option<Arc<LockFreeFluxMatrix>> {
        self.matrices.get(subject).map(|entry| entry.value().clone())
    }
    
    /// List all active subjects
    pub fn list_subjects(&self) -> Vec<String> {
        self.matrices.iter().map(|entry| entry.key().clone()).collect()
    }
    
    /// Generate response using appropriate AI provider
    pub async fn generate_response(
        &mut self,
        message: &str,
        _user_id: &str,
        subject: Option<&str>,
        confidence: f32,
        flux_position: u8,
    ) -> Result<String> {
        // Check if subject exists, create if needed
        let subject_name = if let Some(subj) = subject {
            // Check if matrix exists, create if not
            if !self.matrices.contains_key(subj) {
                println!("📊 Matrix for '{}' not found, creating...", subj);
                self.create_dynamic_subject(subj).await?;
            }
            subj.to_string()
        } else {
            "General".to_string()
        };
        
        // Build context from sacred geometry analysis
        let context = self.build_context(message, confidence, flux_position, &subject_name);
        
        // Route to appropriate AI provider
        if self.use_consensus {
            self.generate_consensus_response(&context).await
        } else {
            self.generate_grok4_response(&context).await
        }
    }
    
    /// Create dynamic subject using AI consensus or Grok 4
    pub async fn create_dynamic_subject(&mut self, subject_name: &str) -> Result<()> {
        println!("🌀 Creating dynamic subject: {}", subject_name);
        
        // Generate flux matrix instructions
        let instructions = if self.use_consensus {
            self.generate_matrix_instructions_consensus(subject_name).await?
        } else {
            self.generate_matrix_instructions_grok4(subject_name).await?
        };
        
        // Execute instructions in order
        self.execute_matrix_instructions(subject_name, &instructions).await?;
        
        println!("✅ Subject '{}' created with {} positions", subject_name, instructions.len());
        Ok(())
    }
    
    /// Generate flux matrix instructions using Grok 4
    async fn generate_matrix_instructions_grok4(&self, subject: &str) -> Result<Vec<FluxMatrixInstruction>> {
        let prompt = self.build_matrix_instruction_prompt(subject);
        
        if let Some(api_key) = &self.grok_api_key {
            match self.call_grok4_api(&prompt, api_key).await {
                Ok(response) => self.parse_matrix_instructions(&response),
                Err(e) => {
                    println!("⚠️  Grok 4 API failed: {}, using fallback", e);
                    Ok(self.generate_fallback_instructions(subject))
                }
            }
        } else {
            println!("ℹ️  No Grok API key, using fallback");
            Ok(self.generate_fallback_instructions(subject))
        }
    }
    
    /// Generate flux matrix instructions using consensus
    async fn generate_matrix_instructions_consensus(&self, subject: &str) -> Result<Vec<FluxMatrixInstruction>> {
        let prompt = self.build_matrix_instruction_prompt(subject);
        
        let mut responses = Vec::new();
        
        // Try each provider
        for provider in &self.consensus_providers {
            if let Ok(response) = self.call_provider(provider, &prompt).await {
                responses.push(response);
            }
        }
        
        if responses.is_empty() {
            println!("⚠️  All AI providers failed, using fallback");
            return Ok(self.generate_fallback_instructions(subject));
        }
        
        // Synthesize consensus from responses
        self.synthesize_consensus_instructions(&responses)
    }
    
    /// Build prompt for flux matrix instruction generation
    fn build_matrix_instruction_prompt(&self, subject: &str) -> String {
        format!(r#"Generate a flux matrix for the subject "{}".

INSTRUCTIONS:
1. Create 10 position definitions (0-9) following sacred geometry principles
2. Position 0 is the neutral center/void
3. Positions 3, 6, 9 are SACRED positions (geometric anchors)
4. Other positions follow the vortex pattern: 1→2→4→8→7→5→1
5. Each position needs:
   - A clear concept name
   - 3-5 positive associations (constructive meanings)
   - 3-5 negative associations (destructive meanings)

ORDER OF OPERATIONS:
1. Define Position 0 (center/void) - neutral concept
2. Define Sacred Positions (3, 6, 9) - core anchors for {subject}
3. Define Vortex Flow Positions (1, 2, 4, 8, 7, 5) - progression
4. Add positive associations to all positions
5. Add negative associations to all positions
6. Validate coherence and connections

OUTPUT FORMAT:
Position 0: [Concept] | Positive: [...] | Negative: [...]
Position 1: [Concept] | Positive: [...] | Negative: [...]
...
Position 9: [Concept] | Positive: [...] | Negative: [...]

SUBJECT: {}
Begin generation:"#, subject, subject)
    }
    
    /// Call Grok 4 API with custom system prompt
    async fn call_grok4_api_with_system(&self, prompt: &str, api_key: &str, system_prompt: &str) -> Result<String> {
        let client = reqwest::Client::new();
        
        let body = serde_json::json!({
            "model": "grok-4",
            "messages": [{
                "role": "system",
                "content": system_prompt
            }, {
                "role": "user",
                "content": prompt
            }],
            "temperature": 0.7,
            "max_tokens": 2000,
        });
        
        let response = client
            .post(&self.grok_endpoint)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| SpatialVortexError::AIProviderError(format!("Grok 4 request failed: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(SpatialVortexError::AIProviderError(
                format!("Grok 4 returned status: {}", response.status())
            ));
        }
        
        let json: serde_json::Value = response.json().await
            .map_err(|e| SpatialVortexError::AIProviderError(format!("Failed to parse Grok 4 response: {}", e)))?;
        
        json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| SpatialVortexError::AIProviderError("No content in Grok 4 response".to_string()))
            .map(|s| s.to_string())
    }
    
    /// Call Grok 4 API for matrix generation (uses sacred geometry system prompt)
    async fn call_grok4_api(&self, prompt: &str, api_key: &str) -> Result<String> {
        let system_prompt = "You are an expert in sacred geometry and vortex mathematics. \
                           Generate precise flux matrix definitions following the 3-6-9 pattern.";
        self.call_grok4_api_with_system(prompt, api_key, system_prompt).await
    }
    
    /// Call any AI provider
    async fn call_provider(&self, provider: &AIProvider, prompt: &str) -> Result<String> {
        match provider {
            AIProvider::Grok4 => {
                if let Some(api_key) = &self.grok_api_key {
                    self.call_grok4_api(prompt, api_key).await
                } else {
                    Err(SpatialVortexError::AIProviderError("No Grok API key".to_string()))
                }
            },
            _ => {
                // TODO: Implement other providers
                Err(SpatialVortexError::AIProviderError(
                    format!("Provider {:?} not yet implemented", provider)
                ))
            }
        }
    }
    
    /// Parse AI response into matrix instructions
    fn parse_matrix_instructions(&self, response: &str) -> Result<Vec<FluxMatrixInstruction>> {
        let mut instructions = Vec::new();
        
        for line in response.lines() {
            if line.trim().is_empty() { continue; }
            
            // Parse format: "Position X: [Concept] | Positive: [...] | Negative: [...]"
            if let Some(pos_str) = line.strip_prefix("Position ") {
                if let Some((pos_num, rest)) = pos_str.split_once(':') {
                    if let Ok(position) = pos_num.trim().parse::<u8>() {
                        if position > 9 { continue; }
                        
                        let parts: Vec<&str> = rest.split('|').collect();
                        if parts.len() >= 3 {
                            let concept = parts[0].trim().to_string();
                            let positive = parts[1].replace("Positive:", "").trim()
                                .split(',').map(|s| s.trim().to_string()).collect();
                            let negative = parts[2].replace("Negative:", "").trim()
                                .split(',').map(|s| s.trim().to_string()).collect();
                            
                            instructions.push(FluxMatrixInstruction {
                                operation: MatrixOperation::Define,
                                position,
                                concept,
                                positive_associations: positive,
                                negative_associations: negative,
                            });
                        }
                    }
                }
            }
        }
        
        if instructions.is_empty() {
            Err(SpatialVortexError::InvalidInput("Failed to parse matrix instructions".to_string()))
        } else {
            Ok(instructions)
        }
    }
    
    /// Generate fallback instructions for subject
    fn generate_fallback_instructions(&self, subject: &str) -> Vec<FluxMatrixInstruction> {
        let _subject_lower = subject.to_lowercase();
        
        // Base template that adapts to subject
        vec![
            FluxMatrixInstruction {
                operation: MatrixOperation::Define,
                position: 0,
                concept: format!("{} Void", subject),
                positive_associations: vec!["potential".into(), "origin".into(), "source".into()],
                negative_associations: vec!["emptiness".into(), "absence".into(), "void".into()],
            },
            FluxMatrixInstruction {
                operation: MatrixOperation::Define,
                position: 1,
                concept: format!("{} Beginning", subject),
                positive_associations: vec!["initiation".into(), "start".into(), "genesis".into()],
                negative_associations: vec!["hesitation".into(), "false start".into()],
            },
            FluxMatrixInstruction {
                operation: MatrixOperation::Define,
                position: 3,
                concept: format!("{} Trinity", subject),
                positive_associations: vec!["harmony".into(), "synthesis".into(), "unity".into()],
                negative_associations: vec!["conflict".into(), "division".into()],
            },
            FluxMatrixInstruction {
                operation: MatrixOperation::Define,
                position: 6,
                concept: format!("{} Balance", subject),
                positive_associations: vec!["equilibrium".into(), "harmony".into(), "symmetry".into()],
                negative_associations: vec!["imbalance".into(), "chaos".into()],
            },
            FluxMatrixInstruction {
                operation: MatrixOperation::Define,
                position: 9,
                concept: format!("{} Completion", subject),
                positive_associations: vec!["fulfillment".into(), "mastery".into(), "achievement".into()],
                negative_associations: vec!["stagnation".into(), "ending".into()],
            },
        ]
    }
    
    /// Synthesize consensus from multiple AI responses
    fn synthesize_consensus_instructions(&self, responses: &[String]) -> Result<Vec<FluxMatrixInstruction>> {
        // Parse all responses
        let mut all_instructions: Vec<Vec<FluxMatrixInstruction>> = Vec::new();
        for response in responses {
            if let Ok(instructions) = self.parse_matrix_instructions(response) {
                all_instructions.push(instructions);
            }
        }
        
        if all_instructions.is_empty() {
            return Err(SpatialVortexError::AIProviderError("No valid responses".to_string()));
        }
        
        // Vote on best concept for each position
        let mut final_instructions = Vec::new();
        for position in 0..=9 {
            let mut concept_votes: HashMap<String, usize> = HashMap::new();
            
            for instructions in &all_instructions {
                if let Some(inst) = instructions.iter().find(|i| i.position == position) {
                    *concept_votes.entry(inst.concept.clone()).or_insert(0) += 1;
                }
            }
            
            // Get most voted concept
            if let Some((concept, _)) = concept_votes.iter().max_by_key(|(_, count)| *count) {
                // Aggregate associations
                let mut positive = Vec::new();
                let mut negative = Vec::new();
                
                for instructions in &all_instructions {
                    if let Some(inst) = instructions.iter().find(|i| i.position == position) {
                        positive.extend(inst.positive_associations.clone());
                        negative.extend(inst.negative_associations.clone());
                    }
                }
                
                // Deduplicate
                positive.sort();
                positive.dedup();
                negative.sort();
                negative.dedup();
                
                final_instructions.push(FluxMatrixInstruction {
                    operation: MatrixOperation::Define,
                    position,
                    concept: concept.clone(),
                    positive_associations: positive.into_iter().take(5).collect(),
                    negative_associations: negative.into_iter().take(5).collect(),
                });
            }
        }
        
        Ok(final_instructions)
    }
    
    /// Execute matrix instructions in proper order
    async fn execute_matrix_instructions(
        &mut self,
        subject: &str,
        instructions: &[FluxMatrixInstruction],
    ) -> Result<()> {
        // Create new matrix
        let matrix = Arc::new(LockFreeFluxMatrix::new(subject.to_string()));
        
        // Sort by position: 0 first, then 3-6-9, then rest
        let mut sorted = instructions.to_vec();
        sorted.sort_by_key(|inst| {
            match inst.position {
                0 => 0,
                3 => 1,
                6 => 2,
                9 => 3,
                p => 4 + p,
            }
        });
        
        // Execute in order - convert instructions to nodes and insert
        for instruction in sorted {
            match instruction.operation {
                MatrixOperation::Define => {
                    let node = self.instruction_to_flux_node(&instruction);
                    matrix.insert(node);
                    
                    println!("  Position {}: {} (Sacred: {})", 
                        instruction.position,
                        instruction.concept,
                        [3, 6, 9].contains(&instruction.position)
                    );
                }
                _ => {
                    // Other operations can be added later
                }
            }
        }
        
        // Store matrix for reuse
        self.matrices.insert(subject.to_string(), matrix);
        
        Ok(())
    }
    
    /// Convert FluxMatrixInstruction to FluxNode
    fn instruction_to_flux_node(&self, instruction: &FluxMatrixInstruction) -> FluxNode {
        // Calculate base value according to vortex pattern
        let base_value = match instruction.position {
            0 => 0,  // Neutral center
            1 => 1,
            2 => 2,
            3 => 3,  // Sacred
            4 => 4,
            5 => 5,
            6 => 6,  // Sacred
            7 => 7,
            8 => 8,
            9 => 9,  // Sacred
            _ => instruction.position,  // Fallback
        };
        
        // Create semantic associations from instruction
        let mut positive_assocs = Vec::new();
        for (idx, word) in instruction.positive_associations.iter().enumerate() {
            positive_assocs.push(SemanticAssociation::new(
                word.clone(),
                (idx + 1) as i16,  // Positive indices
                0.8,  // Default confidence
            ));
        }
        
        let mut negative_assocs = Vec::new();
        for (idx, word) in instruction.negative_associations.iter().enumerate() {
            negative_assocs.push(SemanticAssociation::new(
                word.clone(),
                -((idx + 1) as i16),  // Negative indices (cast before negation)
                0.8,  // Default confidence
            ));
        }
        
        FluxNode {
            position: instruction.position,
            base_value,
            semantic_index: SemanticIndex {
                positive_associations: positive_assocs,
                negative_associations: negative_assocs,
                neutral_base: instruction.concept.clone(),
                predicates: Vec::new(),
                relations: Vec::new(),
            },
            attributes: NodeAttributes {
                properties: HashMap::new(),
                parameters: HashMap::new(),
                state: NodeState {
                    active: true,
                    last_accessed: Utc::now(),
                    usage_count: 0,
                    context_stack: Vec::new(),
                },
                dynamics: NodeDynamics::default(),
            },
            connections: Vec::new(),
        }
    }
    
    /// Build context for AI response generation
    fn build_context(&self, message: &str, signal: f32, position: u8, subject: &str) -> String {
        format!(
            "User message: {}\n\
             Subject domain: {}\n\
             Signal strength: {:.1}% (3-6-9 coherence)\n\
             Flux position: {} ({})\n\
             Sacred position: {}\n\n\
             Respond naturally while incorporating this sacred geometry analysis.",
            message,
            subject,
            signal * 100.0,
            position,
            self.get_position_name(position),
            [3, 6, 9].contains(&position)
        )
    }
    
    /// Generate response using Grok 4
    async fn generate_grok4_response(&self, context: &str) -> Result<String> {
        if let Some(api_key) = &self.grok_api_key {
            // Use the proper chat system prompt for conversational responses
            self.call_grok4_api_with_system(context, api_key, CHAT_SYSTEM_PROMPT).await
        } else {
            Ok("Grok 4 API key not configured. Using fallback response.".to_string())
        }
    }
    
    /// Generate response using consensus
    async fn generate_consensus_response(&self, context: &str) -> Result<String> {
        let mut responses = Vec::new();
        
        for provider in &self.consensus_providers {
            if let Ok(response) = self.call_provider(provider, context).await {
                responses.push(response);
            }
        }
        
        if responses.is_empty() {
            return Ok("No AI providers available.".to_string());
        }
        
        // Simple consensus: use most common response or first one
        Ok(responses[0].clone())
    }
    
    /// Get position name
    fn get_position_name(&self, position: u8) -> &str {
        match position {
            0 => "Divine Source",
            1 => "New Beginning",
            2 => "Duality",
            3 => "Sacred Trinity",
            4 => "Foundation",
            5 => "Transformation",
            6 => "Sacred Balance",
            7 => "Wisdom",
            8 => "Potential",
            9 => "Sacred Completion",
            _ => "Unknown",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_router_creation() {
        let router = AIRouter::new(None, false);
        assert!(!router.use_consensus);
    }
    
    #[test]
    fn test_consensus_mode() {
        let mut router = AIRouter::new(None, false);
        router.set_consensus_mode(true);
        assert!(router.use_consensus);
    }
    
    #[test]
    fn test_fallback_instructions() {
        let router = AIRouter::new(None, false);
        let instructions = router.generate_fallback_instructions("Test Subject");
        assert!(!instructions.is_empty());
        assert!(instructions.iter().any(|i| i.position == 3));
        assert!(instructions.iter().any(|i| i.position == 6));
        assert!(instructions.iter().any(|i| i.position == 9));
    }
}
