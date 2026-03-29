//! # ArcSceneMirror — ARC Grid → EustressEngine Scene Reconstruction
//!
//! Converts ARC game frames (64×64 grids, 0-15 colors) into the
//! EustressEngine entity format for live rendering in Explorer.
//!
//! ## Entity Mapping
//!
//! Each grid cell becomes an entity:
//! - Entity ID: `row * grid_width + col + 1` (1-based, 0 reserved for root)
//! - Position: `[col * cell_size, -row * cell_size, 0.0]` (2D plane, Y-down)
//! - Color: ARC color index → RGBA mapping
//! - Scale: `[cell_size, cell_size, 0.1]` (flat quads)
//!
//! Game metadata entities:
//! - Entity 0: Root workspace (game-level properties)
//! - Entity `grid_size + 1`: Score/level display
//! - Entity `grid_size + 2`: Action model overlay
//!
//! ## Delta Computation
//!
//! Only cells that changed between frames generate deltas.
//! This is efficient for the typical ARC case where actions
//! affect a small number of cells per step.
//!
//! ## Output Modes
//!
//! 1. **TOML file** — Write `.eustress/current.toml` directly (standalone mode)
//! 2. **Iggy stream** — Publish SceneDelta messages (when iggy-streaming feature active)
//! 3. **In-memory** — Return deltas for inspection/testing

use eustress_vortex_grid2d::Grid2D;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

/// Configuration for the ARC scene mirror.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArcMirrorConfig {
    /// Size of each cell quad in world units.
    pub cell_size: f32,
    /// Z-depth for the grid plane.
    pub grid_z: f32,
    /// Whether to include metadata entities (score, action model).
    pub include_metadata: bool,
}

impl Default for ArcMirrorConfig {
    fn default() -> Self {
        Self {
            cell_size: 1.0,
            grid_z: 0.0,
            include_metadata: true,
        }
    }
}

/// A scene delta matching the EustressEngine protocol.
/// This is a standalone version of `iggy_delta::SceneDelta`
/// that doesn't require the rkyv/Bevy dependencies.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArcSceneDelta {
    pub entity: u64,
    pub kind: ArcDeltaKind,
    pub seq: u64,
    pub position: Option<[f32; 3]>,
    pub rotation: Option<[f32; 4]>,
    pub scale: Option<[f32; 3]>,
    pub color: Option<[f32; 4]>,
    pub name: Option<String>,
    pub anchored: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ArcDeltaKind {
    PartAdded,
    PartRemoved,
    TransformChanged,
    PropertiesChanged,
    Renamed,
}

/// Mirror entity matching the EustressEngine format.
///
/// Follows the three-layer EEP metadata hierarchy:
/// - **Properties** (BasePart): position, color, scale, anchored, transparency
/// - **Attributes** (dynamic): arc_color, grid_row, grid_col, ira_score, action_effect
/// - **Tags** (classification): color group, region membership, active/changed state
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ArcMirrorEntity {
    // ── Properties (BasePart) ───────────────────────────────────────
    pub entity: u64,
    pub name: String,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
    pub color: [f32; 4],
    pub anchored: bool,
    pub can_collide: bool,
    pub transparency: f32,
    pub reflectance: f32,
    // ── Attributes (dynamic key-value) ──────────────────────────────
    pub attributes: HashMap<String, AttributeValue>,
    // ── Tags (classification strings) ───────────────────────────────
    pub tags: Vec<String>,
    pub last_seq: u64,
}

/// Attribute value types matching EustressEngine's AttributeValue enum.
/// Subset relevant for ARC game state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AttributeValue {
    String(String),
    Number(f64),
    Int(i64),
    Bool(bool),
}

/// The full scene mirror for one ARC game.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ArcSceneMirror {
    pub version: u32,
    pub game_id: String,
    pub step: u32,
    pub max_seq: u64,
    pub grid_width: usize,
    pub grid_height: usize,
    pub levels_completed: u32,
    pub entities: HashMap<u64, ArcMirrorEntity>,
    /// Game-level properties (exposed as TOML globals).
    pub properties: GameProperties,
    config: ArcMirrorConfig,
}

/// Game-level properties serialized as TOML globals.
/// Uses the EEP rich-schema format:
///   key = { type = "...", value = ..., description = "..." }
/// for display in the Explorer Properties panel.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GameProperties {
    // ── Standard Properties ─────────────────────────────────────
    pub game_id: String,
    pub step: u32,
    pub levels_completed: u32,
    pub win_levels: u32,
    pub total_actions: u32,
    // ── Attributes (dynamic, shown in Properties panel) ────────
    pub global_ira: f32,
    pub dominant_colors: Vec<u8>,
    pub grid_symmetry: String,
    pub last_action: String,
    pub last_action_effect: String,
    // ── Tags (game-level classification) ───────────────────────
    pub tags: Vec<String>,
}

impl ArcSceneMirror {
    pub fn new(game_id: &str, config: ArcMirrorConfig) -> Self {
        Self {
            version: 1,
            game_id: game_id.to_string(),
            config,
            ..Default::default()
        }
    }

    /// Apply a new ARC game frame. Returns the deltas generated.
    ///
    /// On first call: generates PartAdded for every cell.
    /// On subsequent calls: generates PropertiesChanged only for cells that changed color.
    pub fn apply_frame(
        &mut self,
        grid: &Grid2D,
        step: u32,
        levels_completed: u32,
    ) -> Vec<ArcSceneDelta> {
        let mut deltas = Vec::new();
        let first_frame = self.entities.is_empty();

        self.step = step;
        self.levels_completed = levels_completed;
        self.grid_width = grid.width;
        self.grid_height = grid.height;

        // Update game properties
        self.properties.game_id = self.game_id.clone();
        self.properties.step = step;
        self.properties.levels_completed = levels_completed;
        self.properties.dominant_colors = grid.dominant_colors(3);

        // Update game-level tags based on state
        self.properties.tags.clear();
        self.properties.tags.push(format!("step_{}", step));
        self.properties.tags.push(format!("level_{}", levels_completed));
        if first_frame { self.properties.tags.push("first_frame".into()); }
        if grid.width == grid.height { self.properties.tags.push("square_grid".into()); }

        for r in 0..grid.height {
            for c in 0..grid.width {
                let entity_id = (r * grid.width + c + 1) as u64;
                let arc_color = grid.cells[r][c];
                let rgba = arc_color_to_rgba(arc_color);
                let position = [
                    c as f32 * self.config.cell_size,
                    -(r as f32) * self.config.cell_size,
                    self.config.grid_z,
                ];
                let scale = [self.config.cell_size, self.config.cell_size, 0.1];

                self.max_seq += 1;
                let seq = self.max_seq;

                if first_frame {
                    // First frame: add all cells with Properties + Attributes + Tags
                    let bg = grid.background_color();
                    let is_bg = arc_color == bg;

                    // Attributes: ARC-specific metadata
                    let mut attributes = HashMap::new();
                    attributes.insert("arc_color".into(), AttributeValue::Int(arc_color as i64));
                    attributes.insert("grid_row".into(), AttributeValue::Int(r as i64));
                    attributes.insert("grid_col".into(), AttributeValue::Int(c as i64));
                    attributes.insert("is_background".into(), AttributeValue::Bool(is_bg));

                    // Tags: classification for grouping/querying
                    let mut tags = vec![
                        format!("color_{}", arc_color),
                        format!("row_{}", r),
                        format!("col_{}", c),
                    ];
                    if is_bg { tags.push("background".into()); }
                    if r == 0 || r == grid.height - 1 || c == 0 || c == grid.width - 1 {
                        tags.push("border".into());
                    }

                    let entity = ArcMirrorEntity {
                        entity: entity_id,
                        name: format!("cell_{}_{}", r, c),
                        position,
                        rotation: [0.0, 0.0, 0.0, 1.0],
                        scale,
                        color: rgba,
                        anchored: true,
                        can_collide: false,
                        transparency: if is_bg { 0.8 } else { 0.0 },
                        reflectance: 0.0,
                        attributes,
                        tags,
                        last_seq: seq,
                    };
                    self.entities.insert(entity_id, entity);

                    deltas.push(ArcSceneDelta {
                        entity: entity_id,
                        kind: ArcDeltaKind::PartAdded,
                        seq,
                        position: Some(position),
                        rotation: Some([0.0, 0.0, 0.0, 1.0]),
                        scale: Some(scale),
                        color: Some(rgba),
                        name: Some(format!("cell_{}_{}", r, c)),
                        anchored: Some(true),
                    });
                } else if let Some(existing) = self.entities.get_mut(&entity_id) {
                    // Subsequent frames: only emit delta if color changed
                    let old_color = existing.attributes.get("arc_color")
                        .and_then(|v| if let AttributeValue::Int(i) = v { Some(*i as u8) } else { None })
                        .unwrap_or(255);

                    if old_color != arc_color {
                        let bg = grid.background_color();
                        let is_bg = arc_color == bg;

                        // Update Properties
                        existing.color = rgba;
                        existing.transparency = if is_bg { 0.8 } else { 0.0 };
                        existing.last_seq = seq;

                        // Update Attributes
                        existing.attributes.insert("arc_color".into(), AttributeValue::Int(arc_color as i64));
                        existing.attributes.insert("is_background".into(), AttributeValue::Bool(is_bg));
                        existing.attributes.insert("changed_at_step".into(), AttributeValue::Int(step as i64));

                        // Update Tags: swap color tag
                        existing.tags.retain(|t| !t.starts_with("color_") && t != "background" && t != "changed");
                        existing.tags.push(format!("color_{}", arc_color));
                        if is_bg { existing.tags.push("background".into()); }
                        existing.tags.push("changed".into()); // Mark as recently changed

                        deltas.push(ArcSceneDelta {
                            entity: entity_id,
                            kind: ArcDeltaKind::PropertiesChanged,
                            seq,
                            position: None,
                            rotation: None,
                            scale: None,
                            color: Some(rgba),
                            name: None,
                            anchored: None,
                        });
                    } else {
                        // No color change — clear "changed" tag from previous step
                        existing.tags.retain(|t| t != "changed");
                    }
                }
            }
        }

        if !deltas.is_empty() {
            info!(
                "ArcSceneMirror: {} deltas for step {} ({} total entities)",
                deltas.len(), step, self.entities.len()
            );
        }

        deltas
    }

    /// Set the last action info for property display.
    pub fn set_last_action(&mut self, action: &str, effect: &str) {
        self.properties.last_action = action.to_string();
        self.properties.last_action_effect = effect.to_string();
    }

    /// Set grid symmetry property.
    pub fn set_symmetry(&mut self, symmetry: &str) {
        self.properties.grid_symmetry = symmetry.to_string();
    }

    /// Serialize to the `.eustress/current.toml` format.
    /// This produces a TOML file compatible with the EustressEngine
    /// materializer format — readable by Explorer.
    pub fn to_current_toml(&self) -> String {
        let mut out = String::with_capacity(4096);

        // Header
        out.push_str(&format!(
            "# ARC Game: {} — Step {} — Level {}\n",
            self.game_id, self.step, self.levels_completed
        ));
        out.push_str(&format!("# {} entities, seq {}\n\n", self.entities.len(), self.max_seq));

        // Scene metadata (matches SceneMirror format)
        out.push_str(&format!("version = {}\n", self.version));
        out.push_str(&format!("session_id = \"{}\"\n", self.game_id));
        out.push_str(&format!("max_seq = {}\n\n", self.max_seq));

        // ── Game Properties (standard) ────────────────────────────
        out.push_str("[properties]\n");
        out.push_str(&format!("game_id = \"{}\"\n", self.properties.game_id));
        out.push_str(&format!("step = {}\n", self.properties.step));
        out.push_str(&format!("levels_completed = {}\n", self.properties.levels_completed));
        out.push_str(&format!("win_levels = {}\n", self.properties.win_levels));
        out.push_str(&format!("total_actions = {}\n", self.properties.total_actions));
        out.push_str(&format!("grid_width = {}\n", self.grid_width));
        out.push_str(&format!("grid_height = {}\n", self.grid_height));
        out.push('\n');

        // ── Game Attributes (rich-schema for Explorer Properties panel) ──
        out.push_str("[attributes]\n");
        out.push_str(&format!(
            "global_ira = {{ type = \"float\", value = {:.3}, min = 0.0, max = 1.0, description = \"Internal Representation Accuracy\" }}\n",
            self.properties.global_ira
        ));
        out.push_str(&format!(
            "grid_symmetry = {{ type = \"string\", value = \"{}\", description = \"Detected grid symmetry\" }}\n",
            self.properties.grid_symmetry
        ));
        out.push_str(&format!(
            "last_action = {{ type = \"string\", value = \"{}\", description = \"Last action taken\" }}\n",
            self.properties.last_action
        ));
        out.push_str(&format!(
            "last_action_effect = {{ type = \"string\", value = \"{}\", description = \"Reasoning for last action\" }}\n",
            self.properties.last_action_effect.replace('"', "'")
        ));

        if !self.properties.dominant_colors.is_empty() {
            out.push_str(&format!(
                "dominant_colors = {{ type = \"string\", value = \"[{}]\", description = \"Most frequent non-background colors\" }}\n",
                self.properties.dominant_colors.iter()
                    .map(|c| c.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        out.push('\n');

        // ── Game Tags ────────────────────────────────────────────
        if !self.properties.tags.is_empty() {
            out.push_str(&format!(
                "tags = [{}]\n\n",
                self.properties.tags.iter().map(|t| format!("\"{}\"", t)).collect::<Vec<_>>().join(", ")
            ));
        }

        // Entities — EEP format: [entities.{id}] with Properties, Attributes, Tags
        let mut ids: Vec<u64> = self.entities.keys().copied().collect();
        ids.sort();

        for id in ids {
            let e = &self.entities[&id];

            // ── Properties (BasePart standard) ──────────────────────
            out.push_str(&format!("[entities.{}]\n", id));
            out.push_str(&format!("entity = {}\n", e.entity));
            out.push_str(&format!("name = \"{}\"\n", e.name));
            out.push_str(&format!(
                "position = [{:.1}, {:.1}, {:.1}]\n",
                e.position[0], e.position[1], e.position[2]
            ));
            out.push_str(&format!(
                "rotation = [{:.1}, {:.1}, {:.1}, {:.1}]\n",
                e.rotation[0], e.rotation[1], e.rotation[2], e.rotation[3]
            ));
            out.push_str(&format!(
                "scale = [{:.1}, {:.1}, {:.1}]\n",
                e.scale[0], e.scale[1], e.scale[2]
            ));
            out.push_str(&format!(
                "color = [{:.3}, {:.3}, {:.3}, {:.3}]\n",
                e.color[0], e.color[1], e.color[2], e.color[3]
            ));
            out.push_str(&format!("anchored = {}\n", e.anchored));
            out.push_str(&format!("can_collide = {}\n", e.can_collide));
            out.push_str(&format!("transparency = {:.1}\n", e.transparency));
            out.push_str(&format!("reflectance = {:.1}\n", e.reflectance));
            out.push_str(&format!("last_seq = {}\n", e.last_seq));

            // ── Tags (classification) ───────────────────────────────
            if !e.tags.is_empty() {
                out.push_str(&format!(
                    "tags = [{}]\n",
                    e.tags.iter().map(|t| format!("\"{}\"", t)).collect::<Vec<_>>().join(", ")
                ));
            }

            // ── Attributes (dynamic metadata) ───────────────────────
            if !e.attributes.is_empty() {
                out.push_str(&format!("\n[entities.{}.attributes]\n", id));
                let mut attr_keys: Vec<&String> = e.attributes.keys().collect();
                attr_keys.sort();
                for key in attr_keys {
                    let val = &e.attributes[key];
                    match val {
                        AttributeValue::String(s) => out.push_str(&format!("{} = \"{}\"\n", key, s)),
                        AttributeValue::Number(n) => out.push_str(&format!("{} = {:.3}\n", key, n)),
                        AttributeValue::Int(i) => out.push_str(&format!("{} = {}\n", key, i)),
                        AttributeValue::Bool(b) => out.push_str(&format!("{} = {}\n", key, b)),
                    }
                }
            }

            out.push('\n');
        }

        out
    }

    /// Write the current state to `.eustress/current.toml` in the Space directory.
    pub fn write_to_space(&self, space_dir: &std::path::Path) -> Result<(), String> {
        let toml = self.to_current_toml();
        let path = space_dir.join(".eustress").join("current.toml");
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create dir: {}", e))?;
        }
        std::fs::write(&path, &toml)
            .map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;
        Ok(())
    }

    /// Clear all entities (for game reset / new level).
    pub fn clear(&mut self) -> Vec<ArcSceneDelta> {
        let mut deltas = Vec::new();
        for (&entity_id, _) in &self.entities {
            self.max_seq += 1;
            deltas.push(ArcSceneDelta {
                entity: entity_id,
                kind: ArcDeltaKind::PartRemoved,
                seq: self.max_seq,
                position: None,
                rotation: None,
                scale: None,
                color: None,
                name: None,
                anchored: None,
            });
        }
        self.entities.clear();
        deltas
    }
}

/// Map ARC color index (0-15) to linear RGBA.
pub fn arc_color_to_rgba(color: u8) -> [f32; 4] {
    match color {
        0  => [0.0,   0.0,   0.0,   1.0],  // Black (background)
        1  => [0.0,   0.45,  0.82,  1.0],   // Blue
        2  => [1.0,   0.25,  0.0,   1.0],   // Red
        3  => [0.0,   0.75,  0.0,   1.0],   // Green
        4  => [1.0,   0.87,  0.0,   1.0],   // Yellow
        5  => [0.5,   0.5,   0.5,   1.0],   // Gray
        6  => [0.96,  0.0,   0.55,  1.0],   // Magenta
        7  => [1.0,   0.55,  0.0,   1.0],   // Orange
        8  => [0.53,  0.81,  0.92,  1.0],   // Light blue
        9  => [0.55,  0.0,   0.0,   1.0],   // Dark red
        10 => [0.0,   0.35,  0.0,   1.0],   // Dark green
        11 => [0.0,   0.0,   0.55,  1.0],   // Dark blue
        12 => [0.65,  0.55,  0.0,   1.0],   // Dark yellow / brown
        13 => [0.35,  0.0,   0.35,  1.0],   // Purple
        14 => [0.0,   0.55,  0.55,  1.0],   // Teal
        15 => [1.0,   1.0,   1.0,   1.0],   // White
        _  => [0.8,   0.8,   0.8,   1.0],   // Fallback light gray
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Grid2D extension: dominant colors
// ─────────────────────────────────────────────────────────────────────────────

/// Extension trait for Grid2D to get dominant colors.
pub trait GridColorExt {
    fn dominant_colors(&self, n: usize) -> Vec<u8>;
}

impl GridColorExt for Grid2D {
    fn dominant_colors(&self, n: usize) -> Vec<u8> {
        let bg = self.background_color();
        let mut counts: HashMap<u8, usize> = HashMap::new();
        for row in &self.cells {
            for &c in row {
                if c != bg {
                    *counts.entry(c).or_default() += 1;
                }
            }
        }
        let mut sorted: Vec<(u8, usize)> = counts.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.into_iter().take(n).map(|(c, _)| c).collect()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_frame_creates_all_entities() {
        let grid = Grid2D::new(vec![vec![0, 1, 2], vec![3, 4, 5]]);
        let mut mirror = ArcSceneMirror::new("test_game", ArcMirrorConfig::default());
        let deltas = mirror.apply_frame(&grid, 0, 0);

        // 2×3 grid = 6 cells = 6 PartAdded deltas
        assert_eq!(deltas.len(), 6);
        assert!(deltas.iter().all(|d| d.kind == ArcDeltaKind::PartAdded));
        assert_eq!(mirror.entities.len(), 6);
    }

    #[test]
    fn test_second_frame_only_changed_cells() {
        let grid1 = Grid2D::new(vec![vec![0, 1, 2], vec![3, 4, 5]]);
        let grid2 = Grid2D::new(vec![vec![0, 1, 2], vec![3, 9, 5]]); // Only cell (1,1) changed

        let mut mirror = ArcSceneMirror::new("test", ArcMirrorConfig::default());
        mirror.apply_frame(&grid1, 0, 0);
        let deltas = mirror.apply_frame(&grid2, 1, 0);

        // Only 1 cell changed
        assert_eq!(deltas.len(), 1);
        assert_eq!(deltas[0].kind, ArcDeltaKind::PropertiesChanged);
        // Entity ID for (1,1) in a 3-wide grid = 1*3 + 1 + 1 = 5
        assert_eq!(deltas[0].entity, 5);
    }

    #[test]
    fn test_no_change_no_deltas() {
        let grid = Grid2D::new(vec![vec![0, 1], vec![2, 3]]);
        let mut mirror = ArcSceneMirror::new("test", ArcMirrorConfig::default());
        mirror.apply_frame(&grid, 0, 0);
        let deltas = mirror.apply_frame(&grid, 1, 0); // Same grid
        assert_eq!(deltas.len(), 0);
    }

    #[test]
    fn test_toml_output() {
        let grid = Grid2D::new(vec![vec![0, 1], vec![2, 3]]);
        let mut mirror = ArcSceneMirror::new("test", ArcMirrorConfig::default());
        mirror.apply_frame(&grid, 0, 0);

        let toml = mirror.to_current_toml();
        assert!(toml.contains("version = 1"));
        assert!(toml.contains("session_id = \"test\""));
        assert!(toml.contains("[entities.1]"));
        assert!(toml.contains("arc_color = 0"));
        assert!(toml.contains("[properties]"));
        assert!(toml.contains("grid_width = 2"));
    }

    #[test]
    fn test_clear_generates_remove_deltas() {
        let grid = Grid2D::new(vec![vec![0, 1], vec![2, 3]]);
        let mut mirror = ArcSceneMirror::new("test", ArcMirrorConfig::default());
        mirror.apply_frame(&grid, 0, 0);
        assert_eq!(mirror.entities.len(), 4);

        let deltas = mirror.clear();
        assert_eq!(deltas.len(), 4);
        assert!(deltas.iter().all(|d| d.kind == ArcDeltaKind::PartRemoved));
        assert_eq!(mirror.entities.len(), 0);
    }

    #[test]
    fn test_arc_color_mapping() {
        let rgba = arc_color_to_rgba(1); // Blue
        assert!(rgba[2] > 0.5); // Blue channel should be high
        assert!(rgba[3] == 1.0); // Alpha = 1

        let black = arc_color_to_rgba(0);
        assert_eq!(black, [0.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_dominant_colors() {
        let grid = Grid2D::new(vec![
            vec![0, 0, 1, 1, 1],
            vec![0, 2, 2, 1, 1],
            vec![0, 0, 2, 0, 0],
        ]);
        let dominant = grid.dominant_colors(2);
        // Color 1 appears 5 times, color 2 appears 3 times (bg=0 excluded)
        assert_eq!(dominant[0], 1);
        assert_eq!(dominant[1], 2);
    }
}
