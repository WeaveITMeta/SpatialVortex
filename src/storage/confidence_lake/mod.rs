//! Confidence Lake: Secure storage for high-value patterns
//!
//! Encrypted, persistent storage system for preserving patterns that
//! demonstrate high confidence scores. Uses AES-GCM-SIV encryption
//! and memory-mapped files for efficient I/O.
//!
//! # Architecture
//!
//! ```text
//! BeadTensor → ConfidenceScoring → [High Value?] → SecureStorage → ConfidenceLake
//! ```
//!
//! # Example
//!
//! ```no_run
//! use spatial_vortex::confidence_lake::ConfidenceLake;
//! use std::path::Path;
//!
//! # fn main() -> anyhow::Result<()> {
//! let mut lake = ConfidenceLake::create(Path::new("patterns.lake"), 100)?;
//! 
//! // Store high-value pattern
//! let data = vec![1, 2, 3, 4, 5];
//! lake.store(12345, &data)?;
//! 
//! // Retrieve later
//! let retrieved = lake.retrieve(12345)?;
//! # Ok(())
//! # }
//! ```

pub mod encryption;
pub mod storage;
#[cfg(feature = "lake")]
pub mod postgres_backend;

pub use encryption::SecureStorage;
pub use storage::ConfidenceLake;
#[cfg(feature = "lake")]
pub use postgres_backend::{PostgresConfidenceLake, LakeStats};
// FluxMatrix is imported from models, not here
pub use crate::models::FluxMatrix;
