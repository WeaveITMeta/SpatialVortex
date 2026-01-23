//! SpatialVortex API Server Binary
//!
//! Production-ready REST API server for SpatialVortex
//! Provides endpoints for:
//! - Flux matrix generation
//! - ONNX/ASI inference
//! - Confidence Lake storage
//! - Voice pipeline integration
//! - Sacred geometry analysis

use spatial_vortex::ai::server::{start_server, ServerConfig};
use anyhow::Result;

#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
async fn main() -> Result<()> {
    // Load .env file if it exists
    dotenv::dotenv().ok();
    
    // Initialize logger
    env_logger::init();
    
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║         SpatialVortex Production API Server             ║");
    println!("║                                                          ║");
    println!("║  Sacred Geometry · ONNX Inference · Confidence Lake     ║");
    println!("║  Voice Pipeline · Flux Matrix · ASI Integration         ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();
    
    // Initialize Whisper model (if voice feature is enabled)
    #[cfg(feature = "voice")]
    {
        println!("🎤 Initializing Whisper speech-to-text model...");
        if let Err(e) = spatial_vortex::ai::whisper_api::initialize_whisper().await {
            eprintln!("⚠️  Warning: Failed to load Whisper model: {}", e);
            eprintln!("   Voice transcription will not be available.");
            eprintln!("   Download model from: https://huggingface.co/ggerganov/whisper.cpp");
        }
    }
    
    // Configure server from environment or use defaults
    let config = ServerConfig {
        host: std::env::var("API_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
        port: std::env::var("API_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(7000),
        workers: std::env::var("API_WORKERS")
            .ok()
            .and_then(|w| w.parse().ok())
            .unwrap_or(4),
        enable_cors: std::env::var("API_CORS")
            .ok()
            .and_then(|c| c.parse().ok())
            .unwrap_or(true),
    };
    
    // Start server
    start_server(config).await?;
    
    Ok(())
}
