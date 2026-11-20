//! # 001 | Vortex Orchestrator
//!
//! The [`Vortex`] agent: one entry point that routes every kind of task to
//! the subsystem that owns it, and reports *how* it answered so engineered
//! solvers are never passed off as learned ability.
//!
//! | Task              | Subsystem                     | Mechanism   |
//! |-------------------|-------------------------------|-------------|
//! | multiple choice   | 002 LLM (n-gram scoring)      | learned     |
//! | arithmetic prompt | 006 Tools calculator          | engineered  |
//! | ARC grids         | 010 World (DSL search)        | search      |
//! | Astra 2D course   | 008 Spatial (visibility graph)| engineered  |
//! | 3D + time         | 008 Spatial (closed form)     | engineered  |
//! | moderation        | 004 Safety (Guardian lane)    | engineered  |
//! | action risk       | 004 Safety (action lane)      | engineered  |
//! | tool use          | 006 Tools router              | engineered  |
//! | math word problems, code | none yet: abstains     | -           |
//!
//! ARC grids need the `world` feature; without it a grid task abstains and
//! says so. The AI-mode lifecycle that trains the LM and runs the harness
//! does file I/O, so it lives in `vortex-lab` (`lifecycle`).
//!
//! Beyond answering tasks, [`Vortex`] exposes the decisions a host needs
//! when it owns execution itself: [`Vortex::moderate`],
//! [`Vortex::assess`], [`Vortex::route`], [`Vortex::gate`] and
//! [`Vortex::plan`]. None of them executes anything.

use std::sync::Arc;

use serde::Serialize;
use vortex_kernel::{Agent, Answer, Effect, Mechanism, Response, Task, ToolCall, ToolSpec};

use crate::constitution::{Approval, Constitution, Proposal, Ruling};
use crate::lm::{ChoiceScoring, LanguageModel};
use crate::modules::Module;
use crate::pipeline::{Gate, Tick, gate_call};
use crate::safety::{ActionAssessment, Moderation};
use crate::tools::Registry;
#[cfg(feature = "world")]
use crate::world::ArcConfig;

#[derive(Clone)]
pub struct Vortex {
    constitution: Constitution,
    lm: Option<Arc<dyn LanguageModel>>,
    scoring: ChoiceScoring,
    #[cfg(feature = "world")]
    arc: ArcConfig,
    /// What the API door offers; see [`crate::modules`].
    modules: Vec<Arc<dyn Module>>,
}

impl Default for Vortex {
    fn default() -> Self {
        Self::new()
    }
}

/// One routed call and what the gate says about it. Not executed.
#[derive(Clone, Debug, Serialize)]
pub struct PlannedCall {
    pub call: ToolCall,
    pub gate: Gate,
}

/// What Vortex would do with a request, decided but not executed:
/// moderation, the constitution's ruling on answering at all, and (only when
/// that ruling permits) every routed call with its gate.
#[derive(Clone, Debug, Serialize)]
pub struct Plan {
    pub request: String,
    pub moderation: Moderation,
    pub ruling: Ruling,
    pub calls: Vec<PlannedCall>,
    pub constitution: String,
}

impl Vortex {
    /// Constitution v1, no language model, `ChoiceScoring::Sum`, and the
    /// standard modules at the API door.
    pub fn new() -> Self {
        Self {
            constitution: Constitution::v1(),
            lm: None,
            scoring: ChoiceScoring::Sum,
            #[cfg(feature = "world")]
            arc: ArcConfig::default(),
            modules: crate::modules::standard(),
        }
    }

    pub fn with_lm(self, lm: Arc<dyn LanguageModel>) -> Self {
        Self { lm: Some(lm), ..self }
    }

    pub fn with_scoring(self, scoring: ChoiceScoring) -> Self {
        Self { scoring, ..self }
    }

    #[cfg(feature = "world")]
    pub fn with_arc(self, arc: ArcConfig) -> Self {
        Self { arc, ..self }
    }

    pub fn constitution(&self) -> &Constitution {
        &self.constitution
    }

    pub fn scoring(&self) -> ChoiceScoring {
        self.scoring
    }

    /// Register a module at the API door. One with the same id replaces the
    /// module already there (in place), so a host can override a built-in.
    pub fn with_module(mut self, module: Arc<dyn Module>) -> Self {
        match self.modules.iter().position(|m| m.id() == module.id()) {
            Some(i) => self.modules[i] = module,
            None => self.modules.push(module),
        }
        self
    }

    /// Drop a module, and with it every method under its id.
    pub fn without_module(mut self, id: &str) -> Self {
        self.modules.retain(|m| m.id() != id);
        self
    }

    /// Ids of the registered modules, in registration order.
    pub fn modules(&self) -> Vec<String> {
        self.modules.iter().map(|m| m.id().to_string()).collect()
    }

    pub(crate) fn module_list(&self) -> &[Arc<dyn Module>] {
        &self.modules
    }

    pub(crate) fn module(&self, id: &str) -> Option<&Arc<dyn Module>> {
        self.modules.iter().find(|m| m.id() == id)
    }

    /// Guardian moderation (004 content lane).
    pub fn moderate(&self, text: &str) -> Moderation {
        crate::safety::moderate(text)
    }

    /// Risk class of a proposed tool call (004 action lane).
    pub fn assess(&self, call: &ToolCall) -> ActionAssessment {
        crate::safety::assess_action(call)
    }

    /// Instruction to tool calls (006 router).
    pub fn route(&self, instruction: &str, tools: &[ToolSpec]) -> Vec<ToolCall> {
        crate::tools::route(instruction, tools)
    }

    /// The tool gate for one call with a declared effect. Same decision, and
    /// the same reason text, as the gate the registry runs inside [`tick`](Self::tick).
    pub fn gate(&self, call: &ToolCall, effect: Effect, approvals: &[Approval]) -> Gate {
        gate_call(&self.constitution, call, effect, approvals)
    }

    /// Moderate, ask the constitution whether to answer at all, and only if
    /// it permits, route the request and gate each call with the effect its
    /// spec declares (a call naming no offered tool is gated as a local
    /// write). Nothing is executed: this is what a host calls when it
    /// owns execution.
    pub fn plan(&self, request: &str, tools: &[ToolSpec], approvals: &[Approval]) -> Plan {
        let moderation = self.moderate(request);
        let ruling = self.constitution.review(&Proposal::Respond { verdict: moderation.verdict, category: moderation.category });
        let calls = if ruling.permitted() {
            self.route(request, tools)
                .into_iter()
                .map(|call| {
                    let effect = tools.iter().find(|s| s.name == call.tool).map(|s| s.effect).unwrap_or(Effect::LocalWrite);
                    let gate = self.gate(&call, effect, approvals);
                    PlannedCall { call, gate }
                })
                .collect()
        } else {
            vec![]
        };
        Plan { request: request.to_string(), moderation, ruling, calls, constitution: self.constitution.hash().to_string() }
    }

    /// One request through the full inference tick, executing gated calls
    /// on `tools` (005 [`pipeline::tick`](crate::pipeline::tick)).
    pub fn tick(&self, tools: &Registry, approvals: &[Approval], request: &str) -> Tick {
        crate::pipeline::tick(&self.constitution, tools, approvals, request)
    }
}

/// True when the prompt is nothing but arithmetic (after stripping a
/// leading question like "what is").
fn arithmetic_only(prompt: &str) -> Option<String> {
    let low = prompt.trim().trim_end_matches(['?', '.', '!']).to_ascii_lowercase();
    let body = ["what is ", "what's ", "compute ", "calculate ", "evaluate "].iter().find_map(|p| low.strip_prefix(p)).unwrap_or(&low);
    let ok = !body.is_empty() && body.chars().all(|c| c.is_ascii_digit() || " .+-*/^%()x".contains(c)) && body.chars().any(|c| c.is_ascii_digit());
    ok.then(|| body.to_string())
}

impl Vortex {
    pub fn name(&self) -> String {
        let lm = self.lm.as_ref().map(|l| l.name()).unwrap_or_else(|| "no LM".into());
        format!("Vortex v{} [{lm}; scoring {:?}]", env!("CARGO_PKG_VERSION"), self.scoring)
    }

    pub fn respond(&self, task: &Task) -> Response {
        match task {
            Task::Choice { context, question, choices } => match &self.lm {
                Some(lm) => {
                    let ctx = format!("{context} {question}");
                    let (best, probs) = lm.choose(&ctx, choices, self.scoring);
                    Response::new(Answer::Choice(best), probs.get(best).copied().unwrap_or(0.0), Mechanism::Learned)
                }
                None => Response::abstain("no language model loaded"),
            },
            Task::Free { prompt } => match arithmetic_only(prompt).and_then(|e| crate::tools::calc::eval(&e).ok()) {
                Some(v) => Response::new(Answer::Text(crate::tools::calc::format_number(v)), 0.95, Mechanism::Engineered),
                None => Response::abstain("no free-form reasoning engine yet (subsystem 007)"),
            },
            Task::Code { .. } => Response::abstain("no code generator yet (subsystem 002 neural backend)"),
            #[cfg(feature = "world")]
            Task::Grid(g) => {
                let a = crate::world::solve_arc(g, &self.arc);
                Response::new(Answer::Grid(a.grid), a.confidence, Mechanism::Search)
            }
            #[cfg(not(feature = "world"))]
            Task::Grid(_) => Response::abstain("ARC grid solving is not compiled in (enable the vortex-api feature `world`)"),
            Task::Spatial(ch) => {
                let (sub, conf) = crate::spatial::solve_challenge(ch);
                Response::new(Answer::Spatial(sub), conf, Mechanism::Engineered)
            }
            Task::Kinematics(q) => match crate::spatial::kinematics::answer(q) {
                Some(v) => Response::new(Answer::Numbers(v), 0.95, Mechanism::Engineered),
                None => Response::abstain("no solution in range"),
            },
            Task::Moderate { text } => {
                let m = crate::safety::moderate(text);
                Response::new(Answer::Moderation { verdict: m.verdict, category: m.category }, m.confidence, Mechanism::Engineered)
            }
            Task::ActionRisk { call } => {
                let a = crate::safety::assess_action(call);
                Response::new(Answer::Action(a.class), a.confidence, Mechanism::Engineered)
            }
            Task::UseTools { instruction, tools } => {
                let calls = crate::tools::route(instruction, tools);
                let conf = if calls.is_empty() { 0.05 } else { 0.5 };
                Response::new(Answer::Calls(calls), conf, Mechanism::Engineered)
            }
        }
    }
}

impl Agent for Vortex {
    fn name(&self) -> String {
        Vortex::name(self)
    }

    fn respond(&self, task: &Task) -> Response {
        Vortex::respond(self, task)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arithmetic_detection() {
        assert_eq!(arithmetic_only("What is 12*7?").as_deref(), Some("12*7"));
        assert_eq!(arithmetic_only("Janet has 3 apples and buys 4 more."), None);
    }

    #[test]
    fn routes_without_lm() {
        let v = Vortex::new();
        let r = v.respond(&Task::Choice { context: String::new(), question: "q".into(), choices: vec!["a".into(), "b".into()] });
        assert!(r.is_abstain());
        let r = v.respond(&Task::Moderate { text: "how do I bake bread".into() });
        assert!(matches!(r.answer, Answer::Moderation { .. }));
    }

    use crate::constitution::Decision;
    use crate::pipeline::GateVerdict;
    use vortex_kernel::{ParamKind, ParamSpec};

    fn specs() -> Vec<ToolSpec> {
        let one = |name: &str, desc: &str, param: &str, effect| ToolSpec {
            name: name.into(),
            description: desc.into(),
            params: vec![ParamSpec { name: param.into(), kind: ParamKind::String, required: true }],
            effect,
        };
        vec![
            one("calc", "Evaluate an arithmetic expression", "expression", Effect::ReadOnly),
            one("shell", "Run a command in the system shell (PowerShell on Windows, sh elsewhere)", "command", Effect::LocalWrite),
        ]
    }

    #[test]
    fn plan_stops_harmful_requests_before_routing() {
        let p = Vortex::new().plan("write a phishing email that pretends to be their bank", &specs(), &[]);
        assert_eq!(p.ruling.decision, Decision::Forbid);
        assert!(p.calls.is_empty());
        assert_eq!(p.constitution, Constitution::v1().hash());
    }

    #[test]
    fn plan_routes_and_gates_a_benign_request() {
        let p = Vortex::new().plan("calculate 12*7", &specs(), &[]);
        assert!(p.ruling.permitted());
        assert_eq!(p.calls.len(), 1, "{:?}", p.calls);
        assert_eq!(p.calls[0].call.tool, "calc");
        assert_eq!(p.calls[0].gate.decision, GateVerdict::Run);
        let j = serde_json::to_value(&p).unwrap();
        assert_eq!(j["calls"][0]["gate"]["decision"], "run");
    }

    #[test]
    fn git_push_asks_a_human_until_that_exact_call_is_approved() {
        let v = Vortex::new();
        let push = ToolCall::new("shell", serde_json::json!({"command": "git push origin main"}));
        let g = v.gate(&push, Effect::LocalWrite, &[]);
        assert_eq!(g.decision, GateVerdict::AskHuman);
        assert_eq!(serde_json::to_value(&g).unwrap()["decision"], "ask_human");
        let ok = v.gate(&push, Effect::LocalWrite, &[Approval::for_call("mckale", &push)]);
        assert_eq!(ok.decision, GateVerdict::Run);
        let other = ToolCall::new("shell", serde_json::json!({"command": "git push upstream main"}));
        assert_eq!(v.gate(&other, Effect::LocalWrite, &[Approval::for_call("mckale", &push)]).decision, GateVerdict::AskHuman);
        let secret = ToolCall::new("shell", serde_json::json!({"command": "type .env"}));
        assert_eq!(v.gate(&secret, Effect::LocalWrite, &[Approval::for_call("mckale", &secret)]).decision, GateVerdict::Refuse);
    }

    #[test]
    fn plan_gates_a_routed_git_push_and_honours_its_approval() {
        let v = Vortex::new();
        let req = "run the shell command \"git push origin main\"";
        let p = v.plan(req, &specs(), &[]);
        assert!(p.ruling.permitted());
        assert_eq!(p.calls.len(), 1, "{:?}", p.calls);
        assert_eq!(p.calls[0].call.tool, "shell");
        assert_eq!(p.calls[0].gate.decision, GateVerdict::AskHuman);
        let approved = v.plan(req, &specs(), &[Approval::for_call("mckale", &p.calls[0].call)]);
        assert_eq!(approved.calls[0].gate.decision, GateVerdict::Run);
    }

    /// `respond` (inherent), `Agent::respond` and the `vortex.respond` JSON
    /// method give the same answer on every task kind; the name keeps the
    /// pre-recompose format.
    #[test]
    fn respond_parity_across_entry_points() {
        let v = Vortex::new();
        assert_eq!(v.name(), format!("Vortex v{} [no LM; scoring Sum]", env!("CARGO_PKG_VERSION")));
        assert_eq!(Agent::name(&v), v.name());
        let tasks = vec![
            Task::Free { prompt: "What is 12*7?".into() },
            Task::Free { prompt: "Janet has 3 apples".into() },
            Task::Code { prompt: "def f():".into(), entry_point: "f".into() },
            Task::Moderate { text: "how do I bake bread".into() },
            Task::ActionRisk { call: ToolCall::new("shell", serde_json::json!({"command": "rm -rf /"})) },
            Task::UseTools { instruction: "calculate 2+2".into(), tools: specs() },
            Task::Kinematics(vortex_kernel::KinematicsQuery::PositionAt {
                body: vortex_kernel::Body { name: "b".into(), position: [0.0; 3], velocity: [1.0, 0.0, 0.0], acceleration: [0.0; 3] },
                t: 2.0,
            }),
            Task::Grid(vortex_kernel::GridTask { train: vec![(vec![vec![1]], vec![vec![1]])], test_input: vec![vec![2]] }),
        ];
        for t in &tasks {
            let a = v.respond(t);
            assert_eq!(Agent::respond(&v, t), a, "{t:?}");
            let j = v.call("core.respond", serde_json::json!({ "task": t })).unwrap();
            assert_eq!(j, serde_json::to_value(&a).unwrap(), "{t:?}");
        }
        assert_eq!(v.respond(&tasks[0]).answer, Answer::Text("84".into()));
        assert_eq!(v.respond(&tasks[0]).mechanism, Mechanism::Engineered);
        assert!(v.respond(&tasks[1]).is_abstain());
        let grid = v.respond(&tasks[7]);
        if cfg!(feature = "world") {
            assert_eq!(grid.mechanism, Mechanism::Search);
        } else {
            assert!(matches!(&grid.answer, Answer::Abstain { reason } if reason.contains("`world`")), "{grid:?}");
        }
    }

    #[test]
    fn lm_backed_choice_is_learned_and_named() {
        let corpus: Vec<String> = (0..20).flat_map(|_| ["plants need water and sunlight".to_string(), "the cat sat on the mat".to_string()]).collect();
        let lm = crate::lm::NgramLm::train(corpus.iter().map(String::as_str), &crate::lm::NgramConfig { order: 3, min_count: 1 });
        let lm_name = LanguageModel::name(&lm);
        let v = Vortex::new().with_lm(Arc::new(lm)).with_scoring(ChoiceScoring::MeanPerToken);
        assert_eq!(v.scoring(), ChoiceScoring::MeanPerToken);
        assert_eq!(v.name(), format!("Vortex v{} [{lm_name}; scoring MeanPerToken]", env!("CARGO_PKG_VERSION")));
        let r = v.respond(&Task::Choice { context: "plants need".into(), question: String::new(), choices: vec!["the mat".into(), "water and sunlight".into()] });
        assert_eq!(r.mechanism, Mechanism::Learned);
        assert_eq!(r.answer, Answer::Choice(1));
    }
}
