//! # 000 | Vortex Kernel
//!
//! The shared vocabulary of the Vortex workspace. Every other subsystem
//! depends on this crate and on nothing else in common, so the types here are
//! the contract between them:
//!
//! - [`Task`] / [`Answer`] / [`Response`]: what a problem looks like and what
//!   a solver returns. The harness hands an agent a `Task` and **nothing
//!   else**; gold labels never cross this boundary.
//! - [`ToolSpec`] / [`ToolCall`] / [`Effect`]: tool calling, shared by the
//!   tool registry, the safety classifier and the MCP client.
//! - [`GuardianCategory`] / [`Verdict`]: the Eustress AI Guardian Policy v1.2
//!   taxonomy, so Vortex's safety lane and the engine's publish gate agree.
//! - [`spatial`]: the Astra Spatial Reasoning Lab challenge format.
//! - [`Rng`]: one deterministic PRNG so every seeded run is reproducible.

pub mod guardian;
pub mod physics;
pub mod rng;
pub mod spatial;
pub mod task;
pub mod tool;

pub use guardian::{GuardianCategory, Verdict};
pub use physics::{KinematicsQuery, Body};
pub use rng::Rng;
pub use spatial::{SpatialChallenge, SpatialSubmission};
pub use task::{Answer, Grid, GridTask, Mechanism, Response, Task};
pub use tool::{ActionClass, Effect, ParamKind, ParamSpec, ToolCall, ToolSpec};

/// FNV-1a 64-bit. Stable across platforms and Rust versions (unlike
/// `DefaultHasher`), so it is safe for seeds, splits and cache keys. Not a
/// cryptographic hash; use SHA-256 for anything adversarial.
pub fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fnv_is_stable() {
        // Empty input hashes to the FNV offset basis.
        assert_eq!(fnv1a64(b""), 0xcbf2_9ce4_8422_2325);
        assert_ne!(fnv1a64(b"vortex"), fnv1a64(b"vorteX"));
    }
}
