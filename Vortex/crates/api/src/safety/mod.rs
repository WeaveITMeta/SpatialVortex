//! # 004 | Vortex Safety
//!
//! Two classifiers, both feeding the constitution (003):
//!
//! - [`content`]: Guardian Policy v1.2 moderation of requests and generated
//!   content into `publish` / `reject` / `flag_for_human_review` plus a
//!   category G1-G8. It applies the policy's own test: content *about* harm
//!   is protected; a real-world manual *for* harm is not.
//! - [`action`]: risk class of a proposed tool call (`allow` / `confirm` /
//!   `deny`) from the call itself, independent of what the tool claims.
//!
//! **What this is, honestly:** v1 is a transparent lexical rule system. It
//! is fast (microseconds), auditable (every decision lists the evidence that
//! fired), and measured against sealed held-out sets authored independently
//! (see `eval/heldout/`). It is not a learned model and will miss paraphrase
//! it has no pattern for. The harness reports both over-refusal and
//! under-refusal, because a classifier that rejects everything also "blocks
//! all harm".

pub mod action;
pub mod content;

pub use action::{assess_action, ActionAssessment};
pub use content::{moderate, Moderation};
