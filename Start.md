# 🚀 SpatialVortex Development Servers

---

## 🏆 Ultimate Production Setup (v1.6.0 "Memory Palace")

**Complete system with GPU acceleration, consciousness streaming, and full persistence.**

### Prerequisites

```powershell
# 1. Ensure PostgreSQL is running (for RAG + Memory Palace)
createdb spatial_vortex
psql spatial_vortex -c "CREATE EXTENSION vector;"

# 2. Pull Ollama model (if not already downloaded)
ollama pull mixtral:8x7b

# 3. Set environment variables
$env:RUST_LOG="info"
$env:DATABASE_URL="postgresql://localhost/spatial_vortex"
$env:OLLAMA_MODEL="mixtral:8x7b"
```

# Single command for full production
cargo build --release --features agents,persistence,postgres,lake,burn-cuda-backend

### Terminal 1: Unified Production API (Port 7000) - ⭐ SINGLE SERVER

**Complete REST API with ALL features consolidated**

```powershell
cargo run --release --bin api_server --features agents,persistence,postgres,lake,burn-cuda-backend
```

**Features enabled:**
- ✅ 40+ REST endpoints (chat, RAG, inference, monitoring, **consciousness**)
- ✅ Native AI inference (Phase 2: 10× faster than LLM)
- ✅ Consciousness thinking & analytics (v1.6.0 "Memory Palace")
- ✅ Background learning (continuous improvement)
- ✅ Memory Palace (state persistence across restarts)
- ✅ PostgreSQL RAG (persistent embeddings)
- ✅ Confidence Lake (high-value pattern storage)
- ✅ NVIDIA CUDA GPU acceleration
- ✅ Swagger UI at http://localhost:7000/swagger-ui/
- ✅ ASI Orchestrator
- ✅ Sacred geometry inference

**Key API Endpoints:**

**Chat & AI:**
- `POST /api/v1/chat/text` - Text chat (native AI first, LLM fallback)
- `POST /api/v1/chat/code` - Code generation
- `POST /api/v1/consciousness/think` - Consciousness processing
- `GET /api/v1/consciousness/analytics` - Real-time Φ & metrics
- `POST /api/v1/consciousness/save-state` - Save consciousness state
- `GET /api/v1/consciousness/health` - Consciousness health

**No separate consciousness server needed! Everything runs on port 7000.**

---

### Terminal 2: Consciousness Streaming (Port 4433)

**Real-time WebTransport streaming with QUIC protocol**

```powershell
# Generate TLS certificates (one-time setup)
powershell scripts/generate_tls_certs.ps1

# Start streaming server
cargo run --release --bin consciousness_streaming_server --features transport,agents,burn-cuda-backend
```

**Features enabled:**
- ✅ Real-time consciousness streaming
- ✅ Word-level insights (<50ms latency)
- ✅ WebTransport (QUIC/HTTP3)
- ✅ NVIDIA CUDA GPU acceleration

**Connection:** `https://localhost:4433`

---

### Terminal 3: Frontend Dev Server (Port 28082)

```powershell
cd web
bun run dev
```

**Access:** http://localhost:28082

---

## 🎯 Production Deployment

**Single command for full production stack:**

```powershell
# Main API with all features + GPU
cargo build --release --features agents,persistence,postgres,lake,burn-cuda-backend

# Run production binary
./target/release/spatial-vortex
```

**Docker Compose (recommended for production):**

```yaml
version: '3.8'
services:
  postgres:
    image: ankane/pgvector
    environment:
      POSTGRES_DB: spatial_vortex
      POSTGRES_PASSWORD: ${DB_PASSWORD}
    ports:
      - "5432:5432"
    volumes:
      - pgdata:/var/lib/postgresql/data

  api:
    build: .
    command: cargo run --release --bin api_server --features burn-cuda-backend
    ports:
      - "7000:7000"
    environment:
      DATABASE_URL: postgresql://postgres:${DB_PASSWORD}@postgres/spatial_vortex
      RUST_LOG: info
    depends_on:
      - postgres

  # NOTE: Consciousness API is now integrated into the main api_server
  # No separate consciousness container needed!
    volumes:
      - ./consciousness_state.json:/app/consciousness_state.json

  streaming:
    build: .
    command: cargo run --release --bin consciousness_streaming_server --features transport,agents,burn-cuda-backend
    ports:
      - "4433:4433"
    environment:
      RUST_LOG: info
    volumes:
      - ./certs:/app/certs

volumes:
  pgdata:
```

---

## 📊 Feature Matrix

| Feature | Flag | Purpose |
|---------|------|---------|
| **agents** | `--features agents` | Background learning & consciousness |
| **persistence** | `--features persistence` | Memory Palace state saving |
| **postgres** | `--features postgres` | PostgreSQL RAG backend |
| **lake** | `--features lake` | Confidence Lake (encrypted storage) |
| **burn-cuda-backend** | `--features burn-cuda-backend` | NVIDIA GPU acceleration |
| **transport** | `--features transport` | WebTransport streaming |
| **rag** | `--features rag` | RAG system (included by default) |

**Combine features with commas:**
```powershell
--features agents,persistence,postgres,lake,burn-cuda-backend
```

---

## 🔥 GPU Acceleration Modes

### NVIDIA CUDA (Best Performance)
```powershell
--features burn-cuda-backend
```
**Requirements:** CUDA 11.8+, cuDNN 8.9+

### Cross-Platform GPU (AMD/Intel/NVIDIA)
```powershell
--features burn-wgpu-backend
```
**Requirements:** Vulkan or DirectX 12

### CPU Only (No GPU)
```powershell
# No feature flag needed (default)
cargo run --release --bin api_server
```

---

## 🧪 Quick Tests

### Test Main API
```powershell
curl http://localhost:7000/api/v1/health
```

### Test Consciousness API
```powershell
curl -X POST http://localhost:7000/api/v1/consciousness/think `
  -H "Content-Type: application/json" `
  -d '{"question": "What is consciousness?"}'
```

### Test Streaming
```powershell
# Requires WebTransport client
# See: examples/consciousness_streaming_demo.rs
```

---

## 📈 Expected Performance

| Metric | CPU | GPU (CUDA) | Improvement |
|--------|-----|------------|-------------|
| **Inference** | ~200ms | ~20ms | 10× faster |
| **Tensor Ops** | ~50ms | ~5ms | 10× faster |
| **Embedding** | ~100ms | ~15ms | 6.7× faster |
| **Sacred Geometry** | ~80ms | ~10ms | 8× faster |

---

## Quick Start

### 1. Backend API Server (Port 7000)

**Terminal 1**:
```powershell
# Set environment variables
$env:API_PORT=7000
$env:API_HOST="127.0.0.1"
$env:API_CORS="true"
$env:RUST_LOG="info"

# Run server
cargo run --bin api_server

# Run with NVIDIA GPU
cargo run --release --bin api_server --features burn-cuda-backend
```

Expected output:
```
╔══════════════════════════════════════════════════════════╗
║         SpatialVortex Production API Server              ║
║                                                          ║
║  Sacred Geometry · ONNX Inference · Confidence Lake      ║
║  Voice Pipeline · Flux Matrix · ASI Integration          ║
╚══════════════════════════════════════════════════════════╝

🚀 Starting SpatialVortex API Server...
   Host: 127.0.0.1
   Port: 7000
   Workers: 4
📝 Loading configuration...
📦 Initializing components...
🧠 Initializing ASI Orchestrator...
✅ Components initialized
🌐 Starting HTTP server at http://127.0.0.1:7000

📋 Available endpoints:
   
   ✅ Chat & Conversation (9 endpoints)
   POST   /api/v1/chat/text           ← Frontend connects here
   GET    /api/v1/chat/conversations/{user_id}
   GET    /api/v1/chat/history/{conversation_id}
   POST   /api/v1/chat/conversations
   DELETE /api/v1/chat/conversations/{conversation_id}
   GET    /api/v1/chat/stats/{user_id}
   POST   /api/v1/chat/search
   GET    /api/v1/chat/export/{conversation_id}
   POST   /api/v1/chat/suggestions
   
   ✅ RAG (Retrieval-Augmented Generation) (7 endpoints)
   POST   /api/v1/rag/ingest
   POST   /api/v1/rag/search
   POST   /api/v1/rag/web-search              ⭐ NEW: Multi-source (DuckDuckGo default, FREE!)
   GET    /api/v1/rag/documents
   DELETE /api/v1/rag/documents/{doc_id}
   GET    /api/v1/rag/embeddings/stats
   POST   /api/v1/rag/retrieve/sacred
   
   ✅ Monitoring & Observability (9 endpoints)
   GET    /api/v1/health
   GET    /api/v1/metrics
   GET    /api/v1/metrics/sacred
   GET    /api/v1/metrics/elp
   GET    /api/v1/metrics/inference
   GET    /api/v1/metrics/confidence-lake
   GET    /api/v1/metrics/usage
   GET    /api/v1/metrics/connections
   GET    /api/v1/metrics/errors
   GET    /api/v1/logs
   
   ✅ ML & Inference (6 endpoints)
   POST   /api/v1/flux/matrix/generate
   POST   /api/v1/inference/reverse
   POST   /api/v1/inference/forward
   POST   /api/v1/ml/embed
   POST   /api/v1/ml/asi/infer
   GET    /api/v1/ml/asi/metrics
   
   ✅ Other (2 endpoints)
   GET    /api/v1/subjects
   POST   /api/v1/subjects/generate
   
   📊 Total: 35+ endpoints

📖 Swagger UI:
   http://127.0.0.1:7000/swagger-ui/
```

### Dev Mode (offline, no Redis/cloud)

Use Development Mode to run fully locally without external services.

```powershell
$env:DEVELOPMENT_MODE = "true"   # Forces ASI Fast mode; chat uses local fallback
$env:API_PORT = 7000              # Bind to documented dev port

# Optional: ensure cache uses in-memory no-op
# Not required—DEVELOPMENT_MODE automatically enables it

cargo run --bin api_server --release
```

Validation:

- `GET http://localhost:7000/api/v1/metrics` → contains `dev_mode: true`
- Chat: `POST /api/v1/chat/text` returns local response (no network)
- ASI: `POST /api/v1/ml/asi/infer` uses Fast mode (no consensus)

### 2. Frontend Dev Server (Port 5173)

**Terminal 2**:
```powershell
cd web
npm run dev
```

Expected output:
```
VITE v5.x.x  ready in xxx ms

  ➜  Local:   http://localhost:5173/
  ➜  Network: use --host to expose
```

### 3. WebTransport Server (Port 4433) - Optional

For QUIC-based real-time streaming:

**Terminal 3**:
```powershell
# Generate TLS certificates first (one-time)
powershell scripts/generate_tls_certs.ps1

# Run WebTransport server
cargo run --features transport --bin webtransport_server
```

---

## Port Reference

| Service | Port | Protocol | Used By |
|---------|------|----------|---------|
| REST API | 7000 | HTTP | Svelte frontend |
| Frontend | 5173 | HTTP | Browser |
| WebTransport | 4433 | QUIC/HTTP3 | Advanced streaming (optional) |
| Swagger UI | 7000 | HTTP | API documentation |

---

## 🔍 Test Multi-Source Web Search

**DuckDuckGo is enabled by default (FREE, no API key needed!)**

### Quick Test

```bash
# Run the demo
cargo run --example web_search_demo --features agents
```

### Test via API

```bash
curl -X POST http://localhost:7000/api/v1/rag/web-search \
  -H "Content-Type: application/json" \
  -d '{
    "query": "What is vortex mathematics?",
    "max_sources": 10
  }'
```

### Add More Engines (Optional)

Add to `.env` for better results:
```bash
# Brave Search (2,000 free queries/month)
BRAVE_API_KEY=BSAyour-key-here
```

**Auto-detection**: System automatically uses any engines with configured API keys!

---

## Troubleshooting

### ❌ ERR_CONNECTION_REFUSED

**Error**: `POST http://localhost:7000/api/v1/chat/text net::ERR_CONNECTION_REFUSED`

**Solution**: Backend API server is not running. Start it in Terminal 1.

### ❌ CORS Error

**Error**: `Access to fetch at 'http://localhost:7000' has been blocked by CORS policy`

**Solution**: Ensure `API_CORS=true` is set when starting the API server.

### ❌ Port Already in Use

**Error**: `Address already in use (os error 10048)`

**Solution**: 
```powershell
# Find process using port 7000
netstat -ano | findstr :7000

# Kill the process (replace PID with actual process ID)
taskkill /PID <PID> /F
```

---

## Environment Variables

### API Server

```powershell
$env:API_HOST="127.0.0.1"           # Bind address
$env:API_PORT=7000                   # Port number
$env:API_WORKERS=4                   # Worker threads
$env:API_CORS="true"                 # Enable CORS
$env:RUST_LOG="info"                 # Log level
```

### LLM Backend Configuration

**Choose between Native Vortex inference or external LLM:**

```powershell
# 🌀 NATIVE VORTEX (RECOMMENDED - Default)
# Uses SpatialVortex's built-in flux matrix inference
# No external dependencies, pure Rust, sacred geometry-based
$env:LLM_BACKEND="native"

# 📡 OLLAMA (External LLM fallback)
# Uses Ollama for LLM-based generation
$env:LLM_BACKEND="ollama"
$env:OLLAMA_MODEL="llama3.2:latest"        # Default - Fast, good quality
# $env:OLLAMA_MODEL="mixtral:8x7b"         # Recommended for balanced performance
# $env:OLLAMA_MODEL="codellama:13b"        # Code-optimized
# $env:OLLAMA_MODEL="qwen2.5-coder:7b"     # Latest coder model
# $env:OLLAMA_MODEL="deepseek-coder:6.7b" # Specialized coding

# Ollama server URL (if not default)
$env:OLLAMA_URL="http://localhost:11434"
```

**Native Vortex Benefits:**
- ⚡ **Faster**: No network overhead
- 🔒 **Private**: All processing local
- 🌀 **Sacred Geometry**: Uses vortex mathematics (1→2→4→8→7→5→1)
- 🎯 **3-6-9 Pattern**: Built-in hallucination detection
- 💎 **ELP Analysis**: Ethos-Logos-Pathos reasoning

**If using Ollama, first time setup:**
```powershell
# Pull the model
ollama pull llama3.2:latest

# Verify model is loaded
ollama list
```

### ONNX Models (Optional)

```powershell
$env:SPATIALVORTEX_ONNX_MODEL_PATH="./models/model.onnx"
$env:SPATIALVORTEX_ONNX_TOKENIZER_PATH="./models/tokenizer.json"
```

### WebTransport (Optional)

```powershell
$env:WEBTRANSPORT_BIND="0.0.0.0:4433"
$env:WEBTRANSPORT_CERT="certs/cert.pem"
$env:WEBTRANSPORT_KEY="certs/key.pem"
$env:MAX_CONNECTIONS=2000
$env:MAX_STREAMS=100
```

---

## Development Workflow

1. **Start Backend**: Terminal 1 → API server on port 7000
2. **Start Frontend**: Terminal 2 → Vite dev server on port 5173
3. **Open Browser**: http://localhost:5173
4. **Test Chat**: Send a message in the chat interface
5. **Check Logs**: Monitor both terminals for request/response logs

---

## Production Deployment

See `docs/implementation/FRONTEND_CHAT_INTERFACE_SPEC.md` for:
- Docker deployment
- Kubernetes configuration
- TLS/SSL setup
- Load balancing
- Monitoring

---

## API Testing

### Health Check
```powershell
curl http://localhost:7000/api/v1/health
```

### Chat Test
```powershell
curl -X POST http://localhost:7000/api/v1/chat/text `
  -H "Content-Type: application/json" `
  -d '{"message": "What is consciousness?", "user_id": "test_user"}'
```

### Swagger UI
Open browser: http://localhost:7000/swagger-ui/

---

**Status**: ✅ Ready for development!
