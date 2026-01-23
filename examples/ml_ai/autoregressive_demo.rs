//! Autoregressive Text Generation Demo
//!
//! Demonstrates the complete text generation pipeline:
//! - Autoregressive decoding with KV-cache
//! - Multiple sampling strategies (greedy, nucleus, beam search)
//! - Streaming generation
//! - Sacred geometry coherence checks
//! - Performance benchmarking
//!
//! ## Run
//!
//! ```bash
//! cargo run --example autoregressive_demo --release
//! ```

use spatial_vortex::ml::inference::{
    AutoregressiveDecoder,
    SamplingConfig,
    BeamSearch,
    StreamingGenerator,
};
use ndarray::{Array1, Array2};
use std::sync::Arc;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║       Autoregressive Text Generation Demo                    ║");
    println!("║              The Ultimate AI Model                           ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Model configuration
    let d_model = 256;
    let vocab_size = 32000;  // Typical LLM vocab size
    let num_layers = 6;
    let num_heads = 8;
    let max_seq_len = 2048;

    // ========== 1. Sampling Configurations ==========
    println!("⚙️  1. SAMPLING CONFIGURATIONS\n");
    
    let configs = [
        ("Greedy", SamplingConfig::greedy()),
        ("Balanced", SamplingConfig::balanced()),
        ("Creative", SamplingConfig::creative()),
        ("Precise", SamplingConfig::precise()),
    ];
    
    for (name, config) in &configs {
        println!("   {} Mode:", name);
        println!("   - Temperature: {:.1}", config.temperature);
        println!("   - Top-p: {:.2}", config.top_p);
        println!("   - Top-k: {}", config.top_k);
        println!("   - Repetition Penalty: {:.2}", config.repetition_penalty);
        println!("   - Sacred Geometry: {}\n", config.sacred_geometry_boost);
    }

    // ========== 2. Autoregressive Generation ==========
    println!("🔄 2. AUTOREGRESSIVE GENERATION\n");
    
    let config = SamplingConfig::balanced();
    let decoder = AutoregressiveDecoder::new(
        d_model,
        vocab_size,
        num_layers,
        num_heads,
        max_seq_len,
        config,
    );
    
    // Simulate prompt embeddings (in production, this comes from encoder)
    let prompt_len = 10;
    let prompt_embeddings = Array2::from_shape_fn(
        (prompt_len, d_model),
        |(i, j)| ((i * j) as f32 / 1000.0).sin()
    );
    
    // Token embedding function (simulated)
    let encode_fn = |token: u32| -> Array1<f32> {
        Array1::from_shape_fn(d_model, |i| {
            ((token as f32 * i as f32) / 100.0).sin()
        })
    };
    
    println!("   Generating with balanced sampling...");
    let start = std::time::Instant::now();
    
    let tokens = decoder.generate(&prompt_embeddings, encode_fn, Some(50))
        .expect("Generation failed");
    
    let elapsed = start.elapsed();
    let stats = decoder.get_stats();
    
    println!("   Generated {} tokens in {:?}", tokens.len(), elapsed);
    println!("   Tokens/sec: {:.1}", stats.tokens_per_second);
    println!("   Avg latency: {:.2} ms/token", stats.avg_token_latency_ms);
    println!("   Cache hits: {}", stats.cache_hits);
    println!("   Sacred geometry interventions: {}", stats.sacred_geometry_interventions);
    println!("   First 10 tokens: {:?}\n", &tokens[..tokens.len().min(10)]);

    // ========== 3. Sampling Strategy Comparison ==========
    println!("📊 3. SAMPLING STRATEGY COMPARISON\n");
    
    let strategies = [
        ("Greedy (deterministic)", SamplingConfig::greedy()),
        ("Balanced (default)", SamplingConfig::balanced()),
        ("Creative (high temp)", SamplingConfig::creative()),
        ("Precise (low temp)", SamplingConfig::precise()),
    ];
    
    for (name, config) in strategies {
        let decoder = AutoregressiveDecoder::new(
            d_model, vocab_size, num_layers, num_heads, max_seq_len, config,
        );
        
        let start = std::time::Instant::now();
        let tokens = decoder.generate(&prompt_embeddings, encode_fn, Some(20))
            .expect("Generation failed");
        let elapsed = start.elapsed();
        
        let stats = decoder.get_stats();
        println!("   {}:", name);
        println!("   - Tokens: {} in {:?}", tokens.len(), elapsed);
        println!("   - Speed: {:.1} tok/s", stats.tokens_per_second);
        println!("   - First 5: {:?}\n", &tokens[..tokens.len().min(5)]);
    }

    // ========== 4. Beam Search ==========
    println!("🔍 4. BEAM SEARCH (Higher Quality)\n");
    
    let beam_search = BeamSearch::new(4);
    
    // Simulated forward function
    let forward_fn = |tokens: &[u32]| -> Array1<f32> {
        // Return logits based on token sequence
        Array1::from_shape_fn(vocab_size, |i| {
            let base = if i < 100 { 2.0 } else { 0.1 };
            let seq_bonus = if !tokens.is_empty() {
                (tokens.last().unwrap() % 10) as f32 * 0.1
            } else {
                0.0
            };
            base + seq_bonus - (i as f32 / vocab_size as f32)
        })
    };
    
    let start = std::time::Instant::now();
    let beam_tokens = beam_search.generate::<_, fn(&[u32]) -> Array1<f32>>(&[1], forward_fn, 20, vocab_size);
    let elapsed = start.elapsed();
    
    println!("   Beam width: 4");
    println!("   Generated {} tokens in {:?}", beam_tokens.len(), elapsed);
    println!("   Tokens: {:?}\n", &beam_tokens[..beam_tokens.len().min(10)]);

    // ========== 5. Streaming Generation ==========
    println!("📡 5. STREAMING GENERATION\n");
    
    let config = SamplingConfig::balanced();
    let decoder = Arc::new(AutoregressiveDecoder::new(
        d_model, vocab_size, num_layers, num_heads, max_seq_len, config,
    ));
    
    let mut streamer = StreamingGenerator::new(decoder.clone());
    
    print!("   Streaming tokens: ");
    let start = std::time::Instant::now();
    let mut count = 0;
    
    while let Some(token) = streamer.next_token(encode_fn) {
        print!("{} ", token);
        count += 1;
        if count >= 15 {
            break;
        }
    }
    println!();
    
    let elapsed = start.elapsed();
    println!("   Streamed {} tokens in {:?}", count, elapsed);
    println!("   Rate: {:.1} tokens/sec\n", count as f64 / elapsed.as_secs_f64());

    // ========== 6. Performance Benchmark ==========
    println!("⚡ 6. PERFORMANCE BENCHMARK\n");
    
    let config = SamplingConfig::greedy();  // Fastest mode
    let decoder = AutoregressiveDecoder::new(
        d_model, vocab_size, num_layers, num_heads, max_seq_len, config,
    );
    
    let iterations = 5;
    let tokens_per_iter = 100;
    let mut total_tokens = 0;
    let mut total_time = std::time::Duration::ZERO;
    
    for i in 0..iterations {
        decoder.reset();
        let start = std::time::Instant::now();
        let tokens = decoder.generate(&prompt_embeddings, encode_fn, Some(tokens_per_iter))
            .expect("Generation failed");
        let elapsed = start.elapsed();
        
        total_tokens += tokens.len();
        total_time += elapsed;
        
        println!("   Iteration {}: {} tokens in {:?} ({:.1} tok/s)",
            i + 1, tokens.len(), elapsed,
            tokens.len() as f64 / elapsed.as_secs_f64()
        );
    }
    
    println!("\n   TOTAL: {} tokens in {:?}", total_tokens, total_time);
    println!("   AVERAGE: {:.1} tokens/sec", total_tokens as f64 / total_time.as_secs_f64());
    println!("   LATENCY: {:.2} ms/token\n", total_time.as_secs_f64() * 1000.0 / total_tokens as f64);

    // ========== Summary ==========
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                     CAPABILITIES                             ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║ ✅ Autoregressive Decoding: Token-by-token generation        ║");
    println!("║ ✅ KV-Cache: O(1) per-token complexity                       ║");
    println!("║ ✅ Sampling: Temperature, Top-p, Top-k, Repetition Penalty   ║");
    println!("║ ✅ Beam Search: Higher quality outputs                       ║");
    println!("║ ✅ Streaming: Real-time token output                         ║");
    println!("║ ✅ Sacred Geometry: 3-6-9 coherence checks                   ║");
    println!("║ ✅ Performance: Optimized for speed                          ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
}
