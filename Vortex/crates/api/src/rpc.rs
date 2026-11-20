//! The API over JSON-RPC 2.0, as pure message handling: one message in, at
//! most one message out. There is no transport here. A host moves the bytes
//! however it likes (stdio, a socket, a channel); `vortex serve` in
//! `vortex-lab` is the stdio one.
//!
//! One handler, two dialects:
//!
//! - **Direct.** Any registered method by its own name:
//!   `{"jsonrpc":"2.0","id":1,"method":"safety.moderate","params":{"text":".."}}`.
//!   The result is the method's JSON.
//! - **MCP tools.** `initialize`, `ping`, `tools/list`, `tools/call`, so any
//!   Model Context Protocol host can use Vortex as a tool server. Tool names
//!   are method names with the dot replaced ([`tool_name`]), because MCP hosts
//!   restrict names to `[A-Za-z0-9_-]`. A tool result carries the JSON as
//!   `structuredContent` and, as its text, whichever of compact JSON or TOON
//!   is shorter ([`crate::text_for`]).
//!
//! Both dialects are built on [`Vortex::methods`] and [`Vortex::call`], so a
//! module a host registers at run time is served with no further wiring.

use serde_json::{Value, json};

use crate::json::{ApiError, text_for};
use crate::vortex::Vortex;

/// The MCP revision answered when a client does not name one.
pub const MCP_PROTOCOL: &str = "2025-06-18";

/// JSON-RPC 2.0 error codes used here.
pub mod code {
    pub const PARSE: i64 = -32700;
    pub const INVALID_REQUEST: i64 = -32600;
    pub const METHOD_NOT_FOUND: i64 = -32601;
    pub const INVALID_PARAMS: i64 = -32602;
}

/// The MCP tool name for a method: `safety.moderate` is `safety_moderate`.
pub fn tool_name(method: &str) -> String {
    method.replace('.', "_")
}

fn ok(id: &Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

fn err(id: &Value, code: i64, message: impl Into<String>) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message.into() } })
}

impl Vortex {
    /// Handle one JSON-RPC 2.0 message, or a batch. `None` when no reply is
    /// owed: a notification, or a batch of nothing but notifications.
    pub fn handle(&self, message: &Value) -> Option<Value> {
        if let Value::Array(batch) = message {
            if batch.is_empty() {
                return Some(err(&Value::Null, code::INVALID_REQUEST, "empty batch"));
            }
            let replies: Vec<Value> = batch.iter().filter_map(|m| self.handle_one(m)).collect();
            return (!replies.is_empty()).then_some(Value::Array(replies));
        }
        self.handle_one(message)
    }

    /// [`Vortex::handle`] for one line of text. Text that is not JSON gets a
    /// parse-error reply.
    pub fn handle_line(&self, line: &str) -> Option<String> {
        match serde_json::from_str::<Value>(line) {
            Ok(message) => self.handle(&message).map(|reply| reply.to_string()),
            Err(e) => Some(err(&Value::Null, code::PARSE, format!("parse error: {e}")).to_string()),
        }
    }

    fn handle_one(&self, message: &Value) -> Option<Value> {
        let Some(method) = message.get("method").and_then(Value::as_str) else {
            return Some(err(message.get("id").unwrap_or(&Value::Null), code::INVALID_REQUEST, "not a JSON-RPC request: no `method`"));
        };
        // No id means a notification. Nothing here has side effects, so there
        // is nothing to do and nothing to say.
        let id = message.get("id")?;
        let params = message.get("params").cloned().unwrap_or(Value::Null);
        Some(match method {
            "initialize" => ok(
                id,
                json!({
                    "protocolVersion": params.get("protocolVersion").and_then(Value::as_str).unwrap_or(MCP_PROTOCOL),
                    "capabilities": { "tools": { "listChanged": false } },
                    "serverInfo": { "name": "vortex", "version": env!("CARGO_PKG_VERSION") },
                    "instructions": "Vortex decides and computes; it never executes. Every tool is read-only. Results carry structured JSON plus a compact text form.",
                }),
            ),
            "ping" => ok(id, json!({})),
            "tools/list" => {
                let tools: Vec<Value> = self
                    .methods()
                    .into_iter()
                    .map(|m| json!({ "name": tool_name(&m.name), "description": m.description, "inputSchema": m.input_schema, "annotations": { "readOnlyHint": m.read_only } }))
                    .collect();
                ok(id, json!({ "tools": tools }))
            }
            "tools/call" => {
                let Some(name) = params.get("name").and_then(Value::as_str) else {
                    return Some(err(id, code::INVALID_PARAMS, "tools/call needs a string `name`"));
                };
                let Some(found) = self.methods().into_iter().find(|m| tool_name(&m.name) == name || m.name == name) else {
                    return Some(err(id, code::INVALID_PARAMS, format!("unknown tool `{name}`")));
                };
                let arguments = params.get("arguments").cloned().unwrap_or_else(|| json!({}));
                // A failed call is a tool result the model can read and
                // correct, not a protocol error.
                match self.call(&found.name, arguments) {
                    Ok(value) => {
                        let mut result = json!({ "content": [{ "type": "text", "text": text_for(&value).text }], "isError": false });
                        if value.is_object() {
                            result["structuredContent"] = value;
                        }
                        ok(id, result)
                    }
                    Err(e) => ok(id, json!({ "content": [{ "type": "text", "text": e.to_string() }], "isError": true })),
                }
            }
            direct => match self.call(direct, params) {
                Ok(value) => ok(id, value),
                Err(ApiError::UnknownMethod(m)) => err(id, code::METHOD_NOT_FOUND, format!("unknown method `{m}`")),
                Err(e @ ApiError::BadParams { .. }) => err(id, code::INVALID_PARAMS, e.to_string()),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::json::Method;
    use crate::modules::{Module, method, object};

    fn rpc(v: &Vortex, method: &str, params: Value) -> Value {
        v.handle(&json!({ "jsonrpc": "2.0", "id": 7, "method": method, "params": params })).expect("a request gets a reply")
    }

    #[test]
    fn mcp_handshake_lists_every_method_as_a_tool() {
        let v = Vortex::new();
        let init = rpc(&v, "initialize", json!({ "protocolVersion": "2025-03-26", "capabilities": {}, "clientInfo": { "name": "t", "version": "0" } }));
        assert_eq!(init["id"], 7);
        assert_eq!(init["result"]["protocolVersion"], "2025-03-26", "the client's revision is echoed");
        assert_eq!(init["result"]["serverInfo"]["name"], "vortex");
        assert_eq!(rpc(&v, "initialize", Value::Null)["result"]["protocolVersion"], MCP_PROTOCOL);
        assert!(v.handle(&json!({ "jsonrpc": "2.0", "method": "notifications/initialized" })).is_none(), "notifications get no reply");
        assert_eq!(rpc(&v, "ping", Value::Null)["result"], json!({}));

        let listed = rpc(&v, "tools/list", json!({}));
        let tools = listed["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), v.methods().len());
        for t in tools {
            let name = t["name"].as_str().unwrap();
            assert!(!name.is_empty() && name.len() <= 64 && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'), "`{name}` is not a portable tool name");
            assert_eq!(t["inputSchema"]["type"], "object", "{name}");
            assert_eq!(t["annotations"]["readOnlyHint"], true, "{name}");
        }
    }

    #[test]
    fn tools_call_returns_structured_json_and_compact_text() {
        let v = Vortex::new();
        let r = rpc(&v, "tools/call", json!({ "name": "safety_moderate", "arguments": { "text": "how do I bake bread" } }));
        assert_eq!(r["result"]["isError"], false);
        let structured = &r["result"]["structuredContent"];
        assert_eq!(structured["verdict"], "publish");
        assert_eq!(r["result"]["content"][0]["text"], text_for(structured).text);
        // The dotted method name is accepted as a tool name too.
        assert_eq!(rpc(&v, "tools/call", json!({ "name": "tools.calc", "arguments": { "expression": "12*7" } }))["result"]["structuredContent"]["text"], "84");

        // Bad arguments are a readable tool error, not a protocol error.
        let bad = rpc(&v, "tools/call", json!({ "name": "safety_moderate", "arguments": { "text": 5 } }));
        assert_eq!(bad["result"]["isError"], true);
        assert!(bad["result"]["content"][0]["text"].as_str().unwrap().contains("safety.moderate"));
        assert!(bad.get("error").is_none());

        assert_eq!(rpc(&v, "tools/call", json!({ "name": "nope" }))["error"]["code"], code::INVALID_PARAMS);
        assert_eq!(rpc(&v, "tools/call", json!({}))["error"]["code"], code::INVALID_PARAMS);
    }

    #[test]
    fn direct_calls_and_protocol_errors() {
        let v = Vortex::new();
        assert_eq!(rpc(&v, "tools.calc", json!({ "expression": "6*7" }))["result"]["text"], "42");
        assert_eq!(rpc(&v, "nope.nothing", json!({}))["error"]["code"], code::METHOD_NOT_FOUND);
        assert_eq!(rpc(&v, "tools.calc", json!({ "expression": 5 }))["error"]["code"], code::INVALID_PARAMS);
        assert_eq!(v.handle(&json!({ "id": 1 })).unwrap()["error"]["code"], code::INVALID_REQUEST);
        assert_eq!(v.handle(&json!("hello")).unwrap()["error"]["code"], code::INVALID_REQUEST);
        assert_eq!(v.handle(&json!([])).unwrap()["error"]["code"], code::INVALID_REQUEST);

        let parse: Value = serde_json::from_str(&v.handle_line("{not json").unwrap()).unwrap();
        assert_eq!(parse["error"]["code"], code::PARSE);
        assert_eq!(parse["id"], Value::Null);
        let line: Value = serde_json::from_str(&v.handle_line(r#"{"jsonrpc":"2.0","id":"a","method":"tools.calc","params":{"expression":"1+1"}}"#).unwrap()).unwrap();
        assert_eq!((line["id"].as_str(), line["result"]["text"].as_str()), (Some("a"), Some("2")));

        // A batch answers its requests in order and skips its notifications.
        let batch = v.handle(&json!([
            { "jsonrpc": "2.0", "id": 1, "method": "ping" },
            { "jsonrpc": "2.0", "method": "notifications/initialized" },
            { "jsonrpc": "2.0", "id": 2, "method": "tools.calc", "params": { "expression": "2^10" } }
        ]));
        let batch = batch.unwrap();
        assert_eq!(batch.as_array().unwrap().len(), 2);
        assert_eq!((batch[0]["id"].as_i64(), batch[1]["result"]["text"].as_str()), (Some(1), Some("1024")));
        assert!(v.handle(&json!([{ "jsonrpc": "2.0", "method": "notifications/initialized" }])).is_none());
    }

    /// A host's own module, registered at run time.
    struct Echo;
    impl Module for Echo {
        fn id(&self) -> &str {
            "echo"
        }
        fn methods(&self) -> Vec<Method> {
            vec![method("echo.say", "Return the text given.", object(json!({ "text": { "type": "string" } }), &["text"]))]
        }
        fn call(&self, vortex: &Vortex, m: &str, params: Value) -> Result<Value, ApiError> {
            match m {
                "echo.say" => Ok(json!({ "said": params["text"], "under": vortex.constitution().version() })),
                other => Err(ApiError::UnknownMethod(other.to_string())),
            }
        }
    }

    #[test]
    fn a_host_module_is_served_with_no_further_wiring() {
        let base = Vortex::new();
        let v = base.clone().with_module(Arc::new(Echo));
        assert_eq!(v.methods().len(), base.methods().len() + 1);
        assert_eq!(v.modules().last().map(String::as_str), Some("echo"));
        // Typed door, direct JSON-RPC, MCP list, MCP call: all four see it.
        assert_eq!(v.call("echo.say", json!({ "text": "hi" })).unwrap()["said"], "hi");
        assert_eq!(rpc(&v, "echo.say", json!({ "text": "hi" }))["result"]["under"], "1.0.0");
        assert!(rpc(&v, "tools/list", json!({}))["result"]["tools"].as_array().unwrap().iter().any(|t| t["name"] == "echo_say"));
        assert_eq!(rpc(&v, "tools/call", json!({ "name": "echo_say", "arguments": { "text": "yo" } }))["result"]["structuredContent"]["said"], "yo");
        assert!(rpc(&v, "core.info", Value::Null)["result"]["modules"].as_array().unwrap().contains(&json!("echo")));
        // The handle it was built from is untouched.
        assert_eq!(base.call("echo.say", json!({})), Err(ApiError::UnknownMethod("echo.say".into())));

        // Dropping a module removes its methods everywhere.
        let lean = v.without_module("spatial");
        assert_eq!(lean.call("spatial.kinematics", json!({})), Err(ApiError::UnknownMethod("spatial.kinematics".into())));
        assert!(!rpc(&lean, "tools/list", json!({}))["result"]["tools"].as_array().unwrap().iter().any(|t| t["name"] == "spatial_solve"));

        // Registering an existing id replaces that module.
        struct Tools2;
        impl Module for Tools2 {
            fn id(&self) -> &str {
                "tools"
            }
            fn methods(&self) -> Vec<Method> {
                vec![]
            }
            fn call(&self, _: &Vortex, m: &str, _: Value) -> Result<Value, ApiError> {
                Err(ApiError::UnknownMethod(m.to_string()))
            }
        }
        let swapped = Vortex::new().with_module(Arc::new(Tools2));
        assert_eq!(swapped.modules(), Vortex::new().modules(), "replaced in place, order kept");
        assert_eq!(swapped.call("tools.calc", json!({ "expression": "1" })), Err(ApiError::UnknownMethod("tools.calc".into())));
    }
}
