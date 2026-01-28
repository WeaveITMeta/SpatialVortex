//! High-Performance Serving Module
//!
//! Integrates the highest-performing components from SpatialVortex:
//! - **1200+ RPS API serving** via continuous batching
//! - **MoE gating** for bottleneck resolution
//! - **Optimized ops** (SIMD, BLAS) for 10-100x speedup
//! - **FluxOrchestrator** for unified runtime coordination
//! - **MCP Server** for Model Context Protocol training/inference
//! - **Chat API** for web-based interaction with trained models

pub mod batch_scheduler;
pub mod moe_gate;
pub mod optimized_ops;
pub mod mcp_server;
pub mod chat_api;

pub use batch_scheduler::{ContinuousBatchScheduler, BatchedRequest, BatchConfig};
pub use moe_gate::{MoEGate, Expert, ExpertType, MoEConfig};
pub use optimized_ops::{matmul_fast, softmax_fast, normalize_l2_simd, has_avx2};
pub use mcp_server::{MCPServer, MCPServerConfig, MCPRequest, MCPResponse, MCPTool};
pub use chat_api::{ChatEngineState, ChatRequest, ChatResponse, text_to_beams, generate_response};
#[cfg(feature = "web")]
pub use chat_api::configure_chat_routes;
