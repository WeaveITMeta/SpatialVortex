//! Retrieval Augmented Generation (RAG) System
//!
//! Enables continuous learning through automatic document ingestion,
//! embedding, and intelligent retrieval with sacred geometry integration.
//!
//! ## 3-Stage RAG Architecture
//! 1. HNSW Retrieval → topK candidates (sublinear graph navigation)
//! 2. embedvec Dot Rerank → topR reranked (SV8 SIMD-optimized)
//! 3. Autoregressive Decode → coherent text generation

pub mod ingestion;
pub mod vector_store;
pub mod retrieval;
pub mod augmentation;
pub mod training;
pub mod grokipedia_trainer;
pub mod scholar_trainer;
pub mod document_parser;
pub mod rag_engine;
#[cfg(feature = "postgres")]
pub mod postgres_vector_store;

pub use ingestion::{DocumentIngester, Document, DocumentChunk, IngestionConfig};
pub use vector_store::{VectorStore, VectorDatabase, SacredEmbedding};
pub use retrieval::{RAGRetriever, RetrievalConfig, RetrievalResult};
pub use augmentation::{AugmentedGenerator, GenerationConfig};
pub use training::{ContinuousLearner, TrainingConfig, LearningMetrics};
pub use grokipedia_trainer::{GrokipediaTrainer, GrokipediaCategory, TrainingStats};
pub use scholar_trainer::{ScholarTrainer, ScholarCategory, ScholarStats};
pub use rag_engine::{
    Embedder, Retriever, Reranker, Generator,
    SpatialVortexRag, RagConfig, RerankWorker,
    DocumentStore, AlignedF32,
    normalize_l2, normalized_l2,
};
pub use document_parser::{DocumentParser, ParsedDocument, DocumentType, DocumentMetadata};
#[cfg(feature = "postgres")]
pub use postgres_vector_store::{PostgresVectorStore, StoredEmbedding, VectorStoreStats};
