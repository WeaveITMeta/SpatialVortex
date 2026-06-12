//! Mind-map export — the agent's cognitive state rendered as
//! engine-native `Workspace/MindMap` instances inside a game's Space.
//!
//! Claims, win predicates, hazards, and recipes become labeled,
//! colored nodes; epistemic strength maps to transparency (weak
//! belief = see-through); hazards sit at their actual grid positions.
//! The engine's mind-map tooling (drag-snap, gizmos) then gives live
//! spatial debugging of WHY the agent believes what it believes.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;

/// One node in the exported mind map.
#[derive(Clone, Debug)]
pub struct MindNode {
    /// Folder-safe unique name.
    pub name: String,
    /// Human-readable label text shown on the node.
    pub label: String,
    pub color: (u8, u8, u8),
    /// 0.0 opaque (strong belief) … 0.9 nearly invisible (weak).
    pub transparency: f32,
    pub position: (f32, f32, f32),
    /// EEP tags carried on the instance (classification gizmos key off these).
    pub tags: Vec<String>,
}

fn sanitize(name: &str) -> String {
    let mut s: String = name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    s.truncate(48);
    s
}

fn det_uuid(name: &str) -> String {
    let mut h = DefaultHasher::new();
    name.hash(&mut h);
    let a = h.finish();
    name.len().hash(&mut h);
    let b = h.finish();
    format!("{:016x}{:016x}", a, b)
}

fn write_file(path: &Path, content: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("mindmap mkdir {}: {}", parent.display(), e))?;
    }
    std::fs::write(path, content).map_err(|e| format!("mindmap write {}: {}", path.display(), e))
}

fn write_node(map_dir: &Path, node: &MindNode, now: &str) -> Result<(), String> {
    let dir = map_dir.join(sanitize(&node.name));
    let tags = node
        .tags
        .iter()
        .map(|t| format!("    \"{}\",\n", t))
        .collect::<String>();
    let instance = format!(
        r#"tags = [
{tags}]

[asset]
mesh = "parts/ball.glb"
scene = "Scene0"

[metadata]
archivable = true
class_name = "Part"
created = "{now}"
last_modified = "{now}"
uuid = "{uuid}"

[properties]
anchored = true
can_collide = false
cast_shadow = false
color = [{r}, {g}, {b}]
locked = false
material = "Neon"
reflectance = 0.0
transparency = {tr}

[transform]
position = [{x}, {y}, {z}]
rotation = [0.0, 0.0, 0.0, 1.0]
scale = [2.0, 2.0, 2.0]
"#,
        tags = tags,
        now = now,
        uuid = det_uuid(&node.name),
        r = node.color.0,
        g = node.color.1,
        b = node.color.2,
        tr = node.transparency,
        x = node.position.0,
        y = node.position.1,
        z = node.position.2,
    );
    write_file(&dir.join("_instance.toml"), &instance)?;

    let label_dir = dir.join("Label");
    let label_instance = format!(
        r#"[metadata]
class_name = "BillboardGui"
archivable = true
created = "{now}"
uuid = "{uuid}"
"#,
        now = now,
        uuid = det_uuid(&format!("{}__label", node.name)),
    );
    write_file(&label_dir.join("_instance.toml"), &label_instance)?;

    let text = node.label.replace('"', "'");
    let label = format!(
        r#"[metadata]
class_name = "TextLabel"
archivable = true

[gui]
anchor_point = [0.0, 0.5]
position = [0.0, 0.0, 0.5, 0.0]
size = [1.0, 0.0, 0.25, 0.0]
background_color = [0.08, 0.09, 0.14, 0.78]
border_size = 0.0
visible = true
z_index = 8

[text]
text = "{text}"
text_color = [1.0, 1.0, 1.0, 1.0]
font_size = 20
font = "GothamBold"
text_scaled = true
text_x_alignment = "center"
text_y_alignment = "center"
"#,
        text = text,
    );
    write_file(&label_dir.join("Label.textlabel.toml"), &label)
}

/// Quaternion rotating the unit Y axis onto `dir` (normalized).
fn quat_y_to(dir: (f32, f32, f32)) -> (f32, f32, f32, f32) {
    let (dx, dy, dz) = dir;
    // axis = Y × dir, angle = acos(Y · dir)
    let dot = dy.clamp(-1.0, 1.0);
    if dot > 0.9999 {
        return (0.0, 0.0, 0.0, 1.0);
    }
    if dot < -0.9999 {
        return (0.0, 0.0, 1.0, 0.0); // 180° about Z
    }
    let (ax, ay, az) = (dz, 0.0, -dx); // cross((0,1,0), dir)
    let alen = (ax * ax + ay * ay + az * az).sqrt().max(1e-6);
    let half = dot.acos() / 2.0;
    let s = half.sin() / alen;
    (ax * s, ay * s, az * s, half.cos())
}

fn write_edge(
    map_dir: &Path,
    idx: usize,
    from: &MindNode,
    to: &MindNode,
    now: &str,
) -> Result<(), String> {
    let name = format!(
        "Edge_{:02}_{}_{}",
        idx,
        sanitize(&from.name),
        sanitize(&to.name)
    );
    let (x0, y0, z0) = from.position;
    let (x1, y1, z1) = to.position;
    let (dx, dy, dz) = (x1 - x0, y1 - y0, z1 - z0);
    let len = (dx * dx + dy * dy + dz * dz).sqrt().max(0.01);
    let dir = (dx / len, dy / len, dz / len);
    let (qx, qy, qz, qw) = quat_y_to(dir);
    let instance = format!(
        r#"[asset]
mesh = "parts/cylinder.glb"
scene = "Scene0"

[metadata]
archivable = true
class_name = "Part"
created = "{now}"
last_modified = "{now}"
uuid = "{uuid}"

[properties]
anchored = true
can_collide = false
cast_shadow = false
color = [120, 150, 190]
locked = false
material = "SmoothPlastic"
reflectance = 0.0
transparency = 0.4

[transform]
position = [{mx}, {my}, {mz}]
rotation = [{qx}, {qy}, {qz}, {qw}]
scale = [0.25, {half_len}, 0.25]
"#,
        now = now,
        uuid = det_uuid(&name),
        mx = (x0 + x1) / 2.0,
        my = (y0 + y1) / 2.0,
        mz = (z0 + z1) / 2.0,
        qx = qx,
        qy = qy,
        qz = qz,
        qw = qw,
        half_len = len / 2.0,
    );
    let dir_path = map_dir.join(&name);
    write_file(&dir_path.join("_instance.toml"), &instance)
}

/// Write a full mind map under `<space_dir>/Workspace/MindMap`.
/// `edges` are (from_index, to_index) into `nodes`. The directory is
/// replaced wholesale so retired claims disappear instead of lingering.
pub fn write_mindmap(
    space_dir: &Path,
    nodes: &[MindNode],
    edges: &[(usize, usize)],
    now: &str,
) -> Result<(), String> {
    let map_dir = space_dir.join("Workspace/MindMap");
    if map_dir.exists() {
        std::fs::remove_dir_all(&map_dir)
            .map_err(|e| format!("mindmap clear {}: {}", map_dir.display(), e))?;
    }
    std::fs::create_dir_all(&map_dir)
        .map_err(|e| format!("mindmap mkdir {}: {}", map_dir.display(), e))?;

    for node in nodes {
        write_node(&map_dir, node, now)?;
    }
    for (i, (a, b)) in edges.iter().enumerate() {
        if let (Some(from), Some(to)) = (nodes.get(*a), nodes.get(*b)) {
            write_edge(&map_dir, i, from, to, now)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(name: &str, pos: (f32, f32, f32)) -> MindNode {
        MindNode {
            name: name.into(),
            label: name.into(),
            color: (200, 100, 50),
            transparency: 0.2,
            position: pos,
            tags: vec!["claim".into()],
        }
    }

    #[test]
    fn writes_nodes_edges_and_replaces() {
        let tmp = std::env::temp_dir().join(format!("mindmap_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        let nodes = vec![node("Center", (0.0, 25.0, 0.0)), node("click(3,4)", (10.0, 25.0, 0.0))];
        write_mindmap(&tmp, &nodes, &[(0, 1)], "2026-06-12T00:00:00Z").unwrap();
        assert!(tmp.join("Workspace/MindMap/Center/_instance.toml").exists());
        assert!(tmp.join("Workspace/MindMap/click_3_4_/Label/Label.textlabel.toml").exists());
        let edge_dir = std::fs::read_dir(tmp.join("Workspace/MindMap"))
            .unwrap()
            .filter_map(|e| e.ok())
            .find(|e| e.file_name().to_string_lossy().starts_with("Edge_00"));
        assert!(edge_dir.is_some());
        // Re-export with one node: the old claim node must disappear.
        write_mindmap(&tmp, &nodes[..1], &[], "2026-06-12T00:00:01Z").unwrap();
        assert!(!tmp.join("Workspace/MindMap/click_3_4_").exists());
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn quaternion_unit_cases() {
        let (x, y, z, w) = quat_y_to((0.0, 1.0, 0.0));
        assert!((w - 1.0).abs() < 1e-5 && x.abs() + y.abs() + z.abs() < 1e-5);
        let (_, _, _, w) = quat_y_to((1.0, 0.0, 0.0));
        assert!((w - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-4);
    }
}
