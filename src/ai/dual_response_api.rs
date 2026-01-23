//! Multi-Model Chat API - Returns individual model responses + Vortex consensus
//!
//! Enhanced with Vector Field Consensus:
//! - Maps responses to ELP space
//! - Tracks confidence trajectories
//! - Applies diversity-weighted aggregation
//! - Stores high-quality consensus in Confidence Lake

use actix_web::{post, web, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};
use crate::ai::api::AppState;
use crate::ai::consensus::query_multiple_ollama;
use crate::ai::vector_consensus::{ConsensusVectorField, ResponseVector};
use crate::data::models::ELPTensor;

#[derive(Debug, Deserialize)]
pub struct MultiModelRequest {
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct MultiModelResponse {
    /// Individual responses from each Ollama model
    pub model_responses: Vec<ModelMessage>,
    /// Final Vortex consensus synthesized from all models
    pub vortex_consensus: VortexMessage,
}

#[derive(Debug, Serialize)]
pub struct ModelMessage {
    pub model_name: String,
    pub text: String,
    pub confidence: f32,
    pub latency_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct VortexMessage {
    pub text: String,
    pub confidence: f32,
    pub flux_position: u8,
    pub sources_used: Vec<String>,
    /// Consensus field metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consensus_diversity: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sacred_resonance: Option<f32>,
}

/// POST /api/v1/chat/dual-response
/// Returns individual model responses PLUS Vortex native consensus
#[post("/chat/dual-response")]
pub async fn dual_response(
    data: web::Data<AppState>,
    req: web::Json<MultiModelRequest>,
) -> ActixResult<HttpResponse> {
    // Query all Ollama models in parallel
    // DISABLED: Ollama is throwing 500 CUDA errors. Falling back to pure Native mode.
    let ollama_responses: Vec<crate::ai::consensus::ModelResponse> = vec![];
    /*
    let ollama_responses = match query_multiple_ollama(
        &req.message,
        vec!["llama3.2:latest", "mixtral:8x7b", "codellama:13b", "hf.co/CWClabs/CWC-Mistral-Nemo-12B-V2-q4_k_m:latest"],
        None,
    ).await {
        Ok(responses) => responses,
        Err(e) => {
            eprintln!("❌ Failed to query Ollama models: {}", e);
            vec![]
        }
    };
    */

    // Convert to ModelMessages for frontend
    let model_responses: Vec<ModelMessage> = ollama_responses.iter()
        .map(|r| ModelMessage {
            model_name: r.model_name.clone(),
            text: r.response_text.clone(),
            confidence: (r.confidence * 100.0) as f32,
            latency_ms: r.latency_ms,
        }).collect();

    // === VECTOR FIELD CONSENSUS ===
    // Build response vectors for geometric aggregation
    let mut response_vectors: Vec<ResponseVector> = vec![];

    for r in &ollama_responses {
        let mut rv = ResponseVector::new(
            r.response_text.clone(),
            r.model_name.clone(),
            r.confidence as f32,
            r.latency_ms,
        );

        // TODO: Map response to ELP space using flux engine
        // For now, use heuristic mapping based on response characteristics
        let text_len = r.response_text.len() as f64;
        let has_questions = r.response_text.contains('?');
        let has_numbers = r.response_text.chars().any(|c| c.is_numeric());
        
        rv.elp = ELPTensor {
            ethos: if has_questions { 6.0 } else { 5.0 },
            logos: if has_numbers { 7.0 } else { 5.0 },
            pathos: (text_len / 100.0).min(8.0).max(3.0),
        };
        
        // Simple flux position based on confidence
        rv.flux_position = if r.confidence > 0.8 { 9 } else if r.confidence > 0.6 { 6 } else { 3 };

        // TODO: Capture confidence trajectory during streaming
        // For now, single confidence value
        rv.confidence_trajectory = vec![r.confidence as f32];

        response_vectors.push(rv);
    }

    // Build consensus vector field
    let consensus_field = ConsensusVectorField::from_responses(response_vectors);

    tracing::info!(
        "🌀 Vector Consensus: {}",
        consensus_field.summary()
    );

    // Build Vortex consensus with vector field context
    let sources_used: Vec<String> = model_responses.iter()
        .map(|m| m.model_name.clone())
        .collect();

    // Enhanced prompt with consensus field metadata
    let vortex_prompt = format!(
        "Context from {} models:\n\
         - Consensus ELP: ({:.2}, {:.2}, {:.2})\n\
         - Diversity: {:.2} (approach variation)\n\
         - Field Confidence: {:.2}\n\
         - Sacred Resonance: {:.2}\n\
         \nResponses:\n{}\n\
         \nSynthesize the STRONGEST reasoning path for:\n{}",
        consensus_field.vectors.len(),
        consensus_field.consensus_center.ethos,
        consensus_field.consensus_center.logos,
        consensus_field.consensus_center.pathos,
        consensus_field.diversity_score,
        consensus_field.field_confidence,
        consensus_field.sacred_resonance,
        consensus_field.vectors.iter()
            .map(|v| format!("[{}] {}", v.model_name, v.text))
            .collect::<Vec<_>>()
            .join("\n"),
        req.message
    );

    // Get Vortex native response with enhanced context
    let mut asi = data.asi_orchestrator.lock().await;
    let vortex_consensus = match asi.query_ollama(&vortex_prompt, None).await {
        Ok(result) => VortexMessage {
            text: result.result,
            confidence: (result.confidence * 100.0) as f32,
            flux_position: result.flux_position,
            sources_used,
            consensus_diversity: Some(consensus_field.diversity_score),
            sacred_resonance: Some(consensus_field.sacred_resonance),
        },
        Err(e) => VortexMessage {
            text: format!("Vortex synthesis: {}", e),
            confidence: 0.0,
            flux_position: 0,
            sources_used,
            consensus_diversity: Some(consensus_field.diversity_score),
            sacred_resonance: Some(consensus_field.sacred_resonance),
        },
    };

    // TODO: CONFIDENCE LAKE STORAGE
    // Will be enabled once AppState includes confidence_lake field
    // For now, just log the consensus for debugging
    tracing::debug!(
        "📊 Consensus quality: conf={:.2}, div={:.2}, sacred={:.2}",
        consensus_field.field_confidence,
        consensus_field.diversity_score,
        consensus_field.sacred_resonance
    );

    Ok(HttpResponse::Ok().json(MultiModelResponse {
        model_responses,
        vortex_consensus,
    }))
}
