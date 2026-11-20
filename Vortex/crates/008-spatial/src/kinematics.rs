//! Closed-form kinematics in 3D.

use vortex_kernel::{Body, KinematicsQuery};

pub fn answer(q: &KinematicsQuery) -> Option<Vec<f64>> {
    match q {
        KinematicsQuery::PositionAt { body, t } => Some(body.at(*t).to_vec()),
        KinematicsQuery::ClosestApproach { a, b, horizon } => Some(closest_approach(a, b, *horizon).to_vec()),
        KinematicsQuery::TimeToHeight { body, y } => time_to_height(body, *y).map(|t| vec![t]),
    }
}

/// Constant-velocity closest approach over `[0, horizon]`: `[t, distance]`.
pub fn closest_approach(a: &Body, b: &Body, horizon: f64) -> [f64; 2] {
    let r0: Vec<f64> = (0..3).map(|i| a.position[i] - b.position[i]).collect();
    let v: Vec<f64> = (0..3).map(|i| a.velocity[i] - b.velocity[i]).collect();
    let vv: f64 = v.iter().map(|x| x * x).sum();
    let t = if vv < 1e-15 { 0.0 } else { (-(0..3).map(|i| r0[i] * v[i]).sum::<f64>() / vv).clamp(0.0, horizon) };
    let d = (0..3).map(|i| (r0[i] + v[i] * t).powi(2)).sum::<f64>().sqrt();
    [t, d]
}

/// Time at which the body descends through height `y` (largest root of
/// `y0 + vy t + ay t^2 / 2 = y` with `t >= 0`).
pub fn time_to_height(body: &Body, y: f64) -> Option<f64> {
    let (y0, vy, ay) = (body.position[1], body.velocity[1], body.acceleration[1]);
    let (a, b, c) = (0.5 * ay, vy, y0 - y);
    if a.abs() < 1e-15 {
        if b.abs() < 1e-15 {
            return None;
        }
        let t = -c / b;
        return (t >= 0.0).then_some(t);
    }
    let disc = b * b - 4.0 * a * c;
    if disc < 0.0 {
        return None;
    }
    let s = disc.sqrt();
    let roots = [(-b - s) / (2.0 * a), (-b + s) / (2.0 * a)];
    roots.into_iter().filter(|t| *t >= 0.0).fold(None, |m: Option<f64>, t| Some(m.map_or(t, |m| m.max(t))))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn body(p: [f64; 3], v: [f64; 3], a: [f64; 3]) -> Body {
        Body { name: "b".into(), position: p, velocity: v, acceleration: a }
    }

    #[test]
    fn projectile_lands_when_expected() {
        // Thrown up at 10 m/s from 0 under g = 9.81: lands at 2v/g.
        let t = time_to_height(&body([0.0; 3], [0.0, 10.0, 0.0], [0.0, -9.81, 0.0]), 0.0).unwrap();
        assert!((t - 20.0 / 9.81).abs() < 1e-9);
    }

    #[test]
    fn head_on_bodies_meet() {
        let [t, d] = closest_approach(&body([0.0; 3], [1.0, 0.0, 0.0], [0.0; 3]), &body([10.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [0.0; 3]), 100.0);
        assert!((t - 5.0).abs() < 1e-12 && d.abs() < 1e-12);
    }
}
