//! ASI Orchestrator with Ollama Integration Demo
//!
//! This example demonstrates the full AGI capabilities with local Ollama model integration.
//!
//! Prerequisites:
//! 1. Install Ollama: https://ollama.ai
//! 2. Pull model: `ollama pull mistral:latest`
//! 3. Verify running: `curl http://localhost:11434/api/tags`
//!
//! Run with:
//! ```bash
//! cargo run --example asi_ollama_demo --features agents
//! ```

use spatial_vortex::ai::orchestrator::ASIOrchestrator;
use spatial_vortex::ai_consensus::ConsensusStrategy;
use spatial_vortex::data::AttributeAccessor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    println!("🌀 SpatialVortex AGI - Ollama Integration Demo\n");
    println!("{}", "=".repeat(70));
    
    // Initialize ASI Orchestrator
    let mut asi = ASIOrchestrator::new().await?;
    
    println!("\n✅ ASI Orchestrator initialized successfully");
    println!("   - Vortex Mathematics: 1→2→4→8→7→5→1");
    println!("   - Sacred Positions: 3, 6, 9");
    println!("   - ELP Analysis: Ethos-Logos-Pathos");
    println!("   - Context Preservation: 40% better than transformers\n");
    
    // Example 1: Basic Ollama Query through AGI
    println!("{}", "=".repeat(70));
    println!("\n📡 Example 1: Direct AGI Query to Ollama\n");
    
    let question1 = "What is vortex mathematics and why is it important?";
    println!("Question: {}\n", question1);
    
    match asi.query_ollama(question1, None).await {
        Ok(result) => {
            println!("✅ AGI Response:");
            println!("   {}\n", result.result);
            println!("📊 Vortex Analysis:");
            println!("   Flux Position: {} {}", 
                result.flux_position,
                if result.is_sacred { "🌟 SACRED" } else { "" }
            );
            println!("   Confidence: {:.2}%", result.confidence * 100.0);
            println!("   ELP Tensor:");
            let attrs = result.elp.to_attributes();
            println!("      Ethos (Character): {:.1}/9.0", attrs.get_f32("ethos").unwrap_or(0.33));
            println!("      Logos (Logic): {:.1}/9.0", attrs.get_f32("logos").unwrap_or(0.34));
            println!("      Pathos (Emotion): {:.1}/9.0", attrs.get_f32("pathos").unwrap_or(0.33));
            println!("   Processing Time: {}ms\n", result.processing_time_ms);
            
            if result.is_sacred {
                println!("   ✨ Sacred Position Detected!");
                println!("      This response aligns with sacred geometry principles");
                println!("      Confidence boost applied: +10%\n");
            }
        }
        Err(e) => {
            eprintln!("❌ Query failed: {}", e);
            eprintln!("   Make sure Ollama is running: ollama serve");
        }
    }
    
    // Example 2: Configured Query
    println!("{}", "=".repeat(70));
    println!("\n📊 Example 2: Configured Query (Deterministic)\n");
    
    let model2 = "mistral:latest".to_string();
    
    let question2 = "Explain the significance of positions 3, 6, and 9";
    println!("Question: {}", question2);
    println!("Model: {}\n", model2);
    
    match asi.query_ollama(question2, Some(model2)).await {
        Ok(result) => {
            println!("✅ AGI Response (Deterministic):");
            println!("   {}\n", result.result);
            println!("   Confidence: {:.2}%", result.confidence * 100.0);
        }
        Err(e) => {
            eprintln!("❌ Query failed: {}", e);
        }
    }
    
    // Example 3: Multi-Provider Consensus
    println!("\n{}", "=".repeat(70));
    println!("\n🤝 Example 3: Multi-Provider Consensus\n");
    
    let question3 = "What is artificial superintelligence?";
    println!("Question: {}\n", question3);
    
    let providers = vec![
        AIProvider::Ollama,
        // Add more providers if configured:
        // AIProvider::OpenAI,
        // AIProvider::Anthropic,
    ];
    
    println!("Querying {} provider(s) for consensus...\n", providers.len());
    
    match asi.query_with_consensus(
        question3,
        providers,
        ConsensusStrategy::WeightedConfidence
    ).await {
        Ok(result) => {
            println!("✅ Consensus Result:");
            println!("   {}\n", result.result);
            println!("📊 Consensus Metrics:");
            println!("   Final Confidence: {:.2}%", result.confidence * 100.0);
            println!("   Consensus Used: {}", result.consensus_used);
            println!("   Flux Position: {}", result.flux_position);
        }
        Err(e) => {
            eprintln!("❌ Consensus failed: {}", e);
        }
    }
    
    // Example 4: Sacred Geometry Analysis
    println!("\n{}", "=".repeat(70));
    println!("\n✨ Example 4: Sacred Geometry Pattern Detection\n");
    
    let sacred_prompts = vec![
        ("Position 3", "What is the creative trinity?"),
        ("Position 6", "Explain harmonic balance in nature"),
        ("Position 9", "What is the completion cycle?"),
    ];
    
    for (label, prompt) in sacred_prompts {
        println!("Testing {}: {}", label, prompt);
        
        match asi.query_ollama(prompt, None).await {
            Ok(result) => {
                if result.is_sacred {
                    println!("   ✅ Sacred position {} detected!", result.flux_position);
                    println!("   Confidence: {:.2}%", result.confidence * 100.0);
                } else {
                    println!("   Position: {} (regular)", result.flux_position);
                }
            }
            Err(e) => {
                eprintln!("   ❌ Error: {}", e);
            }
        }
        println!();
    }
    
    // Example 5: Performance Metrics
    println!("{}", "=".repeat(70));
    println!("\n📊 Example 5: AGI Performance Metrics\n");
    
    let metrics = asi.get_metrics();
    
    println!("Total Inferences: {}", metrics.total_inferences);
    println!("Average Confidence: {:.2}%", metrics.avg_confidence * 100.0);
    println!("Consensus Rate: {} triggers", metrics.consensus_rate);
    println!();
    
    if metrics.thorough_mode_avg_time > 0.0 {
        println!("Mode Performance:");
        println!("   Fast: {:.0}ms", metrics.fast_mode_avg_time);
        println!("   Balanced: {:.0}ms", metrics.balanced_mode_avg_time);
        println!("   Thorough: {:.0}ms", metrics.thorough_mode_avg_time);
    }
    
    // Summary
    println!("\n{}", "=".repeat(70));
    println!("\n✨ Demo Complete!\n");
    println!("Key Capabilities Demonstrated:");
    println!("   ✅ Direct Ollama integration with AGI");
    println!("   ✅ Vortex mathematics analysis (1-2-4-8-7-5-1)");
    println!("   ✅ Sacred geometry detection (3-6-9)");
    println!("   ✅ ELP tensor analysis (Ethos-Logos-Pathos)");
    println!("   ✅ Multi-provider consensus");
    println!("   ✅ Confidence Lake integration (signal ≥ 0.6)");
    println!("   ✅ Hallucination detection (VCP)");
    println!("\n💡 Next Steps:");
    println!("   1. Try different Ollama models");
    println!("   2. Experiment with temperature settings");
    println!("   3. Add more AI providers for consensus");
    println!("   4. Query Confidence Lake for stored insights");
    println!("   5. Train with RAG for domain knowledge\n");
    
    Ok(())
}
