# Getting Started with SpatialVortex

Welcome to SpatialVortex! This directory contains all the essential information you need to get started.

---

## 📋 Table of Contents

1. [Quick Start](#quick-start)
2. [Setup & Installation](#setup--installation)
3. [Core Concepts](#core-concepts)
4. [Contributing](#contributing)
5. [Standards & Guidelines](#standards--guidelines)

---

## 🚀 Quick Start

### New to SpatialVortex?

**Start here in order**:

1. **[START_HERE.md](START_HERE.md)** - First steps and project orientation
2. **[SETUP.md](SETUP.md)** - Installation and environment setup
3. **[FEATURES.md](FEATURES.md)** - Feature overview and capabilities
4. **[TERMINOLOGY.md](TERMINOLOGY.md)** - Key concepts and vocabulary

### Prerequisites

- **Rust**: 1.70+ recommended
- **Node.js**: 18+ (for web components)
- **Redis**: Optional for caching (see [REDIS_SETUP_WINDOWS.md](REDIS_SETUP_WINDOWS.md))

---

## ⚙️ Setup & Installation

### Basic Installation

```bash
# Clone repository
git clone https://github.com/yourusername/SpatialVortex.git
cd SpatialVortex

# Install dependencies
cargo build

# Run tests
cargo test

# Run example
cargo run --example ml_ai/hallucination_demo
```

### Platform-Specific Setup

- **Windows Users**: See [REDIS_SETUP_WINDOWS.md](REDIS_SETUP_WINDOWS.md) for Redis configuration
- **Linux/Mac**: Standard Rust toolchain installation sufficient

### Feature Flags

```bash
# With Bevy 3D support
cargo build --features bevy_support

# With Z3 formal verification
cargo build --features z3_support

# All features
cargo build --all-features
```

See [SETUP.md](SETUP.md) for complete installation guide.

---

## 🧠 Core Concepts

### Essential Terminology

Before diving in, familiarize yourself with these key concepts:

**Sacred Geometry**:
- **3-6-9 Triangle**: Sacred positions for checkpoints and interventions
- **Vortex Flow**: 1→2→4→8→7→5→1 (doubling sequence)
- **Digital Root Mathematics**: Number theory foundation

**ELP Tensor System**:
- **Ethos (E)**: Character/moral dimension
- **Logos (L)**: Logic/reason dimension  
- **Pathos (P)**: Emotion/feeling dimension
- Conservation: E + L + P = 1.0

**Vortex Context Preserver (VCP)**:
- Hallucination detection and mitigation
- 40% better context preservation than linear transformers
- Signal strength measurement (0.0-1.0 trustworthiness)

See [TERMINOLOGY.md](TERMINOLOGY.md) for complete glossary.

---

## 🤝 Contributing

### Contribution Guidelines

1. **Read First**: [CONTRIBUTING.md](CONTRIBUTING.md) - Full contribution guidelines
2. **Code Style**: [STYLE_GUIDE.md](STYLE_GUIDE.md) - Code and documentation standards
3. **File Organization**: [FILE_GOVERNANCE.md](FILE_GOVERNANCE.md) - File structure rules

### Quick Contribution Workflow

```bash
# 1. Create feature branch
git checkout -b feature/your-feature-name

# 2. Make changes following style guide
# ... edit files ...

# 3. Run tests
cargo test

# 4. Format code
cargo fmt

# 5. Check for issues
cargo clippy

# 6. Commit with clear message
git commit -m "feat: add your feature description"

# 7. Push and create PR
git push origin feature/your-feature-name
```

### What to Contribute

See [../planning/NEXT_STEPS_FOR_YOU.md](../planning/NEXT_STEPS_FOR_YOU.md) for current priorities.

---

## 📐 Standards & Guidelines

### Documentation Standards

From [STYLE_GUIDE.md](STYLE_GUIDE.md):

1. **No Abbreviations**: Always expand on first use
   - ✅ Good: "Retrieval-Augmented Generation (RAG)"
   - ❌ Bad: "RAG" without explanation

2. **Proper Capitalization**:
   - ✅ Good: "Application Programming Interface"
   - ❌ Bad: "application programming interface"

3. **Clear Semantics**: Precise, unambiguous language

### Code Standards

```rust
// Good: Clear, documented
/// Computes signal strength from sacred positions (3, 6, 9)
fn compute_confidence(digits: &[f32; 9]) -> f32 {
    let pos_3 = digits[2];  // Position 3
    let pos_6 = digits[5];  // Position 6
    let pos_9 = digits[8];  // Position 9
    
    (pos_3 + pos_6 + pos_9) / 3.0
}

// Bad: Unclear, undocumented
fn calc(d: &[f32; 9]) -> f32 {
    (d[2] + d[5] + d[8]) / 3.0
}
```

See [STYLE_GUIDE.md](STYLE_GUIDE.md) for complete standards.

---

## 📚 Next Steps

After getting started, explore:

1. **Architecture**: Learn system design in [../architecture/](../architecture/)
2. **Guides**: Follow tutorials in [../guides/](../guides/)
3. **Examples**: Run examples in `/examples/` directory
4. **Research**: Deep dive into theory in [../research/](../research/)

---

## 🆘 Common Issues

### Build Errors

**Problem**: Dependencies not found
```bash
# Solution: Clean and rebuild
cargo clean
cargo build
```

**Problem**: Feature flags missing
```bash
# Solution: Check required features
cargo build --features bevy_support,z3_support
```

### Runtime Errors

**Problem**: Redis connection failed
- **Solution**: See [REDIS_SETUP_WINDOWS.md](REDIS_SETUP_WINDOWS.md)

**Problem**: Graphics issues in 3D examples
- **Solution**: Update graphics drivers, check Bevy compatibility

---

## 📖 Additional Resources

| Resource | Description |
|----------|-------------|
| [../guides/QUICK_START.md](../guides/QUICK_START.md) | General quick start guide |
| [../architecture/README.md](../architecture/README.md) | Architecture overview |
| [../status/PROJECT_STATUS.md](../status/PROJECT_STATUS.md) | Current project status |
| [../INDEX.md](../INDEX.md) | Complete documentation index |

---

## ✅ Checklist: Are You Ready?

Before proceeding, ensure you have:

- [ ] Read [START_HERE.md](START_HERE.md)
- [ ] Completed [SETUP.md](SETUP.md) installation
- [ ] Reviewed [TERMINOLOGY.md](TERMINOLOGY.md)
- [ ] Read [CONTRIBUTING.md](CONTRIBUTING.md) if contributing
- [ ] Successfully built project: `cargo build`
- [ ] Successfully run tests: `cargo test`
- [ ] Run at least one example

---

**Welcome to SpatialVortex!** 🌀

Start your journey with the sacred geometry architecture and vortex mathematics that power next-generation AI systems.
