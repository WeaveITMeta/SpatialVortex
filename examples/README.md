# SpatialVortex Examples

Organized examples demonstrating various features of the SpatialVortex framework.

## 📂 Directory Structure

### `visualization/`
3D and 2D rendering demonstrations using Bevy and custom renderers.

- **epic_flux_3d_native.rs** - Native 3D visualization with Bevy (full window app)
- **flux_2d_visualization.rs** - 2D flux matrix visualization
- **flux_3d_bevy_existing.rs** - 3D Bevy integration demo
- **render_flux_2d.rs** - 2D flux rendering
- **unified_visualization_demo.rs** - Unified visualization interface
- **dynamic_color_generation.md** - Dynamic color system documentation

**Run example:**
```bash
cargo run --example visualization/epic_flux_3d_native --features bevy_support --release
```

---

### `ml_ai/`
Machine Learning and Artificial Intelligence examples including inference, hallucination detection, and context management.

- **ai_router_example.rs** - AI model routing and consensus
- **ml_ensemble_demo.rs** - ML ensemble methods
- **onnx_sacred_geometry_demo.rs** - ONNX runtime with sacred geometry
- **transformer_sacred_geometry_demo.rs** - Transformer architecture with vortex math
- **dynamic_context_demo.rs** - Dynamic positional encoding (unlimited context)
- **hallucination_demo.rs** - VortexContextPreserver hallucination detection
- **grok_vortex_demo.rs** - Grok integration with vortex flow

**Run example:**
```bash
cargo run --example ml_ai/hallucination_demo
```

---

### `pipelines/`
Complete end-to-end pipeline demonstrations.

- **asi_complete_pipeline_demo.rs** - Complete ASI (Artificial Superintelligence) pipeline
- **asi_full_pipeline_demo.rs** - Full ASI pipeline with compression
- **voice_pipeline_demo.rs** - Voice → Tensor → 3D pipeline (design reference)

**Run example:**
```bash
cargo run --example pipelines/asi_complete_pipeline_demo
```

---

### `core/`
Core functionality and foundational system demonstrations.

- **database_test.rs** - Spatial database testing
- **formal_verification_demo.rs** - Z3 SMT solver formal verification (7 axioms, 3 theorems)

**Run example:**
```bash
cargo run --example core/formal_verification_demo --features z3_support
```

---

## 🚀 Quick Start

### Basic Workflow
```bash
# List all examples
cargo run --example

# Run specific example
cargo run --example <category>/<example_name>

# Run with optimizations
cargo run --example <category>/<example_name> --release

# Run with specific features
cargo run --example <category>/<example_name> --features bevy_support,z3_support
```

### Common Features
- `bevy_support` - Enable Bevy 3D rendering
- `z3_support` - Enable Z3 formal verification
- `candle_support` - Enable Candle ML backend
- `burn_support` - Enable Burn ML backend

---

## 📖 Documentation

For more details on each component:
- **Sacred Geometry**: `docs/research/VORTEX_MATHEMATICS_FOUNDATION.md`
- **Hallucination Detection**: `docs/research/HALLUCINATIONS.md`
- **Architecture**: `docs/architecture/`
- **API Reference**: `docs/research/`

---

## 🧪 Testing Examples

```bash
# Test all examples compile
cargo check --examples

# Run tests
cargo test

# Run specific example tests
cargo test --example <category>/<example_name>
```

---

## 📊 Example Categories Summary

| Category | Files | Focus Area |
|----------|-------|------------|
| **visualization/** | 6 | 3D/2D rendering, Bevy integration |
| **ml_ai/** | 7 | ML inference, hallucination detection |
| **pipelines/** | 3 | End-to-end system demos |
| **core/** | 2 | Database, formal verification |

**Total**: 18 examples

---

## 🎯 Recommended Learning Path

1. **Start with Core**: `core/formal_verification_demo.rs` - Understand mathematical foundation
2. **Visualize**: `visualization/flux_2d_visualization.rs` - See the vortex pattern
3. **ML Basics**: `ml_ai/hallucination_demo.rs` - Learn hallucination detection
4. **Full Pipeline**: `pipelines/asi_complete_pipeline_demo.rs` - See it all together
5. **Advanced 3D**: `visualization/epic_flux_3d_native.rs` - Interactive 3D experience

---

## 🔧 Troubleshooting

**Build errors?**
- Ensure all dependencies are installed: `cargo check`
- Check feature flags match example requirements
- Verify Rust version: `rustc --version` (1.70+ recommended)

**Runtime errors?**
- Check logs in console output
- Verify input data paths (if applicable)
- Ensure graphics drivers updated (for visualization examples)

---

## 🤝 Contributing

When adding new examples:
1. Place in appropriate category directory
2. Add entry to this README
3. Include inline documentation
4. Add test cases if applicable
5. Update Cargo.toml `[[example]]` entries if needed

---

**Architecture Grade**: A+ (Organization)  
**Examples Coverage**: Comprehensive across all major components
