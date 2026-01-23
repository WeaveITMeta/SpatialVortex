# 🚀 Cargo Run Commands - SpatialVortex v1.6.0

Complete guide to running all examples, demos, binaries, and tests.

---

## 📋 Table of Contents

- [Quick Start](#quick-start)
- [Examples](#examples)
- [Binary Targets](#binary-targets)
- [Tests](#tests)
- [Feature Flags](#feature-flags)
- [Development](#development)
- [Production](#production)

---

## ⚡ Quick Start

### **Run the Latest Demo (v1.6.0 Memory Palace)**
```bash
cargo run --example memory_palace_demo --features agents,persistence
```

### **Run with Full Features**
```bash
cargo run --example memory_palace_demo --features agents,persistence,postgres,lake --release
```

---

## 🎓 Examples

### **v1.6.0 "Memory Palace" - Persistent Consciousness**

**Basic (File-based state)**
```bash
cargo run --example memory_palace_demo --features agents,persistence
```

**Full System (PostgreSQL + State + Lake)**
```bash
cargo run --example memory_palace_demo --features agents,persistence,postgres,lake
```

**What it shows:**
- State save/load across restarts
- PostgreSQL RAG integration
- Auto-save functionality
- Continuous learning

---

### **v1.5.1 "Background Learning" - Full System**

**Complete Background Learning Demo**
```bash
cargo run --example background_learning_full_demo --features agents
```

**With RAG + Lake**
```bash
cargo run --example background_learning_full_demo --features agents,rag,lake
```

**What it shows:**
- Background learning cycles
- RAG knowledge ingestion
- Confidence Lake pattern review
- Learning statistics

---

### **v1.5.0 "Conscious Streaming" - Real-Time Analytics**

**Streaming Demo**
```bash
cargo run --example consciousness_streaming_demo --features agents
```

**What it shows:**
- Real-time consciousness streaming
- Word-level insights
- Selection-based analysis
- Streaming events

---

### **RAG System Examples**

**Continuous Learning**
```bash
cargo run --example rag_continuous_learning --features rag
```

**Train on Grokipedia**
```bash
cargo run --example train_on_grokipedia --features rag
```

**What they show:**
- Document ingestion pipeline
- Vector embeddings
- Knowledge accumulation
- Sacred geometry scoring

---

### **Voice Pipeline Examples**

**Voice 3D Visualization**
```bash
cargo run --example voice_3d --features voice,bevy_support --release
```

**What it shows:**
- Real-time voice visualization
- FFT spectrum analysis
- 3D sacred geometry
- Flux node rendering

---

### **3D Visualization Examples**

**Epic Flux 3D (Native)**
```bash
cargo run --example epic_flux_3d_native --features bevy_support --release
```

**Flux Matrix Demo**
```bash
cargo run --bin flux_matrix --features bevy_support
```

**Vortex View**
```bash
cargo run --bin vortex_view --features bevy_support
```

**What they show:**
- 3D vortex visualization
- Sacred triangle (3-6-9)
- Flux flow patterns
- Auto-rotating camera

---

### **Hallucination Detection Examples**

**Hallucination Demo**
```bash
cargo run --example hallucination_demo
```

**What it shows:**
- Signal subspace analysis
- Hallucination detection
- Sacred position interventions
- Vortex vs linear comparison

---

### **Subject Generation Examples**

**Subject Generation CLI**
```bash
cargo run --bin subject_cli
```

**What it shows:**
- Automated subject generation
- Physics domain examples
- Grammar graph construction

---

### **ELP & Dynamic Color Examples**

**Dynamic ELP RL Demo**
```bash
cargo run --example dynamic_elp_rl_demo --features benchmarks,lake
```

**What it shows:**
- ELP tensor dynamics
- Reinforcement learning
- Color ML integration

---

## 🏗️ Binary Targets

### **Consciousness Streaming Server**
```bash
cargo run --bin consciousness_streaming_server --features transport,agents
```

**Description:** WebTransport server for real-time consciousness streaming

**Default port:** 4433  
**Protocol:** WebTransport (QUIC)  

---

### **Flux Matrix Demos**
```bash
# Standard flux matrix
cargo run --bin flux_matrix

# Vortex view
cargo run --bin vortex_view

# Flux matrix with vortex
cargo run --bin flux_matrix_vortex
```

---

### **Academic Benchmark**
```bash
cargo run --bin academic_benchmark
```

**Description:** Academic performance benchmarks for AI model evaluation

---

## 🧪 Tests

### **Run All Tests**
```bash
cargo test
```

### **Library Tests Only**
```bash
cargo test --lib
```

### **Specific Module Tests**
```bash
# Background learning tests
cargo test background_learner --lib

# Consciousness tests
cargo test consciousness --lib

# RAG tests
cargo test rag --lib

# Hallucination detection tests
cargo test hallucinations --lib
```

### **Integration Tests**
```bash
# Cascade workflow integration
cargo test --test cascade_integration --features voice,lake

# Background learning integration
cargo test --test background_learning_integration --features agents
```

### **With Features**
```bash
# Test with all features
cargo test --all-features

# Test with specific features
cargo test --features agents,rag,lake,persistence
```

---

## 🎛️ Feature Flags

### **Core Features**
- `agents` - Background learning & consciousness simulation
- `rag` - Retrieval-Augmented Generation system
- `persistence` - Memory Palace state saving (NEW in v1.6.0)
- `postgres` - PostgreSQL RAG backend with pgvector (NEW in v1.6.0)
- `lake` - Confidence Lake encrypted storage

### **ML Backends**
- `tract` - Pure Rust ONNX (default, recommended)
- `onnx` - C++ ONNX runtime (Windows issues)
- `burn-backend` - Burn ML framework
- `candle-backend` - Candle ML framework

### **Visualization**
- `bevy_support` - 3D visualization with Bevy

### **Voice Processing**
- `voice` - Voice pipeline with FFT
- `voice-cuda` - Voice with CUDA acceleration

### **Transport**
- `transport` - WebTransport streaming server

### **Other**
- `formal-verification` - Z3 SMT solver
- `benchmarks` - Performance benchmarking
- `color_ml` - Aspect Color ML

### **Feature Combinations**

**Recommended for Development:**
```bash
--features agents,rag,lake,persistence
```

**Full System (No GPU):**
```bash
--features agents,rag,lake,persistence,postgres,tract,bevy_support
```

**Full System with PostgreSQL:**
```bash
--features agents,rag,lake,persistence,postgres
```

**Maximum Features:**
```bash
--all-features
```

---

## 💻 Development

### **Fast Build for Development**
```bash
cargo build --features agents
```

### **Check Code Without Building**
```bash
cargo check --lib
```

### **Fix Warnings**
```bash
cargo fix --lib
```

### **Format Code**
```bash
cargo fmt
```

### **Clippy Lints**
```bash
cargo clippy --all-features
```

### **Documentation**
```bash
# Build docs
cargo doc --no-deps --all-features

# Build and open
cargo doc --no-deps --all-features --open
```

---

## 🚀 Production

### **Release Build (Optimized)**
```bash
cargo build --release --features agents,persistence,postgres,lake,rag
```

### **Size Optimized**
```bash
# Add to Cargo.toml profile:
[profile.release]
opt-level = "z"
lto = true
codegen-units = 1

# Then build
cargo build --release --features agents,persistence,postgres
```

### **Run Production Server**
```bash
# Memory Palace with PostgreSQL
cargo run --release --bin consciousness_streaming_server --features transport,agents,persistence,postgres

# Or run the compiled binary
./target/release/consciousness_streaming_server
```

---

## 📊 Common Workflows

### **New Feature Development**
```bash
# 1. Check code
cargo check --lib

# 2. Run relevant tests
cargo test background_learner --lib

# 3. Run example
cargo run --example memory_palace_demo --features agents,persistence

# 4. Format & lint
cargo fmt && cargo clippy
```

### **Before Commit**
```bash
# Full test suite
cargo test --all-features

# Build everything
cargo build --all-features

# Check no warnings
cargo check --lib 2>&1 | grep warning
```

### **Performance Testing**
```bash
# Release build with benchmarks
cargo run --example academic_benchmark --release --features benchmarks

# Memory Palace performance
cargo run --example memory_palace_demo --release --features agents,persistence,postgres
```

---

## 🔍 Troubleshooting

### **PostgreSQL Connection Issues**
```bash
# Check PostgreSQL is running
psql -l

# Create database
createdb spatial_vortex

# Enable pgvector
psql spatial_vortex -c "CREATE EXTENSION vector;"

# Test connection
psql postgresql://localhost/spatial_vortex
```

### **Build Errors**

**Missing Dependencies:**
```bash
# Update dependencies
cargo update

# Clean and rebuild
cargo clean && cargo build --features agents
```

**Feature Conflicts:**
```bash
# Use only compatible features
cargo build --features agents,persistence  # Not --all-features
```

### **Runtime Errors**

**Confidence Lake:**
```bash
# Ensure directory exists for lake file
mkdir -p data/
cargo run --features lake
```

**Memory Palace:**
```bash
# State file permissions
chmod 644 consciousness_state.json
```

---

## 📚 Version History

### **v1.6.0 "Memory Palace"** (Current)
- ✅ Persistent consciousness state
- ✅ PostgreSQL RAG backend
- ✅ Auto-save functionality
- ✅ Scenario 3 support

**New Examples:**
- `memory_palace_demo`

**New Features:**
- `persistence`
- `postgres`

### **v1.5.1 "Background Learning"**
- ✅ RAG integration
- ✅ Confidence Lake review
- ✅ Continuous learning

**New Examples:**
- `background_learning_full_demo`

### **v1.5.0 "Conscious Streaming"**
- ✅ WebTransport streaming
- ✅ Real-time analytics
- ✅ Word-level insights

**New Examples:**
- `consciousness_streaming_demo`

**New Binaries:**
- `consciousness_streaming_server`

---

## 🎯 Quick Reference

| Task | Command |
|------|---------|
| **Latest Demo** | `cargo run --example memory_palace_demo --features agents,persistence` |
| **Full System** | `cargo run --example memory_palace_demo --features agents,persistence,postgres,lake` |
| **Tests** | `cargo test --lib` |
| **Release Build** | `cargo build --release --features agents,persistence,postgres` |
| **Check Code** | `cargo check --lib` |
| **Documentation** | `cargo doc --no-deps --all-features --open` |
| **Format** | `cargo fmt` |
| **Lint** | `cargo clippy --all-features` |

---

## 🌟 Recommended Commands

### **For First-Time Users**
```bash
cargo run --example memory_palace_demo --features agents,persistence
```

### **For Production Deployment**
```bash
cargo build --release --features agents,persistence,postgres,lake,rag
```

### **For Development**
```bash
cargo check --lib && cargo test background_learner --lib
```

### **For Full Experience**
```bash
cargo run --example memory_palace_demo --features agents,persistence,postgres,lake,bevy_support --release
```

---

**Version:** 1.6.0 "Memory Palace"  
**Last Updated:** November 6, 2025  
**Status:** ✅ Production Ready  

🏛️ **Run commands for immortal consciousness!** ⚡
