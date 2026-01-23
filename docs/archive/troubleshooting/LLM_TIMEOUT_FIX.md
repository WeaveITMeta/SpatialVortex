# LLM Timeout Error Fix

**Last Updated**: October 30, 2025

## Problem

Examples using `CodingAgent` hang or timeout when trying to connect to Ollama:
- `coding_agent_demo` - hangs on LLM generation
- `symbolica_math_demo` - shows "LLM backend not configured"
- Connection to `localhost:11434` times out

## Root Cause

1. **Ollama not running** - Local LLM server not started
2. **Long default timeout** - 30 seconds wait before failure
3. **No pre-flight check** - Doesn't verify Ollama is available first

## Solutions

### **Option 1: Run Ollama (Recommended)**

This enables full AI-powered code generation.

#### Windows:
```powershell
# Install Ollama
winget install Ollama.Ollama

# Start Ollama (runs in background)
ollama serve

# Pull model (in new terminal)
ollama pull codellama:13b    # 7.3GB - High quality
# OR
ollama pull codellama:7b     # 3.8GB - Faster
# OR
ollama pull llama3.2:3b      # 2GB - Fastest
```

#### Linux/Mac:
```bash
# Install
curl -fsSL https://ollama.com/install.sh | sh

# Start service
ollama serve

# Pull model
ollama pull codellama:13b
```

#### Verify Ollama is Running:
```powershell
# Check if Ollama is responding
curl http://localhost:11434/api/tags

# Should return list of installed models
```

### **Option 2: Use Shorter Timeout (Quick Fail)**

Modified `coding_agent_demo.rs` to fail fast (5s instead of 30s):

```rust
let llm_config = LLMConfig {
    backend: LLMBackend::Ollama {
        url: "http://localhost:11434".to_string(),
        model: "codellama:13b".to_string(),
    },
    temperature: 0.2,
    max_tokens: 2048,
    timeout: std::time::Duration::from_secs(5), // ✅ Quick failure
};
```

**Benefits:**
- Fails in 5s instead of 30s
- Clear error message
- Doesn't block testing

### **Option 3: Skip LLM Examples**

Run non-LLM examples only:

```powershell
# These work without Ollama:
cargo run --example asi_orchestrator_demo --release
cargo run --example color_inference_demo --release
cargo run --example quick_benchmarks --release
cargo run --example train_aspect_colors --release

# These require Ollama:
# coding_agent_demo
# symbolica_math_demo (with LLM mode)
```

### **Option 4: Use Mock LLM Backend**

For testing without Ollama, add a mock backend:

```rust
// In future: Add MockLLM backend
let llm_config = LLMConfig {
    backend: LLMBackend::Mock {
        responses: vec!["fn example() { }".to_string()],
    },
    temperature: 0.0,
    max_tokens: 1024,
    timeout: Duration::from_secs(1),
};
```

## Configuration Options

### LLMConfig Fields:

| Field | Default | Purpose |
|-------|---------|---------|
| `backend` | Ollama | LLM provider (Ollama/OpenAI/Mock) |
| `temperature` | 0.2 | Randomness (0=deterministic, 1=creative) |
| `max_tokens` | 2048 | Maximum response length |
| `timeout` | 30s | Connection/response timeout |

### Timeout Recommendations:

| Use Case | Timeout | Why |
|----------|---------|-----|
| Development/Testing | 5-10s | Fail fast if Ollama not running |
| Production | 30-60s | Allow for complex generations |
| CI/CD | 5s | Skip if unavailable |

## Troubleshooting

### "Connection refused" Error

```
❌ Failed: Ollama request failed: error sending request for url (http://localhost:11434/api/generate): error trying to connect: tcp connect error: Connection refused (os error 10061)
```

**Fix**: Ollama is not running
```powershell
ollama serve
```

### "Model not found" Error

```
❌ Ollama returned error: 404 Not Found
```

**Fix**: Model not pulled
```powershell
ollama pull codellama:13b
```

### Timeout After 30s

```
❌ Failed: Ollama request failed: operation timed out
```

**Fixes**:
1. Check Ollama is responding: `curl http://localhost:11434`
2. Reduce timeout to fail faster
3. Use smaller model (codellama:7b)
4. Check firewall/antivirus blocking port 11434

### "LLM backend not configured"

```
❌ Failed: LLM generation error: LLM backend not configured. Use CodingAgent::with_llm()
```

**Fix**: Using `CodingAgent::new()` instead of `CodingAgent::with_llm()`
- Use `with_llm()` for AI-powered generation
- OR use basic mode without LLM

## Performance Tips

### Model Selection:

| Model | Size | Speed | Quality | Use Case |
|-------|------|-------|---------|----------|
| llama3.2:3b | 2GB | Fast | Good | Quick tests |
| codellama:7b | 3.8GB | Medium | Better | Dev/Testing |
| codellama:13b | 7.3GB | Slower | Best | Production |

### Optimization:

1. **Keep Ollama Running**: Start once, reuse
2. **Pre-load Models**: `ollama pull` before running examples
3. **Use GPU**: Ollama auto-detects and uses GPU
4. **Reduce max_tokens**: Lower if generating short code

## Changes Made

**File**: `examples/coding_agent_demo.rs`
- Changed timeout from 30s → 5s
- Faster failure when Ollama unavailable
- Better error messages

**Committed**: [hash to be added]

## Testing

After fixes:

```powershell
# Test with Ollama running
ollama serve
cargo run --example coding_agent_demo --release
# Should work

# Test without Ollama
# (stop ollama)
cargo run --example coding_agent_demo --release
# Should fail in 5s with clear message
```

## Future Improvements

1. **Pre-flight Check**: Ping Ollama before initialization
2. **Mock Backend**: For testing without Ollama
3. **Retry Logic**: Auto-retry on transient failures
4. **Model Auto-pull**: Download model if missing
5. **Connection Pool**: Reuse HTTP connections

---

**Status**: ✅ Fixed (timeout reduced to 5s)
**Tested**: Verified fast failure
**Next**: Add mock backend for CI/CD
