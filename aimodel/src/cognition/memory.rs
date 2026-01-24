//! Memory Store - RocksDB-backed Persistent Memory
//!
//! Stores and retrieves memories with:
//! - Vector embeddings for semantic search
//! - Sacred geometry metadata
//! - Confidence scoring
//! - Temporal decay

use crate::data::models::BeamTensor;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;
use std::path::Path;

#[cfg(feature = "storage")]
use rocksdb::{DB, Options, ColumnFamilyDescriptor};

/// Memory types for categorization
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MemoryType {
    /// Short-term working memory
    Working,
    /// Long-term episodic memory (events)
    Episodic,
    /// Semantic memory (facts/knowledge)
    Semantic,
    /// Procedural memory (how to do things)
    Procedural,
    /// Constitutional memory (ethical principles)
    Constitutional,
}

/// A single memory unit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: Uuid,
    pub content: String,
    pub memory_type: MemoryType,
    pub embedding: Vec<f32>,
    pub confidence: f32,
    pub importance: f32,
    pub access_count: u32,
    pub last_accessed: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
    pub beam: Option<BeamTensor>,
    pub flux_position: u8,
    pub is_sacred: bool,
}

impl Memory {
    pub fn new(content: String, memory_type: MemoryType) -> Self {
        Self {
            id: Uuid::new_v4(),
            content,
            memory_type,
            embedding: Vec::new(),
            confidence: 0.5,
            importance: 0.5,
            access_count: 0,
            last_accessed: Utc::now(),
            created_at: Utc::now(),
            metadata: HashMap::new(),
            beam: None,
            flux_position: 1,
            is_sacred: false,
        }
    }

    pub fn with_embedding(mut self, embedding: Vec<f32>) -> Self {
        self.embedding = embedding;
        self
    }

    pub fn with_confidence(mut self, conf: f32) -> Self {
        self.confidence = conf;
        self
    }

    pub fn with_importance(mut self, imp: f32) -> Self {
        self.importance = imp;
        self
    }

    pub fn with_position(mut self, pos: u8) -> Self {
        self.flux_position = pos;
        self.is_sacred = matches!(pos, 3 | 6 | 9);
        self
    }

    /// Calculate relevance score with temporal decay
    pub fn relevance_score(&self, query_embedding: &[f32]) -> f32 {
        // Cosine similarity
        let similarity = if !self.embedding.is_empty() && self.embedding.len() == query_embedding.len() {
            let dot: f32 = self.embedding.iter().zip(query_embedding).map(|(a, b)| a * b).sum();
            let norm_a: f32 = self.embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
            let norm_b: f32 = query_embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm_a > 0.0 && norm_b > 0.0 { dot / (norm_a * norm_b) } else { 0.0 }
        } else {
            0.0
        };

        // Temporal decay (half-life of 7 days)
        let age_days = (Utc::now() - self.last_accessed).num_days() as f32;
        let decay = 0.5_f32.powf(age_days / 7.0);

        // Sacred boost
        let sacred_boost = if self.is_sacred { 1.2 } else { 1.0 };

        // Combined score
        similarity * self.confidence * self.importance * decay * sacred_boost
    }

    /// Mark as accessed
    pub fn touch(&mut self) {
        self.access_count += 1;
        self.last_accessed = Utc::now();
    }
}

/// Query for memory retrieval
#[derive(Debug, Clone)]
pub struct MemoryQuery {
    pub text: Option<String>,
    pub embedding: Option<Vec<f32>>,
    pub memory_type: Option<MemoryType>,
    pub min_confidence: f32,
    pub min_importance: f32,
    pub limit: usize,
    pub sacred_only: bool,
}

impl Default for MemoryQuery {
    fn default() -> Self {
        Self {
            text: None,
            embedding: None,
            memory_type: None,
            min_confidence: 0.0,
            min_importance: 0.0,
            limit: 10,
            sacred_only: false,
        }
    }
}

impl MemoryQuery {
    pub fn new() -> Self { Self::default() }
    
    pub fn with_text(mut self, text: &str) -> Self {
        self.text = Some(text.to_string());
        self
    }
    
    pub fn with_embedding(mut self, emb: Vec<f32>) -> Self {
        self.embedding = Some(emb);
        self
    }
    
    pub fn with_type(mut self, t: MemoryType) -> Self {
        self.memory_type = Some(t);
        self
    }
    
    pub fn with_limit(mut self, n: usize) -> Self {
        self.limit = n;
        self
    }
    
    pub fn sacred_only(mut self) -> Self {
        self.sacred_only = true;
        self
    }
}

/// RocksDB-backed memory store
pub struct MemoryStore {
    /// In-memory index (for non-storage feature)
    memories: HashMap<Uuid, Memory>,
    /// Type index
    type_index: HashMap<MemoryType, Vec<Uuid>>,
    /// RocksDB handle
    #[cfg(feature = "storage")]
    db: Option<DB>,
    /// Store path
    path: Option<String>,
}

impl MemoryStore {
    /// Create in-memory store
    pub fn new() -> Self {
        Self {
            memories: HashMap::new(),
            type_index: HashMap::new(),
            #[cfg(feature = "storage")]
            db: None,
            path: None,
        }
    }

    /// Create persistent store with RocksDB
    #[cfg(feature = "storage")]
    pub fn open<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);

        let cf_names = ["memories", "type_index", "embeddings"];
        let cfs: Vec<ColumnFamilyDescriptor> = cf_names
            .iter()
            .map(|name| ColumnFamilyDescriptor::new(*name, Options::default()))
            .collect();

        let db = DB::open_cf_descriptors(&opts, path.as_ref(), cfs)?;
        
        let mut store = Self {
            memories: HashMap::new(),
            type_index: HashMap::new(),
            db: Some(db),
            path: Some(path.as_ref().to_string_lossy().to_string()),
        };

        // Load existing memories
        store.load_all()?;

        Ok(store)
    }

    #[cfg(not(feature = "storage"))]
    pub fn open<P: AsRef<Path>>(_path: P) -> anyhow::Result<Self> {
        Ok(Self::new())
    }

    /// Store a memory
    pub fn store(&mut self, memory: Memory) -> anyhow::Result<Uuid> {
        let id = memory.id;
        let memory_type = memory.memory_type;

        // Persist to RocksDB if available
        #[cfg(feature = "storage")]
        if let Some(ref db) = self.db {
            let cf = db.cf_handle("memories").unwrap();
            let key = id.as_bytes();
            let value = serde_json::to_vec(&memory)?;
            db.put_cf(cf, key, value)?;
        }

        // Update type index
        self.type_index
            .entry(memory_type)
            .or_insert_with(Vec::new)
            .push(id);

        // Store in memory
        self.memories.insert(id, memory);

        Ok(id)
    }

    /// Retrieve a memory by ID
    pub fn get(&mut self, id: &Uuid) -> Option<&Memory> {
        if let Some(memory) = self.memories.get_mut(id) {
            memory.touch();
            return Some(memory);
        }
        None
    }

    /// Query memories
    pub fn query(&self, query: &MemoryQuery) -> Vec<&Memory> {
        let mut results: Vec<(&Memory, f32)> = self.memories
            .values()
            .filter(|m| {
                // Type filter
                if let Some(ref t) = query.memory_type {
                    if m.memory_type != *t {
                        return false;
                    }
                }
                // Confidence filter
                if m.confidence < query.min_confidence {
                    return false;
                }
                // Importance filter
                if m.importance < query.min_importance {
                    return false;
                }
                // Sacred filter
                if query.sacred_only && !m.is_sacred {
                    return false;
                }
                true
            })
            .map(|m| {
                let score = if let Some(ref emb) = query.embedding {
                    m.relevance_score(emb)
                } else {
                    m.confidence * m.importance
                };
                (m, score)
            })
            .collect();

        // Sort by score descending
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Limit results
        results.into_iter()
            .take(query.limit)
            .map(|(m, _)| m)
            .collect()
    }

    /// Get all memories of a type
    pub fn get_by_type(&self, memory_type: MemoryType) -> Vec<&Memory> {
        self.type_index
            .get(&memory_type)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.memories.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Delete a memory
    pub fn delete(&mut self, id: &Uuid) -> anyhow::Result<bool> {
        if let Some(memory) = self.memories.remove(id) {
            // Remove from type index
            if let Some(ids) = self.type_index.get_mut(&memory.memory_type) {
                ids.retain(|i| i != id);
            }

            // Remove from RocksDB
            #[cfg(feature = "storage")]
            if let Some(ref db) = self.db {
                let cf = db.cf_handle("memories").unwrap();
                db.delete_cf(cf, id.as_bytes())?;
            }

            return Ok(true);
        }
        Ok(false)
    }

    /// Get memory count
    pub fn len(&self) -> usize {
        self.memories.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.memories.is_empty()
    }

    /// Load all memories from RocksDB
    #[cfg(feature = "storage")]
    fn load_all(&mut self) -> anyhow::Result<()> {
        if let Some(ref db) = self.db {
            let cf = db.cf_handle("memories").unwrap();
            let iter = db.iterator_cf(cf, rocksdb::IteratorMode::Start);

            for item in iter {
                let (_, value) = item?;
                let memory: Memory = serde_json::from_slice(&value)?;
                let id = memory.id;
                let memory_type = memory.memory_type;

                self.type_index
                    .entry(memory_type)
                    .or_insert_with(Vec::new)
                    .push(id);

                self.memories.insert(id, memory);
            }
        }
        Ok(())
    }

    #[cfg(not(feature = "storage"))]
    fn load_all(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    /// Flush to disk
    #[cfg(feature = "storage")]
    pub fn flush(&self) -> anyhow::Result<()> {
        if let Some(ref db) = self.db {
            db.flush()?;
        }
        Ok(())
    }

    #[cfg(not(feature = "storage"))]
    pub fn flush(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_store() {
        let mut store = MemoryStore::new();

        let memory = Memory::new("Test memory".to_string(), MemoryType::Semantic)
            .with_confidence(0.9)
            .with_importance(0.8);

        let id = store.store(memory).unwrap();
        
        assert_eq!(store.len(), 1);
        assert!(store.get(&id).is_some());
    }

    #[test]
    fn test_memory_query() {
        let mut store = MemoryStore::new();

        store.store(Memory::new("fact 1".to_string(), MemoryType::Semantic).with_confidence(0.9)).unwrap();
        store.store(Memory::new("fact 2".to_string(), MemoryType::Semantic).with_confidence(0.7)).unwrap();
        store.store(Memory::new("event 1".to_string(), MemoryType::Episodic).with_confidence(0.8)).unwrap();

        let query = MemoryQuery::new()
            .with_type(MemoryType::Semantic)
            .with_limit(10);

        let results = store.query(&query);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_sacred_memory() {
        let mut store = MemoryStore::new();

        store.store(Memory::new("normal".to_string(), MemoryType::Semantic).with_position(1)).unwrap();
        store.store(Memory::new("sacred 3".to_string(), MemoryType::Semantic).with_position(3)).unwrap();
        store.store(Memory::new("sacred 9".to_string(), MemoryType::Semantic).with_position(9)).unwrap();

        let query = MemoryQuery::new().sacred_only();
        let results = store.query(&query);
        
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|m| m.is_sacred));
    }
}
