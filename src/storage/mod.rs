//! Storage Module
//!
//! Contains persistence and caching:
//! - Confidence Lake
//! - Spatial Database
//! - Cache

#[cfg(feature = "lake")]
pub mod confidence_lake;
pub mod cache;
pub mod spatial_database;
// REMOVED: Eustress Lake - will be reimplemented via MCP server
// pub mod eustress_lake;
// pub mod eustress_lake_postgres;
// REMOVED: pub mod cache;  // Redis superseded by RocksDB

// Re-exports
#[cfg(feature = "lake")]
pub use confidence_lake::*;
pub use cache::*;
pub use spatial_database::*;
// REMOVED: pub use cache::*;  // Redis superseded by RocksDB
