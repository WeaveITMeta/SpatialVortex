//! Cognition Module - Thinking, Memory, and Self-Improvement
//!
//! Core cognitive architecture:
//! - **ThinkingEngine** - Reasoning loop with sacred geometry
//! - **MemoryStore** - RocksDB-backed persistent memory
//! - **ConstitutionalGuard** - Claude-style ethical constraints
//! - **RAGEngine** - Internet/document learning

pub mod thinking;
pub mod memory;
pub mod constitution;
pub mod rag;
pub mod vortex_runner;

pub use thinking::{ThinkingEngine, Thought, ThoughtChain, ThinkingConfig, ThoughtType};
pub use memory::{MemoryStore, Memory, MemoryType, MemoryQuery};
pub use constitution::{Constitution, Principle, ConstitutionalGuard};
pub use rag::{RAGEngine, RAGConfig, RetrievedContext, Document};
pub use vortex_runner::{VortexRunner, VortexState, Subject, FluxNode, LadderEntry, SourceType};
