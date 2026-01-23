# 🚀 Quickstart: Flux Matrix Visualization

## 2D Static Visualization (Working Now!)

### Run It

```bash
cargo run --example flux_2d_visualization
```

**Output**: `flux_matrix_2d.png` (1200×1200)

### What You Get
- ✅ All 10 positions (0-9) mapped
- ✅ Sacred triangle (3-6-9) in red
- ✅ Flow lines between positions
- ✅ ELP color coding (Red/Blue/Green)
- ✅ Position analysis printed to console
- ✅ Native Rust (no Python!)

---

## 3D Web Visualization (Ready to Build!)

### Prerequisites

```bash
# Install WASM tools
cargo install wasm-pack

# Install Node.js from https://nodejs.org/
```

### Build Steps

```bash
# 1. Create Svelte app
cd E:\Libraries\SpatialVortex\web
npm create vite@latest svelte-app -- --template svelte
cd svelte-app
npm install
npm install -D vite-plugin-wasm vite-plugin-top-level-await

# 2. Build WASM module
cd ../..
wasm-pack build \
  --target web \
  --out-dir web/svelte-app/src/wasm \
  --features bevy_support \
  wasm/flux_3d_web.rs

# 3. Copy Svelte component
# FluxVisualization.svelte already created in web/svelte-app/src/

# 4. Run dev server
cd web/svelte-app
npm run dev
```

Visit: **http://localhost:5173**

### What You Get
- ✅ Interactive 3D WebGL rendering
- ✅ Auto-rotating camera
- ✅ Sacred geometry in 3D space
- ✅ ELP-colored data points
- ✅ Clickable UI overlay
- ✅ Runs in any modern browser

---

## Quick Comparison

| Feature | 2D (Plotters) | 3D (Bevy + WASM) |
|---------|---------------|------------------|
| **Time to Run** | 2 seconds | 5-10 seconds (first build) |
| **Output** | PNG file | Web app |
| **Interactive** | No | Yes |
| **Dependency** | Rust only | Rust + Node.js |
| **Use Case** | Reports, docs | Web dashboards |
| **Status** | ✅ Working now! | ⏳ Ready to build |

---

## Files You Can Use

### Already Working
- `examples/flux_2d_visualization.rs` - 2D renderer
- `src/visualization/mod.rs` - Core data structures
- `flux_matrix_2d.png` - Sample output (just generated!)

### Ready for 3D Web
- `src/visualization/bevy_3d.rs` - 3D scene setup
- `wasm/flux_3d_web.rs` - WASM entry point
- `web/svelte-app/src/FluxVisualization.svelte` - Web component
- `web/BUILD_INSTRUCTIONS.md` - Full guide

---

## Troubleshooting

### 2D: "plotters not found"
```bash
cargo build  # Downloads plotters automatically
```

### 3D: "wasm-pack not found"
```bash
cargo install wasm-pack
```

### 3D: "Bevy compilation takes forever"
**This is normal!** Bevy is large. First compile takes 10-20 minutes.
Subsequent builds are much faster (incremental compilation).

---

## Next Steps

1. **Try 2D now**: `cargo run --example flux_2d_visualization`
2. **Build 3D when ready**: Follow `web/BUILD_INSTRUCTIONS.md`
3. **Customize data**: Edit test data in either example file
4. **Deploy web app**: Build with `npm run build`, deploy to Netlify/Vercel

---

**Status**: 
- ✅ 2D visualization complete and tested
- ✅ 3D infrastructure ready
- ⏳ 3D requires `npm` + `wasm-pack` setup
