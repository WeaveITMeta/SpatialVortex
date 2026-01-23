//! Curated Subject Definitions
//!
//! Pre-defined FluxMatrix structures for key subjects with rich semantic associations
//!
//! ## Order of Operations (Sacred Geometry)
//!
//! Each subject follows the sacred geometry pattern:
//! - Position 0: CENTER (Neutral/Balance)
//! - Position 1: BEGINNING (Ethos - Self/Identity)
//! - Position 2: EXPANSION (Growth/Perception)
//! - Position 3: SACRED ETHOS (Unity/Integration)
//! - Position 4: POWER (Logos - Cognition/Reason)
//! - Position 5: CHANGE (Pathos - Emotion/Dynamics)
//! - Position 6: SACRED PATHOS (Emotional Core)
//! - Position 7: WISDOM (Knowledge/Understanding)
//! - Position 8: MASTERY (Peak/Excellence)
//! - Position 9: SACRED LOGOS (Divine/Ultimate)
//!
//! Vortex Flow: 1→2→4→8→7→5→1 (repeats)

pub mod consciousness;
pub mod ethics;
pub mod truth;
pub mod psychology;
pub mod cognition;
pub mod inference;
pub mod knowledge;
pub mod wisdom;
pub mod perception;
pub mod language;
pub mod reasoning;

use crate::data::models::*;
use crate::subject_definitions::consciousness::SubjectDefinitionWithSemantics;
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

/// Get subject definition by name
pub fn get_subject_definition(name: &str) -> Option<SubjectDefinitionWithSemantics> {
    match name.to_lowercase().as_str() {
        "consciousness" | "awareness" | "mind" => Some(consciousness::definition()),
        "ethics" | "morality" | "moral" | "virtue" => Some(ethics::definition()),
        "truth" | "reality" | "real" => Some(truth::definition()),
        "psychology" | "psyche" | "mental" | "behavior" => Some(psychology::definition()),
        "cognition" | "cognitive" | "thinking" => Some(cognition::definition()),
        "inference" | "inferring" | "deduction" => Some(inference::definition()),
        "knowledge" | "knowing" | "epistemology" => Some(knowledge::definition()),
        "wisdom" | "wise" | "sagacity" => Some(wisdom::definition()),
        "perception" | "perceiving" | "sensing" => Some(perception::definition()),
        "language" | "linguistic" | "communication" => Some(language::definition()),
        "reasoning" | "reason" | "logic" => Some(reasoning::definition()),
        _ => None,
    }
}

/// Get list of all available subjects
pub fn list_subjects() -> Vec<&'static str> {
    vec![
        "consciousness",
        "ethics",
        "truth",
        "psychology",
        "cognition",
        "inference",
        "knowledge",
        "wisdom",
        "perception",
        "language",
        "reasoning",
    ]
}

/// Get subjects by category
pub fn get_subjects_by_category(category: &str) -> Vec<&'static str> {
    match category.to_lowercase().as_str() {
        "foundational" => vec!["consciousness", "ethics", "truth"],
        "cognitive" => vec!["psychology", "cognition", "inference"],
        "epistemological" => vec!["knowledge", "wisdom", "perception"],
        "linguistic" => vec!["language"],
        "logical" => vec!["reasoning", "inference"],
        _ => vec![],
    }
}

/// Get related subjects for inference enrichment
pub fn get_related_subjects(subject_name: &str) -> Vec<&'static str> {
    match subject_name.to_lowercase().as_str() {
        "consciousness" => vec!["psychology", "cognition", "perception"],
        "psychology" => vec!["consciousness", "cognition", "inference"],
        "cognition" => vec!["inference", "reasoning", "knowledge", "psychology"],
        "inference" => vec!["cognition", "truth", "reasoning"],
        "knowledge" => vec!["truth", "wisdom", "cognition"],
        "wisdom" => vec!["knowledge", "ethics", "reasoning"],
        "perception" => vec!["consciousness", "cognition", "truth"],
        "language" => vec!["cognition", "knowledge", "reasoning"],
        "reasoning" => vec!["inference", "cognition", "wisdom"],
        "ethics" => vec!["wisdom", "truth", "reasoning"],
        "truth" => vec!["knowledge", "inference", "perception"],
        _ => vec![],
    }
}

/// Create FluxMatrix from curated definition with populated semantics
pub fn create_matrix_from_curated(
    subject_def: SubjectDefinitionWithSemantics,
    _engine: &crate::core::sacred_geometry::flux_matrix::FluxMatrixEngine,
) -> crate::error::Result<FluxMatrix> {
    let matrix_id = Uuid::new_v4();
    let now = Utc::now();
    let subject = subject_def.name.clone();
    
    let mut nodes = HashMap::new();
    let mut sacred_guides = HashMap::new();
    
    // Create nodes with populated semantics
    for node_def in &subject_def.nodes {
        let base_value = get_flux_value_at_position(node_def.position);
        
        // Populate positive associations
        let mut positive_associations = Vec::new();
        for (word, index, confidence) in &node_def.positive {
            let mut assoc = SemanticAssociation::new(
                word.to_string(),
                *index,
                *confidence,
            );
            
            // Calculate ELP for each association
            let elp = calculate_elp_for_position(node_def.position);
            assoc.set_attribute("ethos".to_string(), elp.ethos as f32);
            assoc.set_attribute("logos".to_string(), elp.logos as f32);
            assoc.set_attribute("pathos".to_string(), elp.pathos as f32);
            
            positive_associations.push(assoc);
        }
        
        // Populate negative associations
        let mut negative_associations = Vec::new();
        for (word, index, confidence) in &node_def.negative {
            let mut assoc = SemanticAssociation::new(
                word.to_string(),
                *index,
                *confidence,
            );
            
            // Invert ELP for negative
            let elp = calculate_elp_for_position(node_def.position);
            assoc.set_attribute("ethos".to_string(), -elp.ethos as f32);
            assoc.set_attribute("logos".to_string(), -elp.logos as f32);
            assoc.set_attribute("pathos".to_string(), -elp.pathos as f32);
            
            negative_associations.push(assoc);
        }
        
        let semantic_index = SemanticIndex {
            positive_associations,
            negative_associations,
            neutral_base: node_def.name.clone(),
            predicates: Vec::new(),
            relations: Vec::new(),
        };
        
        let node = FluxNode {
            position: node_def.position,
            base_value,
            semantic_index,
            attributes: NodeAttributes {
                properties: HashMap::new(),
                parameters: HashMap::new(),
                state: NodeState {
                    active: true,
                    last_accessed: now,
                    usage_count: 0,
                    context_stack: Vec::new(),
                },
                dynamics: NodeDynamics::default(),
            },
            connections: create_node_connections(node_def.position),
        };
        
        nodes.insert(node_def.position, node);
    }
    
    // Create sacred guides with divine properties
    for sacred_def in &subject_def.sacred_guides {
        let mut divine_properties = Vec::new();
        for (property, _confidence) in &sacred_def.divine_properties {
            divine_properties.push(property.to_string());
        }
        
        let guide = SacredGuide {
            position: sacred_def.position,
            divine_properties,
            intersection_points: create_intersection_points(sacred_def.position),
            geometric_significance: format!(
                "Sacred {} in {}: {}",
                sacred_def.position,
                subject,
                sacred_def.name
            ),
        };
        
        sacred_guides.insert(sacred_def.position, guide);
    }
    
    Ok(FluxMatrix {
        id: matrix_id,
        subject,
        nodes,
        sacred_guides,
        created_at: now,
        updated_at: now,
    })
}

/// Get flux pattern value at position
fn get_flux_value_at_position(position: u8) -> u8 {
    let pattern = [1, 2, 4, 8, 7, 5, 1, 2, 4];  // Extended pattern
    pattern.get(position as usize).copied().unwrap_or(1)
}

/// Calculate ELP tensor for position based on sacred geometry
fn calculate_elp_for_position(position: u8) -> crate::models::ELPTensor {
    use crate::models::ELPTensor;
    
    match position {
        0 => ELPTensor { ethos: 0.0, logos: 0.0, pathos: 0.0 },  // Neutral
        1 => ELPTensor { ethos: 5.0, logos: 3.0, pathos: 2.0 },  // Self (Ethos-heavy)
        2 => ELPTensor { ethos: 3.0, logos: 4.0, pathos: 3.0 },  // Perception (Balanced)
        3 => ELPTensor { ethos: 9.0, logos: 6.0, pathos: 3.0 },  // Sacred Ethos
        4 => ELPTensor { ethos: 4.0, logos: 6.0, pathos: 3.0 },  // Cognition (Logos-heavy)
        5 => ELPTensor { ethos: 3.0, logos: 3.0, pathos: 7.0 },  // Emotion (Pathos-heavy)
        6 => ELPTensor { ethos: 3.0, logos: 6.0, pathos: 9.0 },  // Sacred Pathos
        7 => ELPTensor { ethos: 5.0, logos: 7.0, pathos: 4.0 },  // Wisdom (Logos-heavy)
        8 => ELPTensor { ethos: 6.0, logos: 6.0, pathos: 5.0 },  // Mastery (Balanced high)
        9 => ELPTensor { ethos: 6.0, logos: 9.0, pathos: 6.0 },  // Sacred Logos
        _ => ELPTensor { ethos: 0.0, logos: 0.0, pathos: 0.0 },
    }
}

/// Create node connections for position
fn create_node_connections(position: u8) -> Vec<NodeConnection> {
    let vortex_flow = [1, 2, 4, 8, 7, 5];
    let mut connections = Vec::new();
    
    if let Some(idx) = vortex_flow.iter().position(|&p| p == position) {
        // Connect to next in vortex flow
        let next_idx = (idx + 1) % vortex_flow.len();
        connections.push(NodeConnection {
            target_position: vortex_flow[next_idx],
            connection_type: ConnectionType::Sequential,
            weight: 1.0,
            bidirectional: false,
        });
        
        // Connect to previous in vortex flow
        let prev_idx = if idx == 0 { vortex_flow.len() - 1 } else { idx - 1 };
        connections.push(NodeConnection {
            target_position: vortex_flow[prev_idx],
            connection_type: ConnectionType::Sequential,
            weight: 0.8,
            bidirectional: false,
        });
    }
    
    connections
}

/// Create intersection points for sacred position
fn create_intersection_points(position: u8) -> Vec<IntersectionPoint> {
    match position {
        3 => vec![
            IntersectionPoint {
                with_node: 0,  // Center
                significance: "Ethos Trinity - Character Foundation".to_string(),
                computational_value: 3.0,
            },
        ],
        6 => vec![
            IntersectionPoint {
                with_node: 0,  // Center
                significance: "Pathos Hexagon - Emotional Balance".to_string(),
                computational_value: 6.0,
            },
        ],
        9 => vec![
            IntersectionPoint {
                with_node: 0,  // Center
                significance: "Logos Completion - Rational Perfection".to_string(),
                computational_value: 9.0,
            },
        ],
        _ => Vec::new(),
    }
}
