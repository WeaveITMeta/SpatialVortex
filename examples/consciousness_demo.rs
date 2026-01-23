//! Consciousness Simulation Demo
//!
//! Demonstrates the Global Workspace Theory implementation
//! where multiple cognitive agents debate internally before
//! producing a unified conscious response.
//!
//! Run with: cargo run --example consciousness_demo

use spatial_vortex::consciousness::ConsciousnessSimulator;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    println!("🧠 SpatialVortex Consciousness Simulation v1.3.0\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // Create consciousness simulator with internal dialogue enabled
    let simulator = ConsciousnessSimulator::new(true);
    
    // Ask a philosophical question
    let question = "What is the nature of consciousness?";
    
    println!("❓ Question: {}\n", question);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // Run consciousness simulation
    let response = simulator.think(question).await?;
    
    // Display results
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 ELP Analysis:");
    println!("   Ethos (Moral):    {:.1}%", response.ethos_weight * 100.0);
    println!("   Logos (Logical):  {:.1}%", response.logos_weight * 100.0);
    println!("   Pathos (Emotional): {:.1}%", response.pathos_weight * 100.0);
    println!("   Confidence:       {:.1}%", response.confidence * 100.0);
    println!();
    
    println!("🔺 Sacred Checkpoints:");
    for (i, insight) in response.checkpoint_insights.iter().enumerate() {
        println!("   Checkpoint {}: {}", [3, 6, 9][i.min(2)], 
            &insight[..100.min(insight.len())]);
    }
    println!();
    
    println!("✨ Final Conscious Response:");
    println!("{}", response.answer);
    println!();
    
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("💭 Internal Dialogue ({} thoughts):", response.internal_dialogue.len());
    for (i, thought) in response.internal_dialogue.iter().enumerate() {
        println!("\n{}. {} (E:{:.1}/L:{:.1}/P:{:.1})",
            i + 1,
            thought.agent,
            thought.elp_profile.0 * 100.0,
            thought.elp_profile.1 * 100.0,
            thought.elp_profile.2 * 100.0
        );
        println!("   {}", &thought.thought[..150.min(thought.thought.len())]);
    }
    
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ Consciousness simulation complete!");
    
    Ok(())
}
