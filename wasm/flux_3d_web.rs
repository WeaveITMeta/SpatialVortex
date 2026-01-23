//! WASM entry point for 3D Flux Visualization  
//! 
//! Pure Rust WebGL renderer - renders sacred geometry in 3D
//! Renders exact geometry from 2D flux matrix images
//! Usage: wasm-pack build --target web

#![cfg(target_arch = "wasm32")]

use spatial_vortex::{
    lock_free_flux::LockFreeFluxMatrix,
    models::{FluxNode, NodeAttributes, NodeState, NodeDynamics, SemanticIndex, ELPTensor},
};

#[derive(Resource)]
struct FluxMatrixResource(LockFreeFluxMatrix);
use bevy::prelude::*;
use std::collections::HashMap;
use std::f32::consts::PI;
use wasm_bindgen::prelude::*;

fn create_demo_node(name: &str, position: u8, ethos: f64, logos: f64, pathos: f64) -> FluxNode {
    let mut parameters = HashMap::new();
    parameters.insert("ethos".to_string(), ethos);
    parameters.insert("logos".to_string(), logos);
    parameters.insert("pathos".to_string(), pathos);
    
    FluxNode {
        position,
        base_value: position,
        semantic_index: SemanticIndex {
            positive_associations: vec![],
            negative_associations: vec![],
            neutral_base: name.to_string(),
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
            dynamics: NodeDynamics {
                evolution_rate: 1.0,
                stability_index: 1.0,
                interaction_patterns: vec![],
                learning_adjustments: vec![],
            },
        },
        connections: vec![],
    }
}

#[wasm_bindgen(start)]
pub fn run() {
    // Set panic hook for better error messages
    console_error_panic_hook::set_once();
    
    // Create flux matrix with demo data
    let matrix = LockFreeFluxMatrix::new("3D_Web_Demo".to_string());
    
    let demo_data = vec![
        ("Love", 3, 0.7, 0.5, 0.95),
        ("Truth", 6, 0.85, 0.95, 0.5),
        ("Creation", 9, 0.9, 0.6, 0.5),
        ("Joy", 1, 0.6, 0.4, 0.9),
        ("Wisdom", 8, 0.85, 0.95, 0.5),
        ("Courage", 5, 0.95, 0.6, 0.4),
        ("Peace", 2, 0.6, 0.5, 0.8),
        ("Justice", 7, 0.9, 0.7, 0.5),
        ("Beauty", 4, 0.6, 0.6, 0.8),
        ("Freedom", 0, 0.7, 0.8, 0.6),
    ];
    
    for (name, pos, e, l, p) in demo_data {
        matrix.insert(create_demo_node(name, pos, e, l, p));
    }
    
    // Create Bevy app with WASM settings - 0.17.0-dev should support this
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.04, 0.04, 0.08)))
        .insert_resource(FluxMatrixResource(matrix))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Flux Matrix 3D".to_string(),
                canvas: Some("#bevy-canvas".to_string()),
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup_3d_scene)
        .add_systems(Update, rotate_camera)
        .run();
}

/// Setup the 3D scene with exact geometry from 2D images
fn setup_3d_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    matrix: Res<FluxMatrixResource>,
) {
    let radius = 8.0;
    
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    
    // Lighting
    commands.spawn((
        PointLight {
            intensity: 2000.0,
            range: 100.0,
            ..default()
        },
        Transform::from_xyz(10.0, 10.0, 10.0),
    ));
    
    // Sacred Triangle (3-6-9) - Bold white lines
    spawn_line(&mut commands, &mut meshes, &mut materials,
        get_position(3, radius), get_position(6, radius), Color::WHITE);
    spawn_line(&mut commands, &mut meshes, &mut materials,
        get_position(6, radius), get_position(9, radius), Color::WHITE);
    spawn_line(&mut commands, &mut meshes, &mut materials,
        get_position(9, radius), get_position(3, radius), Color::WHITE);
    
    // Doubling sequence star (1→2→4→8→7→5→1)
    let star_sequence = [1, 2, 4, 8, 7, 5, 1];
    for i in 0..star_sequence.len() - 1 {
        spawn_line(&mut commands, &mut meshes, &mut materials,
            get_position(star_sequence[i], radius),
            get_position(star_sequence[i + 1], radius),
            Color::srgb(0.3, 0.3, 0.4));
    }
    
    // Data point spheres
    for pos in 0..=9 {
        if let Some(node) = matrix.0.get(pos) {
            let position_3d = get_position(pos, radius);
            let elp = get_elp_from_node(&node);
            let color = get_dominant_color(elp.ethos as f32, elp.logos as f32, elp.pathos as f32);
            
            // Main sphere
            commands.spawn((
                Mesh3d(meshes.add(Sphere::new(0.5).mesh().ico(5).unwrap())),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: color,
                    emissive: LinearRgba::from(color) * 0.5,
                    ..default()
                })),
                Transform::from_translation(position_3d),
            ));
        }
    }
}

/// Get 3D position for flux position (9 at top, clockwise)
fn get_position(pos: u8, radius: f32) -> Vec3 {
    let angle = ((9 - pos) as f32 / 9.0) * 2.0 * PI - PI / 2.0;
    Vec3::new(
        angle.cos() * radius,
        angle.sin() * radius,
        0.0
    )
}

/// Get dominant ELP color
fn get_dominant_color(e: f32, l: f32, p: f32) -> Color {
    if e > l && e > p {
        Color::srgb(1.0, 0.3, 0.3) // Red - Ethos
    } else if l > p {
        Color::srgb(0.3, 0.3, 1.0) // Blue - Logos
    } else {
        Color::srgb(0.3, 1.0, 0.3) // Green - Pathos
    }
}

/// Extract ELP from node
fn get_elp_from_node(node: &FluxNode) -> ELPTensor {
    ELPTensor::new(
        *node.attributes.parameters.get("ethos").unwrap_or(&0.5),
        *node.attributes.parameters.get("logos").unwrap_or(&0.5),
        *node.attributes.parameters.get("pathos").unwrap_or(&0.5),
    )
}

/// Spawn a line between two points
fn spawn_line(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    start: Vec3,
    end: Vec3,
    color: Color,
) {
    let length = start.distance(end);
    let midpoint = (start + end) / 2.0;
    let direction = (end - start).normalize();
    
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.1, 0.1, length))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: color,
            unlit: true,
            ..default()
        })),
        Transform::from_translation(midpoint)
            .looking_to(direction, Vec3::Y),
    ));
}

/// Rotate camera slowly
fn rotate_camera(time: Res<Time>, mut query: Query<&mut Transform, With<Camera3d>>) {
    for mut transform in query.iter_mut() {
        transform.rotate_around(Vec3::ZERO, Quat::from_rotation_y(time.delta_secs() * 0.2));
    }
}
