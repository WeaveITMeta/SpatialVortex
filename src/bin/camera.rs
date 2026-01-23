use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseWheel, MouseScrollUnit};

#[derive(Component)]
pub struct CameraController {
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub focus_point: Vec3,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: -0.3,
            distance: 10.0,
            focus_point: Vec3::new(0.0, 2.0, 0.0),
        }
    }
}

#[derive(Default)]
pub struct MouseState {
    pub rmb_down: bool,
    pub mmb_down: bool,
    pub rmb_start_px: f32,
    pub mmb_start_px: f32,
}

impl CameraController {
    fn update_camera_transform(&self, transform: &mut Transform) {
        // Spherical to Cartesian conversion for orbit
        let x = self.distance * self.pitch.cos() * self.yaw.sin();
        let y = self.distance * self.pitch.sin();
        let z = self.distance * self.pitch.cos() * self.yaw.cos();
        
        transform.translation = self.focus_point + Vec3::new(x, y, z);
        transform.look_at(self.focus_point, Vec3::Y);
    }
}

// WASD movement - always active
pub fn camera_move_system(
    kb: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut q: Query<(&mut Transform, &mut CameraController)>,
) {
    if let Ok((mut tf, mut ctrl)) = q.get_single_mut() {
        let mut dir = Vec3::ZERO;
        
        // Horizontal plane movement
        let forward = Vec3::new(tf.forward().x, 0.0, tf.forward().z).normalize_or_zero();
        let right = Vec3::new(tf.right().x, 0.0, tf.right().z).normalize_or_zero();
        
        if kb.pressed(KeyCode::W) { dir += forward; }
        if kb.pressed(KeyCode::S) { dir -= forward; }
        if kb.pressed(KeyCode::D) { dir += right; }
        if kb.pressed(KeyCode::A) { dir -= right; }
        if kb.pressed(KeyCode::E) { dir += Vec3::Y; }
        if kb.pressed(KeyCode::Q) { dir -= Vec3::Y; }
        
        if dir.length_squared() > 0.0 {
            let mut speed = 25.0;
            if kb.pressed(KeyCode::LShift) { speed *= 3.0; }
            if kb.pressed(KeyCode::LControl) { speed *= 0.3; }
            
            let movement = dir.normalize() * speed * time.delta_seconds();
            ctrl.focus_point += movement;
            ctrl.update_camera_transform(&mut tf);
        }
    }
}

// RMB drag to rotate view / orbit
pub fn camera_rotate_system(
    mut ev_motion: EventReader<MouseMotion>,
    state: Res<MouseState>,
    mut q: Query<(&mut Transform, &mut CameraController)>,
) {
    if !state.rmb_down {
        ev_motion.clear();
        return;
    }
    
    if let Ok((mut tf, mut ctrl)) = q.get_single_mut() {
        for ev in ev_motion.iter() {
            // Adjusted sensitivity for smoother rotation
            let sensitivity = 0.005;
            ctrl.yaw += ev.delta.x * sensitivity;
            ctrl.pitch = (ctrl.pitch - ev.delta.y * sensitivity).clamp(-1.55, 1.55);
        }
        
        // Proper spherical orbit: rotate around focus point
        // Convert spherical coordinates (yaw, pitch, distance) to Cartesian offset
        let x = ctrl.distance * ctrl.pitch.cos() * ctrl.yaw.sin();
        let y = ctrl.distance * ctrl.pitch.sin();
        let z = ctrl.distance * ctrl.pitch.cos() * ctrl.yaw.cos();
        
        tf.translation = ctrl.focus_point + Vec3::new(x, y, z);
        tf.look_at(ctrl.focus_point, Vec3::Y);
    }
}

// Mouse wheel zoom
pub fn camera_zoom_system(
    mut ev_wheel: EventReader<MouseWheel>,
    mut q: Query<(&mut Transform, &mut CameraController)>,
) {
    if let Ok((mut tf, mut ctrl)) = q.get_single_mut() {
        let mut scroll = 0.0;
        for e in ev_wheel.iter() {
            scroll += match e.unit {
                MouseScrollUnit::Line => e.y,
                MouseScrollUnit::Pixel => e.y / 120.0,
            };
        }
        
        if scroll.abs() > 0.01 {
            ctrl.distance = (ctrl.distance * (1.0 - scroll * 0.1)).clamp(1.0, 500.0);
            ctrl.update_camera_transform(&mut tf);
        }
    }
}

// MMB drag to pan
pub fn camera_pan_system(
    mut ev_motion: EventReader<MouseMotion>,
    mouse: Res<MouseState>,
    mut q: Query<(&mut Transform, &mut CameraController)>,
) {
    if !mouse.mmb_down {
        ev_motion.clear();
        return;
    }
    
    if let Ok((mut tf, mut ctrl)) = q.get_single_mut() {
        for ev in ev_motion.iter() {
            let pan_speed = 0.01 * ctrl.distance;
            let pan = tf.right() * (-ev.delta.x * pan_speed) + tf.up() * (ev.delta.y * pan_speed);
            ctrl.focus_point += pan;
        }
        ctrl.update_camera_transform(&mut tf);
    }
}

// F to focus on selected
pub fn camera_focus_system(
    kb: Res<Input<KeyCode>>,
    mut q_cam: Query<(&mut Transform, &mut CameraController)>,
    q_selected: Query<&GlobalTransform, With<super::Selected>>,
) {
    if !kb.just_pressed(KeyCode::F) { return; }
    
    if let Ok((mut tf, mut ctrl)) = q_cam.get_single_mut() {
        if let Some(target) = q_selected.iter().next() {
            ctrl.focus_point = target.translation();
            ctrl.distance = 10.0;
            ctrl.update_camera_transform(&mut tf);
        }
    }
}

// Home to reset
pub fn camera_reset_system(
    kb: Res<Input<KeyCode>>,
    mut q: Query<(&mut Transform, &mut CameraController)>,
) {
    if !kb.just_pressed(KeyCode::Home) { return; }
    
    if let Ok((mut tf, mut ctrl)) = q.get_single_mut() {
        *ctrl = CameraController::default();
        ctrl.update_camera_transform(&mut tf);
    }
}

// Mouse state tracking
pub fn update_mouse_state(
    buttons: Res<Input<MouseButton>>,
    mut ev_motion: EventReader<MouseMotion>,
    mut state: ResMut<MouseState>,
) {
    let mut moved = 0.0;
    for e in ev_motion.iter() {
        moved += e.delta.length();
    }
    
    if buttons.just_pressed(MouseButton::Right) {
        state.rmb_down = true;
        state.rmb_start_px = 0.0;
    }
    if state.rmb_down {
        state.rmb_start_px += moved;
    }
    if buttons.just_released(MouseButton::Right) {
        state.rmb_down = false;
    }
    
    if buttons.just_pressed(MouseButton::Middle) {
        state.mmb_down = true;
        state.mmb_start_px = 0.0;
    }
    if state.mmb_down {
        state.mmb_start_px += moved;
    }
    if buttons.just_released(MouseButton::Middle) {
        state.mmb_down = false;
    }
}

// Cursor visibility
pub fn cursor_visibility_system(
    state: Res<MouseState>,
    mut windows: ResMut<Windows>,
) {
    if let Some(win) = windows.get_primary_mut() {
        win.set_cursor_visibility(!state.rmb_down);
    }
}
