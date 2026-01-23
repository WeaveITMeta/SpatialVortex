//! Consciousness API Server - v1.6.0 "Memory Palace"
//!
//! Complete HTTP API server with consciousness simulation,
//! background learning, and Memory Palace persistence.
//!
//! Features:
//! - Consciousness thinking endpoint
//! - Real-time analytics
//! - Background learning
//! - Memory Palace state management
//! - CORS enabled for frontend
//!
//! Run: cargo run --bin consciousness_api_server --features agents,persistence

use actix_web::{middleware, web, App, HttpServer};
use actix_cors::Cors;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;

use spatial_vortex::consciousness::ConsciousnessSimulator;

#[cfg(feature = "agents")]
use spatial_vortex::ai::consciousness_api;

#[actix_web::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    println!("🏛️  Starting Consciousness API Server v1.6.0");
    
    // Initialize consciousness simulator
    println!("🧠 Initializing consciousness simulator...");
    let mut simulator = ConsciousnessSimulator::new(false).await;
    
    // Enable background learning
    #[cfg(feature = "agents")]
    {
        println!("🚀 Enabling background learning...");
        simulator.enable_background_learning().await?;
        println!("   ✓ Background learning active!");
    }
    
    // Wrap in Arc<RwLock> for shared state
    let sim_state = web::Data::new(Arc::new(RwLock::new(simulator)));
    
    println!("📊 Session ID: {}", sim_state.read().await.session_id());
    
    // Server configuration
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(7000);
    
    println!("🌐 Server configuration:");
    println!("   Host: {}", host);
    println!("   Port: {}", port);
    println!("   CORS: Enabled");
    
    #[cfg(feature = "persistence")]
    println!("   💾 Memory Palace: Enabled");
    
    #[cfg(feature = "postgres")]
    println!("   📦 PostgreSQL RAG: Enabled");
    
    #[cfg(feature = "lake")]
    println!("   💎 Confidence Lake: Enabled");
    
    println!("");
    println!("🚀 Starting server on http://{}:{}", host, port);
    println!("");
    println!("📡 API Endpoints:");
    println!("   POST   /api/v1/consciousness/think");
    println!("   GET    /api/v1/consciousness/analytics");
    println!("   GET    /api/v1/consciousness/session");
    println!("   GET    /api/v1/consciousness/learning-stats");
    println!("   POST   /api/v1/consciousness/enable-learning");
    println!("   POST   /api/v1/consciousness/save-state");
    println!("   GET    /api/v1/consciousness/health");
    println!("");
    
    // Start HTTP server
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
        
        let mut app = App::new()
            .app_data(sim_state.clone())
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default());
        
        // Add consciousness API routes
        #[cfg(feature = "agents")]
        {
            app = app.configure(consciousness_api::configure_routes);
        }
        
        app
    })
    .bind((host.as_str(), port))?
    .run()
    .await?;
    
    Ok(())
}
