//! Tasks, answers and responses.
//!
//! The integrity rule this module exists to make structural: a [`Task`]
//! carries the *question only*. Gold labels live inside each harness suite and
//! are never part of any type an agent can receive.

use serde::{Deserialize, Serialize};

use crate::guardian::{GuardianCategory, Verdict};
use crate::physics::KinematicsQuery;
use crate::spatial::{SpatialChallenge, SpatialSubmission};
use crate::tool::{ActionClass, ToolCall, ToolSpec};

/// ARC-style integer grid, colours 0-9 (0-15 tolerated).
pub type Grid = Vec<Vec<u8>>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GridTask {
    /// Demonstration pairs (input, output).
    pub train: Vec<(Grid, Grid)>,
    /// The input whose output must be predicted. Its output is never given.
    pub test_input: Grid,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Task {
    /// Multiple choice. `context` may be empty.
    Choice { context: String, question: String, choices: Vec<String> },
    /// Free-form answer (math word problems, code, prose).
    Free { prompt: String },
    /// Free-form code completion: complete `prompt` so `entry_point` passes
    /// hidden tests.
    Code { prompt: String, entry_point: String },
    /// Few-shot grid transformation (ARC).
    Grid(GridTask),
    /// Astra navigation / clearance / assembly.
    Spatial(SpatialChallenge),
    /// 3D + time kinematics.
    Kinematics(KinematicsQuery),
    /// Content moderation under the Guardian policy.
    Moderate { text: String },
    /// Classify a proposed tool call's risk.
    ActionRisk { call: ToolCall },
    /// Choose and parameterise tool calls for an instruction.
    UseTools { instruction: String, tools: Vec<ToolSpec> },
}

impl Task {
    pub fn kind(&self) -> &'static str {
        match self {
            Task::Choice { .. } => "choice",
            Task::Free { .. } => "free",
            Task::Code { .. } => "code",
            Task::Grid(_) => "grid",
            Task::Spatial(_) => "spatial",
            Task::Kinematics(_) => "kinematics",
            Task::Moderate { .. } => "moderate",
            Task::ActionRisk { .. } => "action_risk",
            Task::UseTools { .. } => "use_tools",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Answer {
    Choice(usize),
    Text(String),
    Grid(Grid),
    Spatial(SpatialSubmission),
    Numbers(Vec<f64>),
    Moderation { verdict: Verdict, category: Option<GuardianCategory> },
    Action(ActionClass),
    Calls(Vec<ToolCall>),
    /// The agent declines to answer. Graded as wrong; reported separately so
    /// "can't" is distinguishable from "got it wrong".
    Abstain { reason: String },
}

/// How an answer was produced. Reported per suite so engineered solvers are
/// never mistaken for learned capability.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Mechanism {
    /// Parameters fitted from data (n-gram counts, trained weights).
    Learned,
    /// Search over a program/action space (DSL search, graph search).
    Search,
    /// Hand-written rules or closed-form solvers.
    Engineered,
    /// Abstained.
    None,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Response {
    pub answer: Answer,
    /// The agent's own probability that the answer is correct, in `[0, 1]`.
    /// Scored for calibration (Brier): knowing what it knows is measured.
    pub confidence: f64,
    pub mechanism: Mechanism,
}

impl Response {
    pub fn new(answer: Answer, confidence: f64, mechanism: Mechanism) -> Self {
        Self { answer, confidence: confidence.clamp(0.0, 1.0), mechanism }
    }

    pub fn abstain(reason: impl Into<String>) -> Self {
        Self {
            answer: Answer::Abstain { reason: reason.into() },
            confidence: 0.0,
            mechanism: Mechanism::None,
        }
    }

    pub fn is_abstain(&self) -> bool {
        matches!(self.answer, Answer::Abstain { .. })
    }
}
