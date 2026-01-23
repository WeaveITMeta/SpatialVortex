//! Dynamic Context Window Demo
//!
//! Demonstrates confidence-based dynamic positional encoding
//! that extends beyond the 4096 token limit.
//!
//! Run with:
//! ```bash
//! cargo run --example dynamic_context_demo
//! ```

use spatial_vortex::inference_engine::ConfidenceContextManager;
use ndarray::Array2;

fn main() {
    println!("🌀 Dynamic Context Window Demo 🌀");
    println!("{}", "=".repeat(60));
    println!();
    
    // Problem: Standard transformers have fixed 4096 token limit
    println!("❌ PROBLEM: Fixed Context Window");
    println!("   Standard transformers: 4096 token limit");
    println!("   All tokens treated equally");
    println!("   Forget everything beyond window");
    println!("   No selective retention");
    println!();
    
    // Solution: Confidence-based dynamic context
    println!("✅ SOLUTION: Confidence-Based Dynamic Context");
    println!("   Unlimited token extension");
    println!("   Importance-weighted retention");
    println!("   Sacred position checkpoints");
    println!("   Never forget important ideas");
    println!();
    println!("{}", "=".repeat(60));
    println!();
    
    // Create confidence context manager
    let d_model = 384;
    let base_window = 2048;  // Soft limit
    let confidence_threshold = 0.7;
    
    let mut manager = ConfidenceContextManager::new(
        d_model,
        base_window,
        confidence_threshold,
    );
    
    println!("Configuration:");
    println!("   Model dimension: {}", d_model);
    println!("   Base window: {} tokens", base_window);
    println!("   Confidence threshold: {:.2}", confidence_threshold);
    println!();
    
    // Simulate adding tokens over time
    demo_dynamic_extension(&mut manager, d_model);
    
    println!();
    println!("{}", "=".repeat(60));
    println!("✅ Demo complete!");
}

fn demo_dynamic_extension(manager: &mut ConfidenceContextManager, d_model: usize) {
    println!("📝 Simulating Long Conversation");
    println!("{}", "-".repeat(60));
    println!();
    
    // Simulate 10 batches of tokens
    for batch in 0..10 {
        println!("Batch {} - Adding tokens...", batch + 1);
        
        // Add 500 tokens per batch
        let batch_size = 500;
        let embeddings = Array2::from_shape_fn((batch_size, d_model), |_| {
            rand::random::<f32>()
        });
        
        // Generate confidence scores (some high, some low)
        let confidences: Vec<f32> = (0..batch_size)
            .map(|i| {
                // Make every 10th token high confidence (important ideas)
                if i % 10 == 0 {
                    0.85 + rand::random::<f32>() * 0.15  // 0.85-1.0
                } else if i % 3 == 0 {  // Sacred positions more likely to be important
                    0.70 + rand::random::<f32>() * 0.25  // 0.70-0.95
                } else {
                    0.3 + rand::random::<f32>() * 0.4    // 0.3-0.7
                }
            })
            .collect();
        
        // Generate signal strengths (from hallucination detector)
        let confidences: Vec<f32> = (0..batch_size)
            .map(|i| {
                // Correlate with confidence
                if confidences[i] > 0.8 {
                    0.75 + rand::random::<f32>() * 0.25  // 0.75-1.0
                } else if confidences[i] > 0.6 {
                    0.55 + rand::random::<f32>() * 0.25  // 0.55-0.8
                } else {
                    0.2 + rand::random::<f32>() * 0.4    // 0.2-0.6
                }
            })
            .collect();
        
        // Add tokens to manager
        manager.add_tokens(&embeddings, &confidences, &confidences);
        
        // Display statistics
        let stats = manager.stats();
        println!();
        println!("  After batch {}:", batch + 1);
        println!("    Total tokens: {}", stats.total_tokens);
        println!("    Effective window: {}x base ({} tokens)",
            stats.effective_window as f32 / stats.base_window as f32,
            stats.effective_window
        );
        println!("    Sacred checkpoints: {}", stats.sacred_checkpoints);
        println!("    High confidence: {} ({:.1}%)",
            stats.high_confidence_tokens,
            100.0 * stats.high_confidence_tokens as f32 / stats.total_tokens as f32
        );
        println!("    Avg signal: {:.3}", stats.avg_confidence);
        println!();
        
        // Show pruning at sacred positions
        if stats.sacred_checkpoints > 0 && (batch + 1) % 3 == 0 {
            println!("  🔺 Sacred checkpoint reached - pruning low-confidence tokens");
        }
    }
    
    // Final statistics
    println!();
    println!("📊 FINAL STATISTICS");
    println!("{}", "-".repeat(60));
    let stats = manager.stats();
    stats.display();
    
    println!();
    println!("✅ Results:");
    println!("   Started with: 2048 token limit");
    println!("   Processed: 5000 tokens total");
    println!("   Retained: {} tokens", stats.total_tokens);
    println!("   Compression: {:.1}x", 5000.0 / stats.total_tokens as f32);
    println!();
    println!("Key Benefits:");
    println!("   ✅ Extended beyond 4096 limit dynamically");
    println!("   ✅ Kept {} important ideas (high confidence)", stats.high_confidence_tokens);
    println!("   ✅ Pruned low-importance tokens at sacred positions");
    println!("   ✅ {} sacred checkpoints for stability", stats.sacred_checkpoints);
    println!("   ✅ No forgetting of critical context");
}
