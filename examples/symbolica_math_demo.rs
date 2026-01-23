//! Demo of symbolic mathematics integration with the coding agent
//!
//! Run with: cargo run --example symbolica_math_demo --features agents
//!
//! This demonstrates how the agent uses symbolic math (Symbolica fallback)
//! to solve equations, differentiate, integrate, and generate code.

use spatial_vortex::agents::{CodingAgent, AgentConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🔬 SpatialVortex Symbolic Mathematics Demo\n");
    println!("Note: Using fallback symbolic math (Symbolica integration pending)");
    println!("{}\n", "=".repeat(80));
    
    // Create agent without LLM (uses symbolic math)
    let agent = CodingAgent::with_config(AgentConfig::default());
    
    // Demo 1: Simplify expression
    println!("📝 Demo 1: Simplify Expression");
    println!("Task: Simplify \"x + 0\"\n");
    
    match agent.execute_task("Simplify x + 0").await {
        Ok(result) => {
            println!("Language: {}", result.language.name());
            println!("Flux Position: {:?} (Logos - Math)", result.flux_position);
            println!("\nGenerated Code:");
            println!("{}", result.code);
        }
        Err(e) => {
            println!("❌ Failed: {}", e);
        }
    }
    
    println!("\n{}\n", "=".repeat(80));
    
    // Demo 2: Differentiate
    println!("📝 Demo 2: Differentiate");
    println!("Task: Differentiate x^2 with respect to x\n");
    
    match agent.execute_task("Differentiate x^2 with respect to x").await {
        Ok(result) => {
            println!("Language: {}", result.language.name());
            println!("Flux Position: {:?} (Logos - Math)", result.flux_position);
            println!("\nGenerated Code:");
            println!("{}", result.code);
        }
        Err(e) => {
            println!("❌ Failed: {}", e);
        }
    }
    
    println!("\n{}\n", "=".repeat(80));
    
    // Demo 3: Factor expression
    println!("📝 Demo 3: Factor Expression");
    println!("Task: Factor \"x^2 - 1\"\n");
    
    match agent.execute_task("Factor x^2 - 1").await {
        Ok(result) => {
            println!("Language: {}", result.language.name());
            println!("Flux Position: {:?} (Logos - Math)", result.flux_position);
            println!("\nGenerated Code:");
            println!("{}", result.code);
        }
        Err(e) => {
            println!("❌ Failed: {}", e);
        }
    }
    
    println!("\n{}\n", "=".repeat(80));
    
    // Demo 4: Expand expression
    println!("📝 Demo 4: Expand Expression");
    println!("Task: Expand \"(x + 1)^2\"\n");
    
    match agent.execute_task("Expand (x + 1)^2").await {
        Ok(result) => {
            println!("Language: {}", result.language.name());
            println!("Flux Position: {:?} (Logos - Math)", result.flux_position);
            println!("\nGenerated Code:");
            println!("{}", result.code);
        }
        Err(e) => {
            println!("❌ Failed: {}", e);
        }
    }
    
    println!("\n{}\n", "=".repeat(80));
    
    // Demo 5: Generate code in different languages
    println!("📝 Demo 5: Generate Rust Code for Math");
    println!("Task: Simplify Rust code for x^2 + sqrt(x)\n");
    
    match agent.execute_task("Simplify x^2 + sqrt(x) in Rust").await {
        Ok(result) => {
            println!("Language: {}", result.language.name());
            println!("Flux Position: {:?} (Logos - Math)", result.flux_position);
            println!("\nGenerated Code:");
            println!("{}", result.code);
        }
        Err(e) => {
            println!("❌ Failed: {}", e);
        }
    }
    
    println!("\n✨ Demo complete!");
    println!("\n📊 Symbolic Math Features:");
    println!("   ✅ Simplification (basic patterns)");
    println!("   ✅ Differentiation (power rule)");
    println!("   ✅ Factoring (difference of squares)");
    println!("   ✅ Expansion (binomial squares)");
    println!("   ✅ Multi-language code generation");
    println!("\n🚀 Future: Full Symbolica integration (10x faster than SymPy)");
    
    Ok(())
}
