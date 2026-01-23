//! ML Inference Module (2026 Distilled)
//!
//! Minimal, non-redundant inference stack:
//! - **ProductionEngine** - Primary inference with CALM continuous autoregressive
//! - **ort (ONNX Runtime)** - Production CPU/GPU inference
//! - **RoPE + GQA** - Modern attention mechanisms
//! - **Optimized ops** - SIMD/BLAS operations

pub mod tokenizer;
pub mod autoregressive;
pub mod optimized_ops;
pub mod production_engine;
pub mod rope;
pub mod gqa;

// Re-export tokenizer
pub use tokenizer::{TokenizerWrapper, TokenizedInput};

// Autoregressive decoder exports
pub use autoregressive::{
    AutoregressiveDecoder,
    SamplingConfig,
    TokenSampler,
    KVCache,
    BeamSearch,
    StreamingGenerator,
    GenerationStats,
};

// Optimized operations exports
pub use optimized_ops::{
    matmul,
    matvec,
    softmax,
    softmax_2d,
    layer_norm,
    relu,
    gelu,
    silu,
    dot_product,
    scaled_dot_product_attention,
    fused_transformer_block,
    has_avx2,
    has_avx512,
    QuantizedTensor,
    AlignedBuffer,
    OptimizedOpsStats,
};

// Production engine exports (GPU, RoPE, tokenizers, serving)
pub use production_engine::{
    ProductionEngine,
    ProductionConfig,
    ProductionStats,
    Tokenizer,
    BPETokenizer,
    SentencePieceTokenizer,
    TokenizerType,
    SpecialTokens,
    RoPECache,
    DraftModel,
    ContinuousBatchScheduler,
    BatchedRequest,
    DeviceType,
    OffloadConfig,
    ComputeBackend,
    GpuTensor,
};

// Rotary Position Embeddings (RoPE) exports
pub use rope::{
    RotaryPositionEmbedding,
    RoPEConfig,
    ExtendedRoPE,
};

// Grouped Query Attention (GQA) exports
pub use gqa::{
    GroupedQueryAttention,
    GQAConfig,
    GQAKVCache,
};
