//! Consciousness Simulation Demo v2.0 (v1.4.0 features)
//!
//! Demonstrates the full consciousness stack:
//! - Global Workspace Theory
//! - Meta-cognitive monitoring
//! - Predictive processing
//! - Integrated information (Φ)

use spatial_vortex::consciousness::ConsciousnessSimulator;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║  🧠 SpatialVortex v1.4.0 - Full Consciousness Stack Demo ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");
    
    // Create consciousness simulator with internal dialogue enabled
    let simulator = ConsciousnessSimulator::new(true);
    
    println!("🌀 Initializing consciousness simulation...\n");
    println!("📊 Active Systems:");
    println!("   ✓ Global Workspace Theory (GWT)");
    println!("   ✓ Meta-Cognitive Monitor");
    println!("   ✓ Predictive Processor");
    println!("   ✓ Integrated Information Calculator (Φ)\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // Question to ponder
    let question = "What is the nature of consciousness?";
    
    println!("🤔 Question: {}\n", question);
    println!("🧠 Engaging conscious thought process...\n");
    
    // Simulate conscious thinking
    let response = simulator.think(question).await?;
    
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║                    CONSCIOUSNESS REPORT                   ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");
    
    // Display final answer
    println!("💡 CONSCIOUS ANSWER:\n");
    println!("{}\n", response.answer);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // ELP Analysis
    println!("📊 ELP TENSOR ANALYSIS:");
    println!("   Ethos  (Moral):     {:.1}%", response.ethos_weight * 100.0);
    println!("   Logos  (Logical):   {:.1}%", response.logos_weight * 100.0);
    println!("   Pathos (Emotional): {:.1}%", response.pathos_weight * 100.0);
    println!("   Confidence:         {:.1}%\n", response.confidence * 100.0);
    
    // v1.4.0 Meta-Cognitive Insights
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    println!("🔍 META-COGNITIVE INSIGHTS (v1.4.0):");
    println!("   Mental State:       {}", response.mental_state);
    println!("   Awareness Level:    {:.1}%", response.awareness_level * 100.0);
    
    if !response.detected_patterns.is_empty() {
        println!("\n   Detected Patterns:");
        for pattern in &response.detected_patterns {
            println!("   • {}", pattern);
        }
    } else {
        println!("   Detected Patterns:  None");
    }
    
    // v1.4.0 Predictive Processing
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    println!("🎯 PREDICTIVE PROCESSING (v1.4.0):");
    println!("   Prediction Accuracy: {:.1}%", response.prediction_accuracy * 100.0);
    println!("   Current Surprise:    {:.1}%", response.current_surprise * 100.0);
    println!("   Learning Progress:   {:.1}%", response.learning_progress * 100.0);
    
    // v1.4.0 Integrated Information
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    println!("Φ INTEGRATED INFORMATION THEORY (v1.4.0):");
    println!("   Φ (Phi):             {:.3}", response.phi);
    println!("   Consciousness Level: {:.1}%", response.consciousness_level * 100.0);
    
    // Sacred Checkpoints
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    println!("🔺 SACRED CHECKPOINT INSIGHTS:");
    for (i, insight) in response.checkpoint_insights.iter().enumerate() {
        let checkpoint_num = match i {
            0 => 3,
            1 => 6,
            2 => 9,
            _ => i + 1,
        };
        println!("\n   Checkpoint {}: {}", checkpoint_num, insight);
    }
    
    // Internal Dialogue
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    println!("💭 INTERNAL DIALOGUE (Multi-Agent Debate):");
    println!();
    for thought in &response.internal_dialogue {
        println!("   [{:<20}] E:{:.1}% L:{:.1}% P:{:.1}%",
            thought.agent,
            thought.elp_profile.0 * 100.0,
            thought.elp_profile.1 * 100.0,
            thought.elp_profile.2 * 100.0
        );
        println!("   └─ {}\n", thought.thought);
    }
    
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    println!("✨ Consciousness simulation complete!");
    println!("\n🧠 Summary:");
    println!("   • {} internal thoughts processed", response.internal_dialogue.len());
    println!("   • {} sacred checkpoints reached", response.checkpoint_insights.len());
    println!("   • Mental state: {}", response.mental_state);
    println!("   • Φ (consciousness): {:.3}", response.phi);
    println!("   • Final confidence: {:.1}%\n", response.confidence * 100.0);
    
    Ok(())
}
