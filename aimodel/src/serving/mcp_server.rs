//! MCP Training Server
//!
//! Model Context Protocol server for training and inference.
//! Enables external tools to interact with the VortexRunner for continuous learning.
//!
//! ## Capabilities
//! - Model loading from HuggingFace
//! - RSI (Recursive Self-Improvement) adaptation
//! - Training loop with entropic objective
//! - Inference with pathway optimization

use crate::cognition::{VortexRunner, VortexState};
use crate::ml::pathway::{ExhaustivePathwayOptimizer, PathwayConfig, StackedResult};
use crate::data::models::BeamTensor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// MCP Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPServerConfig {
    /// Server name
    pub name: String,
    /// Server version
    pub version: String,
    /// Host address
    pub host: String,
    /// Port number
    pub port: u16,
    /// Enable training mode
    pub training_enabled: bool,
    /// Max concurrent requests
    pub max_concurrent: usize,
    /// HuggingFace model ID for loading
    pub hf_model_id: Option<String>,
}

impl Default for MCPServerConfig {
    fn default() -> Self {
        Self {
            name: "vortex-mcp".to_string(),
            version: "0.1.0".to_string(),
            host: "127.0.0.1".to_string(),
            port: 8765,
            training_enabled: true,
            max_concurrent: 100,
            hf_model_id: None,
        }
    }
}

/// MCP Request types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method")]
pub enum MCPRequest {
    /// Initialize connection
    #[serde(rename = "initialize")]
    Initialize { client_info: ClientInfo },
    
    /// List available tools
    #[serde(rename = "tools/list")]
    ListTools,
    
    /// Call a tool
    #[serde(rename = "tools/call")]
    CallTool { name: String, arguments: serde_json::Value },
    
    /// Get server capabilities
    #[serde(rename = "capabilities")]
    Capabilities,
    
    /// Training request
    #[serde(rename = "train")]
    Train { input: String, target: Option<String>, epochs: usize },
    
    /// Inference request
    #[serde(rename = "infer")]
    Infer { input: String, max_cycles: Option<u64> },
    
    /// Get current state
    #[serde(rename = "state")]
    GetState,
    
    /// Optimize pathways
    #[serde(rename = "optimize")]
    Optimize { num_stacks: usize },
}

/// Client information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

/// MCP Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPResponse {
    pub success: bool,
    pub data: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl MCPResponse {
    pub fn success(data: impl Serialize) -> Self {
        Self {
            success: true,
            data: serde_json::to_value(data).unwrap_or(serde_json::Value::Null),
            error: None,
        }
    }

    pub fn error(msg: &str) -> Self {
        Self {
            success: false,
            data: serde_json::Value::Null,
            error: Some(msg.to_string()),
        }
    }
}

/// Tool definition for MCP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// Server capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    pub tools: bool,
    pub training: bool,
    pub inference: bool,
    pub pathway_optimization: bool,
    pub huggingface_loading: bool,
}

/// MCP Training Server
pub struct MCPServer {
    config: MCPServerConfig,
    vortex: Arc<VortexRunner>,
    tools: Vec<MCPTool>,
    sessions: Arc<RwLock<HashMap<String, SessionState>>>,
}

/// Session state for connected clients
#[derive(Debug, Clone)]
pub struct SessionState {
    pub client_info: ClientInfo,
    pub connected_at: i64,
    pub request_count: u64,
    pub last_request: i64,
}

impl MCPServer {
    pub fn new(config: MCPServerConfig) -> Self {
        let vortex = Arc::new(VortexRunner::new());
        
        let tools = vec![
            MCPTool {
                name: "think".to_string(),
                description: "Process input through vortex thinking cycles".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "input": { "type": "string", "description": "Input text to process" },
                        "cycles": { "type": "integer", "description": "Number of vortex cycles" }
                    },
                    "required": ["input"]
                }),
            },
            MCPTool {
                name: "learn".to_string(),
                description: "Learn from input content".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "content": { "type": "string", "description": "Content to learn from" },
                        "source": { "type": "string", "description": "Source type (rag, tool, internet, user)" }
                    },
                    "required": ["content"]
                }),
            },
            MCPTool {
                name: "optimize_pathways".to_string(),
                description: "Run exhaustive pathway optimization with entropic objective".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "num_stacks": { "type": "integer", "description": "Number of stacked inference runs" }
                    }
                }),
            },
            MCPTool {
                name: "add_subject".to_string(),
                description: "Add a new subject for learning".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": { "type": "string", "description": "Subject name" }
                    },
                    "required": ["name"]
                }),
            },
            MCPTool {
                name: "get_state".to_string(),
                description: "Get current vortex state".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
        ];

        Self {
            config,
            vortex,
            tools,
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get server capabilities
    pub fn capabilities(&self) -> ServerCapabilities {
        ServerCapabilities {
            tools: true,
            training: self.config.training_enabled,
            inference: true,
            pathway_optimization: true,
            huggingface_loading: self.config.hf_model_id.is_some(),
        }
    }

    /// Handle MCP request
    pub async fn handle_request(&self, request: MCPRequest) -> MCPResponse {
        match request {
            MCPRequest::Initialize { client_info } => {
                let session_id = uuid::Uuid::new_v4().to_string();
                let session = SessionState {
                    client_info: client_info.clone(),
                    connected_at: chrono::Utc::now().timestamp(),
                    request_count: 0,
                    last_request: chrono::Utc::now().timestamp(),
                };
                
                let mut sessions = self.sessions.write().await;
                sessions.insert(session_id.clone(), session);
                
                MCPResponse::success(serde_json::json!({
                    "session_id": session_id,
                    "server_info": {
                        "name": self.config.name,
                        "version": self.config.version
                    },
                    "capabilities": self.capabilities()
                }))
            }
            
            MCPRequest::ListTools => {
                MCPResponse::success(&self.tools)
            }
            
            MCPRequest::CallTool { name, arguments } => {
                self.call_tool(&name, arguments).await
            }
            
            MCPRequest::Capabilities => {
                MCPResponse::success(self.capabilities())
            }
            
            MCPRequest::Train { input, target: _, epochs } => {
                if !self.config.training_enabled {
                    return MCPResponse::error("Training is disabled");
                }
                
                // Run training cycles
                for _ in 0..epochs {
                    self.vortex.learn_from_source(&input, crate::cognition::SourceType::UserInput).await;
                }
                
                let state = self.vortex.state_summary().await;
                MCPResponse::success(serde_json::json!({
                    "epochs_completed": epochs,
                    "state": {
                        "cycle": state.cycle,
                        "energy": state.energy,
                        "alignment": state.sacred_alignment
                    }
                }))
            }
            
            MCPRequest::Infer { input, max_cycles } => {
                let cycles = max_cycles.unwrap_or(64);
                let beams = self.text_to_beams(&input);
                
                // Run inference cycles
                self.vortex.run_n_cycles(cycles).await;
                
                let state = self.vortex.state_summary().await;
                MCPResponse::success(serde_json::json!({
                    "cycles": cycles,
                    "state": {
                        "cycle": state.cycle,
                        "energy": state.energy,
                        "alignment": state.sacred_alignment,
                        "subjects": state.subject_count,
                        "stored_states": state.stored_states
                    }
                }))
            }
            
            MCPRequest::GetState => {
                let state = self.vortex.state_summary().await;
                MCPResponse::success(state)
            }
            
            MCPRequest::Optimize { num_stacks } => {
                let result = self.vortex.optimize_pathways(num_stacks).await;
                MCPResponse::success(serde_json::json!({
                    "total_perms": result.total_perms,
                    "duration_ms": result.total_duration_ms,
                    "entropic_value": result.final_entropic_value,
                    "top_paths": result.top_paths.len()
                }))
            }
        }
    }

    /// Call a specific tool
    async fn call_tool(&self, name: &str, arguments: serde_json::Value) -> MCPResponse {
        match name {
            "think" => {
                let input = arguments.get("input")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let cycles = arguments.get("cycles")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(64);
                
                self.vortex.run_n_cycles(cycles).await;
                let state = self.vortex.state_summary().await;
                
                MCPResponse::success(serde_json::json!({
                    "input": input,
                    "cycles": cycles,
                    "state": state
                }))
            }
            
            "learn" => {
                let content = arguments.get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let source = arguments.get("source")
                    .and_then(|v| v.as_str())
                    .unwrap_or("user");
                
                let source_type = match source {
                    "rag" => crate::cognition::SourceType::RAG,
                    "tool" => crate::cognition::SourceType::Tool,
                    "internet" => crate::cognition::SourceType::Internet,
                    _ => crate::cognition::SourceType::UserInput,
                };
                
                self.vortex.learn_from_source(content, source_type).await;
                let state = self.vortex.state_summary().await;
                
                MCPResponse::success(serde_json::json!({
                    "learned": true,
                    "source_memories": state.source_memories
                }))
            }
            
            "optimize_pathways" => {
                let num_stacks = arguments.get("num_stacks")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(14) as usize;
                
                let result = self.vortex.optimize_pathways(num_stacks).await;
                
                MCPResponse::success(serde_json::json!({
                    "total_perms": result.total_perms,
                    "duration_ms": result.total_duration_ms,
                    "entropic_value": result.final_entropic_value,
                    "stacks": result.stack_stats.len(),
                    "top_paths": result.top_paths.len()
                }))
            }
            
            "add_subject" => {
                let name = arguments.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unnamed");
                
                let subject = self.vortex.add_subject(name).await;
                
                MCPResponse::success(serde_json::json!({
                    "name": subject.name,
                    "nodes": subject.nodes.len(),
                    "sacred_guides": subject.sacred_guides.len()
                }))
            }
            
            "get_state" => {
                let state = self.vortex.state_summary().await;
                MCPResponse::success(state)
            }
            
            _ => MCPResponse::error(&format!("Unknown tool: {}", name)),
        }
    }

    /// Convert text to beams
    fn text_to_beams(&self, text: &str) -> Vec<BeamTensor> {
        let text_lower = text.to_lowercase();
        let words: Vec<&str> = text_lower
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .collect();

        words.iter().map(|word| {
            let mut beam = BeamTensor::default();
            let hash = word.bytes().fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
            for i in 0..9 {
                beam.digits[i] = ((hash >> (i * 7)) & 0x7F) as f32 / 127.0;
            }
            beam
        }).collect()
    }

    /// Get vortex runner reference
    pub fn vortex(&self) -> Arc<VortexRunner> {
        self.vortex.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mcp_server_creation() {
        let config = MCPServerConfig::default();
        let server = MCPServer::new(config);
        
        let caps = server.capabilities();
        assert!(caps.tools);
        assert!(caps.training);
        assert!(caps.inference);
    }

    #[tokio::test]
    async fn test_mcp_initialize() {
        let server = MCPServer::new(MCPServerConfig::default());
        
        let request = MCPRequest::Initialize {
            client_info: ClientInfo {
                name: "test-client".to_string(),
                version: "1.0.0".to_string(),
            },
        };
        
        let response = server.handle_request(request).await;
        assert!(response.success);
    }

    #[tokio::test]
    async fn test_mcp_list_tools() {
        let server = MCPServer::new(MCPServerConfig::default());
        
        let response = server.handle_request(MCPRequest::ListTools).await;
        assert!(response.success);
    }

    #[tokio::test]
    async fn test_mcp_call_tool() {
        let server = MCPServer::new(MCPServerConfig::default());
        
        let response = server.handle_request(MCPRequest::CallTool {
            name: "get_state".to_string(),
            arguments: serde_json::json!({}),
        }).await;
        
        assert!(response.success);
    }
}
