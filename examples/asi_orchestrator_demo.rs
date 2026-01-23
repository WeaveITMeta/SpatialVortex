//! ASI Orchestrator Demo
//!
//! Demonstrates the unified ASI orchestrator with sacred geometry intelligence
//!
//! Run with:
//! ```bash
//! cargo run --example asi_orchestrator_demo
//! ```

use spatial_vortex::ai::orchestrator::{ASIOrchestrator, ExecutionMode};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧠 ASI Orchestrator Demo");
    println!("========================\n");
    
    // Create orchestrator
    let mut asi = ASIOrchestrator::new().await?;
    
    // Test inputs demonstrating different characteristics
    let test_cases = vec![
        ("Hi", ExecutionMode::Fast),
        ("What is consciousness?", ExecutionMode::Balanced),
        ("How do we reconcile the paradox of free will with deterministic physics?", ExecutionMode::Thorough),
        ("I feel excited about this!", ExecutionMode::Balanced),
        ("According to quantum mechanics and relativity theory...", ExecutionMode::Thorough),
    ];
    
    for (i, (input, mode)) in test_cases.iter().enumerate() {
        println!("Test Case #{}: {:?} mode", i + 1, mode);
        println!("Input: {}", input);
        
        let result = asi.process(input, *mode).await?;
        
        println!("  📊 Results:");
        println!("    Flux Position: {} {}", 
            result.flux_position,
            if result.is_sacred { "⭐ SACRED" } else { "" }
        );
        println!("    ELP Channels:");
        println!("      Ethos (Character):  {:.2}", result.elp.ethos);
        println!("      Logos (Logic):      {:.2}", result.elp.logos);
        println!("      Pathos (Emotion):   {:.2}", result.elp.pathos);
        println!("    Confidence: {:.1}%", result.confidence * 100.0);
        println!("    Processing Time: {}ms", result.processing_time_ms);
        
        if result.is_sacred {
            println!("    🔮 Sacred Position Detected!");
            println!("       +10% confidence boost applied");
        }
        
        if result.confidence < 0.5 {
            println!("    ⚠️  Low confidence - potential hallucination");
        }
        
        println!();
    }
    
    // Demonstrate mode differences
    println!("\n🎯 Mode Comparison");
    println!("==================");
    let input = "What is the nature of reality?";
    
    for mode in &[ExecutionMode::Fast, ExecutionMode::Balanced, ExecutionMode::Thorough] {
        let result = asi.process(input, *mode).await?;
        println!("{:?} mode: {:.1}% confidence, {}ms", 
            mode, 
            result.confidence * 100.0,
            result.processing_time_ms
        );
    }
    
    // Demonstrate sacred geometry
    println!("\n🔮 Sacred Geometry Detection");
    println!("============================");
    
    let mut sacred_count = 0;
    let test_inputs = vec![
        "Test 1",
        "Question?",
        "Exclamation!",
        "Long analytical statement with reasoning",
        "Emotional expression!!!",
        "Balanced and thoughtful reflection",
    ];
    
    for input in &test_inputs {
        let result = asi.process(input, ExecutionMode::Balanced).await?;
        if result.is_sacred {
            println!("  ⭐ Position {}: \"{}\"", result.flux_position, input);
            println!("     Confidence: {:.1}% (boosted)", result.confidence * 100.0);
            sacred_count += 1;
        }
    }
    
    println!("\nFound {} sacred positions out of {} tests", sacred_count, test_inputs.len());
    
    println!("\n✅ Demo Complete!");
    println!("The ASI orchestrator successfully:");
    println!("  - Analyzed input complexity");
    println!("  - Calculated ELP channels (Ethos, Logos, Pathos)");
    println!("  - Determined flux positions (0-9)");
    println!("  - Applied sacred geometry boosts");
    println!("  - Detected potential hallucinations");
    println!("  - Provided execution mode flexibility");
    
    Ok(())
}
