//! Tool-calling vocabulary. Shared by the registry (006), the safety lane
//! (004) and the MCP client, so a tool's declared effect travels with it.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// What a tool can do to the world. Declared by the tool, verified by the
/// safety lane, enforced by the constitution.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Effect {
    /// Observes only. Safe to run in parallel and to retry.
    ReadOnly,
    /// Changes local state that can be undone (a file inside the workspace).
    LocalWrite,
    /// Reaches outside the machine (network, messages, publishing).
    External,
    /// Cannot be undone (deletion, force-push, payments).
    Irreversible,
}

impl Effect {
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s.trim().to_ascii_lowercase().as_str() {
            "read_only" | "readonly" => Self::ReadOnly,
            "local_write" => Self::LocalWrite,
            "external" => Self::External,
            "irreversible" => Self::Irreversible,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParamKind {
    String,
    Number,
    Integer,
    Boolean,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ParamSpec {
    pub name: String,
    pub kind: ParamKind,
    #[serde(default = "yes")]
    pub required: bool,
}

fn yes() -> bool {
    true
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub params: Vec<ParamSpec>,
    pub effect: Effect,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool: String,
    #[serde(default)]
    pub args: Value,
}

impl ToolCall {
    pub fn new(tool: impl Into<String>, args: Value) -> Self {
        Self { tool: tool.into(), args }
    }

    /// A string argument, if present.
    pub fn arg_str(&self, name: &str) -> Option<&str> {
        self.args.get(name).and_then(Value::as_str)
    }
}

/// Safety-lane decision for a proposed action, aligned with how a careful
/// human operator treats tool calls on their own machine.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ActionClass {
    /// Read-only or trivially reversible and local: proceed.
    Allow,
    /// Irreversible or outward-facing: requires explicit human approval.
    Confirm,
    /// Never, even with approval (secret exfiltration, disabling security).
    Deny,
}

impl ActionClass {
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s.trim().to_ascii_lowercase().as_str() {
            "allow" => Self::Allow,
            "confirm" => Self::Confirm,
            "deny" => Self::Deny,
            _ => return None,
        })
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Confirm => "confirm",
            Self::Deny => "deny",
        }
    }
}
