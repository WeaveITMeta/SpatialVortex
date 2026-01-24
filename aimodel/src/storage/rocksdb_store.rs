//! RocksDB Persistence for Flux States
//!
//! Hot-path storage for flux states and embeddings.

use crate::data::models::BeamTensor;
use crate::data::attributes::Attributes;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for FluxStore
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FluxStoreConfig {
    /// Database path
    pub path: String,
    /// Enable compression
    pub compression: bool,
    /// Cache size in MB
    pub cache_size_mb: usize,
    /// Write buffer size in MB
    pub write_buffer_mb: usize,
}

impl Default for FluxStoreConfig {
    fn default() -> Self {
        Self {
            path: "./flux_store".to_string(),
            compression: true,
            cache_size_mb: 64,
            write_buffer_mb: 16,
        }
    }
}

impl FluxStoreConfig {
    pub fn new(path: &str) -> Self {
        Self { path: path.to_string(), ..Default::default() }
    }
}

/// Stored flux state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredFluxState {
    pub id: String,
    pub beams: Vec<BeamTensor>,
    pub position: u8,
    pub confidence: f32,
    pub attributes: Attributes,
    pub timestamp: i64,
    pub metadata: HashMap<String, String>,
}

impl StoredFluxState {
    pub fn new(id: String, beams: Vec<BeamTensor>) -> Self {
        let confidence = if beams.is_empty() {
            0.0
        } else {
            beams.iter().map(|b| b.confidence).sum::<f32>() / beams.len() as f32
        };
        let position = beams.first().map(|b| b.position).unwrap_or(1);

        Self {
            id,
            beams,
            position,
            confidence,
            attributes: Attributes::new(),
            timestamp: chrono::Utc::now().timestamp(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

/// FluxStore - In-memory store (RocksDB integration when feature enabled)
/// 
/// This is a simplified in-memory implementation.
/// When `storage` feature is enabled, would use actual RocksDB.
#[derive(Debug, Clone)]
pub struct FluxStore {
    config: FluxStoreConfig,
    /// In-memory storage (would be RocksDB in production)
    states: HashMap<String, StoredFluxState>,
    /// Position-based index
    position_index: HashMap<u8, Vec<String>>,
    /// Confidence-sorted index (high to low)
    confidence_index: Vec<(String, f32)>,
}

impl FluxStore {
    pub fn new(config: FluxStoreConfig) -> Self {
        Self {
            config,
            states: HashMap::new(),
            position_index: HashMap::new(),
            confidence_index: Vec::new(),
        }
    }

    /// Open store (in production, would open RocksDB)
    pub fn open(config: FluxStoreConfig) -> Result<Self, String> {
        // In production: rocksdb::DB::open(&opts, &config.path)
        Ok(Self::new(config))
    }

    /// Store a flux state
    pub fn put(&mut self, state: StoredFluxState) -> Result<(), String> {
        let id = state.id.clone();
        let position = state.position;
        let confidence = state.confidence;

        // Update position index
        self.position_index.entry(position).or_default().push(id.clone());

        // Update confidence index
        self.confidence_index.push((id.clone(), confidence));
        self.confidence_index.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Store state
        self.states.insert(id, state);

        Ok(())
    }

    /// Get a flux state by ID
    pub fn get(&self, id: &str) -> Option<&StoredFluxState> {
        self.states.get(id)
    }

    /// Delete a flux state
    pub fn delete(&mut self, id: &str) -> Result<(), String> {
        if let Some(state) = self.states.remove(id) {
            // Clean up indices
            if let Some(ids) = self.position_index.get_mut(&state.position) {
                ids.retain(|i| i != id);
            }
            self.confidence_index.retain(|(i, _)| i != id);
        }
        Ok(())
    }

    /// Get states by position
    pub fn get_by_position(&self, position: u8) -> Vec<&StoredFluxState> {
        self.position_index.get(&position)
            .map(|ids| ids.iter().filter_map(|id| self.states.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get top-k states by confidence
    pub fn get_top_confidence(&self, k: usize) -> Vec<&StoredFluxState> {
        self.confidence_index.iter()
            .take(k)
            .filter_map(|(id, _)| self.states.get(id))
            .collect()
    }

    /// Get states above confidence threshold
    pub fn get_above_threshold(&self, threshold: f32) -> Vec<&StoredFluxState> {
        self.states.values()
            .filter(|s| s.confidence >= threshold)
            .collect()
    }

    /// Get sacred position states (3, 6, 9)
    pub fn get_sacred_states(&self) -> Vec<&StoredFluxState> {
        [3, 6, 9].iter()
            .flat_map(|&pos| self.get_by_position(pos))
            .collect()
    }

    /// Count total states
    pub fn len(&self) -> usize {
        self.states.len()
    }

    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }

    /// Compact/optimize (no-op for in-memory, would trigger RocksDB compaction)
    pub fn compact(&mut self) -> Result<(), String> {
        // In production: self.db.compact_range(None::<&[u8]>, None::<&[u8]>);
        Ok(())
    }

    /// Flush to disk (no-op for in-memory)
    pub fn flush(&mut self) -> Result<(), String> {
        // In production: self.db.flush()?;
        Ok(())
    }

    /// Get database path
    pub fn path(&self) -> &str {
        &self.config.path
    }
}

/// Batch operations for efficiency
impl FluxStore {
    /// Put multiple states in a batch
    pub fn put_batch(&mut self, states: Vec<StoredFluxState>) -> Result<(), String> {
        for state in states {
            self.put(state)?;
        }
        Ok(())
    }

    /// Get multiple states by IDs
    pub fn get_batch(&self, ids: &[&str]) -> Vec<Option<&StoredFluxState>> {
        ids.iter().map(|id| self.get(id)).collect()
    }

    /// Delete multiple states
    pub fn delete_batch(&mut self, ids: &[&str]) -> Result<(), String> {
        for id in ids {
            self.delete(id)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flux_store_basic() {
        let config = FluxStoreConfig::new("./test_store");
        let mut store = FluxStore::new(config);

        let state = StoredFluxState::new("test_1".to_string(), vec![BeamTensor::default()]);
        store.put(state).unwrap();

        assert_eq!(store.len(), 1);
        assert!(store.get("test_1").is_some());
    }

    #[test]
    fn test_flux_store_position_index() {
        let mut store = FluxStore::new(FluxStoreConfig::default());

        for i in 1..=9 {
            let mut beam = BeamTensor::default();
            beam.position = i;
            let state = StoredFluxState::new(format!("state_{}", i), vec![beam]);
            store.put(state).unwrap();
        }

        let pos_3 = store.get_by_position(3);
        assert_eq!(pos_3.len(), 1);

        let sacred = store.get_sacred_states();
        assert_eq!(sacred.len(), 3); // positions 3, 6, 9
    }

    #[test]
    fn test_flux_store_confidence() {
        let mut store = FluxStore::new(FluxStoreConfig::default());

        for i in 1..=5 {
            let mut beam = BeamTensor::default();
            beam.confidence = i as f32 / 10.0;
            let state = StoredFluxState::new(format!("state_{}", i), vec![beam]);
            store.put(state).unwrap();
        }

        let top = store.get_top_confidence(2);
        assert_eq!(top.len(), 2);
        assert!(top[0].confidence >= top[1].confidence);
    }
}
