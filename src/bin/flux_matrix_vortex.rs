/// Flux Matrix Vortex Math Visualization
/// 
/// Interactive 3D visualization of the Vortex Math pattern:
/// - Position 9 at top (12 o'clock)
/// - Clockwise arrangement: 1, 2, 3, 4, 5, 6, 7, 8
/// - Sacred 3-6-9 triangle emphasized
/// - Internal star pattern connections
/// - Ethos-Logos-Pathos (ELP) tensor visualization

use bevy::prelude::*;
use bevy::window::WindowDescriptor;
use spatial_vortex::{
    lock_free_flux::LockFreeFluxMatrix,
    visualization::{FluxLayout, FluxVisualization, bevy_3d::*},
    models::{FluxNode, NodeAttributes, NodeState, NodeDynamics, SemanticIndex},
};
use std::collections::HashMap;

fn main() {
    // Create flux matrix with sample data
    let matrix = LockFreeFluxMatrix::new("VortexMath3D".to_string());
    
    // Add test data points matching the 2D visualization
    let test_data = vec![
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
    
    println!("\n🌀 VORTEX MATH 3D VISUALIZATION");
    println!("================================\n");
    println!("📦 Loading {} data points...", test_data.len());
    
    for (name, pos, e, l, p) in &test_data {
        let node = create_test_node(name, *pos, *e, *l, *p);
        matrix.insert(node);
        
        let sacred = if [3, 6, 9].contains(pos) { "⭐" } else { " " };
        println!("   {} Position {}: {} (E:{:.2} L:{:.2} P:{:.2})", 
            sacred, pos, name, e, l, p);
    }
    
    // Create visualization data
    let layout = FluxLayout::sacred_geometry_layout();
    let viz = FluxVisualization::from_flux_matrix(
        &matrix,
        layout,
        "Vortex Math - Sacred Geometry 3D".to_string(),
    );
    
    println!("\n🎨 Starting 3D visualization...");
    println!("   Use mouse to rotate camera");
    println!("   Sacred positions (3-6-9) are highlighted\n");
    
    // Run Bevy app
    App::new()
        .insert_resource(WindowDescriptor {
            title: "SpatialVortex - Vortex Math 3D".to_string(),
            width: 1280.0,
            height: 720.0,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .insert_resource(FluxVisualizationData { viz })
        .add_plugin(Flux3DPlugin)
        .run();
}

/// Create a test node with ELP (Ethos-Logos-Pathos) parameters
fn create_test_node(name: &str, position: u8, ethos: f64, logos: f64, pathos: f64) -> FluxNode {
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
