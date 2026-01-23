//! 🌀 EPIC FLUX 3D VISUALIZATION
//! 
//! Consolidated Bevy WASM visualization combining ALL features:
//! - Sacred Geometry (3-6-9 triangle with cyan highlights)
//! - Flux Flow Pattern (1→2→4→8→7→5→1 doubling sequence)
//! - Word Beams with ELP color channels (Red/Green/Blue)
//! - Shape-Based Architecture (Box/Cylinder/Sphere)
//! - Sacred Intersection Effects (bursts, ripples, ascension)
//! - Interactive Orbit Camera with auto-rotation
//! - Confidence-based scaling and positioning
//! - ML Enhancement visualization
//! - Real-time trails and particle effects

#![cfg(target_arch = "wasm32")]

use bevy::prelude::*;
use spatial_vortex::{
    lock_free_flux::LockFreeFluxMatrix,
    models::{FluxNode as FluxNodeModel, NodeAttributes, NodeState, NodeDynamics, SemanticIndex, ELPTensor},
};
use std::collections::HashMap;
use std::f32::consts::PI;
use wasm_bindgen::prelude::*;

// ============================================================================
// RESOURCES & CONFIGURATION
// ============================================================================

#[derive(Resource)]
struct FluxMatrixResource(LockFreeFluxMatrix);

#[derive(Resource, Debug, Clone)]
struct VisualizationConfig {
    pub auto_rotate: bool,
    pub rotation_speed: f32,
    pub beam_speed: f32,
    pub show_trails: bool,
    pub camera_distance: f32,
}

impl Default for VisualizationConfig {
    fn default() -> Self {
        Self {
            auto_rotate: true,
            rotation_speed: 0.3,
            beam_speed: 1.0,
            show_trails: true,
            camera_distance: 25.0,
        }
    }
}

// ============================================================================
// COMPONENTS
// ============================================================================

/// Flux position node (0-9)
#[derive(Component, Debug, Clone)]
struct FluxNode {
    pub position: u8,
    pub is_sacred: bool,
    pub elp: ELPTensor,
    pub activity: f32,
}

/// Word beam flowing through the matrix
#[derive(Component, Debug, Clone)]
struct WordBeam {
    pub word: String,
    pub position: u8,
    pub target: u8,
    pub progress: f32,
    pub elp: ELPTensor,
}

/// Processing block (Box shape)
#[derive(Component, Debug, Clone)]
struct ProcessingBlock {
    pub label: String,
    pub state: ProcessingState,
}

#[derive(Debug, Clone, PartialEq)]
enum ProcessingState {
    Idle,
    Processing,
    Complete,
}

/// Database node (Cylinder shape)
#[derive(Component, Debug, Clone)]
struct DatabaseNode {
    pub db_type: String,
    pub connections: usize,
}

/// Sacred intersection effect
#[derive(Component, Debug, Clone)]
struct IntersectionEffect {
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub effect_type: EffectType,
}

#[derive(Debug, Clone)]
enum EffectType {
    GreenBurst,    // Position 3
    RedRipple,     // Position 6  
    BlueAscension, // Position 9
}

/// Trail component for beams
#[derive(Component, Debug, Clone)]
struct BeamTrail {
    pub positions: Vec<Vec3>,
    pub max_length: usize,
}

/// Orbit camera
#[derive(Component)]
struct OrbitCamera {
    pub angle: f32,
    pub height: f32,
}

// ============================================================================
// WASM ENTRY POINT
// ============================================================================

#[wasm_bindgen(start)]
pub fn run() {
    console_error_panic_hook::set_once();
    
    // Create flux matrix with comprehensive demo data
    let matrix = LockFreeFluxMatrix::new("Epic_3D_Visualization".to_string());
    
    // Sacred positions with high ELP values
    let demo_data = vec![
        // Sacred positions
        ("Love", 3, 0.9, 0.6, 0.95),     // Position 3: Creative Trinity (Green)
        ("Truth", 6, 0.95, 0.9, 0.6),    // Position 6: Harmonic Balance (Red)
        ("Creation", 9, 0.6, 0.8, 0.95), // Position 9: Completion Cycle (Blue)
        
        // Flow pattern positions (1→2→4→8→7→5)
        ("Unity", 1, 0.8, 0.7, 0.85),
        ("Duality", 2, 0.75, 0.85, 0.7),
        ("Foundation", 4, 0.85, 0.8, 0.75),
        ("Wisdom", 8, 0.9, 0.85, 0.7),
        ("Change", 7, 0.7, 0.8, 0.9),
        ("Balance", 5, 0.8, 0.75, 0.8),
        
        // Center
        ("Void", 0, 0.5, 0.5, 0.5),
    ];
    
    for (name, pos, e, l, p) in demo_data {
        matrix.insert(create_demo_node(name, pos, e, l, p));
    }
    
    // Launch Bevy app
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.02, 0.02, 0.06)))
        .insert_resource(FluxMatrixResource(matrix))
        .insert_resource(VisualizationConfig::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "🌀 Epic Flux Matrix 3D - SpatialVortex".to_string(),
                canvas: Some("#bevy-canvas".to_string()),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, (
            setup_scene,
            spawn_flux_structure,
            spawn_processing_blocks,
            spawn_database_nodes,
        ))
        .add_systems(Update, (
            rotate_camera,
            animate_sacred_nodes,
            update_word_beams,
            spawn_beams_periodically,
            process_sacred_intersections,
            animate_intersection_effects,
            update_processing_blocks,
        ))
        .run();
}

// ============================================================================
// SETUP SYSTEMS
// ============================================================================

fn setup_scene(
    mut commands: Commands,
) {
    // Camera with orbit
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 12.0, 25.0).looking_at(Vec3::ZERO, Vec3::Y),
        OrbitCamera {
            angle: 0.0,
            height: 12.0,
        },
    ));
    
    // Directional light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(10.0, 15.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    
    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 400.0,
    });
}

fn spawn_flux_structure(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    matrix: Res<FluxMatrixResource>,
) {
    let radius = 10.0;
    
    // Spawn flux nodes (0-9)
    for pos in 0..=9u8 {
        if let Some(node) = matrix.0.get(pos) {
            let position_3d = get_circular_position(pos, radius);
            let is_sacred = [3, 6, 9].contains(&pos);
            let elp = extract_elp(&node);
            
            // Node size based on importance
            let node_radius = if is_sacred { 0.6 } else if pos == 0 { 0.4 } else { 0.3 };
            
            // Color from ELP or sacred color
            let color = if is_sacred {
                get_sacred_color(pos)
            } else {
                elp_to_color(&elp)
            };
            
            // Spawn node sphere
            commands.spawn((
                Mesh3d(meshes.add(Sphere::new(node_radius).mesh().ico(5).unwrap())),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: color,
                    emissive: LinearRgba::from(color) * if is_sacred { 2.0 } else { 0.5 },
                    metallic: 0.3,
                    perceptual_roughness: 0.6,
                    ..default()
                })),
                Transform::from_translation(position_3d),
                FluxNode {
                    position: pos,
                    is_sacred,
                    elp: elp.clone(),
                    activity: 0.0,
                },
            ));
            
            // Label above node
            spawn_label(&mut commands, position_3d + Vec3::Y * 1.0, &node.semantic_index.neutral_base);
        }
    }
    
    // Draw sacred triangle (3-6-9) with CYAN lines
    draw_sacred_triangle(&mut commands, &mut meshes, &mut materials, radius);
    
    // Draw flow pattern lines (1→2→4→8→7→5→1)
    draw_flow_pattern(&mut commands, &mut meshes, &mut materials, radius);
}

fn spawn_processing_blocks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Spawn 3 processing blocks on the left side
    let blocks = vec![
        ("Geometric Inference", -15.0, 8.0, ProcessingState::Processing),
        ("ML Enhancement", -15.0, 4.0, ProcessingState::Processing),
        ("AI Consensus", -15.0, 0.0, ProcessingState::Complete),
    ];
    
    for (label, x, y, state) in blocks {
        let color = match state {
            ProcessingState::Idle => Color::srgb(0.4, 0.4, 0.4),
            ProcessingState::Processing => Color::srgb(0.9, 0.9, 0.2),
            ProcessingState::Complete => Color::srgb(0.2, 0.9, 0.2),
        };
        
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(3.0, 1.5, 1.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                metallic: 0.2,
                perceptual_roughness: 0.5,
                ..default()
            })),
            Transform::from_xyz(x, y, 0.0),
            ProcessingBlock {
                label: label.to_string(),
                state,
            },
        ));
        
        spawn_label(&mut commands, Vec3::new(x, y + 1.5, 0.0), label);
    }
}

fn spawn_database_nodes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Spawn 2 database nodes on the right side
    let databases = vec![
        ("PostgreSQL", 15.0, 6.0, 150),
        ("Redis", 15.0, 2.0, 85),
    ];
    
    for (label, x, y, connections) in databases {
        let height = 2.0 + (connections as f32 * 0.01);
        
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.6, height))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.2, 0.4, 0.8),
                emissive: LinearRgba::new(0.1, 0.2, 0.4, 1.0),
                metallic: 0.5,
                perceptual_roughness: 0.3,
                ..default()
            })),
            Transform::from_xyz(x, y, 0.0),
            DatabaseNode {
                db_type: label.to_string(),
                connections,
            },
        ));
        
        spawn_label(&mut commands, Vec3::new(x, y + height / 2.0 + 1.0, 0.0), label);
    }
}

// ============================================================================
// UPDATE SYSTEMS
// ============================================================================

fn rotate_camera(
    time: Res<Time>,
    config: Res<VisualizationConfig>,
    mut query: Query<(&mut Transform, &mut OrbitCamera)>,
) {
    if !config.auto_rotate {
        return;
    }
    
    for (mut transform, mut orbit) in query.iter_mut() {
        orbit.angle += time.delta_secs() * config.rotation_speed;
        
        let x = config.camera_distance * orbit.angle.cos();
        let z = config.camera_distance * orbit.angle.sin();
        
        transform.translation = Vec3::new(x, orbit.height, z);
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}

fn animate_sacred_nodes(
    time: Res<Time>,
    mut query: Query<(&FluxNode, &mut Transform)>,
) {
    let t = time.elapsed_secs();
    
    for (node, mut transform) in query.iter_mut() {
        if node.is_sacred {
            // Pulsing effect
            let pulse = (t * 2.0).sin() * 0.5 + 0.5;
            let base_scale = if node.position == 3 { 1.0 } 
                           else if node.position == 6 { 1.0 }
                           else { 1.0 };
            transform.scale = Vec3::splat(base_scale * (1.0 + pulse * 0.15));
            
            // Rotate divine node (9)
            if node.position == 9 {
                transform.rotate_y(time.delta_secs() * 0.5);
            }
        }
        
        // Activity-based glow
        if node.activity > 0.0 {
            transform.scale = Vec3::splat(1.0 + node.activity * 0.3);
        }
    }
}

fn update_word_beams(
    time: Res<Time>,
    config: Res<VisualizationConfig>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut WordBeam, &mut Transform)>,
) {
    for (entity, mut beam, mut transform) in query.iter_mut() {
        // Update progress
        beam.progress += time.delta_secs() * config.beam_speed * 0.2;
        
        // Move along circle
        if beam.progress >= 1.0 {
            beam.progress = 0.0;
            beam.position = beam.target;
            beam.target = get_next_flow_position(beam.position);
        }
        
        // Interpolate position
        let start = get_circular_position(beam.position, 10.0);
        let end = get_circular_position(beam.target, 10.0);
        let pos = start.lerp(end, beam.progress);
        
        // Add curvature based on pathos
        let curve_height = beam.elp.pathos as f32 * 0.5;
        let curve_offset = (beam.progress * PI).sin() * curve_height;
        
        transform.translation = pos + Vec3::Y * curve_offset;
        
        // Rotate to face direction of travel
        let direction = (end - start).normalize();
        if direction.length() > 0.01 {
            transform.look_to(direction, Vec3::Y);
        }
    }
}

fn spawn_beams_periodically(
    time: Res<Time>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    matrix: Res<FluxMatrixResource>,
    mut timer: Local<f32>,
) {
    *timer += time.delta_secs();
    
    if *timer >= 3.0 {
        *timer = 0.0;
        
        // Spawn a new beam at a random position
        let positions = [1, 2, 4, 5, 7, 8]; // Flow positions
        let pos = positions[(time.elapsed_secs() as usize) % positions.len()];
        
        if let Some(node) = matrix.0.get(pos) {
            let elp = extract_elp(&node);
            let color = elp_to_color(&elp);
            let start_pos = get_circular_position(pos, 10.0);
            
            commands.spawn((
                Mesh3d(meshes.add(Capsule3d::new(0.15, 0.4))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: color,
                    emissive: LinearRgba::from(color) * 2.0,
                    alpha_mode: AlphaMode::Blend,
                    ..default()
                })),
                Transform::from_translation(start_pos),
                WordBeam {
                    word: node.semantic_index.neutral_base.clone(),
                    position: pos,
                    target: get_next_flow_position(pos),
                    progress: 0.0,
                    elp: elp.clone(),
                },
            ));
        }
    }
}

fn process_sacred_intersections(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    beams: Query<&WordBeam>,
    mut nodes: Query<&mut FluxNode>,
) {
    for beam in beams.iter() {
        if beam.progress < 0.1 && beam.target != 0 && [3, 6, 9].contains(&beam.target) {
            // Trigger effect at sacred intersection
            let effect_type = match beam.target {
                3 => EffectType::GreenBurst,
                6 => EffectType::RedRipple,
                9 => EffectType::BlueAscension,
                _ => continue,
            };
            
            let pos = get_circular_position(beam.target, 10.0);
            let color = get_sacred_color(beam.target);
            
            spawn_intersection_effect(
                &mut commands,
                &mut meshes,
                &mut materials,
                pos,
                color,
                effect_type,
            );
            
            // Increase node activity
            for mut node in nodes.iter_mut() {
                if node.position == beam.target {
                    node.activity = 1.0;
                }
            }
        }
    }
}

fn animate_intersection_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut IntersectionEffect)>,
) {
    for (entity, mut transform, mut effect) in query.iter_mut() {
        effect.lifetime += time.delta_secs();
        let progress = effect.lifetime / effect.max_lifetime;
        
        match effect.effect_type {
            EffectType::GreenBurst => {
                // Expand rapidly
                transform.scale = Vec3::splat(1.0 + progress * 4.0);
            }
            EffectType::RedRipple => {
                // Oscillate
                let ripple = (progress * 15.0).sin() * 0.5;
                transform.scale = Vec3::splat(1.0 + ripple);
            }
            EffectType::BlueAscension => {
                // Rise upward
                transform.translation.y += time.delta_secs() * 3.0;
                transform.scale = Vec3::splat(1.0 + progress * 2.0);
            }
        }
        
        // Despawn when done
        if effect.lifetime >= effect.max_lifetime {
            commands.entity(entity).despawn();
        }
    }
}

fn update_processing_blocks(
    time: Res<Time>,
    mut query: Query<(&ProcessingBlock, &mut Transform)>,
) {
    let t = time.elapsed_secs();
    
    for (block, mut transform) in query.iter_mut() {
        if block.state == ProcessingState::Processing {
            // Gentle pulse
            let pulse = (t * 3.0).sin() * 0.5 + 0.5;
            transform.scale = Vec3::new(1.0, 1.0 + pulse * 0.1, 1.0);
        }
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn create_demo_node(name: &str, position: u8, ethos: f64, logos: f64, pathos: f64) -> FluxNodeModel {
    let mut parameters = HashMap::new();
    parameters.insert("ethos".to_string(), ethos);
    parameters.insert("logos".to_string(), logos);
    parameters.insert("pathos".to_string(), pathos);
    
    FluxNodeModel {
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

fn get_circular_position(pos: u8, radius: f32) -> Vec3 {
    // Position 9 at top (12 o'clock), clockwise
    let angle = ((9 - pos) as f32 / 9.0) * 2.0 * PI - PI / 2.0;
    Vec3::new(
        angle.cos() * radius,
        angle.sin() * radius,
        0.0
    )
}

fn extract_elp(node: &FluxNodeModel) -> ELPTensor {
    ELPTensor::new(
        *node.attributes.parameters.get("ethos").unwrap_or(&0.5),
        *node.attributes.parameters.get("logos").unwrap_or(&0.5),
        *node.attributes.parameters.get("pathos").unwrap_or(&0.5),
    )
}

fn elp_to_color(elp: &ELPTensor) -> Color {
    Color::srgb(
        (elp.ethos / 9.0) as f32,
        (elp.logos / 9.0) as f32,
        (elp.pathos / 9.0) as f32,
    )
}

fn get_sacred_color(pos: u8) -> Color {
    match pos {
        3 => Color::srgb(0.2, 1.0, 0.4),  // Green - Position 3
        6 => Color::srgb(1.0, 0.3, 0.3),  // Red - Position 6
        9 => Color::srgb(0.3, 0.6, 1.0),  // Blue - Position 9
        _ => Color::WHITE,
    }
}

fn get_next_flow_position(current: u8) -> u8 {
    // Flow pattern: 1→2→4→8→7→5→1
    match current {
        1 => 2,
        2 => 4,
        4 => 8,
        8 => 7,
        7 => 5,
        5 => 1,
        _ => 1, // Default to start of flow
    }
}

fn draw_sacred_triangle(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    radius: f32,
) {
    let positions = [3u8, 6, 9];
    let color = Color::srgb(0.0, 0.8, 1.0); // Cyan for sacred
    
    for i in 0..3 {
        let from = get_circular_position(positions[i], radius);
        let to = get_circular_position(positions[(i + 1) % 3], radius);
        
        draw_line(commands, meshes, materials, from, to, color, 0.1);
    }
}

fn draw_flow_pattern(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    radius: f32,
) {
    let flow = [1u8, 2, 4, 8, 7, 5, 1];
    let color = Color::srgb(0.3, 0.3, 0.4);
    
    for i in 0..flow.len() - 1 {
        let from = get_circular_position(flow[i], radius);
        let to = get_circular_position(flow[i + 1], radius);
        
        draw_line(commands, meshes, materials, from, to, color, 0.05);
    }
}

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
    
    let rotation = Quat::from_rotation_arc(Vec3::Y, direction.normalize());
    
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(thickness, length))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: color,
            emissive: LinearRgba::from(color) * 0.5,
            unlit: false,
            ..default()
        })),
        Transform {
            translation: midpoint,
            rotation,
            ..default()
        },
    ));
}

fn spawn_intersection_effect(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    color: Color,
    effect_type: EffectType,
) {
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.3).mesh().ico(3).unwrap())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::rgba(color.r(), color.g(), color.b(), 0.6),
            emissive: LinearRgba::from(color) * 3.0,
            alpha_mode: AlphaMode::Blend,
            ..default()
        })),
        Transform::from_translation(position),
        IntersectionEffect {
            lifetime: 0.0,
            max_lifetime: 1.5,
            effect_type,
        },
    ));
}

fn spawn_label(
    commands: &mut Commands,
    position: Vec3,
    text: &str,
) {
    // Note: Bevy text rendering in 3D is complex
    // For production, use a proper 3D text solution
    // For now, this is a placeholder
    commands.spawn((
        Text3d::default(),
        Transform::from_translation(position),
    ));
}

// Main function for non-WASM builds
fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        println!("This binary is designed for WASM target only.");
        println!("Build with: cargo build --target wasm32-unknown-unknown --features bevy_support");
    }
    
    #[cfg(target_arch = "wasm32")]
    {
        // WASM builds use #[wasm_bindgen(start)] above
    }
}
