//! High-Performance Serving Module
//!
//! Integrates the highest-performing components from SpatialVortex:
//! - **1200+ RPS API serving** via continuous batching
//! - **MoE gating** for bottleneck resolution
//! - **Optimized ops** (SIMD, BLAS) for 10-100x speedup
//! - **FluxOrchestrator** for unified runtime coordination

pub mod batch_scheduler;
pub mod moe_gate;
pub mod optimized_ops;

pub use batch_scheduler::{ContinuousBatchScheduler, BatchedRequest, BatchConfig};
pub use moe_gate::{MoEGate, Expert, ExpertType, MoEConfig};
pub use optimized_ops::{matmul_fast, softmax_fast, normalize_l2_simd, has_avx2};
