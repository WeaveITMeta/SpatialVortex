# Bevy WASM Integration for HLE Frontend

## ✅ Why Bevy Instead of Three.js

**User Requirement**: Use Bevy (Rust) for 3D visualization instead of Three.js (JavaScript)

**Benefits**:
- ✅ **Same Language**: Rust for both backend and frontend 3D
- ✅ **Type Safety**: Compile-time guarantees across stack
- ✅ **Performance**: Native Rust compiled to WASM
- ✅ **Already Implemented**: Bevy 3D visualization exists!
- ✅ **Sacred Geometry**: Built-in support for 3-6-9 patterns
- ✅ **Code Reuse**: Share data structures between backend and frontend

---

## 🏗️ Existing Bevy Implementation

### **Files Already Created** ✅

**1. Core 3D Visualization** (`src/visualization/bevy_3d.rs` - 400 lines)
- Sacred triangle (3-6-9) rendering
- Flux positions (0-9) as colored spheres
- Vortex flow lines (1→2→4→8→7→5→1)
- ELP color coding (Ethos/Logos/Pathos)
- Orbit camera with rotation
- Data point rendering

**2. WASM Entry Point** (`wasm/flux_3d_web.rs` - 120 lines)
- Bevy app configuration for web
- Demo data generation
- Canvas setup for WebGL
- Input handling

**3. Svelte Integration** (documented in `docs/visualization/BEVY_3D_WEB.md`)
- Svelte component wrapper
- WASM loading logic
- Canvas element management
- Info overlay UI

---

## 🎨 Bevy 3D Scene Features

### **Already Implemented** ✅

**Position Spheres** (0-9):
- Blue spheres for standard positions
- Gold spheres for sacred positions (3, 6, 9)
- Labeled with position numbers
- Positioned in vortex circle pattern

**Sacred Triangle**:
- Cyan lines connecting 3→6→9
- Emphasized with thicker lines
- Pulsing animation on sacred vertices

**Vortex Flow Lines**:
- Forward flow: 1→2→4→8→7→5→1 (cyan)
- Backward flow: 1→5→7→8→4→2→1 (magenta)
- Animated particle flow

**ELP Color Coding**:
- Ethos (Red): Position 3
- Logos (Blue): Position 9  
- Pathos (Green): Position 6
- Mixed colors for other positions

**Camera System**:
- Auto-rotating orbit camera
- Mouse controls for manual rotation
- Zoom in/out
- Reset to default view

**Data Points**:
- Small spheres at flux positions
- Colored by signal strength
- Hover tooltips (planned)
- Click for details (planned)

---

## 🔧 HLE-Specific Integration

### **What Needs to Be Added** (Week 9-10)

#### **1. Real-time Data Updates**
Currently uses demo data. Need to:
- Connect to HLE inference WebSocket
- Update scene with live inference results
- Animate transitions between states
- Show confidence scores on nodes

**Implementation**:
```rust
// In wasm/flux_3d_web.rs
#[wasm_bindgen]
pub fn update_inference_result(
    position: u8,
    confidence: f32,
    elp_ethos: f32,
    elp_logos: f32,
    elp_pathos: f32,
) {
    // Update Bevy scene with new data
    // Trigger animations
    // Update colors based on confidence
}
```

#### **2. Confidence Visualization**
- Node size scales with confidence (bigger = more confident)
- Color intensity based on confidence (bright = high, dim = low)
- Pulsing animation for active inference
- Red highlight for low confidence (<60%)

#### **3. Reasoning Path Highlighting**
- Highlight the inference path through positions
- Animated beam showing seed→sequence→position
- Trail effect for vortex flow
- Sacred position activation when used

#### **4. ELP Channel Bars**
Add 3D bar charts showing:
- Ethos percentage (red bar)
- Logos percentage (blue bar)
- Pathos percentage (green bar)
- Floating above main visualization

---

## 📦 Building Bevy WASM for Web

### **Build Commands**

**Development Build**:
```bash
cd E:\Libraries\SpatialVortex

# Build WASM module
wasm-pack build \
  --target web \
  --out-dir web/src/wasm \
  --features bevy_support \
  wasm/flux_3d_web.rs

# This creates:
# - web/src/wasm/flux_3d_web.js
# - web/src/wasm/flux_3d_web_bg.wasm
# - web/src/wasm/flux_3d_web.d.ts
```

**Production Build** (optimized):
```bash
wasm-pack build \
  --target web \
  --out-dir web/src/wasm \
  --features bevy_support \
  --release \
  wasm/flux_3d_web.rs
```

**Build Time**: ~2-3 minutes for full build, ~30s for incremental

---

## 🎭 Svelte Component Wrapper

### **Create** `web/src/lib/components/hle/FluxVisualizer.svelte`

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import init, { run_bevy_app, update_inference_result } from '$wasm/flux_3d_web';

  export let inferenceData: {
    position: number;
    confidence: number;
    elp: { ethos: number; logos: number; pathos: number };
  } | null = null;

  let wasmReady = false;
  let canvasElement: HTMLCanvasElement;

  onMount(async () => {
    // Initialize WASM module
    await init();
    wasmReady = true;

    // Start Bevy app on canvas
    run_bevy_app(canvasElement);
  });

  // Update visualization when inference data changes
  $: if (wasmReady && inferenceData) {
    update_inference_result(
      inferenceData.position,
      inferenceData.confidence,
      inferenceData.elp.ethos,
      inferenceData.elp.logos,
      inferenceData.elp.pathos
    );
  }
</script>

<div class="flux-visualizer">
  {#if !wasmReady}
    <div class="loading">Loading 3D visualization...</div>
  {/if}
  
  <canvas
    bind:this={canvasElement}
    id="bevy-canvas"
    width="800"
    height="600"
  />

  <div class="controls">
    <button on:click={() => /* reset camera */}>Reset View</button>
    <button on:click={() => /* toggle rotation */}>Toggle Rotation</button>
  </div>
</div>

<style>
  .flux-visualizer {
    position: relative;
    width: 100%;
    height: 600px;
  }

  canvas {
    width: 100%;
    height: 100%;
    border-radius: 8px;
    background: #000;
  }

  .loading {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    color: white;
    font-size: 18px;
  }

  .controls {
    position: absolute;
    bottom: 20px;
    right: 20px;
    display: flex;
    gap: 10px;
  }

  button {
    padding: 8px 16px;
    background: rgba(59, 130, 246, 0.8);
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }

  button:hover {
    background: rgba(59, 130, 246, 1);
  }
</style>
```

---

## 🚀 Development Workflow

### **Step 1: Make Rust Changes**
Edit `src/visualization/bevy_3d.rs` or `wasm/flux_3d_web.rs`

### **Step 2: Rebuild WASM**
```bash
wasm-pack build --target web --out-dir web/src/wasm --features bevy_support wasm/flux_3d_web.rs
```

### **Step 3: Test in Svelte**
```bash
cd web
bun run dev
```

### **Step 4: View in Browser**
Open `http://localhost:5173`

---

## 📊 Performance Considerations

### **WASM Bundle Size**
- **Development**: ~8-10 MB (with debug symbols)
- **Production**: ~2-3 MB (optimized + compressed)
- **Load Time**: ~1-2 seconds on broadband

**Optimization Tips**:
- Use `--release` flag for production
- Enable wasm-opt in Cargo.toml
- Use dynamic imports in Svelte
- Lazy load WASM only when visualization needed

### **Runtime Performance**
- **Frame Rate**: 60 FPS on modern hardware
- **WebGL**: Hardware accelerated
- **Memory**: ~50-100 MB for full scene
- **Render Time**: <16ms per frame

---

## 🎯 Integration Checklist for HLE

**Week 9: Backend Connection**
- [ ] Add WebSocket message types for inference data
- [ ] Create Rust→WASM data serialization
- [ ] Implement real-time updates from backend
- [ ] Test with HLE inference pipeline

**Week 10: Frontend Integration**
- [ ] Create FluxVisualizer.svelte component
- [ ] Integrate with Chat.svelte
- [ ] Add confidence visualization
- [ ] Add reasoning path highlighting
- [ ] Add ELP channel bars
- [ ] Test on multiple browsers (Chrome, Firefox, Safari)

**Polish**:
- [ ] Add loading states
- [ ] Add error handling
- [ ] Add keyboard shortcuts
- [ ] Add touch controls for mobile
- [ ] Add screenshot capability
- [ ] Add animation speed controls

---

## 🔍 Debugging Tips

### **WASM Not Loading**
Check browser console for:
- MIME type errors (need `application/wasm`)
- CORS errors (configure Vite)
- Import path errors (check `web/src/wasm/`)

### **Black Canvas**
- Bevy may need initialization time
- Check WebGL support in browser
- Verify canvas element exists before init

### **Poor Performance**
- Use production build (`--release`)
- Check browser WebGL capabilities
- Reduce particle count for older devices
- Use requestAnimationFrame throttling

---

## 📚 Resources

**Bevy Documentation**:
- https://bevyengine.org/learn/book/
- https://bevy-cheatbook.github.io/

**WASM Bindgen**:
- https://rustwasm.github.io/wasm-bindgen/

**wasm-pack**:
- https://rustwasm.github.io/wasm-pack/

**Existing Docs**:
- `docs/visualization/BEVY_3D_WEB.md` - Complete setup guide
- `web/BUILD_INSTRUCTIONS.md` - Build instructions
- `src/visualization/bevy_3d.rs` - Implementation code

---

## ✅ Summary

**Status**: Bevy 3D visualization **already implemented** ✅

**What's Done**:
- ✅ 3D scene with sacred geometry
- ✅ Vortex flow visualization
- ✅ ELP color coding
- ✅ Camera controls
- ✅ WASM compilation support
- ✅ Svelte integration documented

**What's Needed** (Week 9-10):
- Connect to HLE inference WebSocket
- Add real-time data updates
- Create Svelte wrapper component
- Add confidence visualization
- Add reasoning path highlighting
- Polish and test

**Estimated Effort**: 2-3 days for full HLE integration

**The foundation is solid - we just need to connect it to HLE data!** 🎨🚀
