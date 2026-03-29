//! # Space Writer — EustressEngine Explorer Integration
//!
//! Serializes the ARC game internal model to EEP-compliant TOML files
//! that can be opened in EustressEngine Explorer.
//!
//! ## File System Layout
//!
//! ```text
//! ARC-AGI-3/                              ← Universe
//! └── spaces/
//!     └── game_{id}/                      ← Space per game
//!         ├── .eustress/
//!         │   ├── project.toml            ← ProjectManifest
//!         │   └── current.toml            ← Hot state (materializer target)
//!         ├── space.toml                  ← Game metadata
//!         ├── Workspace/
//!         │   ├── _service.toml
//!         │   └── Grid.part.toml          ← Grid state as colored quads
//!         ├── ServerStorage/
//!         │   ├── _service.toml
//!         │   ├── causal_graph.toml       ← CausalGraph nodes + edges
//!         │   ├── action_models.toml      ← Per-action learned effects
//!         │   └── archetypes.toml         ← Cross-game symbolic archetypes
//!         └── SoulService/
//!             ├── _service.toml
//!             └── action_rules.soul       ← Learned rules as SoulScript
//! ```
//!
//! ## MCP Binary Protocol
//!
//! For real-time streaming, the same data can be published via Iggy
//! to the `eustress/scene_deltas` topic. The TOML materializer in
//! EustressEngine will then write the hot `current.toml` file.

use eustress_vortex_core::CausalGraph;
use eustress_vortex_grid2d::Grid2D;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::info;

use crate::symbolic_decomposer::SymbolicActionDecomposer;
use crate::world_model::{ActionModel, VortexWorldModel};

/// Configuration for where to write Space files.
pub struct SpaceWriterConfig {
    /// Root directory for the ARC-AGI-3 universe.
    /// Default: `Documents/Eustress/ARC-AGI-3/` (standard Eustress Universe location).
    /// Override with `ARC_UNIVERSE_ROOT` env var.
    pub universe_root: PathBuf,
    /// Whether to write the full grid as individual cell parts (verbose)
    /// or as a single Grid.part.toml with cells array (compact).
    pub per_cell_parts: bool,
}

impl Default for SpaceWriterConfig {
    fn default() -> Self {
        let root = std::env::var("ARC_UNIVERSE_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| eustress_universe_root());
        Self {
            universe_root: root,
            per_cell_parts: false,
        }
    }
}

/// Standard Eustress Universe location: `Documents/Eustress/ARC-AGI-3/`.
/// Falls back to `.eustress-arc/` if Documents directory is unavailable.
pub fn eustress_universe_root() -> PathBuf {
    if let Some(docs) = dirs::document_dir() {
        let root = docs.join("Eustress").join("ARC-AGI-3");
        // Ensure the directory exists
        let _ = std::fs::create_dir_all(&root);
        return root;
    }
    PathBuf::from(".eustress-arc")
}

/// Write the current game state as an EEP-compliant Space.
pub fn write_game_space(
    config: &SpaceWriterConfig,
    game_id: &str,
    grid: &Grid2D,
    model: &VortexWorldModel,
) -> Result<PathBuf, String> {
    let space_dir = config.universe_root.join("spaces").join(format!("game_{}", game_id));

    // Scaffold directories
    let dirs = [
        space_dir.join(".eustress"),
        space_dir.join("Workspace"),
        space_dir.join("ServerStorage"),
        space_dir.join("SoulService"),
    ];
    for dir in &dirs {
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("Failed to create {}: {}", dir.display(), e))?;
    }

    // Write all TOML files
    write_project_manifest(&space_dir, game_id)?;
    write_space_metadata(&space_dir, game_id, grid)?;
    write_workspace_service(&space_dir)?;
    write_grid_part(&space_dir, grid)?;
    write_server_storage_service(&space_dir)?;
    write_causal_graph_toml(&space_dir, &model.causal_graph)?;
    write_action_models_toml(&space_dir, &model.action_models)?;
    write_archetypes_toml(&space_dir, &model.decomposer)?;
    write_soul_service(&space_dir)?;
    write_action_rules_soul(&space_dir, model)?;

    // Write hot scene state via ArcSceneMirror → current.toml
    model.scene_mirror.write_to_space(&space_dir)?;

    info!(
        "Wrote Space for game_{} at {} ({} action models, {} archetypes)",
        game_id,
        space_dir.display(),
        model.action_models.len(),
        model.decomposer.archetype_summary().len(),
    );

    Ok(space_dir)
}

/// Delete a game Space after game over (clean slate per game).
pub fn delete_game_space(config: &SpaceWriterConfig, game_id: &str) -> Result<(), String> {
    let space_dir = config.universe_root.join("spaces").join(format!("game_{}", game_id));
    if space_dir.exists() {
        std::fs::remove_dir_all(&space_dir)
            .map_err(|e| format!("Failed to delete {}: {}", space_dir.display(), e))?;
        info!("Deleted Space for game_{}", game_id);
    }
    Ok(())
}

/// Write shared knowledge directory (survives across games).
pub fn write_knowledge(
    config: &SpaceWriterConfig,
    model: &VortexWorldModel,
) -> Result<(), String> {
    let knowledge_dir = config.universe_root.join("knowledge");
    std::fs::create_dir_all(&knowledge_dir)
        .map_err(|e| format!("Failed to create knowledge dir: {}", e))?;

    // Write knowledge as JSON (compact, fast to parse)
    let json = model.export_knowledge();
    std::fs::write(knowledge_dir.join("vortex_knowledge.json"), &json)
        .map_err(|e| format!("Failed to write knowledge JSON: {}", e))?;

    // Also write human-readable TOML summary
    let summary = knowledge_summary_toml(model);
    std::fs::write(knowledge_dir.join("summary.toml"), &summary)
        .map_err(|e| format!("Failed to write knowledge summary: {}", e))?;

    info!(
        "Wrote knowledge to {} ({} bytes)",
        knowledge_dir.display(), json.len()
    );
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Individual TOML writers
// ─────────────────────────────────────────────────────────────────────────────

fn write_project_manifest(space_dir: &Path, game_id: &str) -> Result<(), String> {
    let content = format!(
        r#"[project]
name = "ARC Game {game_id}"
version = "0.1.0"
author = "eustress-arc-agent"
eep_version = "1.0"
scene_format = "toml"
created = "{now}"
"#,
        now = chrono_now(),
    );
    write_file(&space_dir.join(".eustress/project.toml"), &content)
}

fn write_space_metadata(space_dir: &Path, game_id: &str, grid: &Grid2D) -> Result<(), String> {
    let content = format!(
        r#"[space]
name = "game_{game_id}"
game_id = "{game_id}"
grid_width = {w}
grid_height = {h}
colors = {colors}
last_updated = "{now}"
"#,
        w = grid.width,
        h = grid.height,
        colors = grid.distinct_colors(),
        now = chrono_now(),
    );
    write_file(&space_dir.join("space.toml"), &content)
}

fn write_workspace_service(space_dir: &Path) -> Result<(), String> {
    let content = r#"[service]
class_name = "Workspace"
icon = "workspace"
description = "ARC game grid and entities"
can_have_children = true

[properties]
gravity = 0.0
fallen_parts_destroy_height = -500.0
"#;
    write_file(&space_dir.join("Workspace/_service.toml"), content)
}

fn write_grid_part(space_dir: &Path, grid: &Grid2D) -> Result<(), String> {
    // Serialize grid as a single part with cells data in [Extra.grid]
    let mut cells_str = String::from("[\n");
    for row in &grid.cells {
        cells_str.push_str("  [");
        cells_str.push_str(
            &row.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(", ")
        );
        cells_str.push_str("],\n");
    }
    cells_str.push(']');

    // Map ARC color indices to approximate RGBA
    let bg = grid.background_color();
    let bg_rgba = arc_color_to_rgba(bg);

    let content = format!(
        r#"[metadata]
class_name = "Part"
archivable = true
created = "{now}"
last_modified = "{now}"

[transform]
position = [0.0, 0.0, 0.0]
rotation = [0.0, 0.0, 0.0, 1.0]
scale = [1.0, 1.0, 1.0]

[properties]
color = [{r}, {g}, {b}, 1.0]
anchored = true
can_collide = false
locked = true

[Extra.grid]
width = {w}
height = {h}
background = {bg}
cells = {cells}
"#,
        now = chrono_now(),
        w = grid.width,
        h = grid.height,
        bg = bg,
        r = bg_rgba.0, g = bg_rgba.1, b = bg_rgba.2,
        cells = cells_str,
    );
    write_file(&space_dir.join("Workspace/Grid.part.toml"), &content)
}

fn write_server_storage_service(space_dir: &Path) -> Result<(), String> {
    let content = r#"[service]
class_name = "ServerStorage"
icon = "server_storage"
description = "Learned models, causal graph, and archetypes"
can_have_children = true
"#;
    write_file(&space_dir.join("ServerStorage/_service.toml"), content)
}

fn write_causal_graph_toml(space_dir: &Path, graph: &CausalGraph) -> Result<(), String> {
    let mut content = format!(
        "# CausalGraph — {} nodes, {} edges\n# Auto-generated by eustress-arc-agent\n\n",
        graph.node_count(), graph.edge_count()
    );

    content.push_str("[graph]\n");
    content.push_str(&format!("node_count = {}\n", graph.node_count()));
    content.push_str(&format!("edge_count = {}\n", graph.edge_count()));
    content.push_str(&format!("law_count = {}\n", graph.laws().len()));
    content.push_str(&format!("rule_count = {}\n\n", graph.rules().len()));

    // Write laws
    for (id, node) in graph.laws() {
        content.push_str(&format!("[[laws]]\nname = \"{}\"\n", id));
        if let eustress_vortex_core::causal_graph::CausalNode::Law {
            symbolic_form, confidence, evidence_rules, ..
        } = node {
            content.push_str(&format!("symbolic_form = \"{}\"\n", symbolic_form));
            content.push_str(&format!("confidence = {:.3}\n", confidence));
            content.push_str(&format!("evidence_count = {}\n\n", evidence_rules.len()));
        }
    }

    // Write rules (up to 50)
    for (i, (id, node)) in graph.rules().iter().enumerate() {
        if i >= 50 { break; }
        content.push_str(&format!("[[rules]]\nname = \"{}\"\n", id));
        if let eustress_vortex_core::causal_graph::CausalNode::Rule {
            confidence, evidence_count, ..
        } = node {
            content.push_str(&format!("confidence = {:.3}\n", confidence));
            content.push_str(&format!("evidence_count = {}\n\n", evidence_count));
        }
    }

    write_file(&space_dir.join("ServerStorage/causal_graph.toml"), &content)
}

fn write_action_models_toml(
    space_dir: &Path,
    action_models: &HashMap<u32, ActionModel>,
) -> Result<(), String> {
    let mut content = String::from(
        "# Action Models — learned per-action effects\n# Auto-generated by eustress-arc-agent\n\n"
    );

    let mut ids: Vec<u32> = action_models.keys().copied().collect();
    ids.sort();

    for id in ids {
        let model = &action_models[&id];
        content.push_str(&format!("[[action]]\nid = {}\n", id));
        content.push_str(&format!("observations = {}\n", model.observed_patches.len()));
        content.push_str(&format!("has_effect = {}\n", model.has_effect));
        content.push_str(&format!("is_deterministic = {}\n", model.is_deterministic));
        content.push_str(&format!("ira_score = {:.3}\n", model.ira_score));
        content.push_str(&format!("reliability = {:.3}\n", model.reliability()));
        content.push_str(&format!("predictions = {}\n", model.prediction_count));
        content.push_str(&format!("correct = {}\n\n", model.correct_count));
    }

    write_file(&space_dir.join("ServerStorage/action_models.toml"), &content)
}

fn write_archetypes_toml(
    space_dir: &Path,
    decomposer: &SymbolicActionDecomposer,
) -> Result<(), String> {
    let mut content = String::from(
        "# Symbolic Archetypes — cross-game operation patterns\n# Auto-generated by eustress-arc-agent\n\n"
    );

    let summary = decomposer.archetype_summary();
    content.push_str(&format!("[summary]\ntotal = {}\n", summary.len()));
    content.push_str(&format!(
        "cross_game = {}\n\n",
        summary.iter().filter(|(_, gc, _)| *gc > 1).count()
    ));

    for (formula, game_count, confidence) in &summary {
        content.push_str("[[archetype]]\n");
        content.push_str(&format!("formula = \"{}\"\n", formula));
        content.push_str(&format!("games = {}\n", game_count));
        content.push_str(&format!("confidence = {:.3}\n\n", confidence));
    }

    write_file(&space_dir.join("ServerStorage/archetypes.toml"), &content)
}

fn write_soul_service(space_dir: &Path) -> Result<(), String> {
    let content = r#"[service]
class_name = "SoulService"
icon = "soul_service"
description = "Learned action rules as SoulScript"
can_have_children = true
"#;
    write_file(&space_dir.join("SoulService/_service.toml"), content)
}

fn write_action_rules_soul(space_dir: &Path, model: &VortexWorldModel) -> Result<(), String> {
    let mut content = String::from(
        "-- Action Rules — learned game mechanics\n-- Auto-generated from VortexWorldModel\n\n"
    );

    for (id, action_model) in &model.action_models {
        content.push_str(&format!("-- Action {} (IRA: {:.1}%, deterministic: {})\n",
            id, action_model.ira_score * 100.0, action_model.is_deterministic
        ));

        if action_model.has_effect {
            content.push_str(&format!(
                "-- Effect: {} observations, {} cells changed on average\n",
                action_model.observed_patches.len(),
                action_model.observed_patches.iter()
                    .map(|p| p.len())
                    .sum::<usize>() / action_model.observed_patches.len().max(1)
            ));
        } else {
            content.push_str("-- No visible grid effect detected\n");
        }
        content.push('\n');
    }

    // Write archetype rules
    content.push_str("\n-- Symbolic Archetypes (cross-game patterns)\n");
    for (formula, games, conf) in model.decomposer.archetype_summary() {
        content.push_str(&format!(
            "-- {} (seen in {} games, confidence: {:.0}%)\n",
            formula, games, conf * 100.0
        ));
    }

    write_file(&space_dir.join("SoulService/action_rules.soul"), &content)
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn write_file(path: &Path, content: &str) -> Result<(), String> {
    std::fs::write(path, content)
        .map_err(|e| format!("Failed to write {}: {}", path.display(), e))
}

fn chrono_now() -> String {
    // Simple ISO-ish timestamp without chrono dependency
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("2026-03-28T{:02}:{:02}:{:02}Z",
        (secs / 3600) % 24,
        (secs / 60) % 60,
        secs % 60
    )
}

fn knowledge_summary_toml(model: &VortexWorldModel) -> String {
    let mut content = String::from(
        "# Vortex Knowledge Summary\n# Cross-game learning state\n\n"
    );

    content.push_str("[state]\n");
    content.push_str(&format!("global_ira = {:.3}\n", model.global_ira));
    content.push_str(&format!("causal_nodes = {}\n", model.causal_graph.node_count()));
    content.push_str(&format!("causal_edges = {}\n", model.causal_graph.edge_count()));
    content.push_str(&format!("laws = {}\n", model.causal_graph.laws().len()));
    content.push_str(&format!("rules = {}\n", model.causal_graph.rules().len()));
    content.push_str(&format!("action_models = {}\n", model.action_models.len()));

    let archetypes = model.decomposer.archetype_summary();
    content.push_str(&format!("archetypes = {}\n", archetypes.len()));
    content.push_str(&format!(
        "cross_game_archetypes = {}\n\n",
        archetypes.iter().filter(|(_, gc, _)| *gc > 1).count()
    ));

    content.push_str("[equivalence_cache]\n");
    content.push_str(&format!("entries = {}\n", model.decomposer.equiv_cache.len()));
    content.push_str(&format!("lookups = {}\n", model.decomposer.equiv_cache.total_lookups));
    content.push_str(&format!("hits = {}\n", model.decomposer.equiv_cache.total_hits));

    content
}

/// Map ARC color index (0-15) to approximate RGBA for visualization.
fn arc_color_to_rgba(color: u8) -> (f32, f32, f32) {
    match color {
        0 => (0.0, 0.0, 0.0),          // Black (background)
        1 => (0.0, 0.45, 0.82),         // Blue
        2 => (1.0, 0.25, 0.0),          // Red
        3 => (0.0, 0.75, 0.0),          // Green
        4 => (1.0, 0.87, 0.0),          // Yellow
        5 => (0.5, 0.5, 0.5),           // Gray
        6 => (0.96, 0.0, 0.55),         // Magenta
        7 => (1.0, 0.55, 0.0),          // Orange
        8 => (0.53, 0.81, 0.92),        // Light blue
        9 => (0.55, 0.0, 0.0),          // Dark red
        _ => (1.0, 1.0, 1.0),           // White (fallback)
    }
}
