//! Bevy 3D Shape-Based Visualization Components
//!
//! Shape Architecture:
//! - Box (Cuboid): Processing blocks with text labels
//! - Cylinder: Database connections with real-time access
//! - Sphere: Node references with metadata

use bevy::prelude::*;
use crate::models::ELPTensor;
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};

/// Processing block component (visualized as Box/Cuboid)
#[derive(Component, Debug, Clone)]
pub struct ProcessingBlock {
    pub id: String,
    pub label: String,
    pub block_type: BlockType,
    pub state: ProcessingState,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BlockType {
    Transformation,
    Inference,
    Aggregation,
    Routing,
    Validation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProcessingState {
    Idle,
    Processing,
    Complete,
    Error(String),
}

impl ProcessingState {
    pub fn color(&self) -> Color {
        match self {
            ProcessingState::Idle => Color::srgb(0.5, 0.5, 0.5),
            ProcessingState::Processing => Color::srgb(0.9, 0.9, 0.2),
            ProcessingState::Complete => Color::srgb(0.2, 0.9, 0.2),
            ProcessingState::Error(_) => Color::srgb(0.9, 0.2, 0.2),
        }
    }
}

/// Database node component (visualized as Cylinder)
#[derive(Component, Debug, Clone)]
pub struct DatabaseNode {
    pub id: String,
    pub db_type: DatabaseType,
    pub status: ConnectionStatus,
    pub reference_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DatabaseType {
    PostgreSQL,
    Redis,
    MongoDB,
    Neo4j,
    SpatialDB,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Connected,
    Connecting,
    Disconnected,
    Error(String),
}

impl ConnectionStatus {
    pub fn color(&self) -> Color {
        match self {
            ConnectionStatus::Connected => Color::srgb(0.2, 0.4, 0.8),
            ConnectionStatus::Connecting => Color::srgb(0.8, 0.8, 0.2),
            ConnectionStatus::Disconnected => Color::srgb(0.5, 0.5, 0.5),
            ConnectionStatus::Error(_) => Color::srgb(0.8, 0.2, 0.2),
        }
    }
}

/// Node reference component (visualized as Sphere)
#[derive(Component, Debug, Clone)]
pub struct NodeReference {
    pub id: String,
    pub metadata: NodeMetadata,
    pub ref_type: ReferenceType,
}

#[derive(Debug, Clone)]
pub struct NodeMetadata {
    pub name: String,
    pub position: u8,
    pub elp: ELPTensor,
    pub access_count: u64,
    pub tags: Vec<String>,
}

impl NodeMetadata {
    pub fn elp_color(&self) -> Color {
        Color::srgb(
            self.elp.ethos as f32,
            self.elp.logos as f32,
            self.elp.pathos as f32,
        )
    }
    
    pub fn size_from_access(&self) -> f32 {
        0.3 + (self.access_count as f32 + 1.0).log10() * 0.1
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReferenceType {
    FluxNode,
    ProcessingBlock,
    DatabaseEntry,
    ExternalAPI,
    UserDefined,
}

/// Connection between entities
#[derive(Component, Debug, Clone)]
pub struct Connection {
    pub from: Entity,
    pub to: Entity,
    pub connection_type: ConnectionType,
    pub bandwidth: f32,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionType {
    DataFlow,
    Reference,
    Dependency,
    Similarity,
}

impl ConnectionType {
    pub fn color(&self) -> Color {
        match self {
            ConnectionType::DataFlow => Color::srgb(0.2, 0.9, 0.2),
            ConnectionType::Reference => Color::srgb(0.2, 0.4, 0.9),
            ConnectionType::Dependency => Color::srgb(0.9, 0.9, 0.2),
            ConnectionType::Similarity => Color::srgb(0.8, 0.2, 0.9),
        }
    }
}

/// Spawn a processing block (Box shape)
pub fn spawn_processing_block(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    block: ProcessingBlock,
    position: Vec3,
) -> Entity {
    let color = block.state.color();
    
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(2.0, 1.0, 1.0)),
            material: materials.add(StandardMaterial {
                base_color: color,
                metallic: 0.3,
                perceptual_roughness: 0.5,
                ..default()
            }),
            transform: Transform::from_translation(position),
            ..default()
        },
        block,
        Name::new("ProcessingBlock"),
    ))
    .id()
}

/// Spawn a database node (Cylinder shape)
pub fn spawn_database_node(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    db: DatabaseNode,
    position: Vec3,
) -> Entity {
    let color = db.status.color();
    
    // Height based on reference count
    let height = 2.0 + (db.reference_count as f32 * 0.1).min(3.0);
    
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cylinder::new(0.5, height)),
            material: materials.add(StandardMaterial {
                base_color: color,
                metallic: 0.5,
                perceptual_roughness: 0.3,
                emissive: color * 0.2,
                ..default()
            }),
            transform: Transform::from_translation(position),
            ..default()
        },
        db,
        Name::new("DatabaseNode"),
    ))
    .id()
}

/// Spawn a node reference (Sphere shape)
pub fn spawn_node_reference(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    node: NodeReference,
    position: Vec3,
) -> Entity {
    let color = node.metadata.elp_color();
    let radius = node.metadata.size_from_access();
    
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Sphere::new(radius)),
            material: materials.add(StandardMaterial {
                base_color: color,
                metallic: 0.2,
                perceptual_roughness: 0.6,
                emissive: color * 0.5,
                ..default()
            }),
            transform: Transform::from_translation(position),
            ..default()
        },
        node,
        Name::new("NodeReference"),
    ))
    .id()
}

/// System to update processing block colors based on state
pub fn update_processing_blocks(
    mut blocks: Query<(&ProcessingBlock, &Handle<StandardMaterial>), Changed<ProcessingBlock>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (block, material_handle) in blocks.iter_mut() {
        if let Some(material) = materials.get_mut(material_handle) {
            material.base_color = block.state.color();
        }
    }
}

/// System to update database node colors based on status
pub fn update_database_nodes(
    mut nodes: Query<(&DatabaseNode, &Handle<StandardMaterial>), Changed<DatabaseNode>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (node, material_handle) in nodes.iter_mut() {
        if let Some(material) = materials.get_mut(material_handle) {
            material.base_color = node.status.color();
            material.emissive = node.status.color() * 0.2;
        }
    }
}

/// System to update node reference visuals based on metadata changes
pub fn update_node_references(
    mut nodes: Query<(&NodeReference, &Handle<StandardMaterial>, &mut Transform), Changed<NodeReference>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (node, material_handle, mut transform) in nodes.iter_mut() {
        if let Some(material) = materials.get_mut(material_handle) {
            let color = node.metadata.elp_color();
            material.base_color = color;
            material.emissive = color * 0.5;
        }
        
        // Update size based on access count
        let new_scale = node.metadata.size_from_access();
        transform.scale = Vec3::splat(new_scale);
    }
}

/// System to draw connections between entities
pub fn draw_connections(
    mut gizmos: Gizmos,
    connections: Query<&Connection>,
    transforms: Query<&Transform>,
) {
    for conn in connections.iter() {
        if !conn.active {
            continue;
        }
        
        if let (Ok(from_transform), Ok(to_transform)) = 
            (transforms.get(conn.from), transforms.get(conn.to)) 
        {
            let from_pos = from_transform.translation;
            let to_pos = to_transform.translation;
            let color = conn.connection_type.color();
            
            // Draw line with thickness based on bandwidth
            gizmos.line(from_pos, to_pos, color);
            
            // Draw direction arrow
            let direction = (to_pos - from_pos).normalize();
            let arrow_pos = from_pos.lerp(to_pos, 0.8);
            let arrow_size = 0.2;
            
            gizmos.arrow(arrow_pos, arrow_pos + direction * arrow_size, color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_processing_state_colors() {
        assert_eq!(ProcessingState::Idle.color(), Color::srgb(0.5, 0.5, 0.5));
        assert_eq!(ProcessingState::Complete.color(), Color::srgb(0.2, 0.9, 0.2));
    }
    
    #[test]
    fn test_connection_status_colors() {
        assert_eq!(ConnectionStatus::Connected.color(), Color::srgb(0.2, 0.4, 0.8));
        assert_eq!(ConnectionStatus::Disconnected.color(), Color::srgb(0.5, 0.5, 0.5));
    }
    
    #[test]
    fn test_node_metadata_size() {
        let metadata = NodeMetadata {
            name: "Test".to_string(),
            position: 5,
            elp: ELPTensor::new(0.5, 0.5, 0.5),
            access_count: 100,
            tags: vec![],
        };
        
        let size = metadata.size_from_access();
        assert!(size > 0.3);
        assert!(size < 1.0);
    }
}
