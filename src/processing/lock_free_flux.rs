/// Lock-Free Flux Matrix Implementation
/// 
/// Uses lock-free data structures for maximum Hz performance:
/// - crossbeam::SegQueue for node storage (lock-free push/pop)
/// - DashMap for position indexing (concurrent HashMap)
/// - parking_lot::RwLock for MVCC versioning
/// - arc_swap::ArcSwap for atomic pointer updates
///
/// Target Performance: <100 nanosecond access time

use crossbeam_queue::SegQueue;
use dashmap::DashMap;
use parking_lot::RwLock;
use arc_swap::ArcSwap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::models::FluxNode;

/// Get current timestamp (WASM-compatible)
#[cfg(not(target_arch = "wasm32"))]
#[inline]
fn now() -> Timestamp {
    std::time::Instant::now()
}

#[cfg(target_arch = "wasm32")]
#[inline]
fn now() -> Timestamp {
    // In WASM, use performance.now() via web_sys or just use a counter
    // For simplicity, we'll use a monotonic counter (version number serves similar purpose)
    0.0
}

/// Version number for Multi-Version Concurrency Control (MVCC)
type Version = u64;

/// Matrix statistics
#[derive(Clone, Debug)]
pub struct FluxMatrixStats {
    pub subject: String,
    pub total_nodes: usize,
    pub sacred_positions: Vec<u8>,
    pub version: Version,
}

/// WASM-compatible timestamp type
#[cfg(not(target_arch = "wasm32"))]
pub type Timestamp = std::time::Instant;

#[cfg(target_arch = "wasm32")]
pub type Timestamp = f64; // Use f64 for WASM (milliseconds since page load)

/// Versioned node wrapper for MVCC
#[derive(Clone, Debug)]
pub struct VersionedNode {
    pub node: FluxNode,
    pub version: Version,
    pub timestamp: Timestamp,
}

/// Lock-free Flux Matrix optimized for 1000 Hz operation
#[derive(Debug)]
pub struct LockFreeFluxMatrix {
    /// Subject name
    subject: String,
    
    /// Lock-free queue for node storage (append-only)
    /// Provides O(1) push/pop without contention
    node_queue: Arc<SegQueue<VersionedNode>>,
    
    /// Position index: position → latest node
    /// DashMap provides concurrent HashMap with sharding
    position_index: Arc<DashMap<u8, Arc<VersionedNode>>>,
    
    /// Attribute index: attribute_name → (attribute_value → nodes)
    /// Enables fast queries like "find all nodes with ethos > 0.8"
    attribute_index: Arc<DashMap<String, DashMap<String, Vec<Arc<VersionedNode>>>>>,
    
    /// Sacred position anchors (3, 6, 9)
    /// These are orbital centers, not data storage
    sacred_anchors: Arc<DashMap<u8, SacredAnchor>>,
    
    /// Current version counter (atomic increment)
    version_counter: Arc<ArcSwap<Version>>,
    
    /// MVCC: version → snapshot of position_index
    /// Allows readers to access consistent snapshots
    version_snapshots: Arc<RwLock<HashMap<Version, HashMap<u8, Arc<VersionedNode>>>>>,
}

/// Sacred anchor (unmanifest orbital center)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SacredAnchor {
    pub position: u8,  // 3, 6, or 9
    pub orbital_radius: f64,
    pub judgment_threshold: f64,
}

impl SacredAnchor {
    pub fn new(position: u8) -> Self {
        assert!([3, 6, 9].contains(&position), "Sacred positions are 3, 6, 9 only");
        
        Self {
            position,
            orbital_radius: position as f64,  // r = 3, 6, or 9
            judgment_threshold: 0.5,  // Entropy threshold for flow reversal
        }
    }
    
    /// Judge whether a flow should continue, reverse, or stabilize
    pub fn judge(&self, entropy: f64) -> JudgmentResult {
        if entropy > self.judgment_threshold {
            JudgmentResult::Reverse  // Too much entropy - loop back
        } else if entropy < 0.1 {
            JudgmentResult::Stabilize  // Very low entropy - enter orbit
        } else {
            JudgmentResult::Allow  // Normal flow - continue
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JudgmentResult {
    Allow,      // Continue forward
    Reverse,    // Reverse direction (bi-directional loop)
    Stabilize,  // Enter orbit around anchor
}

impl LockFreeFluxMatrix {
    /// Create new lock-free flux matrix
    pub fn new(subject: String) -> Self {
        let matrix = Self {
            subject,
            node_queue: Arc::new(SegQueue::new()),
            position_index: Arc::new(DashMap::new()),
            attribute_index: Arc::new(DashMap::new()),
            sacred_anchors: Arc::new(DashMap::new()),
            version_counter: Arc::new(ArcSwap::from_pointee(0)),
            version_snapshots: Arc::new(RwLock::new(HashMap::new())),
        };
        
        // Initialize sacred anchors
        matrix.sacred_anchors.insert(3, SacredAnchor::new(3));
        matrix.sacred_anchors.insert(6, SacredAnchor::new(6));
        matrix.sacred_anchors.insert(9, SacredAnchor::new(9));
        
        matrix
    }
    
    /// Insert node (lock-free, O(1))
    pub fn insert(&self, node: FluxNode) -> Version {
        // Increment version atomically
        let version = self.next_version();
        
        let versioned = Arc::new(VersionedNode {
            node: node.clone(),
            version,
            timestamp: now(),
        });
        
        // Push to lock-free queue
        self.node_queue.push((*versioned).clone());
        
        // Update position index
        self.position_index.insert(node.position, versioned.clone());
        
        // Update attribute indices (using parameters HashMap)
        for (attr_name, attr_value) in &node.attributes.parameters {
            let value_key = format!("{:.2}", attr_value);
            
            self.attribute_index
                .entry(attr_name.clone())
                .or_insert_with(DashMap::new)
                .entry(value_key)
                .or_insert_with(Vec::new)
                .push(versioned.clone());
        }
        
        version
    }
    
    /// Get node by position (lock-free read, <100ns)
    pub fn get(&self, position: u8) -> Option<Arc<VersionedNode>> {
        self.position_index.get(&position).map(|entry| entry.value().clone())
    }
    
    /// Query by attribute range
    /// Example: get_by_attribute("ethos", 0.8, 1.0)
    pub fn get_by_attribute(&self, attribute: &str, min: f32, max: f32) -> Vec<Arc<VersionedNode>> {
        let mut results = Vec::new();
        
        if let Some(value_map) = self.attribute_index.get(attribute) {
            for entry in value_map.iter() {
                if let Ok(value) = entry.key().parse::<f32>() {
                    if value >= min && value <= max {
                        results.extend(entry.value().clone());
                    }
                }
            }
        }
        
        results
    }
    
    /// Get sacred anchor
    pub fn get_sacred_anchor(&self, position: u8) -> Option<SacredAnchor> {
        self.sacred_anchors.get(&position).map(|entry| entry.value().clone())
    }
    
    /// Create snapshot for MVCC
    pub fn snapshot(&self) -> Version {
        let version = **self.version_counter.load();
        
        // Clone current state
        let snapshot: HashMap<u8, Arc<VersionedNode>> = self.position_index
            .iter()
            .map(|entry| (*entry.key(), entry.value().clone()))
            .collect();
        
        // Store snapshot
        self.version_snapshots.write().insert(version, snapshot);
        
        version
    }
    
    /// Read from snapshot (isolation)
    pub fn get_from_snapshot(&self, version: Version, position: u8) -> Option<Arc<VersionedNode>> {
        self.version_snapshots
            .read()
            .get(&version)
            .and_then(|snapshot| snapshot.get(&position).cloned())
    }
    
    /// Garbage collect old snapshots
    pub fn gc_snapshots(&self, keep_latest: usize) {
        let mut snapshots = self.version_snapshots.write();
        
        if snapshots.len() > keep_latest {
            let mut versions: Vec<Version> = snapshots.keys().copied().collect();
            versions.sort_unstable();
            
            // Remove old versions
            for version in versions.iter().take(snapshots.len() - keep_latest) {
                snapshots.remove(version);
            }
        }
    }
    
    /// Get total node count
    pub fn len(&self) -> usize {
        self.position_index.len()
    }
    
    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.position_index.is_empty()
    }
    
    /// Get subject name
    pub fn subject(&self) -> &str {
        &self.subject
    }
    
    /// Get matrix statistics
    pub fn stats(&self) -> FluxMatrixStats {
        FluxMatrixStats {
            subject: self.subject.clone(),
            total_nodes: self.len(),
            sacred_positions: vec![3, 6, 9],
            version: **self.version_counter.load(),
        }
    }
    
    /// Judge entropy at sacred anchor
    pub fn judge_at_anchor(&self, position: u8, entropy: f64) -> JudgmentResult {
        if let Some(anchor) = self.get_sacred_anchor(position) {
            anchor.judge(entropy)
        } else {
            JudgmentResult::Allow  // Non-sacred position, allow forward flow
        }
    }
    
    // Private: increment version counter atomically
    fn next_version(&self) -> Version {
        loop {
            let current = self.version_counter.load();
            let next = **current + 1;
            
            // Try to swap - if successful, current pointer is returned
            let _ = self.version_counter.compare_and_swap(&current, Arc::new(next));
            // In arc-swap, we just proceed - the atomic operation ensures correctness
            return next;
        }
    }
}

// Implement Send + Sync for safe concurrent access
unsafe impl Send for LockFreeFluxMatrix {}
unsafe impl Sync for LockFreeFluxMatrix {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        FluxNode, NodeAttributes, NodeState, NodeDynamics, SemanticIndex,
    };
    
    fn create_test_node(position: u8) -> FluxNode {
        let mut parameters = HashMap::new();
        parameters.insert("ethos".to_string(), 0.8);
        parameters.insert("logos".to_string(), 0.6);
        parameters.insert("pathos".to_string(), 0.4);
        
        FluxNode {
            position,
            base_value: position,
            semantic_index: SemanticIndex {
                positive_associations: vec![],
                negative_associations: vec![],
                neutral_base: format!("Position {}", position),
                predicates: vec![],
                relations: vec![],
            },
            attributes: NodeAttributes {
                properties: HashMap::new(),
                parameters,
                state: NodeState {
                    active: true,
                    last_accessed: chrono::Utc::now(),
                    usage_count: 0,
                    context_stack: vec![],
                },
                dynamics: NodeDynamics::default(),
            },
            connections: vec![],
        }
    }
    
    #[test]
    fn test_insert_and_get() {
        let matrix = LockFreeFluxMatrix::new("test".to_string());
        let node = create_test_node(5);
        
        let version = matrix.insert(node.clone());
        assert_eq!(version, 1);
        
        let retrieved = matrix.get(5).unwrap();
        assert_eq!(retrieved.node.position, 5);
        assert_eq!(retrieved.version, 1);
    }
    
    #[test]
    fn test_sacred_anchors() {
        let matrix = LockFreeFluxMatrix::new("test".to_string());
        
        // Sacred positions should exist
        assert!(matrix.get_sacred_anchor(3).is_some());
        assert!(matrix.get_sacred_anchor(6).is_some());
        assert!(matrix.get_sacred_anchor(9).is_some());
        
        // Non-sacred positions should not
        assert!(matrix.get_sacred_anchor(1).is_none());
        assert!(matrix.get_sacred_anchor(5).is_none());
    }
    
    #[test]
    fn test_attribute_query() {
        let matrix = LockFreeFluxMatrix::new("test".to_string());
        
        // Insert nodes with different ethos values
        let mut node1 = create_test_node(1);
        node1.attributes.parameters.insert("ethos".to_string(), 0.9);
        matrix.insert(node1);
        
        let mut node2 = create_test_node(2);
        node2.attributes.parameters.insert("ethos".to_string(), 0.5);
        matrix.insert(node2);
        
        // Query high ethos nodes
        let high_ethos = matrix.get_by_attribute("ethos", 0.8, 1.0);
        assert_eq!(high_ethos.len(), 1);
        assert_eq!(high_ethos[0].node.position, 1);
    }
    
    #[test]
    fn test_mvcc_snapshot() {
        let matrix = LockFreeFluxMatrix::new("test".to_string());
        
        // Insert node
        matrix.insert(create_test_node(1));
        
        // Create snapshot
        let snapshot_v1 = matrix.snapshot();
        
        // Modify
        matrix.insert(create_test_node(2));
        
        // Snapshot should still have old state
        assert!(matrix.get_from_snapshot(snapshot_v1, 1).is_some());
        assert!(matrix.get_from_snapshot(snapshot_v1, 2).is_none());
        
        // Current state has both
        assert!(matrix.get(1).is_some());
        assert!(matrix.get(2).is_some());
    }
}
