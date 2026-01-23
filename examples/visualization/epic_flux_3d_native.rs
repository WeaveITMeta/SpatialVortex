//! Epic Flux 3D - Native Bevy Application
//! 
//! Run with: cargo run --example epic_flux_3d_native --features bevy_support

use bevy::prelude::*;
use spatial_vortex::lock_free_flux::LockFreeFluxMatrix;
use spatial_vortex::models::{FluxNode as FluxNodeModel, NodeAttributes, NodeState, NodeDynamics, SemanticIndex, ELPTensor};
use std::collections::HashMap;
use std::f32::consts::PI;

// Component to track flux nodes in 3D
#[derive(Component)]
struct FluxNode {
    position: u8,
    is_sacred: bool,
}

// Component to track camera orbit state
#[derive(Component)]
struct OrbitCamera {
    radius: f32,
    angle: f32,
    height: f32,
}

fn main() {
    println!("🌀 Starting Epic Flux 3D - Native Edition");
    
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "🌀 Epic Flux Matrix 3D - Native".to_string(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup_epic_scene)
        .add_systems(Update, (animate_nodes, orbit_camera))
        .run();
}

fn setup_epic_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    println!("Setting up Epic 3D scene...");
    
    // Create flux matrix
    let matrix = LockFreeFluxMatrix::new(42).expect("Failed to create matrix");
    
    // Camera with orbit controller
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 8.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
        OrbitCamera {
            radius: 17.0,
            angle: 0.0,
            height: 8.0,
        },
    ));
    
    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.4, 0.4, 0.5),
        brightness: 400.0,
    });
    
    // Point light
    commands.spawn((
        PointLight {
            intensity: 3_000_000.0,
            color: Color::srgb(1.0, 0.95, 0.9),
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(10.0, 15.0, 10.0),
    ));
    
    // Render flux matrix nodes
    let radius = 5.0;
    let node_radius = 0.5;
    
    for pos in 0..10 {
        let node = matrix.get_node(pos).expect("Node not found");
        let is_sacred = matches!(pos, 3 | 6 | 9);
        
        // Position on circle
        let angle = (pos as f32) * (2.0 * PI / 10.0) - (PI / 2.0);
        let position_3d = Vec3::new(
            radius * angle.cos(),
            0.0,
            radius * angle.sin(),
        );
        
        // Node sphere color
        let node_color = if is_sacred {
            Color::srgb(0.0, 0.9, 1.0) // Sacred: bright cyan
        } else if pos == 0 {
            Color::srgb(0.9, 0.9, 0.2) // Position 0: yellow/gold
        } else {
            Color::srgb(0.85, 0.85, 0.85) // Others: light gray
        };
        
        // Spawn node sphere
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(node_radius).mesh().ico(5).unwrap())),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: node_color,
                unlit: true,
                ..default()
            })),
            Transform::from_translation(position_3d),
            FluxNode {
                position: pos,
                is_sacred,
            },
        ));
        
        // Labels: number above the node, name centered on the node
        let name_text = node.node.semantic_index.neutral_base.clone();
        let num_text = format!("{}", pos);
        
        // Static number label (above node) - billboarded automatically
        commands.spawn((
            Text2d::new(num_text),
            TextFont { 
                font_size: 24.0,
                ..default() 
            },
            TextColor(Color::srgb(1.0, 1.0, 1.0)),
            Transform::from_translation(position_3d + Vec3::new(0.0, 1.2, 0.0)),
        ));
        
        // Static name label (centered in node) - sacred nodes get cyan
        let label_color = if is_sacred { 
            Color::srgb(0.0, 0.95, 1.0) 
        } else { 
            Color::srgb(0.9, 0.9, 0.9) 
        };
        commands.spawn((
            Text2d::new(name_text),
            TextFont { 
                font_size: 18.0,
                ..default() 
            },
            TextColor(label_color),
            Transform::from_translation(position_3d + Vec3::new(0.0, -0.2, 0.0)),
        ));
    }
    
    // Draw sacred triangle (3-6-9)
    draw_triangle(&mut commands, &mut meshes, &mut materials, radius);
    
    // Draw flow sequence lines (1→2→4→8→7→5→1)
    draw_flow_lines(&mut commands, &mut meshes, &mut materials, radius);
    
    println!("✅ Scene setup complete!");
}

fn draw_triangle(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    radius: f32,
) {
    let sacred = [3, 6, 9];
    for i in 0..3 {
        let from_pos = sacred[i];
        let to_pos = sacred[(i + 1) % 3];
        
        let from_angle = (from_pos as f32) * (2.0 * PI / 10.0) - (PI / 2.0);
        let to_angle = (to_pos as f32) * (2.0 * PI / 10.0) - (PI / 2.0);
        
        let from = Vec3::new(radius * from_angle.cos(), 0.0, radius * from_angle.sin());
        let to = Vec3::new(radius * to_angle.cos(), 0.0, radius * to_angle.sin());
        
        draw_line(commands, meshes, materials, from, to, true);
    }
}

fn draw_flow_lines(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    radius: f32,
) {
    let flow = [1, 2, 4, 8, 7, 5, 1];
    for i in 0..(flow.len() - 1) {
        let from_pos = flow[i];
        let to_pos = flow[i + 1];
        
        let from_angle = (from_pos as f32) * (2.0 * PI / 10.0) - (PI / 2.0);
        let to_angle = (to_pos as f32) * (2.0 * PI / 10.0) - (PI / 2.0);
        
        let from = Vec3::new(radius * from_angle.cos(), 0.0, radius * from_angle.sin());
        let to = Vec3::new(radius * to_angle.cos(), 0.0, radius * to_angle.sin());
        
        draw_line(commands, meshes, materials, from, to, false);
    }
}

fn draw_line(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    from: Vec3,
    to: Vec3,
    is_sacred: bool,
) {
    let direction = to - from;
    let length = direction.length();
    let midpoint = (from + to) / 2.0;
    let rotation = Quat::from_rotation_arc(Vec3::Y, direction.normalize());
    
    let (color, radius) = if is_sacred {
        (Color::srgb(0.0, 0.9, 1.0), 0.12)
    } else {
        (Color::srgb(0.4, 0.4, 0.4), 0.08)
    };
    
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(radius, length))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: color,
            unlit: true,
            ..default()
        })),
        Transform {
            translation: midpoint,
            rotation,
            ..default()
        },
    ));
}

// Animate nodes pulsing
fn animate_nodes(
    time: Res<Time>,
    mut query: Query<(&FluxNode, &mut Transform)>,
) {
    for (node, mut transform) in query.iter_mut() {
        let pulse = (time.elapsed_secs() * 2.0 + node.position as f32 * 0.5).sin() * 0.05 + 1.0;
        transform.scale = Vec3::splat(pulse);
    }
}

// Camera orbit control
fn orbit_camera(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut OrbitCamera)>,
) {
    for (mut transform, mut orbit) in query.iter_mut() {
        orbit.angle += time.delta_secs() * 0.2; // Slow orbit
        
        let x = orbit.radius * orbit.angle.cos();
        let z = orbit.radius * orbit.angle.sin();
        
        transform.translation = Vec3::new(x, orbit.height, z);
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}
