//! WebTransport Server Binary
//!
//! Runs a standalone WebTransport (QUIC) server for chat interface
//!
//! Usage:
//!   cargo run --features transport --bin webtransport_server

use spatial_vortex::transport::{WebTransportServer, WebTransportConfig};
use std::net::SocketAddr;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();
    
    tracing::info!("🚀 SpatialVortex WebTransport Server");
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    // Load configuration from environment or use defaults
    let config = WebTransportConfig {
        bind_address: std::env::var("WEBTRANSPORT_BIND")
            .unwrap_or_else(|_| "0.0.0.0:4433".to_string())
            .parse()?,
        cert_path: std::env::var("WEBTRANSPORT_CERT")
            .unwrap_or_else(|_| "certs/cert.pem".to_string())
            .into(),
        key_path: std::env::var("WEBTRANSPORT_KEY")
            .unwrap_or_else(|_| "certs/key.pem".to_string())
            .into(),
        max_connections: std::env::var("MAX_CONNECTIONS")
            .unwrap_or_else(|_| "2000".to_string())
            .parse()?,
        max_streams_per_connection: std::env::var("MAX_STREAMS")
            .unwrap_or_else(|_| "100".to_string())
            .parse()?,
        keep_alive_interval: 30,
    };
    
    tracing::info!("📋 Configuration:");
    tracing::info!("   Bind address: {}", config.bind_address);
    tracing::info!("   Certificate: {}", config.cert_path.display());
    tracing::info!("   Private key: {}", config.key_path.display());
    tracing::info!("   Max connections: {}", config.max_connections);
    tracing::info!("   Max streams/conn: {}", config.max_streams_per_connection);
    
    // Verify TLS files exist
    if !config.cert_path.exists() {
        tracing::error!("❌ Certificate file not found: {}", config.cert_path.display());
        tracing::error!("   Run: powershell scripts/generate_tls_certs.ps1");
        return Err("Missing TLS certificate".into());
    }
    
    if !config.key_path.exists() {
        tracing::error!("❌ Private key file not found: {}", config.key_path.display());
        tracing::error!("   Run: powershell scripts/generate_tls_certs.ps1");
        return Err("Missing TLS private key".into());
    }
    
    tracing::info!("✅ TLS certificates found");
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    // Create and initialize server
    let mut server = WebTransportServer::new(config)?;
    server.initialize().await?;
    
    tracing::info!("✅ Server initialized");
    tracing::info!("🌐 Protocol: HTTP/3 + QUIC (UDP-based)");
    tracing::info!("🔒 Encryption: TLS 1.3 (built-in)");
    tracing::info!("📡 Listening for WebTransport connections...");
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    // Run server (blocks until shutdown)
    server.run().await?;
    
    Ok(())
}
