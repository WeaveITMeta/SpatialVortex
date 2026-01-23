//! AGI Demo - Flux-Native Reasoning
//!
//! This demonstrates Vortex's core AGI capability: thinking in flux matrices
//! instead of language, only querying LLMs as "oracles" for specific knowledge gaps.
//!
//! Run with:
//! ```bash
//! cargo run --example agi_demo --features "agents,persistence"
//! ```

use spatial_vortex::ai::flux_reasoning::{FluxReasoningChain, EntropyType};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    println!("\n🧠 SpatialVortex AGI Demo");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // Test queries of different types
    let queries = vec![
        ("What is quantum entanglement?", "Factual query - should query physics oracle"),
        ("Why do plants need sunlight?", "Causal query - should query biology oracle"),
        ("How can I be more productive?", "Multi-path query - should query multiple oracles"),
        ("Should AI be regulated?", "Ethical query - should engage ethical reasoning"),
    ];
    
    for (query, description) in queries {
        println!("\n╔══════════════════════════════════════════════════════╗");
        println!("║ Query: {:<46} ║", query);
        println!("║ Type:  {:<46} ║", description);
        println!("╚══════════════════════════════════════════════════════╝\n");
        
        // Create reasoning chain
        let mut chain = FluxReasoningChain::new(query);
        
        // Show initial state
        let initial = chain.current_thought();
        println!("📊 Initial Flux State:");
        println!("   Ethos:  {:.2}", initial.elp_state.ethos);
        println!("   Logos:  {:.2}", initial.elp_state.logos);
        println!("   Pathos: {:.2}", initial.elp_state.pathos);
        println!("   Entropy: {:.2} ({:?})", initial.entropy, initial.entropy_type);
        println!("   Vortex Position: {}", initial.vortex_position);
        
        // Reason (max 10 steps for demo)
        println!("\n🔄 Reasoning Process:");
        println!("   (Limited to 10 steps for demo - real AGI has no limit)\n");
        
        match chain.reason(10).await {
            Ok(final_thought) => {
                println!("\n✅ Reasoning Complete!");
                println!("   Steps: {}", chain.thoughts.len());
                println!("   ⭐ Sacred Milestones: {:?}", chain.sacred_milestones);
                println!("   Final Confidence: {:.0}%", final_thought.certainty * 100.0);
                println!("   Final Entropy: {:.2}%", final_thought.entropy * 100.0);
                println!("   Oracle Queries: {}", final_thought.oracle_contributions.len());
                
                // Show oracle contributions
                if !final_thought.oracle_contributions.is_empty() {
                    println!("\n🔮 Oracle Contributions:");
                    for (i, oracle) in final_thought.oracle_contributions.iter().enumerate() {
                        println!("   {}. {} (entropy reduction: {:.2})", 
                            i + 1, oracle.model, oracle.entropy_reduction);
                    }
                }
                
                // Show final answer
                println!("\n💬 Final Answer:");
                let answer = chain.to_natural_language();
                println!("   {}", answer);
                
                // Show convergence status
                if chain.has_converged() {
                    println!("\n⭐ CONVERGED - High confidence answer achieved");
                } else {
                    println!("\n⚠️  NOT CONVERGED - More reasoning needed for full confidence");
                }
            },
            Err(e) => {
                println!("\n❌ Reasoning failed: {}", e);
            }
        }
        
        println!("\n{}", "─".repeat(60));
    }
    
    println!("\n\n🎯 AGI Demo Complete!");
    println!("\n📈 Key Observations:");
    println!("   1. Different query types trigger different entropy patterns");
    println!("   2. LLMs are queried ONLY when entropy is high");
    println!("   3. Sacred checkpoints (3, 6, 9) consolidate reasoning");
    println!("   4. Vortex cycles through 1→2→4→8→7→5→1 positions");
    println!("   5. Final answer synthesizes all oracle contributions\n");
    
    println!("🚀 This is AGI - geometric reasoning, strategic knowledge acquisition\n");
    
    Ok(())
}

// Helper function to demonstrate internal flux transformation
#[allow(dead_code)]
fn demonstrate_flux_transformation() {
    println!("\n🔬 Flux Transformation Mechanics:");
    println!("   When entropy < 0.7:");
    println!("   ├─ Vortex advances: 1→2→4→8→7→5→1");
    println!("   ├─ ELP state maintained (internal reasoning)");
    println!("   ├─ Certainty increases slightly");
    println!("   └─ Entropy decreases slightly");
    println!("\n   When entropy ≥ 0.7:");
    println!("   ├─ Query LLM oracle for specific knowledge");
    println!("   ├─ Integrate response → ELP update");
    println!("   ├─ Certainty increases significantly");
    println!("   └─ Entropy reduces by ~0.3");
    println!("\n   At sacred checkpoints (3, 6, 9):");
    println!("   ├─ Consolidate reasoning");
    println!("   ├─ Boost confidence if coherent");
    println!("   └─ Record milestone\n");
}

// Helper function to show entropy types
#[allow(dead_code)]
fn show_entropy_types() {
    use EntropyType::*;
    
    println!("\n🎯 Entropy Types & Oracle Strategies:");
    println!("\n   {:?}:", MissingFacts);
    println!("      Query: 'What are the key facts about X?'");
    println!("      Example: 'What is quantum entanglement?'");
    
    println!("\n   {:?}:", UnclearCausality);
    println!("      Query: 'What causes or explains X?'");
    println!("      Example: 'Why do plants need sunlight?'");
    
    println!("\n   {:?}:", MultiplePathways);
    println!("      Query: 'What are all the ways to achieve X?'");
    println!("      Example: 'How can I be more productive?'");
    
    println!("\n   {:?}:", EthicalAmbiguity);
    println!("      Query: 'What are the ethical considerations of X?'");
    println!("      Example: 'Should AI be regulated?'");
    
    println!("\n   {:?}:", Low);
    println!("      Strategy: Internal flux transformation only");
    println!("      Example: Basic reasoning with existing knowledge\n");
}
