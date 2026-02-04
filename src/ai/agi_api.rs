//! AGI API Endpoint - Flux-Native Reasoning
//!
//! This endpoint exposes Vortex's AGI capabilities through a REST API.
//! Unlike traditional chat APIs that just query LLMs, this endpoint:
//! 1. Converts query → flux state
//! 2. Reasons internally in geometric space
//! 3. Queries LLMs only when entropy is high
//! 4. Returns reasoning chain + final answer

use crate::ai::flux_reasoning::{FluxReasoningChain, FluxThought};
use crate::ml::meta_learning::MetaLearningEngine;
use crate::storage::SpatialDatabase;
use actix_web::{web, HttpResponse};
use serde::{Serialize, Deserialize};

/// AGI query request
#[derive(Debug, Deserialize)]
pub struct AGIQuery {
    /// User's question
    pub query: String,
    
    /// Maximum reasoning steps (default: 20)
    #[serde(default = "default_max_steps")]
    pub max_steps: usize,
    
    /// Include full reasoning chain in response? (default: false)
    #[serde(default)]
    pub include_chain: bool,
}

fn default_max_steps() -> usize { 20 }

/// AGI response
#[derive(Debug, Serialize)]
pub struct AGIResponse {
    /// Final answer in natural language
    pub answer: String,
    
    /// Overall confidence (0-100%)
    pub confidence: f32,
    
    /// Final entropy level
    pub final_entropy: f32,
    
    /// Total reasoning steps taken
    pub steps_taken: usize,
    
    /// Number of LLM oracle queries made
    pub oracle_queries: usize,
    
    /// Sacred milestones reached (3, 6, 9)
    pub sacred_milestones: Vec<u8>,
    
    /// Did reasoning converge?
    pub converged: bool,
    
    /// Full reasoning chain (if requested)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_chain: Option<Vec<ReasoningStep>>,
    
    /// Processing time in milliseconds
    pub processing_time_ms: u64,

    /// ID of learned pattern (if successful)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub learned_pattern_id: Option<String>,
}

/// A single step in the reasoning chain (for transparency)
#[derive(Debug, Serialize)]
pub struct ReasoningStep {
    /// Step number
    pub step: usize,
    
    /// Vortex position (1-8)
    pub vortex_position: u8,
    
    /// ELP state
    pub ethos: f64,
    pub logos: f64,
    pub pathos: f64,
    
    /// Confidence at this step
    pub certainty: f32,
    
    /// Entropy at this step
    pub entropy: f32,
    
    /// Was an oracle queried?
    pub oracle_query: Option<String>,
    
    /// Reasoning trace
    pub trace: String,
}

impl From<&FluxThought> for ReasoningStep {
    fn from(thought: &FluxThought) -> Self {
        Self {
            step: 0, // Will be set by caller
            vortex_position: thought.vortex_position,
            ethos: thought.elp_state.ethos,
            logos: thought.elp_state.logos,
            pathos: thought.elp_state.pathos,
            certainty: thought.certainty,
            entropy: thought.entropy,
            oracle_query: thought.oracle_contributions.last()
                .map(|o| format!("{}: {}", o.model, o.question)),
            trace: thought.reasoning_trace.clone(),
        }
    }
}

/// AGI endpoint - The core of SpatialVortex intelligence
///
/// POST /api/agi
/// Body: { "query": "How do I reverse type 2 diabetes?", "max_steps": 20 }
pub async fn agi_endpoint(query: web::Json<AGIQuery>) -> HttpResponse {
    let start_time = std::time::Instant::now();
    
    tracing::info!("🧠 AGI query: {}", query.query);
    
    // Create reasoning chain
    let mut chain = FluxReasoningChain::new(&query.query);
    
    // Reason!
    match chain.reason(query.max_steps).await {
        Ok(final_thought) => {
            let processing_time = start_time.elapsed().as_millis() as u64;
            
            // Build reasoning chain if requested
            let reasoning_chain = if query.include_chain {
                Some(chain.thoughts.iter().enumerate().map(|(i, thought)| {
                    let mut step = ReasoningStep::from(thought);
                    step.step = i + 1;
                    step
                }).collect())
            } else {
                None
            };
            
            // Convert to natural language (synthesize)
            let answer = match chain.synthesize_final_answer().await {
                Ok(a) => a,
                Err(e) => {
                    tracing::error!("Failed to synthesize answer: {}", e);
                    format!("Reasoning complete but synthesis failed: {}", e)
                }
            };
            
            // Try to learn from this chain if it was successful
            let mut learned_pattern_id = None;
            if chain.has_converged() && chain.chain_confidence > 0.7 {
                // We need a DB connection for meta-learning
                // In a real app, this would be injected via app_data
                // TODO: Fix type mismatch between FluxReasoningChain and ReasoningChain
                // if let Ok(_db) = SpatialDatabase::from_env().await {
                //     let engine = MetaLearningEngine::new();
                //     if let Ok(id) = engine.learn_from_chain(&chain).await {
                //         tracing::info!("🧠 Learned new reasoning pattern: {}", id);
                //         learned_pattern_id = Some(id.to_string());
                //     }
                // }
            }

            let response = AGIResponse {
                answer,
                confidence: final_thought.certainty * 100.0,
                final_entropy: final_thought.entropy,
                steps_taken: chain.thoughts.len(),
                oracle_queries: final_thought.oracle_contributions.len(),
                sacred_milestones: chain.sacred_milestones.clone(),
                converged: chain.has_converged(),
                reasoning_chain,
                processing_time_ms: processing_time,
                learned_pattern_id,
            };
            
            tracing::info!("✅ AGI response: {} steps, {:.0}% confidence, {} oracles, {}ms",
                response.steps_taken,
                response.confidence,
                response.oracle_queries,
                response.processing_time_ms
            );
            
            HttpResponse::Ok().json(response)
        },
        Err(e) => {
            tracing::error!("❌ AGI reasoning failed: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Reasoning failed: {}", e)
            }))
        }
    }
}

/// Health check for AGI system
pub async fn agi_health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "system": "SpatialVortex AGI",
        "capabilities": [
            "flux_reasoning",
            "geometric_thought",
            "oracle_queries",
            "sacred_geometry",
            "self_consolidation"
        ],
        "version": "1.0.0-alpha"
    }))
}

/// Configure AGI routes
pub fn configure_agi_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/agi")
            .route("", web::post().to(agi_endpoint))
            .route("/health", web::get().to(agi_health))
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_reasoning_step_conversion() {
        use crate::data::models::ELPTensor;
        use chrono::Utc;
        use crate::ai::flux_reasoning::EntropyType;
        
        let thought = FluxThought {
            elp_state: ELPTensor { ethos: 7.0, logos: 8.0, pathos: 6.0 },
            vortex_position: 2,
            certainty: 0.6,
            entropy: 0.4,
            entropy_type: EntropyType::Low,
            oracle_contributions: vec![],
            timestamp: Utc::now(),
            reasoning_trace: "Test trace".to_string(),
        };
        
        let step = ReasoningStep::from(&thought);
        
        assert_eq!(step.vortex_position, 2);
        assert_eq!(step.certainty, 0.6);
        assert_eq!(step.entropy, 0.4);
    }
}
