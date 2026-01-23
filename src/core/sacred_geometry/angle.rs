pub fn compute_next(angle: f32, digit: u8, alpha: f32) -> f32 {
    let target = std::f32::consts::TAU * (digit as f32 / 9.0);
    angle + (target - angle) * alpha
}
