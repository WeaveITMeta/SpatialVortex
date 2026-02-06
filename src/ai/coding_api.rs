//! Coding API - Enhanced code generation with LLM integration
//!
//! Provides POST /api/v1/chat/code endpoint for code generation with:
//! - EnhancedCodingAgent with reasoning chains
//! - Sacred geometry verification
//! - Multi-language support
//! - Code quality analysis
//! - Export-ready format
//! - Conversation history

use crate::ai::conversation_history::{ConversationHistory, MessageRole, MessageMetadata};
use actix_web::{post, web, HttpResponse, Result};
use actix_web::http::header;
use futures::stream;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;

use crate::agents::coding_agent_enhanced::EnhancedCodingAgent;
use crate::agents::thinking_agent::ThinkingAgent;
use crate::ai::tools::{ToolCall, create_default_registry};
use crate::ai::safety::{SafetyGuard, SafetyResult};
#[cfg(feature = "rag")]
use crate::rag::{RAGRetriever, VectorStore, RetrievalConfig};

#[derive(Debug, Deserialize)]
pub struct CodingRequest {
    pub message: String,
    pub user_id: String,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub context: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct CodeBlockResponse {
    pub language: String,
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_steps: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complexity_score: Option<f32>,
}

#[derive(Debug, Serialize)]
pub struct CodingResponse {
    pub response: String,
    pub code_blocks: Vec<CodeBlockResponse>,
    pub is_code_response: bool,
    pub elp_values: ELPValues,
    pub confidence: f32,
    pub flux_position: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_time_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_steps: Option<usize>,
    /// Semantic color detected by ML (hex format: #RRGGBB)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semantic_color: Option<String>,
    /// Primary meaning/mood detected
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_meaning: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ELPValues {
    pub ethos: f64,
    pub logos: f64,
    pub pathos: f64,
}

/// Shared coding agent state with conversation history
pub struct CodingAgentState {
    pub agent: Arc<Mutex<EnhancedCodingAgent>>,
    pub history: Arc<ConversationHistory>,
}

impl Default for CodingAgentState {
    fn default() -> Self {
        Self::new()
    }
}

impl CodingAgentState {
    /// Create new state with persistence enabled
    pub fn new() -> Self {
        // Initialize with persistence to data/chat directory
        let history = Arc::new(ConversationHistory::with_persistence("data/chat"));
        
        Self {
            agent: Arc::new(Mutex::new(EnhancedCodingAgent::new())),
            history,
        }
    }
    
    /// Load saved sessions on startup
    pub async fn load_sessions(&self) -> usize {
        self.history.load_saved_sessions().await
    }
}

/// Internal function to handle general text queries (explanations, questions, discussions)
async fn handle_text_query(
    message: &str,
    session_id: &str,
    _user_id: &str,
    state: &web::Data<Arc<Mutex<CodingAgentState>>>,
) -> Result<HttpResponse> {
    let start = std::time::Instant::now();
    
    // Safety check FIRST
    let safety_guard = SafetyGuard::new();
    match safety_guard.check_input(message) {
        SafetyResult::Blocked(reason) => {
            return Ok(HttpResponse::Ok().json(CodingResponse {
                response: format!("⚠️ Safety Check Failed: {}", reason),
                code_blocks: vec![],
                is_code_response: false,
                elp_values: ELPValues { ethos: 9.0, logos: 5.0, pathos: 4.0 },
                confidence: 1.0,
                flux_position: 3, // Ethos position
                generation_time_ms: Some(start.elapsed().as_millis() as u64),
                reasoning_steps: None,
                semantic_color: Some("#FF4444".to_string()), // Red for warnings
                primary_meaning: Some("Warning".to_string()),
            }));
        }
        SafetyResult::Warning(warning) => {
            eprintln!("⚠️ Safety warning: {}", warning);
            // Continue but log the warning
        }
        SafetyResult::Safe => {
            // All good, proceed
        }
    }
    
    // Get history Arc
    let history = {
        let state_lock = state.lock().await;
        Arc::clone(&state_lock.history)
    };
    
    // Build context-aware prompt from conversation history
    let contextual_prompt = history.build_contextual_prompt(session_id, message).await;
    
    // Add user message to history
    history.add_message(
        session_id,
        MessageRole::User,
        message.to_string(),
        None,
    ).await;
    
    // Use ThinkingAgent for thoughtful, reasoned responses
    let thinking_agent = ThinkingAgent::new();
    
    // RAG Integration: Retrieve relevant context from knowledge base
    let rag_context = retrieve_rag_context(message).await;
    
    let thinking_result = thinking_agent.think_and_respond(
        message,
        Some(&contextual_prompt),
        rag_context.as_deref(),
    ).await;
    
    let llm_response = match thinking_result {
        Ok(result) => {
            // Use the reasoned answer with full chain-of-thought
            // Sanitize output to remove any accidental PII
            let sanitized = safety_guard.sanitize_output(&result.answer);
            // Apply aggressive formatting for readability
            crate::text_formatting::format_quick(&sanitized)
        }
        Err(e) => format!("I apologize, but I encountered an error while thinking through your question: {}", e),
    };
    
    let elapsed = start.elapsed();
    
    // Calculate ELP based on response content
    let (ethos, logos, pathos) = analyze_content_elp(&llm_response);
    
    // Determine flux position based on ELP dominance
    let flux_position = calculate_flux_position(ethos, logos, pathos);
    
    // Calculate confidence (for text responses, use moderate confidence)
    let confidence = 0.75 + (llm_response.len() as f32 / 1000.0).min(0.15);
    
    // Store response in history
    let metadata = MessageMetadata {
        code_blocks: None,
        confidence: Some(confidence),
        language: None,
    };
    
    history.add_message(
        session_id,
        MessageRole::Assistant,
        llm_response.clone(),
        Some(metadata),
    ).await;
    
    // Derive semantic color from subject using ML Color engine
    let (semantic_color, primary_meaning) = derive_color_from_subject_ml(&llm_response);
    
    Ok(HttpResponse::Ok().json(CodingResponse {
        response: llm_response,
        code_blocks: vec![],
        is_code_response: false,
        elp_values: ELPValues { 
            ethos: ethos as f64, 
            logos: logos as f64, 
            pathos: pathos as f64 
        },
        confidence,
        flux_position: flux_position.max(0).min(9) as u8,
        generation_time_ms: Some(elapsed.as_millis() as u64),
        reasoning_steps: None,
        semantic_color: Some(semantic_color),
        primary_meaning: Some(primary_meaning),
    }))
}

/// Internal function to handle code generation logic with conversation context
async fn handle_code_generation(
    message: &str,
    session_id: &str,
    _user_id: &str,
    state: &web::Data<Arc<Mutex<CodingAgentState>>>,
) -> Result<HttpResponse> {
    let start = std::time::Instant::now();
    
    // Get Arc references first (doesn't hold the lock long)
    let (agent_arc, history) = {
        let state_lock = state.lock().await;
        (Arc::clone(&state_lock.agent), Arc::clone(&state_lock.history))
    }; // state_lock dropped here
    
    // Build context-aware prompt from conversation history
    let contextual_prompt = history.build_contextual_prompt(session_id, message).await;
    
    // Add user message to history
    history.add_message(
        session_id,
        MessageRole::User,
        message.to_string(),
        None,
    ).await;
    
    // Now lock the agent separately
    let agent = agent_arc.lock().await;
    
    // Execute with reasoning using contextual prompt
    let result = match agent.execute_with_reasoning(&contextual_prompt).await {
        Ok(res) => res,
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Code generation failed: {}", e)
            })));
        }
    };
    
    let elapsed = start.elapsed();
    
    // Extract code blocks from generated code
    let code_blocks = vec![CodeBlockResponse {
        language: result.language.name().to_lowercase(),
        code: result.code.clone(),
        filename: None,
        reasoning_steps: Some(result.reasoning_chain.steps.len()),
        complexity_score: None,
    }];
    
    // Build response message
    let response_message = format!(
        "Generated {} code with {} reasoning steps and {:.1}% confidence.",
        result.language.name(),
        result.reasoning_chain.steps.len(),
        result.confidence * 100.0
    );
    
    // Extract ELP from final reasoning step
    let final_elp = result.reasoning_chain.steps.last()
        .map(|step| (step.elp_state.ethos, step.elp_state.logos, step.elp_state.pathos))
        .unwrap_or((7.0, 8.0, 5.0)); // Default values
    
    // Store assistant's response with metadata in history
    let metadata = MessageMetadata {
        code_blocks: Some(vec![result.code.clone()]),
        confidence: Some(result.confidence),
        language: Some(result.language.name().to_string()),
    };
    
    history.add_message(
        session_id,
        MessageRole::Assistant,
        response_message.clone(),
        Some(metadata),
    ).await;
    
    Ok(HttpResponse::Ok().json(CodingResponse {
        response: response_message,
        code_blocks,
        is_code_response: true,
        elp_values: ELPValues {
            ethos: final_elp.0,
            logos: final_elp.1,
            pathos: final_elp.2,
        },
        confidence: result.confidence,
        flux_position: result.reasoning_chain.steps.last()
            .map(|s| s.flux_position)
            .unwrap_or(0),
        generation_time_ms: Some(elapsed.as_millis() as u64),
        reasoning_steps: Some(result.reasoning_chain.steps.len()),
        semantic_color: None, // TODO: Enable with color_ml
        primary_meaning: None,
    }))
}

/// Code generation endpoint - Process coding requests with EnhancedCodingAgent
#[post("/chat/code")]
pub async fn generate_code(
    req: web::Json<CodingRequest>,
    state: web::Data<Arc<Mutex<CodingAgentState>>>,
) -> Result<HttpResponse> {
    let start = std::time::Instant::now();
    
    // Determine if this is actually a coding request
    let is_coding_request = is_code_generation_request(&req.message);
    
    if !is_coding_request {
        // Return text response
        return Ok(HttpResponse::Ok().json(CodingResponse {
            response: "I can help you with code generation. Please ask me to write, create, or implement something.".to_string(),
            code_blocks: vec![],
            is_code_response: false,
            elp_values: ELPValues { ethos: 7.0, logos: 8.0, pathos: 5.0 },
            confidence: 0.8,
            flux_position: 6,
            generation_time_ms: Some(start.elapsed().as_millis() as u64),
            reasoning_steps: None,
            semantic_color: Some("#FFA500".to_string()), // Orange for errors
            primary_meaning: Some("Error".to_string()),
        }));
    }
    
    // Use provided session_id or generate one
    let session_id = req.session_id.clone()
        .unwrap_or_else(|| {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            format!("{}_{}", req.user_id, timestamp)
        });
    
    handle_code_generation(&req.message, &session_id, &req.user_id, &state).await
}

/// Determine if the message is requesting code generation
fn is_code_generation_request(message: &str) -> bool {
    let lower = message.to_lowercase();
    
    // Keywords that indicate code generation or code refinement
    let code_keywords = [
        // Initial generation
        "write", "create", "implement", "build", "code", "function",
        "class", "program", "script", "algorithm", "develop", "generate",
        "make a", "show me", "example of", "how to", "can you write",
        // Refinement and improvement
        "improve", "enhance", "refactor", "optimize", "add", "extend",
        "modify", "update", "fix", "change", "comprehensive", "better",
        "more", "expand", "complete", "finish", "add feature", "include",
        // Code-specific terms
        "rust", "python", "javascript", "typescript", "java", "c++",
        "function", "method", "struct", "class", "module", "package",
        "api", "endpoint", "database", "sql", "async", "await"
    ];
    
    code_keywords.iter().any(|keyword| lower.contains(keyword))
}

/// Analyze content to calculate ELP (Ethos, Logos, Pathos) values
/// Now uses LLM for dynamic analysis instead of static keywords!
/// 
/// TODO: Replace hardcoded ELP values in handle_text_query and unified_chat with this function
#[allow(dead_code)]
async fn analyze_content_elp_dynamic(content: &str, llm: &crate::agents::llm_bridge::LLMBridge) -> (f32, f32, f32) {
    let prompt = format!(
        "Analyze this text for Ethos (character/ethics), Logos (logic/reason), and Pathos (emotion).\n\n\
        Text: \"{}\"\n\n\
        Rate each from 0-13 (where 5 is neutral, 13 is very high).\n\
        Respond ONLY with three numbers separated by spaces, like: 7.5 9.2 6.1\n\
        Format: ethos logos pathos",
        content.chars().take(500).collect::<String>() // Limit to 500 chars
    );
    
    match llm.generate_code(&prompt, crate::agents::language::Language::Rust).await {
        Ok(response) => {
            let parts: Vec<&str> = response.trim().split_whitespace().collect();
            if parts.len() >= 3 {
                let ethos = parts[0].parse::<f32>().unwrap_or(5.0).max(0.0).min(13.0);
                let logos = parts[1].parse::<f32>().unwrap_or(5.0).max(0.0).min(13.0);
                let pathos = parts[2].parse::<f32>().unwrap_or(5.0).max(0.0).min(13.0);
                return (ethos, logos, pathos);
            }
            (5.0, 5.0, 5.0) // Fallback to neutral
        }
        Err(_) => analyze_content_elp_fallback(content), // Use fallback
    }
}

/// Fallback ELP analysis using keywords (only if LLM unavailable)
fn analyze_content_elp_fallback(content: &str) -> (f32, f32, f32) {
    let lower = content.to_lowercase();
    let words: Vec<&str> = lower.split_whitespace().collect();
    let word_count = words.len() as f32;

    if word_count == 0.0 {
        return (5.0, 5.0, 5.0);
    }

    // Simplified keyword detection (FALLBACK only)
    let ethos_keywords = ["should", "must", "ethical", "moral", "principle"];
    let logos_keywords = ["because", "therefore", "proof", "evidence", "logic"];
    let pathos_keywords = ["feel", "emotion", "love", "fear", "passion"];

    let ethos_count = words.iter().filter(|w| ethos_keywords.iter().any(|k| w.contains(k))).count() as f32;
    let logos_count = words.iter().filter(|w| logos_keywords.iter().any(|k| w.contains(k))).count() as f32;
    let pathos_count = words.iter().filter(|w| pathos_keywords.iter().any(|k| w.contains(k))).count() as f32;

    let ethos = 5.0 + (ethos_count / word_count * 100.0).min(8.0);
    let logos = 5.0 + (logos_count / word_count * 100.0).min(8.0);
    let pathos = 5.0 + (pathos_count / word_count * 100.0).min(8.0);

    (ethos, logos, pathos)
}

/// Quick synchronous version for when async not available
fn analyze_content_elp(content: &str) -> (f32, f32, f32) {
    analyze_content_elp_fallback(content)
}

/// Convert AspectColor to hex string for frontend
/// TODO: Use this when returning semantic_color in responses
#[cfg(feature = "color_ml")]
#[allow(dead_code)]
fn aspect_color_to_hex(color: &crate::data::AspectColor) -> String {
    format!(
        "#{:02X}{:02X}{:02X}",
        (color.r * 255.0) as u8,
        (color.g * 255.0) as u8,
        (color.b * 255.0) as u8
    )
}

/// Derive semantic color from text subject using trained ML AspectColor system
/// Uses AspectColor::from_meaning() to map subjects to colors
fn derive_color_from_subject_ml(text: &str) -> (String, String) {
    use crate::data::AspectColor;
    
    let lower = text.to_lowercase();
    
    // Extract primary subject/meaning from text (trained subject categories)
    let meaning = if lower.contains("love") || lower.contains("affection") {
        "love"
    } else if lower.contains("joy") || lower.contains("happy") || lower.contains("delight") {
        "joy"
    } else if lower.contains("wisdom") || lower.contains("philosophy") {
        "wisdom"
    } else if lower.contains("courage") || lower.contains("brave") {
        "courage"
    } else if lower.contains("peace") || lower.contains("calm") {
        "peace"
    } else if lower.contains("sorrow") || lower.contains("sad") {
        "sorrow"
    } else if lower.contains("fear") || lower.contains("afraid") {
        "fear"
    } else if lower.contains("hope") {
        "hope"
    } else {
        "balanced"  // Default neutral subject
    };
    
    // Use trained AspectColor system
    let color = AspectColor::from_meaning(meaning);
    
    let hex = format!(
        "#{:02X}{:02X}{:02X}",
        (color.r * 255.0) as u8,
        (color.g * 255.0) as u8,
        (color.b * 255.0) as u8
    );
    
    (hex, meaning.to_string())
}

/// Generate a concise session title from the first message
#[derive(Debug, Deserialize)]
pub struct GenerateTitleRequest {
    pub first_message: String,
    pub session_id: String,
}

#[derive(Debug, Serialize)]
pub struct GenerateTitleResponse {
    pub title: String,
    pub session_id: String,
}

/// Share a session with permissions
#[derive(Debug, Deserialize)]
pub struct ShareSessionRequest {
    pub session_id: String,
    pub share_mode: String, // "anyone" or "restricted"
    pub allowed_emails: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ShareSessionResponse {
    pub share_token: String,
    pub share_url: String,
    pub expires_at: String,
}

#[post("/generate-title")]
pub async fn generate_session_title(
    req: web::Json<GenerateTitleRequest>,
) -> Result<HttpResponse> {
    // Generate title using simple word extraction
    // TODO: Use LLM for better titles with proper prompt
    let title = if req.first_message.len() < 10 {
        "Quick Question".to_string()
    } else {
        // Extract key words and create title
        let words: Vec<&str> = req.first_message
            .split_whitespace()
            .filter(|w| w.len() > 3) // Filter short words
            .take(5)
            .collect();
        
        if words.is_empty() {
            "New Conversation".to_string()
        } else {
            words.join(" ")
                .chars()
                .take(40)
                .collect::<String>()
        }
    };
    
    Ok(HttpResponse::Ok().json(GenerateTitleResponse {
        title,
        session_id: req.session_id.clone(),
    }))
}

#[post("/share")]
pub async fn share_session(
    _req: web::Json<ShareSessionRequest>,
) -> Result<HttpResponse> {
    use rand::Rng;
    
    // Generate a unique share token
    let token: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();
    
    // TODO: Implement session sharing
    // In production, store in database using _req data:
    // - share_token
    // - session_id from _req.session_id
    // - share_mode from _req.mode
    // - allowed_emails from _req.allowed_emails
    // - created_at
    // NOTE: No expiration - links last forever!
    
    // For now, just return a stub response
    // TODO: Store in Confidence Lake or dedicated shares table
    
    let share_url = format!("/share/{}", token);
    
    Ok(HttpResponse::Ok().json(ShareSessionResponse {
        share_token: token,
        share_url,
        expires_at: "never".to_string(), // Links never expire
    }))
}

/// Retrieve relevant context from RAG system
#[cfg(feature = "rag")]
async fn retrieve_rag_context(query: &str) -> Option<String> {
    // Initialize RAG components (in production, these would be singletons)
    let vector_store = Arc::new(VectorStore::new(384));
    let config = RetrievalConfig {
        rerank_top_n: 3,
        min_similarity: 0.6,
        ..Default::default()
    };
    
    let retriever = RAGRetriever::new(vector_store, config);
    
    // Retrieve relevant documents
    match retriever.retrieve(query).await {
        Ok(results) if !results.is_empty() => {
            let context = results.iter()
                .map(|r| format!("Source: {}\n{}", r.doc_id, r.content))
                .collect::<Vec<_>>()
                .join("\n\n");
            Some(context)
        }
        _ => None,
    }
}

/// Retrieve relevant context from RAG system (stub when RAG feature disabled)
#[cfg(not(feature = "rag"))]
async fn retrieve_rag_context(_query: &str) -> Option<String> {
    None
}

/// Calculate flux position based on ELP dominance
fn calculate_flux_position(ethos: f32, logos: f32, pathos: f32) -> i32 {
    let total = ethos + logos + pathos;
    let e_norm = ethos / total;
    let l_norm = logos / total;
    let p_norm = pathos / total;
    
    // Sacred positions for high coherence (above 0.4 normalized)
    if e_norm > 0.4 && l_norm > 0.4 {
        0 // Divine Source (balanced Ethos + Logos)
    } else if e_norm > l_norm && e_norm > p_norm {
        if e_norm > 0.45 { 3 } else { 1 } // Sacred Trinity or New Beginning
    } else if p_norm > e_norm && p_norm > l_norm {
        if p_norm > 0.45 { 6 } else { 5 } // Sacred Balance or Transformation
    } else if l_norm > e_norm && l_norm > p_norm {
        if l_norm > 0.45 { 9 } else { 8 } // Sacred Completion or Potential
    } else {
        4 // Foundation (balanced)
    }
}

/// Unified chat endpoint that handles both text and code
#[post("/chat/unified")]
pub async fn unified_chat(
    req: web::Json<CodingRequest>,
    state: web::Data<Arc<Mutex<CodingAgentState>>>,
) -> Result<HttpResponse> {
    // Use provided session_id or generate one
    let session_id = req.session_id.clone()
        .unwrap_or_else(|| {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            format!("{}_{}", req.user_id, timestamp)
        });
    
    // Check if this is a code generation request
    let is_coding = is_code_generation_request(&req.message);
    
    if is_coding {
        // Route to code generation handler
        handle_code_generation(&req.message, &session_id, &req.user_id, &state).await
    } else {
        // Route to general text/explanation handler
        handle_text_query(&req.message, &session_id, &req.user_id, &state).await
    }
}

/// Streaming chat endpoint for real-time token-by-token responses
#[post("/chat/unified/stream")]
pub async fn unified_chat_stream(
    req: web::Json<CodingRequest>,
    state: web::Data<Arc<Mutex<CodingAgentState>>>,
) -> Result<HttpResponse> {
    // Use provided session_id or generate one
    let session_id = req.session_id.clone()
        .unwrap_or_else(|| {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            format!("{}_{}", req.user_id, timestamp)
        });
    
    let message = req.message.clone();
    
    // Create channel for streaming
    let (tx, rx) = mpsc::channel::<String>(100);
    
    // Spawn task to generate response and stream it
    let state_clone = state.clone();
    tokio::spawn(async move {
        // Get agent and history
        let (agent_arc, history) = {
            let state_lock = state_clone.lock().await;
            (Arc::clone(&state_lock.agent), Arc::clone(&state_lock.history))
        };
        
        // Build contextual prompt
        let contextual_prompt = history.build_contextual_prompt(&session_id, &message).await;
        
        // Add user message to history
        history.add_message(
            &session_id,
            MessageRole::User,
            message.clone(),
            None,
        ).await;
        
        // Generate response (simplified - sends full response in chunks)
        let agent = agent_arc.lock().await;
        match agent.generate_explanation(&contextual_prompt).await {
            Ok(full_response) => {
                // Aggressively format text (protects code blocks) before streaming
                let formatted = crate::text_formatting::format_quick(&full_response);
                // Stream while PRESERVING whitespace/newlines so markdown & code blocks render correctly
                let mut buf = String::new();
                for ch in formatted.chars() {
                    buf.push(ch);
                    if ch.is_whitespace() {
                        if tx.send(buf.clone()).await.is_err() { break; }
                        buf.clear();
                        tokio::time::sleep(tokio::time::Duration::from_millis(16)).await;
                    }
                }
                if !buf.is_empty() {
                    let _ = tx.send(buf).await;
                }
                
                // Store complete response in history
                let (_ethos, _logos, _pathos) = analyze_content_elp(&formatted);
                let metadata = MessageMetadata {
                    code_blocks: None,
                    confidence: Some(0.75 + (formatted.len() as f32 / 1000.0).min(0.15)),
                    language: None,
                };
                
                history.add_message(
                    &session_id,
                    MessageRole::Assistant,
                    formatted,
                    Some(metadata),
                ).await;
            }
            Err(e) => {
                let _ = tx.send(format!("Error: {}", e)).await;
            }
        }
    });
    
    // Create SSE stream
    let stream = stream::unfold(rx, |mut rx| async move {
        rx.recv().await.map(|chunk| {
            let sse_data = format!("data: {}\n\n", serde_json::to_string(&chunk).unwrap_or_default());
            (Ok::<_, actix_web::Error>(web::Bytes::from(sse_data)), rx)
        })
    });
    
    Ok(HttpResponse::Ok()
        .insert_header((header::CONTENT_TYPE, "text/event-stream"))
        .insert_header((header::CACHE_CONTROL, "no-cache"))
        .insert_header(("X-Accel-Buffering", "no"))
        .streaming(Box::pin(stream)))
}

/// Chat endpoint with tool calling support
#[post("/chat/with-tools")]
pub async fn chat_with_tools(
    req: web::Json<CodingRequest>,
    state: web::Data<Arc<Mutex<CodingAgentState>>>,
) -> Result<HttpResponse> {
    let session_id = req.session_id.clone()
        .unwrap_or_else(|| {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            format!("{}_{}", req.user_id, timestamp)
        });
    
    let start = std::time::Instant::now();
    
    // Initialize tool registry
    let tool_registry = create_default_registry();
    let available_tools = tool_registry.get_tools();
    
    // Get agent and history
    let (agent_arc, history) = {
        let state_lock = state.lock().await;
        (Arc::clone(&state_lock.agent), Arc::clone(&state_lock.history))
    };
    
    // Build contextual prompt
    let contextual_prompt = history.build_contextual_prompt(&session_id, &req.message).await;
    
    // Add user message to history
    history.add_message(
        &session_id,
        MessageRole::User,
        req.message.clone(),
        None,
    ).await;
    
    // Check if message needs tool calling
    let needs_tool = detect_tool_need(&req.message);
    
    let final_response = if needs_tool {
        // Execute tool-augmented flow
        let agent = agent_arc.lock().await;
        
        // Ask LLM which tool to use
        let tool_selection_prompt = format!(
            "{}\n\nAvailable tools:\n{}\n\n\
            Based on the user's query, which tool should be called? \
            Respond with ONLY a JSON object in this format:\n\
            {{\"tool\": \"tool_name\", \"args\": {{...arguments...}}}}\n\
            If no tool is needed, respond with {{\"tool\": \"none\"}}",
            contextual_prompt,
            serde_json::to_string_pretty(&available_tools).unwrap_or_default()
        );
        
        match agent.generate_explanation(&tool_selection_prompt).await {
            Ok(llm_response) => {
                // Try to parse tool call from response
                if let Ok(tool_call) = parse_tool_call_from_llm(&llm_response) {
                    if tool_call.name != "none" {
                        // Execute the tool
                        match tool_registry.execute(&tool_call).await {
                            Ok(tool_result) if tool_result.success => {
                                // Generate final response with tool result
                                let final_prompt = format!(
                                    "{}\n\nTool Result from {}:\n{}\n\n\
                                    Now provide a complete answer to the user incorporating this information:",
                                    contextual_prompt,
                                    tool_result.tool_name,
                                    tool_result.result
                                );
                                
                                agent.generate_explanation(&final_prompt).await
                                    .unwrap_or_else(|e| format!("Error: {}", e))
                            }
                            Ok(tool_result) => {
                                format!("Tool execution failed: {}", 
                                    tool_result.error.unwrap_or_default())
                            }
                            Err(e) => format!("Tool error: {}", e),
                        }
                    } else {
                        // No tool needed, use original response
                        llm_response
                    }
                } else {
                    // Couldn't parse tool call, treat as normal response
                    llm_response
                }
            }
            Err(e) => format!("Error: {}", e),
        }
    } else {
        // No tools needed, normal flow
        let agent = agent_arc.lock().await;
        agent.generate_explanation(&contextual_prompt).await
            .unwrap_or_else(|e| format!("Error: {}", e))
    };
    
    let elapsed = start.elapsed();
    
    // Calculate ELP and flux position
    let (ethos, logos, pathos) = analyze_content_elp(&final_response);
    let flux_position = calculate_flux_position(ethos, logos, pathos);
    let confidence = 0.75 + (final_response.len() as f32 / 1000.0).min(0.15);
    
    // Store response in history
    let metadata = MessageMetadata {
        code_blocks: None,
        confidence: Some(confidence),
        language: None,
    };
    
    history.add_message(
        &session_id,
        MessageRole::Assistant,
        final_response.clone(),
        Some(metadata),
    ).await;
    
    Ok(HttpResponse::Ok().json(CodingResponse {
        response: final_response,
        code_blocks: vec![],
        is_code_response: false,
        elp_values: ELPValues { 
            ethos: ethos as f64, 
            logos: logos as f64, 
            pathos: pathos as f64 
        },
        confidence,
        flux_position: flux_position.max(0).min(9) as u8,
        generation_time_ms: Some(elapsed.as_millis() as u64),
        reasoning_steps: None,
        semantic_color: None, // TODO: Enable with color_ml
        primary_meaning: None,
    }))
}

/// Detect if message needs tool calling (LLM-based, not static keywords!)
/// 
/// TODO: Use this in chat_with_tools instead of static keyword matching
#[allow(dead_code)]
async fn detect_tool_need_dynamic(message: &str, llm: &crate::agents::llm_bridge::LLMBridge) -> bool {
    let prompt = format!(
        "Does this user query require external tools to answer accurately?\n\
        Query: \"{}\"\n\n\
        Available tools: calculator, web search, current time\n\n\
        Answer with ONLY 'yes' or 'no':",
        message
    );
    
    match llm.generate_code(&prompt, crate::agents::language::Language::Rust).await {
        Ok(response) => response.trim().to_lowercase().starts_with("yes"),
        Err(_) => false, // Fallback: no tools needed
    }
}

/// Fallback: Simple heuristic if LLM unavailable
fn detect_tool_need(message: &str) -> bool {
    let lower = message.to_lowercase();
    
    // Basic heuristics as FALLBACK only
    lower.contains("calculate") || 
    lower.contains("search") || 
    lower.contains("what time")
}

/// Parse tool call from LLM response
fn parse_tool_call_from_llm(response: &str) -> anyhow::Result<ToolCall> {
    // Try to extract JSON from response
    let json_start = response.find('{').unwrap_or(0);
    let json_end = response.rfind('}').unwrap_or(response.len());
    
    if json_start < json_end {
        let json_str = &response[json_start..=json_end];
        
        #[derive(Deserialize)]
        struct ToolSelection {
            tool: String,
            #[serde(default)]
            args: serde_json::Value,
        }
        
        let selection: ToolSelection = serde_json::from_str(json_str)?;
        
        Ok(ToolCall {
            name: selection.tool,
            arguments: selection.args,
        })
    } else {
        Ok(ToolCall {
            name: "none".to_string(),
            arguments: serde_json::json!({}),
        })
    }
}
