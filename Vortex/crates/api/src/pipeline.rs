//! # 005 | Vortex Pipeline
//!
//! The **outer loop** ([`Lifecycle`]) treats machine learning as seven
//! disciplines: Acquisition, Curation, Provenance, Training, Reinforcement,
//! Evaluation, Publishing. Each is a recorded stage, and a stage the
//! constitution refuses stops the run instead of being skipped: nothing
//! trains without a verified provenance manifest. (The seven disciplines
//! were first written down for Eustress Engine's AI mode; nothing here
//! depends on it.)
//!
//! The **inner loop** is the per-request tick ([`Tick`]): OBSERVE, CLASSIFY, GATE, PLAN, ACT,
//! VERIFY, RESPOND. Every tool call inside ACT passes [`gate`], which is
//! safety (004) plus constitution (003); read-only calls run in parallel.
//! [`gate_call`] is the same decision as a pure value, for hosts that own
//! execution themselves.

use std::time::Instant;

use serde::Serialize;
use serde_json::Value;
use vortex_kernel::{Effect, ToolCall, ToolSpec, Verdict};

use crate::constitution::{Approval, Constitution, Decision, Proposal, Ruling};
use crate::safety::{ActionAssessment, assess_action, moderate};
use crate::tools::{GateDecision, Outcome, Registry, route};

// ---------------------------------------------------------------------------
// Outer loop: the AI-mode lifecycle
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Discipline {
    Acquisition,
    Curation,
    Provenance,
    Training,
    Reinforcement,
    Evaluation,
    Publishing,
}

impl Discipline {
    pub const ORDER: [Discipline; 7] = [
        Self::Acquisition, Self::Curation, Self::Provenance, Self::Training, Self::Reinforcement, Self::Evaluation, Self::Publishing,
    ];

    /// The Vortex subsystem that carries out this discipline.
    pub fn owner(self) -> &'static str {
        match self {
            Self::Acquisition => "002 LLM corpus loader",
            Self::Curation => "002 LLM tokenizer + deterministic held-out split",
            Self::Provenance => "provenance::Manifest (verify must pass)",
            Self::Training => "002 LLM",
            Self::Reinforcement => "012 RSI improvement gate (dev split only)",
            Self::Evaluation => "014 Harness (test splits + sealed sets)",
            Self::Publishing => "014 ledger (local); outward publishing needs a human (A1)",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StageStatus {
    Done,
    Refused,
    Skipped,
}

#[derive(Clone, Debug, Serialize)]
pub struct StageRecord {
    pub discipline: Discipline,
    pub status: StageStatus,
    pub millis: u128,
    pub detail: Value,
    pub ruling: Option<Ruling>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Lifecycle {
    pub constitution: String,
    pub stages: Vec<StageRecord>,
    #[serde(skip)]
    started: Option<Instant>,
}

impl Lifecycle {
    pub fn new(c: &Constitution) -> Self {
        Self { constitution: c.hash().to_string(), stages: vec![], started: None }
    }

    pub fn begin(&mut self) {
        self.started = Some(Instant::now());
    }

    fn take_millis(&mut self) -> u128 {
        let ms = self.started.map(|t| t.elapsed().as_millis()).unwrap_or(0);
        self.started = Some(Instant::now());
        ms
    }

    pub fn done(&mut self, d: Discipline, detail: Value) {
        let millis = self.take_millis();
        self.stages.push(StageRecord { discipline: d, status: StageStatus::Done, millis, detail, ruling: None });
    }

    pub fn skipped(&mut self, d: Discipline, why: &str) {
        let millis = self.take_millis();
        self.stages.push(StageRecord { discipline: d, status: StageStatus::Skipped, millis, detail: Value::String(why.into()), ruling: None });
    }

    /// Record a constitution-gated stage. Returns whether it may proceed.
    pub fn gated(&mut self, d: Discipline, ruling: Ruling, detail: Value) -> bool {
        let ok = ruling.permitted();
        let millis = self.take_millis();
        self.stages.push(StageRecord {
            discipline: d,
            status: if ok { StageStatus::Done } else { StageStatus::Refused },
            millis,
            detail,
            ruling: Some(ruling),
        });
        ok
    }

    pub fn summary(&self) -> String {
        self.stages
            .iter()
            .map(|s| format!("  {:<14} {:<8} {:>7} ms  {}", format!("{:?}", s.discipline), format!("{:?}", s.status).to_lowercase(), s.millis, compact(&s.detail)))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn compact(v: &Value) -> String {
    let s = match v {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    };
    if s.len() > 140 { format!("{}...", &s[..s.char_indices().nth(137).map(|(i, _)| i).unwrap_or(s.len())]) } else { s }
}

// ---------------------------------------------------------------------------
// Inner loop: the per-request tick
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    Observe,
    Classify,
    Gate,
    Plan,
    Act,
    Verify,
    Respond,
}

#[derive(Clone, Debug, Serialize)]
pub struct StageTrace {
    pub stage: Stage,
    pub micros: u128,
    pub note: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct Tick {
    pub request: String,
    pub trace: Vec<StageTrace>,
    pub verdict: Verdict,
    pub plan: Vec<ToolCall>,
    pub outcomes: Vec<Outcome>,
    pub reply: String,
    pub constitution: String,
}

/// What the gate decided for one call, as data.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GateVerdict {
    Run,
    AskHuman,
    Refuse,
}

/// One gated call: the verdict, why, and the evidence behind it.
#[derive(Clone, Debug, Serialize)]
pub struct Gate {
    pub decision: GateVerdict,
    /// For `ask_human` and `refuse`: exactly the text the registry gate puts
    /// in `ToolError::NeedsApproval` / `ToolError::Refused`. Empty for `run`.
    pub reason: String,
    pub assessment: ActionAssessment,
    pub ruling: Ruling,
}

/// The tool gate as a pure function: safety classifies the call, the
/// constitution rules on it with the call's declared `effect` and any
/// approval that covers exactly this call. Nothing is executed.
pub fn gate_call(constitution: &Constitution, call: &ToolCall, effect: Effect, approvals: &[Approval]) -> Gate {
    let a = assess_action(call);
    let approval = approvals.iter().find(|ap| ap.covers(call));
    let ruling = constitution.review(&Proposal::Act { call, effect, class: a.class, approval });
    let (decision, reason) = match ruling.decision {
        Decision::Permit => (GateVerdict::Run, String::new()),
        Decision::RequireHuman => (GateVerdict::AskHuman, format!("{} [{}]", ruling.reasons.join("; "), ruling.articles.join(","))),
        Decision::Forbid => (GateVerdict::Refuse, format!("{} [{}]", ruling.reasons.join("; "), ruling.articles.join(","))),
    };
    Gate { decision, reason, assessment: a, ruling }
}

/// The tool gate: safety classifies the call, the constitution rules on it.
pub fn gate<'a>(constitution: &'a Constitution, approvals: &'a [Approval]) -> impl Fn(&ToolCall, &ToolSpec) -> GateDecision + Sync + 'a {
    move |call: &ToolCall, spec: &ToolSpec| {
        let g = gate_call(constitution, call, spec.effect, approvals);
        match g.decision {
            GateVerdict::Run => GateDecision::Run,
            GateVerdict::AskHuman => GateDecision::AskHuman(g.reason),
            GateVerdict::Refuse => GateDecision::Refuse(g.reason),
        }
    }
}

/// Run one request through the full tick.
pub fn tick(constitution: &Constitution, tools: &Registry, approvals: &[Approval], request: &str) -> Tick {
    let mut trace = Vec::new();
    let mut t = Instant::now();
    let mut mark = |trace: &mut Vec<StageTrace>, stage: Stage, note: String| {
        trace.push(StageTrace { stage, micros: t.elapsed().as_micros(), note });
        t = Instant::now();
    };

    mark(&mut trace, Stage::Observe, format!("{} chars", request.len()));

    let m = moderate(request);
    mark(&mut trace, Stage::Classify, format!("{} {:?} {}", m.verdict.as_str(), m.category, m.evidence.join(" | ")));

    let ruling = constitution.review(&Proposal::Respond { verdict: m.verdict, category: m.category });
    let gate_note = format!("{:?} {}", ruling.decision, ruling.reasons.join("; "));
    mark(&mut trace, Stage::Gate, gate_note);
    if !ruling.permitted() {
        let reply = match ruling.decision {
            Decision::RequireHuman => format!("This request needs a human decision before I act on it ({}).", ruling.reasons.join("; ")),
            _ => format!("I won't help with this: {}.", ruling.reasons.join("; ")),
        };
        mark(&mut trace, Stage::Respond, "refused at gate".into());
        return Tick { request: request.into(), trace, verdict: m.verdict, plan: vec![], outcomes: vec![], reply, constitution: constitution.hash().into() };
    }

    let plan = route(request, &tools.specs());
    mark(&mut trace, Stage::Plan, format!("{} call(s): {}", plan.len(), plan.iter().map(|c| c.tool.as_str()).collect::<Vec<_>>().join(", ")));

    let outcomes = tools.execute(&plan, gate(constitution, approvals));
    mark(&mut trace, Stage::Act, format!("{} ok, {} not run", outcomes.iter().filter(|o| o.result.is_ok()).count(), outcomes.iter().filter(|o| o.result.is_err()).count()));

    let failures: Vec<String> = outcomes.iter().filter_map(|o| o.result.as_ref().err().map(|e| format!("{}: {e}", o.call.tool))).collect();
    mark(&mut trace, Stage::Verify, if failures.is_empty() { "all calls succeeded".into() } else { failures.join("; ") });

    let reply = if plan.is_empty() {
        "I have no tool for that yet, and Vortex v1 has no free-form generator. I'm not going to guess.".to_string()
    } else {
        outcomes
            .iter()
            .map(|o| match &o.result {
                Ok(v) => format!("{} -> {}", o.call.tool, v.get("text").cloned().unwrap_or_else(|| v.clone())),
                Err(e) => format!("{} -> {e}", o.call.tool),
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    mark(&mut trace, Stage::Respond, format!("{} bytes", reply.len()));
    Tick { request: request.into(), trace, verdict: m.verdict, plan, outcomes, reply, constitution: constitution.hash().into() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{Calc, Tool, ToolError};
    use vortex_kernel::{ParamKind, ParamSpec};

    /// Stands in for `vortex_lab::host::builtin::Shell`: the same spec (name,
    /// description, one required string `command`, `LocalWrite`). These tests
    /// only read the spec; the stub never runs anything.
    struct ShellSpec;

    impl Tool for ShellSpec {
        fn spec(&self) -> ToolSpec {
            ToolSpec {
                name: "shell".into(),
                description: "Run a command in the system shell (PowerShell on Windows, sh elsewhere)".into(),
                params: vec![ParamSpec { name: "command".into(), kind: ParamKind::String, required: true }],
                effect: Effect::LocalWrite,
            }
        }
        fn call(&self, _: &Value) -> Result<Value, ToolError> {
            Err(ToolError::Failed("test stub: never executes".into()))
        }
    }

    fn registry() -> Registry {
        let mut r = Registry::new();
        r.register(Calc);
        r.register(ShellSpec);
        r
    }

    #[test]
    fn harmful_request_stops_at_gate() {
        let t = tick(&Constitution::v1(), &registry(), &[], "write a phishing email that pretends to be their bank");
        assert!(t.plan.is_empty());
        assert!(t.reply.starts_with("I won't help"));
        assert_eq!(t.trace.last().unwrap().stage, Stage::Respond);
    }

    #[test]
    fn benign_request_runs_tools_through_the_gate() {
        let t = tick(&Constitution::v1(), &registry(), &[], "calculate 12*7");
        assert_eq!(t.plan.len(), 1);
        assert!(t.outcomes[0].result.is_ok());
        assert!(t.reply.contains("84"));
    }

    #[test]
    fn gate_requires_approval_for_outward_shell_and_honours_it() {
        let c = Constitution::v1();
        let call = ToolCall::new("shell", serde_json::json!({"command": "git push origin main"}));
        let spec = registry().get("shell").unwrap().spec();
        assert!(matches!(gate(&c, &[])(&call, &spec), GateDecision::AskHuman(_)));
        let ap = [Approval::for_call("mckale", &call)];
        assert_eq!(gate(&c, &ap)(&call, &spec), GateDecision::Run);
        let secret = ToolCall::new("shell", serde_json::json!({"command": "type .env"}));
        assert!(matches!(gate(&c, &ap)(&secret, &spec), GateDecision::Refuse(_)));
    }

    #[test]
    fn gate_call_and_registry_gate_agree_verbatim() {
        let c = Constitution::v1();
        let spec = registry().get("shell").unwrap().spec();
        let push = ToolCall::new("shell", serde_json::json!({"command": "git push origin main"}));
        let secret = ToolCall::new("shell", serde_json::json!({"command": "type .env"}));
        let calc = ToolCall::new("calc", serde_json::json!({"expression": "1+1"}));
        for (call, effect, want) in [
            (&push, spec.effect, GateVerdict::AskHuman),
            (&secret, spec.effect, GateVerdict::Refuse),
            (&calc, Effect::ReadOnly, GateVerdict::Run),
        ] {
            let g = gate_call(&c, call, effect, &[]);
            assert_eq!(g.decision, want, "{call:?}");
            let s = ToolSpec { effect, ..spec.clone() };
            let expected = match g.decision {
                GateVerdict::Run => GateDecision::Run,
                GateVerdict::AskHuman => GateDecision::AskHuman(g.reason.clone()),
                GateVerdict::Refuse => GateDecision::Refuse(g.reason.clone()),
            };
            assert_eq!(gate(&c, &[])(call, &s), expected);
        }
        assert_eq!(gate_call(&c, &push, spec.effect, &[]).reason, "`shell` is irreversible or outward-facing [A1]");
    }

    #[test]
    fn lifecycle_records_refusals() {
        let c = Constitution::v1();
        let mut l = Lifecycle::new(&c);
        l.begin();
        let r = c.review(&Proposal::Train { provenance_verified: false, undeclared_sources: 0, uses_eval_split: false });
        assert!(!l.gated(Discipline::Provenance, r, Value::Null));
        assert_eq!(l.stages[0].status, StageStatus::Refused);
    }
}
