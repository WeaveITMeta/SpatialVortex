//! Background TOML writer with debounced disk I/O.
//!
//! In standalone mode (no Iggy), this replaces the synchronous
//! `space_writer::write_game_space()` every-5-steps pattern with a
//! non-blocking channel + debounce approach:
//!
//! - `policy.decide()` returns immediately
//! - Scene deltas + grid state are sent to a background thread via channel
//! - Background thread coalesces writes with 200ms debounce (matches materializer)
//! - Disk I/O never blocks the main stdin/stdout loop

use eustress_arc_policy::space_writer::{self, SpaceWriterConfig};
use eustress_vortex_grid2d::Grid2D;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use tracing::info;

/// Message sent to the background writer thread.
pub enum WriteMsg {
    /// Write full Space state (grid + model snapshot).
    WriteSpace {
        game_id: String,
        grid: Grid2D,
        /// Serialized scene mirror TOML for current.toml hot state.
        current_toml: String,
    },
    /// Game ended — write knowledge, delete Space.
    GameEnd {
        game_id: String,
    },
    /// Shutdown the writer thread.
    Shutdown,
}

/// Handle to the background writer — send messages, never blocks.
pub struct BackgroundTomlWriter {
    tx: mpsc::Sender<WriteMsg>,
}

impl BackgroundTomlWriter {
    /// Spawn the background writer thread. Returns immediately.
    pub fn spawn() -> Self {
        let (tx, rx) = mpsc::channel::<WriteMsg>();

        std::thread::Builder::new()
            .name("toml-writer".into())
            .spawn(move || writer_loop(rx))
            .expect("failed to spawn toml-writer thread");

        Self { tx }
    }

    /// Queue a Space write (non-blocking, drops if channel full).
    pub fn queue_write(&self, game_id: String, grid: Grid2D, current_toml: String) {
        let _ = self.tx.send(WriteMsg::WriteSpace {
            game_id,
            grid,
            current_toml,
        });
    }

    /// Signal game end (write knowledge, cleanup).
    pub fn queue_game_end(&self, game_id: String) {
        let _ = self.tx.send(WriteMsg::GameEnd { game_id });
    }

    /// Shutdown the writer thread.
    pub fn shutdown(&self) {
        let _ = self.tx.send(WriteMsg::Shutdown);
    }
}

impl Drop for BackgroundTomlWriter {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Background writer loop with 200ms debounce.
fn writer_loop(rx: mpsc::Receiver<WriteMsg>) {
    let config = SpaceWriterConfig::default();
    let debounce = Duration::from_millis(200);
    let mut pending_write: Option<WriteMsg> = None;
    let mut last_write = Instant::now() - debounce; // Allow immediate first write

    loop {
        // Try to receive with timeout matching debounce interval
        let timeout = debounce.saturating_sub(last_write.elapsed());
        match rx.recv_timeout(timeout) {
            Ok(WriteMsg::Shutdown) => {
                // Flush pending write before exit
                if let Some(msg) = pending_write.take() {
                    execute_write(&config, msg);
                }
                info!("[toml-writer] Shutting down");
                return;
            }
            Ok(msg @ WriteMsg::GameEnd { .. }) => {
                // Game end is immediate — flush pending + execute
                if let Some(pending) = pending_write.take() {
                    execute_write(&config, pending);
                }
                execute_write(&config, msg);
                last_write = Instant::now();
            }
            Ok(msg @ WriteMsg::WriteSpace { .. }) => {
                // Coalesce: always keep latest, debounce will flush
                pending_write = Some(msg);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // Debounce expired — flush pending write
                if let Some(msg) = pending_write.take() {
                    execute_write(&config, msg);
                    last_write = Instant::now();
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                // Channel closed — flush and exit
                if let Some(msg) = pending_write.take() {
                    execute_write(&config, msg);
                }
                return;
            }
        }
    }
}

fn execute_write(config: &SpaceWriterConfig, msg: WriteMsg) {
    match msg {
        WriteMsg::WriteSpace { game_id, grid, current_toml } => {
            // Write the hot current.toml (scene mirror state)
            let space_dir = config.universe_root.join("spaces").join(format!("game_{}", game_id));
            let eustress_dir = space_dir.join(".eustress");
            let _ = std::fs::create_dir_all(&eustress_dir);
            let _ = std::fs::write(eustress_dir.join("current.toml"), &current_toml);

            // Write the Grid.part.toml (compact format)
            let workspace_dir = space_dir.join("Workspace");
            let _ = std::fs::create_dir_all(&workspace_dir);
            let grid_toml = grid_to_toml(&grid);
            let _ = std::fs::write(workspace_dir.join("Grid.part.toml"), &grid_toml);
        }
        WriteMsg::GameEnd { game_id } => {
            let _ = space_writer::delete_game_space(config, &game_id);
        }
        WriteMsg::Shutdown => {} // handled in caller
    }
}

fn grid_to_toml(grid: &Grid2D) -> String {
    let mut cells_str = String::from("[\n");
    for row in &grid.cells {
        cells_str.push_str("  [");
        cells_str.push_str(
            &row.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(", ")
        );
        cells_str.push_str("],\n");
    }
    cells_str.push(']');

    format!(
        r#"[metadata]
class_name = "Part"
archivable = true

[transform]
position = [0.0, 0.0, 0.0]
rotation = [0.0, 0.0, 0.0, 1.0]
scale = [1.0, 1.0, 1.0]

[properties]
color = [0.0, 0.0, 0.0, 1.0]
anchored = true
can_collide = false
locked = true

[Extra.grid]
width = {w}
height = {h}
cells = {cells}
"#,
        w = grid.width,
        h = grid.height,
        cells = cells_str,
    )
}
