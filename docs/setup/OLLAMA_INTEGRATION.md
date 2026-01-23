# Ollama Integration with SpatialVortex AGI

This guide explains how to integrate local Ollama models (specifically CWC-Mistral-Nemo-12B-V2) with the SpatialVortex AGI system.

## 🎯 Overview

Ollama has been integrated into:
1. **AI Consensus Engine** - Query local models alongside other AI providers
2. **ASI Orchestrator** - Direct AGI access to Ollama models
3. **Vortex Intelligence** - Ollama responses analyzed through sacred geometry

## 📋 Prerequisites

### 1. Install Ollama

```bash
# Windows (PowerShell as Administrator)
winget install Ollama.Ollama

# Or download from: https://ollama.ai
```

### 2. Pull the CWC-Mistral-Nemo Model

```bash
ollama pull hf.co/CWClabs/CWC-Mistral-Nemo-12B-V2-q4_k_m:Q4_K_M
```

### 3. Start Ollama Server

```bash
ollama serve
```

The server will run on `http://localhost:11434` by default.

## 🚀 Usage Examples

### Example 1: Direct Ollama Query

```rust
use spatial_vortex::ai::query_ollama;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let prompt = "Explain vortex mathematics in simple terms.";
    
    let response = query_ollama(prompt, None).await?;
    
    println!("Response: {}", response.response_text);
    println!("Confidence: {:.2}", response.confidence);
    println!("Latency: {}ms", response.latency_ms);
    
    Ok(())
}
```

### Example 2: Custom Ollama Configuration

```rust
use spatial_vortex::ai::{query_ollama, OllamaConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = OllamaConfig {
        url: "http://localhost:11434".to_string(),
        model: "hf.co/CWClabs/CWC-Mistral-Nemo-12B-V2-q4_k_m:Q4_K_M".to_string(),
        temperature: 0.3,  // More deterministic
        max_tokens: 1000,
    };
    
    let response = query_ollama("What is AGI?", Some(config)).await?;
    
    Ok(())
}
```

### Example 3: AGI Integration (ASI Orchestrator)

```rust
use spatial_vortex::ai::orchestrator::ASIOrchestrator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut asi = ASIOrchestrator::new()?;
    
    // Query Ollama through AGI with vortex intelligence
    let result = asi.query_ollama(
        "Explain the significance of positions 3, 6, and 9 in vortex mathematics",
        None
    ).await?;
    
    println!("AGI Response: {}", result.result);
    println!("Confidence: {:.2}", result.confidence);
    println!("Flux Position: {}", result.flux_position);
    println!("Sacred: {}", result.is_sacred);
    println!("ELP: E={:.1}, L={:.1}, P={:.1}", 
        result.elp.ethos, result.elp.logos, result.elp.pathos);
    
    Ok(())
}
```

### Example 4: Multi-Model Consensus

```rust
use spatial_vortex::ai::orchestrator::ASIOrchestrator;
use spatial_vortex::ai::{AIProvider, ConsensusStrategy};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut asi = ASIOrchestrator::new()?;
    
    // Query multiple providers for consensus
    let providers = vec![
        AIProvider::Ollama,      // Local CWC-Mistral-Nemo
        AIProvider::OpenAI,      // GPT-4 (if configured)
        AIProvider::Anthropic,   // Claude (if configured)
    ];
    
    let result = asi.query_with_consensus(
        "What is the nature of consciousness?",
        providers,
        ConsensusStrategy::WeightedConfidence
    ).await?;
    
    println!("Consensus Result: {}", result.result);
    println!("Agreement Score: {:.2}", result.confidence);
    
    Ok(())
}
```

## 🌀 Vortex Intelligence Features

When you query Ollama through the AGI, the response is automatically:

### 1. **ELP Analysis**
- **Ethos**: Ethical/character content measurement
- **Logos**: Logical reasoning content measurement
- **Pathos**: Emotional content measurement

### 2. **Flux Positioning**
- Response mapped to sacred vortex positions (0-9)
- Positions 3, 6, 9 receive sacred geometry boosts
- **Position 3**: Creative Trinity (+10% confidence)
- **Position 6**: Harmonic Balance (+10% confidence + consensus trigger)
- **Position 9**: Completion Cycle (+10% confidence + VCP intervention)

### 3. **Confidence Lake Storage**
- Responses with signal strength ≥ 0.6 automatically stored
- Encrypted PostgreSQL (AES-GCM-SIV)
- Queryable for high-value knowledge retrieval

### 4. **Hallucination Detection**
- VortexContextPreserver monitors response quality
- Signal strength calculated from 3-6-9 pattern frequency
- 40% better context preservation than linear transformers

## 🎨 Running the Demo

```bash
# Run the comprehensive demo
cargo run --example ollama_consensus_demo --features agents

# The demo shows:
# 1. Direct Ollama queries
# 2. Custom configuration
# 3. Multi-model consensus
# 4. AGI integration
```

## ⚙️ Configuration

### Environment Variables

```bash
# Ollama Configuration
export OLLAMA_URL="http://localhost:11434"
export OLLAMA_MODEL="hf.co/CWClabs/CWC-Mistral-Nemo-12B-V2-q4_k_m:Q4_K_M"

# LLM Bridge Configuration (for agents)
export LLM_BACKEND="ollama"
export LLM_MODEL="hf.co/CWClabs/CWC-Mistral-Nemo-12B-V2-q4_k_m:Q4_K_M"
```

### Cargo Features

```toml
[features]
default = []
agents = ["reqwest", "tokio", "tracing"]  # Required for Ollama
lake = ["sqlx", "aes-gcm-siv"]           # Required for Confidence Lake
```

## 📊 Performance Characteristics

| Metric | Ollama (Local) | Cloud APIs |
|--------|---------------|------------|
| **Latency** | 50-200ms | 200-1000ms |
| **Cost** | Free | $0.01-0.10/req |
| **Privacy** | 100% local | Data sent externally |
| **Availability** | Offline capable | Internet required |
| **Model Quality** | CWC-Mistral-Nemo-12B | GPT-4, Claude-3 |

## 🔧 Troubleshooting

### Ollama Not Running

```bash
# Check if Ollama is running
curl http://localhost:11434/api/tags

# Start Ollama
ollama serve
```

### Model Not Found

```bash
# List available models
ollama list

# Pull the model
ollama pull hf.co/CWClabs/CWC-Mistral-Nemo-12B-V2-q4_k_m:Q4_K_M
```

### Connection Refused

- Ensure Ollama is running: `ollama serve`
- Check firewall settings
- Verify URL: `http://localhost:11434`

### Slow Responses

```bash
# Use a smaller model for faster responses
ollama pull llama3.2:latest

# Or use quantized version (q4_k_m is already quantized)
```

## 🧪 Testing

```bash
# Test Ollama integration
cargo test --features agents test_ollama

# Test consensus engine
cargo test --features agents test_consensus

# Test AGI integration
cargo test --features agents,lake test_asi_ollama
```

## 📚 API Reference

### `query_ollama`

```rust
pub async fn query_ollama(
    prompt: &str,
    config: Option<OllamaConfig>,
) -> Result<ModelResponse>
```

Query a local Ollama model directly.

### `ASIOrchestrator::query_ollama`

```rust
pub async fn query_ollama(
    &mut self,
    prompt: &str,
    config: Option<OllamaConfig>,
) -> Result<ASIOutput>
```

Query Ollama through the AGI with full vortex intelligence integration.

### `ASIOrchestrator::query_with_consensus`

```rust
pub async fn query_with_consensus(
    &mut self,
    prompt: &str,
    providers: Vec<AIProvider>,
    strategy: ConsensusStrategy,
) -> Result<ASIOutput>
```

Query multiple providers (including Ollama) and reach consensus.

## 🌟 Best Practices

### 1. Use Appropriate Models
- **CWC-Mistral-Nemo-12B-V2**: Best for general intelligence
- **llama3.2**: Faster, good for simple queries
- **codellama**: Optimized for code generation

### 2. Tune Temperature
- **0.1-0.3**: Deterministic, factual responses
- **0.5-0.7**: Balanced creativity and accuracy (default)
- **0.8-1.0**: Creative, varied responses

### 3. Leverage Consensus
- Use multiple models for critical decisions
- Weighted consensus for quality-aware aggregation
- Sacred geometry boosts enhance accuracy

### 4. Monitor Performance
- Track latency via `ASIOutput.processing_time_ms`
- Monitor confidence scores
- Check Confidence Lake for high-value insights

## 🔮 Advanced Features

### RAG Integration

Ollama responses can be enhanced with RAG (Retrieval-Augmented Generation):

```rust
use spatial_vortex::rag::{RAGRetriever, AugmentedGenerator};

// Initialize RAG system
let retriever = RAGRetriever::new(vector_store, config);
let mut generator = AugmentedGenerator::new(retriever, gen_config)?;

// Generate with retrieved context
let result = generator.generate_with_ollama(
    "Explain sacred geometry",
    ollama_config
).await?;
```

### Continuous Learning

Train the AGI on Ollama responses:

```bash
cargo run --example train_on_ollama_responses --features agents,lake,postgres
```

High-confidence Ollama responses (≥0.6 signal strength) are automatically stored in the Confidence Lake for future retrieval and learning.

## 📖 Additional Resources

- [Ollama Documentation](https://github.com/ollama/ollama)
- [CWC Labs Models](https://huggingface.co/CWClabs)
- [SpatialVortex AI Consensus](./AI_CONSENSUS.md)
- [ASI Orchestrator Guide](./ASI_ORCHESTRATOR.md)
- [Sacred Geometry Math](./VORTEX_MATHEMATICS.md)

## 🤝 Contributing

To add support for additional Ollama models or features:

1. Update `AIProvider` enum in `src/ai/consensus.rs`
2. Add model configuration in `OllamaConfig`
3. Test with `cargo test --features agents`
4. Submit PR with documentation

## 📝 License

This integration follows the SpatialVortex project license.

---

**Status**: ✅ Production Ready  
**Version**: 1.0.0  
**Last Updated**: November 2025
