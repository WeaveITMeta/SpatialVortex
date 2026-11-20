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
        crate::set_position(&f, *pos)?;
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
