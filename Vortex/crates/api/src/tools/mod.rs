//! # 006 | Vortex Tools
//!
//! How Vortex acts on the world.
//!
//! - [`registry`]: tools behind one interface. Every call passes a caller-
//!   supplied gate (the pipeline wires safety 004 + constitution 003 into it)
//!   before it runs. Read-only calls run **in parallel**; anything that writes
//!   runs sequentially in the order given.
//! - [`Calc`]: the calculator tool, the one built-in with no side effects.
//!   The sandboxed filesystem, clock and gated shell tools, and the Model
//!   Context Protocol client, touch the host and live in `vortex-lab`
//!   (`host::builtin`, `host::mcp`); they plug in through [`Tool`].
//! - [`router`]: turns an English instruction plus a tool list into concrete
//!   calls, splitting independent requests so they can run in parallel.
//! - [`calc`]: the exact arithmetic evaluator behind the calculator.

pub mod calc;
pub mod registry;
pub mod router;

pub use registry::{GateDecision, Outcome, Registry, Tool, ToolError};
pub use router::route;

use serde_json::{Value, json};
use vortex_kernel::{Effect, ParamKind, ParamSpec, ToolSpec};

fn p(name: &str, kind: ParamKind) -> ParamSpec {
    ParamSpec { name: name.into(), kind, required: true }
}

fn str_arg<'a>(args: &'a Value, name: &str) -> Result<&'a str, ToolError> {
    args.get(name).and_then(Value::as_str).ok_or_else(|| ToolError::BadArgs(format!("missing string `{name}`")))
}

pub struct Calc;

impl Tool for Calc {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "calc".into(),
            description: "Evaluate an arithmetic expression".into(),
            params: vec![p("expression", ParamKind::String)],
            effect: Effect::ReadOnly,
        }
    }
    fn call(&self, args: &Value) -> Result<Value, ToolError> {
        let e = str_arg(args, "expression")?;
        let v = calc::eval(e).map_err(|e| ToolError::BadArgs(e.to_string()))?;
        Ok(json!({ "value": v, "text": calc::format_number(v) }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calc_tool() {
        let v = Calc.call(&json!({"expression": "6*7"})).unwrap();
        assert_eq!(v["text"], "42");
    }
}
