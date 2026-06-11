//! Workflow compiler — verified strategies become engine-native structure.
//!
//! A winning action sequence is knowledge the agent paid hundreds of
//! actions to earn, but as a raw `Vec<String>` it is opaque: nothing in
//! the engine can inspect it, no human can audit it, and the agent itself
//! cannot tell which steps rest on Laws and which on guesswork. This
//! module compiles a recipe plus the epistemic ledger into a workflow
//! DOCUMENT: nodes are intents (one per action), each annotated with the
//! causal claim it relies on and that claim's per-claim IRA; edges encode
//! the execution order. The document lands in the game Space at
//! `Workflows/<name>.workflow.toml`, so EustressEngine panels render it,
//! a human supervisor can read or edit it, and the agent can reload and
//! replay it next session — knowledge as structure, not as a buried blob
//! (the 2GB-knowledge-file lesson again).
//!
//! Serialization follows the space_writer convention: hand-formatted TOML
//! for humans (the crate deliberately carries no `toml` dependency), plus
//! a JSON sidecar via serde_json for a lossless machine round-trip. TOML
//! is the inspection surface; JSON is the truth the agent reloads.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::epistemic::EpistemicLedger;

/// One compiled strategy: a replayable chain of intents with provenance.
///
/// `replays`/`successes` make the document self-evaluating — every replay
/// updates the counters, so a workflow whose success rate decays is
/// visibly stale in the Explorer panel before it ever misleads the agent.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct WorkflowDoc {
    pub name: String,
    pub game_id: String,
    /// Step counter at compile time — orders workflows within a session.
    pub created_step: u32,
    pub nodes: Vec<WfNode>,
    pub edges: Vec<WfEdge>,
    /// Times this workflow has been replayed.
    pub replays: u32,
    /// Times a replay reached its goal.
    pub successes: u32,
}

/// One intent in the chain: the action to emit, and WHY we trust it —
/// the ledger claim it rests on and that claim's representation accuracy.
/// An empty `claim_id` with ira 0.5 means "no causal backing, prior only";
/// replay logic can demand confirmation before crossing such a node.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct WfNode {
    pub id: u32,
    /// Action string in agent vocabulary: `"6:x,y"` for clicks, `"2"` etc.
    /// for simple actions — same format `decide()` returns.
    pub action: String,
    /// Ledger claim id backing this step (e.g. `click(3,7)->d(2,5:0->4)x1`),
    /// or empty when the ledger holds nothing for this action.
    pub claim_id: String,
    /// Per-claim IRA at compile time — how much this step deserved trust
    /// when the workflow was frozen.
    pub ira: f32,
}

/// Execution-order edge. Today compile() emits a linear chain; the edge
/// list exists so branching/merging workflows (retry arms, conditional
/// steps) stay representable without a format change.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct WfEdge {
    pub from: u32,
    pub to: u32,
}

/// Compile a winning action sequence into a workflow document.
///
/// Each click action `"6:x,y"` is matched against the ledger: any claim
/// whose id starts with `click(x,y)` backs that node (when several effects
/// were recorded for the same cell, the highest-IRA claim wins — we cite
/// the strongest evidence we have). Non-click actions, and clicks the
/// ledger never saw, get an empty claim with the uninformed prior 0.5 so
/// downstream consumers can tell "verified" from "it merely worked once".
pub fn compile(
    name: &str,
    game_id: &str,
    step: u32,
    actions: &[String],
    ledger: &EpistemicLedger,
) -> WorkflowDoc {
    let nodes: Vec<WfNode> = actions
        .iter()
        .enumerate()
        .map(|(i, action)| {
            let (claim_id, ira) = lookup_claim(action, ledger);
            WfNode {
                id: i as u32,
                action: action.clone(),
                claim_id,
                ira,
            }
        })
        .collect();

    // Linear chain: node i enables node i+1.
    let edges: Vec<WfEdge> = (1..nodes.len() as u32)
        .map(|to| WfEdge { from: to - 1, to })
        .collect();

    WorkflowDoc {
        name: name.to_string(),
        game_id: game_id.to_string(),
        created_step: step,
        nodes,
        edges,
        replays: 0,
        successes: 0,
    }
}

/// Find the ledger claim backing an action, if any.
///
/// Click claim ids embed the cell — `click(x,y)->effect` — so a prefix
/// match on `click(x,y)` (closing paren included, so `(3,7)` never
/// matches `(3,77)`) finds every recorded effect for that cell. Ties go
/// to the highest IRA, then lexicographic id for determinism.
fn lookup_claim(action: &str, ledger: &EpistemicLedger) -> (String, f32) {
    let Some(rest) = action.strip_prefix("6:") else {
        return (String::new(), 0.5);
    };
    let coords: Vec<&str> = rest.split(',').collect();
    if coords.len() != 2 {
        return (String::new(), 0.5);
    }
    let prefix = format!("click({},{})", coords[0].trim(), coords[1].trim());

    let mut best: Option<(&String, f32)> = None;
    for (id, claim) in &ledger.claims {
        if !id.starts_with(&prefix) {
            continue;
        }
        let better = match best {
            None => true,
            Some((best_id, best_ira)) => {
                claim.ira > best_ira || (claim.ira == best_ira && id.as_str() < best_id.as_str())
            }
        };
        if better {
            best = Some((id, claim.ira));
        }
    }
    match best {
        Some((id, ira)) => (id.clone(), ira),
        None => (String::new(), 0.5),
    }
}

/// Write the workflow into the Space at `Workflows/<name>.workflow.toml`,
/// plus a `.workflow.json` sidecar for lossless reload.
///
/// Returns the TOML path (the human-facing artifact). The filename is
/// sanitized so a name like `"win level 3 (fast)"` cannot escape the
/// Workflows directory or trip up the Explorer's file watcher.
pub fn write_to_space(space_dir: &Path, doc: &WorkflowDoc) -> Result<PathBuf, String> {
    let wf_dir = space_dir.join("Workflows");
    std::fs::create_dir_all(&wf_dir)
        .map_err(|e| format!("Failed to create {}: {}", wf_dir.display(), e))?;

    let stem = sanitize_name(&doc.name);
    let toml_path = wf_dir.join(format!("{}.workflow.toml", stem));
    let json_path = wf_dir.join(format!("{}.workflow.json", stem));

    std::fs::write(&toml_path, workflow_toml(doc))
        .map_err(|e| format!("Failed to write {}: {}", toml_path.display(), e))?;

    let json = serde_json::to_string_pretty(doc)
        .map_err(|e| format!("Failed to serialize workflow JSON: {}", e))?;
    std::fs::write(&json_path, json)
        .map_err(|e| format!("Failed to write {}: {}", json_path.display(), e))?;

    info!(
        "Wrote workflow '{}' ({} nodes) to {}",
        doc.name,
        doc.nodes.len(),
        toml_path.display()
    );
    Ok(toml_path)
}

/// Reload every workflow stored in the Space. Best-effort by design: a
/// human may have hand-edited or half-deleted files, and one corrupt
/// document must not cost the agent the rest of its compiled knowledge.
/// Reads the JSON sidecars (the lossless channel); the TOML twin is for
/// eyes, not parsers. Sorted by name so replay selection is deterministic.
pub fn load_from_space(space_dir: &Path) -> Vec<WorkflowDoc> {
    let wf_dir = space_dir.join("Workflows");
    let Ok(entries) = std::fs::read_dir(&wf_dir) else {
        return Vec::new();
    };

    let mut docs = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let is_sidecar = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.ends_with(".workflow.json"))
            .unwrap_or(false);
        if !is_sidecar {
            continue;
        }
        match std::fs::read_to_string(&path) {
            Ok(text) => match serde_json::from_str::<WorkflowDoc>(&text) {
                Ok(doc) => docs.push(doc),
                Err(e) => debug!("Skipping unparseable workflow {}: {}", path.display(), e),
            },
            Err(e) => debug!("Skipping unreadable workflow {}: {}", path.display(), e),
        }
    }
    docs.sort_by(|a, b| a.name.cmp(&b.name));
    docs
}

// ─────────────────────────────────────────────────────────────────────────────
// Serialization helpers (space_writer.rs convention: format!-built TOML)
// ─────────────────────────────────────────────────────────────────────────────

/// Reduce a workflow name to a filesystem-safe stem: alphanumerics, `-`
/// and `_` pass through, everything else becomes `_`.
fn sanitize_name(name: &str) -> String {
    let stem: String = name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();
    if stem.is_empty() {
        "unnamed".to_string()
    } else {
        stem
    }
}

/// Escape a string for a TOML basic ("…") string.
fn toml_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Hand-format the human-facing TOML document.
fn workflow_toml(doc: &WorkflowDoc) -> String {
    let mut content = format!(
        "# Workflow — compiled verified strategy\n\
         # Auto-generated by eustress-arc-policy; edit freely, the agent\n\
         # reloads from the .workflow.json sidecar.\n\n\
         [workflow]\n\
         name = \"{name}\"\n\
         game_id = \"{game}\"\n\
         created_step = {step}\n\
         replays = {replays}\n\
         successes = {successes}\n\
         node_count = {nc}\n\n",
        name = toml_escape(&doc.name),
        game = toml_escape(&doc.game_id),
        step = doc.created_step,
        replays = doc.replays,
        successes = doc.successes,
        nc = doc.nodes.len(),
    );

    for node in &doc.nodes {
        content.push_str(&format!(
            "[[node]]\nid = {}\naction = \"{}\"\nclaim_id = \"{}\"\nira = {:.4}\n\n",
            node.id,
            toml_escape(&node.action),
            toml_escape(&node.claim_id),
            node.ira,
        ));
    }

    for edge in &doc.edges {
        content.push_str(&format!("[[edge]]\nfrom = {}\nto = {}\n\n", edge.from, edge.to));
    }

    content
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::epistemic::{CausalClaim, ClaimStatus};

    /// Ledger with one Law-grade click claim and one weaker rival on the
    /// same cell, mirroring real ids from world_model's click learning.
    fn synthetic_ledger() -> EpistemicLedger {
        let mut ledger = EpistemicLedger::default();
        ledger.claims.insert(
            "click(3,7)->d(2,5:0->4)x1".to_string(),
            CausalClaim {
                status: ClaimStatus::Law,
                observations: 8,
                verifications: 6,
                contradictions: 0,
                ira: 0.92,
            },
        );
        ledger.claims.insert(
            "click(3,7)->none".to_string(),
            CausalClaim {
                status: ClaimStatus::Observed,
                observations: 1,
                verifications: 0,
                contradictions: 1,
                ira: 0.35,
            },
        );
        ledger
    }

    fn actions(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn compile_builds_linear_chain_with_claim_provenance() {
        let ledger = synthetic_ledger();
        let acts = actions(&["6:3,7", "2", "6:9,9"]);
        let doc = compile("win_l1", "tn36", 412, &acts, &ledger);

        assert_eq!(doc.name, "win_l1");
        assert_eq!(doc.game_id, "tn36");
        assert_eq!(doc.created_step, 412);
        assert_eq!(doc.nodes.len(), 3);
        assert_eq!(doc.replays, 0);
        assert_eq!(doc.successes, 0);

        // Click with ledger backing: highest-IRA claim cited.
        assert_eq!(doc.nodes[0].action, "6:3,7");
        assert_eq!(doc.nodes[0].claim_id, "click(3,7)->d(2,5:0->4)x1");
        assert!((doc.nodes[0].ira - 0.92).abs() < 1e-6);

        // Simple action: no causal backing, uninformed prior.
        assert_eq!(doc.nodes[1].action, "2");
        assert_eq!(doc.nodes[1].claim_id, "");
        assert!((doc.nodes[1].ira - 0.5).abs() < 1e-6);

        // Click the ledger never saw: also the prior.
        assert_eq!(doc.nodes[2].claim_id, "");
        assert!((doc.nodes[2].ira - 0.5).abs() < 1e-6);

        // Linear chain edges 0→1→2.
        assert_eq!(
            doc.edges,
            vec![WfEdge { from: 0, to: 1 }, WfEdge { from: 1, to: 2 }]
        );
    }

    #[test]
    fn compile_prefers_strongest_claim_and_respects_cell_boundaries() {
        let mut ledger = synthetic_ledger();
        // A claim on a DIFFERENT cell whose textual id shares the digits:
        // click(3,77) must not back action "6:3,7".
        ledger.claims.insert(
            "click(3,77)->d(0,0:1->2)x1".to_string(),
            CausalClaim {
                status: ClaimStatus::Law,
                observations: 9,
                verifications: 9,
                contradictions: 0,
                ira: 0.99,
            },
        );
        let doc = compile("x", "g", 0, &actions(&["6:3,7"]), &ledger);
        assert_eq!(doc.nodes[0].claim_id, "click(3,7)->d(2,5:0->4)x1");
        assert!((doc.nodes[0].ira - 0.92).abs() < 1e-6);
    }

    #[test]
    fn compile_empty_recipe_yields_no_nodes_or_edges() {
        let doc = compile("empty", "g", 1, &[], &EpistemicLedger::default());
        assert!(doc.nodes.is_empty());
        assert!(doc.edges.is_empty());
    }

    #[test]
    fn write_then_load_round_trips() {
        let ledger = synthetic_ledger();
        let acts = actions(&["6:3,7", "2", "6:3,7"]);
        let doc = compile("win level 1 (fast)", "tn36", 99, &acts, &ledger);

        let space_dir = std::env::temp_dir().join(format!(
            "arc_wf_roundtrip_{}_{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&space_dir);

        let toml_path = write_to_space(&space_dir, &doc).expect("write_to_space failed");

        // Human artifact exists, name sanitized, content mentions the claim.
        assert!(toml_path.ends_with("win_level_1__fast_.workflow.toml"));
        let toml_text = std::fs::read_to_string(&toml_path).unwrap();
        assert!(toml_text.contains("name = \"win level 1 (fast)\""));
        assert!(toml_text.contains("claim_id = \"click(3,7)->d(2,5:0->4)x1\""));
        assert!(toml_text.contains("[[edge]]"));

        // Machine round-trip via the JSON sidecar.
        let loaded = load_from_space(&space_dir);
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, doc.name);
        assert_eq!(loaded[0].nodes.len(), doc.nodes.len());
        let loaded_actions: Vec<&str> =
            loaded[0].nodes.iter().map(|n| n.action.as_str()).collect();
        assert_eq!(loaded_actions, vec!["6:3,7", "2", "6:3,7"]);
        assert_eq!(loaded[0], doc); // full structural equality

        let _ = std::fs::remove_dir_all(&space_dir);
    }

    #[test]
    fn load_skips_corrupt_sidecars_and_sorts_by_name() {
        let ledger = EpistemicLedger::default();
        let space_dir = std::env::temp_dir().join(format!(
            "arc_wf_corrupt_{}_{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&space_dir);

        let b = compile("bbb", "g", 1, &actions(&["1"]), &ledger);
        let a = compile("aaa", "g", 2, &actions(&["2"]), &ledger);
        write_to_space(&space_dir, &b).unwrap();
        write_to_space(&space_dir, &a).unwrap();
        // A hand-mangled sidecar must not poison the load.
        std::fs::write(
            space_dir.join("Workflows").join("broken.workflow.json"),
            "{ not json",
        )
        .unwrap();

        let loaded = load_from_space(&space_dir);
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].name, "aaa");
        assert_eq!(loaded[1].name, "bbb");

        let _ = std::fs::remove_dir_all(&space_dir);
    }

    #[test]
    fn load_from_missing_space_is_empty_not_error() {
        let ghost = std::env::temp_dir().join("arc_wf_does_not_exist_xyzzy");
        assert!(load_from_space(&ghost).is_empty());
    }

    #[test]
    fn sanitize_handles_hostile_names() {
        assert_eq!(sanitize_name("win level 3"), "win_level_3");
        assert_eq!(sanitize_name("../../etc/passwd"), "______etc_passwd");
        assert_eq!(sanitize_name(""), "unnamed");
        assert_eq!(sanitize_name("ok-name_1"), "ok-name_1");
    }
}
