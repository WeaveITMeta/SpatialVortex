//! eustress_schema — SpatialVortex reading Eustress Engine's REAL object model.
//!
//! This is the refactor that stops Vortex being a parallel abstraction and makes
//! it a citizen of the engine's data contract. Three rules it now plays by:
//!
//! 1. **Eustress's real schema, not shadow types.** The DTOs below mirror the
//!    engine's actual on-disk `_instance.toml` interchange format and the field
//!    names of `eustress_common::classes::Instance` (`name`, `class_name`,
//!    `archivable`, `uuid`, `ai`) — verified against
//!    `crates/common/src/classes.rs:1123`.
//!
//! 2. **The `ai` training opt-in is honored.** `Instance::ai` is documented in
//!    the engine as: *"AI training opt-in flag (default: false) — when true,
//!    this entity is included in SpatialVortex training data exports."* That is
//!    a governance contract the engine authored FOR this crate, and every prior
//!    loader here ignored it. [`AiExportPolicy`] makes the decision explicit and
//!    reportable instead of silent.
//!
//! 3. **Format boundary, not link boundary.** `eustress-common` is Bevy-coupled
//!    (`bevy_render`, `bevy_pbr`, `bevy_gltf`), so linking it would drag the
//!    render stack into this CPU-only crate. Reading the documented on-disk
//!    format — the same one the headless `eustress` CLI operates on — keeps
//!    aimodel standalone while still speaking the engine's language.
//!
//! Output is [`NodeContext`] values, so a loaded Space plugs straight into
//! `ml::flux_walk` / `ml::spatial_corpus` with no adapter.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::ml::flux_walk::{NodeContext, NodeId, Oklch, SpatialPos};

/// One `_instance.toml` file: the engine's on-disk representation of an Instance.
#[derive(Debug, Clone, Deserialize)]
pub struct InstanceFile {
    pub metadata: InstanceMetadata,
    #[serde(default)]
    pub transform: Option<TransformData>,
    #[serde(default)]
    pub properties: Option<InstanceProperties>,
    /// `[references]` — UUID-valued links (e.g. `Attachment0`/`Attachment1` on a
    /// constraint). These are the engine's real edges between instances.
    #[serde(default)]
    pub references: HashMap<String, String>,
}

/// `[metadata]` — mirrors `eustress_common::classes::Instance`.
#[derive(Debug, Clone, Deserialize)]
pub struct InstanceMetadata {
    /// Instance label. **Defaulted on purpose.** A required `name` silently
    /// destroyed whole Spaces: `toml::from_str` failed the entire file, the
    /// error was counted as a `parse_failure` and swallowed, and `load_space`
    /// still returned `Ok` — so a caller saw an empty corpus rather than an
    /// error. Measured: 291 of 291 files in `Weave/Spaces/Vortex` were dropped
    /// this way, including the mind map that documents this crate.
    ///
    /// The engine's own identity for an instance is its DIRECTORY name; the
    /// `name` key is a convenience copy that some authoring paths omit. So an
    /// absent key is normal, and `load_space` backfills it from the folder.
    #[serde(default)]
    pub name: String,
    /// The engine's `ClassName` taxonomy (473 variants). Kept as `String` so an
    /// unknown/new class never fails the load — aimodel must tolerate an engine
    /// that evolves faster than it does.
    #[serde(default)]
    pub class_name: String,
    #[serde(default)]
    pub archivable: bool,
    #[serde(default)]
    pub uuid: String,
    #[serde(default)]
    pub unit: Option<String>,
    /// AI training opt-in. Engine default is `false`; absent in Spaces authored
    /// before the flag existed — see [`AiExportPolicy`].
    #[serde(default)]
    pub ai: bool,
    /// Authored display color, sRGB 0-255.
    #[serde(default)]
    pub roblox_color_srgb: Option<[f32; 3]>,
}

/// `[transform]` — position is the load-bearing spatial signal.
#[derive(Debug, Clone, Deserialize)]
pub struct TransformData {
    pub position: [f32; 3],
    #[serde(default)]
    pub rotation: Option<Vec<f32>>,
    #[serde(default)]
    pub scale: Option<[f32; 3]>,
}

/// `[properties]` — the subset Vortex reasons over today.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct InstanceProperties {
    #[serde(default)]
    pub material: Option<String>,
    #[serde(default)]
    pub transparency: Option<f32>,
    #[serde(default)]
    pub anchored: Option<bool>,
    /// Linear RGBA 0-1.
    #[serde(default)]
    pub color: Option<Vec<f32>>,
    /// `[properties.attributes]` — open-ended author metadata.
    #[serde(default)]
    pub attributes: HashMap<String, toml::Value>,
}

/// How to treat the engine's `ai` training opt-in flag.
///
/// The engine's default is `false`, and Spaces authored before the flag existed
/// omit it entirely — so a strict reading of a legacy Space yields ZERO eligible
/// entities. Rather than silently choose, the caller states a policy and the
/// loader reports exactly what it did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiExportPolicy {
    /// Only `ai = true`. Correct for Spaces authored after the flag existed, and
    /// the only policy that honors the contract with no interpretation.
    StrictOptIn,
    /// `ai = true`, plus instances where the field is ABSENT (legacy authoring),
    /// on the owner's explicit instruction. An absent field is treated as
    /// "predates the flag", never as an explicit `false`.
    LegacyOwnerConsent,
    /// Include everything, including explicit `ai = false`. An override for local
    /// analysis only — it disregards a decision the author actually recorded.
    IncludeAll,
}

impl AiExportPolicy {
    fn admits(self, m: &InstanceMetadata, ai_present: bool) -> bool {
        match self {
            AiExportPolicy::StrictOptIn => m.ai,
            AiExportPolicy::LegacyOwnerConsent => m.ai || !ai_present,
            AiExportPolicy::IncludeAll => true,
        }
    }
}

/// What a load actually did — so the governance decision is auditable, not implied.
#[derive(Debug, Clone, Default)]
pub struct LoadReport {
    pub files_scanned: usize,
    pub parse_failures: usize,
    pub admitted: usize,
    pub excluded_opt_out: usize,
    pub excluded_no_transform: usize,
    /// Admitted only because the `ai` field was absent (legacy authoring).
    pub admitted_legacy_absent: usize,
    pub explicit_opt_in: usize,
    pub reference_edges: usize,
    /// Admitted with the label taken from the containing directory because the
    /// `[metadata] name` key was absent. Reported so a corpus that is mostly
    /// folder-named is visible rather than assumed.
    pub name_from_directory: usize,
}

impl LoadReport {
    /// One-line audit string suitable for logs or a provenance record.
    pub fn summary(&self, policy: AiExportPolicy) -> String {
        format!(
            "eustress_schema: policy={:?} scanned={} admitted={} (opt_in={}, legacy_absent={}) \
             excluded(opt_out={}, no_transform={}) parse_failures={} reference_edges={}",
            policy,
            self.files_scanned,
            self.admitted,
            self.explicit_opt_in,
            self.admitted_legacy_absent,
            self.excluded_opt_out,
            self.excluded_no_transform,
            self.parse_failures,
            self.reference_edges,
        )
    }
}

/// A loaded Space: engine-native instances rendered as walkable contexts.
#[derive(Debug, Clone, Default)]
pub struct LoadedSpace {
    pub nodes: Vec<NodeContext>,
    /// Parallel to `nodes`: the authored containment parent (directory path
    /// relative to the Space root) — the engine's real hierarchy.
    pub parents: Vec<String>,
    /// Parallel to `nodes`: `class_name` from the engine's taxonomy.
    pub classes: Vec<String>,
    /// Parallel to `nodes`: stable engine UUID.
    pub uuids: Vec<String>,
    pub report: LoadReport,
}

fn srgb_to_oklch_approx(rgb: [f32; 3]) -> Oklch {
    // Cheap perceptual stand-in: luminance + chroma magnitude + hue angle.
    // Not a colorimetric OKLCH conversion — named `approx` so no caller mistakes
    // it for one. Real conversion belongs in the engine's color module.
    let (r, g, b) = (rgb[0] / 255.0, rgb[1] / 255.0, rgb[2] / 255.0);
    let l = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let c = max - min;
    let h = if c < 1e-6 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / c) % 6.0)
    } else if max == g {
        60.0 * ((b - r) / c + 2.0)
    } else {
        60.0 * ((r - g) / c + 4.0)
    };
    Oklch { l, c, h: if h < 0.0 { h + 360.0 } else { h } }
}

/// Load a Space by walking its directory for `_instance.toml` files — the same
/// on-disk layout the engine and its headless CLI operate on.
///
/// `space_root` is typically `<Space>/Workspace`.
pub fn load_space(space_root: &Path, policy: AiExportPolicy) -> std::io::Result<LoadedSpace> {
    let mut out = LoadedSpace::default();
    let mut stack = vec![space_root.to_path_buf()];
    let mut next_id: NodeId = 0;

    while let Some(dir) = stack.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue, // unreadable subtree: skip, never abort the load
        };
        let mut instance_file: Option<PathBuf> = None;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.file_name().and_then(|s| s.to_str()) == Some("_instance.toml") {
                instance_file = Some(path);
            }
        }
        let Some(path) = instance_file else { continue };
        out.report.files_scanned += 1;

        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(_) => {
                out.report.parse_failures += 1;
                continue;
            }
        };
        // `ai` presence must be detected BEFORE deserialization, because
        // `#[serde(default)]` makes an absent field indistinguishable from
        // `ai = false` afterwards — and that difference is the whole policy.
        let ai_present = text
            .lines()
            .any(|l| l.trim_start().starts_with("ai ") || l.trim_start().starts_with("ai="));

        let mut file: InstanceFile = match toml::from_str(&text) {
            Ok(f) => f,
            Err(_) => {
                out.report.parse_failures += 1;
                continue;
            }
        };

        // Backfill identity from the directory. The engine treats the folder as
        // the instance's name, so this is the authoritative source when the
        // convenience key is absent — not a guess.
        if file.metadata.name.is_empty() {
            if let Some(dir_name) = dir.file_name().and_then(|s| s.to_str()) {
                file.metadata.name = dir_name.to_string();
                out.report.name_from_directory += 1;
            }
        }

        if !policy.admits(&file.metadata, ai_present) {
            out.report.excluded_opt_out += 1;
            continue;
        }
        let Some(tf) = file.transform.as_ref() else {
            out.report.excluded_no_transform += 1;
            continue;
        };

        if file.metadata.ai {
            out.report.explicit_opt_in += 1;
        } else if !ai_present {
            out.report.admitted_legacy_absent += 1;
        }
        out.report.reference_edges += file.references.len();

        let color = file
            .metadata
            .roblox_color_srgb
            .map(srgb_to_oklch_approx)
            .unwrap_or_default();

        out.nodes.push(NodeContext {
            id: next_id,
            semantic_text: file.metadata.name.clone(),
            position: SpatialPos { x: tf.position[0], y: tf.position[1], z: tf.position[2] },
            ontology_class: file.metadata.class_name.clone(),
            color,
            ..Default::default()
        });
        out.parents.push(
            dir.parent()
                .and_then(|p| p.strip_prefix(space_root).ok())
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or_default(),
        );
        out.classes.push(file.metadata.class_name.clone());
        out.uuids.push(file.metadata.uuid.clone());
        next_id += 1;
        out.report.admitted += 1;
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const OPTED_IN: &str = r#"
[metadata]
name = "Understanding"
class_name = "Part"
archivable = true
uuid = "abc123"
ai = true
roblox_color_srgb = [255, 0, 0]

[transform]
position = [1.0, 2.0, 3.0]
scale = [1.0, 1.0, 1.0]

[properties]
material = "Plastic"
"#;

    const OPTED_OUT: &str = r#"
[metadata]
name = "Private"
class_name = "Part"
ai = false

[transform]
position = [4.0, 5.0, 6.0]
"#;

    // Authored before the `ai` flag existed — the Mountain Ascension case.
    const LEGACY_ABSENT: &str = r#"
[metadata]
name = "Legacy"
class_name = "Part"
archivable = true
uuid = "legacy1"

[transform]
position = [7.0, 8.0, 9.0]
"#;

    /// `tag` MUST be unique per test: Rust runs tests in parallel, so two tests
    /// sharing a temp dir would delete each other's fixtures mid-run.
    fn space(tag: &str, files: &[(&str, &str)]) -> (tempdir::TempDirShim, PathBuf) {
        let base = std::env::temp_dir().join(format!("eustress_schema_test_{tag}"));
        let _ = std::fs::remove_dir_all(&base);
        for (name, body) in files {
            let d = base.join(name);
            std::fs::create_dir_all(&d).unwrap();
            std::fs::write(d.join("_instance.toml"), body).unwrap();
        }
        (tempdir::TempDirShim(base.clone()), base)
    }

    // Minimal scoped cleanup guard (aimodel has no tempfile dep in normal deps).
    mod tempdir {
        pub struct TempDirShim(pub std::path::PathBuf);
        impl Drop for TempDirShim {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
    }

    #[test]
    fn parses_the_engines_real_instance_schema() {
        let f: InstanceFile = toml::from_str(OPTED_IN).unwrap();
        assert_eq!(f.metadata.name, "Understanding");
        assert_eq!(f.metadata.class_name, "Part");
        assert!(f.metadata.ai);
        assert!(f.metadata.archivable);
        assert_eq!(f.transform.unwrap().position, [1.0, 2.0, 3.0]);
        assert_eq!(f.properties.unwrap().material.as_deref(), Some("Plastic"));
    }

    #[test]
    fn strict_opt_in_admits_only_flagged_instances() {
        let (_g, root) = space("strict", &[("a", OPTED_IN), ("b", OPTED_OUT), ("c", LEGACY_ABSENT)]);
        let s = load_space(&root, AiExportPolicy::StrictOptIn).unwrap();
        assert_eq!(s.report.admitted, 1, "only ai=true may pass StrictOptIn");
        assert_eq!(s.nodes[0].semantic_text, "Understanding");
        assert_eq!(s.report.explicit_opt_in, 1);
    }

    #[test]
    fn legacy_consent_admits_absent_but_never_explicit_opt_out() {
        let (_g, root) = space("legacy", &[("a", OPTED_IN), ("b", OPTED_OUT), ("c", LEGACY_ABSENT)]);
        let s = load_space(&root, AiExportPolicy::LegacyOwnerConsent).unwrap();
        assert_eq!(s.report.admitted, 2, "opted-in + legacy-absent, NOT the opt-out");
        assert_eq!(s.report.admitted_legacy_absent, 1);
        assert!(
            !s.nodes.iter().any(|n| n.semantic_text == "Private"),
            "an explicit ai=false must never be admitted by LegacyOwnerConsent"
        );
    }

    #[test]
    fn report_is_auditable() {
        let (_g, root) = space("report", &[("a", OPTED_IN), ("b", OPTED_OUT)]);
        let s = load_space(&root, AiExportPolicy::StrictOptIn).unwrap();
        let line = s.report.summary(AiExportPolicy::StrictOptIn);
        assert!(line.contains("policy=StrictOptIn"));
        assert!(line.contains("admitted=1"));
    }

    #[test]
    fn nodes_carry_engine_class_and_are_walk_ready() {
        let (_g, root) = space("walkready", &[("a", OPTED_IN)]);
        let s = load_space(&root, AiExportPolicy::StrictOptIn).unwrap();
        // Engine taxonomy survives into the reasoning layer (ontology head).
        assert_eq!(s.classes[0], "Part");
        assert_eq!(s.nodes[0].ontology_class, "Part");
        // Authored color reaches the color head.
        assert!(s.nodes[0].color.l > 0.0);
        // And the position is the real transform, ready for the spatial head.
        assert_eq!(s.nodes[0].position.x, 1.0);
    }
}
