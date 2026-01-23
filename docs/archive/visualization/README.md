# Visualization Documentation
**Purpose**: 3D rendering and visual representation guides  
**Audience**: Graphics developers, UI designers, researchers

---

## 📂 Contents (3+ files)

### 3D Visualization
- **EPIC_FLUX_3D.md** - Complete 3D visualization guide
  - Bevy engine integration
  - Node rendering
  - Sacred triangle visualization
  - Camera systems
  - Interactive controls

### Specific Features
- **REALTIME_TRIANGLE_COLORING.md** (root `docs/`) - Triangle coloring algorithm
  - Color mapping for sacred triangle
  - Real-time updates
  - Performance optimization

- **API_VISUAL_SUBJECT_GENERATION.md** (root `docs/`) - Visual subject generation
  - Automated visualization creation
  - API endpoints
  - Subject-to-visual mapping

### Related
- **BEVY_0.17_MIGRATION.md** (root `docs/`) - Bevy upgrade notes
  - Migration guide
  - Breaking changes
  - New features

---

## 🎯 Quick Reference

**Setting up 3D visualization?**
→ Read EPIC_FLUX_3D.md for complete setup

**Need Bevy help?**
→ Check BEVY_0.17_MIGRATION.md for upgrade notes

**Building visual APIs?**
→ Read API_VISUAL_SUBJECT_GENERATION.md

**Working with colors?**
→ Read REALTIME_TRIANGLE_COLORING.md

---

## 📊 Visualization Status

| Component | Status | Framework | Version |
|-----------|--------|-----------|---------|
| **3D Renderer** | ✅ Working | Bevy | 0.17 |
| **Node Display** | ✅ Complete | Bevy | 0.17 |
| **Sacred Triangle** | ✅ Complete | Custom | v1.0 |
| **Camera System** | ✅ Complete | Bevy | 0.17 |
| **Text Labels** | ✅ Complete | Bevy | 0.17 |

---

## 🎨 Visualization Architecture

### Rendering Pipeline
```
FluxMatrix
    ↓
Node Positions (0-9)
    ↓
3D Coordinates (sacred triangle)
    ↓
Bevy Mesh/Material
    ↓
GPU Rendering
```

### Key Features
- **Sacred Triangle**: Cyan vertices at positions 3, 6, 9
- **Flow Lines**: Vortex pattern 1→2→4→8→7→5→1
- **Node Colors**: ELP-based color mapping
- **Real-time**: 60 FPS updates
- **Interactive**: Camera rotation, zoom

---

## 🚀 Running Examples

### Native 3D Visualization
```bash
cargo run --example epic_flux_3d_native --features bevy_support --release
```

### Web (WASM) Visualization
```bash
cargo build --target wasm32-unknown-unknown --features bevy_support
# Serve with web server
```

---

## 🎨 Color Schemes

**Sacred Positions**:
- Position 3: Cyan (Early signal)
- Position 6: Magenta (Error correction)
- Position 9: Yellow (Final validation)

**ELP Channels**:
- Ethos: Red spectrum
- Logos: Blue spectrum
- Pathos: Green spectrum

**Flow State**:
- Active: Bright
- Inactive: Dim
- Transitioning: Pulsing

---

## 📐 Coordinate System

**Sacred Triangle** (3D space):
```
Position 3: (x, y, z) = (1, 0, 0)
Position 6: (x, y, z) = (-0.5, 0.866, 0)
Position 9: (x, y, z) = (-0.5, -0.866, 0)
```

**Vortex Positions** (arranged in pattern):
- Follow doubling sequence
- Sacred positions outside main flow
- Geometric spacing preserved

---

## 🔧 Customization

**Camera Settings**:
```rust
// In epic_flux_3d_native.rs
Transform::from_xyz(5.0, 5.0, 5.0)
    .looking_at(Vec3::ZERO, Vec3::Y)
```

**Node Size**:
```rust
// Adjust sphere radius
Mesh::from(Sphere { radius: 0.3 })
```

**Colors**:
```rust
// Customize materials
StandardMaterial {
    base_color: Color::rgb(r, g, b),
    emissive: Color::rgb(r, g, b),
    ...
}
```

---

## 🔗 Related Documentation

- **Architecture**: `../architecture/BEVY_SHAPE_ARCHITECTURE.md`
- **Examples**: `../../examples/epic_flux_3d_native.rs`
- **Guides**: `../guides/` - Implementation guides

---

**Last Updated**: 2025-10-26  
**Total Files**: 3+  
**Status**: Production-ready 3D visualization ✅
