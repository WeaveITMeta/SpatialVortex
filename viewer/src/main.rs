use bevy::input::keyboard::ReceivedCharacter;
use bevy::prelude::*;
use bevy::render::mesh::shape;
use rand::Rng;
use windsurf::{VortexCore, VortexNet};
use windsurf::seeds::{load_seed_matrix, save_seed_matrix};
use tch::{Kind, Tensor};
use bevy::time::Timer;
use bevy::time::TimerMode;
use std::collections::VecDeque;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(handle_text_input)
        .add_system(self_talk_tick)
        .add_system(animate)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 2.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(6.0, 8.0, 6.0),
        ..default()
    });

    let mesh = Mesh::from(shape::Torus::default());
    let material = StandardMaterial { base_color: Color::rgb(1., 0., 0.), ..default() };

    commands
        .spawn(PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(material),
            transform: Transform::from_scale(Vec3::splat(1.0)),
            ..default()
        })
        .insert(VortexCore::new());

    let mut net = VortexNet::new(64, 32, 9);
    let mut rng = rand::thread_rng();
    let samples: Vec<String> = (0..256)
        .map(|_| {
            let len = rng.gen_range(3..12);
            (0..len)
                .map(|_| (rng.gen_range(97u8..123) as char))
                .collect::<String>()
        })
        .collect();
    net.train_on(&samples, 50, 0.05);

    let logits = net.predict_logits("physics");
    let probs = logits.softmax(-1, Kind::Float);
    let digit = (probs.argmax(-1, false).int64_value(&[0]) as i32) + 1;
    commands.insert_resource(SelfLoopTimer(Timer::from_seconds(5.0, TimerMode::Repeating)));
    commands.insert_resource(ModelState { net, latest_logits: logits, latest_probs: probs.shallow_clone(), last_digit: digit, buffer: String::new(), subject: String::from("physics"), last_self_digit: digit, last_self_probs: Tensor::zeros(&[1,9], (Kind::Float, tch::Device::Cpu)), stm: ShortTermMemory::new() });
}

#[derive(Resource)]
struct ModelState {
    net: VortexNet,
    latest_logits: Tensor,
    latest_probs: Tensor,
    last_digit: i32,
    buffer: String,
    subject: String,
    last_self_digit: i32,
    last_self_probs: Tensor,
    stm: ShortTermMemory,
}

#[derive(Resource)]
struct SelfLoopTimer(Timer);

#[derive(Default)]
struct ShortTermMemory {
    tokens: VecDeque<String>,
    vectors: VecDeque<Vec<f32>>,
}

impl ShortTermMemory {
    fn new() -> Self { Self { tokens: VecDeque::with_capacity(10), vectors: VecDeque::with_capacity(10) } }
    fn push_token(&mut self, t: String) { if self.tokens.len() >= 10 { self.tokens.pop_front(); } self.tokens.push_back(t); }
    fn push_vector(&mut self, v: Vec<f32>) { if self.vectors.len() >= 10 { self.vectors.pop_front(); } self.vectors.push_back(v); }
}

fn handle_text_input(
    mut evr_char: EventReader<ReceivedCharacter>,
    kb: Res<Input<KeyCode>>,
    mut state: ResMut<ModelState>,
) {
    for ev in evr_char.iter() {
        let c = ev.char;
        if c.is_ascii_alphanumeric() || c.is_ascii_whitespace() {
            state.buffer.push(c);
        }
    }
    if kb.just_pressed(KeyCode::Back) {
        state.buffer.pop();
    }
    if kb.just_pressed(KeyCode::Return) || kb.just_pressed(KeyCode::NumpadEnter) {
        let text = state.buffer.trim().to_string();
        if !text.is_empty() {
            if let Ok(Some(t)) = load_seed_matrix(&text) {
                let digit = (t.argmax(-1, false).int64_value(&[0]) as i32) + 1;
                state.latest_logits = t.log();
                state.latest_probs = t;
                state.last_digit = digit;
                state.subject = text.clone();
            } else {
                let logits = state.net.predict_logits(&text);
                let probs = logits.softmax(-1, Kind::Float);
                let digit = (probs.argmax(-1, false).int64_value(&[0]) as i32) + 1;
                let _ = save_seed_matrix(&text, &probs);
                state.latest_logits = logits;
                state.latest_probs = probs;
                state.last_digit = digit;
                state.subject = text.clone();
            }
            state.stm.push_token(text.clone());
            let mut vec9 = Vec::with_capacity(9);
            for i in 0..9 { vec9.push(state.latest_probs.double_value(&[0, i]) as f32); }
            state.stm.push_vector(vec9);
        }
        state.buffer.clear();
    }
}

fn self_talk_tick(
    time: Res<Time>,
    mut timer: ResMut<SelfLoopTimer>,
    mut state: ResMut<ModelState>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        let text = if let Some(last) = state.stm.tokens.back() { format!("self {}", last) } else { format!("self {}", state.subject) };
        let logits = state.net.predict_logits(&text);
        let probs = logits.softmax(-1, Kind::Float);
        let digit = (probs.argmax(-1, false).int64_value(&[0]) as i32) + 1;
        state.last_self_probs = probs;
        state.last_self_digit = digit;
        state.stm.push_token(text);
        let mut vec9 = Vec::with_capacity(9);
        for i in 0..9 { vec9.push(state.last_self_probs.double_value(&[0, i]) as f32); }
        state.stm.push_vector(vec9);
    }
}

fn animate(
    state: Res<ModelState>,
    mut q: Query<(&mut Transform, &mut VortexCore)>
) {
    for (mut transform, mut core) in q.iter_mut() {
        let digit = state.last_digit;
        let angle = core.lerp_angle(digit);
        transform.rotation = Quat::from_rotation_z(angle);
        let target_scale = 1.0 + 0.35 * (digit as f32 / 9.0);
        transform.scale = transform.scale.lerp(Vec3::splat(target_scale), 0.15);
    }
}
