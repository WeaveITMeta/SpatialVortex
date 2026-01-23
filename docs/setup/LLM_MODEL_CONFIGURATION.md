# 🤖 LLM Model Configuration

**Fixed**: November 4, 2025  
**Issue**: Models were hardcoded instead of reading from environment variables

---

## 🎯 Environment Variables

### Primary Configuration

```powershell
# Set your preferred Ollama model
$env:OLLAMA_MODEL = "llama3.2"

# Alternative variable name (same effect)
$env:LLM_MODEL = "qwen2.5-coder:7b"

# Custom Ollama URL (optional)
$env:OLLAMA_URL = "http://localhost:11434"
```

### Supported Models

Any Ollama model can be used. Popular choices:

| Model | Best For | Size |
|-------|----------|------|
| `llama3.2` | General reasoning | 3GB |
| `qwen2.5-coder:7b` | Code generation | 4.7GB |
| `codellama:13b` | Code (default) | 7.4GB |
| `deepseek-coder:6.7b` | Code | 3.8GB |
| `mistral` | Fast responses | 4.1GB |
| `mixtral:8x7b` | Advanced reasoning | 26GB |

---

## 📝 How It Works

### Before Fix ❌

Models were **hardcoded**:
```rust
// ThinkingAgent - Always used llama3.2
llm: LLMBridge::with_ollama("llama3.2")

// CodingAgent - Always used codellama:13b
LLMConfig::default() // hardcoded to codellama:13b
```

### After Fix ✅

Models read from **environment variables**:
```rust
// Checks OLLAMA_MODEL first, then LLM_MODEL
let model = std::env::var("OLLAMA_MODEL")
    .or_else(|_| std::env::var("LLM_MODEL"))
    .unwrap_or_else(|_| "codellama:13b".to_string());
```

**Priority Order**:
1. `OLLAMA_MODEL` environment variable
2. `LLM_MODEL` environment variable  
3. Default: `codellama:13b` (for CodingAgent) or `llama3.2` (for ThinkingAgent)

---

## 🚀 Usage Examples

### PowerShell (Windows)

```powershell
# Set model for current session
$env:OLLAMA_MODEL = "qwen2.5-coder:7b"

# Start server
cargo run --release --bin api_server --features agents

# Make persistent (add to profile)
[System.Environment]::SetEnvironmentVariable('OLLAMA_MODEL', 'qwen2.5-coder:7b', 'User')
```

### Bash (Linux/Mac)

```bash
# Set model for current session
export OLLAMA_MODEL="llama3.2"

# Start server
cargo run --release --bin api_server --features agents

# Make persistent (add to ~/.bashrc or ~/.zshrc)
echo 'export OLLAMA_MODEL="llama3.2"' >> ~/.bashrc
```

### .env File

Create `.env` in project root:
```env
OLLAMA_MODEL=qwen2.5-coder:7b
OLLAMA_URL=http://localhost:11434
```

**Note**: Restart server after changing environment variables!

---

## 🔍 Verify Configuration

### Check Active Model

Server startup will show:
```
📡 Sending request to Ollama: model=qwen2.5-coder:7b, prompt_len=...
```

### Test Different Models

```powershell
# Test with llama3.2
$env:OLLAMA_MODEL = "llama3.2"
cargo run --release --bin api_server --features agents

# Test with qwen2.5-coder
$env:OLLAMA_MODEL = "qwen2.5-coder:7b"
cargo run --release --bin api_server --features agents

# Test with default (codellama:13b)
Remove-Item Env:\OLLAMA_MODEL
cargo run --release --bin api_server --features agents
```

---

## 🎛️ Component-Specific Models

### ThinkingAgent
**Default**: `llama3.2`  
**Best for**: General queries, reasoning, truth analysis

```powershell
$env:OLLAMA_MODEL = "llama3.2"  # Fast, good reasoning
$env:OLLAMA_MODEL = "mixtral:8x7b"  # Advanced reasoning
```

### CodingAgent
**Default**: `codellama:13b`  
**Best for**: Code generation, debugging

```powershell
$env:OLLAMA_MODEL = "codellama:13b"  # Specialized for code
$env:OLLAMA_MODEL = "qwen2.5-coder:7b"  # Newer, faster
$env:OLLAMA_MODEL = "deepseek-coder:6.7b"  # Good balance
```

---

## 📊 Performance Comparison

| Model | Speed | Quality | Memory | Best Use |
|-------|-------|---------|--------|----------|
| **llama3.2** | ⭐⭐⭐⭐ | ⭐⭐⭐ | 3GB | Chat, reasoning |
| **qwen2.5-coder:7b** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | 4.7GB | Code + chat |
| **codellama:13b** | ⭐⭐⭐ | ⭐⭐⭐⭐ | 7.4GB | Pure code |
| **mixtral:8x7b** | ⭐⭐ | ⭐⭐⭐⭐⭐ | 26GB | Complex tasks |
| **deepseek-coder:6.7b** | ⭐⭐⭐ | ⭐⭐⭐⭐ | 3.8GB | Code (efficient) |

---

## 🐛 Troubleshooting

### Model Not Found

```
Error: Model 'xyz' not found
```

**Solution**:
```powershell
# Pull the model first
ollama pull llama3.2

# Then start server
$env:OLLAMA_MODEL = "llama3.2"
cargo run --release --bin api_server --features agents
```

### Still Using Wrong Model

```
📡 Sending request to Ollama: model=codellama:13b
```

**Solution**:
```powershell
# Verify env var is set
echo $env:OLLAMA_MODEL

# Restart server (kill and start again)
# Env vars are read at startup, not runtime
```

### Ollama Not Running

```
Failed to connect to Ollama
```

**Solution**:
```powershell
# Start Ollama
ollama serve

# Or run in background (Windows)
Start-Process ollama -ArgumentList "serve" -WindowStyle Hidden
```

---

## 🔧 Files Modified

| File | Change | Lines |
|------|--------|-------|
| `src/agents/llm_bridge.rs` | Read env vars in default | +8 |
| `src/agents/thinking_agent.rs` | Read env vars in new() | +6 |
| `src/agents/coding_agent_enhanced.rs` | Update comments | +2 |

**Total**: 3 files, ~16 lines changed

---

## ✅ Quick Reference

```powershell
# Windows PowerShell
$env:OLLAMA_MODEL = "your-model-name"
cargo run --release --bin api_server --features agents

# Linux/Mac Bash
export OLLAMA_MODEL="your-model-name"
cargo run --release --bin api_server --features agents

# Check available models
ollama list

# Pull new model
ollama pull model-name

# Remove env var (use default)
Remove-Item Env:\OLLAMA_MODEL  # PowerShell
unset OLLAMA_MODEL              # Bash
```

---

## 📚 Related Documentation

- **QUICK_REFERENCE.md** - All common commands
- **GPU_ACCELERATION_GUIDE.md** - GPU setup
- **FIRST_PRINCIPLES_REASONING.md** - ThinkingAgent features
- **TASK_MANAGEMENT.md** - Task system

---

**Status**: ✅ **Production Ready**  
**Backward Compatible**: Yes (defaults unchanged)  
**Breaking Changes**: None

**Your models are now configurable via environment variables!** 🎉
