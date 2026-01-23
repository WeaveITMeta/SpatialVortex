#!/usr/bin/env cargo
//! Multi-Model Local Consensus with Ollama
//! 
//! Demonstrates AGI-level consensus using multiple local LLMs:
//! - llama3.2:latest (2GB - Fast, efficient)
//! - mixtral:8x7b (26GB - High quality)
//! - codellama:13b (7.4GB - Code specialist)
//! 
//! All running locally without external API calls!

use spatial_vortex::ai::consensus::{
    query_multiple_ollama, AIConsensusEngine, ConsensusStrategy, OllamaConfig,
};
use spatial_vortex::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🌀 SpatialVortex Multi-Model Local Consensus Demo\n");
    println!("{}", "=".repeat(70));
    println!("\n🤖 Available Local Models:");
    println!("   1. llama3.2:latest    - Fast & Efficient (2GB)");
    println!("   2. mixtral:8x7b       - High Quality (26GB)");
    println!("   3. codellama:13b      - Code Specialist (7.4GB)");
    println!("   4. CWC-Mistral-Nemo   - Custom Model (12B)");
    println!("\n💡 All models run locally - no external API calls!\n");
    println!("{}", "=".repeat(70));
    
    // Configure models for consensus
    let models = vec![
        "llama3.2:latest",
        "mixtral:8x7b",
        "codellama:13b",
        "hf.co/CWClabs/CWC-Mistral-Nemo-12B-V2-q4_k_m:latest",
    ];
    
    // Create consensus engine (needs minimum 2 models for basic consensus)
    let engine = AIConsensusEngine::new(ConsensusStrategy::WeightedConfidence, 2, 300);
    
    // Base configuration for all models
    let config = OllamaConfig {
        url: "http://localhost:11434".to_string(),
        model: "".to_string(), // Will be overridden per model
        temperature: 0.7,
        max_tokens: 500,
    };
    
    // Test questions that benefit from multi-model perspective
    let questions = vec![
        (
            "What is vortex mathematics?",
            "Mathematical/theoretical question"
        ),
        (
            "Write a Rust function to calculate Fibonacci numbers",
            "Code generation task"
        ),
        (
            "Explain the relationship between positions 3, 6, and 9",
            "Sacred geometry question"
        ),
    ];
    
    for (i, (question, category)) in questions.iter().enumerate() {
        println!("\n{}", "=".repeat(70));
        println!("📊 Question {} - {}", i + 1, category);
        println!("{}", "=".repeat(70));
        println!("\n❓ {}\n", question);
        
        println!("🔄 Querying {} models in parallel...", models.len());
        
        // Query all models in parallel
        let start = std::time::Instant::now();
        match query_multiple_ollama(question, models.clone(), Some(config.clone())).await {
            Ok(responses) => {
                let query_time = start.elapsed();
                
                println!("✅ Received {} responses in {:.1}s\n", responses.len(), query_time.as_secs_f64());
                
                // Show individual model responses
                println!("🤖 Individual Model Responses:");
                for (idx, response) in responses.iter().enumerate() {
                    println!("\n   Model {}: {}", idx + 1, response.model_name);
                    println!("   Confidence: {:.2}%", response.confidence * 100.0);
                    println!("   Latency: {}ms", response.latency_ms);
                    println!("   Tokens: {}", response.tokens_used);
                    
                    // Show preview of response
                    let preview: String = response.response_text
                        .chars()
                        .take(150)
                        .collect();
                    println!("   Preview: {}...", preview);
                }
                
                // Reach consensus
                println!("\n🎯 Computing Consensus...");
                match engine.reach_consensus(responses) {
                    Ok(consensus) => {
                        println!("\n✨ CONSENSUS ACHIEVED ✨");
                        println!("   Strategy: {}", consensus.strategy_used);
                        println!("   Final Confidence: {:.2}%", consensus.confidence * 100.0);
                        println!("   Agreement Score: {:.2}%", consensus.agreement_score * 100.0);
                        println!("   Models Used: {}", consensus.model_responses.len());
                        
                        println!("\n📝 Consensus Response:");
                        println!("{}", "-".repeat(70));
                        println!("{}", consensus.final_response);
                        println!("{}", "-".repeat(70));
                        
                        // Show voting breakdown if available
                        if !consensus.voting_breakdown.is_empty() {
                            println!("\n🗳️  Voting Breakdown:");
                            for (key, votes) in &consensus.voting_breakdown {
                                println!("   {} - {} votes", key, votes);
                            }
                        }
                    }
                    Err(e) => {
                        println!("❌ Consensus failed: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("❌ Failed to query models: {}", e);
                println!("\n💡 Make sure Ollama is running: ollama serve");
                println!("   And models are installed:");
                for model in &models {
                    println!("   - ollama pull {}", model);
                }
            }
        }
    }
    
    println!("\n{}", "=".repeat(70));
    println!("✨ Demo Complete!\n");
    
    println!("📊 Key Benefits of Multi-Model Consensus:");
    println!("   ✅ Diverse perspectives from specialized models");
    println!("   ✅ Higher confidence through agreement");
    println!("   ✅ Error correction via majority voting");
    println!("   ✅ 100% local - no external API dependencies");
    println!("   ✅ Cost-free unlimited queries");
    println!("\n💡 Next Steps:");
    println!("   1. Add more models for stronger consensus");
    println!("   2. Experiment with different consensus strategies");
    println!("   3. Use consensus for critical AGI decisions");
    println!("   4. Integrate with Confidence Lake for learning");
    println!("   5. Compare with native Rust AI as it evolves\n");
    
    Ok(())
}
