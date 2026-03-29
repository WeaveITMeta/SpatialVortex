use eustress_arc_policy::space_writer::{self, SpaceWriterConfig};
use eustress_arc_policy::world_model::VortexWorldModel;
use eustress_arc_types::{ArcStep, PolicyDecision};
use eustress_vortex_grid2d::Grid2D;
use std::path::PathBuf;
use tracing::info;

/// Default knowledge file path (relative to working directory).
const KNOWLEDGE_FILE: &str = "knowledge/vortex_knowledge.json";

/// Stateful policy wrapper that maintains the VortexWorldModel across steps.
/// This is the Phase 3 world-model policy — full Eustress pipeline.
///
/// Supports cross-game knowledge persistence via JSON snapshots
/// and EEP-compliant TOML Spaces viewable in EustressEngine Explorer.
pub struct Policy {
    model: VortexWorldModel,
    knowledge_path: PathBuf,
    space_config: SpaceWriterConfig,
    /// Current game ID for Space CRUD lifecycle.
    current_game: String,
    /// Steps since last Space write (debounce).
    steps_since_write: u32,
}

impl Policy {
    pub fn new() -> Self {
        let knowledge_path = std::env::var("VORTEX_KNOWLEDGE_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(KNOWLEDGE_FILE));

        let mut model = VortexWorldModel::new();

        // Try to load existing cross-game knowledge
        if knowledge_path.exists() {
            match std::fs::read_to_string(&knowledge_path) {
                Ok(json) => {
                    match model.import_knowledge(&json) {
                        Ok(()) => info!(
                            "Loaded cross-game knowledge from {}",
                            knowledge_path.display()
                        ),
                        Err(e) => info!(
                            "Failed to import knowledge (starting fresh): {}",
                            e
                        ),
                    }
                }
                Err(e) => info!(
                    "No existing knowledge file at {} ({}), starting fresh",
                    knowledge_path.display(), e
                ),
            }
        } else {
            info!("No knowledge file at {}, starting with clean slate", knowledge_path.display());
        }

        Self {
            model,
            knowledge_path,
            space_config: SpaceWriterConfig::default(),
            current_game: String::new(),
            steps_since_write: 0,
        }
    }

    /// Access the underlying world model (for scene mirror, etc.).
    pub fn model(&self) -> &VortexWorldModel {
        &self.model
    }

    /// Scene deltas from the last `decide()` call (used in Iggy mode).
    #[allow(dead_code)]
    pub fn last_scene_deltas(&self) -> &[eustress_arc_policy::scene_mirror::ArcSceneDelta] {
        &self.model.last_scene_deltas
    }

    /// Phase 3 decision: full Eustress vortex-core pipeline.
    /// Also writes the game state to an EEP Space for Explorer viewing.
    pub fn decide(&mut self, step: &ArcStep) -> PolicyDecision {
        let game_id = step.game_id().unwrap_or("unknown").to_string();

        // Detect game change — delete old Space, start fresh
        if game_id != self.current_game && !self.current_game.is_empty() {
            self.on_game_end();
        }
        self.current_game = game_id.clone();

        let decision = eustress_arc_policy::decide_world_model(&mut self.model, step);

        // Write Space every 5 steps (debounced to avoid excessive I/O)
        self.steps_since_write += 1;
        if self.steps_since_write >= 5 {
            self.steps_since_write = 0;
            if let Some(cells) = step.frame_grid() {
                let grid = Grid2D::new(cells);
                let _ = space_writer::write_game_space(
                    &self.space_config,
                    &game_id,
                    &grid,
                    &self.model,
                );
            }
        }

        decision
    }

    /// Called when a game ends (WIN or GAME_OVER).
    /// Saves knowledge and cleans up the game Space.
    pub fn on_game_end(&mut self) {
        // Save cross-game knowledge
        self.save_knowledge();
        let _ = space_writer::write_knowledge(&self.space_config, &self.model);

        // Delete the game Space (clean slate per game)
        if !self.current_game.is_empty() {
            let _ = space_writer::delete_game_space(&self.space_config, &self.current_game);
        }
    }

    /// Save cross-game knowledge to disk (JSON + TOML).
    pub fn save_knowledge(&self) {
        let json = self.model.export_knowledge();
        if json.is_empty() { return; }

        // Ensure parent directory exists
        if let Some(parent) = self.knowledge_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        match std::fs::write(&self.knowledge_path, &json) {
            Ok(()) => info!(
                "Saved cross-game knowledge ({} bytes) to {}",
                json.len(),
                self.knowledge_path.display()
            ),
            Err(e) => info!(
                "Failed to save knowledge to {}: {}",
                self.knowledge_path.display(), e
            ),
        }

        // Also write TOML knowledge for Explorer
        let _ = space_writer::write_knowledge(&self.space_config, &self.model);
    }
}

impl Drop for Policy {
    fn drop(&mut self) {
        // Auto-save knowledge when the policy is dropped (agent cleanup)
        self.on_game_end();
    }
}
