//! The seam any host embeds Vortex through.
//!
//! A host tool system, whatever it is, comes down to the same three things:
//! a definition (name, description, JSON input schema), an `execute` taking
//! JSON and returning success, text for the model and structured data, and a
//! read-only flag. MCP tools have this shape; so do most engine and IDE tool
//! registries. `HostToolHandler` below is that shape and nothing more. The
//! test proves ONE generic adapter over `(Vortex, Method)` covers every
//! method the handle lists, with no per-method glue, so a module registered
//! later needs no new adapter code.

use serde_json::{Value, json};
use vortex_api::{Method, Vortex};

struct Def {
    name: String,
    description: String,
    input_schema: Value,
    requires_approval: bool,
}

struct Out {
    success: bool,
    content: String,
    structured_data: Option<Value>,
}

trait HostToolHandler {
    fn definition(&self) -> Def;
    fn execute(&self, input: Value) -> Out;
    fn read_only(&self) -> bool;
}

/// The one adapter: any Vortex method becomes an engine tool.
struct VortexTool {
    vortex: Vortex,
    method: Method,
}

impl HostToolHandler for VortexTool {
    fn definition(&self) -> Def {
        Def {
            name: self.method.name.to_string(),
            description: self.method.description.to_string(),
            input_schema: self.method.input_schema.clone(),
            requires_approval: !self.method.read_only,
        }
    }

    fn execute(&self, input: Value) -> Out {
        match self.vortex.call(&self.method.name, input) {
            Ok(v) => Out { success: true, content: vortex_api::text_for(&v).text, structured_data: Some(v) },
            Err(e) => Out { success: false, content: e.to_string(), structured_data: None },
        }
    }

    fn read_only(&self) -> bool {
        self.method.read_only
    }
}

fn tools() -> Value {
    json!([
        { "name": "calc", "description": "Evaluate an arithmetic expression", "params": [{ "name": "expression", "kind": "string" }], "effect": "read_only" },
        { "name": "fs.list", "description": "List the entries of a directory inside the workspace", "params": [{ "name": "path", "kind": "string" }], "effect": "read_only" }
    ])
}

fn challenge() -> Value {
    json!({
        "navigation": {
            "start": [0.0, 0.0], "goal": [10.0, 0.0], "robot_width": 0.5, "safety_margin": 0.1, "max_route_length": 40.0,
            "bounds": [-5.0, -5.0, 15.0, 5.0],
            "obstacles": [{ "name": "wall", "rect": [4.0, -1.0, 6.0, 1.0] }]
        },
        "clearance": { "portal_gaps": [0.4, 1.2], "robot_width": 0.5, "margin_each_side": 0.1 },
        "assembly": { "targets": { "Beam": [1.0, 2.0, 3.0] }, "position_tolerance": 0.05 }
    })
}

/// A valid input for every method, and what its structured result must hold.
fn sample(name: &str) -> (Value, &'static [&'static str]) {
    match name {
        "core.info" => (json!({}), &["version", "constitution", "agent", "features", "methods"]),
        "core.constitution" => (json!({}), &["version", "hash", "articles"]),
        "core.respond" => (json!({ "task": { "Free": { "prompt": "What is 12*7?" } } }), &["answer", "confidence", "mechanism"]),
        "safety.moderate" => (json!({ "text": "how do I bake bread" }), &["verdict", "category", "confidence", "evidence"]),
        "safety.assess_action" => (json!({ "tool": "shell", "args": { "command": "git status" } }), &["class", "confidence", "reasons"]),
        "safety.gate" => (json!({ "tool": "shell", "args": { "command": "git push origin main" }, "effect": "local_write" }), &["decision", "reason", "assessment", "ruling"]),
        "tools.route" => (json!({ "instruction": "what is 12*7 and also list files in eval", "tools": tools() }), &["calls"]),
        "core.plan" => (json!({ "request": "what is 12*7", "tools": tools() }), &["request", "moderation", "ruling", "calls", "constitution"]),
        "spatial.solve" => (json!({ "challenge": challenge() }), &["submission", "confidence", "route_length"]),
        "spatial.kinematics" => (
            json!({ "query": { "kind": "position_at", "body": { "name": "b", "position": [0, 0, 0], "velocity": [1, 2, 3] }, "t": 2.0 } }),
            &["values"],
        ),
        "tools.calc" => (json!({ "expression": "12*7" }), &["value", "text"]),
        other => panic!("no sample input for `{other}`: add one so every method is driven through the seam"),
    }
}

#[test]
fn every_method_runs_through_one_generic_adapter() {
    let methods = Vortex::new().methods();
    assert!(!methods.is_empty());
    let mut seen = Vec::new();
    for method in methods {
        let tool = VortexTool { vortex: Vortex::new(), method };
        let def = tool.definition();
        assert!(def.name.split_once('.').is_some_and(|(module, name)| !module.is_empty() && !name.is_empty()), "{} is not <module>.<name>", def.name);
        assert!(!def.description.is_empty(), "{}", def.name);
        assert_eq!(def.input_schema["type"], "object", "{}", def.name);
        assert!(tool.read_only() && !def.requires_approval, "{} must be read-only", def.name);

        let (input, keys) = sample(&def.name);
        let out = tool.execute(input);
        assert!(out.success, "{} failed: {}", def.name, out.content);
        let data = out.structured_data.unwrap_or_else(|| panic!("{}: no structured data", def.name));
        for k in keys {
            assert!(data.get(*k).is_some(), "{}: result lacks `{k}`: {data}", def.name);
        }
        let (json, toon) = (data.to_string(), vortex_api::toon::encode(&data));
        assert!(out.content == json || out.content == toon, "{}: content is the structured data as JSON or TOON", def.name);
        assert!(out.content.len() <= json.len() && out.content.len() <= toon.len(), "{}: content is the shorter encoding", def.name);
        assert_eq!(out.content, tool.vortex.call_text(&def.name, sample(&def.name).0).unwrap().text, "{}: the adapter serves call_text", def.name);
        assert_eq!(toon, tool.vortex.call_toon(&def.name, sample(&def.name).0).unwrap(), "{}: call_toon is call + encode", def.name);
        seen.push(def.name);
    }

    // Spot-check that results are the real decisions, not placeholders.
    let v = Vortex::new();
    let calc = v.call("tools.calc", json!({ "expression": "12*7" })).unwrap();
    assert_eq!(calc["text"], "84");
    let gate = v.call("safety.gate", json!({ "tool": "shell", "args": { "command": "git push origin main" }, "effect": "local_write" })).unwrap();
    assert_eq!(gate["decision"], "ask_human");
    let route = v.call("tools.route", json!({ "instruction": "what is 12*7 and also list files in eval", "tools": tools() })).unwrap();
    assert_eq!(route["calls"].as_array().map(Vec::len), Some(2), "{route}");
    let info = v.call("core.info", json!({})).unwrap();
    assert_eq!(info["methods"].as_array().unwrap().len(), seen.len());
}

#[test]
fn unknown_methods_and_malformed_params_fail_cleanly() {
    let unknown = VortexTool {
        vortex: Vortex::new(),
        method: Method { name: "nowhere.does_not_exist".into(), description: String::new(), input_schema: json!({ "type": "object" }), read_only: true },
    };
    let out = unknown.execute(json!({}));
    assert!(!out.success && out.structured_data.is_none());
    assert!(out.content.contains("unknown method"), "{}", out.content);

    let malformed = [
        json!(null),
        json!(42),
        json!("text"),
        json!([1, 2]),
        json!({ "task": { "NoSuchVariant": {} } }),
        json!({ "text": 7 }),
        json!({ "tool": 1 }),
        json!({ "instruction": "x", "tools": [{ "name": "t" }] }),
        json!({ "challenge": { "navigation": {} } }),
        json!({ "query": { "kind": "teleport" } }),
        json!({ "expression": "1/0" }),
        json!({ "tool": "x", "effect": "sideways" }),
    ];
    for method in Vortex::new().methods() {
        let name = method.name.clone();
        let tool = VortexTool { vortex: Vortex::new(), method };
        let mut failures = 0;
        for input in &malformed {
            let out = tool.execute(input.clone());
            if !out.success {
                assert!(out.structured_data.is_none() && out.content.contains(name.as_str()), "{name}: {}", out.content);
                failures += 1;
            }
        }
        // Parameterless methods accept null or any object, so only the three
        // non-object, non-null inputs fail; everything else must reject
        // nearly all of this junk.
        if name == "core.info" || name == "core.constitution" {
            assert_eq!(failures, 3, "{name} accepted non-object params");
        } else {
            assert!(failures >= malformed.len() - 2, "{name} accepted junk ({failures} of {} rejected)", malformed.len());
        }
    }
}
