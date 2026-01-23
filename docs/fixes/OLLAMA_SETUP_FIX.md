# 🔧 Ollama Setup - Issue Resolution

## Issues Encountered & Fixed

### ✅ Issue 1: Model Not Available (FIXED)

**Error**: `400: {"error":"The specified tag is not available in the repository"}`

**Root Cause**: The CWC-Mistral-Nemo model isn't available in Ollama's public registry.

**Solution**: Updated default model to use standard Ollama models.

### ✅ Issue 2: Port Already in Use (NOT AN ERROR)

**Error**: `listen tcp 127.0.0.1:11434: bind: Only one usage of each socket address`

**Root Cause**: Ollama server is already running!

**Solution**: This is actually good - just use the existing server. No action needed.

### ✅ Issue 3: Cargo Build Warning (FIXED)

**Error**: `file found to be present in multiple build targets`

**Root Cause**: `academic_benchmark` was defined as both `[[bin]]` and `[[bench]]`.

**Solution**: Changed to proper `[[bench]]` target in Cargo.toml.

---

## 🚀 Updated Quick Start

### 1. Verify Ollama is Running
```powershell
# Check if Ollama is already running
curl http://localhost:11434/api/tags
```

If you see output, Ollama is running! Skip to step 3.

### 2. Pull an Available Model

Choose one of these models:

#### Option A: Mistral (Recommended - Fast & Smart)
```bash
ollama pull mistral:latest
```

#### Option B: Llama 3.2 (Latest from Meta)
```bash
ollama pull llama3.2:latest
```

#### Option C: Codellama (Best for Code)
```bash
ollama pull codellama:latest
```

#### Option D: Phi-3 (Smallest & Fastest)
```bash
ollama pull phi3:latest
```

### 3. Test the Connection
```bash
curl http://localhost:11434/api/generate -d '{
  "model": "mistral:latest",
  "prompt": "Hello!",
  "stream": false
}'
```

### 4. Run SpatialVortex Examples

```bash
# Run the demo with updated model
cargo run --example ollama_consensus_demo --features agents
```

---

## 📝 Using Custom Models

If you want to use a specific model, configure it in your code:

```rust
use spatial_vortex::ai::OllamaConfig;

let config = OllamaConfig {
    url: "http://localhost:11434".to_string(),
    model: "mistral:latest".to_string(),  // Or any installed model
    temperature: 0.7,
    max_tokens: 2000,
};

let response = query_ollama("Your question", Some(config)).await?;
```

### Available Models to Try

| Model | Size | Best For | Speed |
|-------|------|----------|-------|
| `mistral:latest` | 7B | General intelligence | ⚡⚡⚡ |
| `llama3.2:latest` | 3B | Latest Meta model | ⚡⚡⚡⚡ |
| `codellama:latest` | 7B | Code generation | ⚡⚡⚡ |
| `phi3:latest` | 3.8B | Fast responses | ⚡⚡⚡⚡⚡ |
| `gemma:latest` | 2B | Smallest, fastest | ⚡⚡⚡⚡⚡ |
| `llama3.1:8b` | 8B | Balanced | ⚡⚡⚡ |
| `llama3.1:70b` | 70B | Highest quality | ⚡ |

### Pull Multiple Models
```bash
# Pull several models for consensus
ollama pull mistral:latest
ollama pull llama3.2:latest
ollama pull codellama:latest
```

---

## 🎯 Verify Everything Works

### Test 1: Direct Query
```rust
use spatial_vortex::query_ollama;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let response = query_ollama("What is 2+2?", None).await?;
    println!("{}", response.response_text);
    Ok(())
}
```

### Test 2: List Available Models
```bash
ollama list
```

### Test 3: Run Full Demo
```bash
cargo run --example asi_ollama_demo --features agents
```

---

## 💡 Tips

### If Ollama Stops Responding
```bash
# On Windows (PowerShell as Administrator)
Stop-Process -Name "ollama" -Force
ollama serve
```

### If You Want to Use Multiple Models
```rust
// Query different models in consensus
use spatial_vortex::ai::{AIProvider, OllamaConfig};

let mistral_config = OllamaConfig {
    model: "mistral:latest".to_string(),
    ..Default::default()
};

let llama_config = OllamaConfig {
    model: "llama3.2:latest".to_string(),
    ..Default::default()
};

// Query both for consensus
let response1 = query_ollama(prompt, Some(mistral_config)).await?;
let response2 = query_ollama(prompt, Some(llama_config)).await?;
```

### Change Default Model Globally

Edit `src/ai/consensus.rs`:
```rust
impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:11434".to_string(),
            model: "llama3.2:latest".to_string(),  // Change this
            temperature: 0.7,
            max_tokens: 2000,
        }
    }
}
```

---

## ✅ All Fixed! Now What?

1. **Verify Ollama is running**: `curl http://localhost:11434/api/tags`
2. **Pull a model**: `ollama pull mistral:latest`
3. **Run the demo**: `cargo run --example asi_ollama_demo --features agents`
4. **Enjoy local AGI!** 🎉

---

## 📚 Documentation Updated

All documentation has been updated to use available models:
- ✅ `OLLAMA_QUICKSTART.md`
- ✅ `docs/OLLAMA_INTEGRATION.md`
- ✅ `examples/ollama_consensus_demo.rs`
- ✅ `examples/asi_ollama_demo.rs`

---

**Status**: ✅ All Issues Resolved  
**Date**: November 9, 2025
