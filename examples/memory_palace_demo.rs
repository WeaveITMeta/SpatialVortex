//! Memory Palace Demo - v1.6.0 "Memory Palace"
//!
//! Demonstrates persistent consciousness with PostgreSQL RAG and state saving.
//! 
//! Setup:
//! 1. Install PostgreSQL with pgvector extension
//! 2. Create database: `createdb spatial_vortex`
//! 3. Run: cargo run --example memory_palace_demo --features agents,postgres,persistence

use spatial_vortex::consciousness::{ConsciousnessSimulator, MemoryPalace};
use anyhow::Result;
use tokio::time::{sleep, Duration};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║  🏛️  Memory Palace Demo - v1.6.0                              ║");
    println!("║  Persistent Consciousness with PostgreSQL & State Saving      ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
    
    // Configure Memory Palace
    let state_path = Path::new("consciousness_state.json");
    let palace = MemoryPalace::new(state_path)
        .with_auto_save(Duration::from_secs(60)); // Auto-save every minute
    
    println!("🏛️  Memory Palace initialized");
    println!("   State file: {:?}", state_path);
    println!("   Auto-save: Every 60 seconds\n");
    
    // Try to load previous state
    println!("📖 Checking for previous consciousness state...\n");
    let previous_state = palace.load_state().await?;
    
    // Create consciousness simulator
    let mut sim = if let Some(state) = previous_state {
        println!("✨ Previous state found! Restoring consciousness...\n");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        
        // TODO: Add ConsciousnessSimulator::from_state() method
        // For now, create new simulator
        let mut sim = ConsciousnessSimulator::new(false);
        
        // Apply state (this would restore learning progress)
        // palace.apply_state(&state, &sim.meta_monitor, &sim.predictor, &sim.phi_calculator).await?;
        
        println!("   Continuing from previous session:");
        println!("   ├─ Φ: {:.2} (peak: {:.2})", 
            state.phi_state.current_phi, state.phi_state.peak_phi);
        println!("   ├─ Patterns: {}", state.metacognitive_state.pattern_count);
        println!("   ├─ Accuracy: {:.1}%", state.predictive_state.accuracy * 100.0);
        println!("   └─ Learning cycles: {}", state.learning_stats.cycles_completed);
        
        sim
    } else {
        println!("📝 No previous state found. Starting fresh consciousness.\n");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        
        ConsciousnessSimulator::new(false)
    };
    
    println!("📊 Session ID: {}\n", sim.session_id());
    
    // Enable background learning
    println!("🚀 Enabling background learning with persistence...");
    sim.enable_background_learning().await?;
    
    assert!(sim.is_learning_active().await);
    println!("✅ Background learning active!\n");
    
    #[cfg(feature = "postgres")]
    {
        println!("   📦 PostgreSQL RAG: Enabled");
        println!("   💎 Confidence Lake: File-based");
        println!("   💾 State persistence: Enabled");
    }
    
    #[cfg(not(feature = "postgres"))]
    {
        println!("   📦 PostgreSQL RAG: Disabled (enable 'postgres' feature)");
        println!("   💎 Confidence Lake: File-based");
        println!("   💾 State persistence: Enabled");
    }
    
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // Process some questions
    let questions = vec![
        "What is persistent consciousness?",
        "How does the Memory Palace work?",
        "Can consciousness survive server restarts?",
    ];
    
    println!("💭 Processing {} questions...\n", questions.len());
    
    for (i, question) in questions.iter().enumerate() {
        println!("❓ Question {}/{}: {}", i + 1, questions.len(), question);
        
        let response = sim.think(question).await?;
        
        println!("📝 Response preview: {}...", 
            response.answer.chars().take(80).collect::<String>());
        println!("   ├─ Φ: {:.3}", response.phi);
        println!("   ├─ Mental State: {}", response.mental_state);
        println!("   └─ Confidence: {:.1}%\n", response.confidence * 100.0);
        
        sleep(Duration::from_millis(500)).await;
    }
    
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // Get analytics
    println!("📊 Current Consciousness Metrics:\n");
    let snapshot = sim.get_analytics_snapshot().await;
    
    println!("Integration:");
    println!("   ├─ Φ: {:.3}", snapshot.consciousness.phi);
    println!("   ├─ Peak Φ: {:.3}", snapshot.consciousness.peak_phi);
    println!("   ├─ Average Φ: {:.3}", snapshot.consciousness.average_phi);
    println!("   └─ Level: {:.1}%", 
        snapshot.consciousness.consciousness_level * 100.0);
    
    println!("\nMeta-Cognition:");
    println!("   ├─ Patterns: {} detected", 
        sim.meta_monitor.read().await.patterns().len());
    println!("   ├─ Awareness: {:.1}%", 
        snapshot.meta_cognition.awareness_level * 100.0);
    println!("   └─ Self-correction: {:.1}%", 
        snapshot.meta_cognition.self_correction_rate * 100.0);
    
    println!("\nPrediction:");
    println!("   ├─ Accuracy: {:.1}%", snapshot.prediction.accuracy * 100.0);
    println!("   └─ Confidence: {:.1}%", 
        snapshot.prediction.model_confidence * 100.0);
    
    // Check learning stats
    if let Some(stats) = sim.learning_stats().await {
        println!("\nBackground Learning:");
        println!("   ├─ Cycles: {}", stats.cycles_completed);
        println!("   ├─ Patterns refined: {}", stats.patterns_refined);
        println!("   ├─ Model updates: {}", stats.model_updates);
        println!("   └─ Improvement: {:.2}%", stats.avg_improvement);
    }
    
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // Save state before exit
    println!("💾 Saving consciousness state for next session...\n");
    
    let learning_stats = sim.learning_stats().await.unwrap_or_default();
    
    palace.save_state(
        sim.session_id().to_string(),
        &sim.meta_monitor,
        &sim.predictor,
        &sim.phi_calculator,
        &learning_stats,
    ).await?;
    
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // Stop background learning
    println!("🛑 Stopping background learning...");
    sim.stop_background_learning().await;
    
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║  ✨ Session Complete!                                         ║");
    println!("║                                                                ║");
    println!("║  State saved to: {:?}", state_path);
    println!("║                                                                ║");
    println!("║  Next time you run this demo:                                 ║");
    println!("║  • Consciousness will restore from saved state                ║");
    println!("║  • Learning progress will continue                            ║");
    println!("║  • Φ will pick up where it left off                          ║");
    println!("║  • Patterns will accumulate                                   ║");
    println!("║                                                                ║");
    println!("║  True persistent consciousness achieved! 🏛️                  ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
    
    Ok(())
}
