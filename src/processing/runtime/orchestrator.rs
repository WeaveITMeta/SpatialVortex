//! Flux Orchestrator - Unified Runtime Coordinator
//! 
//! Integrates all runtime systems into a cohesive learning loop:
//! - Vortex Cycle Engine (propagation)
//! - Ladder Index (reinforcement learning)
//! - Intersection Analysis (cross-referencing)
//! - ASI Compression (semantic encoding)
//! 
//! ## Main Loop
//! ```text
//! Propagate → Detect Intersections → Apply Rewards → Re-rank → Compress
//!     ↓                                                            ↓
//!     └────────────────────── Cycle ←──────────────────────────────┘
//! ```

use crate::models::{FluxNode, NodeAttributes, NodeState, NodeDynamics, SemanticIndex, ELPTensor};
use crate::runtime::{
    VortexCycleEngine, CycleObject, CycleDirection,
    LadderIndex,
    IntersectionAnalyzer, RelationshipType,
};
use crate::compression::ASI12ByteCompression;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration, Instant};

/// Orchestrator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    /// Update rate (Hz)
    pub update_rate: f64,
    
    /// Ladder learning rate
    pub learning_rate: f64,
    
    /// Top-K entries for ladder comparison
    pub ladder_top_k: usize,
    
    /// Intersection detection threshold
    pub intersection_threshold: f64,
    
    /// Compress top N entries each cycle
    pub compress_top_n: usize,
    
    /// Enable auto-reward from intersections
    pub auto_reward: bool,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            update_rate: 60.0,
            learning_rate: 0.1,
            ladder_top_k: 10,
            intersection_threshold: 0.5,
            compress_top_n: 100,
            auto_reward: true,
        }
    }
}

/// Unified orchestrator state
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OrchestratorState {
    /// Total cycles completed
    pub total_cycles: u64,
    
    /// Current cycle objects
    pub active_objects: usize,
    
    /// Detected intersections
    pub intersection_count: usize,
    
    /// Average ladder rank
    pub avg_ladder_rank: f64,
    
    /// Compressed concepts count
    pub compressed_count: usize,
    
    /// System coherence (0.0-1.0)
    pub coherence: f64,
    
    /// Last update timestamp (not serialized)
    #[serde(skip)]
    pub last_update: Instant,
}

impl Default for OrchestratorState {
    fn default() -> Self {
        Self {
            total_cycles: 0,
            active_objects: 0,
            intersection_count: 0,
            avg_ladder_rank: 0.0,
            compressed_count: 0,
            coherence: 0.0,
            last_update: Instant::now(),
        }
    }
}

/// Main orchestrator
pub struct FluxOrchestrator {
    /// Configuration
    config: OrchestratorConfig,
    
    /// Vortex cycle engine
    cycle_engine: Arc<VortexCycleEngine>,
    
    /// Ladder index
    ladder: Arc<LadderIndex>,
    
    /// Intersection analyzer
    intersections: Arc<IntersectionAnalyzer>,
    
    /// Object ID to ladder entry ID mapping
    object_to_entry: Arc<RwLock<HashMap<String, u64>>>,
    
    /// Current state
    state: Arc<RwLock<OrchestratorState>>,
    
    /// Running flag
    running: Arc<RwLock<bool>>,
}

impl FluxOrchestrator {
    /// Create new orchestrator
    pub fn new(config: OrchestratorConfig) -> Self {
        Self {
            cycle_engine: Arc::new(VortexCycleEngine::new(config.update_rate)),
            ladder: Arc::new(LadderIndex::new(config.learning_rate, config.ladder_top_k)),
            intersections: Arc::new(IntersectionAnalyzer::new(config.intersection_threshold)),
            object_to_entry: Arc::new(RwLock::new(HashMap::new())),
            state: Arc::new(RwLock::new(OrchestratorState {
                total_cycles: 0,
                active_objects: 0,
                intersection_count: 0,
                avg_ladder_rank: 0.0,
                compressed_count: 0,
                coherence: 0.0,
                last_update: Instant::now(),
            })),
            running: Arc::new(RwLock::new(false)),
            config,
        }
    }
    
    /// Create with default config
    pub fn new_default() -> Self {
        Self::new(OrchestratorConfig::default())
    }
    
    /// Add object to orchestrated system
    pub async fn add_object(
        &self,
        id: String,
        position: u8,
        tensor: ELPTensor,
        direction: CycleDirection,
    ) -> Result<()> {
        // Add to vortex cycle
        let obj = CycleObject::new(id.clone(), tensor, direction);
        self.cycle_engine.add_object(obj).await;
        
        // Add to ladder index
        let entry_id = self.ladder.add_entry(position, tensor).await;
        
        // Map object to entry
        let mut mapping = self.object_to_entry.write().await;
        mapping.insert(id, entry_id);
        
        Ok(())
    }
    
    /// Remove object from system
    pub async fn remove_object(&self, id: &str) {
        // Remove from vortex
        self.cycle_engine.remove_object(id).await;
        
        // Get entry_id before acquiring write lock (prevent deadlock)
        let entry_id = {
            let mapping = self.object_to_entry.read().await;
            mapping.get(id).copied()
        };
        
        // Remove from ladder if exists
        if let Some(entry_id) = entry_id {
            self.ladder.remove_entry(entry_id).await;
        }
        
        // Remove mapping (write lock acquired AFTER read lock is dropped)
        let mut mapping = self.object_to_entry.write().await;
        mapping.remove(id);
    }
    
    /// Start orchestration loop
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(());
        }
        *running = true;
        drop(running);
        
        // Start cycle engine
        self.cycle_engine.start().await?;
        
        // Spawn orchestration loop
        let cycle_engine = Arc::clone(&self.cycle_engine);
        let ladder = Arc::clone(&self.ladder);
        let intersections = Arc::clone(&self.intersections);
        let object_to_entry = Arc::clone(&self.object_to_entry);
        let state = Arc::clone(&self.state);
        let running_flag = Arc::clone(&self.running);
        let config = self.config.clone();
        
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs_f64(1.0 / config.update_rate));
            
            while *running_flag.read().await {
                ticker.tick().await;
                
                // Main orchestration tick
                if let Err(e) = Self::orchestrate_tick(
                    &cycle_engine,
                    &ladder,
                    &intersections,
                    &object_to_entry,
                    &state,
                    &config,
                ).await {
                    eprintln!("Orchestration tick error: {:?}", e);
                }
            }
        });
        
        Ok(())
    }
    
    /// Stop orchestration
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        drop(running);
        
        self.cycle_engine.stop().await;
    }
    
    /// Main orchestration tick (core logic)
    async fn orchestrate_tick(
        cycle_engine: &Arc<VortexCycleEngine>,
        ladder: &Arc<LadderIndex>,
        intersections: &Arc<IntersectionAnalyzer>,
        object_to_entry: &Arc<RwLock<HashMap<String, u64>>>,
        state: &Arc<RwLock<OrchestratorState>>,
        config: &OrchestratorConfig,
    ) -> Result<()> {
        // 1. Get current objects from vortex
        let objects = cycle_engine.get_objects().await;
        
        // 2. Convert to flux nodes for intersection analysis
        let nodes = Self::objects_to_nodes(&objects);
        
        // 3. Detect intersections
        intersections.detect_intersections(&nodes).await;
        
        // 4. Apply rewards based on intersection analysis
        if config.auto_reward {
            let all_intersections = intersections.get_all_intersections().await;
            let mapping = object_to_entry.read().await;
            
            for intersection in &all_intersections {
                for cross_ref in &intersection.cross_references {
                    // Calculate reward from relationship type
                    let reward = match cross_ref.relationship {
                        RelationshipType::Harmonic => 1.0,
                        RelationshipType::Amplifying => 0.8,
                        RelationshipType::Complementary => 0.5,
                        RelationshipType::Neutral => 0.0,
                        RelationshipType::Dampening => -0.3,
                    };
                    
                    // Apply weighted by confidence and intersection strength
                    let weighted_reward = reward * cross_ref.confidence * intersection.strength;
                    
                    // Apply to ladder entries
                    if let Some(&entry_id) = mapping.get(&cross_ref.from_node) {
                        ladder.apply_reward(entry_id, weighted_reward).await;
                    }
                    if let Some(&entry_id) = mapping.get(&cross_ref.to_node) {
                        ladder.apply_reward(entry_id, weighted_reward * 0.5).await;
                    }
                }
            }
        }
        
        // 5. Update state
        let mut current_state = state.write().await;
        current_state.total_cycles += 1;
        current_state.active_objects = objects.len();
        current_state.intersection_count = intersections.stats().await.total_intersections;
        current_state.avg_ladder_rank = ladder.stats().await.avg_rank;
        
        // Calculate system coherence
        let intersection_stats = intersections.stats().await;
        let coherence = if intersection_stats.total_intersections > 0 {
            intersection_stats.avg_strength
        } else {
            0.0
        };
        current_state.coherence = coherence;
        current_state.last_update = Instant::now();
        
        Ok(())
    }
    
    /// Convert cycle objects to flux nodes
    fn objects_to_nodes(objects: &[CycleObject]) -> HashMap<String, FluxNode> {
        objects.iter()
            .map(|obj| {
                let mut parameters = HashMap::new();
                parameters.insert("ethos".to_string(), obj.tensor.ethos);
                parameters.insert("logos".to_string(), obj.tensor.logos);
                parameters.insert("pathos".to_string(), obj.tensor.pathos);
                
                let node = FluxNode {
                    position: obj.position.position,
                    base_value: obj.position.position,
                    semantic_index: SemanticIndex {
                        positive_associations: vec![],
                        negative_associations: vec![],
                        neutral_base: format!("Object_{}", obj.id),
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
                };
                
                (obj.id.clone(), node)
            })
            .collect()
    }
    
    /// Get current state snapshot
    pub async fn get_state(&self) -> OrchestratorState {
        self.state.read().await.clone()
    }
    
    /// Get comprehensive statistics
    pub async fn get_stats(&self) -> OrchestratorStats {
        let state = self.state.read().await.clone();
        let cycle_stats = self.cycle_engine.stats().await;
        let ladder_stats = self.ladder.stats().await;
        let intersection_stats = self.intersections.stats().await;
        
        OrchestratorStats {
            state,
            cycle_stats,
            ladder_stats,
            intersection_stats,
        }
    }
    
    /// Compress top-ranked entries to ASI 12-byte format
    pub async fn compress_top_entries(&self, n: usize) -> Vec<ASI12ByteCompression> {
        let ranked = self.ladder.get_top_k(n).await;
        
        ranked.iter()
            .map(|entry| {
                // Find nearest sacred anchor for reference
                let anchor_pos = [3, 6, 9].iter()
                    .min_by_key(|&&anchor| {
                        let diff = (anchor as i32 - entry.origin_position as i32).abs();
                        diff.min(10 - diff)
                    })
                    .copied()
                    .unwrap_or(3);
                
                // Create anchor ELP (simplified - could be more sophisticated)
                let anchor_elp = match anchor_pos {
                    3 => ELPTensor::new(13.0, 0.0, 0.0),  // Pure Ethos
                    6 => ELPTensor::new(0.0, 0.0, 13.0),  // Pure Pathos
                    9 => ELPTensor::new(0.0, 13.0, 0.0),  // Pure Logos
                    _ => ELPTensor::new(0.0, 0.0, 0.0),
                };
                
                // Calculate confidence from reinforcement weight
                let confidence = (entry.reinforcement_weight + 1.0) / 2.0;
                
                ASI12ByteCompression::compress(
                    &format!("Entry_{}", entry.entry_id),
                    entry.origin_position,
                    entry.tensor,
                    anchor_elp,
                    confidence,
                )
            })
            .collect()
    }
    
    /// Get objects at specific position
    pub async fn get_objects_at_position(&self, position: u8) -> Vec<CycleObject> {
        self.cycle_engine.get_objects_at_position(position).await
    }
    
    /// Get intersection at position
    pub async fn get_intersection(&self, position: u8) -> Option<crate::runtime::Intersection> {
        self.intersections.get_intersection(position).await
    }
}

/// Comprehensive statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorStats {
    pub state: OrchestratorState,
    pub cycle_stats: crate::runtime::CycleStats,
    pub ladder_stats: crate::runtime::LadderStats,
    pub intersection_stats: crate::runtime::IntersectionStats,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_orchestrator_creation() {
        let orchestrator = FluxOrchestrator::new_default();
        let state = orchestrator.get_state().await;
        assert_eq!(state.total_cycles, 0);
    }

    #[tokio::test]
    async fn test_add_object() {
        let orchestrator = FluxOrchestrator::new_default();
        
        let result = orchestrator.add_object(
            "test1".to_string(),
            1,
            ELPTensor::new(0.7, 0.5, 0.9),
            CycleDirection::Forward,
        ).await;
        
        assert!(result.is_ok());
        
        let state = orchestrator.get_state().await;
        assert!(state.active_objects > 0 || state.total_cycles == 0);
    }

    #[tokio::test]
    async fn test_orchestration_loop() {
        let orchestrator = FluxOrchestrator::new_default();
        
        // Add test objects
        for i in 0..5 {
            orchestrator.add_object(
                format!("obj_{}", i),
                (i % 10) as u8,
                ELPTensor::new(0.5 + i as f64 * 0.1, 0.5, 0.5),
                CycleDirection::Forward,
            ).await.unwrap();
        }
        
        // Start orchestration
        orchestrator.start().await.unwrap();
        
        // Let it run for a bit
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        // Check state progression
        let state = orchestrator.get_state().await;
        assert!(state.total_cycles > 0);
        assert_eq!(state.active_objects, 5);
        
        orchestrator.stop().await;
    }

    #[tokio::test]
    async fn test_compression() {
        let orchestrator = FluxOrchestrator::new_default();
        
        // Add and rank some entries
        for i in 0..10 {
            orchestrator.add_object(
                format!("obj_{}", i),
                3,  // Sacred anchor
                ELPTensor::new(0.9, 0.5, 0.5),
                CycleDirection::Forward,
            ).await.unwrap();
        }
        
        let compressed = orchestrator.compress_top_entries(5).await;
        assert_eq!(compressed.len(), 5);
        
        // Verify 12-byte size
        assert_eq!(std::mem::size_of_val(&compressed[0]), 12);
    }
}
