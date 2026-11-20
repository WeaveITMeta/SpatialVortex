//! # Vortex Lab
//!
//! Everything Vortex does that touches disk, processes or the network,
//! kept out of the pure, embeddable `vortex-api`:
//!
//! | Module          | Subsystem                                                       |
//! |-----------------|-----------------------------------------------------------------|
//! | [`lifecycle`]   | 001 Orchestrator: the seven-stage lifecycle end to end, ledgers |
//! | [`corpus`]      | 002 LLM: provenance-verified training corpora read from disk    |
//! | [`host`]        | 006 Tools: clock, sandboxed filesystem, shell, MCP stdio client |
//! | [`ledger`]      | 012 RSI: the append-only RSI ledger                             |
//! | [`economics`]   | 015 Economics                                                   |
//! | [`astra`]       | 008 Spatial: a Spatial Reasoning Lab on disk, graded by its own evaluator |
//!
//! The `vortex` CLI (`src/bin/vortex.rs`) is built on this crate, including
//! `vortex serve`: the API over JSON-RPC 2.0 and MCP on stdio.

pub mod astra;
pub mod corpus;
pub mod economics;
pub mod host;
pub mod ledger;
pub mod lifecycle;
