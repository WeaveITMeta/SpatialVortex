//! Model Context Protocol client: JSON-RPC 2.0 over stdio, one JSON object
//! per line (the MCP stdio transport). Speaks `initialize`,
//! `notifications/initialized`, `tools/list` (with pagination) and
//! `tools/call`. Server-to-client notifications are skipped while waiting for
//! a response.
//!
//! An MCP tool's annotations decide its [`Effect`]: `readOnlyHint` makes it
//! read-only, `destructiveHint` irreversible, `openWorldHint` external, and a
//! tool that declares nothing is treated as a local write, never as safe.

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

use serde_json::{Value, json};
use vortex_api::tools::{Tool, ToolError};
use vortex_kernel::{Effect, ParamKind, ParamSpec, ToolSpec};

pub const PROTOCOL_VERSION: &str = "2025-06-18";

#[derive(Debug)]
pub enum McpError {
    Io(std::io::Error),
    Protocol(String),
    Remote { code: i64, message: String },
}

impl std::fmt::Display for McpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            McpError::Io(e) => write!(f, "io: {e}"),
            McpError::Protocol(s) => write!(f, "protocol: {s}"),
            McpError::Remote { code, message } => write!(f, "server error {code}: {message}"),
        }
    }
}

impl From<std::io::Error> for McpError {
    fn from(e: std::io::Error) -> Self {
        McpError::Io(e)
    }
}

#[derive(Debug, Clone)]
pub struct McpToolInfo {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub annotations: Value,
}

impl McpToolInfo {
    pub fn to_spec(&self, prefix: &str) -> ToolSpec {
        let required: Vec<&str> = self.input_schema["required"]
            .as_array()
            .map(|a| a.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default();
        let mut params: Vec<ParamSpec> = self.input_schema["properties"]
            .as_object()
            .map(|props| {
                props
                    .iter()
                    .map(|(name, schema)| ParamSpec {
                        name: name.clone(),
                        kind: match schema["type"].as_str() {
                            Some("number") => ParamKind::Number,
                            Some("integer") => ParamKind::Integer,
                            Some("boolean") => ParamKind::Boolean,
                            _ => ParamKind::String,
                        },
                        required: required.contains(&name.as_str()),
                    })
                    .collect()
            })
            .unwrap_or_default();
        params.sort_by(|a, b| b.required.cmp(&a.required).then(a.name.cmp(&b.name)));
        let a = &self.annotations;
        let effect = if a["destructiveHint"].as_bool() == Some(true) {
            Effect::Irreversible
        } else if a["readOnlyHint"].as_bool() == Some(true) {
            Effect::ReadOnly
        } else if a["openWorldHint"].as_bool() == Some(true) {
            Effect::External
        } else {
            Effect::LocalWrite
        };
        ToolSpec { name: format!("{prefix}{}", self.name), description: self.description.clone(), params, effect }
    }
}

pub struct McpClient {
    reader: Box<dyn BufRead + Send>,
    writer: Box<dyn Write + Send>,
    next_id: u64,
    child: Option<Child>,
    pub server_info: Value,
}

impl McpClient {
    /// Launch an MCP server as a child process and complete the handshake.
    pub fn spawn(program: &str, args: &[&str]) -> Result<Self, McpError> {
        let mut child = Command::new(program)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let stdin = child.stdin.take().ok_or_else(|| McpError::Protocol("no stdin".into()))?;
        let stdout = child.stdout.take().ok_or_else(|| McpError::Protocol("no stdout".into()))?;
        let mut c = Self::from_streams(BufReader::new(stdout), stdin);
        c.child = Some(child);
        c.initialize()?;
        Ok(c)
    }

    /// Wrap existing streams (no handshake; call [`McpClient::initialize`]).
    pub fn from_streams(reader: impl BufRead + Send + 'static, writer: impl Write + Send + 'static) -> Self {
        Self { reader: Box::new(reader), writer: Box::new(writer), next_id: 1, child: None, server_info: Value::Null }
    }

    fn send(&mut self, msg: &Value) -> Result<(), McpError> {
        let mut line = serde_json::to_string(msg).map_err(|e| McpError::Protocol(e.to_string()))?;
        line.push('\n');
        self.writer.write_all(line.as_bytes())?;
        self.writer.flush()?;
        Ok(())
    }

    pub fn request(&mut self, method: &str, params: Value) -> Result<Value, McpError> {
        let id = self.next_id;
        self.next_id += 1;
        self.send(&json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }))?;
        let mut line = String::new();
        loop {
            line.clear();
            if self.reader.read_line(&mut line)? == 0 {
                return Err(McpError::Protocol(format!("server closed the stream before answering `{method}`")));
            }
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let msg: Value = serde_json::from_str(trimmed).map_err(|e| McpError::Protocol(format!("bad JSON: {e}")))?;
            // Notifications and server-initiated requests carry no matching id.
            if msg.get("id").and_then(Value::as_u64) != Some(id) {
                continue;
            }
            if let Some(err) = msg.get("error") {
                return Err(McpError::Remote {
                    code: err["code"].as_i64().unwrap_or(0),
                    message: err["message"].as_str().unwrap_or("").to_string(),
                });
            }
            return Ok(msg.get("result").cloned().unwrap_or(Value::Null));
        }
    }

    pub fn notify(&mut self, method: &str, params: Value) -> Result<(), McpError> {
        self.send(&json!({ "jsonrpc": "2.0", "method": method, "params": params }))
    }

    pub fn initialize(&mut self) -> Result<Value, McpError> {
        let result = self.request(
            "initialize",
            json!({
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": {},
                "clientInfo": { "name": "vortex", "version": env!("CARGO_PKG_VERSION") }
            }),
        )?;
        self.server_info = result.clone();
        self.notify("notifications/initialized", json!({}))?;
        Ok(result)
    }

    pub fn list_tools(&mut self) -> Result<Vec<McpToolInfo>, McpError> {
        let mut out = Vec::new();
        let mut cursor: Option<String> = None;
        loop {
            let params = match &cursor {
                Some(c) => json!({ "cursor": c }),
                None => json!({}),
            };
            let r = self.request("tools/list", params)?;
            for t in r["tools"].as_array().cloned().unwrap_or_default() {
                out.push(McpToolInfo {
                    name: t["name"].as_str().unwrap_or("").to_string(),
                    description: t["description"].as_str().unwrap_or("").to_string(),
                    input_schema: t["inputSchema"].clone(),
                    annotations: t["annotations"].clone(),
                });
            }
            match r["nextCursor"].as_str() {
                Some(c) if !c.is_empty() => cursor = Some(c.to_string()),
                _ => break,
            }
        }
        Ok(out)
    }

    pub fn call_tool(&mut self, name: &str, arguments: Value) -> Result<Value, McpError> {
        let r = self.request("tools/call", json!({ "name": name, "arguments": arguments }))?;
        if r["isError"].as_bool() == Some(true) {
            let text = r["content"][0]["text"].as_str().unwrap_or("tool reported an error").to_string();
            return Err(McpError::Remote { code: 0, message: text });
        }
        Ok(r)
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        if let Some(child) = &mut self.child {
            // Our own child, by handle: never a by-name kill.
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// One MCP server tool exposed as a Vortex [`Tool`].
pub struct McpTool {
    client: Arc<Mutex<McpClient>>,
    info: McpToolInfo,
    spec: ToolSpec,
}

impl McpTool {
    /// Every tool a server offers, names prefixed (e.g. `"eustress."`).
    pub fn all(client: Arc<Mutex<McpClient>>, prefix: &str) -> Result<Vec<McpTool>, McpError> {
        let infos = client.lock().map_err(|_| McpError::Protocol("client lock poisoned".into()))?.list_tools()?;
        Ok(infos
            .into_iter()
            .map(|info| {
                let spec = info.to_spec(prefix);
                McpTool { client: client.clone(), info, spec }
            })
            .collect())
    }
}

impl Tool for McpTool {
    fn spec(&self) -> ToolSpec {
        self.spec.clone()
    }
    fn call(&self, args: &Value) -> Result<Value, ToolError> {
        let mut c = self.client.lock().map_err(|_| ToolError::Failed("client lock poisoned".into()))?;
        c.call_tool(&self.info.name, args.clone()).map_err(|e| ToolError::Failed(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal MCP server on the other end of two OS pipes.
    fn fake_server() -> McpClient {
        let (c2s_r, c2s_w) = std::io::pipe().unwrap();
        let (s2c_r, mut s2c_w) = std::io::pipe().unwrap();
        std::thread::spawn(move || {
            let reader = BufReader::new(c2s_r);
            for line in reader.lines() {
                let Ok(line) = line else { break };
                let msg: Value = serde_json::from_str(&line).unwrap();
                let Some(id) = msg.get("id").cloned() else { continue }; // notification
                // Interleave a server notification to prove the client skips it.
                writeln!(s2c_w, r#"{{"jsonrpc":"2.0","method":"notifications/message","params":{{}}}}"#).unwrap();
                let result = match msg["method"].as_str().unwrap() {
                    "initialize" => json!({"protocolVersion": PROTOCOL_VERSION, "serverInfo": {"name": "fake"}, "capabilities": {"tools": {}}}),
                    "tools/list" if msg["params"].get("cursor").is_none() => json!({
                        "tools": [{"name": "list_spaces", "description": "List Spaces", "inputSchema": {"type": "object", "properties": {"universe": {"type": "string"}}, "required": ["universe"]}, "annotations": {"readOnlyHint": true}}],
                        "nextCursor": "p2"
                    }),
                    "tools/list" => json!({
                        "tools": [{"name": "delete_entity", "description": "Delete", "inputSchema": {"type": "object", "properties": {"id": {"type": "integer"}}}, "annotations": {"destructiveHint": true}}]
                    }),
                    "tools/call" => json!({"content": [{"type": "text", "text": format!("called {}", msg["params"]["name"])}], "isError": false}),
                    _ => json!(null),
                };
                writeln!(s2c_w, "{}", json!({"jsonrpc": "2.0", "id": id, "result": result})).unwrap();
            }
        });
        let mut c = McpClient::from_streams(BufReader::new(s2c_r), c2s_w);
        c.initialize().unwrap();
        c
    }

    #[test]
    fn handshake_pagination_effects_and_calls() {
        let mut c = fake_server();
        assert_eq!(c.server_info["serverInfo"]["name"], "fake");
        let tools = c.list_tools().unwrap();
        assert_eq!(tools.len(), 2, "pagination must follow nextCursor");
        let specs: Vec<_> = tools.iter().map(|t| t.to_spec("eustress.")).collect();
        assert_eq!(specs[0].name, "eustress.list_spaces");
        assert_eq!(specs[0].effect, Effect::ReadOnly);
        assert!(specs[0].params[0].required);
        assert_eq!(specs[1].effect, Effect::Irreversible);
        let r = c.call_tool("list_spaces", json!({"universe": "Astra"})).unwrap();
        assert_eq!(r["content"][0]["text"], "called \"list_spaces\"");
    }
}
