//! Background Learning Demo with RAG + Confidence Lake
//!
//! Demonstrates the complete background learning system with:
//! - RAG knowledge ingestion
//! - Confidence Lake pattern review
//! - Continuous model improvement
//!
//! Run: cargo run --example background_learning_full_demo --features agents,rag,lake

use spatial_vortex::consciousness::ConsciousnessSimulator;
use anyhow::Result;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<()> {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║  🧠 Background Learning Demo - Full System                    ║");
    println!("║  RAG + Confidence Lake + Continuous Improvement               ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
    
    // Create consciousness simulator
    println!("🔧 Creating consciousness simulator...");
    let mut sim = ConsciousnessSimulator::new(false).await;
    
    println!("📊 Session ID: {}", sim.session_id());
    
    // Enable background learning
    println!("\n🚀 Enabling background learning...");
    sim.enable_background_learning().await?;
    
    // Verify it's active
    assert!(sim.is_learning_active().await);
    println!("✅ Background learning active!");
    
    #[cfg(feature = "rag")]
    println!("   📚 RAG ingestion: Enabled");
    #[cfg(not(feature = "rag"))]
    println!("   📚 RAG ingestion: Not available (enable 'rag' feature)");
    
    #[cfg(feature = "lake")]
    println!("   💎 Confidence Lake: Enabled");
    #[cfg(not(feature = "lake"))]
    println!("   💎 Confidence Lake: Not available (enable 'lake' feature)");
    
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // Ask some questions to generate learning data
    let questions = vec![
        "What is consciousness?",
        "How does self-awareness emerge?",
        "What is the relationship between mind and brain?",
        "Can artificial intelligence achieve consciousness?",
        "What role does experience play in consciousness?",
    ];
    
    println!("💭 Processing {} questions...\n", questions.len());
    
    for (i, question) in questions.iter().enumerate() {
        println!("❓ Question {}/{}: {}", i + 1, questions.len(), question);
        
        let response = sim.think(question).await?;
        
        println!("📝 Response preview: {}...", 
            response.answer.chars().take(100).collect::<String>());
        println!("   ├─ Φ: {:.3}", response.phi);
        println!("   ├─ Mental State: {}", response.mental_state);
        println!("   └─ Confidence: {:.1}%\n", response.confidence * 100.0);
        
        // Small delay between questions
        sleep(Duration::from_millis(500)).await;
    }
    
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // Check learning statistics
    println!("📊 Checking learning statistics...\n");
    
    if let Some(stats) = sim.learning_stats().await {
        println!("Learning Progress:");
        println!("   ├─ Cycles completed: {}", stats.cycles_completed);
        println!("   ├─ Patterns refined: {}", stats.patterns_refined);
        println!("   ├─ Model updates: {}", stats.model_updates);
        println!("   ├─ Knowledge ingested: {} bytes", stats.knowledge_ingested);
        println!("   └─ Average improvement: {:.2}%", stats.avg_improvement);
        
        if let Some(last) = stats.last_learning {
            let elapsed = std::time::SystemTime::now()
                .duration_since(last)
                .unwrap_or_default();
            println!("\n   Last learning: {:?} ago", elapsed);
        }
    } else {
        println!("⚠️ No learning statistics available yet");
    }
    
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // Wait for at least one learning cycle (5 minutes in production, but let's wait briefly)
    println!("⏳ Waiting for learning cycle (this would be 5 minutes in production)...");
    println!("   For demo purposes, we'll just show current state\n");
    
    sleep(Duration::from_secs(2)).await;
    
    // Get analytics snapshot
    println!("📸 Current Analytics Snapshot:\n");
    let snapshot = sim.get_analytics_snapshot().await;
    
    println!("Consciousness Metrics:");
    println!("   ├─ Φ: {:.3}", snapshot.consciousness.phi);
    println!("   ├─ Peak Φ: {:.3}", snapshot.consciousness.peak_phi);
    println!("   ├─ Average Φ: {:.3}", snapshot.consciousness.average_phi);
    println!("   ├─ Consciousness Level: {:.1}%", 
        snapshot.consciousness.consciousness_level * 100.0);
    println!("   ├─ Network: {} nodes, {} connections",
        snapshot.consciousness.network_size,
        snapshot.consciousness.connection_count);
    println!("   └─ Integration: {:.3}", snapshot.consciousness.integration_strength);
    
    println!("\nMeta-Cognition:");
    println!("   ├─ Mental State: {}", snapshot.meta_cognition.mental_state);
    println!("   ├─ Awareness: {:.1}%", 
        snapshot.meta_cognition.awareness_level * 100.0);
    println!("   ├─ Introspection: {:.1}%", 
        snapshot.meta_cognition.introspection_depth * 100.0);
    println!("   ├─ Pattern Recognition: {:.1}%", 
        snapshot.meta_cognition.pattern_recognition * 100.0);
    println!("   └─ Self-Correction: {:.1}%", 
        snapshot.meta_cognition.self_correction_rate * 100.0);
    
    println!("\nPredictive Processing:");
    println!("   ├─ Accuracy: {:.1}%", snapshot.prediction.accuracy * 100.0);
    println!("   ├─ Current Surprise: {:.1}%", 
        snapshot.prediction.current_surprise * 100.0);
    println!("   ├─ Learning Progress: {:.1}%", 
        snapshot.prediction.learning_progress * 100.0);
    println!("   └─ Model Confidence: {:.1}%", 
        snapshot.prediction.model_confidence * 100.0);
    
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // Demonstrate stopping and restarting
    println!("🛑 Stopping background learning...");
    sim.stop_background_learning().await;
    
    sleep(Duration::from_millis(500)).await;
    assert!(!sim.is_learning_active().await);
    println!("✅ Background learning stopped\n");
    
    println!("🔄 Restarting background learning...");
    sim.start_background_learning().await?;
    
    sleep(Duration::from_millis(500)).await;
    assert!(sim.is_learning_active().await);
    println!("✅ Background learning restarted\n");
    
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    println!("💡 What's Happening in the Background:\n");
    println!("Every 5 minutes, the system:");
    println!("   1. ✅ Analyzes meta-cognitive patterns");
    println!("   2. ✅ Refines predictive model based on accuracy");
    println!("   3. ✅ Optimizes Φ network (prunes proactively)");
    
    #[cfg(feature = "rag")]
    println!("   4. ✅ Ingests new knowledge from RAG sources");
    #[cfg(not(feature = "rag"))]
    println!("   4. ⚠️ RAG ingestion (requires 'rag' feature)");
    
    #[cfg(feature = "lake")]
    println!("   5. ✅ Reviews Confidence Lake for high-value patterns");
    #[cfg(not(feature = "lake"))]
    println!("   5. ⚠️ Confidence Lake review (requires 'lake' feature)");
    
    println!("\nResult: AI that gets smarter every day! 🧠📈\n");
    
    // Stop background learning before exit
    println!("🛑 Stopping background learning for clean exit...");
    sim.stop_background_learning().await;
    
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║  ✨ Demo Complete!                                            ║");
    println!("║  The system is now continuously learning in the background    ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
    
    Ok(())
}
