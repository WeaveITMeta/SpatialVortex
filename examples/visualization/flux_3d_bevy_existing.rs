/// 3D Flux Visualization using EXISTING Bevy implementation
/// 
/// Uses the already-created flux_mesh and beam_renderer modules
/// Run: cargo run --example flux_3d_bevy_existing --features bevy_support

#[cfg(feature = "bevy_support")]
use bevy::prelude::*;
#[cfg(feature = "bevy_support")]
use spatial_vortex::{
    lock_free_flux::LockFreeFluxMatrix,
    flux_mesh::{FluxGeometry, spawn_flux_structure, animate_sacred_nodes},
    beam_renderer::{BeamRenderConfig, spawn_word_beam, update_beam_flow, render_beam_trails},
    models::{FluxNode, NodeAttributes, NodeState, NodeDynamics, SemanticIndex, BeamTensor},
};
use std::collections::HashMap;

#[cfg(feature = "bevy_support")]
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

#[cfg(feature = "bevy_support")]
fn main() {
    println!("\n🌀 3D FLUX MATRIX VISUALIZATION (Using Existing Bevy Implementation)");
    println!("=================================================================\n");
    
    // Create flux matrix with demo data
    let matrix = LockFreeFluxMatrix::new("3D_Demo".to_string());
    
    let demo_data = vec![
        ("Love", 3, 0.7, 0.5, 0.95),       // Sacred - high pathos
        ("Truth", 6, 0.85, 0.95, 0.5),     // Sacred - high logos
        ("Creation", 9, 0.9, 0.6, 0.5),    // Sacred - high ethos
        ("Joy", 1, 0.6, 0.4, 0.9),
        ("Wisdom", 8, 0.85, 0.95, 0.5),
        ("Courage", 5, 0.95, 0.6, 0.4),
        ("Peace", 2, 0.6, 0.5, 0.8),
        ("Justice", 7, 0.9, 0.7, 0.5),
        ("Beauty", 4, 0.6, 0.6, 0.8),
        ("Freedom", 0, 0.7, 0.8, 0.6),
    ];
    
    println!("📦 Creating {} data points...", demo_data.len());
    for (name, pos, e, l, p) in &demo_data {
        matrix.insert(create_demo_node(name, *pos, *e, *l, *p));
        let sacred = if [3, 6, 9].contains(pos) { "⭐" } else { " " };
        println!("   {} Position {}: {} (E:{:.2} L:{:.2} P:{:.2})", 
            sacred, pos, name, e, l, p);
    }
    
    println!("\n🎨 Launching Bevy 3D visualization...");
    println!("   Controls:");
    println!("   - Mouse: Drag to rotate camera");
    println!("   - Scroll: Zoom in/out");
    println!("   - Space: Spawn word beam");
    println!("   - ESC: Exit\n");
    
    // Launch Bevy app with existing implementation
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "SpatialVortex - Flux Matrix 3D".to_string(),
                width: 1280.0,
                height: 720.0,
                ..default()
            },
            ..default()
        }))
        // Resources
        .insert_resource(BeamRenderConfig::default())
        .insert_resource(FluxGeometry::new(2.0))
        .insert_resource(ClearColor(Color::rgb(0.05, 0.05, 0.1)))
        
        // Startup systems
        .add_startup_system(setup_scene)
        .add_startup_system(spawn_flux_structure)
        .add_startup_system(spawn_demo_beams)
        
        // Update systems
        .add_system(animate_sacred_nodes)
        .add_system(update_beam_flow)
        .add_system(render_beam_trails)
        .add_system(camera_controls)
        
        .run();
}

#[cfg(feature = "bevy_support")]
fn setup_scene(mut commands: Commands) {
    // Camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 8.0, 15.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    
    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::rgb(0.2, 0.2, 0.3),
        brightness: 0.5,
    });
    
    // Directional light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLightBundle {
            illuminance: 10000.0,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

#[cfg(feature = "bevy_support")]
fn spawn_demo_beams(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    geometry: Res<FluxGeometry>,
) {
    // Create demo beam tensors for visualization
    let demo_words = vec![
        ("Love", 3, 0.95, 0.7, 0.5, 0.95),
        ("Truth", 6, 0.85, 0.85, 0.95, 0.5),
        ("Creation", 9, 0.90, 0.9, 0.6, 0.5),
    ];
    
    for (word, pos, conf, e, l, p) in demo_words {
        let beam = BeamTensor {
            word: word.to_string(),
            position: pos,
            confidence: conf,
            ethos: (e * 9.0) as u8,
            logos: (l * 9.0) as u8,
            pathos: (p * 9.0) as u8,
            curviness_signed: 0.5,
        };
        
        spawn_word_beam(&mut commands, &mut meshes, &mut materials, &beam, &geometry);
    }
}

#[cfg(feature = "bevy_support")]
fn camera_controls(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Camera>>,
) {
    for mut transform in query.iter_mut() {
        // Auto-rotate around Y axis
        let rotation_speed = 0.2;
        transform.rotate_around(
            Vec3::ZERO,
            Quat::from_rotation_y(rotation_speed * time.delta_seconds()),
        );
    }
}

#[cfg(not(feature = "bevy_support"))]
fn main() {
    eprintln!("❌ This example requires the 'bevy_support' feature.");
    eprintln!("   Run with: cargo run --example flux_3d_bevy_existing --features bevy_support");
    std::process::exit(1);
}
