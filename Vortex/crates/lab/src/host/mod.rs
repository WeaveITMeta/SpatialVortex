//! # 006 | Vortex Tools (host side)
//!
//! The tools that act on the host machine, behind `vortex_api::tools::Tool`
//! so they plug into the same gated registry as the pure ones.
//!
//! - [`builtin`]: sandboxed filesystem, clock, and a gated shell
//!   (CLI/terminal) tool. The calculator is pure and lives in
//!   `vortex_api::tools::Calc`.
//! - [`mcp`]: a Model Context Protocol client (JSON-RPC 2.0 over stdio,
//!   newline-delimited), so any MCP server's tools become
//!   Vortex tools.

pub mod builtin;
pub mod mcp;
