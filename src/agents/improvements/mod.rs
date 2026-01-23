//! AI Model Improvements
//!
//! Based on real user conversation analysis, this module provides:
//! - Context management to prevent repetitive responses
//! - Tool detection to actually USE tools instead of explaining them
//! - Integration with v1.5.0 consciousness for transparency

pub mod context_manager;
pub mod tool_detector;

pub use context_manager::{ContextManager, VerbosityLevel};
pub use tool_detector::{ToolDetector, ToolCapability, ToolRequirement, Priority};
