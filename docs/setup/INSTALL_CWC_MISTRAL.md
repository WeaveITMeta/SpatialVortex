# Installing CWC-Mistral-Nemo-12B for Ollama

## Option 1: Direct Import from HuggingFace

The model files are in GGUF format on HuggingFace. Here's how to get them:

### Step 1: Check Available Files

Visit: https://huggingface.co/CWClabs/CWC-Mistral-Nemo-12B-V2-q4_k_m/tree/main

Look for files ending in `.gguf` - the naming might be:
- `model.gguf`
- `ggml-model-Q4_K_M.gguf`
- Or similar variations

### Step 2: Download Using Git LFS (Recommended)

```bash
# Install git-lfs if you don't have it
git lfs install

# Clone the repo (this will download the model)
cd %USERPROFILE%\.ollama\models
git clone https://huggingface.co/CWClabs/CWC-Mistral-Nemo-12B-V2-q4_k_m
```

### Step 3: Create Ollama Model

Once downloaded, create a `Modelfile`:

```bash
cd %USERPROFILE%\.ollama\models\CWC-Mistral-Nemo-12B-V2-q4_k_m

# Create Modelfile (use the actual GGUF filename you find)
@"
FROM ./model.gguf

PARAMETER temperature 0.7
PARAMETER top_p 0.9
PARAMETER top_k 40
PARAMETER num_ctx 4096
PARAMETER repeat_penalty 1.1

SYSTEM You are CWC-Mistral-Nemo, a highly capable AI assistant.
"@ | Out-File -FilePath Modelfile -Encoding UTF8

# Create the model in Ollama
ollama create cwc-mistral-nemo -f Modelfile
```

### Step 4: Test It

```bash
ollama run cwc-mistral-nemo "Hello!"
```

### Step 5: Update SpatialVortex

Edit `src/ai/consensus.rs` line 325:

```rust
model: "cwc-mistral-nemo".to_string(),
```

---

## Option 2: Use Existing Models

You already have excellent models installed:

### Quick Comparison:

| Model | Size | Best For | Speed |
|-------|------|----------|-------|
| **llama3.2** | 2GB | General, Fast | ⚡⚡⚡⚡⚡ |
| **codellama:13b** | 7.4GB | Code generation | ⚡⚡⚡ |
| **mixtral:8x7b** | 26GB | High quality | ⚡⚡ |
| **deepseek-v3** | 404GB | Maximum capability | ⚡ |

### To Use Different Models:

Edit your code or pass custom config:

```rust
// Option 1: Edit default in src/ai/consensus.rs
model: "codellama:13b".to_string(),  // For code tasks
// or
model: "mixtral:8x7b".to_string(),   // For best quality

// Option 2: Pass custom config
let config = OllamaConfig {
    url: "http://localhost:11434".to_string(),
    model: "deepseek-v3:latest".to_string(),  // Most powerful!
    temperature: 0.7,
    max_tokens: 2000,
};
let response = query_ollama(prompt, Some(config)).await?;
```

---

## Option 3: Alternative GGUF Models

If CWC-Mistral is hard to find, try these similar models:

### Mistral-Nemo Variants:

```bash
# Official Mistral Nemo (12B)
ollama pull mistral-nemo

# Or standard Mistral (7B, smaller/faster)
ollama pull mistral:latest
```

---

## Troubleshooting

### "404 Not Found" from HuggingFace

The repo might:
- Use a different file naming convention
- Require you to accept a license (visit the repo in browser first)
- Be in a different format than expected

**Solution**: Visit the repo in your browser, click "Files and versions", and manually download the `.gguf` file.

### Model Too Large

CWC-Mistral-Nemo-12B at Q4_K_M is ~5-7GB. With your GTX 1080 Ti (11GB VRAM), you can run:
- ✅ Models up to ~10GB
- ✅ Llama3.2, Mistral, CodeLlama easily
- ✅ Mixtral (will use some system RAM)
- ⚠️ DeepSeek-V3 (404GB) will be very slow without distributed setup

---

## Current Setup

✅ **You're already configured to use `llama3.2:latest`**

Run the demo now:
```bash
cargo run --example asi_ollama_demo --features agents
```

To switch models later, just edit line 325 in `src/ai/consensus.rs`!
