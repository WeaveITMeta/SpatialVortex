# 🚀 Ollama Quick Start Guide

## Setup (5 minutes)

### 1. Install Ollama
```powershell
winget install Ollama.Ollama
```

### 2. Pull the Model
```bash
# Recommended: Mistral (7B, fast and smart)
ollama pull mistral:latest

# Or choose: llama3.2, codellama, phi3
```

### 3. Start the Server
```bash
ollama serve
```

## Usage

### Option 1: Direct Query
```rust
use spatial_vortex::query_ollama;

let response = query_ollama("Your question here", None).await?;
println!("{}", response.response_text);
```

### Option 2: Through AGI (Recommended)
```rust
use spatial_vortex::ai::orchestrator::ASIOrchestrator;

let mut asi = ASIOrchestrator::new()?;
let result = asi.query_ollama("Your question", None).await?;

println!("Response: {}", result.result);
println!("Confidence: {:.2}%", result.confidence * 100.0);
println!("Flux Position: {}", result.flux_position);
```

### Option 3: Multi-Model Consensus
```rust
use spatial_vortex::ai::{AIProvider, ConsensusStrategy};

let providers = vec![AIProvider::Ollama, AIProvider::OpenAI];
let result = asi.query_with_consensus(
    "Your question",
    providers,
    ConsensusStrategy::WeightedConfidence
).await?;
```

## Run Examples

```bash
# Comprehensive demo
cargo run --example ollama_consensus_demo --features agents

# AGI integration demo
cargo run --example asi_ollama_demo --features agents
```

## Key Features

- ✅ **Local & Private**: All processing on your machine
- ✅ **Vortex Intelligence**: Sacred geometry analysis (3-6-9)
- ✅ **ELP Analysis**: Ethos-Logos-Pathos measurement
- ✅ **Confidence Lake**: Auto-storage of high-quality responses (≥0.6)
- ✅ **Hallucination Detection**: VortexContextPreserver monitoring
- ✅ **Multi-Provider**: Consensus with OpenAI, Anthropic, etc.

## Troubleshooting

**Connection refused?**
```bash
ollama serve
```

**Model not found?**
```bash
ollama list
ollama pull mistral:latest
```

**Slow responses?**
- Use a smaller model: `ollama pull llama3.2`
- Lower max_tokens in config
- Check system resources

## Full Documentation

See [OLLAMA_INTEGRATION.md](./docs/OLLAMA_INTEGRATION.md) for complete guide.
