//! Ladder Index - Dynamic Ranking System for Flux Matrix
//! 
//! A reinforcement learning-based ranking mechanism that determines node positions
//! during vortex cycle propagation. Acts as both a compression algorithm and a
//! dynamic state manager.
//! 
//! ## Concept
//! - **Static ID**: Each node has a permanent entry ID
//! - **Dynamic Rank**: Position changes based on proximity to anchors and ELP similarity
//! - **Value Flux**: Nodes lose their indexed value when re-entering propagation cycles
//! - **Compression**: Rankings compress high-dimensional tensor data to ordinal positions
//! - **Reasoning**: Supports deductive and abductive inference
//! 
//! ## Cycle Behavior
//! ```text
//! Enter → Ranked → Propagate (1→2→4→8→7→5→1) → Re-rank → Exit
//!   ↑                                                        ↓
//!   └────────────────── Re-enter ←──────────────────────────┘
//! ```

use crate::models::ELPTensor;
use crate::runtime::vortex_cycle::SACRED_ANCHORS;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BTreeMap};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Entry in the ladder index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LadderEntry {
    /// Static identifier (never changes)
    pub entry_id: u64,
    
    /// Original position when first entered (0-9)
    pub origin_position: u8,
    
    /// Current dynamic rank (changes during flux)
    pub current_rank: f64,
    
    /// ELP tensor for similarity comparison
    pub tensor: ELPTensor,
    
    /// Reinforcement weight (±1.0, pulls toward/away from anchors)
    pub reinforcement_weight: f64,
    
    /// Number of cycles this entry has been through
    pub cycle_count: u64,
    
    /// Proximity to nearest sacred anchor (0.0-1.0)
    pub anchor_proximity: f64,
    
    /// Metadata for reasoning
    pub metadata: HashMap<String, String>,
}

impl LadderEntry {
    /// Create new ladder entry
    pub fn new(entry_id: u64, position: u8, tensor: ELPTensor) -> Self {
        let anchor_proximity = Self::calculate_anchor_proximity(position);
        
        Self {
            entry_id,
            origin_position: position,
            current_rank: 0.0,
            tensor,
            reinforcement_weight: 0.0,
            cycle_count: 0,
            anchor_proximity,
            metadata: HashMap::new(),
        }
    }
    
    /// Calculate proximity to nearest sacred anchor
    #[inline]
    fn calculate_anchor_proximity(position: u8) -> f64 {
        let nearest = SACRED_ANCHORS
            .iter()
            .map(|&anchor| {
                let diff = (anchor as i32 - position as i32).abs();
                diff.min(10 - diff) as f64
            })
            .fold(f64::INFINITY, f64::min);
        
        // Closer = higher value (0.0 = far, 1.0 = at anchor)
        1.0 - (nearest / 5.0)
    }
    
    /// Apply reinforcement learning update
    pub fn apply_reinforcement(&mut self, reward: f64, learning_rate: f64) {
        // Update weight toward anchors if reward is positive
        self.reinforcement_weight += learning_rate * reward * self.anchor_proximity;
        self.reinforcement_weight = self.reinforcement_weight.clamp(-1.0, 1.0);
    }
    
    /// Calculate rank based on multiple factors
    pub fn calculate_rank(&self, comparison_entries: &[&LadderEntry]) -> f64 {
        let mut rank = 0.0;
        
        // 1. Base rank from anchor proximity
        rank += self.anchor_proximity * 100.0;
        
        // 2. Reinforcement weight contribution
        rank += self.reinforcement_weight * 50.0;
        
        // 3. ELP similarity to high-performing entries
        if !comparison_entries.is_empty() {
            let avg_similarity: f64 = comparison_entries
                .iter()
                .map(|other| {
                    let distance = self.tensor.distance(&other.tensor);
                    1.0 / (1.0 + distance) // Inverse distance = similarity
                })
                .sum::<f64>() / comparison_entries.len() as f64;
            
            rank += avg_similarity * 30.0;
        }
        
        // 4. Experience bonus (more cycles = more stable)
        rank += (self.cycle_count as f64).ln() * 10.0;
        
        rank
    }
    
    /// Reset dynamic values when re-entering flux
    pub fn reset_for_flux(&mut self) {
        // Lose indexed value but keep static ID and history
        self.current_rank = 0.0;
        self.cycle_count += 1;
        // reinforcement_weight and anchor_proximity persist
    }
}

/// Ladder index manager
pub struct LadderIndex {
    /// All entries indexed by static ID
    entries: Arc<RwLock<HashMap<u64, LadderEntry>>>,
    
    /// Ranked entries (rank → entry_id)
    rankings: Arc<RwLock<BTreeMap<OrderedFloat, u64>>>,
    
    /// Next entry ID
    next_id: Arc<RwLock<u64>>,
    
    /// Learning rate for reinforcement
    learning_rate: f64,
    
    /// Top-K entries for comparison
    top_k: usize,
}

/// Wrapper for f64 that implements Ord for BTreeMap
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
struct OrderedFloat(f64);

impl Eq for OrderedFloat {}

impl Ord for OrderedFloat {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.partial_cmp(&other.0).unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl LadderIndex {
    /// Create new ladder index
    pub fn new(learning_rate: f64, top_k: usize) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            rankings: Arc::new(RwLock::new(BTreeMap::new())),
            next_id: Arc::new(RwLock::new(0)),
            learning_rate,
            top_k,
        }
    }
    
    /// Add entry to ladder
    pub async fn add_entry(&self, position: u8, tensor: ELPTensor) -> u64 {
        let entry_id = {
            let mut next_id = self.next_id.write().await;
            let id = *next_id;
            *next_id += 1;
            id
        };
        
        let entry = LadderEntry::new(entry_id, position, tensor);
        
        let mut entries = self.entries.write().await;
        entries.insert(entry_id, entry);
        
        // Initial ranking
        self.rerank_entry(entry_id).await;
        
        entry_id
    }
    
    /// Remove entry from ladder
    pub async fn remove_entry(&self, entry_id: u64) -> Option<LadderEntry> {
        let mut entries = self.entries.write().await;
        let entry = entries.remove(&entry_id)?;
        
        // Remove from rankings
        let mut rankings = self.rankings.write().await;
        rankings.retain(|_, id| *id != entry_id);
        
        Some(entry)
    }
    
    /// Apply reinforcement learning reward to entry
    pub async fn apply_reward(&self, entry_id: u64, reward: f64) {
        let mut entries = self.entries.write().await;
        
        if let Some(entry) = entries.get_mut(&entry_id) {
            entry.apply_reinforcement(reward, self.learning_rate);
        }
        
        drop(entries);
        
        // Re-rank after update
        self.rerank_entry(entry_id).await;
    }
    
    /// Re-rank a specific entry
    async fn rerank_entry(&self, entry_id: u64) {
        let entries = self.entries.read().await;
        
        // Get top-K entries for comparison
        let rankings = self.rankings.read().await;
        let top_entries: Vec<&LadderEntry> = rankings
            .values()
            .rev()
            .take(self.top_k)
            .filter_map(|id| entries.get(id))
            .collect();
        
        if let Some(entry) = entries.get(&entry_id) {
            let new_rank = entry.calculate_rank(&top_entries);
            
            drop(entries);
            drop(rankings);
            
            // Update ranking
            let mut rankings = self.rankings.write().await;
            
            // Remove old ranking
            rankings.retain(|_, id| *id != entry_id);
            
            // Insert new ranking
            rankings.insert(OrderedFloat(new_rank), entry_id);
            
            // Update entry's current rank
            let mut entries = self.entries.write().await;
            if let Some(entry) = entries.get_mut(&entry_id) {
                entry.current_rank = new_rank;
            }
        }
    }
    
    /// Reset entry for re-entry into flux cycle
    pub async fn reset_for_flux(&self, entry_id: u64) {
        let mut entries = self.entries.write().await;
        
        if let Some(entry) = entries.get_mut(&entry_id) {
            entry.reset_for_flux();
        }
        
        drop(entries);
        
        // Re-rank with reset values
        self.rerank_entry(entry_id).await;
    }
    
    /// Get ranked entries (highest to lowest)
    pub async fn get_ranked_entries(&self) -> Vec<LadderEntry> {
        let rankings = self.rankings.read().await;
        let entries = self.entries.read().await;
        
        rankings
            .values()
            .rev()
            .filter_map(|id| entries.get(id).cloned())
            .collect()
    }
    
    /// Get entry by ID
    pub async fn get_entry(&self, entry_id: u64) -> Option<LadderEntry> {
        let entries = self.entries.read().await;
        entries.get(&entry_id).cloned()
    }
    
    /// Get top-K ranked entries
    pub async fn get_top_k(&self, k: usize) -> Vec<LadderEntry> {
        let rankings = self.rankings.read().await;
        let entries = self.entries.read().await;
        
        rankings
            .values()
            .rev()
            .take(k)
            .filter_map(|id| entries.get(id).cloned())
            .collect()
    }
    
    /// Compress entries to 2D positions based on rankings
    pub async fn compress_to_positions(&self) -> HashMap<u64, u8> {
        let ranked = self.get_ranked_entries().await;
        let total = ranked.len();
        
        ranked
            .into_iter()
            .enumerate()
            .map(|(idx, entry)| {
                // Map rank to position 0-9
                let normalized = idx as f64 / total.max(1) as f64;
                let position = (normalized * 9.0).round() as u8;
                (entry.entry_id, position)
            })
            .collect()
    }
    
    /// Deductive reasoning: Find entries matching criteria
    pub async fn deduce(&self, predicate: impl Fn(&LadderEntry) -> bool) -> Vec<LadderEntry> {
        let entries = self.entries.read().await;
        
        entries
            .values()
            .filter(|entry| predicate(entry))
            .cloned()
            .collect()
    }
    
    /// Abductive reasoning: Find best explanation for observation
    pub async fn abduce(&self, target_tensor: &ELPTensor, top_n: usize) -> Vec<(LadderEntry, f64)> {
        let entries = self.entries.read().await;
        
        let mut similarities: Vec<(LadderEntry, f64)> = entries
            .values()
            .map(|entry| {
                let distance = entry.tensor.distance(target_tensor);
                let similarity = 1.0 / (1.0 + distance);
                (entry.clone(), similarity)
            })
            .collect();
        
        // Sort by similarity (highest first)
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        similarities.into_iter().take(top_n).collect()
    }
    
    /// Get statistics
    pub async fn stats(&self) -> LadderStats {
        let entries = self.entries.read().await;
        let rankings = self.rankings.read().await;
        
        let total_entries = entries.len();
        let avg_rank = if !rankings.is_empty() {
            rankings.keys().map(|k| k.0).sum::<f64>() / rankings.len() as f64
        } else {
            0.0
        };
        
        let avg_cycles = if !entries.is_empty() {
            entries.values().map(|e| e.cycle_count).sum::<u64>() as f64 / entries.len() as f64
        } else {
            0.0
        };
        
        let avg_reinforcement = if !entries.is_empty() {
            entries.values().map(|e| e.reinforcement_weight).sum::<f64>() / entries.len() as f64
        } else {
            0.0
        };
        
        LadderStats {
            total_entries,
            avg_rank,
            avg_cycles,
            avg_reinforcement,
            learning_rate: self.learning_rate,
        }
    }
}

/// Ladder index statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LadderStats {
    pub total_entries: usize,
    pub avg_rank: f64,
    pub avg_cycles: f64,
    pub avg_reinforcement: f64,
    pub learning_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ladder_entry_creation() {
        let tensor = ELPTensor::new(0.7, 0.5, 0.9);
        let entry = LadderEntry::new(1, 3, tensor);
        
        assert_eq!(entry.entry_id, 1);
        assert_eq!(entry.origin_position, 3);
        assert!(entry.anchor_proximity > 0.9); // Position 3 is a sacred anchor
    }

    #[test]
    fn test_reinforcement_learning() {
        let tensor = ELPTensor::new(0.7, 0.5, 0.9);
        let mut entry = LadderEntry::new(1, 5, tensor);
        
        let initial_weight = entry.reinforcement_weight;
        
        // Positive reward
        entry.apply_reinforcement(1.0, 0.1);
        assert!(entry.reinforcement_weight > initial_weight);
        
        // Negative reward
        entry.apply_reinforcement(-1.0, 0.1);
        // Weight should decrease
    }

    #[test]
    fn test_flux_reset() {
        let tensor = ELPTensor::new(0.7, 0.5, 0.9);
        let mut entry = LadderEntry::new(1, 3, tensor);
        
        entry.current_rank = 100.0;
        entry.cycle_count = 0;
        
        entry.reset_for_flux();
        
        assert_eq!(entry.current_rank, 0.0); // Rank reset
        assert_eq!(entry.cycle_count, 1); // Cycle incremented
        assert_eq!(entry.entry_id, 1); // ID preserved
    }

    #[tokio::test]
    async fn test_ladder_index() {
        let ladder = LadderIndex::new(0.1, 10);
        
        let _id1 = ladder.add_entry(3, ELPTensor::new(0.7, 0.5, 0.9)).await;
        let _id2 = ladder.add_entry(6, ELPTensor::new(0.5, 0.5, 0.8)).await;
        let _id3 = ladder.add_entry(1, ELPTensor::new(0.6, 0.6, 0.7)).await;
        
        let ranked = ladder.get_ranked_entries().await;
        assert_eq!(ranked.len(), 3);
        
        // Top entry should have highest rank
        assert!(ranked[0].current_rank >= ranked[1].current_rank);
        assert!(ranked[1].current_rank >= ranked[2].current_rank);
    }

    #[tokio::test]
    async fn test_compression_to_positions() {
        let ladder = LadderIndex::new(0.1, 10);
        
        for i in 0..10 {
            ladder.add_entry(i, ELPTensor::new(0.5, 0.5, 0.5)).await;
        }
        
        let positions = ladder.compress_to_positions().await;
        assert_eq!(positions.len(), 10);
        
        // All positions should be 0-9
        for pos in positions.values() {
            assert!(*pos <= 9);
        }
    }

    #[tokio::test]
    async fn test_deductive_reasoning() {
        let ladder = LadderIndex::new(0.1, 10);
        
        ladder.add_entry(3, ELPTensor::new(0.9, 0.5, 0.5)).await; // High ethos
        ladder.add_entry(6, ELPTensor::new(0.5, 0.5, 0.9)).await; // High pathos
        ladder.add_entry(9, ELPTensor::new(0.5, 0.9, 0.5)).await; // High logos
        
        // Find entries with high ethos
        let high_ethos = ladder.deduce(|e| e.tensor.ethos > 0.8).await;
        assert_eq!(high_ethos.len(), 1);
        assert_eq!(high_ethos[0].origin_position, 3);
    }

    #[tokio::test]
    async fn test_abductive_reasoning() {
        let ladder = LadderIndex::new(0.1, 10);
        
        ladder.add_entry(3, ELPTensor::new(0.9, 0.5, 0.5)).await;
        ladder.add_entry(6, ELPTensor::new(0.5, 0.5, 0.9)).await;
        ladder.add_entry(9, ELPTensor::new(0.5, 0.9, 0.5)).await;
        
        // Find best explanation for observation
        let target = ELPTensor::new(0.85, 0.5, 0.5);
        let explanations = ladder.abduce(&target, 2).await;
        
        assert_eq!(explanations.len(), 2);
        // Most similar should be position 3 (high ethos)
        assert_eq!(explanations[0].0.origin_position, 3);
    }
}
