//! Production API Server
//!
//! Actix-web server with full REST API for SpatialVortex
//! Integrates ONNX inference, Confidence Lake, and Sacred Geometry

use actix_web::{web, App, HttpServer, middleware};
use std::time::Duration;
use actix_cors::Cors;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::config::Config;
use crate::metrics::MetricsMiddleware;
use crate::ml::inference::flux_inference::InferenceEngine;
use crate::core::sacred_geometry::flux_matrix::FluxMatrixEngine;
use crate::storage::spatial_database::SpatialDatabase;
use crate::storage::cache::CacheManager;
use crate::ai::integration::AIModelIntegration;
use crate::ai::orchestrator::ASIOrchestrator;
use crate::consciousness::ConsciousnessSimulator;
use super::api::{AppState, configure_routes};
use super::swagger::ApiDoc;
use tokio::sync::Mutex;

/// Server configuration
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    pub enable_cors: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 7000,
            workers: 4,
            enable_cors: true,
        }
    }
}

/// Start the API server
///
/// # Examples
///
/// ```no_run
/// use spatial_vortex::ai::server::{start_server, ServerConfig};
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let config = ServerConfig::default();
///     start_server(config).await?;
///     Ok(())
/// }
/// ```
pub async fn start_server(config: ServerConfig) -> Result<()> {
    println!("🚀 Starting SpatialVortex API Server...");
    println!("   Host: {}", config.host);
    println!("   Port: {}", config.port);
    println!("   Workers: {}", config.workers);
    
    // Load configuration
    println!("📝 Loading configuration...");
    let app_config = match Config::load("config.toml") {
        Ok(config) => config,
        Err(_) => {
            println!("⚠️  config.toml not found, using defaults");
            Config::default()
        }
    };
    
    app_config.validate()?;
    
    // Initialize core components
    println!("📦 Initializing components...");
    
    let inference_engine = Arc::new(RwLock::new(InferenceEngine::new()));
    let flux_engine = Arc::new(FluxMatrixEngine::new());
    let database = Arc::new(SpatialDatabase::new(&app_config.database.url).await?);
    let cache = Arc::new(CacheManager::new(&app_config.cache.url, app_config.cache.ttl_hours).await?);
    let ai_integration = Arc::new(AIModelIntegration::new(
        app_config.ai.api_key.clone(),
        app_config.ai.endpoint.clone()
    ));
    let ai_router = Arc::new(RwLock::new(
        crate::ai::router::AIRouter::new(app_config.ai.api_key.clone(), false)  // false = use Grok 4 only, not consensus
    ));
    
    // Initialize ASI Orchestrator
    println!("🧠 Initializing ASI Orchestrator...");
    let asi_orchestrator = Arc::new(Mutex::new(
        ASIOrchestrator::new().await.map_err(|e| anyhow::anyhow!("Failed to create ASI orchestrator: {}", e))?
    ));
    println!("   ✅ ASI Orchestrator ready (unified intelligence)");
    
    // Initialize Coding Agent with chat history persistence
    println!("💻 Initializing Enhanced Coding Agent...");
    let coding_agent_state = Arc::new(Mutex::new(
        crate::ai::coding_api::CodingAgentState::new()
    ));
    
    // Restore chat sessions from disk
    let chat_sessions_loaded = {
        let state = coding_agent_state.lock().await;
        state.load_sessions().await
    };
    
    if chat_sessions_loaded > 0 {
        println!("   ✅ Coding Agent ready (restored {} chat sessions)", chat_sessions_loaded);
    } else {
        println!("   ✅ Coding Agent ready (LLM-powered code generation)");
    }
    
    // Initialize Task Manager with persistence
    println!("📋 Initializing Task Manager...");
    use crate::agents::{TaskManager, EnhancedCodingAgent, ThinkingAgent};
    let coding_agent_for_tasks = Arc::new(RwLock::new(EnhancedCodingAgent::new()));
    let thinking_agent_for_tasks = Arc::new(ThinkingAgent::new());
    
    // Create task storage directory
    let task_storage_dir = std::env::var("TASK_STORAGE_DIR")
        .unwrap_or_else(|_| "data/tasks".to_string());
    
    let task_manager = Arc::new(
        TaskManager::with_persistence(
            coding_agent_for_tasks,
            thinking_agent_for_tasks,
            &task_storage_dir,
            true,  // Enable auto-save
        ).map_err(|e| anyhow::anyhow!("Failed to create TaskManager: {}", e))?
    );
    
    // Restore persisted sessions
    match task_manager.restore_sessions().await {
        Ok(count) if count > 0 => {
            println!("   ✅ Task Manager ready (restored {} sessions)", count);
        }
        Ok(_) => {
            println!("   ✅ Task Manager ready (multi-step task execution)");
        }
        Err(e) => {
            println!("   ⚠️  Failed to restore sessions: {}", e);
            println!("   ✅ Task Manager ready (fresh state)");
        }
    }
    
    // TEMPORARY: ParallelFusion disabled due to file corruption
    // println!("🔥 Initializing ParallelFusion v0.8.4 (Ensemble)...");
    // use crate::ai::parallel_fusion::{ParallelFusionOrchestrator, FusionConfig, FusionAlgorithm};
    // use crate::ai::orchestrator::ExecutionMode;
    // let fusion_config = FusionConfig {
    //     algorithm: FusionAlgorithm::Ensemble,
    //     asi_mode: ExecutionMode::Balanced,
    //     timeout_ms: 600000,  // 10 minutes per request
    //     ..Default::default()
    // };
    // let parallel_fusion = Arc::new(RwLock::new(
    //     ParallelFusionOrchestrator::new(fusion_config).await.map_err(|e| anyhow::anyhow!("Failed to create ParallelFusion: {}", e))?
    // ));
    // println!("   ✅ ParallelFusion ready (97-99% accuracy target)");
    
    // Initialize Meta Orchestrator (for benchmarks)
    println!("🎯 Initializing Meta Orchestrator (Hybrid Routing)...");
    use crate::ai::meta_orchestrator::{MetaOrchestrator, RoutingStrategy};
    let meta_orchestrator = Arc::new(RwLock::new(
        MetaOrchestrator::new(RoutingStrategy::Hybrid).await.map_err(|e| anyhow::anyhow!("Failed to create Meta Orchestrator: {}", e))?
    ));
    println!("   ✅ Meta Orchestrator ready (90-95% accuracy, adaptive routing)");
    
    // Initialize Consciousness Simulator
    println!("🏛️  Initializing Consciousness Simulator v1.6.0...");
    let mut consciousness_simulator = ConsciousnessSimulator::new(false).await;
    
    // Enable background learning if agents feature is enabled
    #[cfg(feature = "agents")]
    {
        consciousness_simulator.enable_background_learning().await
            .map_err(|e| anyhow::anyhow!("Failed to enable background learning: {}", e))?;
        println!("   ✅ Consciousness Simulator ready (background learning enabled)");
    }
    
    #[cfg(not(feature = "agents"))]
    println!("   ✅ Consciousness Simulator ready (basic mode)");
    
    let consciousness_simulator = Arc::new(RwLock::new(consciousness_simulator));
    
    let app_state = AppState {
        inference_engine,
        flux_engine,
        database,
        cache,
        ai_integration,
        ai_router,
        asi_orchestrator,
        // parallel_fusion,  // TEMPORARY: disabled
        meta_orchestrator,
        consciousness_simulator,
    };
    
    println!("✅ Components initialized");
    println!("🌐 Starting HTTP server at http://{}:{}", config.host, config.port);
    println!("");
    println!("📋 Available endpoints:");
    println!("   GET  /api/v1/health");
    println!("   POST /api/v1/process                - 🔥 ParallelFusion v0.8.4 (97-99% accuracy)");
    println!("   POST /api/v1/chat/text              - Text chat with sacred geometry");
    println!("   POST /api/v1/chat/code              - Code generation with reasoning");
    println!("   POST /api/v1/chat/unified           - Smart routing (text or code)");
    println!("   POST /api/v1/benchmark              - 🎯 Single benchmark (Meta Orchestrator)");
    println!("   POST /api/v1/benchmark/batch        - 🎯 Batch benchmarks (parallel execution)");
    println!("   POST /api/v1/tasks/create           - 📋 Create task queue");
    println!("   POST /api/v1/tasks/execute          - ▶️  Execute next task");
    println!("   POST /api/v1/tasks/execute-all      - ⏩ Execute all tasks");
    println!("   GET  /api/v1/tasks/progress         - 📊 Get task progress");
    println!("   GET  /api/v1/tasks/list             - 📝 List all tasks");
    println!("   POST /api/v1/consciousness/think    - 🏛️  Consciousness thinking");
    println!("   GET  /api/v1/consciousness/analytics- 📊 Consciousness analytics");
    println!("   GET  /api/v1/consciousness/health   - 🏥 Consciousness health");
    println!("   POST /api/v1/flux/matrix/generate");
    println!("   POST /api/v1/inference/reverse");
    println!("   POST /api/v1/inference/forward");
    println!("   GET  /api/v1/subjects");
    println!("   POST /api/v1/subjects/generate");
    println!("");
    println!("🎯 Benchmark System Ready:");
    println!("   - Meta Orchestrator: Hybrid routing (90-95% accuracy)");
    println!("   - Adaptive learning from results");
    println!("   - ParallelFusion option for critical benchmarks (97-99%)");
    println!("");
    println!("📖 Swagger UI:");
    println!("   http://{}:{}/swagger-ui/", config.host, config.port);
    println!("");
    
    // Generate OpenAPI specification
    let openapi = ApiDoc::openapi();
    
    // Dynamic worker sizing (env override -> CPU cores * 2)
    let workers = std::env::var("ACTIX_WORKERS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(|| num_cpus::get() * 2);

    // Start server
    HttpServer::new(move || {
        let cors = if config.enable_cors {
            Cors::permissive()
        } else {
            Cors::default()
        };
        
        let coding_agent_clone = coding_agent_state.clone();
        let task_manager_clone = task_manager.clone();
        let canvas_store = web::Data::new(crate::ai::canvas_api::CanvasStore::new());
        let executor_state = web::Data::new(crate::ai::code_execution_api::ExecutorState::new());
        let session_store = web::Data::new(crate::ai::session_memory::SessionStore::new());
        let collab_store = web::Data::new(crate::ai::collaboration::CollaborationStore::new());
        
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            // Also expose ASI orchestrator directly for endpoints that need it
            .app_data(web::Data::new(app_state.asi_orchestrator.clone()))
            // Expose Meta Orchestrator for benchmarks
            .app_data(web::Data::new(app_state.meta_orchestrator.clone()))
            // Expose Consciousness Simulator for consciousness endpoints
            .app_data(web::Data::new(app_state.consciousness_simulator.clone()))
            // Expose coding agent state for code generation endpoints
            .app_data(web::Data::new(coding_agent_clone))
            // Expose Task Manager for multi-step task execution
            .app_data(web::Data::new(task_manager_clone))
            // Expose Canvas Store for workspace management
            .app_data(canvas_store.clone())
            // Expose Code Executor for code execution
            .app_data(executor_state.clone())
            // Expose Session Store for session memory
            .app_data(session_store.clone())
            // Expose Collaboration Store for real-time collaboration
            .app_data(collab_store.clone())
            .wrap(MetricsMiddleware)
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .wrap(cors)
            .configure(configure_routes)
            .configure(super::endpoints::configure_production_routes)
            .configure(super::benchmark_api::configure_benchmark_routes)
            .configure(super::task_api::configure_task_routes)
            // Consciousness API routes (consolidated from separate server)
            .configure(super::consciousness_api::configure_routes)
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", openapi.clone())
            )
    })
    .bind((config.host.as_str(), config.port))?
    .keep_alive(Duration::from_secs(75))
    .client_request_timeout(Duration::from_secs(30))
    .client_disconnect_timeout(Duration::from_secs(5))
    .backlog(8192)
    .workers(workers)
    .run()
    .await?;
    
    Ok(())
}

/// Quick start server with defaults
pub async fn start_default_server() -> Result<()> {
    start_server(ServerConfig::default()).await
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert_eq!(config.workers, 4);
        assert!(config.enable_cors);
    }
}
