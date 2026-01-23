use crate::ai_integration::AIModelIntegration;
use crate::cache::CacheManager;
use crate::error::Result;
use crate::flux_matrix::FluxMatrixEngine;
use crate::inference_engine::InferenceEngine;
use crate::models::*;
use crate::storage::spatial_database::SpatialDatabase;
use actix_web::{web, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use uuid::Uuid;

use crate::ai::orchestrator::ASIOrchestrator;
use crate::consciousness::ConsciousnessSimulator;

/// Application state containing shared components
#[derive(Clone)]
pub struct AppState {
    pub inference_engine: Arc<RwLock<InferenceEngine>>,
    pub flux_engine: Arc<FluxMatrixEngine>,
    pub database: Arc<SpatialDatabase>,
    pub cache: Arc<CacheManager>,
    pub ai_integration: Arc<AIModelIntegration>,
    pub ai_router: Arc<RwLock<crate::ai::router::AIRouter>>,
    /// ASI Orchestrator - Unified intelligence coordinator
    pub asi_orchestrator: Arc<Mutex<ASIOrchestrator>>,
    // TEMPORARY: ParallelFusion disabled
    // /// ParallelFusion Orchestrator - v0.8.4 Ensemble fusion (97-99% accuracy)
    // pub parallel_fusion: Arc<RwLock<crate::ai::parallel_fusion::ParallelFusionOrchestrator>>,
    /// Meta Orchestrator - Hybrid routing with Flux + ASI (90-95% accuracy, adaptive)
    pub meta_orchestrator: Arc<RwLock<crate::ai::meta_orchestrator::MetaOrchestrator>>,
    /// Consciousness Simulator - v1.6.0 "Memory Palace"
    pub consciousness_simulator: Arc<RwLock<ConsciousnessSimulator>>,
}

/// API request/response models
#[derive(Debug, Deserialize)]
pub struct FluxGenerationRequest {
    pub subject: String,
    pub seed_number: Option<u64>,
    pub use_ai_generation: Option<bool>,
    pub sacred_guides_enabled: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct FluxGenerationResponse {
    pub matrix_id: Uuid,
    pub subject: String,
    pub nodes: Vec<FluxNodeResponse>,
    pub sacred_guides: Vec<SacredGuideResponse>,
    pub generation_source: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct FluxNodeResponse {
    pub position: u8,
    pub base_value: u8,
    pub primary_meaning: String,
    pub positive_associations: Vec<String>,
    pub negative_associations: Vec<String>,
    pub connections: Vec<NodeConnectionResponse>,
}

#[derive(Debug, Serialize)]
pub struct NodeConnectionResponse {
    pub target_position: u8,
    pub connection_type: String,
    pub weight: f32,
}

#[derive(Debug, Serialize)]
pub struct SacredGuideResponse {
    pub position: u8,
    pub divine_properties: Vec<String>,
    pub geometric_significance: String,
}

#[derive(Debug, Deserialize)]
pub struct InferenceRequest {
    pub seed_numbers: Vec<u64>,
    pub subject_filter: String, // "specific", "general_intelligence", "category:X", "all"
    pub include_synonyms: Option<bool>,
    pub include_antonyms: Option<bool>,
    pub confidence_threshold: Option<f32>,
    pub max_depth: Option<u8>,
    pub use_sacred_guides: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct InferenceResponse {
    pub inference_id: Uuid,
    pub inferred_meanings: Vec<InferredMeaningResponse>,
    pub confidence_score: f32,
    pub processing_time_ms: u64,
    pub moral_alignment_summary: String,
}

#[derive(Debug, Serialize)]
pub struct InferredMeaningResponse {
    pub subject: String,
    pub node_position: u8,
    pub primary_meaning: String,
    pub semantic_associations: Vec<SemanticAssociationResponse>,
    pub contextual_relevance: f32,
    pub moral_alignment: String,
}

#[derive(Debug, Serialize)]
pub struct SemanticAssociationResponse {
    pub word: String,
    pub index: i16,
    pub confidence: f32,
    pub context: String,
    pub source: String,
}

#[derive(Debug, Deserialize)]
pub struct TokenizeRequest {
    pub target_meanings: Vec<String>,
    pub subject_filter: String,
    pub max_results: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct TokenizeResponse {
    pub candidate_seeds: Vec<u64>,
    pub explanation: String,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub inference_engine_stats: InferenceEngineStats,
    pub database_status: String,
    pub cache_status: String,
}

#[derive(Debug, Serialize)]
pub struct InferenceEngineStats {
    pub total_matrices: usize,
    pub cached_inferences: usize,
    pub available_subjects: Vec<String>,
}

/// Health check endpoint
pub async fn health_check(data: web::Data<AppState>) -> ActixResult<HttpResponse> {
    let inference_engine = data.inference_engine.read().await;
    let stats = inference_engine.get_statistics();

    let database_status = match data.database.health_check().await {
        Ok(_) => "healthy".to_string(),
        Err(_) => "unhealthy".to_string(),
    };

    let cache_status = match data.cache.health_check().await {
        Ok(_) => "healthy".to_string(),
        Err(_) => "unhealthy".to_string(),
    };

    let response = HealthResponse {
        status: "healthy".to_string(),
        version: "0.1.0".to_string(),
        inference_engine_stats: InferenceEngineStats {
            total_matrices: stats.total_matrices,
            cached_inferences: stats.cached_inferences,
            available_subjects: stats.subjects,
        },
        database_status,
        cache_status,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Generate flux matrix for a subject
pub async fn generate_flux_matrix(
    data: web::Data<AppState>,
    req: web::Json<FluxGenerationRequest>,
) -> ActixResult<HttpResponse> {
    let result = generate_matrix_internal(&data, req.into_inner()).await;

    match result {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(e) => Ok(HttpResponse::InternalServerError().json(format!("Error: {}", e))),
    }
}

async fn generate_matrix_internal(
    data: &AppState,
    req: FluxGenerationRequest,
) -> Result<FluxGenerationResponse> {
    // Check if matrix already exists in cache
    if let Some(cached_matrix) = data.cache.get_matrix(&req.subject).await? {
        return Ok(convert_matrix_to_response(
            cached_matrix,
            "cache".to_string(),
        ));
    }

    // Check if matrix exists in database
    if let Some(db_matrix) = data.database.get_matrix_by_subject(&req.subject).await? {
        // Cache it for future use
        data.cache.store_matrix(db_matrix.clone()).await?;
        return Ok(convert_matrix_to_response(
            db_matrix,
            "database".to_string(),
        ));
    }

    // Generate new matrix
    let matrix = if req.use_ai_generation.unwrap_or(false) {
        // Use AI to generate matrix
        data.ai_integration
            .generate_subject_matrix(&req.subject)
            .await?
    } else {
        // Use flux engine to create basic matrix
        data.flux_engine.create_matrix(req.subject.clone())?
    };

    // Store in database and cache
    data.database.store_matrix(&matrix).await?;
    data.cache.store_matrix(matrix.clone()).await?;

    // Update inference engine
    let mut inference_engine = data.inference_engine.write().await;
    inference_engine.update_subject_matrix(matrix.clone());

    Ok(convert_matrix_to_response(matrix, "generated".to_string()))
}

/// Reverse inference: process seed numbers to extract meanings (seeds → meanings)
pub async fn reverse_inference_handler(
    data: web::Data<AppState>,
    req: web::Json<InferenceRequest>,
) -> ActixResult<HttpResponse> {
    let result = process_inference_internal(&data, req.into_inner()).await;

    match result {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(e) => Ok(HttpResponse::BadRequest().json(format!("Error: {}", e))),
    }
}

async fn process_inference_internal(
    data: &AppState,
    req: InferenceRequest,
) -> Result<InferenceResponse> {
    // Parse subject filter
    let subject_filter = parse_subject_filter(&req.subject_filter)?;

    // Create processing options
    let processing_options = ProcessingOptions {
        include_synonyms: req.include_synonyms.unwrap_or(true),
        include_antonyms: req.include_antonyms.unwrap_or(true),
        max_depth: req.max_depth.unwrap_or(5),
        confidence_threshold: req.confidence_threshold.unwrap_or(0.3),
        use_sacred_guides: req.use_sacred_guides.unwrap_or(true),
    };

    // Create modern inference input (compression hashes are preferred, seeds for backward compat)
    let inference_input = InferenceInput {
        compression_hashes: Vec::new(), // TODO: Support compression hashes in API
        seed_numbers: req.seed_numbers,
        subject_filter,
        processing_options,
    };

    // Process inference using modern method
    let mut inference_engine = data.inference_engine.write().await;
    let result = inference_engine.process_inference(inference_input).await?;

    Ok(convert_inference_result_to_response(result))
}

/// Forward inference: find seeds for target meanings (meanings → seeds)
pub async fn forward_inference_handler(
    data: web::Data<AppState>,
    req: web::Json<TokenizeRequest>,
) -> ActixResult<HttpResponse> {
    let result = tokenize_internal(&data, req.into_inner()).await;

    match result {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(e) => Ok(HttpResponse::BadRequest().json(format!("Error: {}", e))),
    }
}

async fn tokenize_internal(data: &AppState, req: TokenizeRequest) -> Result<TokenizeResponse> {
    let subject_filter = parse_subject_filter(&req.subject_filter)?;

    let inference_engine = data.inference_engine.read().await;
    let candidate_seeds = inference_engine
        .forward_inference(req.target_meanings.clone(), &subject_filter)
        .await?;

    let explanation = format!(
        "Found {} candidate seeds that could produce meanings related to: {}",
        candidate_seeds.len(),
        req.target_meanings.join(", ")
    );

    Ok(TokenizeResponse {
        candidate_seeds,
        explanation,
    })
}

/// Get all available subjects
pub async fn get_subjects(data: web::Data<AppState>) -> ActixResult<HttpResponse> {
    let inference_engine = data.inference_engine.read().await;
    let stats = inference_engine.get_statistics();

    Ok(HttpResponse::Ok().json(stats.subjects))
}

/// Get specific matrix by subject
pub async fn get_matrix_by_subject(
    data: web::Data<AppState>,
    path: web::Path<String>,
) -> ActixResult<HttpResponse> {
    let subject = path.into_inner();

    let inference_engine = data.inference_engine.read().await;
    if let Some(matrix) = inference_engine.get_subject_matrix(&subject) {
        let response = convert_matrix_to_response(matrix.clone(), "current".to_string());
        Ok(HttpResponse::Ok().json(response))
    } else {
        Ok(HttpResponse::NotFound().json("Matrix not found"))
    }
}

/// Clear inference cache
pub async fn clear_cache(data: web::Data<AppState>) -> ActixResult<HttpResponse> {
    let mut inference_engine = data.inference_engine.write().await;
    inference_engine.clear_cache();

    if let Err(e) = data.cache.clear_all().await {
        return Ok(HttpResponse::InternalServerError().json(format!("Cache clear failed: {}", e)));
    }

    Ok(HttpResponse::Ok().json("Cache cleared successfully"))
}

// Helper functions

fn parse_subject_filter(filter_str: &str) -> Result<SubjectFilter> {
    match filter_str.to_lowercase().as_str() {
        "all" => Ok(SubjectFilter::All),
        "general_intelligence" => Ok(SubjectFilter::GeneralIntelligence),
        s if s.starts_with("category:") => {
            let category = s.strip_prefix("category:").unwrap().to_string();
            Ok(SubjectFilter::Category(category))
        }
        s => Ok(SubjectFilter::Specific(s.to_string())),
    }
}

fn convert_matrix_to_response(matrix: FluxMatrix, source: String) -> FluxGenerationResponse {
    let nodes: Vec<FluxNodeResponse> = matrix
        .nodes
        .values()
        .map(|node| FluxNodeResponse {
            position: node.position,
            base_value: node.base_value,
            primary_meaning: node.semantic_index.neutral_base.clone(),
            positive_associations: node
                .semantic_index
                .positive_associations
                .iter()
                .map(|a| a.word.clone())
                .collect(),
            negative_associations: node
                .semantic_index
                .negative_associations
                .iter()
                .map(|a| a.word.clone())
                .collect(),
            connections: node
                .connections
                .iter()
                .map(|c| NodeConnectionResponse {
                    target_position: c.target_position,
                    connection_type: format!("{:?}", c.connection_type),
                    weight: c.weight,
                })
                .collect(),
        })
        .collect();

    let sacred_guides: Vec<SacredGuideResponse> = matrix
        .sacred_guides
        .values()
        .map(|guide| SacredGuideResponse {
            position: guide.position,
            divine_properties: guide.divine_properties.clone(),
            geometric_significance: guide.geometric_significance.clone(),
        })
        .collect();

    FluxGenerationResponse {
        matrix_id: matrix.id,
        subject: matrix.subject,
        nodes,
        sacred_guides,
        generation_source: source,
        created_at: matrix.created_at.to_rfc3339(),
    }
}

fn convert_inference_result_to_response(result: InferenceResult) -> InferenceResponse {
    let inferred_meanings: Vec<InferredMeaningResponse> = result
        .inferred_meanings
        .iter()
        .map(|meaning| InferredMeaningResponse {
            subject: meaning.subject.clone(),
            node_position: meaning.node_position,
            primary_meaning: meaning.primary_meaning.clone(),
            semantic_associations: meaning
                .semantic_associations
                .iter()
                .map(|assoc| SemanticAssociationResponse {
                    word: assoc.word.clone(),
                    index: assoc.index,
                    confidence: assoc.confidence as f32,
                    context: assoc.get_attribute("context").map(|v| format!("attr:{}", v)).unwrap_or_else(|| "unknown".to_string()),
                    source: "inferred".to_string(),
                })
                .collect(),
            contextual_relevance: meaning.contextual_relevance,
            moral_alignment: format!("{:?}", meaning.moral_alignment),
        })
        .collect();

    // Calculate moral alignment summary
    let constructive_count = result
        .inferred_meanings
        .iter()
        .filter(|m| matches!(m.moral_alignment, MoralAlignment::Constructive(_)))
        .count();
    let destructive_count = result
        .inferred_meanings
        .iter()
        .filter(|m| matches!(m.moral_alignment, MoralAlignment::Destructive(_)))
        .count();
    let neutral_count = result
        .inferred_meanings
        .iter()
        .filter(|m| matches!(m.moral_alignment, MoralAlignment::Neutral))
        .count();

    let moral_alignment_summary = format!(
        "Constructive: {}, Destructive: {}, Neutral: {}",
        constructive_count, destructive_count, neutral_count
    );

    InferenceResponse {
        inference_id: result.id,
        inferred_meanings,
        confidence_score: result.confidence_score,
        processing_time_ms: result.processing_time_ms,
        moral_alignment_summary,
    }
}

/// Get flux node by ID
pub async fn get_flux_node(
    _data: web::Data<AppState>,
    path: web::Path<String>,
) -> ActixResult<HttpResponse> {
    let node_id = path.into_inner();

    // Parse as UUID or position
    if let Ok(position) = node_id.parse::<u8>() {
        // Return node at position from any available matrix
        Ok(HttpResponse::Ok().json(format!("Node at position {}", position)))
    } else {
        Ok(HttpResponse::BadRequest().json("Invalid node ID"))
    }
}

/// Get sacred geometry intersections
pub async fn get_sacred_intersections(
    data: web::Data<AppState>,
    query: web::Query<SacredIntersectionQuery>,
) -> ActixResult<HttpResponse> {
    let dimension = query.dimension.unwrap_or(3);
    let subject = query.subject.clone();

    let inference_engine = data.inference_engine.read().await;

    let mut intersections = Vec::new();

    // Get matrices to analyze
    let matrices: Vec<_> = if let Some(subj) = subject {
        if let Some(matrix) = inference_engine.get_subject_matrix(&subj) {
            vec![matrix.clone()]
        } else {
            vec![]
        }
    } else {
        let stats = inference_engine.get_statistics();
        stats
            .subjects
            .iter()
            .filter_map(|s| inference_engine.get_subject_matrix(s))
            .cloned()
            .collect()
    };

    // Extract sacred guide intersections
    for matrix in matrices {
        for (pos, guide) in &matrix.sacred_guides {
            for intersection in &guide.intersection_points {
                intersections.push(SacredIntersectionResult {
                    subject: matrix.subject.clone(),
                    guide_position: *pos,
                    with_node: intersection.with_node,
                    significance: intersection.significance.clone(),
                    computational_value: intersection.computational_value,
                });
            }
        }
    }

    let total_count = intersections.len();
    Ok(HttpResponse::Ok().json(SacredIntersectionResponse {
        dimension,
        intersections,
        total_count,
    }))
}

/// Generate procedural universe
pub async fn generate_universe(
    data: web::Data<AppState>,
    req: web::Json<UniverseGenerationRequest>,
) -> ActixResult<HttpResponse> {
    let universe_id = Uuid::new_v4();

    // Generate flux pattern for the universe
    let seed = req.seed.unwrap_or(1);
    let flux_pattern = data.flux_engine.seed_to_flux_sequence(seed);

    let response = UniverseGenerationResponse {
        universe_id,
        subject: req.subject.clone(),
        dimensions: req.dimensions.unwrap_or(3),
        flux_pattern: flux_pattern.iter().map(|&v| v as i32).collect(),
        center_index: "1245780".to_string(),
        metadata: UniverseMetadata {
            created_at: chrono::Utc::now().to_rfc3339(),
            generation_method: "flux_matrix".to_string(),
            sacred_guides_active: true,
        },
    };

    Ok(HttpResponse::Ok().json(response))
}

#[derive(Debug, Deserialize)]
pub struct SacredIntersectionQuery {
    pub dimension: Option<u8>,
    pub subject: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SacredIntersectionResponse {
    pub dimension: u8,
    pub intersections: Vec<SacredIntersectionResult>,
    pub total_count: usize,
}

#[derive(Debug, Serialize)]
pub struct SacredIntersectionResult {
    pub subject: String,
    pub guide_position: u8,
    pub with_node: u8,
    pub significance: String,
    pub computational_value: f64,
}

#[derive(Debug, Deserialize)]
pub struct UniverseGenerationRequest {
    pub subject: String,
    pub dimensions: Option<u8>,
    pub seed: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct UniverseGenerationResponse {
    pub universe_id: Uuid,
    pub subject: String,
    pub dimensions: u8,
    pub flux_pattern: Vec<i32>,
    pub center_index: String,
    pub metadata: UniverseMetadata,
}

#[derive(Debug, Serialize)]
pub struct UniverseMetadata {
    pub created_at: String,
    pub generation_method: String,
    pub sacred_guides_active: bool,
}

/// Configure API routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/health", web::get().to(health_check))
            .service(crate::ai::chat_api::chat_text)  // Multi-modal chat endpoint
            .service(crate::ai::dual_response_api::dual_response)  // Dual response (Consensus + Native)
            .service(crate::ai::coding_api::generate_code)  // Code generation endpoint
            .service(crate::ai::coding_api::unified_chat)  // Unified chat (text + code)
            .service(crate::ai::coding_api::unified_chat_stream)  // Streaming chat (SSE)
            .service(crate::ai::coding_api::chat_with_tools)  // Tool-calling chat
            .service(crate::ai::coding_api::generate_session_title)  // Auto-generate session titles
            .service(crate::ai::coding_api::share_session)  // Share sessions with permissions
            // Chat history endpoints
            .service(crate::ai::chat_history_api::list_user_sessions)  // List user's chat sessions
            .service(crate::ai::chat_history_api::get_session_history)  // Get full session history
            .service(crate::ai::chat_history_api::delete_session)  // Delete a session
            .service(crate::ai::chat_history_api::get_chat_stats)  // Get chat statistics
            .service(crate::ai::chat_history_api::continue_session)  // Continue existing session
            .route(
                "/flux/matrix/generate",
                web::post().to(generate_flux_matrix),
            )
            .route("/flux/nodes/{nodeId}", web::get().to(get_flux_node))
            .route(
                "/sacred/geometry/intersections",
                web::get().to(get_sacred_intersections),
            )
            .route("/inference/reverse", web::post().to(reverse_inference_handler))
            .route("/inference/forward", web::post().to(forward_inference_handler))
            .route("/universes/generate", web::post().to(generate_universe))
            .route("/subjects", web::get().to(get_subjects))
            .route("/subjects/generate", web::post().to(generate_subject))
            .route("/subjects/generate-from-visual", web::post().to(generate_subject_from_visual))
            .route("/matrix/{subject}", web::get().to(get_matrix_by_subject))
            .route("/matrix/{subject}/visual-analysis", web::get().to(analyze_matrix_visual))
            .route("/matrix/generate-dynamic", web::post().to(generate_dynamic_color_matrix))
            .route("/cache/clear", web::post().to(clear_cache))
            // Call production routes configurator INSIDE the /api/v1 scope
            .configure(crate::ai::endpoints::configure_production_routes),
    );
}

/// Request to generate a new subject
#[derive(Debug, Deserialize)]
pub struct GenerateSubjectRequest {
    pub subject_name: String,
    pub subjects_dir: Option<String>,
}

/// Response from subject generation
#[derive(Debug, Serialize)]
pub struct GenerateSubjectResponse {
    pub success: bool,
    pub subject_name: String,
    pub module_name: String,
    pub filename: String,
    pub message: String,
}

/// Generate a new subject module dynamically
pub async fn generate_subject(
    data: web::Data<AppState>,
    req: web::Json<GenerateSubjectRequest>,
) -> ActixResult<HttpResponse> {
    use crate::subject_generator::SubjectGenerator;

    let generator = SubjectGenerator::new((*data.ai_integration).clone(), req.subjects_dir.clone());

    match generator.create_subject(&req.subject_name).await {
        Ok(()) => {
            let module_name = req.subject_name.to_lowercase().replace(" ", "_");
            let filename = format!("{}.rs", module_name);

            Ok(HttpResponse::Ok().json(GenerateSubjectResponse {
                success: true,
                subject_name: req.subject_name.clone(),
                module_name,
                filename,
                message: format!(
                    "Subject '{}' generated successfully. Rebuild application to use it.",
                    req.subject_name
                ),
            }))
        }
        Err(e) => Ok(HttpResponse::BadRequest().json(GenerateSubjectResponse {
            success: false,
            subject_name: req.subject_name.clone(),
            module_name: String::new(),
            filename: String::new(),
            message: format!("Failed to generate subject: {}", e),
        })),
    }
}

/// Request to generate subject from visual flux matrix data
#[derive(Debug, Deserialize)]
pub struct GenerateSubjectFromVisualRequest {
    pub subject_name: String,
    pub visual_data: crate::visual_subject_generator::FluxMatrixVisualData,
    pub subjects_dir: Option<String>,
}

/// Generate subject from 2D flux matrix visualization analysis
pub async fn generate_subject_from_visual(
    data: web::Data<AppState>,
    req: web::Json<GenerateSubjectFromVisualRequest>,
) -> ActixResult<HttpResponse> {
    use crate::subject_generator::SubjectGenerator;
    use crate::visual_subject_generator::VisualSubjectGenerator;

    let visual_gen = VisualSubjectGenerator::new((*data.ai_integration).clone());
    let subject_gen = SubjectGenerator::new((*data.ai_integration).clone(), req.subjects_dir.clone());

    // Step 1: Generate subject definition from visual data
    match visual_gen.generate_from_visual_data(&req.visual_data).await {
        Ok(generated) => {
            // Step 2: Write the subject files
            match subject_gen.write_subject_file(&generated) {
                Ok(filename) => {
                    // Step 3: Update mod.rs
                    let _ = subject_gen.update_mod_file(&generated);
                    let _ = subject_gen.update_subject_getter(&generated);

                    let module_name = req.subject_name.to_lowercase().replace(" ", "_");

                    Ok(HttpResponse::Ok().json(GenerateSubjectResponse {
                        success: true,
                        subject_name: req.subject_name.clone(),
                        module_name,
                        filename,
                        message: format!(
                            "Subject '{}' generated from visual analysis. Rebuild to use.",
                            req.subject_name
                        ),
                    }))
                }
                Err(e) => Ok(HttpResponse::BadRequest().json(GenerateSubjectResponse {
                    success: false,
                    subject_name: req.subject_name.clone(),
                    module_name: String::new(),
                    filename: String::new(),
                    message: format!("Failed to write subject files: {}", e),
                })),
            }
        }
        Err(e) => Ok(HttpResponse::BadRequest().json(GenerateSubjectResponse {
            success: false,
            subject_name: req.subject_name.clone(),
            module_name: String::new(),
            filename: String::new(),
            message: format!("Failed to analyze visual data: {}", e),
        })),
    }
}

/// Analyze existing flux matrix and return visual data
pub async fn analyze_matrix_visual(
    data: web::Data<AppState>,
    subject: web::Path<String>,
) -> ActixResult<HttpResponse> {
    use crate::visual_subject_generator::VisualSubjectGenerator;

    let inference_engine = data.inference_engine.read().await;
    
    match inference_engine.get_subject_matrix(&subject) {
        Some(matrix) => {
            let visual_data = VisualSubjectGenerator::extract_visual_data_from_matrix(matrix);
            Ok(HttpResponse::Ok().json(visual_data))
        }
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": format!("Matrix not found for subject: {}", subject)
        }))),
    }
}

/// Request for dynamic color-based matrix generation
#[derive(Debug, Deserialize)]
pub struct DynamicColorRequest {
    pub subject: String,
    pub input: String, // Text or voice transcription
}

/// Response with matrix and color analysis
#[derive(Debug, Serialize)]
pub struct DynamicColorResponse {
    pub matrix: FluxMatrix,
    pub aspect_analysis: crate::dynamic_color_flux::AspectAnalysis,
    pub brick_color_name: String,
    pub dominant_channel: String,
}

/// Generate flux matrix dynamically from text/voice with color-based ML
#[cfg(not(target_arch = "wasm32"))]
pub async fn generate_dynamic_color_matrix(
    data: web::Data<AppState>,
    req: web::Json<DynamicColorRequest>,
) -> ActixResult<HttpResponse> {
    use crate::dynamic_color_flux::DynamicColorFluxGenerator;
    use crate::visualization::dynamic_color_renderer::{render_dynamic_flux_matrix, DynamicColorRenderConfig};

    let generator = DynamicColorFluxGenerator::new();

    match generator.generate_from_input(req.subject.clone(), &req.input).await {
        Ok((matrix, analysis)) => {
            // Generate 2D visualization with colored triangle
            let image_filename = format!("{}_dynamic.png", matrix.subject.to_lowercase().replace(" ", "_"));
            let image_path = format!("outputs/{}", image_filename);
            
            // Create outputs directory if it doesn't exist
            let _ = std::fs::create_dir_all("outputs");
            
            // Render the visualization
            let config = DynamicColorRenderConfig::default();
            if let Err(e) = render_dynamic_flux_matrix(&image_path, &matrix, &analysis, config) {
                eprintln!("Failed to render visualization: {}", e);
            }

            // Store in database and cache
            let _ = data.database.store_matrix(&matrix).await;
            let _ = data.cache.store_matrix(matrix.clone()).await;

            // Update inference engine
            let mut inference_engine = data.inference_engine.write().await;
            inference_engine.update_subject_matrix(matrix.clone());

            Ok(HttpResponse::Ok().json(DynamicColorResponse {
                matrix,
                aspect_analysis: analysis.clone(),
                brick_color_name: analysis.brick_color.name.to_string(),
                dominant_channel: format!("{:?}", analysis.brick_color.dominant_channel),
            }))
        }
        Err(e) => Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": format!("Failed to generate matrix: {}", e)
        }))),
    }
}
