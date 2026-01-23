# Bevy WASM Consolidation Plan

## 🚨 Problem Identified

**We have 3 different Bevy WASM entry points:**

| File | Lines | Features | Status |
|------|-------|----------|--------|
| `src/epic_wasm.rs` | 336 | Clean, simple, static camera | ⚠️ Duplicate |
| `src/bin/epic_flux_3d_wasm.rs` | 778 | **COMPREHENSIVE** - all features | ✅ Keep as base |
| `wasm/flux_3d_web.rs` | 224 | Basic geometry only | ⚠️ Duplicate |

**Result**: Confusing, hard to maintain, wasted effort

---

## ✅ Proposed Solution

**Consolidate into ONE canonical WASM entry point**: `src/epic_wasm.rs`

**Why `src/epic_wasm.rs`?**
- ✅ Clean structure
- ✅ In main `src/` directory (not `bin/` or `wasm/`)
- ✅ Already has initialization guard (`INITIALIZED` atomic)
- ✅ Good error handling
- ✅ Console logging helpers
- ✅ Can be imported by both standalone WASM and HLE frontend

---

## 📋 Consolidation Checklist

### **Phase 1: Merge Features** ✅

**From `src/bin/epic_flux_3d_wasm.rs` (778 lines) → INTO `src/epic_wasm.rs`**:

1. **Add Components**:
   - ✅ `WordBeam` - Flowing beams through matrix
   - ✅ `ProcessingBlock` - Box shapes for processing units
   - ✅ `DatabaseNode` - Cylinder shapes for databases
   - ✅ `IntersectionEffect` - Sacred position effects
   - ✅ `BeamTrail` - Trail rendering for beams
   - ✅ `OrbitCamera` - Auto-rotating camera

2. **Add Resources**:
   - ✅ `VisualizationConfig` - Runtime configuration
     - `auto_rotate: bool`
     - `rotation_speed: f32`
     - `beam_speed: f32`
     - `show_trails: bool`
     - `camera_distance: f32`

3. **Add Systems**:
   - ✅ `spawn_word_beams` - Periodic beam spawning
   - ✅ `update_word_beams` - Beam animation
   - ✅ `process_sacred_intersections` - Trigger effects at 3-6-9
   - ✅ `animate_intersection_effects` - Effect animations
   - ✅ `spawn_processing_blocks` - Processing units
   - ✅ `spawn_database_nodes` - Database cylinders
   - ✅ `update_processing_blocks` - Block pulsing
   - ✅ `rotate_camera` - Orbit camera system

4. **Add Effect Types**:
   - ✅ `GreenBurst` - Position 3 (ethos)
   - ✅ `RedRipple` - Position 6 (pathos)
   - ✅ `BlueAscension` - Position 9 (logos)

**From `wasm/flux_3d_web.rs` (224 lines)**:
- Nothing unique - simpler version of what we already have

---

### **Phase 2: Create WASM Bindings** 🔧

**Add HLE-specific exports to `src/epic_wasm.rs`**:

```rust
/// Update scene with HLE inference data
#[wasm_bindgen]
pub fn update_inference_result(
    position: u8,
    confidence: f32,
    elp_ethos: f32,
    elp_logos: f32,
    elp_pathos: f32,
) {
    // Update FluxNode at position
    // Trigger animation based on confidence
    // Update ELP visualization
}

/// Highlight reasoning path
#[wasm_bindgen]
pub fn highlight_reasoning_path(positions: Vec<u8>) {
    // Draw animated path through positions
    // Emphasize sacred positions if included
}

/// Set camera mode
#[wasm_bindgen]
pub fn set_camera_mode(auto_rotate: bool, distance: f32) {
    // Update VisualizationConfig
}

/// Reset scene
#[wasm_bindgen]
pub fn reset_scene() {
    // Clear all dynamic elements
    // Reset to default state
}

/// Get current state as JSON
#[wasm_bindgen]
pub fn get_scene_state() -> String {
    // Return JSON with all node states
}
```

---

### **Phase 3: Remove Duplicates** 🗑️

1. **Delete** `src/bin/epic_flux_3d_wasm.rs` (after merging features)
2. **Delete** `wasm/flux_3d_web.rs` (after verifying no unique code)
3. **Update** `Cargo.toml` to remove bin target:
   ```toml
   # REMOVE THIS:
   [[bin]]
   name = "epic_flux_3d_wasm"
   path = "src/bin/epic_flux_3d_wasm.rs"
   required-features = ["bevy_support"]
   ```

4. **Update** documentation references:
   - `docs/visualization/BEVY_3D_WEB.md`
   - `web/BUILD_INSTRUCTIONS.md`
   - `docs/implementation/BEVY_WASM_INTEGRATION.md`

---

### **Phase 4: Update Build Process** 🔨

**New Single Build Command**:

```bash
# Build consolidated WASM module
wasm-pack build \
  --target web \
  --out-dir web/src/wasm \
  --features bevy_support \
  --lib \
  -- --no-default-features
```

**Key Changes**:
- Use `--lib` (not a bin target)
- Builds from `src/epic_wasm.rs` (via `src/lib.rs`)
- Outputs to `web/src/wasm/spatial_vortex.js`

---

### **Phase 5: Svelte Integration** 🎨

**Update** `web/src/lib/components/hle/FluxVisualizer.svelte`:

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import init, { 
    epic_flux_3d_init,
    update_inference_result,
    highlight_reasoning_path,
    set_camera_mode 
  } from '$wasm/spatial_vortex';

  export let inferenceData: InferenceData | null = null;
  export let autoRotate = true;
  export let cameraDistance = 25.0;

  let wasmReady = false;

  onMount(async () => {
    await init();  // Initialize WASM module
    epic_flux_3d_init();  // Start Bevy app
    wasmReady = true;
  });

  $: if (wasmReady && inferenceData) {
    update_inference_result(
      inferenceData.position,
      inferenceData.confidence,
      inferenceData.elp.ethos,
      inferenceData.elp.logos,
      inferenceData.elp.pathos
    );
  }

  $: if (wasmReady) {
    set_camera_mode(autoRotate, cameraDistance);
  }
</script>

<div class="visualizer">
  <canvas id="bevy-canvas"></canvas>
</div>
```

---

## 🎯 Benefits of Consolidation

### **1. Single Source of Truth**
- ✅ One file to maintain (`src/epic_wasm.rs`)
- ✅ No confusion about which version to use
- ✅ Easier to add new features

### **2. Better Code Reuse**
- ✅ Share code between standalone WASM and HLE frontend
- ✅ Consistent behavior across use cases
- ✅ Easier testing

### **3. Smaller Bundle Size**
- ✅ Remove duplicate code
- ✅ Better tree-shaking
- ✅ Faster load times

### **4. HLE Integration Ready**
- ✅ Clean exports for Svelte
- ✅ Real-time updates via WASM bindings
- ✅ Camera controls
- ✅ State management

---

## 📊 Feature Matrix (After Consolidation)

| Feature | Status | Source File |
|---------|--------|-------------|
| Sacred Triangle (3-6-9) | ✅ | epic_wasm.rs |
| Vortex Flow (1→2→4→8→7→5→1) | ✅ | epic_wasm.rs |
| ELP Color Coding | ✅ | epic_wasm.rs |
| Auto-Rotating Camera | ✅ | epic_flux_3d_wasm.rs → epic_wasm.rs |
| Word Beams | ✅ | epic_flux_3d_wasm.rs → epic_wasm.rs |
| Processing Blocks | ✅ | epic_flux_3d_wasm.rs → epic_wasm.rs |
| Database Nodes | ✅ | epic_flux_3d_wasm.rs → epic_wasm.rs |
| Sacred Intersection Effects | ✅ | epic_flux_3d_wasm.rs → epic_wasm.rs |
| Beam Trails | ✅ | epic_flux_3d_wasm.rs → epic_wasm.rs |
| Real-time Updates | 🔧 | New WASM bindings |
| Reasoning Path Highlight | 🔧 | New WASM bindings |
| Confidence Visualization | 🔧 | New WASM bindings |
| HLE Data Integration | 🔧 | New WASM bindings |

---

## 🚀 Implementation Steps

### **Week 9 Tasks**:

**Day 1-2: Merge Features**
- [ ] Copy all components from `epic_flux_3d_wasm.rs` → `epic_wasm.rs`
- [ ] Copy all systems
- [ ] Copy all helper functions
- [ ] Test compilation

**Day 3: Add WASM Bindings**
- [ ] Add `update_inference_result()`
- [ ] Add `highlight_reasoning_path()`
- [ ] Add `set_camera_mode()`
- [ ] Add `reset_scene()`
- [ ] Add `get_scene_state()`

**Day 4: Remove Duplicates**
- [ ] Delete `src/bin/epic_flux_3d_wasm.rs`
- [ ] Delete `wasm/flux_3d_web.rs`
- [ ] Update `Cargo.toml`
- [ ] Update documentation

**Day 5: Integration Testing**
- [ ] Build WASM module
- [ ] Test in Svelte
- [ ] Verify all features work
- [ ] Performance testing

---

## 📝 Updated File Structure (After Consolidation)

```
src/
├── epic_wasm.rs                    # ✅ SINGLE WASM ENTRY POINT (consolidated)
│   └── All features merged:
│       - Sacred geometry
│       - Vortex flow
│       - Word beams
│       - Processing blocks
│       - Database nodes
│       - Sacred effects
│       - Camera systems
│       - HLE bindings
│
├── visualization/
│   └── bevy_3d.rs                  # Shared Bevy components (if needed)
│
└── bin/
    ├── flux_matrix.rs              # CLI demos (non-WASM)
    ├── vortex_view.rs
    └── (epic_flux_3d_wasm.rs DELETED)

wasm/
└── (flux_3d_web.rs DELETED)

web/src/wasm/
├── spatial_vortex.js               # Generated by wasm-pack
├── spatial_vortex_bg.wasm          # Generated by wasm-pack
└── spatial_vortex.d.ts             # TypeScript definitions
```

---

## ⚠️ Migration Notes

### **Breaking Changes**:
1. WASM module name changes from `flux_3d_web` → `spatial_vortex`
2. Build command changes (use `--lib` not `--bin`)
3. Import path in Svelte changes

### **Backward Compatibility**:
- Keep old build for 1-2 releases (deprecated)
- Add migration guide in docs
- Update all examples

---

## ✅ Success Criteria

**Consolidation is complete when**:
1. ✅ Only ONE WASM entry point exists (`src/epic_wasm.rs`)
2. ✅ All features from all 3 files are merged
3. ✅ HLE-specific bindings added
4. ✅ Duplicate files deleted
5. ✅ Build process simplified
6. ✅ Documentation updated
7. ✅ Svelte integration works
8. ✅ All tests pass
9. ✅ Performance equal or better
10. ✅ Bundle size smaller

---

## 📚 Related Documentation

**Update After Consolidation**:
- `docs/implementation/BEVY_WASM_INTEGRATION.md` - Update build commands
- `docs/visualization/BEVY_3D_WEB.md` - Update file references
- `web/BUILD_INSTRUCTIONS.md` - Update wasm-pack commands
- `README.md` - Update examples

---

**Status**: 📋 PLAN READY  
**Effort**: ~2-3 days  
**Priority**: HIGH (blocks HLE frontend integration)  
**Owner**: Week 9, Day 1-3  

**Let's consolidate and create ONE amazing Bevy WASM visualization!** 🌀🚀
