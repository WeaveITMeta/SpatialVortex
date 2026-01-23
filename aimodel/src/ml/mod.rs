//! Machine Learning Module (2026 Distilled)
//!
//! Minimal, non-redundant ML stack:
//! - **ProductionEngine** - Primary inference with CALM
//! - **VortexModel** - Unified transformer with sacred geometry
//! - **VCP** - Vortex Context Preserver (hallucination detection)
//! - **EBRM** - Energy-Based Reasoning Model
//! - **RoPE + GQA** - Modern attention mechanisms

pub mod hallucinations;
pub mod ebrm;
pub mod vortex_model;
pub mod production_engine;
pub mod autoregressive;
pub mod rope;
pub mod gqa;
pub mod optimized_ops;
pub mod tokenizer;

// Re-exports
pub use hallucinations::{VortexContextPreserver, SignalSubspace, HallucinationDetector};
pub use ebrm::{EnergyBasedReasoningModel, TraceEnergy, PositionEnergy};
pub use vortex_model::{VortexModel, VortexModelConfig};
pub use production_engine::{ProductionEngine, ProductionConfig};
pub use rope::{RotaryPositionEmbedding, RoPEConfig};
pub use gqa::{GroupedQueryAttention, GQAConfig};
