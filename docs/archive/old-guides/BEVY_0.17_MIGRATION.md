# Bevy 0.17 Migration Guide
## API Breaking Changes and Fixes

**Issue**: Code written for Bevy 0.13/0.14 fails to compile with Bevy 0.17-dev  
**Impact**: 99 compilation errors across visualization modules  
**Time to Fix**: 2-3 hours for complete migration

---

## 🔧 Critical API Changes

### 1. Entity Spawning (35 errors)
```rust
// OLD (Bevy 0.13)
commands.spawn()
    .insert_bundle(PbrBundle { ... });

// NEW (Bevy 0.17)
commands.spawn(PbrBundle { ... });
```

### 2. Color API (20 errors)
```rust
// OLD
Color::rgb(1.0, 0.0, 0.0)
Color::rgba(1.0, 0.0, 0.0, 0.5)
Color::RED
Color::GREEN
Color::CYAN

// NEW
Color::srgb(1.0, 0.0, 0.0)
Color::srgba(1.0, 0.0, 0.0, 0.5)
Color::srgb(1.0, 0.0, 0.0)  // RED
Color::srgb(0.0, 1.0, 0.0)  // GREEN
Color::srgb(0.0, 1.0, 1.0)  // CYAN
```

### 3. Time API (8 errors)
```rust
// OLD
time.delta_seconds()
time.seconds_since_startup()

// NEW
time.delta_secs()
time.elapsed_secs()
```

### 4. System Registration (3 errors)
```rust
// OLD
app.add_startup_system(setup)
   .add_system(update)

// NEW
app.add_systems(Startup, setup)
   .add_systems(Update, update)
```

### 5. Resource Trait (2 errors)
```rust
// OLD
pub struct AmbientLight { ... }
pub struct BeamRenderConfig { ... }

// NEW
#[derive(Resource)]
pub struct AmbientLight {
    affects_lightmapped_meshes: bool,  // NEW field
    ...
}

#[derive(Resource)]
pub struct BeamRenderConfig { ... }
```

### 6. Mesh API (2 errors)
```rust
// OLD
Mesh::new(PrimitiveTopology::LineList)
mesh.set_indices(...)

// NEW
Mesh::new(
    PrimitiveTopology::LineList,
    RenderAssetUsages::default()
)
mesh.insert_indices(...)
```

### 7. Color Operations (5 errors)
```rust
// OLD
let emissive = color * 0.5;
let r = color.r();

// NEW
let emissive = color.to_linear() * 0.5;  // Convert to LinearRgba
let Srgba { red, green, blue, alpha } = color.to_srgba();
```

### 8. Query with Handle<T> (3 errors)
```rust
// OLD
Query<(&Block, &Handle<StandardMaterial>)>

// NEW  
Query<(&Block, &MeshMaterial3d<StandardMaterial>)>
// OR remove Handle from query entirely
```

---

## 📝 File-by-File Fixes

### Fix 1: `src/models.rs`
```rust
// Add Default derive to BeamTensor
#[derive(Clone, Debug, Serialize, Deserialize, Default)]  // Add Default
pub struct BeamTensor {
    // ... fields
}
```

### Fix 2: `src/visualization/bevy_3d.rs`
```rust
use bevy::prelude::*;
use bevy::render::mesh::PrimitiveTopology;
use bevy::render::render_asset::RenderAssetUsages;

// Fix spawn calls (lines 177, 209, 226, 301, 319)
commands.spawn(DirectionalLightBundle {
    directional_light: DirectionalLight {
        illuminance: 20000.0,
        shadows_enabled: true,
        ..default()
    },
    ..default()
});

// Fix AmbientLight (line 190)
#[derive(Resource)]  // Add to struct definition
pub struct AmbientLightConfig {
    pub color: Color,
    pub brightness: f32,
    pub affects_lightmapped_meshes: bool,
}

commands.insert_resource(AmbientLightConfig {
    color: Color::WHITE,
    brightness: 0.3,
    affects_lightmapped_meshes: false,
});

// Fix Color::rgb calls (lines 204, 206, etc.)
Color::srgb(1.0, 0.0, 0.0)  // Was Color::rgb
Color::srgb(0.3, 0.3, 0.8)

// Fix emissive color multiplication (line 231)
emissive: Color::WHITE.to_linear(),  // Or Color::srgb(1.0, 1.0, 1.0).into()

// Fix time (line 378)
orbit.angle += time.delta_secs() * 0.3;  // Was delta_seconds

// Fix system registration (line 396)
app.add_systems(Startup, setup_flux_3d);  // Was add_startup_system
```

### Fix 3: `src/visualization/bevy_shapes.rs`
```rust
// Fix color multiplication (lines 204, 234)
emissive: color.to_linear() * 0.2,  // Convert to LinearRgba first

// Fix queries with Handle (lines 251, 263, 276)
// Option A: Remove Handle from query
Query<&ProcessingBlock, With<MeshMaterial3d<StandardMaterial>>>

// Option B: Use MeshMaterial3d
Query<(&ProcessingBlock, &MeshMaterial3d<StandardMaterial>)>
```

### Fix 4: `src/flux_mesh.rs`
```rust
use bevy::render::render_asset::RenderAssetUsages;

// Fix Mesh::new (line 97)
let mut mesh = Mesh::new(
    PrimitiveTopology::LineList,
    RenderAssetUsages::default()
);

// Fix set_indices (line 113)
mesh.insert_indices(Indices::U32(indices));  // Was set_indices

// Fix ico resolution (line 120)
Sphere::new(radius).mesh().ico(resolution).unwrap()  // Already u32, remove cast

// Fix color multiplication (line 262)
emissive: node_color.to_linear() * 2.0,

// Fix time methods (lines 295, 300)
let pulse = (time.elapsed_secs() * 2.0).sin() * 0.5 + 0.5;  // Was seconds_since_startup
transform.rotate_y(time.delta_secs() * 0.5);  // Was delta_seconds
```

### Fix 5: `src/beam_renderer.rs`
```rust
// Fix BeamRenderConfig Resource (line 28)
#[derive(Resource)]  // Add this derive
pub struct BeamRenderConfig {
    pub beam_speed: f32,  // Make sure field exists
    pub show_trails: bool,
}

// Fix time methods (lines 151, 181, 292, 309)
beam.progress += time.delta_secs() * config.beam_speed / beam.path.len() as f32;
let wobble = (time.elapsed_secs() * 5.0).sin() * beam.intensity * 0.1;

// Fix Color constants (lines 216, 227, 238)
Color::srgb(0.0, 1.0, 0.0)  // GREEN
Color::srgb(1.0, 0.0, 0.0)  // RED
Color::srgb(0.0, 1.0, 1.0)  // CYAN

// Fix Color::rgba and component access (line 261)
let Srgba { red, green, blue, alpha } = color.to_srgba();
let transparent_color = Color::srgba(red, green, blue, 0.5);

// Fix system registration (line 337)
app.add_systems(Update, (
    update_beam_flow,
    update_beam_positions,
    update_beam_effects,
));  // Was add_system
```

---

## 🚀 Quick Fix Script

Create `scripts/migrate_bevy_017.ps1`:

```powershell
# Automated migration for common patterns

$files = Get-ChildItem -Path "src" -Filter "*.rs" -Recurse

foreach ($file in $files) {
    $content = Get-Content $file.FullName -Raw
    
    # Fix Color API
    $content = $content -replace 'Color::rgb\(', 'Color::srgb('
    $content = $content -replace 'Color::rgba\(', 'Color::srgba('
    $content = $content -replace 'Color::RED', 'Color::srgb(1.0, 0.0, 0.0)'
    $content = $content -replace 'Color::GREEN', 'Color::srgb(0.0, 1.0, 0.0)'
    $content = $content -replace 'Color::BLUE', 'Color::srgb(0.0, 0.0, 1.0)'
    $content = $content -replace 'Color::CYAN', 'Color::srgb(0.0, 1.0, 1.0)'
    $content = $content -replace 'Color::WHITE', 'Color::srgb(1.0, 1.0, 1.0)'
    
    # Fix time API
    $content = $content -replace '\.delta_seconds\(\)', '.delta_secs()'
    $content = $content -replace '\.seconds_since_startup\(\)', '.elapsed_secs()'
    
    # Fix mesh indices
    $content = $content -replace '\.set_indices\(', '.insert_indices('
    
    Set-Content -Path $file.FullName -Value $content
}

Write-Host "Automated migration complete. Manual fixes still needed for:"
Write-Host "1. spawn().insert_bundle() → spawn(bundle)"
Write-Host "2. add_startup_system/add_system → add_systems"
Write-Host "3. Color multiplication (needs .to_linear())"
Write-Host "4. Handle<T> in queries"
Write-Host "5. Resource derives"
```

---

## ✅ Manual Fix Checklist

### High Priority (Blocks Compilation)
- [ ] Add `#[derive(Default)]` to `BeamTensor` in `models.rs`
- [ ] Add `#[derive(Resource)]` to `BeamRenderConfig` in `beam_renderer.rs`
- [ ] Fix all `commands.spawn()` calls (35 locations)
- [ ] Replace all `Color::rgb/rgba` with `Color::srgb/srgba` (20 locations)
- [ ] Fix all `time.delta_seconds()` → `time.delta_secs()` (8 locations)
- [ ] Fix `add_startup_system` → `add_systems(Startup, ...)` (3 locations)
- [ ] Fix `Mesh::new()` to include `RenderAssetUsages` (1 location)

### Medium Priority (API Compatibility)
- [ ] Fix color multiplication with `.to_linear()` (5 locations)
- [ ] Fix `set_indices` → `insert_indices` (1 location)
- [ ] Fix Query with `Handle<T>` (3 locations)
- [ ] Add `affects_lightmapped_meshes` to AmbientLight configs
- [ ] Fix `seconds_since_startup()` → `elapsed_secs()` (2 locations)

### Low Priority (Warnings)
- [ ] Update deprecated `Color` constants to explicit RGB
- [ ] Review and update any custom color operations
- [ ] Check for other deprecated APIs in warnings

---

## 📚 References

**Bevy 0.17 Migration Guide**: https://bevyengine.org/learn/migration-guides/0-16-to-0-17/

**Key Changes**:
- Color system overhaul (color spaces)
- Entity spawning simplified
- Resource trait now required
- Time API simplified
- System registration unified

---

## ⏱️ Estimated Time

**Automated Script**: 10 minutes  
**Manual spawn() fixes**: 45 minutes (35 locations)  
**Color/Time API fixes**: 30 minutes  
**System registration**: 15 minutes  
**Testing**: 30 minutes  

**Total**: 2-3 hours for complete migration

---

## 🧪 Testing After Migration

```bash
# 1. Clean build
cargo clean

# 2. Update dependencies
cargo update

# 3. Build
cargo build --features bevy_support

# 4. Run tests
cargo test --features bevy_support

# 5. Run examples
cargo run --example epic_flux_3d_native --features bevy_support --release
```

---

## 💡 Prevention

Add to `Cargo.toml`:
```toml
[dependencies]
bevy = "=0.16.0"  # Pin to stable version until ready for 0.17
```

Or create feature flag:
```toml
[features]
bevy_0_17 = ["bevy/0.17"]
bevy_support = []  # Use 0.16 by default
```

---

**Status**: Migration guide complete, ready for implementation  
**Priority**: P1 - Blocks all Bevy visualization features  
**Next**: Run automated script, then manual fixes  

