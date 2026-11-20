//! `safety`: the content lane, the action lane, and the gate (004 + 003).
//! Each method decides; none executes.

use serde::Deserialize;
use serde_json::{Value, json};
use vortex_kernel::{Effect, ToolCall};

use super::{Module, approvals_schema, method, object, parse, to_value};
use crate::constitution::Approval;
use crate::json::{ApiError, Method};
use crate::vortex::Vortex;

pub struct Safety;

impl Module for Safety {
    fn id(&self) -> &str {
        "safety"
    }

    fn methods(&self) -> Vec<Method> {
        vec![
            method("safety.moderate", "Guardian Policy v1.2 moderation verdict for a text, with category, confidence and evidence.", object(json!({ "text": { "type": "string" } }), &["text"])),
            method(
                "safety.assess_action",
                "Risk class (allow, confirm, deny) of a proposed tool call, read from the call itself.",
                object(json!({ "tool": { "type": "string" }, "args": { "description": "The call's arguments; omitted means null." } }), &["tool"]),
            ),
            method(
                "safety.gate",
                "Gate one tool call: safety assessment plus constitution ruling, given its declared effect and any approvals. Decides run, ask_human or refuse; executes nothing.",
                object(
                    json!({
                        "tool": { "type": "string" },
                        "args": { "description": "The call's arguments; omitted means null." },
                        "effect": { "type": "string", "enum": ["read_only", "local_write", "external", "irreversible"], "description": "What the tool can do to the world. Omitted means local_write: a tool that declares nothing is never treated as safe." },
                        "approvals": approvals_schema()
                    }),
                    &["tool"],
                ),
            ),
        ]
    }

    fn call(&self, vortex: &Vortex, m: &str, params: Value) -> Result<Value, ApiError> {
        match m {
            "safety.moderate" => {
                #[derive(Deserialize)]
                struct P {
                    text: String,
                }
                let p: P = parse(m, params)?;
                to_value(m, &vortex.moderate(&p.text))
            }
            "safety.assess_action" => {
                let call: ToolCall = parse(m, params)?;
                to_value(m, &vortex.assess(&call))
            }
            "safety.gate" => {
                #[derive(Deserialize)]
                struct P {
                    tool: String,
                    #[serde(default)]
                    args: Value,
                    #[serde(default)]
                    effect: Option<String>,
                    #[serde(default)]
                    approvals: Vec<Approval>,
                }
                let p: P = parse(m, params)?;
                let effect = match p.effect.as_deref() {
                    None => Effect::LocalWrite,
                    Some(s) => Effect::parse(s).ok_or_else(|| ApiError::BadParams {
                        method: m.to_string(),
                        message: format!("unknown effect `{s}`: expected read_only, local_write, external or irreversible"),
                    })?,
                };
                let call = ToolCall::new(p.tool, p.args);
                to_value(m, &vortex.gate(&call, effect, &p.approvals))
            }
            other => Err(ApiError::UnknownMethod(other.to_string())),
        }
    }
}
