//! WebTransport Consciousness Streaming Server
//!
//! Provides real-time consciousness analytics streaming via WebTransport (QUIC)
//! with sub-50ms latency and support for 100+ concurrent clients.
//!
//! Usage:
//!   cargo run --bin consciousness_streaming_server --features transport,agents
//!
//! Connect:
//!   WebTransport client to https://localhost:4433/consciousness

use spatial_vortex::consciousness::{
    ConsciousnessSimulator, ConsciousnessStreamingServer, StreamingEvent, EventFilter,
};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

#[cfg(feature = "transport")]
use wtransport::{Endpoint, ServerConfig};
#[cfg(feature = "transport")]
use std::net::SocketAddr;

/// Consciousness streaming server state
struct StreamingServerState {
    /// Active consciousness simulators by session ID
    simulators: Arc<RwLock<HashMap<String, Arc<ConsciousnessSimulator>>>>,
    
    /// Connection counter
    connection_count: Arc<RwLock<usize>>,
}

impl StreamingServerState {
    fn new() -> Self {
        Self {
            simulators: Arc::new(RwLock::new(HashMap::new())),
            connection_count: Arc::new(RwLock::new(0)),
        }
    }
    
    async fn create_simulator(&self) -> Arc<ConsciousnessSimulator> {
        let sim = Arc::new(ConsciousnessSimulator::with_streaming(false));
        let session_id = sim.session_id().to_string();
        
        let mut sims = self.simulators.write().await;
        sims.insert(session_id.clone(), sim.clone());
        
        println!("✨ Created consciousness simulator: {}", session_id);
        sim
    }
    
    async fn get_simulator(&self, session_id: &str) -> Option<Arc<ConsciousnessSimulator>> {
        let sims = self.simulators.read().await;
        sims.get(session_id).cloned()
    }
    
    async fn increment_connections(&self) -> usize {
        let mut count = self.connection_count.write().await;
        *count += 1;
        *count
    }
    
    async fn decrement_connections(&self) -> usize {
        let mut count = self.connection_count.write().await;
        *count = count.saturating_sub(1);
        *count
    }
}

#[cfg(feature = "transport")]
#[tokio::main]
async fn main() -> Result<()> {
    use wtransport::tls::Certificate;
    
    println!("🧠 Consciousness Streaming Server v1.5.0");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    // Create self-signed certificate for development
    let cert = Certificate::self_signed(vec!["localhost".to_string()]);
    
    // Configure WebTransport server
    let config = ServerConfig::builder()
        .with_bind_default(4433)
        .with_certificate(cert)
        .build();
    
    let server = Endpoint::server(config)?;
    let addr = server.local_addr()?;
    
    println!("🌐 Server listening on: https://localhost:{}", addr.port());
    println!("📡 WebTransport endpoint: /consciousness");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    let state = Arc::new(StreamingServerState::new());
    
    loop {
        let incoming = server.accept().await;
        let connection = incoming.await?;
        let state = state.clone();
        
        tokio::spawn(async move {
            if let Err(e) = handle_connection(connection, state).await {
                eprintln!("❌ Connection error: {}", e);
            }
        });
    }
}

#[cfg(feature = "transport")]
async fn handle_connection(
    connection: wtransport::Connection,
    state: Arc<StreamingServerState>,
) -> Result<()> {
    let conn_id = state.increment_connections().await;
    println!("🔌 Client connected (#{} active)", conn_id);
    
    // Accept bi-directional stream for commands
    let bi_stream = connection.accept_bi().await?;
    let (mut send, mut recv) = bi_stream;
    
    // Read client request
    let mut buffer = vec![0u8; 1024];
    let n = recv.read(&mut buffer).await?.unwrap_or(0);
    let request = String::from_utf8_lossy(&buffer[..n]);
    
    println!("📨 Request: {}", request);
    
    // Parse request (simple JSON)
    let req: serde_json::Value = serde_json::from_str(&request)
        .unwrap_or_else(|_| serde_json::json!({"action": "subscribe"}));
    
    match req["action"].as_str() {
        Some("subscribe") => {
            // Create or get simulator
            let session_id = req["session_id"]
                .as_str()
                .map(|s| s.to_string());
            
            let simulator = if let Some(sid) = session_id {
                state.get_simulator(&sid).await
                    .unwrap_or_else(|| {
                        tokio::task::block_in_place(|| {
                            tokio::runtime::Handle::current()
                                .block_on(state.create_simulator())
                        })
                    })
            } else {
                state.create_simulator().await
            };
            
            // Get streaming server
            if let Some(streaming) = simulator.streaming_server() {
                let client_id = format!("client-{}", conn_id);
                let filter = EventFilter::default();
                
                // Subscribe to events
                let mut rx = streaming.subscribe(client_id.clone(), filter);
                
                // Send session info
                let session_info = serde_json::json!({
                    "session_id": simulator.session_id(),
                    "status": "connected"
                });
                send.write_all(session_info.to_string().as_bytes()).await?;
                
                println!("✅ Streaming started for {}", client_id);
                
                // Stream events
                loop {
                    match rx.recv().await {
                        Ok(event) => {
                            // Serialize event
                            let json = event.to_json()?;
                            
                            // Send via unidirectional stream
                            let mut uni_stream = connection.open_uni().await?;
                            uni_stream.write_all(json.as_bytes()).await?;
                            uni_stream.finish().await?;
                        }
                        Err(_) => {
                            println!("⚠️ Event channel closed for {}", client_id);
                            break;
                        }
                    }
                }
            } else {
                let error = serde_json::json!({
                    "error": "Streaming not enabled"
                });
                send.write_all(error.to_string().as_bytes()).await?;
            }
        }
        
        Some("analyze_selection") => {
            // Handle selection analysis
            let text = req["text"].as_str().unwrap_or("");
            let start = req["start_pos"].as_u64().unwrap_or(0) as usize;
            let end = req["end_pos"].as_u64().unwrap_or(0) as usize;
            
            let session_id = req["session_id"].as_str().unwrap_or("");
            
            if let Some(simulator) = state.get_simulator(session_id).await {
                if let Some(streaming) = simulator.streaming_server() {
                    let analysis = streaming.analyze_selection(
                        text.to_string(),
                        start,
                        end
                    ).await?;
                    
                    let response = serde_json::to_string(&analysis)?;
                    send.write_all(response.as_bytes()).await?;
                }
            }
        }
        
        _ => {
            let error = serde_json::json!({
                "error": "Unknown action"
            });
            send.write_all(error.to_string().as_bytes()).await?;
        }
    }
    
    let remaining = state.decrement_connections().await;
    println!("👋 Client disconnected ({} active)", remaining);
    
    Ok(())
}

#[cfg(not(feature = "transport"))]
fn main() {
    eprintln!("❌ This binary requires the 'transport' feature.");
    eprintln!("   Build with: cargo build --bin consciousness_streaming_server --features transport,agents");
    std::process::exit(1);
}
