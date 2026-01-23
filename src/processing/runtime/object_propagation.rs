//! Object Propagation System
//! 
//! Manages multiple objects floating through the vortex cycle.
//! Each object represents a subject in the flux matrix, propagating
//! through positions with exponential runtime dynamics.

use crate::runtime::vortex_cycle::{CycleObject, CycleDirection, VortexCycleEngine};
use crate::models::{FluxNode, ELPTensor};
use crate::error::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Propagation behavior for flux matrix subjects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PropagationMode {
    /// Inference mode: Forward doubling sequence
    Inference,
    /// Training mode: Backward halving sequence
    Training,
    /// Oscillating: Alternates between forward/backward
    Oscillating,
    /// Custom: User-defined pattern
    Custom,
}

/// Configuration for object propagation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagationConfig {
    /// Propagation mode
    pub mode: PropagationMode,
    
    /// Base velocity (positions per second)
    pub velocity: f64,
    
    /// Exponential slowdown factor
    pub exponential_slowdown: bool,
    
    /// Sacred anchor attraction strength (0.0-1.0)
    pub sacred_attraction: f64,
    
    /// Target position (if any)
    pub target_position: Option<u8>,
}

impl Default for PropagationConfig {
    fn default() -> Self {
        Self {
            mode: PropagationMode::Inference,
            velocity: 1.0,
            exponential_slowdown: true,
            sacred_attraction: 0.3,
            target_position: None,
        }
    }
}

/// Manager for flux matrix object propagation
pub struct ObjectPropagationManager {
    /// Cycle engine
    engine: Arc<VortexCycleEngine>,
    
    /// Objects mapped by flux node position
    position_map: Arc<RwLock<HashMap<u8, Vec<String>>>>,
    
    /// Configuration per object
    configs: Arc<RwLock<HashMap<String, PropagationConfig>>>,
}

impl ObjectPropagationManager {
    /// Create new propagation manager
    pub fn new(update_rate: f64) -> Self {
        Self {
            engine: Arc::new(VortexCycleEngine::new(update_rate)),
            position_map: Arc::new(RwLock::new(HashMap::new())),
            configs: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Start propagation engine
    pub async fn start(&self) -> Result<()> {
        self.engine.start().await
    }
    
    /// Stop propagation engine
    pub async fn stop(&self) {
        self.engine.stop().await
    }
    
    /// Add flux node as propagating object
    pub async fn add_node(
        &self,
        node: &FluxNode,
        config: PropagationConfig,
    ) -> Result<String> {
        let id = format!("node_{}_{}", node.position, uuid::Uuid::new_v4());
        
        // Extract ELP from node
        let tensor = ELPTensor::new(
            *node.attributes.parameters.get("ethos").unwrap_or(&0.5),
            *node.attributes.parameters.get("logos").unwrap_or(&0.5),
            *node.attributes.parameters.get("pathos").unwrap_or(&0.5),
        );
        
        // Determine direction from mode
        let direction = match config.mode {
            PropagationMode::Inference => CycleDirection::Forward,
            PropagationMode::Training => CycleDirection::Backward,
            _ => CycleDirection::Forward,
        };
        
        // Create cycle object
        let mut obj = CycleObject::new(id.clone(), tensor, direction);
        obj.velocity = config.velocity;
        obj.target_position = config.target_position;
        
        // Add to engine
        self.engine.add_object(obj).await;
        
        // Store config
        let mut configs = self.configs.write().await;
        configs.insert(id.clone(), config);
        
        // Update position map
        let mut pos_map = self.position_map.write().await;
        pos_map.entry(node.position).or_insert_with(Vec::new).push(id.clone());
        
        Ok(id)
    }
    
    /// Remove propagating object
    pub async fn remove_object(&self, id: &str) {
        self.engine.remove_object(id).await;
        
        let mut configs = self.configs.write().await;
        configs.remove(id);
        
        let mut pos_map = self.position_map.write().await;
        for objects in pos_map.values_mut() {
            objects.retain(|obj_id| obj_id != id);
        }
    }
    
    /// Get all objects at position
    pub async fn objects_at_position(&self, position: u8) -> Vec<CycleObject> {
        self.engine.get_objects_at_position(position).await
    }
    
    /// Get interpolated 3D positions for visualization
    pub async fn get_visualization_data(&self) -> Vec<VisualizationObject> {
        let objects = self.engine.get_objects().await;
        
        objects
            .into_iter()
            .map(|obj| {
                let interp_pos = obj.position.interpolated_position();
                let angle = ((9.0 - interp_pos) / 9.0) * 2.0 * std::f64::consts::PI
                    - std::f64::consts::PI / 2.0;
                
                let radius = 8.0;
                let x = angle.cos() * radius;
                let y = angle.sin() * radius;
                
                // Z coordinate from sacred bend
                let z = obj.position.sacred_bend_factor() * 2.0;
                
                VisualizationObject {
                    id: obj.id.clone(),
                    position: (x as f32, y as f32, z as f32),
                    tensor: obj.tensor,
                    cycle_position: obj.position.position,
                    interpolation: obj.position.interpolation,
                    velocity: obj.velocity,
                }
            })
            .collect()
    }
    
    /// Apply dynamics between nodes
    /// Objects influence each other based on proximity and tensor similarity
    pub async fn apply_node_dynamics(&self) {
        let objects = self.engine.get_objects().await;
        
        // Group by position for local interactions
        let mut by_position: HashMap<u8, Vec<CycleObject>> = HashMap::new();
        for obj in objects {
            by_position
                .entry(obj.position.position)
                .or_insert_with(Vec::new)
                .push(obj);
        }
        
        // Apply dynamics within each position group
        for (_pos, group) in by_position {
            if group.len() > 1 {
                // Calculate tensor centroid
                let _avg_ethos = group.iter().map(|o| o.tensor.ethos).sum::<f64>() / group.len() as f64;
                let _avg_logos = group.iter().map(|o| o.tensor.logos).sum::<f64>() / group.len() as f64;
                let _avg_pathos = group.iter().map(|o| o.tensor.pathos).sum::<f64>() / group.len() as f64;
                
                // TODO: Apply centroid attraction to objects
                // This creates emergent clustering behavior
            }
        }
    }
    
    /// Get statistics
    pub async fn stats(&self) -> PropagationStats {
        let cycle_stats = self.engine.stats().await;
        let configs = self.configs.read().await;
        
        let inference_count = configs
            .values()
            .filter(|c| c.mode == PropagationMode::Inference)
            .count();
        let training_count = configs
            .values()
            .filter(|c| c.mode == PropagationMode::Training)
            .count();
        
        PropagationStats {
            total_objects: cycle_stats.total_objects,
            inference_mode: inference_count,
            training_mode: training_count,
            avg_velocity: cycle_stats.avg_velocity,
            avg_cycles: cycle_stats.avg_cycle_count,
        }
    }
}

/// Object data for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationObject {
    pub id: String,
    pub position: (f32, f32, f32),
    pub tensor: ELPTensor,
    pub cycle_position: u8,
    pub interpolation: f64,
    pub velocity: f64,
}

/// Propagation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagationStats {
    pub total_objects: usize,
    pub inference_mode: usize,
    pub training_mode: usize,
    pub avg_velocity: f64,
    pub avg_cycles: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{NodeAttributes, NodeState, NodeDynamics, SemanticIndex};
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_propagation_manager() {
        let manager = ObjectPropagationManager::new(60.0);
        
        let node = FluxNode {
            position: 1,
            base_value: 1,
            semantic_index: SemanticIndex {
                positive_associations: vec![],
                negative_associations: vec![],
                neutral_base: "Test".to_string(),
                predicates: vec![],
                relations: vec![],
            },
            attributes: NodeAttributes {
                properties: HashMap::new(),
                parameters: {
                    let mut params = HashMap::new();
                    params.insert("ethos".to_string(), 0.7);
                    params.insert("logos".to_string(), 0.5);
                    params.insert("pathos".to_string(), 0.9);
                    params
                },
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
        
        let config = PropagationConfig::default();
        let id = manager.add_node(&node, config).await.unwrap();
        
        assert!(!id.is_empty());
        
        let stats = manager.stats().await;
        assert_eq!(stats.total_objects, 1);
    }
}
