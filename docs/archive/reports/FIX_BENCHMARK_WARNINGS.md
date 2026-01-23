# Fix Benchmark Warnings

The benchmark file `benchmarks/custom/geometric_reasoning_benchmark.rs` has 2 warnings that need fixing.

---

## ✅ Issue 1: Library Test Already Fixed

The f32/f64 type mismatch in `src/compression/asi_12byte.rs` is **already fixed**:

```rust
// ✅ CORRECT (line 353):
let concepts: Vec<(String, u8, ELPTensor, f64)> = (0..100)
```

Run `cargo check --lib` to verify - it should pass now.

---

## ⚠️ Issue 2: Benchmark Warnings

### Location
`benchmarks/custom/geometric_reasoning_benchmark.rs`

### Warning 1: Unused Import (Line 15)
```rust
// ⚠️ CURRENT:
use spatial_vortex::flux_matrix::FluxMatrixEngine;
```

### Warning 2: Unused Variable (Line 420)
```rust
// ⚠️ CURRENT:
if let Some(gold_pos) = task.gold_position {
    // gold_pos is not used!
}
```

---

## 🔧 Quick Fix Option (Suppress Warnings)

If you want to keep the code as-is but silence the warnings:

```rust
// Line 15: Allow unused import
#[allow(unused_imports)]
use spatial_vortex::flux_matrix::FluxMatrixEngine;

// Line 420: Prefix with underscore
if let Some(_gold_pos) = task.gold_position {
    // Indicates intentionally unused
}
```

---

## ✨ Recommended Fix (Use the Features)

Implement the FluxMatrixEngine properly as documented:

### Step 1: Add to imports (keep line 15)
```rust
use spatial_vortex::flux_matrix::FluxMatrixEngine;
use spatial_vortex::visualization::FluxLayout;
```

### Step 2: Replace line 420 code
```rust
// ✅ NEW: Actually use gold_pos with FluxMatrixEngine
if let Some(gold_pos) = task.gold_position {
    let engine = FluxMatrixEngine::new();
    let layout = FluxLayout::sacred_geometry_layout();
    
    // Calculate flux distance
    let flux_distance = engine.reduce_digits(
        (predicted_position as i16 - gold_pos as i16).abs() as u64
    ) as u8;
    
    // Check sacred positions
    let pred_is_sacred = engine.sacred_positions.contains(&predicted_position);
    let gold_is_sacred = engine.sacred_positions.contains(&gold_pos);
    
    // Print results
    println!("Predicted: {} (sacred: {}) | Gold: {} (sacred: {})",
        predicted_position, pred_is_sacred, gold_pos, gold_is_sacred);
    println!("Flux Distance: {}", flux_distance);
    
    // Calculate spatial accuracy
    let gold_coords = layout.positions.get(&gold_pos).copied();
    if let Some(gold_c) = gold_coords {
        let pred_coords = layout.positions.get(&predicted_position)
            .copied()
            .unwrap_or(layout.center);
        let spatial_error = pred_coords.distance_to(&gold_c);
        println!("Spatial Error: {:.2} units", spatial_error);
    }
}
```

---

## 📊 For Full Implementation (Option 1)

See the complete implementation in:
- `FLUX_ENGINE_BENCHMARK_IMPLEMENTATION.md`
- `FLUX_MATRIX_ENGINE_GEOMETRIC_REASONING.md`
- `IMPLEMENTATION_GUIDE_BENCHMARK.md`

These provide the full flow-aware evaluation code.

---

## 🚀 Quick Commands

### Check if library is fixed
```bash
cargo check --lib
# Should show: 0 errors, 0 warnings
```

### Check benchmark status
```bash
cargo check --bins
# Will show the 2 warnings above
```

### Run benchmark after fixing
```bash
cargo build --release --bin geometric_reasoning_benchmark
.\target\release\geometric_reasoning_benchmark.exe
```

---

## 🎯 Summary

| Issue | Status | Action Needed |
|-------|--------|---------------|
| **f32/f64 type error** | ✅ Fixed | None - already corrected |
| **Unused FluxMatrixEngine** | ⚠️ Warning | Add 5-10 lines of code at line 420 |
| **Unused gold_pos** | ⚠️ Warning | Will be fixed by using FluxMatrixEngine |

**Recommended**: Implement the "Use the Features" fix above (takes 2 minutes).

---

## 📝 Minimal Fix (Copy-Paste)

Replace the code around line 420 with this:

```rust
if let Some(gold_pos) = task.gold_position {
    let engine = FluxMatrixEngine::new();
    
    // Calculate flux distance using digit reduction
    let flux_dist = engine.reduce_digits(
        (predicted_position as i16 - gold_pos as i16).abs() as u64
    );
    
    // Check if sacred positions
    let pred_sacred = engine.sacred_positions.contains(&predicted_position);
    let gold_sacred = engine.sacred_positions.contains(&gold_pos);
    
    println!("Predicted: {} (sacred: {}) | Gold: {} (sacred: {}) | Flux Distance: {}",
        predicted_position, pred_sacred, gold_pos, gold_sacred, flux_dist);
}
```

This uses both `FluxMatrixEngine` and `gold_pos`, eliminating both warnings! ✅
