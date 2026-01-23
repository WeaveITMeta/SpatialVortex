/// 3D Flux Matrix Visualization using Bevy
/// 
/// Vortex Math Pattern in 3D:
/// - Position 9 at top (12 o'clock)
/// - Clockwise arrangement: 1, 2, 3, 4, 5, 6, 7, 8
/// - Sacred 3-6-9 triangle emphasized
/// - Internal star pattern connections
/// - WASM-compatible for web deployment
/// - Interactive camera controls

#[cfg(feature = "bevy_support")]
use bevy::{
    prelude::*,
    render::mesh::{Mesh, PrimitiveTopology},
    asset::Assets,
};
use crate::visualization::{FluxVisualization, Point2D};
use std::f32::consts::PI;

/// Component marking a flux position sphere
#[cfg(feature = "bevy_support")]
#[derive(Component)]
pub struct FluxPositionMarker {
    pub position: u8,
    pub is_sacred: bool,
}

/// Component marking a data point
#[cfg(feature = "bevy_support")]
#[derive(Component)]
pub struct FluxDataMarker {
    pub id: String,
    pub position: u8,
}

/// Component for rotating camera
#[cfg(feature = "bevy_support")]
#[derive(Component)]
pub struct OrbitCamera {
    pub radius: f32,
    pub angle: f32,
    pub height: f32,
}

/// Resource holding visualization data
#[cfg(feature = "bevy_support")]
#[derive(Resource)]
pub struct FluxVisualizationData {
    pub viz: FluxVisualization,
}

/// Convert 2D layout to 3D coordinates
pub fn layout_to_3d(point: &Point2D, height: f32) -> Vec3 {
    Vec3::new(point.x as f32 * 3.0, height, point.y as f32 * 3.0)
}

/// Create a sphere mesh (replaces deprecated shape::UVSphere)
#[cfg(feature = "bevy_support")]
pub fn create_sphere_mesh(radius: f32, sectors: usize, stacks: usize) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    for i in 0..=stacks {
        let phi = PI * i as f32 / stacks as f32;
        let sin_phi = phi.sin();
        let cos_phi = phi.cos();

        for j in 0..=sectors {
            let theta = 2.0 * PI * j as f32 / sectors as f32;
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();

            let x = sin_phi * cos_theta;
            let y = cos_phi;
            let z = sin_phi * sin_theta;

            positions.push([x * radius, y * radius, z * radius]);
            normals.push([x, y, z]);
            uvs.push([j as f32 / sectors as f32, i as f32 / stacks as f32]);
        }
    }

    for i in 0..stacks {
        for j in 0..sectors {
            let first = (i * (sectors + 1) + j) as u32;
            let second = first + sectors as u32 + 1;

            indices.push(first);
            indices.push(second);
            indices.push(first + 1);

            indices.push(second);
            indices.push(second + 1);
            indices.push(first + 1);
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));
    mesh
}

/// Create a cylinder mesh (replaces deprecated shape::Cylinder)
#[cfg(feature = "bevy_support")]
pub fn create_cylinder_mesh(radius: f32, height: f32, resolution: usize) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    let half_height = height / 2.0;

    // Top and bottom circles
    for i in 0..=resolution {
        let theta = 2.0 * PI * i as f32 / resolution as f32;
        let x = radius * theta.cos();
        let z = radius * theta.sin();

        // Top
        positions.push([x, half_height, z]);
        normals.push([x / radius, 0.0, z / radius]);
        uvs.push([i as f32 / resolution as f32, 1.0]);

        // Bottom
        positions.push([x, -half_height, z]);
        normals.push([x / radius, 0.0, z / radius]);
        uvs.push([i as f32 / resolution as f32, 0.0]);
    }

    // Side faces
    for i in 0..resolution {
        let base = (i * 2) as u32;
        indices.push(base);
        indices.push(base + 2);
        indices.push(base + 1);

        indices.push(base + 1);
        indices.push(base + 2);
        indices.push(base + 3);
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));
    mesh
}

/// Setup 3D flux visualization with Vortex Math pattern
#[cfg(feature = "bevy_support")]
pub fn setup_flux_3d(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    viz_data: Res<FluxVisualizationData>,
) {
    // Add camera with orbit controls
    commands.spawn()
        .insert_bundle(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 8.0, 12.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(OrbitCamera {
            radius: 15.0,
            angle: 0.0,
            height: 8.0,
        });

    // Add directional light
    commands.spawn()
        .insert_bundle(DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: 10000.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(4.0, 8.0, 4.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        });

    // Add ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.3,
    });

    let viz = &viz_data.viz;
    
    // Create position markers (0-9) as spheres
    for (pos, coords) in &viz.layout.positions {
        let pos_3d = layout_to_3d(coords, 0.0);
        let is_sacred = [3, 6, 9].contains(pos);
        
        // Position sphere
        let color = if is_sacred {
            Color::rgb(1.0, 0.0, 0.0) // Red for sacred
        } else {
            Color::rgb(0.3, 0.3, 0.8) // Blue for normal
        };
        
        commands.spawn()
            .insert_bundle(PbrBundle {
                mesh: meshes.add(create_sphere_mesh(0.3, 32, 16)),
                material: materials.add(StandardMaterial {
                    base_color: color,
                    emissive: color * 0.5,
                    ..default()
                }),
                transform: Transform::from_translation(pos_3d),
                ..default()
            })
            .insert(FluxPositionMarker {
                position: *pos,
                is_sacred,
            });
        
        // Position label (as small sphere above)
        commands.spawn()
            .insert_bundle(PbrBundle {
                mesh: meshes.add(create_sphere_mesh(0.1, 8, 4)),
                material: materials.add(StandardMaterial {
                    base_color: Color::WHITE,
                    emissive: Color::WHITE,
                    ..default()
                }),
                transform: Transform::from_translation(pos_3d + Vec3::Y * 0.5),
                ..default()
            });
    }
    
    // Draw sacred triangle
    let sacred_positions = [3u8, 6, 9];
    for i in 0..3 {
        let from_pos = sacred_positions[i];
        let to_pos = sacred_positions[(i + 1) % 3];
        
        if let (Some(from_coords), Some(to_coords)) = (
            viz.layout.positions.get(&from_pos),
            viz.layout.positions.get(&to_pos),
        ) {
            let from_3d = layout_to_3d(from_coords, 0.0);
            let to_3d = layout_to_3d(to_coords, 0.0);
            
            draw_line(
                &mut commands,
                &mut meshes,
                &mut materials,
                from_3d,
                to_3d,
                Color::rgb(1.0, 0.0, 0.0),
                0.05,
            );
        }
    }
    
    // Draw flow lines between adjacent positions
    for flow_line in &viz.flow_lines {
        let from_3d = layout_to_3d(&flow_line.from_coords, 0.0);
        let to_3d = layout_to_3d(&flow_line.to_coords, 0.0);
        
        let color = if flow_line.is_sacred {
            Color::rgb(1.0, 0.3, 0.3)
        } else {
            Color::rgb(0.2, 0.2, 0.2)
        };
        
        draw_line(
            &mut commands,
            &mut meshes,
            &mut materials,
            from_3d,
            to_3d,
            color,
            0.02,
        );
    }
    
    // Create data points
    for point in &viz.data_points {
        let pos_3d = layout_to_3d(&point.coords, 0.5);
        
        // Color by dominant ELP channel
        let color = match point.dominant_channel() {
            "Ethos" => Color::rgb(1.0, 0.0, 0.0),   // Red
            "Logos" => Color::rgb(0.0, 0.0, 1.0),   // Blue
            "Pathos" => Color::rgb(0.0, 1.0, 0.0),  // Green
            _ => Color::rgb(0.5, 0.5, 0.5),
        };
        
        // Size by tensor magnitude
        let size = (point.tensor_magnitude() * 0.3) as f32;
        
        commands.spawn()
            .insert_bundle(PbrBundle {
                mesh: meshes.add(create_sphere_mesh(size, 16, 8)),
                material: materials.add(StandardMaterial {
                    base_color: color,
                    emissive: color * 0.8,
                    ..default()
                }),
                transform: Transform::from_translation(pos_3d),
                ..default()
            })
            .insert(FluxDataMarker {
                id: point.id.clone(),
                position: point.position,
            });
        
        // Add halo effect for sacred positions
        if point.is_sacred {
            commands.spawn()
                .insert_bundle(PbrBundle {
                    mesh: meshes.add(create_sphere_mesh(size * 1.5, 16, 8)),
                    material: materials.add(StandardMaterial {
                        base_color: Color::rgba(1.0, 1.0, 0.0, 0.3),
                        alpha_mode: AlphaMode::Blend,
                        ..default()
                    }),
                    transform: Transform::from_translation(pos_3d),
                    ..default()
                });
        }
    }
}

/// Draw a line between two points using a thin cylinder
#[cfg(feature = "bevy_support")]
fn draw_line(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    from: Vec3,
    to: Vec3,
    color: Color,
    thickness: f32,
) {
    let direction = to - from;
    let length = direction.length();
    let midpoint = (from + to) / 2.0;
    
    // Calculate rotation to align cylinder with line
    let up = Vec3::Y;
    let rotation = Quat::from_rotation_arc(up, direction.normalize());
    
    commands.spawn()
        .insert_bundle(PbrBundle {
            mesh: meshes.add(create_cylinder_mesh(thickness, length, 8)),
            material: materials.add(StandardMaterial {
                base_color: color,
                emissive: color * 0.5,
                ..default()
            }),
            transform: Transform {
                translation: midpoint,
                rotation,
                ..default()
            },
            ..default()
        });
}

/// Update orbit camera
#[cfg(feature = "bevy_support")]
pub fn update_orbit_camera(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut OrbitCamera)>,
) {
    for (mut transform, mut orbit) in query.iter_mut() {
        // Auto-rotate
        orbit.angle += time.delta_seconds() * 0.3;
        
        // Calculate new position
        let x = orbit.radius * orbit.angle.cos();
        let z = orbit.radius * orbit.angle.sin();
        
        transform.translation = Vec3::new(x, orbit.height, z);
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}

/// Bevy plugin for flux 3D visualization
#[cfg(feature = "bevy_support")]
pub struct Flux3DPlugin;

#[cfg(feature = "bevy_support")]
impl Plugin for Flux3DPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_flux_3d)
            .add_system(update_orbit_camera);
    }
}
