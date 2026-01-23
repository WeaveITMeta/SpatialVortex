/// Flux Matrix Visualization: Interactive 3D view of the AGI consciousness engine
/// Shows words as colored light beams flowing through sacred geometry

use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseWheel};

use spatial_vortex::flux_mesh::{
    FluxGeometry,
    spawn_flux_structure, animate_sacred_nodes,
};

use spatial_vortex::beam_renderer::{
    BeamRenderConfig,
    spawn_word_beam, update_beam_flow,
    process_sacred_intersections, animate_intersection_effects,
    render_beam_trails,
};

use spatial_vortex::models::BeamTensor;

/// Main entry point for flux matrix visualization
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "SpatialVortex - Flux Matrix AGI Visualization".to_string(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(FluxMatrixVisualizationPlugin)
        .run();
}

/// Plugin containing all flux matrix visualization systems
pub struct FluxMatrixVisualizationPlugin;

impl Plugin for FluxMatrixVisualizationPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .insert_resource(BeamRenderConfig::default())
            .insert_resource(FluxGeometry::new(2.0))
            .insert_resource(DemoState::default())
            
            // Startup systems
            .add_systems(Startup, (setup_scene, spawn_flux_structure))
            
            // Update systems
            .add_systems(Update, (
                animate_sacred_nodes,
                update_beam_flow,
                process_sacred_intersections,
                animate_intersection_effects,
                render_beam_trails,
                demo_word_spawner,
                camera_controls,
                keyboard_input,
            ));
    }
}

/// Resource to manage demo state
#[derive(Default)]
struct DemoState {
    spawn_timer: f32,
    word_index: usize,
    auto_spawn: bool,
    camera_follow: bool,
}

/// Setup the 3D scene with camera and lighting
fn setup_scene(
    mut commands: Commands,
) {
    // Camera with interactive controls
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 8.0, 15.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(CameraController::default());
    
    // Ambient lighting
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.2, 0.2, 0.3),
        brightness: 0.5,
    });
    
    // Point lights at sacred positions
    // Position 3 - Green light (Good/Easy)
    commands
        .spawn(PointLightBundle {
            point_light: PointLight {
                intensity: 1500.0,
                color: Color::srgb(0.0, 1.0, 0.5),
                radius: 10.0,
                ..default()
            },
            transform: Transform::from_xyz(2.0, -2.0, 2.0),
            ..default()
        });
    
    // Position 6 - Red light (Bad/Hard)
    commands
        .spawn(PointLightBundle {
            point_light: PointLight {
                intensity: 1500.0,
                color: Color::srgb(1.0, 0.2, 0.2),
                radius: 10.0,
                ..default()
            },
            transform: Transform::from_xyz(-2.0, -2.0, 2.0),
            ..default()
        });
    
    // Position 9 - Blue light (Divine/Righteous)
    commands
        .spawn(PointLightBundle {
            point_light: PointLight {
                intensity: 2000.0,
                color: Color::srgb(0.2, 0.5, 1.0),
                radius: 12.0,
                ..default()
            },
            transform: Transform::from_xyz(0.0, 2.0, 2.0),
            ..default()
        });
    
    // UI instructions
    spawn_ui_instructions(&mut commands);
}

/// Spawn UI text with controls
fn spawn_ui_instructions(commands: &mut Commands) {
    commands
        .spawn(TextBundle {
        text: Text::from_sections([
            TextSection {
                value: "SpatialVortex Flux Matrix AGI\n".to_string(),
                style: TextStyle {
                    font_size: 24.0,
                    color: Color::WHITE,
                    ..default()
                },
            },
            TextSection {
                value: "Controls:\n".to_string(),
                style: TextStyle {
                    font_size: 18.0,
                    color: Color::CYAN,
                    ..default()
                },
            },
            TextSection {
                value: concat!(
                    "Space: Spawn test word\n",
                    "A: Toggle auto-spawn\n",
                    "F: Toggle camera follow\n",
                    "1-9: Focus on flux position\n",
                    "Mouse: Rotate camera\n",
                    "Scroll: Zoom\n",
                    "\n",
                    "Sacred Intersections:\n",
                    "3 (Green): Good/Easy\n",
                    "6 (Red): Bad/Hard\n",
                    "9 (Blue): Divine/Righteous"
                ).to_string(),
                style: TextStyle {
                    font_size: 14.0,
                    color: Color::GRAY,
                    ..default()
                },
            },
        ]),
        style: Style {
            position_type: PositionType::Absolute,
            left: Val::Px(10.0),
            top: Val::Px(10.0),
            ..default()
        },
        ..default()
    });
}

/// Demo system that spawns test words periodically
fn demo_word_spawner(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut state: ResMut<DemoState>,
    time: Res<Time>,
    geometry: Res<FluxGeometry>,
    keyboard: Res<Input<KeyCode>>,
) {
    // Manual spawn on spacebar
    if keyboard.just_pressed(KeyCode::Space) {
        spawn_demo_word(&mut commands, &mut meshes, &mut materials, &geometry, &mut state);
    }
    
    // Auto-spawn if enabled
    if state.auto_spawn {
        state.spawn_timer += time.delta_seconds();
        if state.spawn_timer >= 2.0 {
            state.spawn_timer = 0.0;
            spawn_demo_word(&mut commands, &mut meshes, &mut materials, &geometry, &mut state);
        }
    }
}

/// Spawn a demo word with random properties
fn spawn_demo_word(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    geometry: &FluxGeometry,
    state: &mut DemoState,
) {
    // Demo words to cycle through
    let demo_words = vec![
        ("consciousness", 9.0, 8.0, 7.0),  // High ethos/logos
        ("emotion", 3.0, 2.0, 9.0),        // High pathos
        ("logic", 2.0, 9.0, 3.0),          // High logos
        ("divine", 9.0, 7.0, 8.0),         // Balanced high
        ("struggle", 5.0, 6.0, 7.0),       // Challenge
        ("easy", 3.0, 4.0, 6.0),           // Simple
        ("complex", 7.0, 8.0, 5.0),        // Analytical
        ("love", 2.0, 3.0, 9.0),           // Emotional
        ("truth", 8.0, 9.0, 6.0),          // Logical divine
    ];
    
    let (word, ethos, logos, pathos) = demo_words[state.word_index % demo_words.len()];
    state.word_index += 1;
    
    // Create beam tensor for the word
    let mut beam = BeamTensor::default();
    beam.word = word.to_string();
    beam.ethos = ethos;
    beam.logos = logos;
    beam.pathos = pathos;
    beam.confidence = 0.5 + (ethos + logos + pathos) / 27.0;
    beam.position = (state.word_index as u8) % 10;
    beam.curviness_signed = (pathos - logos) / 9.0;
    
    // Spawn the word beam
    spawn_word_beam(commands, meshes, materials, &beam, &geometry);
}

/// Camera controller component
#[derive(Component, Default)]
struct CameraController {
    pub rotation: Vec2,
    pub distance: f32,
    pub focus_target: Vec3,
}

/// System for camera controls
fn camera_controls(
    mut query: Query<(&mut Transform, &mut CameraController), With<Camera>>,
    mouse: Res<Input<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut scroll: EventReader<MouseWheel>,
    keyboard: Res<Input<KeyCode>>,
    geometry: Res<FluxGeometry>,
    time: Res<Time>,
) {
    for (mut transform, mut controller) in query.iter_mut() {
        // Initialize controller values
        if controller.distance == 0.0 {
            controller.distance = 15.0;
        }
        
        // Mouse rotation
        if mouse.pressed(MouseButton::Left) || mouse.pressed(MouseButton::Right) {
            for event in mouse_motion.iter() {
                controller.rotation.x -= event.delta.y * 0.01;
                controller.rotation.y -= event.delta.x * 0.01;
                controller.rotation.x = controller.rotation.x.clamp(-1.5, 1.5);
            }
        }
        
        // Scroll zoom
        for event in scroll.iter() {
            controller.distance -= event.y * 2.0;
            controller.distance = controller.distance.clamp(5.0, 50.0);
        }
        
        // Number keys focus on flux positions
        let key_map = [
            (KeyCode::Key0, KeyCode::Numpad0, 0),
            (KeyCode::Key1, KeyCode::Numpad1, 1),
            (KeyCode::Key2, KeyCode::Numpad2, 2),
            (KeyCode::Key3, KeyCode::Numpad3, 3),
            (KeyCode::Key4, KeyCode::Numpad4, 4),
            (KeyCode::Key5, KeyCode::Numpad5, 5),
            (KeyCode::Key6, KeyCode::Numpad6, 6),
            (KeyCode::Key7, KeyCode::Numpad7, 7),
            (KeyCode::Key8, KeyCode::Numpad8, 8),
            (KeyCode::Key9, KeyCode::Numpad9, 9),
        ];
        
        for (key, numpad_key, position_num) in &key_map {
            if keyboard.just_pressed(*key) || keyboard.just_pressed(*numpad_key) {
                let position = geometry.get_position(*position_num);
                controller.focus_target = position;
            }
        }
        
        // Smooth camera movement
        let target = controller.focus_target;
        let spherical = Vec3::new(
            controller.distance * controller.rotation.x.cos() * controller.rotation.y.cos(),
            controller.distance * controller.rotation.x.sin(),
            controller.distance * controller.rotation.x.cos() * controller.rotation.y.sin(),
        );
        
        let desired_pos = target + spherical;
        let smoothing = 5.0 * time.delta_seconds();
        transform.translation = transform.translation.lerp(desired_pos, smoothing);
        transform.look_at(target, Vec3::Y);
    }
}

/// System for keyboard input handling
fn keyboard_input(
    keyboard: Res<Input<KeyCode>>,
    mut state: ResMut<DemoState>,
) {
    // Toggle auto-spawn
    if keyboard.just_pressed(KeyCode::A) {
        state.auto_spawn = !state.auto_spawn;
        println!("Auto-spawn: {}", state.auto_spawn);
    }
    
    // Toggle camera follow
    if keyboard.just_pressed(KeyCode::F) {
        state.camera_follow = !state.camera_follow;
        println!("Camera follow: {}", state.camera_follow);
    }
    
    // Reset view
    if keyboard.just_pressed(KeyCode::R) {
        // Reset handled in camera_controls
        println!("Reset view");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_demo_state_initialization() {
        let state = DemoState::default();
        assert_eq!(state.word_index, 0);
        assert!(!state.auto_spawn);
        assert!(!state.camera_follow);
    }
}
