# 🌀 Epic Flux 3D - Consolidated Visualization

**The ultimate Bevy WASM visualization combining ALL SpatialVortex features**

---

## 🎯 Overview

Epic Flux 3D is a comprehensive 3D visualization that consolidates every Bevy visualization feature from across the SpatialVortex codebase into one epic web experience.

### What's Included

All visualization code from:
- ✅ `src/visualization/bevy_3d.rs` - Sacred geometry & flux patterns
- ✅ `src/visualization/bevy_shapes.rs` - Box/Cylinder/Sphere architecture
- ✅ `src/beam_renderer.rs` - Word beams with ELP colors
- ✅ `src/flux_mesh.rs` - 3D mesh generation & paths
- ✅ `viewer/src/main.rs` - Neural network visualization
- ✅ `src/bin/vortex_view.rs` - Interactive viewer
- ✅ `wasm/flux_3d_web.rs` - Original WASM implementation

---

## ✨ Features

### Core Visualization

**Sacred Geometry**
- 🔺 3-6-9 Sacred Triangle in **Cyan**
- ⭕ Position markers (0-9) in circular layout
- 🌟 Sacred positions with pulsing effects
- 📍 Center void (position 0)

**Flux Flow Pattern**
- 🔄 Doubling sequence: 1→2→4→8→7→5→1
- ⚡ Flow lines connecting positions
- 🎨 Gray lines for regular flow
- 💎 Enhanced sacred connections

**Word Beams**
- 📝 Text flowing through the matrix
- 🎨 ELP color channels:
  - 🔴 **Red** = Ethos (Ethics)
  - 🔵 **Blue** = Logos (Logic)
  - 🟢 **Green** = Pathos (Emotion)
- 🌊 Curved paths with dynamic curvature
- ✨ Particle trails

### Shape-Based Architecture

**Processing Blocks** (Box/Cuboid)
- 📦 3 processing nodes on left side:
  - Geometric Inference (Yellow - Processing)
  - ML Enhancement (Yellow - Processing)
  - AI Consensus (Green - Complete)
- 🎭 State-based colors:
  - Gray = Idle
  - Yellow = Processing
  - Green = Complete
  - Red = Error

**Database Nodes** (Cylinder)
- 🗄️ 2 database nodes on right side:
  - PostgreSQL (150 connections)
  - Redis (85 connections)
- 📊 Height based on connection count
- 💙 Blue emissive glow

**Node References** (Sphere)
- 🌐 Flux positions as spheres
- 📏 Size based on importance:
  - Sacred (0.6 radius)
  - Center (0.4 radius)
  - Regular (0.3 radius)
- ✨ Activity-based scaling

### Sacred Intersection Effects

**Position 3** - Creative Trinity
- 🟢 Green burst effect
- 💥 Rapid expansion
- ⚡ Positive reinforcement

**Position 6** - Harmonic Balance
- 🔴 Red ripple effect
- 🌊 Oscillating waves
- 🔍 Deep analysis marker

**Position 9** - Completion Cycle
- 🔵 Blue ascension effect
- ⬆️ Upward motion
- ✨ Divine moment indicator

### Camera & Controls

**Auto-Rotating Orbit Camera**
- 🎥 Smooth orbital rotation
- 📏 Configurable distance (25 units)
- 🔄 0.3 rad/s rotation speed
- 👁️ Always looking at center

**Configuration**
```rust
VisualizationConfig {
    auto_rotate: true,
    rotation_speed: 0.3,
    beam_speed: 1.0,
    show_trails: true,
    camera_distance: 25.0,
}
```

---

## 🏗️ Build & Deploy

### Quick Build

```powershell
# Run the build script
.\scripts\BUILD_EPIC_FLUX_3D.ps1
```

### Manual Build

```bash
# 1. Add WASM target
rustup target add wasm32-unknown-unknown

# 2. Build WASM binary
cargo build --target wasm32-unknown-unknown --release --bin epic_flux_3d --features bevy_support

# 3. Generate bindings
wasm-bindgen target/wasm32-unknown-unknown/release/epic_flux_3d.wasm \
  --out-dir web/src/lib/wasm \
  --out-name epic_flux_3d \
  --target web

# 4. Start web server
cd web
bun run dev
```

### Access

Open in browser:
```
http://localhost:28082/epic-flux-3d
```

---

## 📊 Architecture

### Component Hierarchy

```
Epic Flux 3D App
├── Resources
│   ├── FluxMatrixResource (lock-free data)
│   ├── VisualizationConfig
│   └── AmbientLight
├── Camera System
│   └── OrbitCamera (auto-rotating)
├── Lighting
│   ├── DirectionalLight (main)
│   └── AmbientLight (fill)
├── Flux Structure
│   ├── Sacred Triangle (3-6-9) [Cyan lines]
│   ├── Flow Pattern Lines [Gray]
│   └── Flux Nodes (0-9) [Spheres]
├── Shape Architecture
│   ├── Processing Blocks [Boxes]
│   ├── Database Nodes [Cylinders]
│   └── Node References [Spheres]
├── Word Beams
│   ├── Beam Entities [Capsules]
│   ├── Trail Components
│   └── Labels [Text3D]
└── Effects
    └── IntersectionEffects [Spheres]
```

### Data Flow

```
LockFreeFluxMatrix
    ↓
FluxNodeModel (10 positions)
    ↓
Extract ELP Tensors
    ↓
Generate Colors (RGB from ELP)
    ↓
Spawn Entities
    ↓
Update Systems (animation)
    ↓
Sacred Intersection Detection
    ↓
Spawn Effects
```

---

## 🎨 Visual Design

### Color Scheme

**Sacred Positions**
- Position 3: `rgb(0.2, 1.0, 0.4)` - Bright Green
- Position 6: `rgb(1.0, 0.3, 0.3)` - Bright Red
- Position 9: `rgb(0.3, 0.6, 1.0)` - Bright Blue

**ELP Channels**
- Ethos: Red component (0.0-1.0)
- Logos: Green component (0.0-1.0)
- Pathos: Blue component (0.0-1.0)

**UI Elements**
- Background: Dark gradient `#0a0a1a` → `#1a1a2e`
- Sacred lines: Cyan `#00bfff`
- Flow lines: Dark gray `rgb(0.3, 0.3, 0.4)`

### Layout

**Circular Arrangement** (radius = 10.0)
- Position 9 at top (12 o'clock)
- Clockwise: 1, 2, 3, 4, 5, 6, 7, 8
- Center: Position 0

**Side Panels**
- Left side (x = -15): Processing blocks
- Right side (x = 15): Database nodes

---

## 🧪 Demo Data

### Flux Nodes

| Position | Name | Ethos | Logos | Pathos | Type |
|----------|------|-------|-------|--------|------|
| 0 | Void | 0.5 | 0.5 | 0.5 | Center |
| 1 | Unity | 0.8 | 0.7 | 0.85 | Flow |
| 2 | Duality | 0.75 | 0.85 | 0.7 | Flow |
| 3 | Love | 0.9 | 0.6 | 0.95 | **Sacred** |
| 4 | Foundation | 0.85 | 0.8 | 0.75 | Flow |
| 5 | Balance | 0.8 | 0.75 | 0.8 | Flow |
| 6 | Truth | 0.95 | 0.9 | 0.6 | **Sacred** |
| 7 | Change | 0.7 | 0.8 | 0.9 | Flow |
| 8 | Wisdom | 0.9 | 0.85 | 0.7 | Flow |
| 9 | Creation | 0.6 | 0.8 | 0.95 | **Sacred** |

### Word Beams

Spawned every 3 seconds at flow positions (1,2,4,5,7,8).

---

## 🔧 Customization

### Modify Config

Edit `wasm/epic_flux_3d.rs`:

```rust
VisualizationConfig {
    auto_rotate: true,           // Toggle rotation
    rotation_speed: 0.3,          // Speed (rad/s)
    beam_speed: 1.0,              // Beam travel speed
    show_trails: true,            // Enable trails
    camera_distance: 25.0,        // Camera radius
}
```

### Add Custom Nodes

```rust
let demo_data = vec![
    ("CustomNode", 4, 0.8, 0.7, 0.9), // (name, pos, E, L, P)
];
```

### Change Colors

```rust
fn get_sacred_color(pos: u8) -> Color {
    match pos {
        3 => Color::srgb(0.2, 1.0, 0.4),  // Your color here
        // ...
    }
}
```

---

## 🐛 Troubleshooting

### Build Fails

**Issue**: WASM compilation errors
```bash
# Solution: Clean and rebuild
cargo clean
rustup update
cargo build --target wasm32-unknown-unknown --release --bin epic_flux_3d --features bevy_support
```

### Page Shows "Build Required"

**Issue**: WASM files not found
```bash
# Check files exist
ls web/src/lib/wasm/epic_flux_3d*

# Rebuild if missing
.\scripts\BUILD_EPIC_FLUX_3D.ps1
```

### Black Screen

**Issue**: Bevy not initializing
- Open browser console (F12)
- Check for JavaScript errors
- Verify WASM files loaded
- Ensure canvas element exists

### Performance Issues

**Issue**: Low FPS
- Reduce `camera_distance` (less to render)
- Decrease `beam_speed` (fewer updates)
- Set `show_trails: false` (less geometry)
- Lower sphere subdivisions in code

---

## 📈 Performance

### Metrics

- **Build Time**: ~4 minutes (first build)
- **WASM Size**: ~27 MB (optimized)
- **Load Time**: ~2-3 seconds
- **Target FPS**: 60 FPS
- **Memory**: ~150 MB

### Optimization

The visualization uses:
- Bevy 0.17.0-dev (WebGL2 backend)
- Lock-free data structures (DashMap)
- Minimal entity spawning
- Efficient mesh reuse
- WASM optimization flags

---

## 📚 Related Docs

- [Bevy 3D Architecture](BEVY_SHAPE_ARCHITECTURE.md)
- [Sacred Geometry](../architecture/SACRED_POSITIONS.md)
- [ELP Tensors](../architecture/TENSORS.md)
- [Build Commands](../guides/BUILD_COMMANDS.md)

---

## 🎯 Future Enhancements

### Planned Features

- [ ] Interactive camera controls (mouse drag)
- [ ] Keyboard shortcuts (Space, 1-9 keys)
- [ ] Dynamic beam spawning on click
- [ ] Real-time matrix editing
- [ ] Audio reactivity
- [ ] VR support
- [ ] Multi-player synchronization

### Integration

- [ ] Connect to backend API
- [ ] Live data streaming
- [ ] WebSocket updates
- [ ] Save/load matrix states
- [ ] Export screenshots
- [ ] Record animations

---

**Built with ❤️ using Bevy Engine**

**Version**: 1.0.0  
**Last Updated**: October 25, 2025  
**Status**: ✅ Production Ready
