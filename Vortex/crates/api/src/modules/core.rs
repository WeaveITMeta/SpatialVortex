//! `core`: the orchestrator (001) at the door. What this handle is, the rules
//! it runs under, one task answered, and one request planned.

use serde::Deserialize;
use serde_json::{Value, json};
use vortex_kernel::{Task, ToolSpec};

use super::{Module, approvals_schema, method, no_params, no_params_given, object, parse, task_schema, to_value, tool_spec_schema};
use crate::constitution::Approval;
use crate::json::{ApiError, Method};
use crate::vortex::Vortex;

pub struct Core;

impl Module for Core {
    fn id(&self) -> &str {
        "core"
    }

    fn methods(&self) -> Vec<Method> {
        vec![
            method("core.info", "Vortex version, constitution version and hash, agent name, compiled features, registered modules and method names.", no_params()),
            method("core.constitution", "The constitution's version, hash and articles.", no_params()),
            method(
                "core.respond",
                "Answer one task (multiple choice, arithmetic, grid, spatial, kinematics, moderation, action risk, tool use). Returns a Response with answer, confidence and mechanism.",
                object(json!({ "task": task_schema() }), &["task"]),
            ),
            method(
                "core.plan",
                "Moderate a request, rule on answering it, and if permitted route it and gate every call. The plan for a host that owns execution; nothing is executed.",
                object(json!({ "request": { "type": "string" }, "tools": { "type": "array", "items": tool_spec_schema() }, "approvals": approvals_schema() }), &["request", "tools"]),
            ),
        ]
    }

    fn call(&self, vortex: &Vortex, m: &str, params: Value) -> Result<Value, ApiError> {
        match m {
            "core.info" => {
                no_params_given(m, &params)?;
                let mut features: Vec<&str> = Vec::new();
                if cfg!(feature = "world") {
                    features.push("world");
                }
                Ok(json!({
                    "version": env!("CARGO_PKG_VERSION"),
                    "constitution": { "version": vortex.constitution().version(), "hash": vortex.constitution().hash() },
                    "agent": vortex.name(),
                    "features": features,
                    "modules": vortex.modules(),
                    "methods": vortex.methods().into_iter().map(|x| x.name).collect::<Vec<_>>(),
                }))
            }
            "core.constitution" => {
                no_params_given(m, &params)?;
                let c = vortex.constitution();
                Ok(json!({ "version": c.version(), "hash": c.hash(), "articles": c.articles() }))
            }
            "core.respond" => {
                #[derive(Deserialize)]
                struct P {
                    task: Task,
                }
                let p: P = parse(m, params)?;
                to_value(m, &vortex.respond(&p.task))
            }
            "core.plan" => {
                #[derive(Deserialize)]
                struct P {
                    request: String,
                    tools: Vec<ToolSpec>,
                    #[serde(default)]
                    approvals: Vec<Approval>,
                }
                let p: P = parse(m, params)?;
                to_value(m, &vortex.plan(&p.request, &p.tools, &p.approvals))
            }
            other => Err(ApiError::UnknownMethod(other.to_string())),
        }
    }
}
