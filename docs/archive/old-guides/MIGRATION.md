# Migration Guide: diamond_viz → flux_matrix

**Version**: 1.0.0 → 1.1.0  
**Date**: October 22, 2025

## Breaking Changes

### Binary Renamed

The 3D visualization binary has been renamed for consistency:

**Old Name**: `diamond_viz`  
**New Name**: `flux_matrix`

### Required Changes

#### 1. Build Commands

**Before:**
```bash
cargo build --bin diamond_viz --features bevy_support
cargo run --bin diamond_viz --features bevy_support
```

**After:**
```bash
cargo build --bin flux_matrix --features bevy_support
cargo run --bin flux_matrix --features bevy_support
```

#### 2. WASM Build

**Before:**
```bash
cargo build --target wasm32-unknown-unknown --release --bin diamond_viz --features bevy_support
wasm-bindgen target/wasm32-unknown-unknown/release/diamond_viz.wasm \
  --out-dir web/static/bevy \
  --target web
```

**After:**
```bash
cargo build --target wasm32-unknown-unknown --release --bin flux_matrix --features bevy_support
wasm-bindgen target/wasm32-unknown-unknown/release/flux_matrix.wasm \
  --out-dir web/static/bevy \
  --target web
```

#### 3. Scripts & CI/CD

If you have custom build scripts or CI/CD pipelines, update any references:

- `diamond_viz` → `flux_matrix`
- `diamond_viz.wasm` → `flux_matrix.wasm`
- Any hardcoded paths or binary names

### Code Changes

#### Rust Module Names

If you import from the renamed module:

**Before:**
```rust
use spatial_vortex::diamond_mesh::{DiamondGeometry, ...};
```

**After:**
```rust
use spatial_vortex::flux_mesh::{FluxGeometry, ...};
```

#### TypeScript/JavaScript

**Before:**
```typescript
interface DiamondNode {
  position: number;
  // ...
}

window.initDiamond?.();
```

**After:**
```typescript
interface FluxNode {
  position: number;
  // ...
}

window.initFluxMatrix?.();
```

### What Stays the Same

The following are **NOT** affected by this change:

- ✅ `Diamond` struct in `models.rs` (different concept - high-confidence moments)
- ✅ API endpoints and responses
- ✅ Core compression and inference algorithms
- ✅ Subject matrices and flux patterns
- ✅ Test suite (automatically updated)

## Verification

After updating, verify the changes work:

```bash
# 1. Clean build
cargo clean
cargo build --release --features bevy_support

# 2. Run new binary
cargo run --bin flux_matrix --features bevy_support

# 3. Run tests
cargo test --lib

# 4. Check available binaries
cargo run --bin  # Press tab to see available binaries
```

## Available Binaries

After this update, these binaries are available:

```bash
# 3D Flux Matrix Visualization (renamed)
cargo run --bin flux_matrix --features bevy_support

# Alternative Vortex Viewer
cargo run --bin vortex_view --features bevy_support

# Subject Matrix CLI
cargo run --bin subject_cli

# Camera System (experimental)
cargo run --bin camera --features bevy_support
```

## Documentation Updates

All documentation has been updated:

- ✅ README.md
- ✅ QUICK_START.md
- ✅ docs/architecture/
- ✅ docs/guides/
- ✅ docs/integration/
- ✅ docs/reports/

## Rollback (if needed)

If you need to temporarily revert:

```bash
git checkout 682568a  # Previous commit before rename
cargo build --bin diamond_viz --features bevy_support
```

However, we recommend updating to the new naming convention for consistency.

## Questions or Issues?

- 📖 Documentation: See [docs/README.md](docs/README.md)
- 🐛 Issues: [GitHub Issues](https://github.com/WeaveSolutions/SpatialVortex/issues)
- 💬 Discussions: [GitHub Discussions](https://github.com/WeaveSolutions/SpatialVortex/discussions)

## Summary

This is a **naming consistency update**. The functionality remains identical - only the binary and module names have changed to align with the project's "flux matrix" terminology. Update your build commands and you're good to go!
