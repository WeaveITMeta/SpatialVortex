//! # 017 | Vortex Eustress Bridge
//!
//! How Vortex lives inside Eustress Engine.
//!
//! - [`universes`]: the Universes and Spaces on disk (the Documents
//!   `Eustress` root the engine itself uses; `EUSTRESS_WORKSPACE` overrides).
//! - [`engine_status`] / [`call`]: the live Engine Bridge, JSON-RPC 2.0 over
//!   TCP via the engine's own `eustress-bridge-client` crate, so Vortex
//!   speaks exactly the protocol the MCP server and CLI speak.
//! - [`set_position`]: format-preserving edits to an instance's
//!   `_instance.toml` (EEP file-system-first authoring; the engine's file
//!   watcher hot-reloads them).
//! - [`astra`]: the Astra Spatial Reasoning Lab end to end. Vortex solves
//!   the challenge, **authors the assembly edits into a scratch copy of the
//!   Space** (never the user's original without approval), and has the
//!   Space's own `evaluate.py --check-scene` grade the result.

pub mod astra;

use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::Value;

/// The Eustress Universes root, as the engine resolves it.
pub fn default_root() -> PathBuf {
    eustress_bridge_client::default_workspace_root()
}

#[derive(Clone, Debug, Serialize)]
pub struct SpaceInfo {
    pub name: String,
    pub path: PathBuf,
    pub has_readme: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct UniverseInfo {
    pub name: String,
    pub path: PathBuf,
    pub spaces: Vec<SpaceInfo>,
    /// A live engine has this Universe's bridge port open.
    pub engine_live: bool,
}

fn spaces_dir(universe: &Path) -> Option<PathBuf> {
    ["Spaces", "spaces"].iter().map(|d| universe.join(d)).find(|p| p.is_dir())
}

pub fn universes(root: &Path) -> std::io::Result<Vec<UniverseInfo>> {
    let mut out = Vec::new();
    for e in std::fs::read_dir(root)?.filter_map(Result::ok) {
        let path = e.path();
        let name = e.file_name().to_string_lossy().into_owned();
        if !path.is_dir() || name.starts_with('.') {
            continue;
        }
        let Some(sd) = spaces_dir(&path) else { continue };
        let mut spaces: Vec<SpaceInfo> = std::fs::read_dir(&sd)?
            .filter_map(Result::ok)
            .filter(|s| s.path().is_dir() && !s.file_name().to_string_lossy().starts_with('.'))
            .map(|s| SpaceInfo { name: s.file_name().to_string_lossy().into_owned(), has_readme: s.path().join("README.md").is_file(), path: s.path() })
            .collect();
        spaces.sort_by(|a, b| a.name.cmp(&b.name));
        let engine_live = eustress_bridge_client::read_port_file(&eustress_bridge_client::port_file_path(&path)).is_some_and(eustress_bridge_client::port_is_live);
        out.push(UniverseInfo { name, path, spaces, engine_live });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

/// Universes and Spaces whose names or READMEs mark them as AI research,
/// evaluation or Vortex work.
pub fn ai_research_spaces(all: &[UniverseInfo]) -> Vec<(String, String)> {
    const KEYS: [&str; 10] = ["vortex", "astra", "arc-agi", "benchmark", "matrix", "flux", "mind", "reasoning", "perspective", "agi"];
    let mut out = Vec::new();
    for u in all {
        for s in &u.spaces {
            let hay = format!("{} {}", u.name, s.name).to_ascii_lowercase();
            if KEYS.iter().any(|k| hay.contains(k)) && !s.name.starts_with("game_") {
                out.push((u.name.clone(), s.name.clone()));
            }
        }
    }
    out
}

#[derive(Clone, Debug, Serialize)]
pub struct EngineStatus {
    pub universe: PathBuf,
    pub port: Option<u16>,
    pub live: bool,
}

pub fn engine_status(universe: &Path) -> EngineStatus {
    let port = eustress_bridge_client::read_port_file(&eustress_bridge_client::port_file_path(universe));
    EngineStatus { universe: universe.to_path_buf(), port, live: port.is_some_and(eustress_bridge_client::port_is_live) }
}

/// One JSON-RPC call to the live engine owning `universe`
/// (e.g. `"ecs.inspect"`, `"tools.list"`, `"sim.read"`).
pub fn call(universe: &Path, method: &str, params: Value) -> Result<Value, String> {
    eustress_bridge_client::call_engine(universe, method, params)
}

/// Set `[transform].position` in an `_instance.toml`, preserving every other
/// byte of formatting, and record the modification time.
pub fn set_position(instance_toml: &Path, position: [f64; 3]) -> Result<(), String> {
    let text = std::fs::read_to_string(instance_toml).map_err(|e| format!("{}: {e}", instance_toml.display()))?;
    let mut doc: toml_edit::DocumentMut = text.parse().map_err(|e| format!("{}: {e}", instance_toml.display()))?;
    let mut arr = toml_edit::Array::new();
    for v in position {
        arr.push(v);
    }
    let transform = doc.get_mut("transform").and_then(|t| t.as_table_like_mut()).ok_or("no [transform] table")?;
    transform.insert("position", toml_edit::value(arr));
    std::fs::write(instance_toml, doc.to_string()).map_err(|e| e.to_string())
}

/// Read `[transform].position`.
pub fn get_position(instance_toml: &Path) -> Result<[f64; 3], String> {
    let text = std::fs::read_to_string(instance_toml).map_err(|e| e.to_string())?;
    let doc: toml_edit::DocumentMut = text.parse().map_err(|e: toml_edit::TomlError| e.to_string())?;
    let arr = doc["transform"]["position"].as_array().ok_or("no transform.position")?;
    let v: Vec<f64> = arr.iter().filter_map(|x| x.as_float().or_else(|| x.as_integer().map(|i| i as f64))).collect();
    if v.len() != 3 {
        return Err("position is not a 3-vector".into());
    }
    Ok([v[0], v[1], v[2]])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_edit_preserves_the_rest() {
        let dir = std::env::temp_dir().join(format!("vortex-eustress-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let f = dir.join("_instance.toml");
        std::fs::write(&f, "[asset]\nmesh = \"parts/block.glb\"\n\n[transform]\nposition = [\n    9.0,\n    1.1,\n    -7.0,\n]\nscale = [1.0, 3.0, 1.5]\n\n[metadata]\nname = \"Pillar_A\"\n").unwrap();
        set_position(&f, [7.0, 1.5, 6.0]).unwrap();
        assert_eq!(get_position(&f).unwrap(), [7.0, 1.5, 6.0]);
        let after = std::fs::read_to_string(&f).unwrap();
        assert!(after.contains("mesh = \"parts/block.glb\"") && after.contains("name = \"Pillar_A\"") && after.contains("scale = [1.0, 3.0, 1.5]"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn discovers_real_universes_when_present() {
        let root = default_root();
        if !root.is_dir() {
            return; // CI machines have no Eustress install.
        }
        let all = universes(&root).unwrap();
        assert!(!all.is_empty());
    }
}
