//! 3D Voice Visualization with Bevy
//!
//! Real-time 3D visualization of voice pipeline data flowing through
//! the sacred geometry flux matrix.

use bevy::prelude::*;
use bevy::window::WindowMode;
use std::sync::Arc;
use parking_lot::RwLock;
use crate::voice_pipeline::SpectralFeatures;
use crate::models::{ELPTensor, BeadTensor};

/// Voice visualization plugin for Bevy
pub struct VoiceVisualizationPlugin;

impl Plugin for VoiceVisualizationPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<VoiceData>()
            .init_resource::<VisualizationSettings>()
            .add_systems(Startup, setup_3d_scene)
            .add_systems(Update, (
                update_voice_spectrum,
                update_flux_positions,
                animate_sacred_geometry,
                update_elp_channels,
                rotate_camera,
            ));
    }
}

/// Shared voice data from the pipeline
#[derive(Resource, Default)]
pub struct VoiceData {
    pub spectrum: Arc<RwLock<Vec<f32>>>,
    pub features: Arc<RwLock<Option<SpectralFeatures>>>,
    pub elp_tensor: Arc<RwLock<Option<ELPTensor>>>,
    pub bead_tensor: Arc<RwLock<Option<BeadTensor>>>,
    pub confidence: Arc<RwLock<f32>>,
}

/// Visualization settings
#[derive(Resource)]
pub struct VisualizationSettings {
    pub show_spectrum: bool,
    pub show_flux_matrix: bool,
    pub show_sacred_geometry: bool,
    pub show_elp_channels: bool,
    pub animation_speed: f32,
}

impl Default for VisualizationSettings {
    fn default() -> Self {
        Self {
            show_spectrum: true,
            show_flux_matrix: true,
            show_sacred_geometry: true,
            show_elp_channels: true,
            animation_speed: 1.0,
        }
    }
}

/// Component for spectrum bars
#[derive(Component)]
struct SpectrumBar {
    frequency_bin: usize,
}

/// Component for flux position nodes
#[derive(Component)]
struct FluxNode {
    position: u8,
    is_sacred: bool,
}

/// Component for ELP channel visualizer
#[derive(Component)]
struct ELPChannel {
    channel: ELPChannelType,
}

#[derive(Clone, Copy)]
enum ELPChannelType {
    Ethos,
    Logos,
    Pathos,
}

/// Setup the 3D scene
fn setup_3d_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    voice_data: Res<VoiceData>,
) {
    // Camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 15.0, 20.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    
    // Lights
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: false,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -std::f32::consts::PI / 4.0,
            std::f32::consts::PI / 4.0,
            0.0,
        )),
        ..default()
    });
    
    // Floor plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(20.0))),
        material: materials.add(Color::rgb(0.1, 0.1, 0.15)),
        transform: Transform::from_xyz(0.0, -2.0, 0.0),
        ..default()
    });
    
    // Create spectrum bars (64 frequency bins)
    let spectrum_width = 16.0;
    let bar_width = spectrum_width / 64.0;
    
    for i in 0..64 {
        let x = -spectrum_width / 2.0 + i as f32 * bar_width + bar_width / 2.0;
        
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Cuboid::new(bar_width * 0.8, 0.1, 0.5)),
                material: materials.add(Color::hsl(
                    240.0 + (i as f32 / 64.0) * 120.0,
                    0.7,
                    0.5,
                )),
                transform: Transform::from_xyz(x, 0.0, -8.0),
                ..default()
            },
            SpectrumBar { frequency_bin: i },
        ));
    }
    
    // Create flux matrix nodes (0-9)
    let radius = 6.0;
    for i in 0..10 {
        let angle = std::f32::consts::PI / 2.0 - (i as f32) * 2.0 * std::f32::consts::PI / 10.0;
        let x = radius * angle.cos();
        let z = radius * angle.sin();
        let is_sacred = [3, 6, 9].contains(&i);
        
        let color = if is_sacred {
            Color::rgb(1.0, 0.843, 0.0) // Gold for sacred positions
        } else {
            Color::rgb(0.3, 0.3, 0.8) // Blue for regular positions
        };
        
        let size = if is_sacred { 0.8 } else { 0.5 };
        
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Sphere::new(size)),
                material: materials.add(color),
                transform: Transform::from_xyz(x, 1.0, z),
                ..default()
            },
            FluxNode {
                position: i,
                is_sacred,
            },
        ));
        
        // Add position label
        commands.spawn(Text2dBundle {
            text: Text::from_section(
                i.to_string(),
                TextStyle {
                    font_size: 20.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            transform: Transform::from_xyz(x, 2.5, z),
            ..default()
        });
    }
    
    // Create sacred triangle lines
    let sacred_positions = [(3, 6), (6, 9), (9, 3)];
    for (from, to) in sacred_positions {
        let from_angle = std::f32::consts::PI / 2.0 - (from as f32) * 2.0 * std::f32::consts::PI / 10.0;
        let to_angle = std::f32::consts::PI / 2.0 - (to as f32) * 2.0 * std::f32::consts::PI / 10.0;
        
        let from_pos = Vec3::new(
            radius * from_angle.cos(),
            1.0,
            radius * from_angle.sin(),
        );
        let to_pos = Vec3::new(
            radius * to_angle.cos(),
            1.0,
            radius * to_angle.sin(),
        );
        
        let midpoint = (from_pos + to_pos) / 2.0;
        let direction = to_pos - from_pos;
        let length = direction.length();
        
        commands.spawn(PbrBundle {
            mesh: meshes.add(Cuboid::new(length, 0.1, 0.1)),
            material: materials.add(Color::rgb(0.9, 0.7, 0.0)),
            transform: Transform::from_translation(midpoint)
                .looking_at(to_pos, Vec3::Y),
            ..default()
        });
    }
    
    // Create ELP channel visualizers
    let elp_colors = [
        (ELPChannelType::Ethos, Color::rgb(1.0, 0.3, 0.3)),   // Red
        (ELPChannelType::Logos, Color::rgb(0.3, 0.3, 1.0)),   // Blue
        (ELPChannelType::Pathos, Color::rgb(0.3, 1.0, 0.3)),  // Green
    ];
    
    for (i, (channel, color)) in elp_colors.iter().enumerate() {
        let x = -4.0 + i as f32 * 4.0;
        
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Cylinder::new(0.3, 0.1)),
                material: materials.add(*color),
                transform: Transform::from_xyz(x, 0.0, 4.0),
                ..default()
            },
            ELPChannel { channel: *channel },
        ));
    }
}

/// Update spectrum visualization from voice data
fn update_voice_spectrum(
    voice_data: Res<VoiceData>,
    mut query: Query<(&mut Transform, &SpectrumBar)>,
    settings: Res<VisualizationSettings>,
) {
    if !settings.show_spectrum {
        return;
    }
    
    let spectrum = voice_data.spectrum.read();
    
    for (mut transform, bar) in query.iter_mut() {
        if bar.frequency_bin < spectrum.len() {
            let magnitude = spectrum[bar.frequency_bin];
            let height = magnitude.min(10.0).max(0.1);
            transform.scale.y = height;
            transform.translation.y = height / 2.0;
        }
    }
}

/// Update flux node positions based on voice features
fn update_flux_positions(
    voice_data: Res<VoiceData>,
    mut query: Query<(&mut Transform, &FluxNode)>,
    time: Res<Time>,
    settings: Res<VisualizationSettings>,
) {
    if !settings.show_flux_matrix {
        return;
    }
    
    let confidence = *voice_data.confidence.read();
    
    for (mut transform, node) in query.iter_mut() {
        // Pulse sacred nodes based on signal strength
        if node.is_sacred {
            let pulse = (time.elapsed_seconds() * 2.0).sin() * 0.2 + 1.0;
            let scale = 0.8 + confidence * 0.4 * pulse;
            transform.scale = Vec3::splat(scale);
        } else {
            // Regular nodes vibrate slightly
            let vibration = (time.elapsed_seconds() * 5.0 + node.position as f32).sin() * 0.05;
            transform.translation.y = 1.0 + vibration * confidence;
        }
    }
}

/// Animate sacred geometry based on voice
fn animate_sacred_geometry(
    voice_data: Res<VoiceData>,
    mut gizmos: Gizmos,
    time: Res<Time>,
    settings: Res<VisualizationSettings>,
) {
    if !settings.show_sacred_geometry {
        return;
    }
    
    let features = voice_data.features.read();
    
    if let Some(features) = features.as_ref() {
        // Draw energy flow based on pitch
        let flow_speed = features.pitch / 100.0;
        let t = (time.elapsed_seconds() * flow_speed) % 1.0;
        
        // Vortex flow pattern: 1→2→4→8→7→5→1
        let flow_pattern = [1, 2, 4, 8, 7, 5, 1];
        let radius = 6.0;
        
        for i in 0..flow_pattern.len() - 1 {
            let from = flow_pattern[i];
            let to = flow_pattern[i + 1];
            
            let from_angle = std::f32::consts::PI / 2.0 - (from as f32) * 2.0 * std::f32::consts::PI / 10.0;
            let to_angle = std::f32::consts::PI / 2.0 - (to as f32) * 2.0 * std::f32::consts::PI / 10.0;
            
            let from_pos = Vec3::new(
                radius * from_angle.cos(),
                1.5,
                radius * from_angle.sin(),
            );
            let to_pos = Vec3::new(
                radius * to_angle.cos(),
                1.5,
                radius * to_angle.sin(),
            );
            
            // Interpolate position for flowing effect
            let current_pos = from_pos.lerp(to_pos, t);
            
            gizmos.sphere(
                current_pos,
                Quat::IDENTITY,
                0.2,
                Color::rgba(0.5, 0.8, 1.0, 0.6),
            );
        }
        
        // Draw spectral centroid indicator
        let centroid_height = features.spectral_centroid / 1000.0;
        gizmos.line(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, centroid_height.min(10.0), 0.0),
            Color::rgb(1.0, 0.5, 0.0),
        );
    }
}

/// Update ELP channel visualizers
fn update_elp_channels(
    voice_data: Res<VoiceData>,
    mut query: Query<(&mut Transform, &ELPChannel)>,
    settings: Res<VisualizationSettings>,
) {
    if !settings.show_elp_channels {
        return;
    }
    
    let elp = voice_data.elp_tensor.read();
    
    if let Some(elp) = elp.as_ref() {
        let total = elp.ethos + elp.logos + elp.pathos;
        let normalized_ethos = elp.ethos / total;
        let normalized_logos = elp.logos / total;
        let normalized_pathos = elp.pathos / total;
        
        for (mut transform, channel) in query.iter_mut() {
            let height = match channel.channel {
                ELPChannelType::Ethos => normalized_ethos * 5.0,
                ELPChannelType::Logos => normalized_logos * 5.0,
                ELPChannelType::Pathos => normalized_pathos * 5.0,
            };
            
            transform.scale.y = height.max(0.1);
            transform.translation.y = height / 2.0;
        }
    }
}

/// Rotate camera around the scene
fn rotate_camera(
    mut query: Query<&mut Transform, With<Camera3d>>,
    time: Res<Time>,
    settings: Res<VisualizationSettings>,
) {
    for mut transform in query.iter_mut() {
        let angle = time.elapsed_seconds() * 0.1 * settings.animation_speed;
        let radius = 25.0;
        let height = 15.0;
        
        transform.translation.x = radius * angle.cos();
        transform.translation.y = height;
        transform.translation.z = radius * angle.sin();
        
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}

/// Create and run the voice visualization app
pub fn run_voice_visualization(voice_data: VoiceData) {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "SpatialVortex Voice Visualization".to_string(),
                mode: WindowMode::Windowed,
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(VoiceVisualizationPlugin)
        .insert_resource(voice_data)
        .run();
}
