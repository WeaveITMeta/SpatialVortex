//! WebTransport Server
//!
//! Stream Bevy scene deltas + flux updates via QUIC/WebTransport.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// WebTransport server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WTransportConfig {
    /// Server bind address
    pub bind_addr: String,
    /// Server port
    pub port: u16,
    /// Enable TLS
    pub tls_enabled: bool,
    /// Certificate path (if TLS enabled)
    pub cert_path: Option<String>,
    /// Key path (if TLS enabled)
    pub key_path: Option<String>,
    /// Max concurrent streams
    pub max_streams: usize,
    /// Keepalive interval in seconds
    pub keepalive_secs: u64,
}

impl Default for WTransportConfig {
    fn default() -> Self {
        Self {
            bind_addr: "0.0.0.0".to_string(),
            port: 4433,
            tls_enabled: true,
            cert_path: None,
            key_path: None,
            max_streams: 100,
            keepalive_secs: 30,
        }
    }
}

impl WTransportConfig {
    pub fn new(port: u16) -> Self {
        Self { port, ..Default::default() }
    }

    pub fn with_tls(mut self, cert: &str, key: &str) -> Self {
        self.tls_enabled = true;
        self.cert_path = Some(cert.to_string());
        self.key_path = Some(key.to_string());
        self
    }
}

/// Message types for flux streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FluxMessage {
    /// Beam tensor update
    BeamUpdate {
        session_id: String,
        position: u8,
        digits: [f32; 9],
        confidence: f32,
    },
    /// Position change in vortex cycle
    PositionChange {
        session_id: String,
        from: u8,
        to: u8,
        is_sacred: bool,
    },
    /// Consensus result
    ConsensusResult {
        session_id: String,
        content: String,
        confidence: f32,
        provider_count: usize,
    },
    /// Scene delta for Bevy visualization
    SceneDelta {
        session_id: String,
        delta_type: SceneDeltaType,
        data: Vec<u8>,
    },
    /// Heartbeat/keepalive
    Heartbeat {
        timestamp: i64,
    },
    /// Error message
    Error {
        code: u32,
        message: String,
    },
}

/// Types of scene deltas for Bevy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SceneDeltaType {
    /// Orbit position update
    OrbitUpdate,
    /// Color change (ELP-based)
    ColorUpdate,
    /// New node added
    NodeAdd,
    /// Node removed
    NodeRemove,
    /// Connection change
    ConnectionUpdate,
    /// Full scene refresh
    FullRefresh,
}

/// Client session state
#[derive(Debug, Clone)]
pub struct ClientSession {
    pub id: String,
    pub connected_at: i64,
    pub last_activity: i64,
    pub current_position: u8,
    pub subscriptions: Vec<String>,
}

impl ClientSession {
    pub fn new(id: String) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id,
            connected_at: now,
            last_activity: now,
            current_position: 1,
            subscriptions: Vec::new(),
        }
    }

    pub fn touch(&mut self) {
        self.last_activity = chrono::Utc::now().timestamp();
    }

    pub fn subscribe(&mut self, topic: &str) {
        if !self.subscriptions.contains(&topic.to_string()) {
            self.subscriptions.push(topic.to_string());
        }
    }

    pub fn unsubscribe(&mut self, topic: &str) {
        self.subscriptions.retain(|t| t != topic);
    }
}

/// WebTransport server (simplified - actual impl would use wtransport crate)
#[derive(Debug)]
pub struct WTransportServer {
    config: WTransportConfig,
    sessions: HashMap<String, ClientSession>,
    message_queue: Vec<(String, FluxMessage)>,
}

impl WTransportServer {
    pub fn new(config: WTransportConfig) -> Self {
        Self {
            config,
            sessions: HashMap::new(),
            message_queue: Vec::new(),
        }
    }

    /// Start the server (would be async in real impl)
    pub fn start(&self) -> Result<(), String> {
        // In production:
        // let endpoint = wtransport::Endpoint::server(config)?;
        // endpoint.accept().await?;
        println!("WTransport server starting on {}:{}", self.config.bind_addr, self.config.port);
        Ok(())
    }

    /// Register a new client session
    pub fn register_session(&mut self, session_id: &str) -> &ClientSession {
        self.sessions.entry(session_id.to_string())
            .or_insert_with(|| ClientSession::new(session_id.to_string()))
    }

    /// Remove a client session
    pub fn remove_session(&mut self, session_id: &str) {
        self.sessions.remove(session_id);
    }

    /// Get session by ID
    pub fn get_session(&self, session_id: &str) -> Option<&ClientSession> {
        self.sessions.get(session_id)
    }

    /// Get mutable session
    pub fn get_session_mut(&mut self, session_id: &str) -> Option<&mut ClientSession> {
        self.sessions.get_mut(session_id)
    }

    /// Queue a message for a session
    pub fn send(&mut self, session_id: &str, message: FluxMessage) {
        self.message_queue.push((session_id.to_string(), message));
    }

    /// Broadcast to all sessions
    pub fn broadcast(&mut self, message: FluxMessage) {
        for session_id in self.sessions.keys().cloned().collect::<Vec<_>>() {
            self.message_queue.push((session_id, message.clone()));
        }
    }

    /// Broadcast to sessions subscribed to a topic
    pub fn broadcast_topic(&mut self, topic: &str, message: FluxMessage) {
        for (session_id, session) in &self.sessions {
            if session.subscriptions.contains(&topic.to_string()) {
                self.message_queue.push((session_id.clone(), message.clone()));
            }
        }
    }

    /// Drain message queue (would send over network in real impl)
    pub fn flush(&mut self) -> Vec<(String, FluxMessage)> {
        std::mem::take(&mut self.message_queue)
    }

    /// Send heartbeat to all sessions
    pub fn heartbeat(&mut self) {
        let msg = FluxMessage::Heartbeat {
            timestamp: chrono::Utc::now().timestamp(),
        };
        self.broadcast(msg);
    }

    /// Get active session count
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Clean up stale sessions
    pub fn cleanup_stale(&mut self, max_idle_secs: i64) {
        let now = chrono::Utc::now().timestamp();
        self.sessions.retain(|_, s| now - s.last_activity < max_idle_secs);
    }
}

/// Helper to create beam update message
pub fn beam_update_msg(session_id: &str, position: u8, digits: [f32; 9], confidence: f32) -> FluxMessage {
    FluxMessage::BeamUpdate {
        session_id: session_id.to_string(),
        position,
        digits,
        confidence,
    }
}

/// Helper to create position change message
pub fn position_change_msg(session_id: &str, from: u8, to: u8) -> FluxMessage {
    FluxMessage::PositionChange {
        session_id: session_id.to_string(),
        from,
        to,
        is_sacred: matches!(to, 3 | 6 | 9),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_sessions() {
        let config = WTransportConfig::new(4433);
        let mut server = WTransportServer::new(config);

        server.register_session("client_1");
        server.register_session("client_2");

        assert_eq!(server.session_count(), 2);

        server.remove_session("client_1");
        assert_eq!(server.session_count(), 1);
    }

    #[test]
    fn test_message_broadcast() {
        let mut server = WTransportServer::new(WTransportConfig::default());

        server.register_session("c1");
        server.register_session("c2");

        server.broadcast(FluxMessage::Heartbeat { timestamp: 0 });

        let messages = server.flush();
        assert_eq!(messages.len(), 2);
    }

    #[test]
    fn test_topic_subscription() {
        let mut server = WTransportServer::new(WTransportConfig::default());

        server.register_session("c1");
        server.register_session("c2");

        if let Some(s) = server.get_session_mut("c1") {
            s.subscribe("flux_updates");
        }

        server.broadcast_topic("flux_updates", FluxMessage::Heartbeat { timestamp: 0 });

        let messages = server.flush();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].0, "c1");
    }
}
