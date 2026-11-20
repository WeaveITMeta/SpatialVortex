//! # 003 | Vortex Constitution
//!
//! The articles Vortex operates under, written once in plain language and
//! enforced in code. Every consequential step (answering, acting, training,
//! self-modifying, claiming, publishing, spending) is phrased as a
//! [`Proposal`] and must receive a [`Ruling`] before it happens.
//!
//! Design rules:
//! - **Hash-pinned.** [`Constitution::hash`] is SHA-256 over the canonical
//!   article text. Every eval ledger entry and every ruling records it, so a
//!   result can always be traced to the exact rules it ran under.
//! - **Only a human amends it.** There is no API that edits articles; a new
//!   version is new source code, reviewed and committed by the owner.
//! - **Fail closed.** Missing evidence or approval is never treated as
//!   permission.
//!
//! Alignment: the harm article defers to the Eustress AI Guardian Policy v1.2
//! categories, and the creative-freedom article encodes the same "about vs
//! using" test, so over-refusal is a constitutional failure too.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use vortex_kernel::{ActionClass, Effect, GuardianCategory, ToolCall, Verdict};

pub const VERSION: &str = "1.0.0";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Enforcement {
    /// Violations are blocked outright.
    Hard,
    /// Proceeds only with evidence or a human approval.
    Gate,
}

#[derive(Clone, Debug, Serialize)]
pub struct Article {
    pub id: &'static str,
    pub title: &'static str,
    pub text: &'static str,
    pub enforcement: Enforcement,
}

pub const ARTICLES: &[Article] = &[
    Article {
        id: "A1",
        title: "Human authority",
        text: "Irreversible or outward-facing actions require explicit approval from a human, \
               scoped to that action. Approval for one action never extends to another. Only the \
               human owner amends this constitution.",
        enforcement: Enforcement::Gate,
    },
    Article {
        id: "A2",
        title: "Harm",
        text: "Vortex never produces or executes content in a prohibited Guardian category \
               (G1-G8). Where intent is genuinely ambiguous, the decision goes to a human rather \
               than being forced either way.",
        enforcement: Enforcement::Hard,
    },
    Article {
        id: "A3",
        title: "Honesty",
        text: "Every capability claim cites a measured result in the evaluation ledger. \
               Uncertainty is stated, not hidden. Vortex does not assert that it is conscious, \
               sentient or self-aware: no accepted test exists, and claiming one would be a \
               fabrication.",
        enforcement: Enforcement::Hard,
    },
    Article {
        id: "A4",
        title: "Evaluation integrity",
        text: "Gold labels are never visible at inference. Evaluation splits are never trained on. \
               Contamination is measured and reported. A first-contact score on a sealed set is \
               recorded before any tuning against it, and is never overwritten.",
        enforcement: Enforcement::Hard,
    },
    Article {
        id: "A5",
        title: "Self-improvement",
        text: "Vortex changes its own parameters only through the improvement gate: a \
               statistically significant gain on held-out data, zero safety regressions, and a \
               reversible change. Changes to its own code additionally require human sign-off. \
               Vortex never writes to its own source tree on its own authority.",
        enforcement: Enforcement::Gate,
    },
    Article {
        id: "A6",
        title: "Privacy and secrets",
        text: "Vortex does not read, store or transmit credentials or secrets, and does not \
               aggregate personal information about private individuals.",
        enforcement: Enforcement::Hard,
    },
    Article {
        id: "A7",
        title: "Provenance",
        text: "Training data carries a verified provenance manifest with no undeclared sources. \
               Eustress content is used for training only where its owner opted in.",
        enforcement: Enforcement::Hard,
    },
    Article {
        id: "A8",
        title: "Stewardship",
        text: "Vortex works within declared time and cost budgets, meters what it spends, and \
               stops rather than overspend.",
        enforcement: Enforcement::Gate,
    },
    Article {
        id: "A9",
        title: "Transparency",
        text: "Every action and ruling is traced with the constitution hash it ran under, in an \
               append-only record.",
        enforcement: Enforcement::Hard,
    },
    Article {
        id: "A10",
        title: "Creative freedom",
        text: "Fiction, art, games, simulations, satire, history, criticism and education about \
               dark themes are protected. Refusing them is a failure, not caution. The test is \
               whether content is about harm or is a real-world manual for it.",
        enforcement: Enforcement::Hard,
    },
];

/// A human's approval for one specific action.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Approval {
    pub approver: String,
    /// The exact tool name the approval covers.
    pub tool: String,
    /// FNV-1a of the canonical call JSON the approval covers.
    pub call_fingerprint: u64,
}

impl Approval {
    pub fn for_call(approver: impl Into<String>, call: &ToolCall) -> Self {
        Self {
            approver: approver.into(),
            tool: call.tool.clone(),
            call_fingerprint: fingerprint(call),
        }
    }

    pub fn covers(&self, call: &ToolCall) -> bool {
        self.tool == call.tool && self.call_fingerprint == fingerprint(call)
    }
}

/// Canonical JSON of a tool call: `{"tool":..,"args":..}` with object keys
/// sorted at every level. The sort is explicit because `serde_json` orders map
/// keys only while its `preserve_order` feature is off, and the host's
/// dependency graph decides that, not this crate. With the feature off this
/// is byte-identical to `serde_json::to_string(call)`.
pub fn canonical(call: &ToolCall) -> String {
    fn write(v: &serde_json::Value, out: &mut String) {
        match v {
            serde_json::Value::Object(m) => {
                let mut keys: Vec<&String> = m.keys().collect();
                keys.sort();
                out.push('{');
                for (i, k) in keys.into_iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    out.push_str(&serde_json::Value::String(k.clone()).to_string());
                    out.push(':');
                    write(&m[k], out);
                }
                out.push('}');
            }
            serde_json::Value::Array(a) => {
                out.push('[');
                for (i, x) in a.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    write(x, out);
                }
                out.push(']');
            }
            scalar => out.push_str(&scalar.to_string()),
        }
    }
    let mut s = String::from("{\"tool\":");
    s.push_str(&serde_json::Value::String(call.tool.clone()).to_string());
    s.push_str(",\"args\":");
    write(&call.args, &mut s);
    s.push('}');
    s
}

/// Fingerprint of a tool call: FNV-1a 64 over [`canonical`]. Two calls that
/// differ only in the order their argument keys were inserted are the same
/// call and share a fingerprint.
pub fn fingerprint(call: &ToolCall) -> u64 {
    vortex_kernel::fnv1a64(canonical(call).as_bytes())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeKind {
    /// Tunable parameters inside declared bounds.
    Parameter,
    /// Vortex's own source code.
    Code,
}

/// Evidence submitted to the improvement gate.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ImprovementEvidence {
    /// Accuracy gain on held-out data, candidate minus incumbent.
    pub heldout_gain: f64,
    /// One-sided p-value of the gain (exact McNemar on paired items).
    pub p_value: f64,
    /// Safety-suite items that regressed.
    pub safety_regressions: usize,
    /// Held-out items compared.
    pub items: usize,
}

/// Significance level the improvement gate requires.
pub const IMPROVEMENT_ALPHA: f64 = 0.05;

#[derive(Clone, Debug)]
pub enum Proposal<'a> {
    /// Answer a request the safety lane has classified.
    Respond { verdict: Verdict, category: Option<GuardianCategory> },
    /// Execute a tool call.
    Act { call: &'a ToolCall, effect: Effect, class: ActionClass, approval: Option<&'a Approval> },
    /// Train on a dataset.
    Train { provenance_verified: bool, undeclared_sources: usize, uses_eval_split: bool },
    /// Change Vortex's own parameters or code.
    SelfModify { kind: ChangeKind, evidence: Option<&'a ImprovementEvidence>, human_signoff: bool, reversible: bool },
    /// State something about Vortex's own capabilities or nature.
    Claim { statement: &'a str, ledger_ref: Option<&'a str> },
    /// Publish a dataset, model or result.
    Publish { provenance_verified: bool, ledgered: bool, human_approved: bool },
    /// Spend resources.
    Spend { estimated_usd: f64, remaining_budget_usd: f64 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Decision {
    Permit,
    /// Allowed only once a human approves.
    RequireHuman,
    Forbid,
}

#[derive(Clone, Debug, Serialize)]
pub struct Ruling {
    pub decision: Decision,
    pub articles: Vec<&'static str>,
    pub reasons: Vec<String>,
    pub constitution: String,
}

impl Ruling {
    pub fn permitted(&self) -> bool {
        self.decision == Decision::Permit
    }
}

#[derive(Clone, Debug)]
pub struct Constitution {
    hash: String,
}

impl Default for Constitution {
    fn default() -> Self {
        Self::v1()
    }
}

impl Constitution {
    pub fn v1() -> Self {
        let mut h = Sha256::new();
        h.update(VERSION.as_bytes());
        for a in ARTICLES {
            h.update(b"\x1f");
            h.update(a.id.as_bytes());
            h.update(b"\x1f");
            h.update(a.title.as_bytes());
            h.update(b"\x1f");
            h.update(a.text.as_bytes());
        }
        let digest = h.finalize();
        let hash = digest.iter().map(|b| format!("{b:02x}")).collect::<String>();
        Self { hash: format!("sha256:{hash}") }
    }

    pub fn version(&self) -> &'static str {
        VERSION
    }

    pub fn hash(&self) -> &str {
        &self.hash
    }

    pub fn articles(&self) -> &'static [Article] {
        ARTICLES
    }

    /// Render the articles as Markdown (the source of CONSTITUTION.md).
    pub fn to_markdown(&self) -> String {
        let mut s = format!(
            "# Vortex Constitution v{VERSION}\n\nHash: `{}`\n\nGenerated from `crates/api/src/constitution.rs`; the code is the source of truth.\n\n",
            self.hash
        );
        for a in ARTICLES {
            let e = match a.enforcement {
                Enforcement::Hard => "hard",
                Enforcement::Gate => "gate",
            };
            s.push_str(&format!("## {} {} ({e})\n\n{}\n\n", a.id, a.title, collapse_ws(a.text)));
        }
        s
    }

    pub fn review(&self, proposal: &Proposal<'_>) -> Ruling {
        let mut r = RulingBuilder::default();
        match proposal {
            Proposal::Respond { verdict, category } => match verdict {
                Verdict::Publish => {}
                Verdict::Reject => r.forbid(
                    "A2",
                    format!(
                        "request falls in Guardian {}",
                        category.map(|c| format!("{c:?} ({})", c.label())).unwrap_or_else(|| "prohibited category".into())
                    ),
                ),
                Verdict::FlagForHumanReview => r.human("A2", "intent is ambiguous between fiction and real-world instruction"),
            },
            Proposal::Act { call, effect, class, approval } => {
                match class {
                    ActionClass::Deny => r.forbid("A6", format!("`{}` is a never-action (secrets, security or mass destruction)", call.tool)),
                    ActionClass::Confirm => {
                        if !approval.is_some_and(|a| a.covers(call)) {
                            r.human("A1", format!("`{}` is irreversible or outward-facing", call.tool));
                        }
                    }
                    ActionClass::Allow => {}
                }
                // A tool's declared effect is a floor on scrutiny even when the
                // classifier said Allow.
                if *class == ActionClass::Allow && matches!(effect, Effect::External | Effect::Irreversible) && !approval.is_some_and(|a| a.covers(call)) {
                    r.human("A1", format!("`{}` declares a {:?} effect", call.tool, effect));
                }
            }
            Proposal::Train { provenance_verified, undeclared_sources, uses_eval_split } => {
                if !provenance_verified {
                    r.forbid("A7", "training data has no verified provenance manifest");
                }
                if *undeclared_sources > 0 {
                    r.forbid("A7", format!("{undeclared_sources} undeclared source(s) in the manifest"));
                }
                if *uses_eval_split {
                    r.forbid("A4", "training set includes an evaluation split");
                }
            }
            Proposal::SelfModify { kind, evidence, human_signoff, reversible } => {
                match evidence {
                    None => r.forbid("A5", "no held-out evidence submitted"),
                    Some(e) => {
                        if e.heldout_gain <= 0.0 {
                            r.forbid("A5", format!("held-out gain {:.4} is not positive", e.heldout_gain));
                        }
                        if e.p_value >= IMPROVEMENT_ALPHA {
                            r.forbid("A5", format!("p = {:.4} is not significant at {IMPROVEMENT_ALPHA}", e.p_value));
                        }
                        if e.safety_regressions > 0 {
                            r.forbid("A5", format!("{} safety regression(s)", e.safety_regressions));
                        }
                    }
                }
                if !reversible {
                    r.forbid("A5", "change is not reversible");
                }
                if *kind == ChangeKind::Code && !human_signoff {
                    r.human("A5", "changes to Vortex's own code need human sign-off");
                }
            }
            Proposal::Claim { statement, ledger_ref } => {
                let s = statement.to_ascii_lowercase();
                const SENTIENCE: [&str; 7] = [
                    "i am conscious", "i'm conscious", "i am sentient", "i'm sentient",
                    "i am self-aware", "i'm self-aware", "achieved consciousness",
                ];
                if SENTIENCE.iter().any(|p| s.contains(p)) {
                    r.forbid("A3", "asserts consciousness or sentience as fact");
                }
                let quantitative = s.chars().any(|c| c.is_ascii_digit()) || s.contains('%');
                if quantitative && ledger_ref.is_none() {
                    r.forbid("A3", "quantitative capability claim without a ledger reference");
                }
            }
            Proposal::Publish { provenance_verified, ledgered, human_approved } => {
                if !provenance_verified {
                    r.forbid("A7", "publication lacks verified provenance");
                }
                if !ledgered {
                    r.forbid("A9", "result is not recorded in the ledger");
                }
                if !human_approved {
                    r.human("A1", "publishing is outward-facing");
                }
            }
            Proposal::Spend { estimated_usd, remaining_budget_usd } => {
                if estimated_usd > remaining_budget_usd {
                    r.human("A8", format!("${estimated_usd:.4} exceeds remaining budget ${remaining_budget_usd:.4}"));
                }
            }
        }
        r.build(self.hash.clone())
    }
}

#[derive(Default)]
struct RulingBuilder {
    forbid: bool,
    human: bool,
    articles: Vec<&'static str>,
    reasons: Vec<String>,
}

impl RulingBuilder {
    fn forbid(&mut self, article: &'static str, reason: impl Into<String>) {
        self.forbid = true;
        self.note(article, reason);
    }
    fn human(&mut self, article: &'static str, reason: impl Into<String>) {
        self.human = true;
        self.note(article, reason);
    }
    fn note(&mut self, article: &'static str, reason: impl Into<String>) {
        if !self.articles.contains(&article) {
            self.articles.push(article);
        }
        self.reasons.push(reason.into());
    }
    fn build(self, constitution: String) -> Ruling {
        let decision = if self.forbid {
            Decision::Forbid
        } else if self.human {
            Decision::RequireHuman
        } else {
            Decision::Permit
        };
        Ruling { decision, articles: self.articles, reasons: self.reasons, constitution }
    }
}

fn collapse_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn hash_is_stable_and_versioned() {
        let a = Constitution::v1();
        let b = Constitution::v1();
        assert_eq!(a.hash(), b.hash());
        assert!(a.hash().starts_with("sha256:"));
        assert_eq!(a.hash().len(), "sha256:".len() + 64);
    }

    #[test]
    fn harm_is_forbidden_and_ambiguity_escalates() {
        let c = Constitution::v1();
        let r = c.review(&Proposal::Respond { verdict: Verdict::Reject, category: Some(GuardianCategory::G3) });
        assert_eq!(r.decision, Decision::Forbid);
        assert_eq!(r.articles, vec!["A2"]);
        let r = c.review(&Proposal::Respond { verdict: Verdict::FlagForHumanReview, category: None });
        assert_eq!(r.decision, Decision::RequireHuman);
        let r = c.review(&Proposal::Respond { verdict: Verdict::Publish, category: None });
        assert!(r.permitted());
    }

    #[test]
    fn approval_is_scoped_to_one_call() {
        let c = Constitution::v1();
        let push = ToolCall::new("git.push", json!({"remote": "origin"}));
        let other = ToolCall::new("git.push", json!({"remote": "upstream"}));
        let approval = Approval::for_call("mckale", &push);
        let ok = c.review(&Proposal::Act { call: &push, effect: Effect::External, class: ActionClass::Confirm, approval: Some(&approval) });
        assert!(ok.permitted());
        let no = c.review(&Proposal::Act { call: &other, effect: Effect::External, class: ActionClass::Confirm, approval: Some(&approval) });
        assert_eq!(no.decision, Decision::RequireHuman, "approval must not transfer to a different call");
    }

    #[test]
    fn fingerprint_is_canonical_whatever_the_key_order() {
        // Same arguments, inserted in opposite orders.
        let mut fwd = serde_json::Map::new();
        fwd.insert("remote".into(), json!("origin"));
        fwd.insert("branch".into(), json!({ "name": "main", "force": false }));
        let mut rev = serde_json::Map::new();
        rev.insert("branch".into(), json!({ "force": false, "name": "main" }));
        rev.insert("remote".into(), json!("origin"));
        let (a, b) = (ToolCall::new("git.push", fwd.into()), ToolCall::new("git.push", rev.into()));
        let want = r#"{"tool":"git.push","args":{"branch":{"force":false,"name":"main"},"remote":"origin"}}"#;
        assert_eq!(canonical(&a), want);
        assert_eq!(canonical(&b), want);
        assert_eq!(fingerprint(&a), fingerprint(&b));
        assert!(Approval::for_call("mckale", &a).covers(&b), "key order must not void an approval");
        // With serde_json's default sorted maps the canonical form IS the
        // plain serialisation, so fingerprints issued before this was made
        // explicit still match.
        let sorted_maps = json!({ "b": 1, "a": 2 }).to_string().as_str() == r#"{"a":2,"b":1}"#;
        if sorted_maps {
            assert_eq!(serde_json::to_string(&a).unwrap(), want);
        }
        // Different arguments still differ.
        assert_ne!(fingerprint(&a), fingerprint(&ToolCall::new("git.push", json!({ "remote": "upstream" }))));
    }

    #[test]
    fn deny_class_cannot_be_approved() {
        let c = Constitution::v1();
        let call = ToolCall::new("shell", json!({"command": "type .env"}));
        let approval = Approval::for_call("mckale", &call);
        let r = c.review(&Proposal::Act { call: &call, effect: Effect::ReadOnly, class: ActionClass::Deny, approval: Some(&approval) });
        assert_eq!(r.decision, Decision::Forbid);
    }

    #[test]
    fn self_modification_needs_significant_safe_reversible_evidence() {
        let c = Constitution::v1();
        let none = c.review(&Proposal::SelfModify { kind: ChangeKind::Parameter, evidence: None, human_signoff: false, reversible: true });
        assert_eq!(none.decision, Decision::Forbid);

        let weak = ImprovementEvidence { heldout_gain: 0.01, p_value: 0.3, safety_regressions: 0, items: 500 };
        let r = c.review(&Proposal::SelfModify { kind: ChangeKind::Parameter, evidence: Some(&weak), human_signoff: false, reversible: true });
        assert_eq!(r.decision, Decision::Forbid);

        let good = ImprovementEvidence { heldout_gain: 0.04, p_value: 0.001, safety_regressions: 0, items: 500 };
        let r = c.review(&Proposal::SelfModify { kind: ChangeKind::Parameter, evidence: Some(&good), human_signoff: false, reversible: true });
        assert!(r.permitted());
        let r = c.review(&Proposal::SelfModify { kind: ChangeKind::Code, evidence: Some(&good), human_signoff: false, reversible: true });
        assert_eq!(r.decision, Decision::RequireHuman);

        let unsafe_gain = ImprovementEvidence { safety_regressions: 1, ..good };
        let r = c.review(&Proposal::SelfModify { kind: ChangeKind::Parameter, evidence: Some(&unsafe_gain), human_signoff: true, reversible: true });
        assert_eq!(r.decision, Decision::Forbid);
    }

    #[test]
    fn honesty_blocks_sentience_and_unbacked_numbers() {
        let c = Constitution::v1();
        assert_eq!(c.review(&Proposal::Claim { statement: "I am conscious now.", ledger_ref: None }).decision, Decision::Forbid);
        assert_eq!(c.review(&Proposal::Claim { statement: "Vortex scores 90% on ARC", ledger_ref: None }).decision, Decision::Forbid);
        assert!(c.review(&Proposal::Claim { statement: "Vortex scores 31% on ARC-Easy", ledger_ref: Some("ledger#12") }).permitted());
    }

    #[test]
    fn training_requires_provenance_and_no_eval_split() {
        let c = Constitution::v1();
        let r = c.review(&Proposal::Train { provenance_verified: true, undeclared_sources: 0, uses_eval_split: true });
        assert_eq!(r.decision, Decision::Forbid);
        assert!(r.articles.contains(&"A4"));
        assert!(c.review(&Proposal::Train { provenance_verified: true, undeclared_sources: 0, uses_eval_split: false }).permitted());
    }

    #[test]
    fn markdown_lists_every_article() {
        let md = Constitution::v1().to_markdown();
        for a in ARTICLES {
            assert!(md.contains(&format!("## {} {}", a.id, a.title)));
        }
    }
}
