//! # Vortex API
//!
//! The whole in-process Vortex surface in one crate that any host can embed,
//! with nothing but `serde`, `serde_json` and `sha2` underneath. It knows no
//! host: nothing here names an engine, an editor or a product.
//!
//! ## Three properties
//!
//! - **Universal.** One surface, four ways to reach it, all built on the
//!   same two functions ([`Vortex::methods`], [`Vortex::call`]): typed Rust
//!   calls, calls by name with JSON, JSON-RPC 2.0 messages, and the MCP tool
//!   dialect ([`rpc`]). A host picks the one that fits.
//! - **Dynamic.** What the handle offers is decided when it is built, not
//!   when this crate is compiled. Methods are discovered at run time with
//!   their schemas; language models ([`lm::LanguageModel`]), tools
//!   ([`tools::Tool`]) and modules ([`Module`]) are trait objects a host
//!   supplies.
//! - **Modular.** Every capability is a [`Module`] under its own namespace.
//!   A host adds its own, replaces a built-in, or drops one
//!   ([`Vortex::with_module`], [`Vortex::without_module`]).
//!
//! ## Purity invariant
//!
//! Nothing under `src/` reads or writes files, spawns processes, opens
//! sockets or reads environment variables. Monotonic timing
//! (`std::time::Instant`) and scoped threads (`std::thread::scope`, used for
//! parallel read-only tool calls) are the only host facilities touched.
//! Anything with side effects (corpora on disk, ledgers, shell and
//! filesystem tools, the MCP client, the stdio transport) lives in
//! `vortex-lab` and plugs in through the traits above. `tests/purity.rs`
//! enforces this by scanning the source.
//!
//! ## Contents (subsystem numbers from `SUBSYSTEMS.md`)
//!
//! | Module           | What                                              |
//! |------------------|---------------------------------------------------|
//! | [`vortex`]       | 001 Orchestrator: the [`Vortex`] handle           |
//! | [`lm`]           | 002 LLM                                           |
//! | [`constitution`] | 003 Constitution                                  |
//! | [`provenance`]   | 003 A7: provenance manifest for training data     |
//! | [`safety`]       | 004 Safety                                        |
//! | [`pipeline`]     | 005 Pipeline: lifecycle record, gate, tick        |
//! | [`tools`]        | 006 Tools: registry, router, calc                 |
//! | [`spatial`]      | 008 Spatial                                       |
//! | `world`          | 010 World Model (feature `world`)                 |
//! | [`rsi`]          | 012 RSI improvement gate                          |
//! | [`modules`]      | The API door's modules and the [`Module`] trait   |
//! | [`json`]         | The API door: [`Vortex::methods`], [`Vortex::call`] |
//! | [`rpc`]          | The door over JSON-RPC 2.0 and MCP                |
//! | [`toon`]         | JSON to TOON encoder (spec 4.4)                   |
//!
//! ## Features
//!
//! | Feature | Default | Adds                                                        |
//! |---------|---------|-------------------------------------------------------------|
//! | `world` | off     | `world` module and ARC grid solving via `eustress-vortex-core` and `eustress-vortex-grid2d`. Without it, `Task::Grid` abstains and says why. |
//!
//! ## The ways in
//!
//! - Typed: [`Vortex::respond`], [`Vortex::moderate`], [`Vortex::assess`],
//!   [`Vortex::route`], [`Vortex::gate`], [`Vortex::plan`], [`Vortex::tick`].
//! - By name: [`Vortex::methods`] lists every method with a JSON Schema;
//!   [`Vortex::call`] runs one and never panics. Every built-in method is
//!   read-only: it decides, it never executes.
//! - Text for a model: [`Vortex::call_text`] is `call` with the result as
//!   the shorter of compact JSON and TOON ([`toon`]). TOON is the smaller
//!   one for tables and the larger one for nested data, so the choice is
//!   made per result; [`Vortex::call_toon`] forces TOON.
//! - As messages: [`Vortex::handle`] takes one JSON-RPC 2.0 message and
//!   returns the reply, for direct calls and for MCP `tools/list` and
//!   `tools/call`. No transport is included; the host moves the bytes.

pub mod constitution;
pub mod json;
pub mod lm;
pub mod modules;
pub mod pipeline;
pub mod provenance;
pub mod rpc;
pub mod rsi;
pub mod safety;
pub mod spatial;
pub mod tools;
pub mod toon;
pub mod vortex;
#[cfg(feature = "world")]
pub mod world;

pub use vortex_kernel as kernel;
pub use vortex_kernel::{
    ActionClass, Agent, Answer, Body, Effect, Grid, GridTask, GuardianCategory, KinematicsQuery, Mechanism, ParamKind, ParamSpec,
    Response, Rng, SpatialChallenge, SpatialSubmission, Task, ToolCall, ToolSpec, Verdict,
};

pub use json::{ApiError, Method, Text, TextFormat, text_for};
pub use modules::Module;
pub use pipeline::{Gate, GateVerdict};
pub use vortex::{Plan, PlannedCall, Vortex};
