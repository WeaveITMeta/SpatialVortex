//! # 012 | Vortex RSI: the append-only ledger
//!
//! The improvement gate (`vortex_api::rsi`) decides; this records every
//! decision, accepted or not, with the incumbent kept for rollback. Writing a
//! file is I/O, so it lives here rather than in the pure API.

use std::io::Write;
use std::path::Path;

use vortex_api::rsi::Selection;

/// Append a decision to the RSI ledger (JSONL, append-only).
pub fn record(path: &Path, selection: &Selection) -> std::io::Result<()> {
    if let Some(p) = path.parent() {
        std::fs::create_dir_all(p)?;
    }
    let mut f = std::fs::OpenOptions::new().create(true).append(true).open(path)?;
    let line = serde_json::to_string(selection).map_err(std::io::Error::other)?;
    writeln!(f, "{line}")
}
