//! Astra Spatial Reasoning Lab, end to end, graded by the Space's own
//! evaluator.
//!
//! The blind protocol from the Space's README is followed: Vortex reads
//! `DataService/challenge.json` and the scene, and never opens
//! `reference_solution.json`. Edits go into a scratch copy containing just
//! what `evaluate.py --check-scene` reads (the evaluator, the challenge and
//! `Workspace/Assembly`), so the user's Space is untouched.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;
use vortex_kernel::{SpatialChallenge, SpatialSubmission};

pub fn load_challenge(space: &Path) -> Result<SpatialChallenge, String> {
    let p = space.join("DataService").join("challenge.json");
    let text = std::fs::read_to_string(&p).map_err(|e| format!("{}: {e}", p.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("{}: {e}", p.display()))
}

fn copy_dir(from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for e in std::fs::read_dir(from)? {
        let e = e?;
        let dst = to.join(e.file_name());
        if e.file_type()?.is_dir() {
            copy_dir(&e.path(), &dst)?;
        } else {
            std::fs::copy(e.path(), dst)?;
        }
    }
    Ok(())
}

/// Build the scratch copy. Returns its root.
pub fn scratch_copy(space: &Path, scratch: &Path) -> Result<PathBuf, String> {
    let root = scratch.join("astra-run");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("DataService")).map_err(|e| e.to_string())?;
    std::fs::copy(space.join("evaluate.py"), root.join("evaluate.py")).map_err(|e| format!("evaluate.py: {e}"))?;
    std::fs::copy(space.join("DataService/challenge.json"), root.join("DataService/challenge.json")).map_err(|e| format!("challenge.json: {e}"))?;
    copy_dir(&space.join("Workspace/Assembly"), &root.join("Workspace/Assembly")).map_err(|e| format!("Workspace/Assembly: {e}"))?;
    Ok(root)
}

/// Author the assembly placements into the scratch scene.
pub fn author_assembly(root: &Path, sub: &SpatialSubmission) -> Result<Vec<String>, String> {
    let mut edited = Vec::new();
    for (name, pos) in &sub.assembly_positions {
        let f = root.join("Workspace/Assembly").join(name).join("_instance.toml");
        set_position(&f, *pos)?;
        edited.push(name.clone());
    }
    Ok(edited)
}

/// Run the Space's own evaluator on a submission. `check_scene` also
/// verifies the on-disk placements.
pub fn evaluate(root: &Path, sub: &SpatialSubmission, check_scene: bool) -> Result<Value, String> {
    let sub_path = root.join("submission.json");
    std::fs::write(&sub_path, serde_json::to_string_pretty(sub).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    let python = if cfg!(windows) { "python" } else { "python3" };
    let mut cmd = Command::new(python);
    cmd.arg(root.join("evaluate.py")).arg(&sub_path).current_dir(root);
    if check_scene {
        cmd.arg("--check-scene");
    }
    let out = cmd.output().map_err(|e| format!("python: {e}"))?;
    if !out.status.success() {
        return Err(format!("evaluate.py failed: {}", String::from_utf8_lossy(&out.stderr)));
    }
    serde_json::from_slice(&out.stdout).map_err(|e| format!("evaluate.py output: {e}"))
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
        let dir = std::env::temp_dir().join(format!("vortex-astra-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let f = dir.join("_instance.toml");
        std::fs::write(&f, "[asset]\nmesh = \"parts/block.glb\"\n\n[transform]\nposition = [\n    9.0,\n    1.1,\n    -7.0,\n]\nscale = [1.0, 3.0, 1.5]\n\n[metadata]\nname = \"Pillar_A\"\n").unwrap();
        set_position(&f, [7.0, 1.5, 6.0]).unwrap();
        assert_eq!(get_position(&f).unwrap(), [7.0, 1.5, 6.0]);
        let after = std::fs::read_to_string(&f).unwrap();
        assert!(after.contains("mesh = \"parts/block.glb\"") && after.contains("name = \"Pillar_A\"") && after.contains("scale = [1.0, 3.0, 1.5]"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
