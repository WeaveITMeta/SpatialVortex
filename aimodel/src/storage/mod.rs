//! Storage Module
//!
//! Hot-path storage with RocksDB and embedvec integration.
//! Unified store provides single-database architecture for all model data.

pub mod embeddings;
pub mod rocksdb_store;
pub mod unified_store;
pub mod trait_ledger;

pub use embeddings::{
    SacredEmbedding, SacredEmbeddingIndex, EmbeddingsConfig, 
    SearchResult, beam_to_embedding
};
pub use rocksdb_store::{FluxStore, FluxStoreConfig};
pub use unified_store::{
    UnifiedStore, UnifiedStoreConfig, ModelTier, QuantizationLevel,
    StoredWeight, StoredEmbedding, StoredLatent, StoredPattern, StoreStats,
};
pub use trait_ledger::{
    TraitLedger, TraitValue, TraitDelta, TraitRevision,
    ProvenanceRecord, ProvenanceSource, DiffResult,
    RollbackPolicy, WriteResult, LedgerStats,
};
