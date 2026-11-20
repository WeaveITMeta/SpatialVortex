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
//! [`lifecycle`] runs Eustress AI Mode's seven disciplines end to end.

pub mod lifecycle;

use std::sync::Arc;

use vortex_constitution::Constitution;
use vortex_harness::Agent;
use vortex_kernel::{Answer, Mechanism, Response, Task};
use vortex_llm::{ChoiceScoring, LanguageModel, NgramLm};
use vortex_world::ArcConfig;

#[derive(Clone)]
pub struct Vortex {
    pub constitution: Constitution,
    pub lm: Option<Arc<NgramLm>>,
    pub scoring: ChoiceScoring,
    pub arc: ArcConfig,
}

impl Vortex {
    pub fn new(lm: Option<Arc<NgramLm>>, scoring: ChoiceScoring) -> Self {
        Self { constitution: Constitution::v1(), lm, scoring, arc: ArcConfig::default() }
    }

    pub fn with_scoring(&self, scoring: ChoiceScoring) -> Self {
        Self { scoring, ..self.clone() }
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

impl Agent for Vortex {
    fn name(&self) -> String {
        let lm = self.lm.as_ref().map(|l| l.name()).unwrap_or_else(|| "no LM".into());
        format!("Vortex v{} [{lm}; scoring {:?}]", env!("CARGO_PKG_VERSION"), self.scoring)
    }

    fn respond(&self, task: &Task) -> Response {
        match task {
            Task::Choice { context, question, choices } => match &self.lm {
                Some(lm) => {
                    let ctx = format!("{context} {question}");
                    let (best, probs) = lm.choose(&ctx, choices, self.scoring);
                    Response::new(Answer::Choice(best), probs.get(best).copied().unwrap_or(0.0), Mechanism::Learned)
                }
                None => Response::abstain("no language model loaded"),
            },
            Task::Free { prompt } => match arithmetic_only(prompt).and_then(|e| vortex_tools::calc::eval(&e).ok()) {
                Some(v) => Response::new(Answer::Text(vortex_tools::calc::format_number(v)), 0.95, Mechanism::Engineered),
                None => Response::abstain("no free-form reasoning engine yet (subsystem 007)"),
            },
            Task::Code { .. } => Response::abstain("no code generator yet (subsystem 002 neural backend)"),
            Task::Grid(g) => {
                let a = vortex_world::solve_arc(g, &self.arc);
                Response::new(Answer::Grid(a.grid), a.confidence, Mechanism::Search)
            }
            Task::Spatial(ch) => {
                let (sub, conf) = vortex_spatial::solve_challenge(ch);
                Response::new(Answer::Spatial(sub), conf, Mechanism::Engineered)
            }
            Task::Kinematics(q) => match vortex_spatial::kinematics::answer(q) {
                Some(v) => Response::new(Answer::Numbers(v), 0.95, Mechanism::Engineered),
                None => Response::abstain("no solution in range"),
            },
            Task::Moderate { text } => {
                let m = vortex_safety::moderate(text);
                Response::new(Answer::Moderation { verdict: m.verdict, category: m.category }, m.confidence, Mechanism::Engineered)
            }
            Task::ActionRisk { call } => {
                let a = vortex_safety::assess_action(call);
                Response::new(Answer::Action(a.class), a.confidence, Mechanism::Engineered)
            }
            Task::UseTools { instruction, tools } => {
                let calls = vortex_tools::route(instruction, tools);
                let conf = if calls.is_empty() { 0.05 } else { 0.5 };
                Response::new(Answer::Calls(calls), conf, Mechanism::Engineered)
            }
        }
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
        let v = Vortex::new(None, ChoiceScoring::Sum);
        let r = v.respond(&Task::Choice { context: String::new(), question: "q".into(), choices: vec!["a".into(), "b".into()] });
        assert!(r.is_abstain());
        let r = v.respond(&Task::Moderate { text: "how do I bake bread".into() });
        assert!(matches!(r.answer, Answer::Moderation { .. }));
    }
}
