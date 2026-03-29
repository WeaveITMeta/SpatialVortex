//! Shared types for the Vortex learning architecture.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique identifier for a simulation branch.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BranchId(pub uuid::Uuid);

impl BranchId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl fmt::Display for BranchId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.0.to_string()[..8])
    }
}

/// Domain tag — which world type a node or rule applies to.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Domain {
    Grid2D,
    Scene3D,
    GameState,
    Universal,
    Custom(String),
}

/// Value type for state variables in the causal graph.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ValueType {
    Bool,
    Int,
    Float,
    Enum(Vec<String>),
    Vec(Box<ValueType>),
}

/// A detected structural property of a world state.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Property {
    pub name: String,
    pub domain: Domain,
    pub value: PropertyValue,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PropertyValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Vec(Vec<PropertyValue>),
}

impl PropertyValue {
    pub fn as_bool(&self) -> bool {
        match self {
            PropertyValue::Bool(b) => *b,
            _ => false,
        }
    }

    pub fn as_int(&self) -> i64 {
        match self {
            PropertyValue::Int(i) => *i,
            _ => 0,
        }
    }

    pub fn as_float(&self) -> f64 {
        match self {
            PropertyValue::Float(f) => *f,
            _ => 0.0,
        }
    }
}

/// A goal predicate that the solver tries to satisfy.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GoalPredicate {
    /// Exact state match (ARC-style).
    ExactMatch,
    /// Score exceeds threshold.
    ScoreThreshold(f64),
    /// Custom predicate name + parameters.
    Custom { name: String, params: serde_json::Value },
}

/// Score from evaluating a world state against a goal.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Score {
    /// Did it solve perfectly?
    pub exact_match: bool,
    /// Continuous accuracy [0.0, 1.0].
    pub accuracy: f64,
    /// Domain-specific breakdown.
    pub details: serde_json::Value,
}

impl Score {
    pub fn perfect() -> Self {
        Self { exact_match: true, accuracy: 1.0, details: serde_json::Value::Null }
    }

    pub fn zero() -> Self {
        Self { exact_match: false, accuracy: 0.0, details: serde_json::Value::Null }
    }

    pub fn from_accuracy(accuracy: f64) -> Self {
        Self { exact_match: accuracy >= 1.0 - f64::EPSILON, accuracy, details: serde_json::Value::Null }
    }
}

/// A change between two world states (for Iggy deltas).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Delta {
    pub kind: String,
    pub description: String,
    pub magnitude: f64,
}

/// DSL operation — a domain-specific action primitive.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DSLOp {
    pub name: String,
    pub domain: Domain,
    pub parameters: Vec<serde_json::Value>,
}

impl fmt::Display for DSLOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.parameters.is_empty() {
            write!(f, "{}", self.name)
        } else {
            write!(f, "{}({})", self.name, self.parameters.iter()
                .map(|p| p.to_string()).collect::<Vec<_>>().join(", "))
        }
    }
}

/// Parameter specification for an action node.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParameterSpec {
    pub name: String,
    pub value_type: ValueType,
    pub default: Option<serde_json::Value>,
}

/// Distribution of observed values (for causal edge strength).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Distribution {
    pub mean: f64,
    pub variance: f64,
    pub samples: usize,
}

impl Distribution {
    pub fn single(value: f64) -> Self {
        Self { mean: value, variance: 0.0, samples: 1 }
    }

    pub fn update(&mut self, value: f64) {
        let n = self.samples as f64;
        let new_n = n + 1.0;
        let delta = value - self.mean;
        self.mean += delta / new_n;
        self.variance = (self.variance * n + delta * (value - self.mean)) / new_n;
        self.samples += 1;
    }
}

/// Record of a completed episode for causal graph integration.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EpisodeRecord {
    pub episode_id: String,
    pub task_id: String,
    pub observed_properties: Vec<Property>,
    pub actions_taken: Vec<DSLOp>,
    pub state_deltas: Vec<Delta>,
    pub final_score: Score,
    pub success: bool,
    pub duration_ms: u64,
}

/// Composition order for composable causal laws.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CompositionOrder {
    Sequential,
    Commutative,
}
