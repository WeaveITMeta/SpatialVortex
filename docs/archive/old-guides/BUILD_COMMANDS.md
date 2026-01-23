# Build Commands Reference

Quick reference for all build and run commands in SpatialVortex.

## 🏗️ Core Library

### Build Library
```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# With Bevy 3D support
cargo build --release --features bevy_support
```

### Run Tests
```bash
# All tests
cargo test

# Specific test module
cargo test inference_engine
cargo test compression
cargo test flux_matrix

# Integration tests
cargo test --test integration_tests

# With output
cargo test -- --nocapture

# Specific test by name
cargo test test_controlled_inference
```

---

## 🎮 Binary Executables

### 1. Flux Matrix Visualization (3D)
**Interactive 3D visualization of the flux pattern with Bevy**

```bash
# Build
cargo build --bin flux_matrix --features bevy_support --release

# Run
cargo run --bin flux_matrix --features bevy_support

# Release mode (faster)
cargo run --bin flux_matrix --features bevy_support --release
```

**Controls:**
- `Space` - Spawn test word
- `A` - Toggle auto-spawn
- `F` - Toggle camera follow
- `1-9` - Focus on flux position
- `Mouse` - Rotate camera
- `Scroll` - Zoom

### 2. Vortex View
**Alternative viewer with neural network visualization**

```bash
cargo run --bin vortex_view --features bevy_support
```

### 3. Subject CLI
**Generate new subject matrices**

```bash
# Generate single subject
cargo run --bin subject_cli -- generate --subject "Chemistry"

# Shorter syntax
cargo run --bin subject_cli -- "Chemistry"

# Multiple subjects
cargo run --bin subject_cli -- "Biology"
cargo run --bin subject_cli -- "Economics"
cargo run --bin subject_cli -- "Psychology"
```

**Requirements:**
- Set `GROK_API_KEY` environment variable
- Or configure in `.env` file

### 4. Camera (Experimental)
```bash
cargo run --bin camera --features bevy_support
```

---

## 🌐 WASM Build (Web)

### Build WASM Binary
```bash
# Add WASM target (one-time setup)
rustup target add wasm32-unknown-unknown

# Build
cargo build --target wasm32-unknown-unknown --release --bin flux_matrix --features bevy_support

# Bind for web
wasm-bindgen target/wasm32-unknown-unknown/release/flux_matrix.wasm \
  --out-dir web/static/bevy \
  --target web
```

### Full Web Stack
```bash
# Terminal 1: Backend API
cd backend-rs
cargo run

# Terminal 2: Frontend UI
cd web
bun install      # or: pnpm install / npm install
bun run dev      # or: pnpm dev / npm run dev
```

**Access:**
- Frontend: http://localhost:5173
- Backend API: http://localhost:28080
- Health check: http://localhost:28080/health

---

## 📦 Package Management

### Web Dependencies
```bash
cd web

# Using Bun (recommended)
bun install
bun run dev
bun run build
bun run preview

# Using pnpm
pnpm install
pnpm dev
pnpm build

# Using npm
npm install
npm run dev
npm run build
```

---

## 🧹 Cleanup

```bash
# Clean Rust build artifacts
cargo clean

# Clean web node_modules
cd web
rm -rf node_modules
rm -rf .svelte-kit
```

---

## 🔍 Documentation Generation

```bash
# Generate and open Rust docs
cargo doc --open --no-deps

# Generate with private items
cargo doc --document-private-items --no-deps
```

---

## 🚀 Quick Start Commands

### First Time Setup
```bash
# Clone repository
git clone https://github.com/WeaveSolutions/SpatialVortex.git
cd SpatialVortex

# Build everything
cargo build --release --features bevy_support

# Run tests
cargo test --lib

# Start backend
cd backend-rs
cargo run &

# Start frontend
cd ../web
bun install
bun run dev
```

### Daily Development
```bash
# Run tests (fastest feedback)
cargo test

# Build and run specific binary
cargo run --bin flux_matrix --features bevy_support

# Format code
cargo fmt

# Check for issues
cargo clippy
```

---

## 📊 Verification

### After Updates
```bash
# 1. Clean build
cargo clean
cargo build --release --features bevy_support

# 2. Run all tests
cargo test --lib

# 3. Verify binaries
cargo run --bin flux_matrix --features bevy_support
cargo run --bin subject_cli -- --help

# 4. Check web build
cd web
bun run build
```

### Check Available Binaries
```bash
# List all binaries in Cargo.toml
cargo run --bin  # Press Tab to autocomplete
```

---

## 🛠️ Advanced

### Performance Profiling
```bash
# Build with debug symbols
cargo build --profile release-with-debug

# Use flamegraph
cargo flamegraph --bin flux_matrix --features bevy_support
```

### Cross-Compilation
```bash
# Add target
rustup target add x86_64-pc-windows-gnu

# Build for target
cargo build --target x86_64-pc-windows-gnu --release
```

### Custom Features
```bash
# Build without Bevy support
cargo build --release --no-default-features

# Build with specific features
cargo build --release --features "compression,inference"
```

---

## ⚠️ Common Issues

### Issue: Binary not found
```bash
# Solution: Use full path or --bin flag
cargo run --bin flux_matrix --features bevy_support
```

### Issue: WASM bindgen version mismatch
```bash
# Solution: Match versions
cargo install wasm-bindgen-cli --version 0.2.92  # Match Cargo.toml
```

### Issue: Port already in use
```bash
# Find process on port 28080
lsof -i :28080        # macOS/Linux
netstat -ano | findstr :28080  # Windows

# Kill process
kill -9 <PID>         # macOS/Linux
taskkill /PID <PID> /F  # Windows
```

---

## 📚 Related Documentation

- [Quick Start Guide](docs/guides/QUICK_START.md)
- [Migration Guide](MIGRATION.md)
- [Contributing Guidelines](CONTRIBUTING.md)
- [Full Documentation](docs/README.md)

---

**Last Updated**: October 22, 2025  
**Version**: 1.1.0
