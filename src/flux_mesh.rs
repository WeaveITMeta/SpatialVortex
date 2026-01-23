/// Flux Mesh: 3D visualization of the flux pattern structure
/// Sacred geometry with positions 0-9 mapped to vertices and edges
/// Sacred intersections (3-6-9) are highlighted with emissive materials

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};

/// Flux pattern layout matching the user's diagram:
/// ```
///         8 ←────────→ 9 ←────────→ 1
///          ╲           │           ╱
///           ╲          │          ╱
///            7 ←──→ CENTER ←──→ 2
///           ╱          │          ╲
///          ╱           │           ╲
///         6 ←────────→ 5 ←────────→ 3
///                      │
///                      4
/// ```
#[derive(Debug, Clone)]
pub struct FluxGeometry {
    pub vertices: Vec<Vec3>,
    pub edges: Vec<(usize, usize)>,
    pub sacred_positions: Vec<usize>,
}

impl FluxGeometry {
    /// Create the flux pattern geometry
    pub fn new(scale: f32) -> Self {
        // Map flux positions to 3D coordinates in diamond pattern
        // Position 0 is center, 1-9 form the flux shape
        let vertices = vec![
            // 0: Center (void/neutral)
            Vec3::new(0.0, 0.0, 0.0),
            
            // Top row (1, 9, 8)
            Vec3::new(2.0 * scale, 2.0 * scale, 0.0),   // 1 - top right
            Vec3::new(-2.0 * scale, 2.0 * scale, 0.0),  // 8 - top left  
            Vec3::new(0.0, 2.0 * scale, 0.0),           // 9 - top center (sacred)
            
            // Middle row (2, 7)
            Vec3::new(3.0 * scale, 0.0, 0.0),           // 2 - right
            Vec3::new(-3.0 * scale, 0.0, 0.0),          // 7 - left
            
            // Bottom row (3, 4, 5, 6)
            Vec3::new(2.0 * scale, -2.0 * scale, 0.0),  // 3 - bottom right (sacred)
            Vec3::new(0.0, -3.0 * scale, 0.0),          // 4 - bottom
            Vec3::new(-1.0 * scale, -2.0 * scale, 0.0), // 5 - bottom left-center
            Vec3::new(-2.0 * scale, -2.0 * scale, 0.0), // 6 - bottom left (sacred)
        ];
        
        // Define edges following the flux pattern and sacred connections
        // Gray edges: Regular flux flow (1→2→4→8→7→5→1)
        // Cyan edges: Sacred connections (3-6-9)
        let edges = vec![
            // Top flux connections
            (1, 3),  // 1 to 9 (sacred)
            (3, 2),  // 9 to 8
            (2, 1),  // 8 to 1
            
            // Diagonal connections
            (1, 4),  // 1 to 2
            (2, 5),  // 8 to 7
            (4, 0),  // 2 to center
            (5, 0),  // 7 to center
            
            // Bottom flux connections
            (6, 8),  // 3 to 5
            (8, 7),  // 5 to 4
            (9, 8),  // 6 to 5
            (9, 6),  // 6 to 3
            
            // Sacred triangle (3-6-9) - cyan connections
            (6, 3),  // 3 to 9 (vertical)
            (9, 3),  // 6 to 9 (diagonal)
            (6, 9),  // 3 to 6 (horizontal)
            
            // Center connections
            (0, 3),  // Center to 9
            (0, 7),  // Center to 4
        ];
        
        // Sacred positions that act as processing intersections
        let sacred_positions = vec![3, 6, 9];  // Indices for positions 3, 6, 9
        
        Self {
            vertices,
            edges,
            sacred_positions,
        }
    }
    
    /// Convert to Bevy mesh for rendering
    pub fn to_mesh(&self) -> Mesh {
        // Create a mesh that represents the flux structure
        // Using lines to show the edges clearly
        let mut mesh = Mesh::new(PrimitiveTopology::LineList);
        
        // Convert Vec3 to array format for mesh
        let positions: Vec<[f32; 3]> = self.vertices
            .iter()
            .map(|v| [v.x, v.y, v.z])
            .collect();
        
        // Create indices for line segments
        let mut indices = Vec::new();
        for (start, end) in &self.edges {
            indices.push(*start as u32);
            indices.push(*end as u32);
        }
        
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.set_indices(Some(Indices::U32(indices)));
        
        mesh
    }
    
    /// Create sphere mesh for flux nodes
    pub fn create_node_sphere(radius: f32, resolution: u32) -> Mesh {
        Sphere::new(radius).mesh().ico(resolution as usize).unwrap()
    }
    
    /// Get world position for a flux position (0-9)
    pub fn get_position(&self, flux_position: u8) -> Vec3 {
        // Map flux position to vertex index
        let index = self.flux_to_vertex_index(flux_position);
        self.vertices[index]
    }
    
    /// Map flux position (0-9) to vertex index in our geometry
    fn flux_to_vertex_index(&self, flux_position: u8) -> usize {
        match flux_position {
            0 => 0,  // Center
            1 => 1,  // Top right
            2 => 4,  // Right
            3 => 6,  // Bottom right (sacred)
            4 => 7,  // Bottom
            5 => 8,  // Bottom left-center
            6 => 9,  // Bottom left (sacred)
            7 => 5,  // Left
            8 => 2,  // Top left
            9 => 3,  // Top center (sacred)
            _ => 0,  // Default to center
        }
    }
    
    /// Check if a position is sacred (3, 6, or 9)
    pub fn is_sacred_position(&self, flux_position: u8) -> bool {
        matches!(flux_position, 3 | 6 | 9)
    }
    
    /// Get color for a flux position based on its properties
    pub fn get_position_color(&self, flux_position: u8) -> Color {
        match flux_position {
            3 => Color::srgb(0.0, 1.0, 0.5),   // Green (Good/Easy)
            6 => Color::srgb(1.0, 0.2, 0.2),   // Red (Bad/Hard)
            9 => Color::srgb(0.2, 0.5, 1.0),   // Blue (Divine/Righteous)
            0 => Color::srgb(0.5, 0.5, 0.5),   // Gray (Neutral center)
            _ => Color::srgb(0.8, 0.8, 0.8),   // Light gray (Regular positions)
        }
    }
    
    /// Calculate path for word beam flowing through flux pattern
    pub fn calculate_beam_path(
        &self,
        start_position: u8,
        end_position: u8,
        curvature: f32,
    ) -> Vec<Vec3> {
        let start = self.get_position(start_position);
        let end = self.get_position(end_position);
        
        // Number of segments for smooth curve
        const SEGMENTS: usize = 20;
        let mut path = Vec::with_capacity(SEGMENTS);
        
        // Calculate control point for quadratic bezier curve
        let midpoint = (start + end) / 2.0;
        let perpendicular = Vec3::new(
            -(end.y - start.y),
            end.x - start.x,
            0.0,
        ).normalize_or_zero();
        
        // Control point offset by curvature
        let control = midpoint + perpendicular * curvature;
        
        // Generate bezier curve points
        for i in 0..SEGMENTS {
            let t = i as f32 / (SEGMENTS - 1) as f32;
            let t_inv = 1.0 - t;
            
            // Quadratic bezier: P(t) = (1-t)²P₀ + 2(1-t)tP₁ + t²P₂
            let point = start * (t_inv * t_inv) +
                       control * (2.0 * t_inv * t) +
                       end * (t * t);
            
            path.push(point);
        }
        
        path
    }
}

/// Component for nodes in the diamond structure
#[derive(Component, Debug, Clone)]
pub struct FluxNode {
    pub position: u8,      // Flux position (0-9)
    pub is_sacred: bool,   // Is this a sacred intersection (3, 6, 9)
    pub activity: f32,     // Current activity level (0.0 to 1.0)
}

/// Component for word beams flowing through the flux matrix
#[derive(Component, Debug, Clone)]
pub struct WordBeam {
    pub word: String,
    pub current_position: u8,
    pub target_position: u8,
    pub progress: f32,        // 0.0 to 1.0 along path
    pub color: Color,          // RGB from ELP channels
    pub intensity: f32,        // Brightness/confidence
    pub path: Vec<Vec3>,       // Calculated path points
}

/// System to spawn the flux structure
pub fn spawn_flux_structure(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let geometry = FluxGeometry::new(1.0);
    
    // Spawn flux wireframe
    let flux_mesh = meshes.add(geometry.to_mesh());
    commands
        .spawn(PbrBundle {
            mesh: flux_mesh,
            material: materials.add(StandardMaterial {
                base_color: Color::srgb(0.7, 0.7, 0.7),
                emissive: LinearRgba::new(0.1, 0.1, 0.1, 1.0),
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        });
    
    // Spawn nodes at each flux position
    for position in 0..=9u8 {
        let world_pos = geometry.get_position(position);
        let is_sacred = geometry.is_sacred_position(position);
        let node_color = geometry.get_position_color(position);
        
        // Node size based on importance
        let radius = if is_sacred { 0.3 } else if position == 0 { 0.25 } else { 0.15 };
        
        let node_mesh = meshes.add(FluxGeometry::create_node_sphere(radius, 16));
        
        // Emissive material for sacred nodes
        let material = if is_sacred {
            materials.add(StandardMaterial {
                base_color: node_color,
                emissive: node_color * 2.0,
                ..default()
            })
        } else {
            materials.add(StandardMaterial {
                base_color: node_color,
                ..default()
            })
        };
        
        commands
            .spawn(PbrBundle {
                mesh: node_mesh,
                material,
                transform: Transform::from_translation(world_pos),
                ..default()
            })
            .insert(FluxNode {
                position,
                is_sacred,
                activity: 0.0,
            });
    }
}

/// System to animate sacred nodes
pub fn animate_sacred_nodes(
    time: Res<Time>,
    mut query: Query<(&FluxNode, &mut Transform), With<FluxNode>>,
) {
    for (node, mut transform) in query.iter_mut() {
        if node.is_sacred {
            // Pulsing effect for sacred nodes
            let pulse = (time.seconds_since_startup() as f32 * 2.0).sin() * 0.5 + 0.5;
            transform.scale = Vec3::splat(1.0 + pulse * 0.2);
            
            // Rotation for divine node (9)
            if node.position == 9 {
                transform.rotate_y(time.delta_seconds() * 0.5);
            }
        }
        
        // Activity-based scaling
        if node.activity > 0.0 {
            let scale = 1.0 + node.activity * 0.5;
            transform.scale = Vec3::splat(scale);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_flux_geometry_creation() {
        let geometry = FluxGeometry::new(1.0);
        assert_eq!(geometry.vertices.len(), 10); // 0-9 positions
        assert!(!geometry.edges.is_empty());
        assert_eq!(geometry.sacred_positions.len(), 3);
    }
    
    #[test]
    fn test_flux_position_mapping() {
        let geometry = FluxGeometry::new(1.0);
        
        // Test center position
        let center = geometry.get_position(0);
        assert_eq!(center, Vec3::ZERO);
        
        // Test sacred positions exist
        for sacred in &[3, 6, 9] {
            assert!(geometry.is_sacred_position(*sacred));
            let _ = geometry.get_position(*sacred); // Should not panic
        }
    }
    
    #[test]
    fn test_beam_path_calculation() {
        let geometry = FluxGeometry::new(1.0);
        
        // Test path from 1 to 5 with curvature
        let path = geometry.calculate_beam_path(1, 5, 0.5);
        assert!(!path.is_empty());
        assert_eq!(path.len(), 20); // Should have 20 segments
        
        // First point should be at position 1
        let start = geometry.get_position(1);
        assert!((path[0] - start).length() < 0.01);
    }
}
