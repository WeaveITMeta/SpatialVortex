use bevy::prelude::*;
use bevy::window::ReceivedCharacter;
use bevy::render::camera::Camera;
#[cfg(not(target_arch = "wasm32"))]
use bevy::render::settings::{Backends, WgpuSettings};
#[cfg(target_arch = "wasm32")]
use console_error_panic_hook;
use spatial_vortex::change_dot::{parse_change_dot, ChangeDotIter, ChangeDotEvent};
use spatial_vortex::flux_matrix::FluxMatrixEngine;

mod camera;
use camera::{CameraController, MouseState};

#[derive(Default)]
struct InputBuffer {
    text: String,
}

#[derive(Component)]
pub struct Selected;

fn compute_net_lambda(
    mut q_hinge: Query<&mut Hinge, With<NodeZero>>,
    q_markers: Query<&Marker>,
) {
    if let Ok(mut hinge) = q_hinge.get_single_mut() {
        let mut total = 0.0f32;
        let mut count = 0usize;
        for m in q_markers.iter() {
            let sign = if m.index > 0 { 1.0 } else if m.index < 0 { -1.0 } else { 0.0 };
            if m.mass > 0.0 {
                total += m.lambda * m.mass * sign;
                count += 1;
            }
        }
        let net = if count > 0 { total / (count as f32) } else { 0.0 };
        let torque_const = 2.0f32; // rads per second scale
        hinge.target_spin = net * torque_const;
    }
}

fn apply_hinge_spin(
    time: Res<Time>,
    mut q_hinge: Query<(&mut Hinge, &GlobalTransform), With<NodeZero>>,
    mut q_markers: Query<&mut Transform, (With<Marker>, Without<NodeZero>)>,
) {
    if let Ok((mut hinge, origin_gt)) = q_hinge.get_single_mut() {
        let dt = time.delta_seconds();
        // critically damped approach to target spin
        let k = 5.0f32;
        hinge.spin_velocity += (hinge.target_spin - hinge.spin_velocity) * k * dt;
        // light friction
        hinge.spin_velocity *= 0.99;

        let angle = hinge.spin_velocity * dt;
        if angle.abs() < 1e-5 { return; }
        let center = origin_gt.translation();
        let rot = Quat::from_rotation_y(angle);
        for mut tf in q_markers.iter_mut() {
            let rel = tf.translation - center;
            tf.translation = center + rot * rel;
        }
    }
}

fn spin_momentum_system(
    time: Res<Time>,
    mut q: Query<(&mut Transform, &mut Velocity), With<Marker>>,
) {
    let dt = time.delta_seconds();
    let damping = 0.98f32;
    for (mut tf, mut vel) in q.iter_mut() {
        let [x, y, z] = vel.linvel;
        tf.translation.x += x * dt;
        tf.translation.y += y * dt;
        tf.translation.z += z * dt;
        vel.linvel = [x * damping, y * damping, z * damping];
    }
}

fn animate_markers(time: Res<Time>, mut q: Query<(&Marker, &mut Transform)>) {
    let dt = time.delta_seconds();
    let t = time.seconds_since_startup() as f32;
    for (marker, mut tf) in q.iter_mut() {
        // Rotate around Y using per-marker speed
        tf.rotate_y(marker.rotation_speed * dt);
        // Gentle pulse around base scale
        let pulse = 1.0 + 0.08 * (t * (0.5 + marker.rotation_speed)).sin();
        tf.scale = Vec3::splat(marker.base_scale * pulse);
    }
}

fn confidence_to_distance(c: f32) -> f32 {
    // Map confidence in [1,9] -> distance in [0.5, 9.0]
    let c = c.clamp(1.0, 9.0);
    0.5 + (c - 1.0) * (9.0 - 0.5) / (9.0 - 1.0)
}

fn apply_confidence_multiplier(
    zero: Query<&GlobalTransform, With<NodeZero>>,
    mut q: Query<(&Confidence, &mut Transform), (With<Marker>, Without<NodeZero>)>,
) {
    if let Ok(gt) = zero.get_single() {
        let anchor_y = gt.translation().y;
        for (conf, mut tf) in q.iter_mut() {
            let target_y = anchor_y + confidence_to_distance(conf.0);
            let alpha = 0.18; // ease factor for breathing
            tf.translation.y = tf.translation.y + (target_y - tf.translation.y) * alpha;
        }
    }
}

fn rebuild_markers_on_space(
    kb: Res<Input<KeyCode>>,
    node: Query<&GlobalTransform, With<NodeZero>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    markers: Query<Entity, With<Marker>>,
) {
    if !kb.just_pressed(KeyCode::Space) { return; }
    // Clear existing markers
    for e in markers.iter() {
        commands.entity(e).despawn_recursive();
    }
    // Rebuild from node-zero origin
    if let Ok(gt) = node.get_single() {
        let origin = gt.translation();
        let point_mesh = meshes.add(create_cube_mesh(0.12));
        let mat_zero = materials.add(StandardMaterial { base_color: Color::WHITE, unlit: true, ..Default::default() });
        let mat_pos = materials.add(StandardMaterial { base_color: Color::GREEN, unlit: true, ..Default::default() });
        let mat_neg = materials.add(StandardMaterial { base_color: Color::RED, unlit: true, ..Default::default() });

        // Respawn zero marker (for completeness)
        commands
            .spawn_bundle(PbrBundle {
                mesh: point_mesh.clone(),
                material: mat_zero.clone(),
                transform: Transform::from_translation(origin),
                ..Default::default()
            })
            .insert(RayHit { index: 0 })
            .insert(Marker { index: 0, digit: 0, rotation_speed: 0.6, base_scale: 0.12, lambda: 0.0, mass: 0.0 })
            .insert(Name::new("zero-0: 0"));

        // Positive (up)
        for i in 1..=3 {
            let y = origin.y + i as f32;
            let digit = FluxMatrixEngine::new().reduce_digits((i as u64) * 2) as u8;
            commands
                .spawn_bundle(PbrBundle {
                    mesh: point_mesh.clone(),
                    material: mat_pos.clone(),
                    transform: Transform::from_translation(Vec3::new(origin.x, y, origin.z)),
                    ..Default::default()
                })
                .insert(RayHit { index: i })
                .insert(Marker { index: i, digit, rotation_speed: 1.0 + 0.2 * (i as f32), base_scale: 0.12, lambda: 0.7 + 0.1 * (i as f32), mass: (digit as f32) / 9.0 })
                .insert(Name::new(format!("pos-{}: {}", i, digit)));
            // Stacks
            for s in 1..=2 {
                let z_off = 0.3 * (s as f32);
                let scale = 0.08 - 0.015 * (s as f32 - 1.0);
                commands
                    .spawn_bundle(PbrBundle {
                        mesh: point_mesh.clone(),
                        material: mat_pos.clone(),
                        transform: Transform::from_translation(Vec3::new(origin.x, y, origin.z + z_off)),
                        ..Default::default()
                    })
                    .insert(RayHit { index: i })
                    .insert(Marker { index: i, digit, rotation_speed: 1.2 + 0.2 * (s as f32), base_scale: scale, lambda: 0.7 + 0.1 * (i as f32), mass: (digit as f32) / 9.0 })
                    .insert(Velocity { linvel: [0.0, 0.0, 0.0] })
                    .insert(Name::new(format!("pos-{}.{}: {}", i, s, digit)));
            }
        }

        // Negative (down)
        for i in 1..=3 {
            let y = origin.y - i as f32;
            let digit = FluxMatrixEngine::new().reduce_digits((i as u64) * 2) as u8;
            commands
                .spawn_bundle(PbrBundle {
                    mesh: point_mesh.clone(),
                    material: mat_neg.clone(),
                    transform: Transform::from_translation(Vec3::new(origin.x, y, origin.z)),
                    ..Default::default()
                })
                .insert(RayHit { index: -i })
                .insert(Marker { index: -i, digit, rotation_speed: 0.8 + 0.2 * (i as f32), base_scale: 0.12, lambda: 0.7 + 0.1 * (i as f32), mass: (digit as f32) / 9.0 })
                .insert(Velocity { linvel: [0.0, 0.0, 0.0] })
                .insert(Name::new(format!("neg-{}: {}", i, digit)));
            // Stacks
            for s in 1..=2 {
                let z_off = 0.3 * (s as f32);
                let scale = 0.08 - 0.015 * (s as f32 - 1.0);
                commands
                    .spawn_bundle(PbrBundle {
                        mesh: point_mesh.clone(),
                        material: mat_neg.clone(),
                        transform: Transform::from_translation(Vec3::new(origin.x, y, origin.z + z_off)),
                        ..Default::default()
                    })
                    .insert(RayHit { index: -i })
                    .insert(Marker { index: -i, digit, rotation_speed: 1.0 + 0.2 * (s as f32), base_scale: scale, lambda: 0.7 + 0.1 * (i as f32), mass: (digit as f32) / 9.0 })
                    .insert(Velocity { linvel: [0.0, 0.0, 0.0] })
                    .insert(Name::new(format!("neg-{}.{}: {}", i, s, digit)));
            }
        }
    }
}

fn create_cube_mesh(size: f32) -> Mesh {
    // Use built-in cube mesh to ensure valid vertex layout for PBR
    Mesh::from(bevy::prelude::shape::Cube { size })
}

fn screen_to_world_ray(cursor_pos: Vec2, window_size: Vec2, camera: &Camera, cam_gt: &GlobalTransform) -> Option<(Vec3, Vec3)> {
    if window_size.x <= 0.0 || window_size.y <= 0.0 { return None; }
    let ndc = Vec3::new(
        (cursor_pos.x / window_size.x) * 2.0 - 1.0,
        1.0 - (cursor_pos.y / window_size.y) * 2.0,
        1.0,
    );
    let proj_inv = camera.projection_matrix().inverse();
    let world_from_ndc = cam_gt.compute_matrix() * proj_inv;
    let near = world_from_ndc * Vec4::new(ndc.x, ndc.y, 0.0, 1.0);
    let near = near.truncate() / near.w;
    let far = world_from_ndc * Vec4::new(ndc.x, ndc.y, 1.0, 1.0);
    let far = far.truncate() / far.w;
    let dir = (far - near).normalize();
    Some((near, dir))
}

fn selection_system(
    buttons: Res<Input<MouseButton>>,
    kb: Res<Input<KeyCode>>,
    windows: Res<Windows>,
    q_cam: Query<(&Camera, &GlobalTransform), With<Camera>>,
    q_pick: Query<(Entity, &GlobalTransform, Option<&Marker>), Or<(With<Marker>, With<NodeZero>)>>,
    mut commands: Commands,
    q_selected: Query<Entity, With<Selected>>,
) {
    if !buttons.just_pressed(MouseButton::Left) { return; }
    let win = if let Some(w) = windows.get_primary() { w } else { return; };
    let cursor = if let Some(p) = win.cursor_position() { p } else { return; };
    let (cam, cam_gt) = if let Ok(v) = q_cam.get_single() { v } else { return; };
    let ray = if let Some(r) = screen_to_world_ray(cursor, Vec2::new(win.width(), win.height()), cam, cam_gt) { r } else { return; };
    if !kb.pressed(KeyCode::LShift) {
        for e in q_selected.iter() { commands.entity(e).remove::<Selected>(); }
    }
    let origin = ray.0; let dir = ray.1;
    let mut best: Option<(Entity, f32)> = None;
    for (e, gt, marker) in q_pick.iter() {
        let p = gt.translation();
        let w = p - origin;
        let d = w.cross(dir).length();
        let radius = marker.map(|m| m.base_scale * 2.0).unwrap_or(0.24);
        if d <= radius {
            let t = w.dot(dir);
            if t > 0.0 && best.map(|(_, bt)| t < bt).unwrap_or(true) { best = Some((e, t)); }
        }
    }
    if let Some((hit, _)) = best {
        commands.entity(hit).insert(Selected);
    }
}

#[derive(Component)]
struct VortexCore {
    iter: ChangeDotIter,
    angle: f32,
    pattern: [u8; 9],
}

#[derive(Component)]
struct NodeZero(u8);

#[derive(Component)]
struct RayHit {
    index: i32, // positive above, negative below, 0 at node-zero
}

#[derive(Component)]
struct Marker {
    index: i32,
    digit: u8,
    rotation_speed: f32,
    base_scale: f32,
    lambda: f32,
    mass: f32,
}

#[derive(Component)]
struct Velocity {
    linvel: [f32; 3],
}

#[derive(Component)]
struct Confidence(pub f32); // 0..9 maps to 0.5..9 studs

#[derive(Component)]
struct Hinge {
    spin_velocity: f32,
    target_spin: f32,
}

fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    let mut app = App::new();

    #[cfg(not(target_arch = "wasm32"))]
    {
        app.insert_resource(WgpuSettings { backends: Some(Backends::VULKAN), ..Default::default() });
    }

    // Bevy 0.8: no PluginGroup builder customization here; use DefaultPlugins for both targets
    app.add_plugins(DefaultPlugins);

    app
        .insert_resource(Msaa { samples: 1 })
        .insert_resource(bevy::pbr::AmbientLight { color: Color::rgb(1.0, 1.0, 1.0), brightness: 0.8 })
        .insert_resource(InputBuffer::default())
        .insert_resource(MouseState::default())
        .add_startup_system(setup)
        .add_system(animate_vortex)
        .add_system(animate_markers)
        .add_system(camera::update_mouse_state)
        .add_system(camera::camera_move_system)
        .add_system(camera::camera_rotate_system)
        .add_system(camera::camera_zoom_system)
        .add_system(camera::camera_pan_system)
        .add_system(selection_system)
        .add_system(camera::camera_focus_system)
        .add_system(camera::camera_reset_system)
        .add_system(camera::cursor_visibility_system);

    #[cfg(not(target_arch = "wasm32"))]
    {
        app.add_system(spin_momentum_system)
            .add_system(compute_net_lambda)
            .add_system(apply_hinge_spin)
            .add_system(apply_confidence_multiplier)
            .add_system(rebuild_markers_on_space);
    }

    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let ctrl = CameraController::default();
    let offset = Quat::from_euler(EulerRot::YXZ, ctrl.yaw, ctrl.pitch, 0.0) 
        * Vec3::new(0.0, 0.0, ctrl.distance);
    let cam_pos = ctrl.focus_point - offset;
    
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_translation(cam_pos).looking_at(ctrl.focus_point, Vec3::Y),
        ..Default::default()
    }).insert(ctrl);

    // Ensure at least one light exists to keep bevy_pbr::prepare_lights on a non-empty slice path
    commands.spawn_bundle(DirectionalLightBundle {
        transform: Transform::from_xyz(4.0, 6.0, 4.0).looking_at(Vec3::new(0.0, 2.0, 0.0), Vec3::Y),
        ..Default::default()
    });

    // Use ambient-only lighting (point light extraction path appears unstable on this driver/runtime)

    let mesh = create_tetrahedron_mesh();
    let material = StandardMaterial {
        base_color: Color::rgb(0.8, 0.3, 0.9),
        unlit: true,
        ..Default::default()
    };

    let engine = FluxMatrixEngine::new();
    let iter = ChangeDotIter::from_seed(1_248_751, &engine);

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(material),
            transform: Transform::from_scale(Vec3::splat(1.0)),
            ..Default::default()
        })
        .insert(VortexCore { iter, angle: 0.0, pattern: [3, 6, 9, 1, 2, 4, 8, 7, 5] });

    // Node-zero marker at (0,2,0) and integer-Y ray hits above/below
    let origin = Vec3::new(0.0, 2.0, 0.0);
    let point_mesh = meshes.add(create_cube_mesh(0.12));
    let mat_zero = materials.add(StandardMaterial { base_color: Color::WHITE, unlit: true, ..Default::default() });
    let mat_pos = materials.add(StandardMaterial { base_color: Color::GREEN, unlit: true, ..Default::default() });
    let mat_neg = materials.add(StandardMaterial { base_color: Color::RED, unlit: true, ..Default::default() });

    // Spawn node-zero
    commands
        .spawn_bundle(PbrBundle {
            mesh: point_mesh.clone(),
            material: mat_zero.clone(),
            transform: Transform::from_translation(origin),
            ..Default::default()
        })
        .insert(NodeZero(0))
        .insert(Hinge { spin_velocity: 0.0, target_spin: 0.0 })
        .insert(RayHit { index: 0 })
        .insert(Marker { index: 0, digit: 0, rotation_speed: 0.6, base_scale: 0.12, lambda: 0.0, mass: 0.0 })
        .insert(Velocity { linvel: [0.0, 0.0, 0.0] })
        .insert(Confidence(0.0))
        .insert(Name::new("zero-0: 0"));

    // Hits above (including origin as 0)
    let up_steps: i32 = 3;
    for i in 1..=up_steps {
        let y = origin.y + i as f32;
        let digit = FluxMatrixEngine::new().reduce_digits((i as u64) * 2) as u8;
        commands
            .spawn_bundle(PbrBundle {
                mesh: point_mesh.clone(),
                material: mat_pos.clone(),
                transform: Transform::from_translation(Vec3::new(origin.x, y, origin.z)),
                ..Default::default()
            })
            .insert(RayHit { index: i })
            .insert(Marker { index: i, digit, rotation_speed: 1.0 + 0.2 * (i as f32), base_scale: 0.12, lambda: 0.7 + 0.1 * (i as f32), mass: (digit as f32) / 9.0 })
            .insert(Velocity { linvel: [0.0, 0.0, 0.0] })
            .insert(Confidence(digit as f32))
            .insert(Name::new(format!("pos-{}: {}", i, digit)))
            ;

        // Z-axis stack: two mini markers at +Z offsets
        for s in 1..=2 {
            let z_off = 0.3 * (s as f32);
            let scale = 0.08 - 0.015 * (s as f32 - 1.0);
            commands
                .spawn_bundle(PbrBundle {
                    mesh: point_mesh.clone(),
                    material: mat_pos.clone(),
                    transform: Transform::from_translation(Vec3::new(origin.x, y, origin.z + z_off)),
                    ..Default::default()
                })
                .insert(RayHit { index: i })
                .insert(Marker { index: i, digit, rotation_speed: 1.2 + 0.2 * (s as f32), base_scale: scale, lambda: 0.7 + 0.1 * (i as f32), mass: (digit as f32) / 9.0 })
                .insert(Velocity { linvel: [0.0, 0.0, 0.0] })
                .insert(Confidence(digit as f32))
                .insert(Name::new(format!("pos-{}.{}: {}", i, s, digit)));
        }
    }

    // Hits below (negative indices)
    let down_steps: i32 = 3;
    for i in 1..=down_steps {
        let y = origin.y - i as f32;
        let digit = FluxMatrixEngine::new().reduce_digits((i as u64) * 2) as u8;
        commands
            .spawn_bundle(PbrBundle {
                mesh: point_mesh.clone(),
                material: mat_neg.clone(),
                transform: Transform::from_translation(Vec3::new(origin.x, y, origin.z)),
                ..Default::default()
            })
            .insert(RayHit { index: -i })
            .insert(Marker { index: -i, digit, rotation_speed: 0.8 + 0.2 * (i as f32), base_scale: 0.12, lambda: 0.7 + 0.1 * (i as f32), mass: (digit as f32) / 9.0 })
            .insert(Velocity { linvel: [0.0, 0.0, 0.0] })
            .insert(Confidence(digit as f32))
            .insert(Name::new(format!("neg-{}: {}", i, digit)));

        // Z-axis stack for negatives as well
        for s in 1..=2 {
            let z_off = 0.3 * (s as f32);
            let scale = 0.08 - 0.015 * (s as f32 - 1.0);
            commands
                .spawn_bundle(PbrBundle {
                    mesh: point_mesh.clone(),
                    material: mat_neg.clone(),
                    transform: Transform::from_translation(Vec3::new(origin.x, y, origin.z + z_off)),
                    ..Default::default()
                })
                .insert(RayHit { index: -i })
                .insert(Marker { index: -i, digit, rotation_speed: 1.0 + 0.2 * (s as f32), base_scale: scale, lambda: 0.7 + 0.1 * (i as f32), mass: (digit as f32) / 9.0 })
                .insert(Velocity { linvel: [0.0, 0.0, 0.0] })
                .insert(Confidence(digit as f32))
                .insert(Name::new(format!("neg-{}.{}: {}", i, s, digit)));
            }
        }
}

fn animate_vortex(mut q: Query<(&mut Transform, &mut VortexCore)>) {
    for (mut transform, mut vortex) in q.iter_mut() {
        let mut digit_val: u8 = 1;
        if let Some(ev) = vortex.iter.next() {
            match ev {
                ChangeDotEvent::Step { to, .. } => { digit_val = to; }
                ChangeDotEvent::SacredHit { .. } => { /* no-op */ }
                ChangeDotEvent::Loop { .. } => {
                    let s = transform.scale.x;
                    transform.scale = Vec3::splat((s * 1.05).min(2.0));
                }
            }
        }
        let digit = digit_val as f32;
        let target_angle = std::f32::consts::TAU * (digit / 9.0);
        let alpha = 0.15;
        vortex.angle = vortex.angle + (target_angle - vortex.angle) * alpha;
        transform.rotation = Quat::from_rotation_y(vortex.angle);
        let current_scale = transform.scale.x;
        let target_scale = 1.0 + 0.35 * (digit / 9.0);
        let new_scale = current_scale + (target_scale - current_scale) * alpha;
        transform.scale = Vec3::splat(new_scale);
    }
}

fn handle_text_input(
    mut evr_char: EventReader<ReceivedCharacter>,
    kb: Res<Input<KeyCode>>,
    mut buffer: ResMut<InputBuffer>,
    mut q: Query<&mut VortexCore>,
) {
    for ev in evr_char.iter() {
        let c = ev.char;
        if c.is_ascii_digit() || c == '.' || c.is_ascii_whitespace() {
            buffer.text.push(c);
        }
    }

    if kb.just_pressed(KeyCode::Back) {
        buffer.text.pop();
    }

    if kb.just_pressed(KeyCode::Return) || kb.just_pressed(KeyCode::NumpadEnter) {
        let engine = FluxMatrixEngine::new();
        let mut iters = parse_change_dot(buffer.text.trim(), &engine);
        if let Some(first) = iters.pop() {
            for mut vortex in q.iter_mut() {
                vortex.iter = first.clone();
            }
        }
        buffer.text.clear();
    }
}

fn create_tetrahedron_mesh() -> Mesh {
    // Prefer a robust built-in sphere-like mesh for the core to avoid invalid UV/normal combos
    Mesh::from(bevy::prelude::shape::Icosphere { radius: 0.6, subdivisions: 3 })
}
