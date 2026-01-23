//! Demo of the multi-language coding agent with LLM integration
//!
//! Run with: cargo run --example coding_agent_demo --features agents
//!
//! Prerequisites:
//! 1. Install Ollama: curl -fsSL https://ollama.com/install.sh | sh
//! 2. Pull a code model: ollama pull codellama:13b
//! 3. Start Ollama: ollama serve

use spatial_vortex::agents::{CodingAgent, AgentConfig, LLMConfig, LLMBackend};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🤖 SpatialVortex Coding Agent Demo\n");
    
    // Configure agent with Ollama backend
    let agent_config = AgentConfig {
        max_correction_attempts: 3,
        enable_self_correction: true,
        enable_flux_routing: true,
        ..Default::default()
    };
    
    let llm_config = LLMConfig {
        backend: LLMBackend::Ollama {
            url: "http://localhost:11434".to_string(),
            model: "codellama:13b".to_string(),
        },
        temperature: 0.2, // Low for deterministic code
        max_tokens: 2048,
        timeout: std::time::Duration::from_secs(5), // Shorter timeout for quick failure
    };
    
    println!("📡 Connecting to Ollama at http://localhost:11434...");
    
    let agent = match CodingAgent::with_llm(agent_config, llm_config) {
        Ok(agent) => {
            println!("✅ Agent initialized with LLM backend\n");
            agent
        }
        Err(e) => {
            eprintln!("❌ Failed to initialize agent: {}", e);
            eprintln!("\n💡 Make sure Ollama is running:");
            eprintln!("   1. Install: curl -fsSL https://ollama.com/install.sh | sh");
            eprintln!("   2. Pull model: ollama pull codellama:13b");
            eprintln!("   3. Start server: ollama serve");
            return Ok(());
        }
    };
    
    // Demo 1: Python - Position 9 (Logos/Math)
    println!("📝 Demo 1: Python Math Algorithm (Logos Position 9)");
    println!("Task: Write a function to calculate Fibonacci numbers\n");
    
    match agent.execute_task("Write a Python function to calculate Fibonacci numbers").await {
        Ok(result) => {
            println!("Language: {}", result.language.name());
            println!("Flux Position: {:?}", result.flux_position);
            println!("Attempts: {}", result.attempts);
            println!("\nGenerated Code:");
            println!("```python\n{}\n```\n", result.code);
            
            if let Some(exec) = result.execution {
                if exec.success {
                    println!("✅ Execution successful!");
                    if !exec.stdout.is_empty() {
                        println!("Output:\n{}", exec.stdout);
                    }
                } else {
                    println!("❌ Execution failed:");
                    println!("{}", exec.stderr);
                }
            }
        }
        Err(e) => {
            println!("❌ Failed: {}", e);
        }
    }
    
    println!("\n{}\n", "=".repeat(80));
    
    // Demo 2: Rust - Position 3 (Ethos/Architecture)
    println!("📝 Demo 2: Rust Architecture (Ethos Position 3)");
    println!("Task: Design a clean trait-based plugin system\n");
    
    match agent.execute_task("Design a clean Rust trait-based plugin system").await {
        Ok(result) => {
            println!("Language: {}", result.language.name());
            println!("Flux Position: {:?}", result.flux_position);
            println!("Attempts: {}", result.attempts);
            println!("\nGenerated Code:");
            println!("```rust\n{}\n```\n", result.code);
        }
        Err(e) => {
            println!("❌ Failed: {}", e);
        }
    }
    
    println!("\n{}\n", "=".repeat(80));
    
    // Demo 3: JavaScript - Position 6 (Pathos/UX)
    println!("📝 Demo 3: JavaScript UI Component (Pathos Position 6)");
    println!("Task: Create a beautiful button component\n");
    
    match agent.execute_task("Create a beautiful React button component with hover effects").await {
        Ok(result) => {
            println!("Language: {}", result.language.name());
            println!("Flux Position: {:?}", result.flux_position);
            println!("Attempts: {}", result.attempts);
            println!("\nGenerated Code:");
            println!("```javascript\n{}\n```\n", result.code);
        }
        Err(e) => {
            println!("❌ Failed: {}", e);
        }
    }
    
    println!("\n{}\n", "=".repeat(80));
    
    // Demo 4: Elixir - Functional programming
    println!("📝 Demo 4: Elixir GenServer (Functional)");
    println!("Task: Create a counter GenServer\n");
    
    match agent.execute_task("Create an Elixir GenServer for a simple counter").await {
        Ok(result) => {
            println!("Language: {}", result.language.name());
            println!("Flux Position: {:?}", result.flux_position);
            println!("Attempts: {}", result.attempts);
            println!("\nGenerated Code:");
            println!("```elixir\n{}\n```\n", result.code);
        }
        Err(e) => {
            println!("❌ Failed: {}", e);
        }
    }
    
    println!("\n✨ Demo complete!");
    println!("\n📊 Sacred Geometry Routing:");
    println!("   Position 3 (Ethos) → Architecture/Design");
    println!("   Position 6 (Pathos) → UI/UX");
    println!("   Position 9 (Logos) → Math/Logic");
    
    Ok(())
}
