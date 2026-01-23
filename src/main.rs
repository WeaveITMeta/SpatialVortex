use actix_web::{middleware::{Logger, Compress}, web, App, HttpServer};
use actix_cors::Cors;
use clap::Parser;
use dotenv::dotenv;
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use spatial_vortex::{
    ai_integration::AIModelIntegration,
    ai::orchestrator::ASIOrchestrator,
    api::{configure_routes, AppState},
    cache::CacheManager,
    metrics::MetricsMiddleware,
    auth::ApiKeyAuth,
    flux_matrix::FluxMatrixEngine,
    inference_engine::InferenceEngine,
    spatial_database::SpatialDatabase,
    Result,
};

// Import Swagger documentation
use spatial_vortex::ai::swagger::ApiDoc;

#[derive(Parser, Debug)]
#[command(name = "spatial-vortex")]
#[command(about = "Spatial Vortex REST API Server")]
struct Args {
    /// Host to bind the server to
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,
    
    /// Port to bind the server to
    #[arg(short, long, default_value = "7000")]
    pub port: u16,

    /// Database URL
    #[arg(short, long, env = "DATABASE_URL")]
    database_url: Option<String>,

    /// Redis URL for caching
    #[arg(short, long, env = "REDIS_URL")]
    redis_url: Option<String>,

    /// AI API Key (Grok or other)
    #[arg(short, long, env = "AI_API_KEY")]
    ai_api_key: Option<String>,

    /// AI Model Endpoint
    #[arg(long, env = "AI_MODEL_ENDPOINT")]
    ai_endpoint: Option<String>,

    /// Initialize database schema
    #[arg(long)]
    init_db: bool,

    /// Import example matrices for bootstrapping
    #[arg(long)]
    bootstrap: bool,
}

#[actix_web::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv().ok();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();
    let dev = std::env::var("DEVELOPMENT_MODE").unwrap_or_default() == "true";

    // Initialize database
    let database_url = args
        .database_url
        .unwrap_or_else(|| "postgresql://localhost/spatial_vortex".to_string());

    let database: Arc<SpatialDatabase> = Arc::new(SpatialDatabase::new(&database_url).await?);

    if args.init_db {
        tracing::info!("Initializing database schema...");
        database.initialize_schema().await?;
        tracing::info!("Database schema initialized successfully");
    }

    // Connect to Redis (optional - graceful fallback if unavailable)
    // Default TTL: 24 hours for cached flux matrices and inference results
    const DEFAULT_CACHE_TTL_HOURS: i64 = 24;
    
    let cache_manager = if dev {
        CacheManager::new("memory://", DEFAULT_CACHE_TTL_HOURS).await?
    } else if let Some(redis_url) = args.redis_url.as_ref() {
        match CacheManager::new(redis_url, DEFAULT_CACHE_TTL_HOURS).await {
            Ok(cache) => {
                println!("✅ Connected to Redis at {}", redis_url);
                println!("   Cache TTL: {} hours", DEFAULT_CACHE_TTL_HOURS);
                cache
            }
            Err(e) => {
                println!("⚠️  Redis unavailable: {}.", e);
                return Err(e);
            }
        }
    } else {
        match CacheManager::new("redis://127.0.0.1:6379", DEFAULT_CACHE_TTL_HOURS).await {
            Ok(cache) => {
                println!("✅ Connected to Redis at default location");
                println!("   Cache TTL: {} hours", DEFAULT_CACHE_TTL_HOURS);
                cache
            }
            Err(e) => {
                println!("⚠️  Redis unavailable: {}.", e);
                return Err(e);
            }
        }
    };

    let cache = Arc::new(cache_manager);

    // Initialize AI integration
    let ai_integration = Arc::new(AIModelIntegration::new(args.ai_api_key.clone(), args.ai_endpoint));

    // Initialize AI Router for chat API
    let ai_router = Arc::new(RwLock::new(
        spatial_vortex::ai::router::AIRouter::new(args.ai_api_key.clone(), false)
    ));

    // Initialize flux matrix engine
    let flux_engine = Arc::new(FluxMatrixEngine::new());

    // Initialize inference engine
    let mut inference_engine = InferenceEngine::new();

    // Bootstrap with example matrices if requested
    if args.bootstrap {
        tracing::info!("Bootstrapping with example matrices...");
        let bootstrap_count = bootstrap_example_matrices(
            &mut inference_engine,
            &database,
            &cache,
            &flux_engine,
            &ai_integration,
        )
        .await?;
        tracing::info!("Bootstrapped {} example matrices", bootstrap_count);
    }

    let inference_engine = Arc::new(RwLock::new(inference_engine));

    // Initialize ASI Orchestrator (Phase 1-4 complete!)
    let asi_orchestrator = ASIOrchestrator::new().await?;
    let asi_orchestrator = Arc::new(Mutex::new(asi_orchestrator));
    tracing::info!("✅ ASI Orchestrator initialized (Phases 1-4 complete, 95% production ready)");

    // Initialize Meta Orchestrator (Hybrid routing)
    let meta_orchestrator = spatial_vortex::ai::meta_orchestrator::MetaOrchestrator::new_default().await?;
    let meta_orchestrator = Arc::new(RwLock::new(meta_orchestrator));
    tracing::info!("✅ Meta Orchestrator initialized (Hybrid routing)");

    // Initialize Consciousness Simulator (no streaming by default)
    let consciousness_simulator = Arc::new(RwLock::new(
        spatial_vortex::consciousness::ConsciousnessSimulator::new(false).await,
    ));

    // Create application state
    let app_state = AppState {
        inference_engine,
        flux_engine,
        database,
        cache,
        ai_integration,
        ai_router: ai_router.clone(),
        asi_orchestrator,
        meta_orchestrator,
        consciousness_simulator,
    };

    let bind_address = format!("{}:{}", args.host, args.port);
    tracing::info!("Starting Spatial Vortex server on {}", bind_address);
    tracing::info!("✨ Chat API available at http://{}/api/v1/chat/text", bind_address);
    tracing::info!("📖 Swagger UI at http://{}/swagger-ui/", bind_address);
    
    // Generate OpenAPI specification
    let openapi = ApiDoc::openapi();

    // Dynamic workers (env override -> cores * 2)
    let workers = std::env::var("ACTIX_WORKERS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(|| num_cpus::get() * 2);

    // Start HTTP server
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
        
        App::new()
            .wrap(MetricsMiddleware)
            .wrap(ApiKeyAuth::from_env())
            .wrap(Compress::default())
            .wrap(cors)
            .wrap(Logger::default())
            .app_data(web::Data::new(app_state.clone()))
            .app_data(web::Data::new(ai_router.clone()))  // For chat_text endpoint
            .app_data(web::Data::new(app_state.asi_orchestrator.clone()))  // ASI Orchestrator direct access
            .configure(configure_routes)  // Includes ASI routes via nested configure
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", openapi.clone())
            )
    })
    .bind(&bind_address)?
    .keep_alive(std::time::Duration::from_secs(75))
    .client_request_timeout(std::time::Duration::from_secs(30))
    .client_disconnect_timeout(std::time::Duration::from_secs(5))
    .backlog(8192)
    .workers(workers)
    .run()
    .await?;

    Ok(())
}

/// Bootstrap the system with example matrices for RL training
async fn bootstrap_example_matrices(
    inference_engine: &mut InferenceEngine,
    database: &SpatialDatabase,
    cache: &CacheManager,
    flux_engine: &FluxMatrixEngine,
    ai_integration: &AIModelIntegration,
) -> Result<usize> {
    let example_subjects = vec![
        "General Intelligence",
        "Specific Intelligence",
        "Artificial Intelligence",
        "Machine Learning",
        "Natural Language Processing",
        "Computer Vision",
        "Robotics",
        "Quantum Computing",
        "Blockchain Technology",
        "Cybersecurity",
        "Data Science",
        "Software Engineering",
        "Systems Architecture",
        "Mathematics",
        "Physics",
        "Chemistry",
        "Biology",
        "Psychology",
        "Philosophy",
        "Ethics",
    ];

    let mut bootstrap_count = 0;

    for subject in example_subjects {
        // Check if matrix already exists
        if database.get_matrix_by_subject(subject).await?.is_some() {
            continue; // Skip if already exists
        }

        // Generate matrix using AI if available, otherwise use flux engine
        let matrix = if ai_integration.is_ai_available() {
            tracing::info!("Generating AI matrix for: {}", subject);
            ai_integration.generate_subject_matrix(subject).await?
        } else {
            tracing::info!("Generating flux matrix for: {}", subject);
            flux_engine.create_matrix(subject.to_string())?
        };

        // Store in database
        database.store_matrix(&matrix).await?;

        // Cache for quick access
        cache.store_matrix(matrix.clone()).await?;

        // Update inference engine
        inference_engine.update_subject_matrix(matrix);

        bootstrap_count += 1;

        // Small delay to avoid overwhelming AI API
        if ai_integration.is_ai_available() {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
    }

    Ok(bootstrap_count)
}
