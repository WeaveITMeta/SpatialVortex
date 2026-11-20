//! `tools`: the instruction router and the exact calculator (006).

use serde::Deserialize;
use serde_json::{Value, json};
use vortex_kernel::ToolSpec;

use super::{Module, method, object, parse, to_value, tool_spec_schema};
use crate::json::{ApiError, Method};
use crate::tools::{Calc, Tool};
use crate::vortex::Vortex;

pub struct Tools;

impl Module for Tools {
    fn id(&self) -> &str {
        "tools"
    }

    fn methods(&self) -> Vec<Method> {
        vec![
            method(
                "tools.route",
                "Turn an instruction plus the offered tools into concrete tool calls. Executes nothing.",
                object(json!({ "instruction": { "type": "string" }, "tools": { "type": "array", "items": tool_spec_schema() } }), &["instruction", "tools"]),
            ),
            method("tools.calc", "Evaluate an arithmetic expression exactly (+ - * / ^ %, parentheses, unary minus).", object(json!({ "expression": { "type": "string" } }), &["expression"])),
        ]
    }

    fn call(&self, vortex: &Vortex, m: &str, params: Value) -> Result<Value, ApiError> {
        match m {
            "tools.route" => {
                #[derive(Deserialize)]
                struct P {
                    instruction: String,
                    tools: Vec<ToolSpec>,
                }
                let p: P = parse(m, params)?;
                Ok(json!({ "calls": to_value(m, &vortex.route(&p.instruction, &p.tools))? }))
            }
            "tools.calc" => {
                #[derive(Deserialize)]
                struct P {
                    expression: String,
                }
                let p: P = parse(m, params)?;
                Calc.call(&json!({ "expression": p.expression })).map_err(|e| ApiError::BadParams { method: m.to_string(), message: e.to_string() })
            }
            other => Err(ApiError::UnknownMethod(other.to_string())),
        }
    }
}
