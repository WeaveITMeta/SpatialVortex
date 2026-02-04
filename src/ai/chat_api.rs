//! Chat API - Text-based chat with ONNX embeddings and sacred geometry
//!
//! Provides POST /api/v1/chat/text endpoint for real-time chat with:
//! - ONNX text embeddings (sentence-transformers)
//! - Sacred geometry transformation (3-6-9 pattern)
//! - ELP channel analysis
//! - Flux position calculation
//! - AI response generation via router

use actix_web::{post, web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::ai::response_processor::{ResponseProcessor, ExtractedTask};
use crate::ai::orchestrator::ExecutionMode;
use crate::error::SpatialVortexError;

#[cfg(feature = "onnx")]
use once_cell::sync::Lazy;
#[cfg(feature = "onnx")]
use ort::{session::Session, session::builder::GraphOptimizationLevel, value::Value};
#[cfg(feature = "onnx")]
use tokenizers::Tokenizer;

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    pub user_id: String,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub context: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub response: String,
    pub elp_values: ELPValues,
    pub confidence: f64,
    pub flux_position: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processing_time_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tasks: Vec<ExtractedTask>,
    /// Whether native inference was used (Phase 2: Primary Native)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub native_used: Option<bool>,
    /// Controller debug fields (only when SV_DEBUG_CONTROLLER=true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub controller_used: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkpoint_position: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk_score: Option<f32>,
}

#[derive(Debug, Serialize)]
pub struct ELPValues {
    pub ethos: f32,
    pub logos: f32,
    pub pathos: f32,
}

/// Chat endpoint - Process text with ONNX embeddings and sacred geometry
/// 
/// Phase 2: Primary Native - Uses native inference first with LLM fallback
#[post("/chat/text")]
pub async fn chat_text(
    req: web::Json<ChatRequest>,
    state: web::Data<crate::ai::api::AppState>,
) -> Result<HttpResponse> {
    let start = std::time::Instant::now();

    // Step 0: If session_id is provided, route through the cyclic controller.
    // This is the authoritative stateful path that preserves/compresses context.
    if let Some(session_id) = req.session_id.clone() {
        let asi = state.asi_orchestrator.lock().await;

        let max_tokens = if req.message.len() > 200 { 512 } else { 256 };

        match asi.process_controlled(&session_id, &req.message, max_tokens).await {
            Ok(result) => {
                let elapsed = start.elapsed().as_millis() as u64;

                let processor = ResponseProcessor::new();
                let processed = processor.process(&result.result);

                let session_id_clone = session_id.clone();
                let mut response = ChatResponse {
                    response: processed.content,
                    elp_values: ELPValues {
                        ethos: result.elp.ethos as f32,
                        logos: result.elp.logos as f32,
                        pathos: result.elp.pathos as f32,
                    },
                    confidence: result.confidence as f64,
                    flux_position: result.flux_position,
                    processing_time_ms: Some(elapsed),
                    session_id: Some(session_id_clone),
                    subject: None,
                    tasks: processed.tasks,
                    native_used: Some(result.native_used),
                    controller_used: None,
                    fallback_reason: None,
                    checkpoint_position: None,
                    risk_score: None,
                };

                let debug_enabled = std::env::var("SV_DEBUG_CONTROLLER")
                    .ok()
                    .and_then(|v| v.parse::<bool>().ok())
                    .unwrap_or(false);

                if debug_enabled {
                    response.controller_used = Some(true);
                    response.checkpoint_position = Some(result.flux_position);
                    // Extract risk_score from session state if available
                    if let Ok(state) = asi.get_session_state(&session_id).await {
                        if let Some(risk) = state.read().await.last_vcp_risk {
                            response.risk_score = Some(risk);
                        }
                    }
                }

                return Ok(HttpResponse::Ok().json(response));
            }
            Err(e) => {
                tracing::warn!("⚠️ Controlled session inference failed: {}, falling back", e);
            }
        }
    }
    
    // Step 1: Try SPATIAL inference first (Phase 3: Spatial ASI)
    // This uses EustressEngine context for spatially-aware responses
    let use_spatial = std::env::var("USE_SPATIAL_INFERENCE")
        .ok()
        .and_then(|v| v.parse::<bool>().ok())
        .unwrap_or(true);
    
    if use_spatial {
        let asi = state.asi_orchestrator.lock().await;
        
        // REMOVED: EustressIntegration check - will be reimplemented via MCP server
        // Use production engine if available instead
        if asi.has_production_engine() {
            // Determine max tokens based on message complexity
            let max_tokens = if req.message.len() > 200 { 512 } else { 256 };
            
            match asi.generate_with_spatial_context(&req.message, max_tokens).await {
                Ok(result) => {
                    let elapsed = start.elapsed().as_millis() as u64;
                    
                    tracing::info!("🌐 Spatial inference used: {:.1}% confidence, position {}{}", 
                        result.confidence * 100.0,
                        result.flux_position,
                        if result.is_sacred { " (sacred)" } else { "" });
                    
                    // Process response - extract tasks and format markdown
                    let processor = ResponseProcessor::new();
                    let processed = processor.process(&result.result);
                    
                    return Ok(HttpResponse::Ok().json(ChatResponse {
                        response: processed.content,
                        elp_values: ELPValues {
                            ethos: result.elp.ethos as f32,
                            logos: result.elp.logos as f32,
                            pathos: result.elp.pathos as f32,
                        },
                        confidence: result.confidence as f64,
                        flux_position: result.flux_position,
                        processing_time_ms: Some(elapsed),
                        session_id: req.session_id.clone(),
                        subject: None,
                        tasks: processed.tasks,
                        native_used: Some(result.native_used),
                        controller_used: Some(false),
                        fallback_reason: Some("spatial_inference".to_string()),
                        checkpoint_position: None,
                        risk_score: None,
                    }));
                }
                Err(e) => {
                    tracing::warn!("⚠️ Spatial inference failed: {}, falling back to native", e);
                }
            }
        }
    }
    
    // Step 2: Try native inference (Phase 2: Primary Native)
    let use_native = std::env::var("USE_NATIVE_INFERENCE")
        .ok()
        .and_then(|v| v.parse::<bool>().ok())
        .unwrap_or(true);
    
    if use_native {
        let asi = state.asi_orchestrator.lock().await;
        
        // Check if native is enabled
        if asi.is_native_enabled() {
            // Determine execution mode based on message complexity
            let mode = if req.message.len() > 200 {
                ExecutionMode::Thorough
            } else if req.message.contains('?') {
                ExecutionMode::Balanced
            } else {
                ExecutionMode::Fast
            };
            
            match asi.process(&req.message, mode).await {
                Ok(result) if result.native_used => {
                    let elapsed = start.elapsed().as_millis() as u64;
                    
                    tracing::info!("✅ Native inference used: {:.1}% confidence", 
                        result.confidence * 100.0);
                    
                    // Process response - extract tasks and format markdown
                    let processor = ResponseProcessor::new();
                    let processed = processor.process(&result.result);
                    
                    return Ok(HttpResponse::Ok().json(ChatResponse {
                        response: processed.content,
                        elp_values: ELPValues {
                            ethos: result.elp.ethos as f32,
                            logos: result.elp.logos as f32,
                            pathos: result.elp.pathos as f32,
                        },
                        confidence: result.confidence as f64,
                        flux_position: result.flux_position,
                        processing_time_ms: Some(elapsed),
                        session_id: req.session_id.clone(),
                        subject: None,
                        tasks: processed.tasks,
                        native_used: Some(true),
                        controller_used: Some(false),
                        fallback_reason: Some("native_inference".to_string()),
                        checkpoint_position: None,
                        risk_score: None,
                    }));
                }
                Ok(_) => {
                    tracing::info!("🔄 Native confidence low, falling back to AIRouter");
                }
                Err(e) => {
                    tracing::warn!("⚠️ Native inference failed: {}, falling back to AIRouter", e);
                }
            }
        }
    }
    
    // Step 2: Fallback to existing AIRouter logic (ONNX + sacred geometry + LLM)
    tracing::info!("📡 Using AIRouter (external LLM)");
    
    // Get ONNX embedding
    let embedding = match embed_text(&req.message).await {
        Ok(emb) => emb,
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Embedding failed: {}", e)
            })));
        }
    };
    
    // Transform through sacred geometry
    let (signal, ethos, logos, pathos) = match transform_to_sacred_geometry(&embedding) {
        Ok(result) => result,
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Sacred geometry transform failed: {}", e)
            })));
        }
    };
    
    // Calculate flux position
    let position = calculate_flux_position(ethos, logos, pathos, signal);
    
    // Determine or create subject based on content
    let subject = determine_subject(&req.message, ethos, logos, pathos).await;
    
    let dev = std::env::var("DEVELOPMENT_MODE").unwrap_or_default() == "true";
    let ai_response = if dev {
        generate_fallback_response(&req.message, signal, position, &subject)
    } else {
        let mut router_lock = state.ai_router.write().await;
        match router_lock.generate_response(
            &req.message,
            &req.user_id,
            subject.as_deref(),
            signal,
            position,
        ).await {
            Ok(response) => response,
            Err(_e) => generate_fallback_response(&req.message, signal, position, &subject),
        }
    };
    
    let elapsed = start.elapsed().as_millis() as u64;
    
    // Process response - extract tasks and format markdown
    let processor = ResponseProcessor::new();
    let processed = processor.process(&ai_response);
    
    Ok(HttpResponse::Ok().json(ChatResponse {
        response: processed.content,
        elp_values: ELPValues {
            ethos: ethos * 13.0,
            logos: logos * 13.0,
            pathos: pathos * 13.0,
        },
        confidence: calculate_confidence(signal, position),
        flux_position: position,
        processing_time_ms: Some(elapsed),
        session_id: req.session_id.clone(),
        subject,
        tasks: processed.tasks,
        native_used: Some(false),
        controller_used: Some(false),
        fallback_reason: Some("ai_router".to_string()),
        checkpoint_position: None,
        risk_score: None,
    }))
}

#[cfg(feature = "onnx")]
static ONNX_SESSION: Lazy<Option<Arc<RwLock<Session>>>> = Lazy::new(|| {
    let model_path = std::env::var("SPATIALVORTEX_ONNX_MODEL_PATH")
        .unwrap_or_else(|_| "./models/model.onnx".to_string());
    
    match Session::builder()
        .unwrap()
        .with_optimization_level(GraphOptimizationLevel::Level3)
        .unwrap()
        .with_intra_threads(4)
        .unwrap()
        .commit_from_file(&model_path)
    {
        Ok(session) => {
            tracing::info!("✅ ONNX model loaded from {}", model_path);
            Some(Arc::new(RwLock::new(session)))
        }
        Err(e) => {
            tracing::warn!("⚠️ Failed to load ONNX model: {}. Using placeholder.", e);
            None
        }
    }
});

#[cfg(feature = "onnx")]
static TOKENIZER: Lazy<Option<Tokenizer>> = Lazy::new(|| {
    let tokenizer_path = std::env::var("SPATIALVORTEX_ONNX_TOKENIZER_PATH")
        .unwrap_or_else(|_| "./models/tokenizer.json".to_string());
    
    match Tokenizer::from_file(&tokenizer_path) {
        Ok(tokenizer) => {
            tracing::info!("✅ Tokenizer loaded from {}", tokenizer_path);
            Some(tokenizer)
        }
        Err(e) => {
            tracing::warn!("⚠️ Failed to load tokenizer: {}. Using placeholder.", e);
            None
        }
    }
});

/// Embed text using ONNX sentence-transformers model
async fn embed_text(text: &str) -> Result<Vec<f32>, SpatialVortexError> {
    #[cfg(feature = "onnx")]
    {
        // Try to use real ONNX model if available
        if let (Some(session), Some(tokenizer)) = (ONNX_SESSION.as_ref(), TOKENIZER.as_ref()) {
            return embed_text_onnx(text, session, tokenizer).await;
        }
    }
    
    // Fallback to placeholder
    embed_text_placeholder(text)
}

#[cfg(feature = "onnx")]
async fn embed_text_onnx(
    text: &str,
    session: &Arc<RwLock<Session>>,
    tokenizer: &Tokenizer,
) -> Result<Vec<f32>, SpatialVortexError> {
    // Tokenize the input
    let encoding = tokenizer
        .encode(text, true)
        .map_err(|e| SpatialVortexError::AIProviderError(format!("Tokenization failed: {}", e)))?;
    
    let input_ids = encoding.get_ids();
    let attention_mask = encoding.get_attention_mask();
    
    // Convert to i64 arrays (ONNX expects int64)
    let input_ids_i64: Vec<i64> = input_ids.iter().map(|&id| id as i64).collect();
    let attention_mask_i64: Vec<i64> = attention_mask.iter().map(|&mask| mask as i64).collect();
    
    // Create input tensors using (shape, data) tuples
    let seq_len = input_ids_i64.len();
    let shape = vec![1, seq_len];
    
    // Create input values
    let input_ids_value = Value::from_array((shape.clone(), input_ids_i64))
        .map_err(|e| SpatialVortexError::AIProviderError(format!("Failed to create input tensor: {}", e)))?;
    let attention_mask_value = Value::from_array((shape.clone(), attention_mask_i64))
        .map_err(|e| SpatialVortexError::AIProviderError(format!("Failed to create attention mask tensor: {}", e)))?;
    
    // Run inference with named inputs and extract data immediately
    let embeddings_vec = {
        let mut session_guard = session.write().await;
        let outputs = session_guard
            .run(vec![
                ("input_ids", &input_ids_value),
                ("attention_mask", &attention_mask_value),
            ])
            .map_err(|e| SpatialVortexError::AIProviderError(format!("ONNX inference failed: {}", e)))?;
        
        // Extract embeddings from output and clone the data
        let (_shape, embeddings_data) = outputs[0]
            .try_extract_tensor::<f32>()
            .map_err(|e| SpatialVortexError::AIProviderError(format!("Failed to extract tensor: {}", e)))?;
        
        // Clone the data to own it
        embeddings_data.to_vec()
    }; // session_guard is dropped here
    
    // Mean pooling over sequence dimension
    let embedding_dim = 384;
    let mut pooled = vec![0.0f32; embedding_dim];
    
    // embeddings_vec is a flat array with shape [1, seq_len, 384]
    for i in 0..seq_len {
        for j in 0..embedding_dim {
            let idx = i * embedding_dim + j;  // Index into flat array
            pooled[j] += embeddings_vec[idx];
        }
    }
    
    // Average
    for val in pooled.iter_mut() {
        *val /= seq_len as f32;
    }
    
    // Normalize
    let norm: f32 = pooled.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for val in pooled.iter_mut() {
            *val /= norm;
        }
    }
    
    Ok(pooled)
}

fn embed_text_placeholder(text: &str) -> Result<Vec<f32>, SpatialVortexError> {
    // Placeholder: Generate deterministic embedding from text
    let mut embedding = vec![0.0f32; 384];
    for (i, byte) in text.bytes().enumerate() {
        embedding[i % 384] += (byte as f32) / 255.0;
    }
    
    // Normalize
    let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for val in embedding.iter_mut() {
            *val /= norm;
        }
    }
    
    Ok(embedding)
}

/// Transform embedding through sacred geometry (3-6-9 pattern)
fn transform_to_sacred_geometry(embedding: &[f32]) -> Result<(f32, f32, f32, f32), SpatialVortexError> {
    if embedding.len() != 384 {
        return Err(SpatialVortexError::InvalidInput(
            format!("Expected 384-d embedding, got {}", embedding.len())
        ));
    }
    
    // Split into thirds (positions 3, 6, 9)
    let third = embedding.len() / 3;
    let pos_3 = &embedding[0..third];           // Ethos region
    let pos_6 = &embedding[third..2*third];     // Pathos region
    let pos_9 = &embedding[2*third..];          // Logos region
    
    // Calculate channel energies
    let ethos: f32 = pos_3.iter().sum::<f32>() / third as f32;
    let pathos: f32 = pos_6.iter().sum::<f32>() / third as f32;
    let logos: f32 = pos_9.iter().sum::<f32>() / third as f32;
    
    // Calculate signal strength (3-6-9 pattern coherence)
    let sacred_sum = ethos.abs() + pathos.abs() + logos.abs();
    let total_energy: f32 = embedding.iter().map(|x| x.abs()).sum();
    let confidence = if total_energy > 0.0 {
        sacred_sum / total_energy
    } else {
        0.0
    };
    
    // Normalize channels
    let total = ethos + pathos + logos;
    let e_norm = if total.abs() > 1e-6 { ethos / total } else { 0.33 };
    let l_norm = if total.abs() > 1e-6 { logos / total } else { 0.33 };
    let p_norm = if total.abs() > 1e-6 { pathos / total } else { 0.33 };
    
    Ok((confidence, e_norm, l_norm, p_norm))
}

/// Calculate flux position (0-9) based on ELP channels and signal strength
fn calculate_flux_position(ethos: f32, logos: f32, pathos: f32, signal: f32) -> u8 {
    // Position 0: Divine Source (balanced)
    if (ethos - logos).abs() < 0.1 && 
       (logos - pathos).abs() < 0.1 && 
       (pathos - ethos).abs() < 0.1 {
        return 0;
    }
    
    // Sacred positions (3, 6, 9) for high signal strength
    if signal > 0.7 {
        if ethos > logos && ethos > pathos {
            return 3; // Sacred Trinity (Ethos dominant)
        }
        if pathos > ethos && pathos > logos {
            return 6; // Sacred Balance (Pathos dominant)
        }
        if logos > ethos && logos > pathos {
            return 9; // Sacred Completion (Logos dominant)
        }
    }
    
    // Regular positions based on dominant channel
    if ethos > logos && ethos > pathos {
        if signal > 0.5 { 1 } else { 2 }
    } else if pathos > ethos && pathos > logos {
        if signal > 0.5 { 5 } else { 4 }
    } else if logos > ethos && logos > pathos {
        if signal > 0.5 { 8 } else { 7 }
    } else {
        4 // Foundation (fallback)
    }
}

/// Determine subject based on message content and ELP analysis
async fn determine_subject(message: &str, ethos: f32, logos: f32, pathos: f32) -> Option<String> {
    let lower = message.to_lowercase();
    
    // Keyword-based subject detection
    if lower.contains("physics") || lower.contains("force") || lower.contains("energy") {
        Some("Physics".to_string())
    } else if lower.contains("math") || lower.contains("number") || lower.contains("equation") {
        Some("Mathematics".to_string())
    } else if lower.contains("ethics") || lower.contains("moral") || lower.contains("virtue") {
        Some("Ethics".to_string())
    } else if lower.contains("logic") || lower.contains("reason") || lower.contains("proof") {
        Some("Logic".to_string())
    } else if lower.contains("emotion") || lower.contains("feel") || lower.contains("heart") {
        Some("Emotion".to_string())
    } else if lower.contains("sacred") || lower.contains("geometry") || lower.contains("3-6-9") {
        Some("Sacred Geometry".to_string())
    } else if lower.contains("consciousness") || lower.contains("awareness") || lower.contains("mind") {
        Some("Consciousness".to_string())
    } else {
        // Use ELP dominance to suggest subject
        if ethos > 0.4 { Some("Ethics".to_string()) }
        else if logos > 0.4 { Some("Logic".to_string()) }
        else if pathos > 0.4 { Some("Emotion".to_string()) }
        else { None }
    }
}

/// Calculate confidence based on signal strength and position
fn calculate_confidence(signal: f32, position: u8) -> f64 {
    let mut confidence = signal as f64;
    
    // Boost for sacred positions
    if position == 3 || position == 6 || position == 9 {
        confidence += 0.15;
    }
    
    // Boost for position 0 (balanced)
    if position == 0 {
        confidence += 0.10;
    }
    
    // Clamp to [0, 1]
    confidence.min(1.0).max(0.0)
}

/// Generate fallback response when AI router fails
fn generate_fallback_response(_message: &str, signal: f32, position: u8, subject: &Option<String>) -> String {
    let position_names = [
        "Divine Source", "New Beginning", "Duality", "Sacred Trinity",
        "Foundation", "Transformation", "Sacred Balance", "Wisdom",
        "Potential", "Sacred Completion"
    ];
    
    let signal_desc = if signal > 0.7 {
        "strong 3-6-9 coherence"
    } else if signal > 0.5 {
        "moderate pattern coherence"
    } else {
        "emerging pattern"
    };
    
    let subject_text = subject.as_ref()
        .map(|s| format!(" in the domain of {}", s))
        .unwrap_or_default();
    
    format!(
        "Your message has been analyzed through sacred geometry{}. \
        Signal strength: {:.1}% ({}). \
        Flux position: {} - {}. {}",
        subject_text,
        signal * 100.0,
        signal_desc,
        position,
        position_names[position as usize],
        if position == 3 || position == 6 || position == 9 {
            "✨ This is a SACRED position with heightened geometric significance!"
        } else {
            "The vortex pattern reveals meaningful structure in your query."
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_embed_text() {
        let text = "What is consciousness?";
        let embedding = embed_text(text).await.unwrap();
        assert_eq!(embedding.len(), 384);
        
        // Check normalization
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.01);
    }
    
    #[test]
    fn test_sacred_geometry_transform() {
        let embedding = vec![0.1; 384];
        let (signal, ethos, logos, pathos) = transform_to_sacred_geometry(&embedding).unwrap();
        
        assert!(signal > 0.0 && signal <= 1.0);
        assert!(ethos > 0.0 && ethos <= 1.0);
        assert!(logos > 0.0 && logos <= 1.0);
        assert!(pathos > 0.0 && pathos <= 1.0);
    }
    
    #[test]
    fn test_flux_position_sacred() {
        // High signal with Ethos dominant
        let position = calculate_flux_position(0.5, 0.3, 0.2, 0.8);
        assert_eq!(position, 3); // Sacred Trinity
        
        // High signal with Pathos dominant
        let position = calculate_flux_position(0.2, 0.3, 0.5, 0.8);
        assert_eq!(position, 6); // Sacred Balance
        
        // High signal with Logos dominant
        let position = calculate_flux_position(0.2, 0.5, 0.3, 0.8);
        assert_eq!(position, 9); // Sacred Completion
    }
    
    #[test]
    fn test_flux_position_balanced() {
        // Balanced channels
        let position = calculate_flux_position(0.33, 0.33, 0.34, 0.5);
        assert_eq!(position, 0); // Divine Source
    }
}
