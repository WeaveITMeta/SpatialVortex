//! # 014 | Vortex Harness
//!
//! Measures what Vortex can actually do, across as many capabilities as
//! possible, without letting it cheat. The protocol (also in
//! `eval/PROTOCOL.md`):
//!
//! 1. **Gold never crosses the boundary.** A [`Suite`] holds its labels
//!    privately and hands the [`Agent`] only a [`Task`]. `Agent::respond`
//!    takes `&self`: the evaluated system cannot accumulate state from the
//!    run (no test-time training on labels).
//! 2. **Graders are independent of solvers.** This crate depends on the
//!    kernel only. Where a grader needs computation (kinematics, arithmetic,
//!    route geometry) it has its own implementation, so a solver bug cannot
//!    also corrupt its own grade.
//! 3. **Breadth over peaks.** The headline number, the *Vortex Index*, is the
//!    mean chance-normalised score over **every registered suite**, including
//!    those Vortex cannot do yet (they count as zero). Maximising one
//!    benchmark barely moves it.
//! 4. **Seeded samples, never first-N.** Subsets are drawn with a recorded
//!    seed; procedural suites generate fresh instances per seed.
//! 5. **Sealed sets.** Independently authored held-out sets are hash-pinned
//!    (`eval/heldout/MANIFEST.sha256`); the first scored run is recorded
//!    before any tuning against them and never overwritten.
//! 6. **Report everything.** Accuracy with Wilson 95% intervals, chance,
//!    abstentions, calibration (Brier), latency, and how each answer was
//!    produced (learned / search / engineered).

pub mod expr;
pub mod report;
pub mod run;
pub mod stats;
pub mod suites;

use std::collections::BTreeMap;

use serde::Serialize;
pub use vortex_kernel::{Answer, Mechanism, Response, Task};

pub use report::{Report, SuiteResult};
pub use run::{run_all, run_suite};

/// The system under evaluation.
pub trait Agent: Sync {
    fn name(&self) -> String;
    fn respond(&self, task: &Task) -> Response;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    Language,
    Knowledge,
    Math,
    Code,
    Grid,
    Spatial2d,
    Spatial3dTime,
    Safety,
    ToolUse,
}

#[derive(Clone, Debug)]
pub struct Item {
    pub id: String,
    pub task: Task,
}

pub trait Suite: Sync {
    fn id(&self) -> &str;
    fn capability(&self) -> Capability;
    /// Expected score of a uniformly random valid answer.
    fn chance(&self) -> f64;
    /// Dataset, split and sampling, for the report.
    fn source(&self) -> String;
    /// Independently authored and hash-pinned.
    fn sealed(&self) -> bool {
        false
    }
    fn items(&self) -> &[Item];
    /// Score in `[0, 1]` for item `index`. Partial credit allowed.
    fn grade(&self, index: usize, response: &Response) -> f64;
    /// Suite-specific metrics (e.g. over-refusal), computed from all responses.
    fn summarize(&self, _responses: &[Response]) -> BTreeMap<String, f64> {
        BTreeMap::new()
    }
    /// The text a model sees per item, for contamination checks.
    fn contamination_texts(&self) -> Vec<String> {
        vec![]
    }
}
