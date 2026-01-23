//! Production Engine Demo
//!
//! Demonstrates the full production inference pipeline:
//! - BPE/SentencePiece tokenization
//! - RoPE position embeddings
//! - Draft model for speculative decoding
//! - Continuous batching for serving
//! - GPU offload configuration
//!
//! ## Run
//!
//! ```bash
//! cargo run --example production_engine_demo --release
//! ```

use spatial_vortex::ml::inference::{
    ProductionEngine,
    ProductionConfig,
    TokenizerType,
    DeviceType,
    OffloadConfig,
    BPETokenizer,
    SentencePieceTokenizer,
    RoPECache,
    DraftModel,
    ContinuousBatchScheduler,
};
use std::time::Instant;
use std::sync::Arc;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║       Production Inference Engine Demo                       ║");
    println!("║    GPU Offload • RoPE • Draft Model • Tokenizers • Batching  ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // ========== 1. Tokenizer Demo ==========
    println!("📝 1. TOKENIZER COMPARISON\n");
    
    let bpe = BPETokenizer::new(32000);
    let sp = SentencePieceTokenizer::new(32000);
    
    let test_texts = [
        "Hello, world!",
        "The quick brown fox jumps over the lazy dog.",
        "Machine learning is transforming AI.",
    ];
    
    for text in test_texts {
        let bpe_tokens = bpe.encode(text);
        let sp_tokens = sp.encode(text);
        
        println!("   Text: \"{}\"", text);
        println!("   BPE tokens: {} -> {:?}", bpe_tokens.len(), &bpe_tokens[..bpe_tokens.len().min(10)]);
        println!("   SP tokens:  {} -> {:?}", sp_tokens.len(), &sp_tokens[..sp_tokens.len().min(10)]);
        
        let bpe_decoded = bpe.decode(&bpe_tokens);
        let sp_decoded = sp.decode(&sp_tokens);
        println!("   BPE decoded: \"{}\"", bpe_decoded);
        println!("   SP decoded:  \"{}\"", sp_decoded);
        println!();
    }

    // ========== 2. RoPE Demo ==========
    println!("📐 2. ROTARY POSITION EMBEDDING (RoPE)\n");
    
    let rope = RoPECache::new(64, 2048, 10000.0);
    
    println!("   RoPE Cache: head_dim=64, max_seq=2048, base=10000");
    println!("   Precomputed cos/sin for all positions");
    
    // Show how RoPE modifies vectors at different positions
    use ndarray::Array2;
    let mut q = Array2::from_shape_fn((4, 64), |(i, j)| ((i * j) as f32 / 100.0).sin());
    let mut k = Array2::from_shape_fn((4, 64), |(i, j)| ((i + j) as f32 / 100.0).cos());
    
    let q_before = q[[0, 0]];
    rope.apply(&mut q, &mut k, 100);
    let q_after = q[[0, 0]];
    
    println!("   Position 100: q[0,0] changed from {:.4} to {:.4}", q_before, q_after);
    println!("   RoPE encodes position into Q/K for attention\n");

    // ========== 3. Draft Model Demo ==========
    println!("🚀 3. DRAFT MODEL (Speculative Decoding)\n");
    
    let draft = DraftModel::new(512, 1000);
    
    println!("   Draft model: 1/4 size of main model (128d vs 512d)");
    println!("   Purpose: Fast token speculation for 2-4x speedup\n");
    
    let context = vec![1, 2, 3, 4, 5];
    
    let start = Instant::now();
    let iterations = 100;
    for _ in 0..iterations {
        let _ = draft.speculate(&context, 4);
    }
    let draft_time = start.elapsed() / iterations;
    
    let tokens = draft.speculate(&context, 4);
    println!("   Speculated 4 tokens: {:?}", tokens);
    println!("   Draft inference time: {:?} per speculation", draft_time);
    println!("   Tokens/sec (draft): {:.0}\n", 4.0 / draft_time.as_secs_f64());

    // ========== 4. Continuous Batching Demo ==========
    println!("📦 4. CONTINUOUS BATCHING\n");
    
    let scheduler = ContinuousBatchScheduler::new(8);
    
    // Submit multiple requests
    let prompts = [
        vec![1, 2, 3],
        vec![4, 5, 6, 7],
        vec![8, 9],
        vec![10, 11, 12, 13, 14],
    ];
    
    let mut request_ids = Vec::new();
    for prompt in &prompts {
        let id = scheduler.submit(prompt.clone(), 10, 0.7, 0.9, None);
        request_ids.push(id);
        println!("   Submitted request {} with {} prompt tokens", id, prompt.len());
    }
    
    println!("\n   Pending: {}, Active: {}", scheduler.pending_count(), scheduler.batch_size());
    
    // Fill batch
    scheduler.fill_batch();
    println!("   After fill: Pending: {}, Active: {}", scheduler.pending_count(), scheduler.batch_size());
    
    // Simulate token generation
    for step in 0..3 {
        let batch = scheduler.get_batch();
        let updates: Vec<_> = batch.iter()
            .map(|r| (r.id, (step + 100) as u32))
            .collect();
        scheduler.update_batch(&updates);
        
        let (tokens, requests, avg) = scheduler.stats();
        println!("   Step {}: Generated {} tokens, {} requests, avg {:.1}", 
            step + 1, tokens, requests, avg);
    }
    println!();

    // ========== 5. GPU Offload Configuration ==========
    println!("🖥️  5. GPU OFFLOAD CONFIGURATION\n");
    
    let configs = [
        ("CPU Only", OffloadConfig {
            device: DeviceType::CPU,
            ..Default::default()
        }),
        ("CUDA GPU 0", OffloadConfig {
            device: DeviceType::CUDA(0),
            gpu_flash_attention: true,
            ..Default::default()
        }),
        ("WGPU (WebGPU)", OffloadConfig {
            device: DeviceType::WGPU(0),
            gpu_flash_attention: true,
            ..Default::default()
        }),
        ("Hybrid (4 CPU layers)", OffloadConfig {
            device: DeviceType::CUDA(0),
            cpu_layers: 4,
            ..Default::default()
        }),
        ("Tensor Parallel (2 GPUs)", OffloadConfig {
            device: DeviceType::CUDA(0),
            tensor_parallel: true,
            num_gpus: 2,
            ..Default::default()
        }),
    ];
    
    for (name, config) in configs {
        println!("   {}: {:?}", name, config.device);
        if config.cpu_layers > 0 {
            println!("      - {} layers on CPU", config.cpu_layers);
        }
        if config.tensor_parallel {
            println!("      - Tensor parallel across {} GPUs", config.num_gpus);
        }
        if config.gpu_flash_attention {
            println!("      - Flash attention enabled");
        }
    }
    println!();

    // ========== 6. Full Production Engine ==========
    println!("⚡ 6. FULL PRODUCTION ENGINE\n");
    
    let config = ProductionConfig {
        d_model: 256,
        num_layers: 4,
        num_heads: 8,
        head_dim: 32,
        vocab_size: 1000,
        max_seq_len: 512,
        tokenizer_type: TokenizerType::BPE,
        use_speculative: true,
        speculative_k: 4,
        max_batch_size: 16,
        rope_base: 10000.0,
        offload: OffloadConfig::default(),
    };
    
    println!("   Config:");
    println!("   - Model: {}d, {} layers, {} heads", config.d_model, config.num_layers, config.num_heads);
    println!("   - Vocab: {}, Max seq: {}", config.vocab_size, config.max_seq_len);
    println!("   - Tokenizer: {:?}", config.tokenizer_type);
    println!("   - Speculative: k={}", config.speculative_k);
    println!("   - Batch size: {}", config.max_batch_size);
    println!();
    
    let engine = ProductionEngine::new(config);
    
    // Test tokenization round-trip
    let text = "Hello, this is a test of the production engine.";
    let tokens = engine.encode(text);
    let decoded = engine.decode(&tokens);
    
    println!("   Tokenization test:");
    println!("   Input:   \"{}\"", text);
    println!("   Tokens:  {} tokens", tokens.len());
    println!("   Decoded: \"{}\"", decoded);
    println!();
    
    // Submit requests and process
    println!("   Submitting requests...");
    let id1 = engine.submit_request("Hello world", 20, 0.7, 0.9);
    let id2 = engine.submit_request("Machine learning", 20, 0.7, 0.9);
    let id3 = engine.submit_request("Rust programming", 20, 0.7, 0.9);
    
    println!("   Request IDs: {}, {}, {}", id1, id2, id3);
    
    // Process steps
    let start = Instant::now();
    let mut total_tokens = 0;
    for step in 0..20 {
        let generated = engine.step();
        total_tokens += generated;
        
        if step % 5 == 4 {
            println!("   Step {}: {} tokens generated this step", step + 1, generated);
        }
    }
    let elapsed = start.elapsed();
    
    let stats = engine.get_stats();
    println!("\n   Final Stats:");
    println!("   - Total tokens: {}", stats.tokens_generated);
    println!("   - Requests: {}", stats.requests_completed);
    println!("   - Avg tokens/request: {:.1}", stats.avg_tokens_per_request);
    println!("   - Batch utilization: {:.1}%", stats.batch_utilization * 100.0);
    println!("   - Time: {:?}", elapsed);
    println!("   - Throughput: {:.1} tokens/sec", total_tokens as f64 / elapsed.as_secs_f64());
    println!();

    // ========== 7. Streaming Demo ==========
    println!("📡 7. STREAMING GENERATION\n");
    
    let engine2 = ProductionEngine::new(ProductionConfig {
        d_model: 128,
        num_layers: 2,
        num_heads: 4,
        head_dim: 32,
        vocab_size: 500,
        max_seq_len: 256,
        use_speculative: false,
        max_batch_size: 4,
        ..Default::default()
    });
    
    let streamed_tokens = Arc::new(std::sync::Mutex::new(Vec::new()));
    let tokens_clone = streamed_tokens.clone();
    
    let callback = Arc::new(move |token: u32| {
        tokens_clone.lock().unwrap().push(token);
    });
    
    let stream_id = engine2.submit_streaming("Test streaming", 10, 0.7, 0.9, callback);
    
    println!("   Streaming request {} submitted", stream_id);
    
    for _ in 0..15 {
        engine2.step();
    }
    
    let received = streamed_tokens.lock().unwrap();
    println!("   Streamed {} tokens: {:?}", received.len(), &received[..received.len().min(10)]);
    println!();

    // ========== Summary ==========
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                     SUMMARY                                   ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║ ✅ BPE Tokenizer: Byte-level with merge rules                ║");
    println!("║ ✅ SentencePiece: Unigram model with ▁ word boundary         ║");
    println!("║ ✅ RoPE: Precomputed cos/sin, fused Q/K rotation             ║");
    println!("║ ✅ Draft Model: 1/4 size for fast speculation                ║");
    println!("║ ✅ Continuous Batching: Dynamic request scheduling           ║");
    println!("║ ✅ GPU Offload: CPU/CUDA/WGPU/Metal abstraction              ║");
    println!("║ ✅ Streaming: Callback-based token delivery                  ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    println!("📈 PRODUCTION DEPLOYMENT:");
    println!("   1. Load real model weights (GGUF/Safetensors)");
    println!("   2. Enable GPU offload for 10-100x speedup");
    println!("   3. Tune batch size for throughput vs latency");
    println!("   4. Use streaming for real-time responses");
}
