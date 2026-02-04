// Core modules (2026 Distilled)
pub mod error;
pub mod config;
pub mod core;
pub mod data;
pub mod ml;
pub mod processing;
pub mod storage;
pub mod visualization;
pub mod text_formatting;
pub mod embodiment;
pub mod optimization;
pub mod cache;
pub mod subject_generator;
pub mod visual_subject_generator;
pub mod generators;
pub mod inference_engine;

// AI & API integration
pub mod ai;
pub mod metrics;
pub mod monitoring;
pub mod auth;

// WebTransport server
#[cfg(feature = "transport")]
pub mod transport;

// Multi-language coding agents
pub mod agents;

// Consciousness simulation
pub mod consciousness;

// RAG
#[cfg(feature = "rag")]
pub mod rag;

// Benchmark suite
pub mod benchmarks;

// ASI Core
#[cfg(not(target_arch = "wasm32"))]
pub mod asi;

// Backward compatibility - compression is now in data
pub use data::compression;
// Backward compatibility - these are now in core::sacred_geometry
pub use core::sacred_geometry::flux_matrix;
pub use core::sacred_geometry::change_dot;
pub use core::sacred_geometry::angle;
// Backward compatibility - now in core::sacred_geometry
pub use core::sacred_geometry::geometric_inference;

// Backward compatibility - AI modules now in ai/
#[cfg(not(target_arch = "wasm32"))]
pub use ai::consensus as ai_consensus;
#[cfg(not(target_arch = "wasm32"))]
pub use ai::orchestrator;
#[cfg(not(target_arch = "wasm32"))]
pub use ai::consensus::{query_ollama, OllamaConfig, call_multiple_models};
#[cfg(not(target_arch = "wasm32"))]
pub use ai::integration as ai_integration;
#[cfg(not(target_arch = "wasm32"))]
pub use ai::api;
#[cfg(not(target_arch = "wasm32"))]
pub use ai::flux_reasoning;

// Backward compatibility - models is now in data
pub use data::models;

// Curated subject definitions with semantic associations
pub mod subject_definitions;

pub mod dynamic_color_flux;
#[cfg(feature = "voice")]
pub mod voice_pipeline;
// Backward compatibility - lock_free_flux is now in processing
pub use processing::lock_free_flux;
#[cfg(target_arch = "wasm32")]
pub mod epic_wasm;
// Backward compatibility - runtime is now in processing
#[cfg(not(target_arch = "wasm32"))]
pub use processing::runtime;
// Backward compatibility - vector_search is now in data
pub use data::vector_search;
#[cfg(all(feature = "bevy_support", not(target_arch = "wasm32")))]
pub mod flux_mesh;
#[cfg(all(feature = "bevy_support", not(target_arch = "wasm32")))]
pub mod beam_renderer;
// Backward compatibility - now in core
pub use core::normalization;
// Backward compatibility - confidence_scoring is now in processing
pub use processing::confidence_scoring;

// Backward compatibility - confidence_lake is now in storage
#[cfg(feature = "lake")]
pub use storage::confidence_lake;

// Backward compatibility - spatial_database is now in storage
#[cfg(not(target_arch = "wasm32"))]
pub use storage::spatial_database;

// Backward compatibility - training is now ml::training
pub use ml::training as training_module;

// Backward compatibility - hallucinations now in ml/
pub use ml::hallucinations;

pub use error::{Result, SpatialVortexError};
pub use data::models::{BeamTensor, BeadTensor, StoredFluxMatrix, ELPTensor};
pub use hallucinations::{SignalSubspace, HallucinationDetector, VortexContextPreserver};

// Formal verification
#[cfg(feature = "formal-verification")]
pub use core::formal_logic::{FormalLogicEngine, Axiom, Theorem, VerificationResult};

// WASM entry point for 3D visualization
#[cfg(all(target_arch = "wasm32", feature = "bevy_support"))]
pub mod wasm_entry {
    pub use crate::processing::lock_free_flux::LockFreeFluxMatrix;
    pub use crate::data::models::{FluxNode, NodeAttributes, NodeState, NodeDynamics, SemanticIndex, ELPTensor};
    pub use bevy::prelude::*;
    pub use std::collections::HashMap;
    pub use std::f32::consts::PI;
    pub use wasm_bindgen::prelude::*;
    
    #[derive(Resource)]
    struct FluxMatrixResource(LockFreeFluxMatrix);
    
    // DISABLED: Using epic_wasm instead
    // #[wasm_bindgen(start)]
    pub fn run_old() {
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
            let mut parameters = HashMap::new();
            parameters.insert("ethos".to_string(), e);
            parameters.insert("logos".to_string(), l);
            parameters.insert("pathos".to_string(), p);
            
            let node = FluxNode {
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
            };
            matrix.insert(node);
        }
        
        // Create Bevy app with WASM settings
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
        for (start, end) in [(3, 6), (6, 9), (9, 3)] {
            spawn_line(&mut commands, &mut meshes, &mut materials,
                get_position(start, radius), get_position(end, radius), Color::WHITE);
        }
        
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
                let ethos = *node.node.attributes.parameters.get("ethos").unwrap_or(&0.5) as f32;
                let logos = *node.node.attributes.parameters.get("logos").unwrap_or(&0.5) as f32;
                let pathos = *node.node.attributes.parameters.get("pathos").unwrap_or(&0.5) as f32;
                let color = get_dominant_color(ethos, logos, pathos);
                
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
    
    fn get_position(pos: u8, radius: f32) -> Vec3 {
        let angle = ((9 - pos) as f32 / 9.0) * 2.0 * PI - PI / 2.0;
        Vec3::new(
            angle.cos() * radius,
            angle.sin() * radius,
            0.0
        )
    }
    
    fn get_dominant_color(e: f32, l: f32, p: f32) -> Color {
        if e > l && e > p {
            Color::srgb(1.0, 0.3, 0.3) // Red - Ethos
        } else if l > p {
            Color::srgb(0.3, 0.3, 1.0) // Blue - Logos
        } else {
            Color::srgb(0.3, 1.0, 0.3) // Green - Pathos
        }
    }
    
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
    
    fn rotate_camera(time: Res<Time>, mut query: Query<&mut Transform, With<Camera3d>>) {
        for mut transform in query.iter_mut() {
            transform.rotate_around(Vec3::ZERO, Quat::from_rotation_y(time.delta_secs() * 0.2));
        }
    }
}
