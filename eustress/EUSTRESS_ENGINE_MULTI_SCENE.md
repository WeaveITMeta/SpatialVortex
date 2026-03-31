# EustressEngine: Multi-Scene Universe Browsing

## Context

The ARC-AGI-3 agent (`eustress-arc-agent`) now writes game state as EEP-compliant TOML Spaces into a standard Eustress Universe at:

```
Documents/Eustress/ARC-AGI-3/          <- Universe
├── knowledge/                          <- Cross-game persistent learning
│   └── vortex_knowledge.json
└── spaces/
    ├── game_abc123/                    <- Space per game (created on game start)
    │   ├── .eustress/project.toml
    │   ├── space.toml
    │   ├── Workspace/
    │   │   ├── _service.toml
    │   │   └── Grid.part.toml          <- Grid cells array (GridSceneLoaderPlugin reads this)
    │   ├── ServerStorage/
    │   │   ├── causal_graph.toml
    │   │   ├── action_models.toml
    │   │   └── archetypes.toml
    │   └── SoulService/
    │       └── action_rules.soul
    ├── game_def456/                    <- Another game (CRUD: deleted on game over)
    └── game_ghi789/
```

Each game creates a Space on start and deletes it on game over. Multiple games can exist simultaneously. The engine needs to browse and switch between them without restarting.

## What Already Works

- `open_space(world, &path)` in `space_ops.rs` fully handles Space switching (despawn, clear registry, update SpaceRoot, rescan)
- `SpaceFileLoaderPlugin` hot-reloads file changes within the current Space
- `GridSceneLoaderPlugin` (new, in `grid_scene_loader.rs`) scans `Documents/Eustress/ARC-AGI-3/spaces/` for `Grid.part.toml` files and renders colored quads with hot-reload
- `new_universe()` in `space_ops.rs` scaffolds Universe directories
- File event pipeline: Slint UI -> SlintAction -> FileAction -> `open_space()`

## What Needs to Change in EustressEngine

### 1. Add `--space` and `--universe` CLI flags

**File**: `crates/engine/src/startup.rs`

```rust
pub struct StartupArgs {
    // ... existing fields ...
    /// Open a specific Space directory at startup
    pub space_dir: Option<PathBuf>,
    /// Open a specific Universe directory at startup (loads first Space)
    pub universe_dir: Option<PathBuf>,
}
```

In `StartupPlugin::build()`, if `space_dir` is set, override `SpaceRoot` before `SpaceFileLoaderPlugin` runs:

```rust
if let Some(ref space_dir) = args.space_dir {
    app.insert_resource(SpaceRoot(space_dir.clone()));
} else if let Some(ref universe_dir) = args.universe_dir {
    if let Some(space) = first_space_root_in_universe(universe_dir) {
        app.insert_resource(SpaceRoot(space));
    }
}
```

### 2. Create `UniverseRegistry` resource

**New file**: `crates/engine/src/space/universe_registry.rs`

```rust
use bevy::prelude::*;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Resource, Default, Debug, Clone)]
pub struct UniverseRegistry {
    pub universes: Vec<UniverseInfo>,
    pub current_universe: Option<String>,
    pub current_space: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UniverseInfo {
    pub path: PathBuf,
    pub name: String,
    pub spaces: Vec<SpaceInfo>,
}

#[derive(Debug, Clone)]
pub struct SpaceInfo {
    pub path: PathBuf,
    pub name: String,
    pub modified: SystemTime,
    pub has_grid: bool,  // Quick check: does Workspace/Grid.part.toml exist?
}
```

Add a startup system that scans `Documents/Eustress/` and an Update system that re-scans every ~5 seconds (debounced) to detect new Spaces created by external agents.

### 3. Expose Universe Browser in Slint UI

**Sidebar panel** (Explorer tree or dedicated panel) showing:

```
Universes
├── ARC-AGI-3                          <- Universe name
│   ├── game_abc123  [ACTIVE]          <- Currently loaded Space
│   ├── game_def456                    <- Click to switch
│   └── game_ghi789
├── My Project                          <- Other universes
│   └── Main
└── City Builder
    ├── Downtown
    └── Suburbs
```

**Interactions**:
- Click Space name -> `open_space(world, &space_info.path)` (infrastructure exists)
- Right-click -> "Delete Space", "Rename Space"
- Double-click Universe -> expand/collapse
- "+" button on Universe row -> `scaffold_new_space()` (infrastructure exists)
- "+" button at top -> `new_universe()` (infrastructure exists)

**Slint component** (add to sidebar):

```slint
export component UniverseBrowser inherits VerticalLayout {
    in property <[UniverseItem]> universes;
    in property <string> active-space-path;

    callback space-selected(/* space_path: */ string);
    callback space-deleted(/* space_path: */ string);
    callback new-space-requested(/* universe_path: */ string);

    for universe in universes: VerticalLayout {
        Text { text: universe.name; font-weight: 700; }
        for space in universe.spaces: TouchArea {
            clicked => { root.space-selected(space.path); }
            Rectangle {
                background: space.path == active-space-path ? #2a4a7f : transparent;
                Text { text: space.name; }
            }
        }
    }
}
```

### 4. Update window title to show Universe > Space

**File**: `crates/engine/src/main.rs` (window title generation) and add an Update system:

```rust
fn update_window_title(
    space_root: Res<SpaceRoot>,
    mut windows: Query<&mut Window>,
) {
    if !space_root.is_changed() { return; }
    let Ok(mut window) = windows.single_mut() else { return };

    let space_name = space_root.0.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Untitled".into());

    let universe_name = space_root.0.parent()  // spaces/
        .and_then(|p| p.parent())               // Universe/
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Unknown".into());

    window.title = format!("{} > {} - Eustress Engine", universe_name, space_name);
}
```

### 5. Watch for Space CRUD from external agents

The ARC agent creates and deletes Spaces while the engine is running. The `UniverseRegistry` scan system handles discovery, but for instant reactivity, add a `notify` file watcher on `Documents/Eustress/*/spaces/`:

```rust
fn watch_universe_spaces(
    registry: Res<UniverseRegistry>,
    mut events: EventReader<FileChangeEvent>,
) {
    for event in events.read() {
        if event.path.ends_with("space.toml") || event.path.ends_with("project.toml") {
            // A new Space was created or deleted — trigger registry rescan
            // The UniverseBrowser UI will update automatically via Slint data binding
        }
    }
}
```

### 6. GridSceneLoaderPlugin integration (already done)

The `GridSceneLoaderPlugin` in `grid_scene_loader.rs`:
- Scans `Documents/Eustress/ARC-AGI-3/spaces/` for `Grid.part.toml` files
- Hot-reloads every ~1 second when files change
- Renders cells as colored `Cuboid` quads on the XZ plane
- Uses the standard ARC 16-color palette
- Parent entity gets `ArcGrid` component; children get `GridCell` components

When the user switches Spaces via `open_space()`, the grid watcher will detect the new Grid.part.toml in the active Space and render it automatically.

## File Layout After Implementation

```
crates/engine/src/
├── space/
│   ├── mod.rs                  <- Add: pub mod universe_registry; export UniverseRegistry
│   ├── universe_registry.rs    <- NEW: scan + cache + watch universes/spaces
│   ├── space_ops.rs            <- Existing: open_space(), new_universe(), scaffold_new_space()
│   └── file_loader.rs          <- Already updated: Grid FileType recognized
├── grid_scene_loader.rs        <- Already done: ARC grid visualization
├── startup.rs                  <- Update: --space and --universe CLI flags
├── main.rs                     <- Update: register update_window_title system
└── ui/
    ├── slint_ui.rs             <- Update: wire UniverseBrowser callbacks
    └── file_event_handler.rs   <- Existing: FileAction::OpenSpace pipeline works
```

## Priority Order

1. **`--space` / `--universe` CLI flags** — enables `eustress-engine --universe "Documents/Eustress/ARC-AGI-3"` to open the ARC Universe directly
2. **`UniverseRegistry` + periodic scan** — discovers Spaces created by the running ARC agent
3. **Window title update** — visual feedback for which Universe/Space is active
4. **Slint UniverseBrowser sidebar** — full interactive browsing
5. **`notify` watcher on universes** — instant reactivity when Spaces are created/deleted
