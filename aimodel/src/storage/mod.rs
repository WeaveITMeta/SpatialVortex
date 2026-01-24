//! Storage Module
//!
//! Hot-path storage with RocksDB and embedvec integration.

pub mod embeddings;
pub mod rocksdb_store;

pub use embeddings::{
    SacredEmbedding, SacredEmbeddingIndex, EmbeddingsConfig, 
    SearchResult, beam_to_embedding
};
pub use rocksdb_store::{FluxStore, FluxStoreConfig};
