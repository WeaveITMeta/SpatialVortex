//! Response Quality Enhancement System
//!
//! Addresses chat quality issues by implementing:
//! - Adaptive response modes (Concise, Balanced, Detailed, Interactive)
//! - Context-aware prompt engineering
//! - Formatting validation and guidelines
//! - Conversation flow intelligence

use crate::core::sacred_geometry::MatrixInferenceContext;
use std::collections::VecDeque;

/// Response mode for adaptive generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseMode {
    /// 1-2 sentences, direct answer
    Concise,
    /// 2-4 paragraphs with light formatting
    Balanced,
    /// Full explanation with examples
    Detailed,
    /// Questions to clarify before answering
    Interactive,
}

/// Response quality metrics
#[derive(Debug, Clone)]
pub struct ResponseQuality {
    /// Response length is appropriate for query
    pub length_appropriate: bool,
    /// Formatting is clean (minimal markdown)
    pub format_clean: bool,
    /// Tone matches user's formality
    pub tone_matched: bool,
    /// Response is relevant to query
    pub context_relevant: bool,
    /// User can act on response
    pub actionable: bool,
    /// Overall quality score (0.0-1.0)
    pub score: f32,
}

impl ResponseQuality {
    /// Calculate overall quality score
    pub fn calculate_score(&mut self) {
        let mut score = 0.0;
        if self.length_appropriate { score += 0.25; }
        if self.format_clean { score += 0.20; }
        if self.tone_matched { score += 0.15; }
        if self.context_relevant { score += 0.30; }
        if self.actionable { score += 0.10; }
        self.score = score;
    }
    
    /// Check if quality is acceptable (>= 0.7)
    pub fn is_acceptable(&self) -> bool {
        self.score >= 0.7
    }
}

/// Conversation context for flow tracking
#[derive(Debug, Clone)]
pub struct ConversationContext {
    /// Recent user messages
    pub message_history: VecDeque<String>,
    /// Recent responses
    pub response_history: VecDeque<String>,
    /// Topic continuity score (0.0-1.0)
    pub topic_continuity: f32,
    /// Inferred user satisfaction
    pub user_satisfaction: f32,
    /// Maximum history to keep
    max_history: usize,
}

impl ConversationContext {
    /// Create new conversation context
    pub fn new() -> Self {
        Self {
            message_history: VecDeque::with_capacity(10),
            response_history: VecDeque::with_capacity(10),
            topic_continuity: 1.0,
            user_satisfaction: 0.8,
            max_history: 10,
        }
    }
    
    /// Add user message to history
    pub fn add_message(&mut self, message: String) {
        if self.message_history.len() >= self.max_history {
            self.message_history.pop_front();
        }
        self.message_history.push_back(message);
    }
    
    /// Add response to history
    pub fn add_response(&mut self, response: String) {
        if self.response_history.len() >= self.max_history {
            self.response_history.pop_front();
        }
        self.response_history.push_back(response);
    }
    
    /// Get last user message
    pub fn last_message(&self) -> Option<&String> {
        self.message_history.back()
    }
    
    /// Get last response
    pub fn last_response(&self) -> Option<&String> {
        self.response_history.back()
    }
    
    /// Validate response relevance
    pub fn validate_relevance(&self, proposed_response: &str, query: &str) -> bool {
        // If user asks greeting, don't respond with code
        if is_greeting(query) && contains_code(proposed_response) {
            return false;
        }
        
        // If user asks for simplicity, response should be SHORT
        if asks_for_simplicity(query) && proposed_response.len() > 500 {
            return false;
        }
        
        // Check if response is way too long for question
        let query_words = query.split_whitespace().count();
        let response_words = proposed_response.split_whitespace().count();
        if query_words < 10 && response_words > 300 {
            return false;  // Don't write essays for simple questions
        }
        
        true
    }
}

impl Default for ConversationContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Response quality analyzer
pub struct ResponseQualityAnalyzer {
    /// Conversation context
    context: ConversationContext,
}

impl ResponseQualityAnalyzer {
    /// Create new analyzer
    pub fn new() -> Self {
        Self {
            context: ConversationContext::new(),
        }
    }
    
    /// Determine appropriate response mode
    pub fn determine_mode(
        &self,
        query: &str,
        inference_context: Option<&MatrixInferenceContext>,
    ) -> ResponseMode {
        let word_count = query.split_whitespace().count();
        let has_simplify = asks_for_simplicity(query);
        let is_question = query.contains('?');
        
        // Check if it's a greeting or very simple query
        if is_greeting(query) || word_count <= 3 {
            return ResponseMode::Concise;
        }
        
        // User explicitly wants simple/brief
        if has_simplify {
            return ResponseMode::Concise;
        }
        
        // Check complexity from inference context
        let complexity = inference_context
            .map(|ctx| {
                // More positive associations = more complex topic
                (ctx.positive_associations.len() as f32 / 10.0).min(1.0)
            })
            .unwrap_or(0.5);
        
        // Check if query is vague - might need clarification
        if is_vague(query) {
            return ResponseMode::Interactive;
        }
        
        // Determine mode based on query characteristics
        match (word_count, is_question, complexity) {
            (0..=5, _, _) => ResponseMode::Concise,
            (6..=15, true, c) if c < 0.5 => ResponseMode::Balanced,
            (6..=15, _, _) => ResponseMode::Balanced,
            (16..=30, _, c) if c > 0.7 => ResponseMode::Detailed,
            (_, _, _) => ResponseMode::Balanced,
        }
    }
    
    /// Build context-aware prompt
    pub fn build_prompt(
        &self,
        query: &str,
        inference_context: &MatrixInferenceContext,
        mode: ResponseMode,
    ) -> String {
        let mut prompt = String::new();
        
        // Core instruction rules
        prompt.push_str("CRITICAL RESPONSE RULES:\n");
        prompt.push_str("- NEVER start with 'As Vortex, I...' or similar meta-commentary\n");
        prompt.push_str("- Avoid explaining your process or methodology\n");
        prompt.push_str("- Use markdown SPARINGLY: max ONE # header, minimal **bold**\n");
        prompt.push_str("- NEVER use: ===, ###, tables (unless explicitly requested), excessive bullets\n");
        prompt.push_str("- Match the user's tone and complexity level\n");
        prompt.push_str("- Be conversational, not documentary\n\n");
        
        // Mode-specific instructions
        match mode {
            ResponseMode::Concise => {
                prompt.push_str("RESPONSE MODE: CONCISE\n");
                prompt.push_str("- Maximum 2-3 sentences\n");
                prompt.push_str("- Direct answer only\n");
                prompt.push_str("- No explanations unless critical\n\n");
            }
            ResponseMode::Balanced => {
                prompt.push_str("RESPONSE MODE: BALANCED\n");
                prompt.push_str("- 2-4 short paragraphs maximum\n");
                prompt.push_str("- Brief explanation with 1 example if helpful\n");
                prompt.push_str("- Use bullet points ONLY if listing 3+ distinct items\n\n");
            }
            ResponseMode::Detailed => {
                prompt.push_str("RESPONSE MODE: DETAILED\n");
                prompt.push_str("- Provide comprehensive explanation\n");
                prompt.push_str("- Include 2-3 concrete examples\n");
                prompt.push_str("- Break into clear sections (max 3)\n");
                prompt.push_str("- Still conversational, not academic\n\n");
            }
            ResponseMode::Interactive => {
                prompt.push_str("RESPONSE MODE: INTERACTIVE\n");
                prompt.push_str("- Query is vague or ambiguous\n");
                prompt.push_str("- Ask 2-3 clarifying questions\n");
                prompt.push_str("- Offer specific options to help user clarify\n\n");
            }
        }
        
        // Add semantic context
        prompt.push_str("SEMANTIC CONTEXT:\n");
        prompt.push_str(&format!("Topic: {}\n", inference_context.neutral_base));
        
        if !inference_context.positive_associations.is_empty() {
            let relevant: Vec<_> = inference_context.positive_associations
                .iter()
                .take(5)
                .map(|a| a.word.as_str())
                .collect();
            prompt.push_str(&format!("Related concepts: {}\n", relevant.join(", ")));
        }
        
        // Sacred position guidance
        if inference_context.is_sacred {
            prompt.push_str(&format!(
                "Sacred Position {}: Handle with extra care for accuracy\n",
                inference_context.position
            ));
        }
        
        prompt.push_str(&format!("\nUSER QUERY: {}\n\n", query));
        prompt.push_str("Respond naturally and directly:\n");
        
        prompt
    }
    
    /// Score response quality
    pub fn score_response(
        &self,
        query: &str,
        response: &str,
        mode: ResponseMode,
    ) -> ResponseQuality {
        let mut quality = ResponseQuality {
            length_appropriate: check_length_appropriate(response, mode),
            format_clean: check_format_clean(response),
            tone_matched: check_tone_match(query, response),
            context_relevant: self.context.validate_relevance(response, query),
            actionable: has_clear_takeaway(response),
            score: 0.0,
        };
        
        quality.calculate_score();
        quality
    }
    
    /// Add message and response to context
    pub fn update_context(&mut self, message: String, response: String) {
        self.context.add_message(message);
        self.context.add_response(response);
    }
    
    /// Get conversation context
    pub fn context(&self) -> &ConversationContext {
        &self.context
    }
}

impl Default for ResponseQualityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

// Helper functions

/// Check if query is a greeting
fn is_greeting(query: &str) -> bool {
    let lower = query.to_lowercase();
    let greetings = [
        "hi", "hello", "hey", "howdy", "greetings",
        "good morning", "good afternoon", "good evening",
        "how are you", "how do you do", "what's up", "sup",
    ];
    
    greetings.iter().any(|g| lower.contains(g))
}

/// Check if query asks for simplification
fn asks_for_simplicity(query: &str) -> bool {
    let lower = query.to_lowercase();
    let keywords = [
        "simpler", "simple", "brief", "short", "quickly",
        "tldr", "tl;dr", "summary", "summarize", "in short",
        "eli5", "explain like", "plain english",
    ];
    
    keywords.iter().any(|k| lower.contains(k))
}

/// Check if response contains code
fn contains_code(response: &str) -> bool {
    response.contains("```") || 
    response.contains("import ") ||
    response.contains("def ") ||
    response.contains("function ") ||
    response.contains("class ")
}

/// Check if query is vague
fn is_vague(query: &str) -> bool {
    let word_count = query.split_whitespace().count();
    let has_pronouns = query.contains("this") || query.contains("that") || query.contains("it");
    
    // Vague if very short and uses pronouns without context
    (word_count < 5 && has_pronouns) ||
    query.trim() == "?" ||
    query.to_lowercase().starts_with("what about")
}

/// Check if length is appropriate for mode
fn check_length_appropriate(response: &str, mode: ResponseMode) -> bool {
    let word_count = response.split_whitespace().count();
    
    match mode {
        ResponseMode::Concise => word_count <= 50,
        ResponseMode::Balanced => word_count >= 50 && word_count <= 300,
        ResponseMode::Detailed => word_count >= 200 && word_count <= 600,
        ResponseMode::Interactive => word_count <= 100,
    }
}

/// Check if formatting is clean
fn check_format_clean(response: &str) -> bool {
    // Count excessive formatting
    let header_count = response.matches('#').count();
    let bold_count = response.matches("**").count();
    let equals_count = response.matches("===").count();
    let triple_hash = response.matches("###").count();
    
    // Clean if minimal markdown
    header_count <= 2 &&      // Max 2 headers
    bold_count <= 8 &&        // Max 4 bold items (2 markers each)
    equals_count == 0 &&      // No ascii art headers
    triple_hash == 0          // No ### headers
}

/// Check if tone matches query
fn check_tone_match(query: &str, response: &str) -> bool {
    let query_lower = query.to_lowercase();
    let response_lower = response.to_lowercase();
    
    // Informal query should get informal response
    let query_informal = query_lower.contains("hey") || 
                        query_lower.contains("what's") ||
                        query_lower.contains("gonna");
    
    let response_informal = !response_lower.contains("furthermore") &&
                           !response_lower.contains("moreover") &&
                           !response_lower.contains("subsequently");
    
    // If query is informal, response should be too
    if query_informal && !response_informal {
        return false;
    }
    
    // Check for meta-commentary (bad tone)
    if response.contains("As Vortex,") || 
       response.contains("my advanced") ||
       response.contains("I will now") {
        return false;
    }
    
    true
}

/// Check if response has clear takeaway
fn has_clear_takeaway(response: &str) -> bool {
    let lines: Vec<&str> = response.lines().collect();
    
    // Has takeaway if it ends with action or clear conclusion
    if let Some(last_line) = lines.last() {
        let lower = last_line.to_lowercase();
        return !lower.trim().is_empty() &&
               (lower.contains("you can") ||
                lower.contains("try") ||
                lower.contains("consider") ||
                lower.ends_with('.') ||
                lower.ends_with('!'));
    }
    
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_greeting_detection() {
        assert!(is_greeting("Hello there"));
        assert!(is_greeting("How do you do?"));
        assert!(is_greeting("Hey"));
        assert!(!is_greeting("What are trade-offs?"));
    }
    
    #[test]
    fn test_simplicity_detection() {
        assert!(asks_for_simplicity("Can you explain this in simpler terms?"));
        assert!(asks_for_simplicity("tldr please"));
        assert!(asks_for_simplicity("Keep it brief"));
        assert!(!asks_for_simplicity("What are the details?"));
    }
    
    #[test]
    fn test_response_mode_determination() {
        let analyzer = ResponseQualityAnalyzer::new();
        
        // Greeting should be concise
        assert_eq!(analyzer.determine_mode("Hi there!", None), ResponseMode::Concise);
        
        // Simple request should be concise
        assert_eq!(
            analyzer.determine_mode("Explain in simple terms", None),
            ResponseMode::Concise
        );
    }
    
    #[test]
    fn test_format_validation() {
        let clean = "This is a simple response with **one bold** term.";
        assert!(check_format_clean(clean));
        
        let messy = "=== Header ===\n### Subheader ###\n**Bold** **Bold** **Bold** **Bold** **Bold**";
        assert!(!check_format_clean(messy));
    }
}
