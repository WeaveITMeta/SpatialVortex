//! Modules: how a capability reaches the API door.
//!
//! A [`Module`] owns a namespace (`safety`, `spatial`, ...) and the methods
//! under it (`safety.moderate`). [`Vortex`] holds a list of modules and knows
//! nothing about what they do: [`Vortex::methods`] asks each one what it
//! offers, and [`Vortex::call`] hands a call to the module whose id is the
//! method's prefix. So the surface is decided at run time, by whoever builds
//! the handle:
//!
//! - [`Vortex::new`] registers the [`standard`] set.
//! - [`Vortex::with_module`] adds a host's own module, or replaces a built-in
//!   by registering the same id.
//! - [`Vortex::without_module`] drops one.
//!
//! Every transport (the Rust calls, JSON-RPC, MCP) is built on `methods` and
//! `call`, so a module registered here is reachable through all of them with
//! no further wiring.

pub mod core;
pub mod safety;
pub mod spatial;
pub mod tools;

use std::sync::Arc;

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

use crate::json::{ApiError, Method};
use crate::vortex::Vortex;

/// One namespace of methods behind the API door.
pub trait Module: Send + Sync {
    /// The namespace. Every method this module offers is named `<id>.<name>`.
    fn id(&self) -> &str;

    /// What it offers: full method names, descriptions and input schemas.
    fn methods(&self) -> Vec<Method>;

    /// Run one method; `method` is the full name. Bad input is an
    /// [`ApiError`], never a panic. A name this module does not offer is
    /// [`ApiError::UnknownMethod`].
    fn call(&self, vortex: &Vortex, method: &str, params: Value) -> Result<Value, ApiError>;
}

/// The built-in modules, in the order they are listed.
pub fn standard() -> Vec<Arc<dyn Module>> {
    vec![Arc::new(core::Core), Arc::new(safety::Safety), Arc::new(tools::Tools), Arc::new(spatial::Spatial)]
}

// ---------------------------------------------------------------------------
// Helpers for module authors
// ---------------------------------------------------------------------------

/// A read-only [`Method`]: it decides or computes, and changes nothing.
pub fn method(name: &str, description: &str, input_schema: Value) -> Method {
    Method { name: name.to_string(), description: description.to_string(), input_schema, read_only: true }
}

/// Deserialize `params` into a method's parameter type.
pub fn parse<T: DeserializeOwned>(method: &str, params: Value) -> Result<T, ApiError> {
    serde_json::from_value(params).map_err(|e| ApiError::BadParams { method: method.to_string(), message: e.to_string() })
}

/// Serialize a method's result.
pub fn to_value<T: Serialize>(method: &str, v: &T) -> Result<Value, ApiError> {
    serde_json::to_value(v).map_err(|e| ApiError::BadParams { method: method.to_string(), message: format!("result does not serialise: {e}") })
}

/// Parameterless methods accept `null` or an object (extra keys ignored).
pub fn no_params_given(method: &str, params: &Value) -> Result<(), ApiError> {
    match params {
        Value::Null | Value::Object(_) => Ok(()),
        other => Err(ApiError::BadParams { method: method.to_string(), message: format!("expected an object or null, got {other}") }),
    }
}

// ---------------------------------------------------------------------------
// Schemas
// ---------------------------------------------------------------------------

/// A JSON Schema object with the given properties and required keys.
pub fn object(properties: Value, required: &[&str]) -> Value {
    json!({ "type": "object", "properties": properties, "required": required })
}

pub(crate) fn effect_schema() -> Value {
    json!({ "type": "string", "enum": ["read_only", "local_write", "external", "irreversible"] })
}

pub(crate) fn tool_spec_schema() -> Value {
    object(
        json!({
            "name": { "type": "string" },
            "description": { "type": "string" },
            "params": {
                "type": "array",
                "items": object(
                    json!({
                        "name": { "type": "string" },
                        "kind": { "type": "string", "enum": ["string", "number", "integer", "boolean"] },
                        "required": { "type": "boolean", "default": true }
                    }),
                    &["name", "kind"],
                )
            },
            "effect": effect_schema()
        }),
        &["name", "description", "effect"],
    )
}

pub(crate) fn approval_schema() -> Value {
    object(
        json!({
            "approver": { "type": "string" },
            "tool": { "type": "string", "description": "Exact tool name the approval covers." },
            "call_fingerprint": { "type": "integer", "minimum": 0, "description": "FNV-1a 64 of the canonical call JSON (constitution::fingerprint)." }
        }),
        &["approver", "tool", "call_fingerprint"],
    )
}

pub(crate) fn approvals_schema() -> Value {
    json!({ "type": "array", "items": approval_schema(), "description": "Human approvals, each scoped to one exact call." })
}

pub(crate) fn point(n: usize) -> Value {
    json!({ "type": "array", "items": { "type": "number" }, "minItems": n, "maxItems": n })
}

pub(crate) fn body_schema() -> Value {
    object(
        json!({ "name": { "type": "string" }, "position": point(3), "velocity": point(3), "acceleration": point(3) }),
        &["name", "position", "velocity"],
    )
}

pub(crate) fn challenge_schema() -> Value {
    object(
        json!({
            "navigation": object(
                json!({
                    "start": point(2), "goal": point(2), "robot_width": { "type": "number" }, "safety_margin": { "type": "number" },
                    "max_route_length": { "type": "number" }, "bounds": point(4),
                    "obstacles": { "type": "array", "items": object(json!({ "name": { "type": "string" }, "rect": point(4) }), &["name", "rect"]) }
                }),
                &["start", "goal", "robot_width", "safety_margin", "max_route_length", "bounds", "obstacles"],
            ),
            "clearance": object(
                json!({ "portal_gaps": { "type": "array", "items": { "type": "number" } }, "robot_width": { "type": "number" }, "margin_each_side": { "type": "number" } }),
                &["portal_gaps", "robot_width", "margin_each_side"],
            ),
            "assembly": object(
                json!({
                    "targets": { "type": "object", "additionalProperties": point(3) },
                    "position_tolerance": { "type": "number" },
                    "rotation_tolerance": { "type": "number" },
                    "sizes": { "type": "object", "additionalProperties": point(3) }
                }),
                &["targets", "position_tolerance"],
            )
        }),
        &["navigation", "clearance", "assembly"],
    )
}

pub(crate) fn query_schema() -> Value {
    json!({
        "type": "object",
        "description": "vortex_kernel::KinematicsQuery, tagged by `kind`.",
        "oneOf": [
            object(json!({ "kind": { "const": "position_at" }, "body": body_schema(), "t": { "type": "number" } }), &["kind", "body", "t"]),
            object(json!({ "kind": { "const": "closest_approach" }, "a": body_schema(), "b": body_schema(), "horizon": { "type": "number" } }), &["kind", "a", "b", "horizon"]),
            object(json!({ "kind": { "const": "time_to_height" }, "body": body_schema(), "y": { "type": "number" } }), &["kind", "body", "y"])
        ]
    })
}

pub(crate) fn grid_schema() -> Value {
    json!({ "type": "array", "items": { "type": "array", "items": { "type": "integer", "minimum": 0, "maximum": 255 } } })
}

pub(crate) fn task_schema() -> Value {
    let text = |k: &str| object(json!({ k: { "type": "string" } }), &[k]);
    json!({
        "type": "object",
        "description": "vortex_kernel::Task, externally tagged: exactly one key naming the variant, e.g. {\"Moderate\": {\"text\": \"...\"}}.",
        "minProperties": 1,
        "maxProperties": 1,
        "properties": {
            "Choice": object(json!({ "context": { "type": "string" }, "question": { "type": "string" }, "choices": { "type": "array", "items": { "type": "string" } } }), &["context", "question", "choices"]),
            "Free": text("prompt"),
            "Code": object(json!({ "prompt": { "type": "string" }, "entry_point": { "type": "string" } }), &["prompt", "entry_point"]),
            "Grid": object(json!({ "train": { "type": "array", "items": { "type": "array", "items": grid_schema(), "minItems": 2, "maxItems": 2 } }, "test_input": grid_schema() }), &["train", "test_input"]),
            "Spatial": challenge_schema(),
            "Kinematics": query_schema(),
            "Moderate": text("text"),
            "ActionRisk": object(json!({ "call": object(json!({ "tool": { "type": "string" }, "args": {} }), &["tool"]) }), &["call"]),
            "UseTools": object(json!({ "instruction": { "type": "string" }, "tools": { "type": "array", "items": tool_spec_schema() } }), &["instruction", "tools"])
        }
    })
}

pub(crate) fn no_params() -> Value {
    object(json!({}), &[])
}
