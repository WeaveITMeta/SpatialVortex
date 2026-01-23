/// Beam Renderer: Visualize words as colored light beams flowing through the flux matrix
/// Uses Bevy's line rendering for efficient beam visualization
/// Colors represent ELP channels: Red=Pathos, Green=Logos, Blue=Ethos

use bevy::prelude::*;
use crate::models::BeamTensor;
use crate::flux_mesh::{FluxGeometry, WordBeam, FluxNode};

/// Resource to manage beam rendering configuration
#[derive(Debug, Clone)]
pub struct BeamRenderConfig {
    pub beam_width_base: f32,
    pub beam_width_confidence_scale: f32,
    pub beam_glow_intensity: f32,
    pub beam_speed: f32,
    pub trail_length: usize,
    pub particle_count: u32,
}

impl Default for BeamRenderConfig {
    fn default() -> Self {
        Self {
            beam_width_base: 0.05,
            beam_width_confidence_scale: 0.1,
            beam_glow_intensity: 2.0,
            beam_speed: 1.0,
            trail_length: 10,
            particle_count: 20,
        }
    }
}

/// Component for beam trail visualization
#[derive(Component, Debug, Clone)]
pub struct BeamTrail {
    pub positions: Vec<Vec3>,
    pub colors: Vec<Color>,
    pub max_length: usize,
}

/// Component for sacred intersection effects
#[derive(Component, Debug, Clone)]
pub struct IntersectionEffect {
    pub position: Vec3,
    pub color: Color,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub effect_type: IntersectionEffectType,
}

#[derive(Debug, Clone)]
pub enum IntersectionEffectType {
    PositiveReinforcement,  // Position 3 - Green burst
    DeepAnalysis,          // Position 6 - Red ripple
    DivineMoment,          // Position 9 - Blue ascension
}

/// Spawn a word beam in the scene
pub fn spawn_word_beam(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    beam_tensor: &BeamTensor,
    geometry: &FluxGeometry,
) -> Entity {
    // Calculate beam color from ELP channels
    let beam_color = Color::srgb(
        beam_tensor.pathos / 9.0,    // Red: Emotion
        beam_tensor.logos / 9.0,     // Green: Logic
        beam_tensor.ethos / 9.0,     // Blue: Ethics
    );
    
    // Calculate beam path with curvature based on tensor properties
    let path = geometry.calculate_beam_path(
        beam_tensor.position,
        (beam_tensor.position + 1) % 10,  // Next position in flux
        beam_tensor.curviness_signed,
    );
    
    // Create beam mesh (cylinder that will be stretched along path)
    let beam_mesh = meshes.add(Capsule3d::new(0.02 + beam_tensor.confidence * 0.03, 0.1));
    
    // Emissive material for glowing effect
    let beam_material = materials.add(StandardMaterial {
        base_color: beam_color,
        emissive: LinearRgba::from(beam_color) * beam_tensor.confidence * 2.0,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    
    // Spawn the beam entity
    let beam_entity = commands
        .spawn(PbrBundle {
            mesh: beam_mesh,
            material: beam_material,
            transform: Transform::from_translation(path[0]),
            ..default()
        })
        .insert(WordBeam {
            word: beam_tensor.word.clone(),
            current_position: beam_tensor.position,
            target_position: (beam_tensor.position + 1) % 10,
            progress: 0.0,
            color: beam_color,
            intensity: beam_tensor.confidence,
            path: path.clone(),
        })
        .insert(BeamTrail {
            positions: vec![path[0]],
            colors: vec![beam_color],
            max_length: 10,
        })
        .id();
    
    // Spawn text label for the word
    spawn_word_label(commands, &beam_tensor.word, path[0], beam_color);
    
    beam_entity
}

/// Spawn a text label that follows the beam
fn spawn_word_label(
    commands: &mut Commands,
    word: &str,
    position: Vec3,
    color: Color,
) {
    // Note: Bevy 0.8 doesn't have built-in 3D text, so we'll use a sprite
    // In production, you'd want to use bevy_text_mesh or similar
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color,
                custom_size: Some(Vec2::new(100.0, 20.0)),
                ..default()
            },
            transform: Transform::from_translation(position + Vec3::Y * 0.5),
            ..default()
        })
        .insert(Name::new(word.to_string()));
}

/// System to update beam positions along their paths
pub fn update_beam_flow(
    time: Res<Time>,
    config: Res<BeamRenderConfig>,
    mut query: Query<(&mut WordBeam, &mut Transform, &mut BeamTrail)>,
) {
    for (mut beam, mut transform, mut trail) in query.iter_mut() {
        // Update progress along path
        beam.progress += time.delta_seconds() * config.beam_speed / beam.path.len() as f32;
        
        // Loop back to start if we've completed the path
        if beam.progress >= 1.0 {
            beam.progress = 0.0;
            beam.current_position = beam.target_position;
            beam.target_position = (beam.target_position + 1) % 10;
            
            // Recalculate path for new segment
            // In production, this would query the FluxGeometry resource
        }
        
        // Interpolate position along path
        let path_index = (beam.progress * (beam.path.len() - 1) as f32) as usize;
        let path_index = path_index.min(beam.path.len() - 1);
        
        let current_pos = beam.path[path_index];
        transform.translation = current_pos;
        
        // Update trail
        trail.positions.push(current_pos);
        trail.colors.push(beam.color);
        
        // Limit trail length
        if trail.positions.len() > trail.max_length {
            trail.positions.remove(0);
            trail.colors.remove(0);
        }
        
        // Add wobble based on pathos (emotion)
        let wobble = (time.seconds_since_startup() as f32 * 5.0).sin() * beam.intensity * 0.1;
        transform.translation.x += wobble;
    }
}

/// System to trigger effects at sacred intersections
pub fn process_sacred_intersections(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    beams: Query<&WordBeam>,
    nodes: Query<(&FluxNode, &Transform)>,
    geometry: Res<FluxGeometry>,
) {
    // Check each beam against sacred positions
    for beam in beams.iter() {
        if !geometry.is_sacred_position(beam.current_position) {
            continue;
        }
        
        // Find the corresponding node
        for (node, node_transform) in nodes.iter() {
            if node.position != beam.current_position {
                continue;
            }
            
            // Trigger effect based on sacred position type
            match beam.current_position {
                3 => {
                    // Good/Easy - Green burst effect
                    spawn_intersection_effect(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        node_transform.translation,
                        Color::GREEN,
                        IntersectionEffectType::PositiveReinforcement,
                    );
                }
                6 => {
                    // Bad/Hard - Red ripple effect
                    spawn_intersection_effect(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        node_transform.translation,
                        Color::RED,
                        IntersectionEffectType::DeepAnalysis,
                    );
                }
                9 => {
                    // Divine/Righteous - Blue ascension effect
                    spawn_intersection_effect(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        node_transform.translation,
                        Color::CYAN,
                        IntersectionEffectType::DivineMoment,
                    );
                }
                _ => {}
            }
        }
    }
}

/// Spawn a visual effect at an intersection
fn spawn_intersection_effect(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    color: Color,
    effect_type: IntersectionEffectType,
) {
    // Create effect mesh (expanding sphere)
    let effect_mesh = meshes.add(Sphere::new(0.1).mesh().ico(4).unwrap());
    
    // Semi-transparent material - create color with alpha manually
    let transparent_color = Color::rgba(color.r(), color.g(), color.b(), 0.5);
    let effect_material = materials.add(StandardMaterial {
        base_color: transparent_color,
        emissive: LinearRgba::from(color) * 2.0,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    
    commands
        .spawn(PbrBundle {
            mesh: effect_mesh,
            material: effect_material,
            transform: Transform::from_translation(position),
            ..default()
        })
        .insert(IntersectionEffect {
            position,
            color,
            lifetime: 0.0,
            max_lifetime: 1.0,
            effect_type,
        });
}

/// System to animate intersection effects
pub fn animate_intersection_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut IntersectionEffect)>,
) {
    for (entity, mut transform, mut effect) in query.iter_mut() {
        effect.lifetime += time.delta_seconds();
        
        // Calculate progress (0 to 1)
        let progress = effect.lifetime / effect.max_lifetime;
        
        // Animate based on effect type
        match effect.effect_type {
            IntersectionEffectType::PositiveReinforcement => {
                // Expand outward quickly
                transform.scale = Vec3::splat(1.0 + progress * 3.0);
            }
            IntersectionEffectType::DeepAnalysis => {
                // Ripple effect
                transform.scale = Vec3::splat(1.0 + (progress * 10.0).sin() * 0.5);
            }
            IntersectionEffectType::DivineMoment => {
                // Ascend upward
                transform.translation.y += time.delta_seconds() * 2.0;
                transform.scale = Vec3::splat(1.0 + progress);
            }
        }
        
        // Despawn when lifetime expires
        if effect.lifetime >= effect.max_lifetime {
            commands.entity(entity).despawn();
        }
    }
}

/// System to render beam trails
/// Note: Gizmos not available in Bevy 0.8, using placeholder
pub fn render_beam_trails(
    _query: Query<&BeamTrail>,
) {
    // TODO: Implement trail rendering with LineList mesh or Debug Lines
    // Gizmos API was added in Bevy 0.9+
    // For now, trails are stored but not rendered
}

/// Bundle all beam rendering systems into a plugin
pub struct BeamRenderPlugin;

impl Plugin for BeamRenderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BeamRenderConfig::default())
            .add_system(update_beam_flow)
            .add_system(process_sacred_intersections)
            .add_system(animate_intersection_effects)
            .add_system(render_beam_trails);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_beam_render_config() {
        let config = BeamRenderConfig::default();
        assert!(config.beam_width_base > 0.0);
        assert!(config.beam_speed > 0.0);
    }
    
    #[test]
    fn test_intersection_effect_types() {
        let effect = IntersectionEffect {
            position: Vec3::ZERO,
            color: Color::WHITE,
            lifetime: 0.0,
            max_lifetime: 1.0,
            effect_type: IntersectionEffectType::DivineMoment,
        };
        
        matches!(effect.effect_type, IntersectionEffectType::DivineMoment);
    }
}
