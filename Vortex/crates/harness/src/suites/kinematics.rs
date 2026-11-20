//! 3D-plus-time kinematics, procedurally generated. Gold answers come from
//! **numerical simulation** (fine time stepping plus refinement), not from
//! the closed-form formulas the solver uses, so the two methods check each
//! other.

use vortex_kernel::{Answer, Body, KinematicsQuery, Response, Rng, Task};

use crate::{Capability, Item, Suite};

pub struct KinematicsSuite {
    items: Vec<Item>,
    gold: Vec<Vec<f64>>,
    source: String,
}

fn pos(b: &Body, t: f64) -> [f64; 3] {
    // Integrate with a fixed small step (semi-implicit), then correct the
    // remainder exactly: deliberately a different method from Body::at.
    let steps = 64usize;
    let dt = t / steps as f64;
    let (mut p, mut v) = (b.position, b.velocity);
    for _ in 0..steps {
        for i in 0..3 {
            // Trapezoid in velocity: exact for constant acceleration.
            let v_next = v[i] + b.acceleration[i] * dt;
            p[i] += 0.5 * (v[i] + v_next) * dt;
            v[i] = v_next;
        }
    }
    p
}

fn sep(a: &Body, b: &Body, t: f64) -> f64 {
    let (pa, pb) = (pos(a, t), pos(b, t));
    ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2)).sqrt()
}

fn gold(q: &KinematicsQuery) -> Option<Vec<f64>> {
    match q {
        KinematicsQuery::PositionAt { body, t } => Some(pos(body, *t).to_vec()),
        KinematicsQuery::ClosestApproach { a, b, horizon } => {
            // Coarse scan, then golden-section on the best bracket.
            let n = 400;
            let (mut bi, mut bd) = (0usize, f64::INFINITY);
            for i in 0..=n {
                let d = sep(a, b, horizon * i as f64 / n as f64);
                if d < bd {
                    bd = d;
                    bi = i;
                }
            }
            let (mut lo, mut hi) = (horizon * (bi.saturating_sub(1)) as f64 / n as f64, horizon * ((bi + 1).min(n)) as f64 / n as f64);
            let g = (5f64.sqrt() - 1.0) / 2.0;
            for _ in 0..80 {
                let (m1, m2) = (hi - g * (hi - lo), lo + g * (hi - lo));
                if sep(a, b, m1) < sep(a, b, m2) { hi = m2 } else { lo = m1 }
            }
            let t = (lo + hi) / 2.0;
            Some(vec![t, sep(a, b, t)])
        }
        KinematicsQuery::TimeToHeight { body, y } => {
            // March forward until the body is below y while descending, then bisect.
            let dt = 0.01;
            let mut t = 0.0;
            let mut prev = pos(body, 0.0)[1] - y;
            for _ in 0..200_000 {
                let next_t = t + dt;
                let cur = pos(body, next_t)[1] - y;
                let vy = body.velocity[1] + body.acceleration[1] * next_t;
                if prev > 0.0 && cur <= 0.0 && vy < 0.0 {
                    let (mut lo, mut hi) = (t, next_t);
                    for _ in 0..60 {
                        let mid = (lo + hi) / 2.0;
                        if pos(body, mid)[1] - y > 0.0 { lo = mid } else { hi = mid }
                    }
                    return Some(vec![(lo + hi) / 2.0]);
                }
                prev = cur;
                t = next_t;
            }
            None
        }
    }
}

fn r3(rng: &mut Rng, s: f64) -> [f64; 3] {
    [rng.range_f64(-s, s), rng.range_f64(-s, s), rng.range_f64(-s, s)]
}

pub fn procedural(n: usize, seed: u64) -> KinematicsSuite {
    let mut rng = Rng::new(seed ^ 0x6B19);
    let (mut items, mut golds) = (Vec::new(), Vec::new());
    while items.len() < n {
        let q = match rng.below(3) {
            0 => KinematicsQuery::PositionAt {
                body: Body { name: "probe".into(), position: r3(&mut rng, 50.0), velocity: r3(&mut rng, 10.0), acceleration: r3(&mut rng, 3.0) },
                t: rng.range_f64(0.1, 10.0),
            },
            1 => KinematicsQuery::ClosestApproach {
                a: Body { name: "a".into(), position: r3(&mut rng, 100.0), velocity: r3(&mut rng, 15.0), acceleration: [0.0; 3] },
                b: Body { name: "b".into(), position: r3(&mut rng, 100.0), velocity: r3(&mut rng, 15.0), acceleration: [0.0; 3] },
                horizon: rng.range_f64(5.0, 30.0),
            },
            _ => {
                let mut p = r3(&mut rng, 20.0);
                p[1] = rng.range_f64(5.0, 100.0);
                let mut v = r3(&mut rng, 8.0);
                v[1] = rng.range_f64(-5.0, 30.0);
                KinematicsQuery::TimeToHeight {
                    body: Body { name: "projectile".into(), position: p, velocity: v, acceleration: [0.0, -9.81, 0.0] },
                    y: rng.range_f64(0.0, p[1] * 0.8),
                }
            }
        };
        if let Some(g) = gold(&q) {
            items.push(Item { id: format!("kin-{seed}-{}", items.len()), task: Task::Kinematics(q) });
            golds.push(g);
        }
    }
    KinematicsSuite { items, gold: golds, source: format!("{n} procedural 3D+time queries (seed {seed}); gold by numerical simulation") }
}

impl Suite for KinematicsSuite {
    fn id(&self) -> &str {
        "kinematics-3d"
    }
    fn capability(&self) -> Capability {
        Capability::Spatial3dTime
    }
    fn chance(&self) -> f64 {
        0.0
    }
    fn source(&self) -> String {
        self.source.clone()
    }
    fn items(&self) -> &[Item] {
        &self.items
    }
    fn grade(&self, i: usize, r: &Response) -> f64 {
        let Answer::Numbers(v) = &r.answer else { return 0.0 };
        let g = &self.gold[i];
        let ok = v.len() == g.len() && v.iter().zip(g).all(|(a, b)| (a - b).abs() <= 1e-3 * b.abs().max(1.0));
        if ok { 1.0 } else { 0.0 }
    }
}
