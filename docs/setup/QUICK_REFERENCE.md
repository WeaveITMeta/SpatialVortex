# 🚀 SpatialVortex Quick Reference

**Last Updated**: November 4, 2025

---

## 🤖 LLM Model Configuration

### Set Model via Environment Variable

```powershell
# Windows PowerShell
$env:OLLAMA_MODEL = "llama3.2"
$env:OLLAMA_MODEL = "qwen2.5-coder:7b"
$env:OLLAMA_MODEL = "codellama:13b"  # Default

# Linux/Mac Bash
export OLLAMA_MODEL="llama3.2"
```

**Note**: Restart server after changing!

### Defaults
- **ThinkingAgent**: `llama3.2` (if no env var)
- **CodingAgent**: `codellama:13b` (if no env var)

See `LLM_MODEL_CONFIGURATION.md` for full details.

---

## 🎮 GPU Acceleration Commands

### NVIDIA GPU (CUDA)
```powershell
# Start API server with CUDA
cargo run --release --bin api_server --features burn-cuda-backend

# Verify GPU is active
nvidia-smi
```

### Any GPU (WGPU - Cross-platform)
```powershell
# Works with NVIDIA, AMD, Intel, Apple Silicon
cargo run --release --bin api_server --features burn-wgpu-backend
```

### CPU Only (Fallback)
```powershell
# No GPU features
cargo run --release --bin api_server
```

### Check Active Backend
```powershell
# Look for this in server output:
# "✅ Selected backend: Burn (CUDA/NVIDIA)" 
# or
# "✅ Selected backend: Burn (WGPU/GPU)"
```

---

## 📝 Text Formatting

**Fixed in**: November 4, 2025

### What Was Fixed
- ✅ Text formatting now applied to **all ThinkingAgent responses**
- ✅ Applied to regular queries
- ✅ Applied to truth analysis (first principles)
- ✅ Applied to web search results
- ✅ **Markdown tables now preserved** (no more broken tables!)

### How It Works
```rust
// All responses now pass through:
crate::text_formatting::format_quick(&response)
```

**Benefits**:
- Better paragraph spacing
- Improved line breaks
- Enhanced readability
- Word wrapping at sensible limits
- ✅ **Markdown tables stay formatted**
- ✅ **Headers, code blocks, and rules preserved**

### Markdown Preservation
The formatter now **skips wrapping** for:
- Tables (lines starting with `|`)
- Headers (lines starting with `#`)
- Code blocks (lines starting with `` ` ``)
- Horizontal rules (`---`)

See `MARKDOWN_TABLE_FIX.md` for details.

---

## 💬 Chat Session Persistence

**Fixed in**: November 4, 2025

### What Was Fixed
- ✅ **Chat sessions now persist to disk** automatically
- ✅ **Full history preserved** across server restarts
- ✅ **Resume conversations** anytime with same session_id
- ✅ **5 new API endpoints** for session management

### How It Works
```bash
# Start conversation (generates session_id)
POST /api/v1/chat/unified
{"message": "Hello", "user_id": "alice"}
# Returns: session_id = "alice_1730745600"

# Continue conversation later
POST /api/v1/chat/unified
{"message": "Continue from yesterday", "session_id": "alice_1730745600"}
# ✅ Full context preserved!
```

### New Endpoints
- `GET /api/v1/chat/sessions?user_id=xxx` - List all sessions
- `GET /api/v1/chat/history/{session_id}` - Get full history
- `DELETE /api/v1/chat/sessions/{session_id}` - Delete session
- `GET /api/v1/chat/stats` - Get statistics
- `POST /api/v1/chat/continue` - Continue session helper

### Storage
- **Location**: `data/chat/{session_id}.json`
- **Size**: ~1-5 KB per session
- **Auto-save**: After every message
- **Auto-restore**: On server startup

See `CHAT_SESSION_PERSISTENCE.md` for full details.

---

## 🧠 First Principles Reasoning

### Trigger Keywords
```
"is this true"
"is this false"
"truth"
"lie"
"sarcasm"
"deception"
"misleading"
```

### Example
```bash
curl -X POST http://localhost:7000/api/v1/chat/unified \
  -d '{"message": "Is this true: The sky is blue and the sky is not blue"}'
```

---

## 🌐 Web Search

### Trigger Keywords
```
"search the web"
"search the internet"
"weather in [location]"
"current weather"
```

### Example
```bash
curl -X POST http://localhost:7000/api/v1/chat/unified \
  -d '{"message": "Weather in Seattle, WA"}'
```

---

## 📋 Task Management

### Create Tasks
```bash
POST /api/v1/tasks/create
{
  "session_id": "demo",
  "instructions": "1. Task A\n2. Task B\n3. Task C"
}
```

### Execute All
```bash
POST /api/v1/tasks/execute-all
{"session_id": "demo"}
```

---

## 🏃 Common Commands

### Start Server (Standard)
```powershell
cargo run --release --bin api_server
```

### Start Server (with agents)
```powershell
cargo run --release --bin api_server --features agents
```

### Start Server (GPU + Agents)
```powershell
# CUDA
cargo run --release --bin api_server --features "burn-cuda-backend,agents"

# WGPU
cargo run --release --bin api_server --features "burn-wgpu-backend,agents"
```

### Check Compilation
```powershell
cargo check --lib
```

### Run Tests
```powershell
# All tests
cargo test

# Specific module
cargo test first_principles
cargo test task_manager
```

### Run Examples
```powershell
# First principles demo
cargo run --example first_principles_demo

# Task management demo
cargo run --example task_demo
```

---

## 🔍 Verify Features

### Check Active Features
```powershell
# Server startup shows:
# - Selected ML backend (CUDA/WGPU/CPU)
# - Task Manager status
# - API endpoints available
```

### GPU Verification
```powershell
# NVIDIA
nvidia-smi

# Task Manager (Windows)
# Performance → GPU → 3D/Compute usage
```

---

## 📊 Performance Expectations

| Component | Latency | Notes |
|-----------|---------|-------|
| **API** | <200ms | P99 latency |
| **GPU Inference** | <5ms | With CUDA |
| **WGPU Inference** | <10ms | Cross-platform |
| **CPU Inference** | <50ms | Fallback |
| **Web Search** | <2s | External API |
| **First Principles** | <10ms | Local analysis |
| **Task Execution** | Varies | Based on task type |

---

## 🐛 Quick Troubleshooting

### GPU Not Detected
```powershell
# Update drivers
# Verify CUDA: nvcc --version
# Try WGPU instead of CUDA
```

### Formatting Not Applied
```powershell
# Fixed in latest version
# Restart server to load new code
cargo clean
cargo run --release --bin api_server
```

### Web Search Not Working
```powershell
# Check internet connection
# Verify wttr.in is accessible
curl https://wttr.in/London?format=j1
```

### Task Manager Not Restoring
```powershell
# Check directory exists
ls data/tasks/

# Verify JSON files valid
Get-Content data/tasks/session_id.json | ConvertFrom-Json
```

---

## 📚 Documentation Files

| File | Purpose |
|------|---------|
| **FIRST_PRINCIPLES_REASONING.md** | Truth detection guide |
| **WEB_SEARCH_FIX.md** | Web search implementation |
| **TASK_MANAGEMENT.md** | Multi-step task system |
| **TASK_API_QUICKSTART.md** | API testing guide |
| **TASK_PERSISTENCE_GUIDE.md** | Session recovery |
| **GPU_ACCELERATION_GUIDE.md** | Full GPU setup |
| **QUICK_REFERENCE.md** | This file |

---

## ✅ Feature Status

| Feature | Status | Command |
|---------|--------|---------|
| **GPU Acceleration** | ✅ Ready | `--features burn-cuda-backend` |
| **Text Formatting** | ✅ Fixed | Automatic |
| **First Principles** | ✅ Ready | Automatic detection |
| **Web Search** | ✅ Ready | Automatic detection |
| **Task Management** | ✅ Ready | API endpoints |
| **Task Persistence** | ✅ Ready | Automatic |

---

**Quick Help**:
- Server not starting? → Check `cargo check --lib`
- GPU not working? → See GPU_ACCELERATION_GUIDE.md
- API not responding? → Check port 7000 is free
- Formatting issues? → Restart server with latest code

**Support**: All documentation in project root and `docs/` folder
