//! The API door: every capability as a named method taking and returning
//! `serde_json::Value`, each described by a JSON Schema.
//!
//! [`Vortex::methods`] lists what the registered [modules](crate::modules)
//! offer and [`Vortex::call`] runs one by name. Nothing here knows which
//! methods exist: the list is whatever the modules say when asked.
//!
//! [`Vortex::call`] never panics: unknown names and malformed params come
//! back as [`ApiError`], and a panic inside a module on pathological input is
//! caught and reported as `BadParams` rather than unwinding into the host.

use std::fmt;
use std::panic::{AssertUnwindSafe, catch_unwind};

use serde::Serialize;
use serde_json::Value;

use crate::vortex::Vortex;

/// One callable method: what a host needs to offer it as a tool.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Method {
    /// `<module>.<name>`.
    pub name: String,
    pub description: String,
    /// JSON Schema (draft 2020-12 subset) for the `params` object.
    pub input_schema: Value,
    /// True when the method has no side effects. Every built-in method is.
    pub read_only: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ApiError {
    UnknownMethod(String),
    BadParams { method: String, message: String },
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::UnknownMethod(m) => write!(f, "unknown method `{m}`"),
            ApiError::BadParams { method, message } => write!(f, "bad params for `{method}`: {message}"),
        }
    }
}

impl std::error::Error for ApiError {}

/// Which encoding [`text_for`] chose.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TextFormat {
    Json,
    Toon,
}

/// A result as the text a model reads, in whichever encoding is shorter.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Text {
    pub format: TextFormat,
    pub text: String,
}

/// The shorter of compact JSON and TOON for `value`, by bytes; a tie goes to
/// JSON. TOON wins on tables (arrays of same-shaped objects) and loses on
/// deeply nested or irregular data, where its indentation costs more than
/// JSON's punctuation. Measured on this API's own results it ranges from 69%
/// to 122% of the compact-JSON token count, so neither encoding is always
/// right; bytes are the proxy because no tokenizer belongs in this crate.
pub fn text_for(value: &Value) -> Text {
    let json = value.to_string();
    let toon = crate::toon::encode(value);
    if toon.len() < json.len() { Text { format: TextFormat::Toon, text: toon } } else { Text { format: TextFormat::Json, text: json } }
}

impl Vortex {
    /// Every method the registered modules offer, in registration order.
    pub fn methods(&self) -> Vec<Method> {
        self.module_list().iter().flat_map(|m| m.methods()).collect()
    }

    /// Call one method by name. Never panics.
    pub fn call(&self, method: &str, params: Value) -> Result<Value, ApiError> {
        match catch_unwind(AssertUnwindSafe(|| self.dispatch(method, params))) {
            Ok(r) => r,
            Err(panic) => {
                let what = panic.downcast_ref::<&str>().map(|s| s.to_string()).or_else(|| panic.downcast_ref::<String>().cloned()).unwrap_or_else(|| "unknown panic".into());
                Err(ApiError::BadParams { method: method.to_string(), message: format!("internal error on this input: {what}") })
            }
        }
    }

    /// [`Vortex::call`] with the result as the text a model reads: the
    /// shorter of compact JSON and TOON ([`text_for`]). A host keeps the
    /// JSON from `call` as its machine payload and gives this to the model.
    pub fn call_text(&self, method: &str, params: Value) -> Result<Text, ApiError> {
        self.call(method, params).map(|v| text_for(&v))
    }

    /// [`Vortex::call`] with the result always encoded as TOON
    /// ([`crate::toon`]). Prefer [`Vortex::call_text`] unless the consumer
    /// wants one fixed format.
    pub fn call_toon(&self, method: &str, params: Value) -> Result<String, ApiError> {
        self.call(method, params).map(|v| crate::toon::encode(&v))
    }

    /// The module is the method's prefix: `safety.moderate` goes to `safety`.
    fn dispatch(&self, method: &str, params: Value) -> Result<Value, ApiError> {
        let id = method.split_once('.').map_or(method, |(id, _)| id);
        match self.module(id) {
            Some(module) => module.call(self, method, params),
            None => Err(ApiError::UnknownMethod(method.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use serde_json::json;

    use super::*;
    use crate::modules::Module;

    #[test]
    fn text_for_picks_toon_for_tables_and_json_for_nesting() {
        let table = Value::Array((0..20).map(|i| json!({ "id": i, "name": format!("item{i}"), "ok": i % 2 == 0 })).collect());
        let t = text_for(&table);
        assert_eq!(t.format, TextFormat::Toon);
        assert!(t.text.starts_with("[20]{"), "{}", t.text);
        assert!(t.text.len() < table.to_string().len());

        let nested = json!({ "a": { "b": { "c": { "d": [[1, 2], [3, 4]], "e": { "f": [0.5, 1.5] } } } } });
        let n = text_for(&nested);
        assert_eq!(n.format, TextFormat::Json);
        assert_eq!(n.text, nested.to_string());

        // Never longer than either encoding, and always one of the two.
        for v in [table, nested, json!({}), json!([]), json!("x"), json!(null)] {
            let t = text_for(&v);
            let (j, o) = (v.to_string(), crate::toon::encode(&v));
            assert!(t.text.len() <= j.len() && t.text.len() <= o.len());
            assert!(t.text == j || t.text == o);
        }
    }

    #[test]
    fn every_method_is_namespaced_unique_read_only_and_has_an_object_schema() {
        let v = Vortex::new();
        let ms = v.methods();
        assert_eq!(v.modules(), ["core", "safety", "tools", "spatial"]);
        assert_eq!(ms.len(), 11);
        for m in &ms {
            let module = m.name.split_once('.').map(|(id, _)| id).unwrap_or_default();
            assert!(v.modules().iter().any(|id| id == module), "`{}` names no registered module", m.name);
            assert!(m.read_only, "{}", m.name);
            assert_eq!(m.input_schema["type"], "object", "{}", m.name);
            assert!(m.input_schema["required"].is_array(), "{}", m.name);
            for r in m.input_schema["required"].as_array().unwrap() {
                assert!(m.input_schema["properties"].get(r.as_str().unwrap()).is_some(), "{}: required `{r}` is not a property", m.name);
            }
        }
        let mut names: Vec<&str> = ms.iter().map(|m| m.name.as_str()).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), ms.len(), "method names are unique");
    }

    #[test]
    fn gate_effect_is_an_effect_name_or_omitted() {
        let v = Vortex::new();
        for e in ["read_only", "external", "Irreversible", " local_write "] {
            assert!(v.call("safety.gate", json!({ "tool": "calc", "args": { "expression": "1" }, "effect": e })).is_ok(), "{e}");
        }
        assert_eq!(v.call("safety.gate", json!({ "tool": "calc", "args": { "expression": "1" } })).unwrap()["decision"], "run");
        assert_eq!(v.call("safety.gate", json!({ "tool": "git.push", "effect": "external" })).unwrap()["decision"], "ask_human");
        // No host's private vocabulary is understood here.
        for bad in ["sideways", "network", "write", ""] {
            assert!(matches!(v.call("safety.gate", json!({ "tool": "x", "effect": bad })), Err(ApiError::BadParams { .. })), "{bad}");
        }
    }

    #[test]
    fn never_panics_on_junk() {
        let v = Vortex::new();
        let junk = [json!(null), json!(1), json!("s"), json!([]), json!({}), json!({ "task": 5 }), json!({ "text": [] }), json!({ "tools": "x" })];
        for m in v.methods() {
            for p in &junk {
                let _ = v.call(&m.name, p.clone());
            }
        }
        for unknown in ["nope", "", ".", "core.", "core.nope", "nope.info"] {
            assert_eq!(v.call(unknown, json!({})), Err(ApiError::UnknownMethod(unknown.into())), "{unknown:?}");
        }
        // A ragged grid and a degenerate spatial challenge must not unwind.
        let _ = v.call("core.respond", json!({ "task": { "Grid": { "train": [[[[1, 2], [3]], [[1], []]]], "test_input": [[], [1, 2, 3]] } } }));
        let _ = v.call(
            "spatial.solve",
            json!({ "challenge": {
                "navigation": { "start": [0, 0], "goal": [0, 0], "robot_width": -1, "safety_margin": 0, "max_route_length": 0, "bounds": [0, 0, 0, 0], "obstacles": [] },
                "clearance": { "portal_gaps": [], "robot_width": 0, "margin_each_side": 0 },
                "assembly": { "targets": {}, "position_tolerance": 0 }
            } }),
        );
    }

    #[test]
    fn a_panicking_module_is_contained() {
        struct Boom;
        impl Module for Boom {
            fn id(&self) -> &str {
                "boom"
            }
            fn methods(&self) -> Vec<Method> {
                vec![crate::modules::method("boom.go", "Panics.", crate::modules::object(json!({}), &[]))]
            }
            fn call(&self, _: &Vortex, _: &str, _: Value) -> Result<Value, ApiError> {
                panic!("module bug")
            }
        }
        let v = Vortex::new().with_module(Arc::new(Boom));
        match v.call("boom.go", json!({})) {
            Err(ApiError::BadParams { method, message }) => assert!(method == "boom.go" && message.contains("module bug"), "{message}"),
            other => panic!("expected a contained error, got {other:?}"),
        }
        // The handle still works afterwards.
        assert_eq!(v.call("tools.calc", json!({ "expression": "2+2" })).unwrap()["text"], "4");
    }
}
