//! Storage Module
//!
//! Contains persistence and caching:
//! - Confidence Lake
//! - Spatial Database
//! - Cache

#[cfg(feature = "lake")]
pub mod confidence_lake;
pub mod spatial_database;
// REMOVED: Eustress Lake - will be reimplemented via MCP server
// pub mod eustress_lake;
// pub mod eustress_lake_postgres;
pub mod cache;

// Re-exports
#[cfg(feature = "lake")]
pub use confidence_lake::*;
pub use spatial_database::*;
pub use cache::*;
