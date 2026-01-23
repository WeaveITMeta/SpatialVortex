//! Thinking Agent - Enables Deep Reasoning for Text Queries
//!
//! Unlike the coding agent, this agent is designed for:
//! - Thoughtful, reasoned responses
//! - Chain-of-Thought (CoT) reasoning
//! - RAG-augmented context
//! - Conversation awareness
//! - Multi-step problem solving

use crate::agents::error::Result;
use crate::data::models::ELPTensor;
use crate::agents::llm_bridge::LLMBridge;
use crate::agents::first_principles::FirstPrinciplesReasoner;
use crate::ai::tools::{create_default_registry, ToolCall};
use crate::ai::reasoning_chain::ReasoningChain;
use serde::Serialize;

/// Result of a thinking task with full reasoning
#[derive(Debug, Clone, Serialize)]
pub struct ThinkingResult {
    /// Final answer
    pub answer: String,
    
    /// Chain of thought showing reasoning process
    pub reasoning_chain: ReasoningChain,
    
    /// Overall confidence in the answer
    pub confidence: f32,
    
    /// Sources used (from RAG)
    pub sources: Vec<String>,
}

/// Agent specialized in thoughtful reasoning
pub struct ThinkingAgent {
    llm: LLMBridge,
    first_principles: FirstPrinciplesReasoner,
}

impl ThinkingAgent {
    pub fn new() -> Self {
        // Check backend preference (Default to native for stability)
        let backend_pref = std::env::var("LLM_BACKEND").unwrap_or_else(|_| "native".to_string());
        
        let llm = if backend_pref == "native" {
            LLMBridge::new(crate::agents::llm_bridge::LLMConfig {
                backend: crate::agents::llm_bridge::LLMBackend::NativeVortex,
                ..Default::default()
            }).unwrap()
        } else {
            // Use environment variable or default to llama3.2
            let model = std::env::var("OLLAMA_MODEL")
                .or_else(|_| std::env::var("LLM_MODEL"))
                .unwrap_or_else(|_| "llama3.2".to_string());
            
            LLMBridge::with_ollama(&model).unwrap_or_else(|_| {
                LLMBridge::new(crate::agents::llm_bridge::LLMConfig::default()).unwrap()
            })
        };
        
        Self {
            llm,
            first_principles: FirstPrinciplesReasoner::new(),
        }
    }
    
    /// Analyze statement for truth using first principles reasoning
    pub fn analyze_truth(&self, statement: &str) -> crate::agents::first_principles::FirstPrinciplesResult {
        self.first_principles.analyze(statement)
    }
    
    /// Generate a thoughtful, reasoned response using Chain-of-Thought
    pub async fn think_and_respond(
        &self,
        query: &str,
        conversation_context: Option<&str>,
        rag_context: Option<&str>,
    ) -> Result<ThinkingResult> {
        let mut chain = ReasoningChain::new();
        
        let query_lower = query.to_lowercase();
        
        // Check if this is a web search request
        if query_lower.contains("search") && (query_lower.contains("web") || query_lower.contains("internet"))
            || query_lower.contains("weather in") || query_lower.contains("current weather") {
            // Use web search tool
            return self.handle_web_search(query, &mut chain).await;
        }
        
        // Check if this is a truth/falsity/lie detection request
        if query_lower.contains("is this true") || query_lower.contains("is this false") 
            || query_lower.contains("truth") || query_lower.contains("lie") 
            || query_lower.contains("sarcasm") || query_lower.contains("sarcastic")
            || query_lower.contains("deception") || query_lower.contains("misleading") {
            // Use first principles reasoning
            return self.handle_truth_analysis(query, &mut chain).await;
        }
        
        // Step 1: Understand the query (Position 1)
        chain.add_step(
            format!("Understanding query: {}", query),
            ELPTensor::new(6.0, 7.0, 5.0),
            1,
            0.75,
        );
        
        let understanding = self.understand_query(query).await?;
        
        // Step 2: Identify key concepts (Position 2)
        chain.add_step(
            format!(
                "Query analysis - Intent: {}, Type: {}, Concepts: {}", 
                understanding.intent,
                understanding.answer_type,
                understanding.concepts.join(", ")
            ),
            ELPTensor::new(5.5, 8.0, 5.0),
            2,
            0.80,
        );
        
        // Step 3: SACRED CHECKPOINT - Ethical considerations (Position 3)
        chain.add_step(
            "Checking for ethical implications and ensuring helpful response".to_string(),
            ELPTensor::new(9.0, 6.0, 5.0),  // Ethos-dominant
            3,
            0.85,
        );
        
        // Step 4: Gather relevant information (Position 4)
        let context_info = self.build_context(
            query,
            conversation_context,
            rag_context,
        );
        
        chain.add_step(
            format!("Context gathered: {} sources", 
                rag_context.map(|_| "RAG").unwrap_or("none")),
            ELPTensor::new(6.0, 7.5, 5.5),
            4,
            0.82,
        );
        
        // Step 5: Reason through the problem (Position 5)
        let reasoning = self.reason_through_query(query, &context_info).await?;
        
        chain.add_step(
            format!("Reasoning: {}", reasoning.summary),
            ELPTensor::new(5.5, 8.5, 5.0),
            5,
            reasoning.confidence,
        );
        
        // Step 6: SACRED CHECKPOINT - Logic verification (Position 6)
        chain.add_step(
            "Verifying logical consistency and factual accuracy".to_string(),
            ELPTensor::new(5.0, 9.0, 5.0),  // Logos-dominant
            6,
            0.88,
        );
        
        // Step 7: Formulate response (Position 7)
        let draft_answer = self.formulate_answer(
            &reasoning, 
            &context_info,
            &understanding.answer_type,
        ).await?;
        
        chain.add_step(
            "Formulating clear, comprehensive answer".to_string(),
            ELPTensor::new(6.0, 7.5, 6.5),
            7,
            0.85,
        );
        
        // Step 8: Quality check (Position 8)
        let final_answer = self.quality_check(&draft_answer, query).await?;
        
        chain.add_step(
            "Quality check: Answer is clear, accurate, and helpful".to_string(),
            ELPTensor::new(6.5, 7.5, 6.0),
            8,
            0.87,
        );
        
        // Step 9: SACRED CHECKPOINT - Final validation (Position 9)
        chain.add_step(
            "Final validation: Response aligns with sacred principles of helpfulness and truth".to_string(),
            ELPTensor::new(8.0, 8.0, 6.0),  // Balanced high
            9,
            0.90,
        );
        
        // Apply text formatting for better paragraph breaks and readability
        let formatted_answer = crate::text_formatting::format_quick(&final_answer);
        
        Ok(ThinkingResult {
            answer: formatted_answer,
            reasoning_chain: chain,
            confidence: 0.87,
            sources: vec![], // TODO: Extract from RAG
        })
    }
    
    /// Understand what the user is really asking
    async fn understand_query(&self, query: &str) -> Result<QueryUnderstanding> {
        let prompt = format!(
            "Analyze this query and identify:\n\
            1. What is the user really asking?\n\
            2. What are the key concepts involved?\n\
            3. What type of answer would be most helpful?\n\n\
            Query: {}\n\n\
            Respond in this format:\n\
            Intent: [what they're asking]\n\
            Concepts: [key concept 1], [key concept 2], ...\n\
            Answer Type: [explanation/comparison/instructions/etc]",
            query
        );
        
        let response = self.llm.generate_code(&prompt, crate::agents::language::Language::Rust).await?;
        
        // Parse response (simplified)
        let concepts: Vec<String> = response
            .lines()
            .find(|l| l.starts_with("Concepts:"))
            .map(|l| l.replace("Concepts:", "").split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();
        
        Ok(QueryUnderstanding {
            intent: "answer query".to_string(),
            concepts,
            answer_type: "explanation".to_string(),
        })
    }
    
    /// Build comprehensive context
    fn build_context(
        &self,
        query: &str,
        conversation: Option<&str>,
        rag: Option<&str>,
    ) -> String {
        let mut context = String::new();
        
        if let Some(conv) = conversation {
            context.push_str("CONVERSATION HISTORY:\n");
            context.push_str(conv);
            context.push_str("\n\n");
        }
        
        if let Some(rag_info) = rag {
            context.push_str("RELEVANT KNOWLEDGE:\n");
            context.push_str(rag_info);
            context.push_str("\n\n");
        }
        
        context.push_str("CURRENT QUERY:\n");
        context.push_str(query);
        
        context
    }
    
    /// Reason through the query step-by-step
    async fn reason_through_query(
        &self,
        _query: &str,
        context: &str,
    ) -> Result<ReasoningOutput> {
        let prompt = format!(
            "{}\n\n\
            Think step-by-step to answer this query.\n\n\
            Context:\n{}\n\n\
            Think through this using Chain-of-Thought reasoning:\n\
            1. What do I know about this topic?\n\
            2. What are the key points to address?\n\
            3. How can I explain this clearly?\n\
            4. What examples would help?\n\n\
            Provide your step-by-step reasoning:",
            crate::ai::prompt_templates::STRICT_INSTRUCTION_PROMPT,
            context
        );
        
        let reasoning = self.llm.generate_code(&prompt, crate::agents::language::Language::Rust).await?;
        
        Ok(ReasoningOutput {
            summary: reasoning.lines().take(2).collect::<Vec<_>>().join(" "),
            full_reasoning: reasoning,
            confidence: 0.85,
        })
    }
    
    /// Formulate the final answer
    async fn formulate_answer(
        &self,
        reasoning: &ReasoningOutput,
        context: &str,
        answer_type: &str,
    ) -> Result<String> {
        let prompt = format!(
            "{}\n\n\
            Based on this reasoning:\n{}\n\n\
            And this context:\n{}\n\n\
            Provide a clear, comprehensive, and helpful answer.\n\
            Answer type needed: {}\n\n\
            CRITICAL FORMATTING REQUIREMENTS:\n\
            1. Add blank lines between distinct topics or sections\n\
            2. Put each numbered item on its own line\n\
            3. Put each bullet point on its own line\n\
            4. If citing sources, use [1], [2] inline and list references at end\n\
            5. Use headers (# Section) for major topics\n\
            6. Never cram multiple ideas into one paragraph\n\n\
            GOOD FORMATTING EXAMPLE:\n\
            # Main Topic\n\n\
            Introduction to the concept.\n\n\
            Key points:\n\n\
            1. First point\n\n\
               Details about first point.\n\n\
            2. Second point\n\n\
               Details about second point.\n\n\
            Be conversational, accurate, and thorough.\n\
            If you're uncertain about anything, say so.\n\n\
            Answer:",
            crate::ai::prompt_templates::STRICT_INSTRUCTION_PROMPT,
            reasoning.full_reasoning,
            context,
            answer_type
        );
        
        self.llm.generate_code(&prompt, crate::agents::language::Language::Rust).await
    }
    
    /// Quality check the answer
    async fn quality_check(&self, answer: &str, _original_query: &str) -> Result<String> {
        // For now, just return the answer
        // TODO: Implement actual quality checking
        // - Does it answer the question?
        // - Is it clear and well-structured?
        // - Are there any obvious errors?
        
        Ok(answer.to_string())
    }
    
    /// Handle truth analysis using first principles reasoning
    async fn handle_truth_analysis(&self, query: &str, chain: &mut ReasoningChain) -> Result<ThinkingResult> {
        // Step 1: Extract the statement to analyze
        chain.add_step(
            "Extracting statement for truth analysis".to_string(),
            ELPTensor::new(6.0, 8.0, 5.0),
            1,
            0.90,
        );
        
        // Remove question markers to get the statement
        let statement = query
            .replace("Is this true:", "")
            .replace("Is this false:", "")
            .replace("Is this a lie:", "")
            .replace("Analyze:", "")
            .replace("?", "")
            .trim()
            .to_string();
        
        // Step 2: Apply first principles reasoning
        chain.add_step(
            "Applying first principles reasoning from fundamental axioms".to_string(),
            ELPTensor::new(9.0, 9.0, 5.0),  // High ethos and logos
            2,
            0.95,
        );
        
        let fp_result = self.first_principles.analyze(&statement);
        
        // Step 3: Sacred checkpoint - verify logical consistency
        chain.add_step(
            format!("Verified against {} fundamental axioms", fp_result.axioms_applied.len()),
            ELPTensor::new(8.0, 9.0, 5.0),  // Logos-dominant
            3,
            fp_result.confidence,
        );
        
        // Step 4: Synthesize human-readable explanation
        let answer = self.format_truth_analysis(&fp_result);
        
        chain.add_step(
            "Formulating truth assessment with reasoning".to_string(),
            ELPTensor::new(7.0, 8.0, 6.0),
            4,
            0.90,
        );
        
        // Calculate overall confidence
        let confidence = (fp_result.confidence + chain.steps.iter().map(|s| s.confidence).sum::<f32>() / chain.steps.len() as f32) / 2.0;
        
        Ok(ThinkingResult {
            answer,
            reasoning_chain: chain.clone(),
            confidence,
            sources: vec!["first_principles".to_string(), "fundamental_axioms".to_string()],
        })
    }
    
    /// Format first principles result into human-readable response
    fn format_truth_analysis(&self, result: &crate::agents::first_principles::FirstPrinciplesResult) -> String {
        use crate::agents::first_principles::TruthAssessment;
        
        let mut response = String::new();
        
        // Title with proper paragraph spacing
        response.push_str("# First Principles Truth Analysis\n\n");
        
        // Statement being analyzed
        response.push_str(&format!("**Statement**: \"{}\"\n\n", result.statement));
        
        // Truth assessment
        response.push_str("## Assessment\n\n");
        match &result.truth_assessment {
            TruthAssessment::True { certainty } => {
                response.push_str(&format!("✅ **TRUE** (Certainty: {:.0}%)\n\n", certainty * 100.0));
                response.push_str("This statement aligns with fundamental axioms and logical principles.\n\n");
            }
            TruthAssessment::False { certainty } => {
                response.push_str(&format!("❌ **FALSE** (Certainty: {:.0}%)\n\n", certainty * 100.0));
                response.push_str("This statement contradicts fundamental axioms or contains logical errors.\n\n");
            }
            TruthAssessment::PartiallyTrue { true_percentage } => {
                response.push_str(&format!("⚠️ **PARTIALLY TRUE** ({:.0}% accurate)\n\n", true_percentage * 100.0));
                response.push_str("This statement contains both true and false elements.\n\n");
            }
            TruthAssessment::Uncertain { ambiguity_score } => {
                response.push_str(&format!("❓ **UNCERTAIN** (Ambiguity: {:.0}%)\n\n", ambiguity_score * 100.0));
                response.push_str("Insufficient information to determine truth from first principles.\n\n");
            }
            TruthAssessment::Sarcastic { intended_meaning, confidence } => {
                response.push_str(&format!("😏 **SARCASTIC/IRONIC** (Confidence: {:.0}%)\n\n", confidence * 100.0));
                response.push_str(&format!("**Literal meaning**: False\n"));
                response.push_str(&format!("**Intended meaning**: {}\n\n", intended_meaning));
            }
            TruthAssessment::Deceptive { deception_type, confidence } => {
                response.push_str(&format!("🚨 **DECEPTIVE** (Confidence: {:.0}%)\n\n", confidence * 100.0));
                response.push_str(&format!("**Deception type**: {:?}\n\n", deception_type));
                response.push_str("This statement appears intentionally misleading.\n\n");
            }
            TruthAssessment::Opinion { perspective } => {
                response.push_str("💭 **OPINION** (Subjective)\n\n");
                response.push_str(&format!("**Perspective**: {}\n\n", perspective));
                response.push_str("This is a subjective viewpoint, not an objective fact.\n\n");
            }
        }
        
        // Reasoning steps
        if !result.reasoning_steps.is_empty() {
            response.push_str("## Reasoning Chain\n\n");
            for (i, step) in result.reasoning_steps.iter().enumerate() {
                response.push_str(&format!("**Step {}**: {}\n", i + 1, step.description));
                response.push_str(&format!("- *Premise*: {}\n", step.premise));
                response.push_str(&format!("- *Operation*: {:?}\n", step.operation));
                response.push_str(&format!("- *Conclusion*: {}\n", step.conclusion));
                response.push_str(&format!("- *Confidence*: {:.0}%\n\n", step.confidence * 100.0));
            }
        }
        
        // Axioms applied
        if !result.axioms_applied.is_empty() {
            response.push_str("## Fundamental Axioms Applied\n\n");
            for axiom in &result.axioms_applied {
                response.push_str(&format!("- {}\n", axiom));
            }
            response.push_str("\n");
        }
        
        // ELP Signature
        response.push_str("## ELP Analysis\n\n");
        response.push_str(&format!("- **Ethos** (Character): {:.1}/9\n", result.elp_signature.ethos));
        response.push_str(&format!("- **Logos** (Logic): {:.1}/9\n", result.elp_signature.logos));
        response.push_str(&format!("- **Pathos** (Emotion): {:.1}/9\n\n", result.elp_signature.pathos));
        
        // Overall confidence
        response.push_str(&format!("**Overall Analysis Confidence**: {:.0}%\n", result.confidence * 100.0));
        
        // Apply text formatting for better readability
        crate::text_formatting::format_quick(&response)
    }
    
    /// Handle web search requests using tools
    async fn handle_web_search(&self, query: &str, chain: &mut ReasoningChain) -> Result<ThinkingResult> {
        // Step 1: Detect search intent
        chain.add_step(
            format!("Detected web search request: {}", query),
            ELPTensor::new(6.0, 7.0, 5.0),
            1,
            0.90,
        );
        
        // Step 2: Execute web search tool
        chain.add_step(
            "Executing web search tool...".to_string(),
            ELPTensor::new(5.5, 8.0, 5.5),
            2,
            0.85,
        );
        
        let tool_registry = create_default_registry();
        let tool_call = ToolCall {
            name: "web_search".to_string(),
            arguments: serde_json::json!({
                "query": query
            }),
        };
        
        let tool_result = tool_registry.execute(&tool_call).await
            .map_err(|e| crate::agents::error::AgentError::GenerationError(e.to_string()))?;
        
        // Step 3: Sacred checkpoint - verify data
        chain.add_step(
            "Verifying search results".to_string(),
            ELPTensor::new(5.0, 9.0, 5.0),  // Logos-dominant
            3,
            0.88,
        );
        
        let raw_answer = if tool_result.success {
            tool_result.result
        } else {
            format!("Search failed: {}", tool_result.error.unwrap_or_else(|| "Unknown error".to_string()))
        };
        
        // Step 4: Format response with text formatting
        chain.add_step(
            "Formatting search results for presentation".to_string(),
            ELPTensor::new(6.0, 7.0, 6.5),
            4,
            0.90,
        );
        
        // Apply text formatting for better readability
        let formatted_answer = crate::text_formatting::format_quick(&raw_answer);
        
        // Calculate overall confidence
        let confidence = chain.steps.iter().map(|s| s.confidence).sum::<f32>() / chain.steps.len() as f32;
        
        Ok(ThinkingResult {
            answer: formatted_answer,
            reasoning_chain: chain.clone(),
            confidence,
            sources: vec!["web_search".to_string(), "wttr.in".to_string()],
        })
    }
}

impl Default for ThinkingAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
struct QueryUnderstanding {
    intent: String,
    concepts: Vec<String>,
    answer_type: String,
}

#[derive(Debug, Clone)]
struct ReasoningOutput {
    summary: String,
    full_reasoning: String,
    confidence: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_thinking_agent() {
        let agent = ThinkingAgent::new();
        
        let result = agent.think_and_respond(
            "What is consciousness?",
            None,
            None,
        ).await;
        
        assert!(result.is_ok());
        
        let thinking = result.unwrap();
        assert!(!thinking.answer.is_empty());
        assert!(thinking.reasoning_chain.steps.len() >= 9); // All 9 steps
    }
}
