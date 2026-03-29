//! ARC-AGI-3 Vortex Agent — DEPRECATED
//!
//! This binary has been superseded by `eustress-arc-agent` which uses the
//! full Eustress vortex-core infrastructure (CausalGraph, HypothesisTree,
//! solve loop, Grid2D WorldState, IRA prediction tracking).
//!
//! Build the new agent:
//!   cd eustress && cargo build --release -p eustress-arc-agent
//!
//! Run standalone (Python bridge):
//!   ARC_AGENT_MODE=standalone eustress-arc-agent
//!
//! Run with Iggy streaming:
//!   ARC_AGENT_MODE=iggy eustress-arc-agent --features iggy-streaming
//!
//! The Python bridge (vortex_agent.py) now launches eustress-arc-agent
//! automatically with VORTEX_BINARY pointing to the new binary.

fn main() {
    eprintln!("arc-agent-iggy is deprecated. Use eustress-arc-agent instead.");
    eprintln!("Build: cd eustress && cargo build --release -p eustress-arc-agent");
    eprintln!("The Python bridge (vortex_agent.py) will use it automatically.");
    std::process::exit(1);
}
