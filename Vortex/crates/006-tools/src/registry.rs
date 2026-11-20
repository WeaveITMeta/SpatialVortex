//! Tool registry with gated, parallel dispatch.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Instant;

use serde::Serialize;
use serde_json::Value;
use vortex_kernel::{Effect, ToolCall, ToolSpec};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum ToolError {
    UnknownTool(String),
    BadArgs(String),
    Failed(String),
    /// The gate refused the call; carries the reason.
    Refused(String),
    /// The gate wants a human to approve this exact call first.
    NeedsApproval(String),
}

impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolError::UnknownTool(t) => write!(f, "unknown tool `{t}`"),
            ToolError::BadArgs(s) => write!(f, "bad arguments: {s}"),
            ToolError::Failed(s) => write!(f, "tool failed: {s}"),
            ToolError::Refused(s) => write!(f, "refused: {s}"),
            ToolError::NeedsApproval(s) => write!(f, "needs human approval: {s}"),
        }
    }
}

pub trait Tool: Send + Sync {
    fn spec(&self) -> ToolSpec;
    fn call(&self, args: &Value) -> Result<Value, ToolError>;
}

/// What the gate decided for one call.
#[derive(Debug, Clone, PartialEq)]
pub enum GateDecision {
    Run,
    Refuse(String),
    AskHuman(String),
}

#[derive(Debug, Clone, Serialize)]
pub struct Outcome {
    pub call: ToolCall,
    pub result: Result<Value, ToolError>,
    pub micros: u128,
}

#[derive(Default, Clone)]
pub struct Registry {
    tools: BTreeMap<String, Arc<dyn Tool>>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, tool: impl Tool + 'static) {
        let name = tool.spec().name;
        self.tools.insert(name, Arc::new(tool));
    }

    pub fn register_arc(&mut self, tool: Arc<dyn Tool>) {
        self.tools.insert(tool.spec().name, tool);
    }

    pub fn specs(&self) -> Vec<ToolSpec> {
        self.tools.values().map(|t| t.spec()).collect()
    }

    pub fn get(&self, name: &str) -> Option<&Arc<dyn Tool>> {
        self.tools.get(name)
    }

    pub fn len(&self) -> usize {
        self.tools.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    /// Run `calls` through `gate`. Read-only calls that pass run concurrently
    /// (one scoped thread each); every other call runs afterwards, in order,
    /// so writes never race each other. Results come back in input order.
    pub fn execute<G>(&self, calls: &[ToolCall], gate: G) -> Vec<Outcome>
    where
        G: Fn(&ToolCall, &ToolSpec) -> GateDecision + Sync,
    {
        let mut slots: Vec<Option<Outcome>> = vec![None; calls.len()];
        let mut parallel = Vec::new();
        let mut sequential = Vec::new();

        for (i, call) in calls.iter().enumerate() {
            let Some(tool) = self.tools.get(&call.tool) else {
                slots[i] = Some(Outcome { call: call.clone(), result: Err(ToolError::UnknownTool(call.tool.clone())), micros: 0 });
                continue;
            };
            let spec = tool.spec();
            match gate(call, &spec) {
                GateDecision::Run => {
                    if spec.effect == Effect::ReadOnly {
                        parallel.push((i, tool.clone()));
                    } else {
                        sequential.push((i, tool.clone()));
                    }
                }
                GateDecision::Refuse(r) => {
                    slots[i] = Some(Outcome { call: call.clone(), result: Err(ToolError::Refused(r)), micros: 0 })
                }
                GateDecision::AskHuman(r) => {
                    slots[i] = Some(Outcome { call: call.clone(), result: Err(ToolError::NeedsApproval(r)), micros: 0 })
                }
            }
        }

        let run = |tool: &Arc<dyn Tool>, call: &ToolCall| {
            let t0 = Instant::now();
            let result = tool.call(&call.args);
            Outcome { call: call.clone(), result, micros: t0.elapsed().as_micros() }
        };

        std::thread::scope(|s| {
            let handles: Vec<_> = parallel
                .iter()
                .map(|(i, tool)| {
                    let call = &calls[*i];
                    (*i, s.spawn(move || run(tool, call)))
                })
                .collect();
            for (i, h) in handles {
                slots[i] = Some(h.join().unwrap_or_else(|_| Outcome {
                    call: calls[i].clone(),
                    result: Err(ToolError::Failed("tool panicked".into())),
                    micros: 0,
                }));
            }
        });
        for (i, tool) in sequential {
            slots[i] = Some(run(&tool, &calls[i]));
        }
        slots.into_iter().map(|o| o.expect("every slot filled")).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::time::Duration;

    struct Sleepy(&'static str, Effect);
    impl Tool for Sleepy {
        fn spec(&self) -> ToolSpec {
            ToolSpec { name: self.0.into(), description: "sleeps".into(), params: vec![], effect: self.1 }
        }
        fn call(&self, _: &Value) -> Result<Value, ToolError> {
            std::thread::sleep(Duration::from_millis(150));
            Ok(json!(self.0))
        }
    }

    #[test]
    fn read_only_calls_run_in_parallel_and_keep_order() {
        let mut r = Registry::new();
        r.register(Sleepy("a", Effect::ReadOnly));
        r.register(Sleepy("b", Effect::ReadOnly));
        r.register(Sleepy("c", Effect::ReadOnly));
        let calls: Vec<_> = ["a", "b", "c"].iter().map(|n| ToolCall::new(*n, json!({}))).collect();
        let t0 = Instant::now();
        let out = r.execute(&calls, |_, _| GateDecision::Run);
        let took = t0.elapsed();
        assert!(took < Duration::from_millis(400), "3 x 150ms read-only calls took {took:?}; not parallel");
        let names: Vec<_> = out.iter().map(|o| o.result.clone().unwrap()).collect();
        assert_eq!(names, vec![json!("a"), json!("b"), json!("c")]);
    }

    #[test]
    fn gate_refusals_and_unknown_tools_are_reported_not_run() {
        let mut r = Registry::new();
        r.register(Sleepy("w", Effect::LocalWrite));
        let calls = vec![ToolCall::new("w", json!({})), ToolCall::new("nope", json!({}))];
        let out = r.execute(&calls, |_, _| GateDecision::Refuse("test".into()));
        assert_eq!(out[0].result, Err(ToolError::Refused("test".into())));
        assert_eq!(out[1].result, Err(ToolError::UnknownTool("nope".into())));
    }
}
