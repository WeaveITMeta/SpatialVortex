# SpatialVortex Cargo Run Commands

**Complete command reference for running SpatialVortex as an orchestrated ASI system**

Version: 0.7.0  
Date: October 29, 2025

---

## 📋 Table of Contents

1. [Quick Start](#quick-start)
2. [Production API Server](#production-api-server)
3. [Binary Targets](#binary-targets)
4. [Training Examples](#training-examples)
5. [Benchmarks](#benchmarks)
6. [Orchestrated ASI Workflow](#orchestrated-asi-workflow)
7. [Feature Flags](#feature-flags)
8. [Environment Variables](#environment-variables)

---

## 🚀 Quick Start

### Start Everything (Recommended)

```bash
# Terminal 1: API Server (PostgreSQL + Redis)
cargo run --release

# Terminal 2: Frontend (Svelte on port 28082)
cd web && npm run dev

# Terminal 3: Train on Knowledge (Optional)
cargo run --example train_on_grokipedia --release --features "rag,lake"
```

**Access Points:**
- API: http://localhost:7000
- Frontend: http://localhost:28082
- Swagger: http://localhost:7000/swagger-ui/

---

## 🖥️ Production API Server

### Main Server (spatial-vortex binary)

The main binary `spatial-vortex` (`src/main.rs`) runs the full REST API server with:
- ✅ Sacred Geometry flux matrix engine
- ✅ ONNX/Tract inference
- ✅ PostgreSQL persistence
- ✅ Redis caching
- ✅ AI chat endpoints
- ✅ RAG retrieval
- ✅ ASI orchestration (Phases 1-4)

```bash
# Basic start (uses default binary: spatial-vortex)
cargo run --release

# Or explicitly specify the binary
cargo run --bin spatial-vortex --release

# With options
cargo run --bin spatial-vortex --release -- --help

# Initialize database schema
cargo run --release -- --init-db

# Bootstrap with example matrices
cargo run --release -- --bootstrap

# Custom host/port
cargo run --release -- --host 0.0.0.0 --port 8080

# With specific database
cargo run --release -- --database-url "postgresql://spatialvortex:spatialvortex@localhost:5432/spatialvortex"
```

**Environment Variables:**
```bash
DATABASE_URL=postgresql://spatialvortex:spatialvortex@localhost:5432/spatialvortex
REDIS_URL=redis://127.0.0.1:6379
AI_API_KEY=your_api_key
AI_MODEL_ENDPOINT=https://api.x.ai/v1
RUST_LOG=info
```

**Default Configuration:**
- Host: `127.0.0.1`
- Port: `7000`
- Database: PostgreSQL (from `.env`)
- Cache TTL: 24 hours
- AI: Optional (graceful fallback)

---

## 🎯 Binary Targets

### 1. API Server (Alternative)

Standalone API server with explicit configuration:

```bash
cargo run --bin api_server --release
```

**Features:**
- REST API on port 7000
- Swagger documentation
- Full ASI orchestration
- Same as main binary but explicit

---

### 2. WebTransport Server

High-performance QUIC/HTTP3 server for real-time chat:

```bash
# Requires TLS certificates
cargo run --bin webtransport_server --release --features transport
```

**Configuration:**
```bash
WEBTRANSPORT_BIND=0.0.0.0:4433
WEBTRANSPORT_CERT=certs/cert.pem
WEBTRANSPORT_KEY=certs/key.pem
MAX_CONNECTIONS=2000
MAX_STREAMS=100
```

**Features:**
- HTTP/3 over QUIC
- Bidirectional streams
- 2000+ concurrent connections
- Low latency chat

**Generate TLS Certificates:**
```bash
mkdir -p certs
openssl req -x509 -newkey rsa:4096 -keyout certs/key.pem -out certs/cert.pem -days 365 -nodes
```

---

### 3. Subject CLI

Interactive CLI for subject domain generation:

```bash
cargo run --bin subject_cli

# Generate specific subjects
cargo run --bin subject_cli -- "Sacred Geometry"
cargo run --bin subject_cli -- "Quantum Physics"
```

**Use Cases:**
- Quick subject matrix generation
- Testing flux matrix engine
- Semantic space exploration

---

### 4. Coding Agent CLI

Multi-language coding assistant with LLM integration:

```bash
cargo run --bin coding_agent_cli --features agents

# With specific task
cargo run --bin coding_agent_cli --features agents -- "Write a Rust function to calculate Fibonacci"
```

**Requirements:**
- `AI_API_KEY` environment variable
- `agents` feature flag
- Internet connection

**Capabilities:**
- Rust, Python, JavaScript, TypeScript, Go
- Code generation
- Code review
- Bug fixing
- Optimization suggestions

---

### 5. Flux Matrix Visualizer

3D Bevy visualization of flux patterns:

```bash
cargo run --bin flux_matrix --release --features bevy_support
```

**Controls:**
- Mouse: Rotate camera
- Scroll: Zoom
- Spacebar: Spawn new beams
- ESC: Exit

**Features:**
- Real-time 3D rendering
- Sacred geometry (3-6-9 triangle)
- ELP color channels
- Vortex flow animation

---

### 6. Flux Matrix Vortex

Vortex flow visualization with sacred positions:

```bash
cargo run --bin flux_matrix_vortex --release --features bevy_support
```

**Shows:**
- Vortex cycling: 1→2→4→8→7→5→1
- Sacred checkpoints: 3, 6, 9
- Beam trajectories
- Signal strength

---

### 7. Vortex View

Advanced 3D viewer with multiple visualization modes:

```bash
cargo run --bin vortex_view --release --features bevy_support
```

**Modes:**
- Sacred triangle
- Vortex flow
- Signal subspace
- Beam paths

---

### 8. Camera System

Bevy camera controls demo:

```bash
cargo run --bin camera --release --features bevy_support
```

**Camera Modes:**
- Orbital
- Free-look
- Follow
- First-person

---

### 9. WASM 3D Viewer

Web-based 3D visualization (WASM):

```bash
# Build WASM
wasm-pack build --target web --features bevy_support

# Or run via trunk
trunk serve
```

**Access:** http://localhost:8080

---

### 10. Geometric Reasoning Benchmark

Performance benchmarks for core geometry:

```bash
cargo run --bin geometric_reasoning_benchmark --release
```

**Measures:**
- Flux matrix generation
- Sacred geometry calculations
- Vortex cycling performance
- Digital root operations

---

## 📚 Training Examples

### Grokipedia Training

Train on sacred geometry, AI, quantum computing articles:

```bash
cargo run --example train_on_grokipedia --release --features "rag,lake"
```

**Training Phases:**
1. Sacred Geometry (3 articles)
2. AI & Machine Learning (5 articles)
3. Advanced Topics (5 articles)
4. Comprehensive (14 articles)

**Output:**
- 27,000 embeddings
- 384-dimensional vectors
- Sacred positions (3-6-9)
- PostgreSQL storage

---

### Google Scholar Training

Train on academic ethics papers:

```bash
cargo run --example train_on_google_scholar --release --features "rag,lake"
```

**Training Phases:**
1. Virtue Ethics (16 articles)
2. Ethical Frameworks (18 articles)
3. Advanced Ethics (20 articles)
4. Interdisciplinary (16 articles)
5. Comprehensive Ethos (60 articles)

**Output:**
- 130,000 embeddings
- Credibility ≥ 0.6
- Ethos-boosted content
- PostgreSQL storage

**Comparison:**
| Dataset | Articles | Embeddings | Focus |
|---------|----------|------------|-------|
| Grokipedia | 27 | 27,000 | Technical knowledge |
| Google Scholar | 130 | 130,000 | Ethical reasoning |

---

### RAG Continuous Learning

Continuous document ingestion and learning:

```bash
cargo run --example rag_continuous_learning --release --features "rag,lake"
```

**Features:**
- Real-time document monitoring
- Automatic ingestion
- Vector embeddings
- Sacred geometry awareness

---

### ASI Orchestrator Demo

Demonstrate full ASI orchestration:

```bash
cargo run --example asi_orchestrator_demo --release
```

**Shows:**
- Phase 1: Fast inference
- Phase 2: Thorough inference
- Phase 3: Consensus
- Phase 4: Confidence validation

---

### Dynamic ELP Reinforcement Learning

RL training for ELP channel optimization:

```bash
cargo run --example dynamic_elp_rl_demo --release
```

**Trains:**
- ELP weighting
- Sacred position detection
- Signal strength optimization

---

## 🔬 Benchmarks

### Core Benchmarks

```bash
cargo run --example core_benchmarks --release
```

**Measures:**
- Flux matrix creation: ~500K-700K/sec
- ELP tensor: ~2M-3M/sec
- BeamTensor: >100x target

---

### Actual Benchmarks

Real-world performance tests:

```bash
cargo run --example actual_benchmarks --release
```

**Tests:**
- Database operations
- Cache performance
- API latency
- Inference speed

---

### Quick Benchmarks

Fast smoke tests:

```bash
cargo run --example quick_benchmarks --release
```

**Duration:** ~30 seconds  
**Coverage:** Core operations only

---

### Optimization Benchmark

Lock-free structure performance:

```bash
cargo run --example optimization_benchmark --release
```

**Compares:**
- Lock-free queue
- Standard mutex
- RwLock
- DashMap

---

### Dynamic Config Benchmark

Configuration hot-reload testing:

```bash
cargo run --example dynamic_config_benchmark --release
```

---

## 🌀 Orchestrated ASI Workflow

### Complete Training → Inference Pipeline

**Step 1: Database Setup**
```bash
# Start PostgreSQL (Docker)
docker start spatialvortex-db

# Initialize schema
cargo run --release -- --init-db
```

**Step 2: Train Knowledge Base**
```bash
# Terminal 1: Grokipedia (Technical)
cargo run --example train_on_grokipedia --release --features "rag,lake"

# Terminal 2: Google Scholar (Ethics)
cargo run --example train_on_google_scholar --release --features "rag,lake"
```

**Step 3: Start Services**
```bash
# Terminal 3: API Server
cargo run --release

# Terminal 4: Frontend
cd web && npm run dev
```

**Step 4: Test ASI**
```bash
# Terminal 5: Orchestrator Demo
cargo run --example asi_orchestrator_demo --release
```

**Step 5: Visualize**
```bash
# Terminal 6: 3D Viewer
cargo run --bin flux_matrix --release --features bevy_support
```

---

### Multi-Model Consensus Flow

```mermaid
graph TD
    A[User Query] --> B[API Server :7000]
    B --> C[ASI Orchestrator]
    C --> D[Phase 1: Fast]
    C --> E[Phase 2: Thorough]
    C --> F[Phase 3: Consensus]
    D --> G[Grokipedia Knowledge]
    E --> H[Scholar Knowledge]
    F --> I[Weighted Voting]
    I --> J[Phase 4: Confidence]
    J --> K[Response + Metrics]
    K --> L[Frontend :28082]
```

---

## 🎛️ Feature Flags

### Default Features
```bash
# Pure Rust ONNX (recommended)
cargo run --release
```

Includes: `tract` (pure Rust ONNX)

---

### Optional Features

```bash
# Voice pipeline
cargo run --features voice --release

# Confidence Lake (SQLite)
cargo run --features lake --release

# RAG (document ingestion)
cargo run --features rag --release

# Coding agents (LLM)
cargo run --features agents --release

# Bevy 3D visualization
cargo run --features bevy_support --release

# ONNX C++ runtime
cargo run --features onnx --release

# Burn ML framework
cargo run --features burn-backend --release

# Candle ML framework
cargo run --features candle --release

# Formal verification (Z3)
cargo run --features formal-verification --release

# WebTransport server
cargo run --features transport --release
```

---

### Combined Features

```bash
# Full stack (recommended)
cargo run --release --features "rag,lake,agents"

# Training pipeline
cargo run --example train_on_grokipedia --release --features "rag,lake"

# 3D visualization
cargo run --bin flux_matrix --release --features bevy_support

# WebTransport chat
cargo run --bin webtransport_server --release --features transport

# ML backends
cargo run --release --features "tract,burn-backend"
```

---

## 🔐 Environment Variables

### Required

```bash
# PostgreSQL connection
DATABASE_URL=postgresql://spatialvortex:spatialvortex@localhost:5432/spatialvortex
```

---

### Optional

```bash
# Redis cache
REDIS_URL=redis://127.0.0.1:6379

# AI integration
AI_API_KEY=your_grok_or_openai_key
AI_MODEL_ENDPOINT=https://api.x.ai/v1

# API server
API_HOST=127.0.0.1
API_PORT=7000

# WebTransport
WEBTRANSPORT_BIND=0.0.0.0:4433
WEBTRANSPORT_CERT=certs/cert.pem
WEBTRANSPORT_KEY=certs/key.pem
MAX_CONNECTIONS=2000
MAX_STREAMS=100

# Logging
RUST_LOG=info
RUST_BACKTRACE=1

# Frontend (in web/.env)
VITE_API_URL=http://localhost:7000
```

---

## 📊 System Requirements

### Minimum
- **CPU:** 4 cores
- **RAM:** 8 GB
- **Storage:** 10 GB
- **OS:** Windows 10/11, Linux, macOS

### Recommended
- **CPU:** 8+ cores
- **RAM:** 16 GB
- **Storage:** 50 GB SSD
- **GPU:** Optional (for Candle CUDA)

---

## 🐳 Docker Setup

```bash
# PostgreSQL
docker run -d --name spatialvortex-db \
  -e POSTGRES_USER=spatialvortex \
  -e POSTGRES_PASSWORD=spatialvortex \
  -e POSTGRES_DB=spatialvortex \
  -p 5432:5432 \
  postgres:17

# Redis
docker run -d --name spatialvortex-redis \
  -p 6379:6379 \
  redis:latest
```

---

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run specific tests
cargo test --lib
cargo test --example asi_orchestrator_demo

# Run with features
cargo test --features "rag,lake"

# Benchmarks
cargo bench
```

---

## 📖 Documentation

```bash
# Generate docs
cargo doc --no-deps --open

# With private items
cargo doc --document-private-items --open

# Specific crate
cargo doc --package spatial-vortex --open
```

---

## 🔧 Troubleshooting

### Issue: Database connection failed
```bash
# Check PostgreSQL is running
docker ps | grep spatialvortex-db

# Verify connection
psql postgresql://spatialvortex:spatialvortex@localhost:5432/spatialvortex

# Reinitialize schema
cargo run --release -- --init-db
```

---

### Issue: Redis unavailable
```bash
# Start Redis
docker start spatialvortex-redis

# Or run without Redis (will fail gracefully)
# Server will exit with error message
```

---

### Issue: Port already in use
```bash
# Find process on port 7000
netstat -ano | findstr :7000

# Kill process (Windows)
taskkill /PID <pid> /F

# Or use different port
cargo run --release -- --port 8080
```

---

### Issue: WASM build fails
```bash
# Install wasm-pack
cargo install wasm-pack

# Or use trunk
cargo install trunk

# Build with correct target
wasm-pack build --target web
```

---

## 🚀 Production Deployment

### Build Release Binary

```bash
# Optimized release
cargo build --release --features "rag,lake"

# Strip symbols (smaller binary)
strip target/release/spatial-vortex
```

**Binary location:** `target/release/spatial-vortex`

---

### Run in Production

```bash
# With systemd
[Service]
ExecStart=/path/to/spatial-vortex --host 0.0.0.0 --port 7000
Environment="DATABASE_URL=postgresql://..."
Environment="REDIS_URL=redis://..."
Restart=always

# Or with PM2
pm2 start target/release/spatial-vortex --name spatialvortex
```

---

## 📝 Notes

- **Default port:** Frontend (28082), API (7000), WebTransport (4433)
- **PostgreSQL required:** All data persists to PostgreSQL
- **Redis optional:** Graceful fallback if unavailable
- **AI optional:** Works without AI integration
- **Features:** Use minimal features for faster builds
- **WASM:** Requires `wasm-pack` or `trunk`
- **TLS:** WebTransport requires certificates

---

## 🎯 Quick Reference - All Available Binaries

### Main Binaries

| Binary | Command | Purpose | Port | Features |
|--------|---------|---------|------|----------|
| `spatial-vortex` | `cargo run --release` | Full API Server (default) | 7000 | - |
| `api_server` | `cargo run --bin api_server --release` | Alternative API Server | 7000 | - |
| `webtransport_server` | `cargo run --bin webtransport_server --release --features transport` | WebTransport Server | 4433 | `transport` |
| `subject_cli` | `cargo run --bin subject_cli` | Subject Generation CLI | - | - |
| `coding_agent_cli` | `cargo run --bin coding_agent_cli --features agents` | Coding Assistant CLI | - | `agents` |
| `flux_matrix` | `cargo run --bin flux_matrix --release --features bevy_support` | 3D Flux Visualizer | - | `bevy_support` |
| `flux_matrix_vortex` | `cargo run --bin flux_matrix_vortex --release --features bevy_support` | Vortex Flow Viewer | - | `bevy_support` |
| `vortex_view` | `cargo run --bin vortex_view --release --features bevy_support` | Advanced 3D Viewer | - | `bevy_support` |
| `camera` | `cargo run --bin camera --release --features bevy_support` | Camera System Demo | - | `bevy_support` |
| `epic_flux_3d_wasm` | `wasm-pack build --features bevy_support` | WASM 3D Viewer | 8080 | `bevy_support` |
| `geometric_reasoning_benchmark` | `cargo run --bin geometric_reasoning_benchmark --release` | Geometry Benchmarks | - | - |

### Common Examples

| Command | Purpose |
|---------|---------|
| `cargo run --example train_on_grokipedia --release --features "rag,lake"` | Train on Grokipedia |
| `cargo run --example train_on_google_scholar --release --features "rag,lake"` | Train on Google Scholar |
| `cargo run --example asi_orchestrator_demo` | ASI Demo |
| `cd web && npm run dev` | Frontend (Svelte) |

---

## 📚 See Also

- [API Documentation](./api/README.md)
- [Architecture](./architecture/README.md)
- [Training Guide](./guides/TRAINING.md)
- [Frontend Setup](../web/README.md)
- [Docker Setup](./guides/DOCKER.md)

---

**Version:** 0.7.0  
**Last Updated:** October 29, 2025  
**License:** MIT
