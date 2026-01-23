//! AIModel - Distilled SpatialVortex AGI/ASI Seed
//!
//! Sacred-geometry-centric AI framework with:
//! - **FluxMatrixEngine** - Vortex cycles (1→2→4→8→7→5→1), 3-6-9 anchors
//! - **VCP** - Vortex Context Preserver (hallucination detection)
//! - **EBRM** - Energy-Based Reasoning Model
//! - **ProductionEngine** - High-throughput autoregressive generation
//! - **AIConsensusEngine** - Multi-LLM fusion
//!
//! ## 2026 Distilled Stack
//!
//! - `ort` - ONNX Runtime inference (primary)
//! - `burn` - ML training framework with tch-rs backend
//! - `wtransport` - WebTransport/QUIC networking
//! - `rocksdb` - Hot-path storage
//! - `embedvec` - Vector embeddings
//! - `bevy` - 3D visualization

pub mod error;
pub mod core;
pub mod data;
pub mod ml;
pub mod ai;
pub mod visualization;
pub mod storage;

// Re-exports
pub use error::{Result, AIModelError};
pub use data::models::{BeamTensor, ELPTensor, FluxMatrix};
