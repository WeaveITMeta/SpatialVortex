//! ML Inference Module (2025 Edition)
//!
//! Contains all inference engines and related functionality:
//! - Transformer architecture
//! - ONNX runtime integration (C++ deps)
//! - tract runtime (pure Rust ONNX)
//! - Dynamic context windows
//! - ASI integration
//! - High-performance inference (2025 best practices)
//!
//! ## 2025 Features
//!
//! - **Quantization**: INT8/FP16 for 4x faster inference
//! - **Parallel Batch**: Rayon-based parallel processing
//! - **Zero-Copy**: Memory-efficient tensor operations
//! - **GPU Acceleration**: WGPU/CUDA backend support
//! - **Session Pooling**: Concurrent request handling

pub mod flux_inference;
pub mod onnx_runtime;
pub mod onnx_pool;  // Session pooling for ONNX
pub mod tract_runtime;  // Pure Rust alternative
pub mod tokenizer;
pub mod asi_integration;
pub mod transformer;
pub mod dynamic_context;
pub mod color_inference;  // Color-aware inference
pub mod high_performance;  // 2025 high-performance inference
pub mod autoregressive;  // Autoregressive decoder with sampling
pub mod optimized_ops;  // SIMD/BLAS optimized operations
pub mod ultra_fast;  // Ultimate speed optimizations
pub mod integrated_engine;  // Fully integrated inference pipeline
pub mod production_engine;  // Production-ready with GPU, RoPE, tokenizers
pub mod rope;  // Rotary Position Embeddings (RoPE)
pub mod gqa;   // Grouped Query Attention (GQA)

// Re-export main types
pub use flux_inference::InferenceEngine;
pub use onnx_runtime::OnnxInferenceEngine;
pub use onnx_pool::{OnnxSessionPool, initialize_global_pool, get_global_pool};
pub use tract_runtime::{TractInferenceEngine, TractInferenceStats};  // Recommended for Windows
pub use tokenizer::{TokenizerWrapper, TokenizedInput};
pub use asi_integration::{ASIIntegrationEngine, SemanticBeadTensor, ASIInferenceResult};
pub use color_inference::{
    ColorInferenceEngine,
    ColorInferenceConfig,
    ColorPrediction,
    ColorContext,
    InferenceStats as ColorInferenceStats,
};

// 2025 High-Performance Inference exports
pub use high_performance::{
    HighPerformanceInferenceEngine,
    HighPerformanceConfig,
    QuantizationLevel,
    ExecutionMode,
    BatchProcessor,
    Quantizer,
    TensorBuffer,
    ZeroCopyTensor,
    InferenceStats,
};

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

// Ultra-fast inference exports
pub use ultra_fast::{
    UltraFastEngine,
    UltraFastConfig,
    UltraFastStats,
    SpeculativeDecoder,
    ContinuousBatcher,
    PagedKVCache,
    HzAmplifier,
    flash_attention,
    calculate_theoretical_max,
};

// Integrated engine exports (fully wired pipeline)
pub use integrated_engine::{
    IntegratedEngine,
    IntegratedConfig,
    IntegratedKVCache,
    IntegratedTransformerLayer,
    QuantizedWeights,
    EngineStats,
    flash_attention_forward,
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

pub use transformer::{
    PositionalEncoding,
    SelfAttention,
    MultiHeadAttention,
    FeedForwardNetwork,
    TransformerBlock,
    ActivationFunction,
};

pub use dynamic_context::{
    DynamicPositionalEncoding,
    ConfidenceContextManager,
    ContextStats,
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
