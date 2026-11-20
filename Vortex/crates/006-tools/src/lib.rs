//! # 006 | Vortex Tools
//!
//! How Vortex acts on the world.
//!
//! - [`registry`]: tools behind one interface. Every call passes a caller-
//!   supplied gate (the pipeline wires safety 004 + constitution 003 into it)
//!   before it runs. Read-only calls run **in parallel**; anything that writes
//!   runs sequentially in the order given.
//! - [`builtin`]: calculator, sandboxed filesystem, clock, and a gated
//!   shell (CLI/terminal) tool.
//! - [`mcp`]: a Model Context Protocol client (JSON-RPC 2.0 over stdio,
//!   newline-delimited), so any MCP server's tools, the Eustress MCP server
//!   included, become Vortex tools.
//! - [`router`]: turns an English instruction plus a tool list into concrete
//!   calls, splitting independent requests so they can run in parallel.
//! - [`calc`]: the exact arithmetic evaluator behind the calculator.

pub mod builtin;
pub mod calc;
pub mod mcp;
pub mod registry;
pub mod router;

pub use registry::{GateDecision, Outcome, Registry, Tool, ToolError};
pub use router::route;
