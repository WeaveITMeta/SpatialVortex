//! Epic Flux 3D WASM Entry Point
//! 
//! This module provides the WASM entry point for the consolidated
//! Bevy 3D visualization. Import and call epic_flux_3d_init() from JavaScript.

#![cfg(target_arch = "wasm32")]

use wasm_bindgen::prelude::*;
use bevy::prelude::*;
use crate::lock_free_flux::LockFreeFluxMatrix;
use crate::models::{FluxNode as FluxNodeModel, NodeAttributes, NodeState, NodeDynamics, SemanticIndex, ELPTensor};
use std::collections::HashMap;
use std::f32::consts::PI;

use std::sync::atomic::{AtomicBool, Ordering};

static INITIALIZED: AtomicBool = AtomicBool::new(false);

// Re-export for WASM - manual initialization with guard
#[wasm_bindgen]
pub fn epic_flux_3d_init() {
    // Prevent multiple initialization
    if INITIALIZED.swap(true, Ordering::SeqCst) {
        console::log_1(&"⚠️ Already initialized, skipping...".into());
        return;
    }
    
    console_error_panic_hook::set_once();
    
    console::log_1(&"🌀 Initializing Epic Flux 3D...".into());
    
    // Create demo matrix
    let matrix = create_demo_matrix();
    
    console::log_1(&"✅ Matrix created, launching Bevy app...".into());
    
    // For WASM, we need to use the special Bevy WASM runner
    // Don't call .run() directly - use set_runner for WASM compatibility
    console::log_1(&"🚀 Configuring Bevy app for WASM...".into());
    
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.02, 0.02, 0.06)))
        .insert_resource(FluxMatrixResource(matrix))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "🌀 Epic Flux Matrix 3D".to_string(),
                canvas: Some("#bevy-canvas".to_string()),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup_epic_scene)
        .add_systems(Update, animate_nodes)
        .run();  // In WASM, this uses WinitPlugin's runner which handles the event loop properly
}

#[derive(Resource)]
struct FluxMatrixResource(LockFreeFluxMatrix);


#[derive(Component)]
struct FluxNode {
    position: u8,
    is_sacred: bool,
}


fn create_demo_matrix() -> LockFreeFluxMatrix {
    let matrix = LockFreeFluxMatrix::new("Epic_3D".to_string());
    
    let data = vec![
        ("Love", 3, 0.9, 0.6, 0.95),
        ("Truth", 6, 0.95, 0.9, 0.6),
        ("Creation", 9, 0.6, 0.8, 0.95),
        ("Unity", 1, 0.8, 0.7, 0.85),
        ("Duality", 2, 0.75, 0.85, 0.7),
        ("Foundation", 4, 0.85, 0.8, 0.75),
        ("Balance", 5, 0.8, 0.75, 0.8),
        ("Wisdom", 8, 0.9, 0.85, 0.7),
        ("Change", 7, 0.7, 0.8, 0.9),
        ("Void", 0, 0.5, 0.5, 0.5),
    ];
    
    for (name, pos, e, l, p) in data {
        let mut params = HashMap::new();
        params.insert("ethos".to_string(), e);
        params.insert("logos".to_string(), l);
        params.insert("pathos".to_string(), p);
        
        matrix.insert(FluxNodeModel {
            position: pos,
            base_value: pos,
            semantic_index: SemanticIndex {
                positive_associations: vec![],
                negative_associations: vec![],
                neutral_base: name.to_string(),
                predicates: vec![],
                relations: vec![],
            },
            attributes: NodeAttributes {
                properties: HashMap::new(),
                parameters: params,
                state: NodeState {
                    active: true,
                    last_accessed: chrono::DateTime::from_timestamp(0, 0).unwrap(),
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
        });
    }
    
    matrix
}

fn setup_epic_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    matrix: Res<FluxMatrixResource>,
) {
    console::log_1(&"Setting up Epic 3D scene...".into());
    
    // Static camera - good viewing angle, no rotation glitching
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, -11.0, 35.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    
    // Multiple strong lights for full coverage
    commands.spawn((
        DirectionalLight {
            illuminance: 20000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(10.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    
    commands.spawn((
        DirectionalLight {
            illuminance: 15000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(-10.0, 10.0, -10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    
    let radius = 13.0;  // Sacred 13-scale
    
    // Spawn nodes
    for pos in 0..=9u8 {
        if let Some(node) = matrix.0.get(pos) {
            let position_3d = get_position(pos, radius);
            let is_sacred = [3, 6, 9].contains(&pos);
            
            let _elp = ELPTensor::new(
                *node.node.attributes.parameters.get("ethos").unwrap_or(&0.5),
                *node.node.attributes.parameters.get("logos").unwrap_or(&0.5),
                *node.node.attributes.parameters.get("pathos").unwrap_or(&0.5),
            );
            
            // All nodes are white - unlit to avoid PBR shader variants on WebGL
            let node_radius = if is_sacred { 0.6 } else if pos == 0 { 0.4 } else { 0.3 };
            
            let _node_entity = commands.spawn((
                Mesh3d(meshes.add(Sphere::new(node_radius).mesh().ico(5).unwrap())),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(1.0, 1.0, 1.0),
                    unlit: true,
                    ..default()
                })),
                Transform::from_translation(position_3d),
                FluxNode {
                    position: pos,
                    is_sacred,
                },
            )).id();

            // Labels: number above the node, name centered on the node
            let name_text = node.node.semantic_index.neutral_base.clone();
            let num_text = format!("{}", pos);

            // Static number label (above node) - billboarded automatically
            commands.spawn((
                Text2d::new(num_text),
                TextFont { 
                    font_size: 20.0,
                    ..default() 
                },
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
                Transform::from_translation(position_3d + Vec3::new(0.0, 0.9, 0.0)),
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
                    font_size: 16.0,
                    ..default() 
                },
                TextColor(label_color),
                Transform::from_translation(position_3d),
            ));
        }
    }
    
    // Draw sacred triangle
    draw_triangle(&mut commands, &mut meshes, &mut materials, radius);
    
    // Draw flow sequence lines (1→2→4→8→7→5→1)
    draw_flow_lines(&mut commands, &mut meshes, &mut materials, radius);
    
    console::log_1(&"✅ Scene setup complete!".into());
}

fn get_position(pos: u8, radius: f32) -> Vec3 {
    // Circular on XY plane, position 9 at top (add PI to flip)
    let angle = ((9 - pos) as f32 / 9.0) * 2.0 * PI - PI / 2.0 + PI;
    Vec3::new(angle.cos() * radius, angle.sin() * radius, 0.0)
}

fn draw_triangle(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    radius: f32,
) {
    let positions = [3u8, 6, 9];
    for i in 0..3 {
        let from = get_position(positions[i], radius);
        let to = get_position(positions[(i + 1) % 3], radius);
        draw_line(commands, meshes, materials, from, to, true);
    }
}

fn draw_flow_lines(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    radius: f32,
) {
    // Doubling sequence: 1→2→4→8→7→5→1
    let flow = [1u8, 2, 4, 8, 7, 5];
    for i in 0..flow.len() {
        let from = get_position(flow[i], radius);
        let to = get_position(flow[(i + 1) % flow.len()], radius);
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
    // Cylinder points along Y, so rotate Y axis to point along direction
    let rotation = Quat::from_rotation_arc(Vec3::Y, direction.normalize());
    
    let (color, radius) = if is_sacred {
        // Sacred triangle (3-6-9) - bright cyan
        (Color::srgb(0.0, 0.9, 1.0), 0.12)
    } else {
        // Flow lines (1-2-4-8-7-5) - subtle gray
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


fn animate_nodes(
    time: Res<Time>,
    mut query: Query<(&FluxNode, &mut Transform)>,
) {
    let t = time.elapsed_secs();
    
    for (node, mut transform) in query.iter_mut() {
        if node.is_sacred {
            let pulse = (t * 2.0).sin() * 0.5 + 0.5;
            transform.scale = Vec3::splat(1.0 + pulse * 0.15);
            
            if node.position == 9 {
                transform.rotate_y(time.delta_secs() * 0.5);
            }
        }
    }
}

// Console logging helper
mod console {
    use wasm_bindgen::prelude::*;
    
    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = console)]
        pub fn log(s: &str);
    }
    
    pub fn log_1(s: &JsValue) {
        log(&format!("{:?}", s));
    }
}
