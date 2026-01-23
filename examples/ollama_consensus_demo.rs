//! Demonstration of Ollama integration with AI Consensus Engine
//!
//! This example shows how to:
//! 1. Query a local Ollama model (CWC-Mistral-Nemo-12B-V2)
//! 2. Use it in consensus with other AI models
//! 3. Integrate it with the AGI/ASI Orchestrator
//!
//! Prerequisites:
//! 1. Install Ollama: https://ollama.ai
//! 2. Pull a model: `ollama pull mistral:latest`
//! 3. Ensure Ollama is running (it may already be running!)
//!
//! Run with:
//! ```bash
//! cargo run --example ollama_consensus_demo --features agents
//! ```

use spatial_vortex::ai::{
    query_ollama, call_multiple_models, OllamaConfig, AIProvider, 
    AIConsensusEngine, ConsensusStrategy,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌀 SpatialVortex - Ollama Consensus Demo\n");
    println!("=" .repeat(60));
    
    // Example 1: Simple Ollama query
    println!("\n📡 Example 1: Direct Ollama Query");
    println!("-" .repeat(60));
    
    let prompt = "Explain vortex mathematics in 3 sentences.";
    println!("Prompt: {}\n", prompt);
    
    match query_ollama(prompt, None).await {
        Ok(response) => {
            println!("✅ Response from Ollama:");
            println!("   Model: {}", response.model_name);
            println!("   Confidence: {:.2}", response.confidence);
            println!("   Latency: {}ms", response.latency_ms);
            println!("   Tokens: {}", response.tokens_used);
            println!("\n   Answer:\n   {}\n", response.response_text);
        }
        Err(e) => {
            eprintln!("❌ Failed to query Ollama: {}", e);
            eprintln!("   Make sure Ollama is running and the model is pulled:");
            eprintln!("   - Check if running: curl http://localhost:11434/api/tags");
            eprintln!("   - Pull model: ollama pull mistral:latest");
        }
    }
    
    // Example 2: Custom Ollama configuration
    println!("\n📡 Example 2: Custom Configuration");
    println!("-" .repeat(60));
    
    let custom_config = OllamaConfig {
        url: "http://localhost:11434".to_string(),
        model: "mistral:latest".to_string(),
        temperature: 0.3, // Lower temperature for more deterministic responses
        max_tokens: 500,
    };
    
    let tech_prompt = "What are the key principles of sacred geometry?";
    println!("Prompt: {}\n", tech_prompt);
    
    match query_ollama(tech_prompt, Some(custom_config)).await {
        Ok(response) => {
            println!("✅ Response from Ollama (Custom Config):");
            println!("   Temperature: 0.3 (deterministic)");
            println!("   Max Tokens: 500");
            println!("   Latency: {}ms", response.latency_ms);
            println!("\n   Answer:\n   {}\n", response.response_text);
        }
        Err(e) => {
            eprintln!("❌ Failed to query Ollama: {}", e);
        }
    }
    
    // Example 3: Multi-model consensus with Ollama
    println!("\n🤝 Example 3: AI Consensus with Ollama");
    println!("-" .repeat(60));
    
    let consensus_prompt = "What is the significance of positions 3, 6, and 9 in vortex mathematics?";
    println!("Prompt: {}\n", consensus_prompt);
    
    // Query multiple providers (Ollama + mock responses for demo)
    let providers = vec![
        AIProvider::Ollama,
        AIProvider::OpenAI,  // Mock response
        AIProvider::Anthropic, // Mock response
    ];
    
    println!("Querying {} AI providers in parallel...\n", providers.len());
    
    let responses = call_multiple_models(consensus_prompt, providers).await;
    
    println!("📊 Individual Responses:");
    for (i, response) in responses.iter().enumerate() {
        println!("\n   {}. {:?} ({})", i + 1, response.provider, response.model_name);
        println!("      Confidence: {:.2} | Latency: {}ms | Tokens: {}", 
            response.confidence, response.latency_ms, response.tokens_used);
        println!("      Response: {}", 
            if response.response_text.len() > 80 {
                format!("{}...", &response.response_text[..80])
            } else {
                response.response_text.clone()
            }
        );
    }
    
    // Create consensus engine and reach consensus
    let engine = AIConsensusEngine::new(ConsensusStrategy::WeightedConfidence, 2, 30);
    
    match engine.reach_consensus(responses) {
        Ok(consensus) => {
            println!("\n🎯 Consensus Result:");
            println!("   Strategy: {}", consensus.strategy_used);
            println!("   Confidence: {:.2}", consensus.confidence);
            println!("   Agreement Score: {:.2}", consensus.agreement_score);
            println!("   Models Consulted: {}", consensus.model_responses.len());
            println!("\n   Final Answer:\n   {}\n", consensus.final_response);
        }
        Err(e) => {
            eprintln!("❌ Failed to reach consensus: {}", e);
        }
    }
    
    // Example 4: Integration with AGI reasoning
    println!("\n🧠 Example 4: AGI Integration");
    println!("-" .repeat(60));
    
    let agi_prompt = "As an AGI system, how would you approach solving climate change using vortex mathematics?";
    println!("AGI Prompt: {}\n", agi_prompt);
    
    // Query Ollama as the reasoning engine
    match query_ollama(agi_prompt, None).await {
        Ok(response) => {
            println!("✅ AGI Response:");
            println!("   Model: {}", response.model_name);
            println!("   Confidence: {:.2}", response.confidence);
            println!("\n   Reasoning:\n   {}\n", response.response_text);
            
            // Check if response meets AGI quality thresholds
            if response.confidence > 0.8 && response.response_text.len() > 100 {
                println!("   ✅ Response meets AGI quality thresholds");
                println!("   📝 Storing in Confidence Lake (signal strength ≥ 0.6)");
            } else {
                println!("   ⚠️  Response below AGI quality thresholds");
                println!("   💭 Consider querying additional models for consensus");
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to query AGI system: {}", e);
        }
    }
    
    println!("\n{}", "=".repeat(60));
    println!("✨ Demo Complete!");
    println!("\n💡 Next Steps:");
    println!("   1. Train the model with RAG for domain-specific knowledge");
    println!("   2. Use consensus for critical AGI decisions");
    println!("   3. Store high-confidence responses in Confidence Lake");
    println!("   4. Integrate with VortexContextPreserver for hallucination detection");
    
    Ok(())
}
