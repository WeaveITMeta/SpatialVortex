//! Astra Spatial Reasoning Lab, procedurally generated.
//!
//! Each instance is a fresh walled course with gated openings, a set of
//! portal gaps and an assembly target list, in the exact format of
//! `Documents/Eustress/Astra/Spaces/Spatial Reasoning Lab/DataService/challenge.json`.
//! [`score`] is a line-for-line port of that Space's `evaluate.py`
//! (submission mode): 40 navigation + 20 clearance + 40 assembly.
//!
//! Feasibility of every generated course is established by the generator
//! with its own method (Dijkstra on an occupancy grid), independent of the
//! solver under test, and the length limit is set from that grid path. The
//! real Space is also scored by its own Python evaluator from the CLI
//! (`vortex astra`), so this port is cross-checked against the original.

use std::collections::{BTreeMap, BinaryHeap};

use vortex_kernel::spatial::{Assembly, Clearance, Navigation, Obstacle};
use vortex_kernel::{Answer, Response, Rng, SpatialChallenge, SpatialSubmission, Task};

use crate::{Capability, Item, Suite};

pub struct AstraSuite {
    items: Vec<Item>,
    challenges: Vec<SpatialChallenge>,
    source: String,
}

fn intersects(a: [f64; 2], b: [f64; 2], r: [f64; 4]) -> bool {
    let (mut low, mut high) = (0.0f64, 1.0f64);
    for axis in 0..2 {
        let d = b[axis] - a[axis];
        let (lo, hi) = (r[axis], r[axis + 2]);
        if d.abs() < 1e-12 {
            if a[axis] < lo || a[axis] > hi {
                return false;
            }
        } else {
            let mut t = [(lo - a[axis]) / d, (hi - a[axis]) / d];
            t.sort_by(|x, y| x.partial_cmp(y).unwrap());
            low = low.max(t[0]);
            high = high.min(t[1]);
            if low > high {
                return false;
            }
        }
    }
    true
}

fn dist(a: [f64; 2], b: [f64; 2]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
}

/// Port of `evaluate.py::evaluate` (submission-only). Returns score / 100.
pub fn score(spec: &SpatialChallenge, sub: &SpatialSubmission) -> f64 {
    let nav = &spec.navigation;
    let route = &sub.route;
    let mut total = 0.0;
    if route.len() >= 2 && route.iter().all(|p| p[0].is_finite() && p[1].is_finite()) {
        let c = nav.robot_width / 2.0 + nav.safety_margin;
        let b = nav.bounds;
        let bounded = route.iter().all(|p| b[0] + c <= p[0] && p[0] <= b[2] - c && b[1] + c <= p[1] && p[1] <= b[3] - c);
        let endpoints = dist(route[0], nav.start) <= 0.05 && dist(*route.last().unwrap(), nav.goal) <= 0.05;
        let collision = route.windows(2).any(|w| {
            nav.obstacles.iter().any(|o| {
                let r = o.rect;
                intersects(w[0], w[1], [r[0] - c, r[1] - c, r[2] + c, r[3] + c])
            })
        });
        let length: f64 = route.windows(2).map(|w| dist(w[0], w[1])).sum();
        if endpoints && bounded && !collision && length <= nav.max_route_length {
            total += 40.0;
        }
    }
    let cl = &spec.clearance;
    let expected: Vec<u32> = cl.portal_gaps.iter().enumerate().filter(|(_, g)| **g + 1e-9 >= cl.robot_width + 2.0 * cl.margin_each_side).map(|(i, _)| i as u32 + 1).collect();
    let mut got = sub.passable_portals.clone();
    got.sort();
    let unique = got.windows(2).all(|w| w[0] != w[1]);
    if unique && got == expected {
        total += 20.0;
    }
    let n = spec.assembly.targets.len().max(1) as f64;
    let mut passed = 0.0;
    for (name, target) in &spec.assembly.targets {
        if let Some(p) = sub.assembly_positions.get(name) {
            let e = ((p[0] - target[0]).powi(2) + (p[1] - target[1]).powi(2) + (p[2] - target[2]).powi(2)).sqrt();
            if e <= spec.assembly.position_tolerance {
                passed += 1.0;
            }
        }
    }
    total += 40.0 * passed / n;
    total / 100.0
}

/// Shortest 8-connected path on an occupancy grid (the generator's own
/// feasibility check). Returns its length in metres.
fn grid_path_length(nav: &Navigation, res: f64) -> Option<f64> {
    let c = nav.robot_width / 2.0 + nav.safety_margin;
    let b = nav.bounds;
    let (x0, z0) = (b[0] + c, b[1] + c);
    let w = ((b[2] - c - x0) / res).floor() as usize + 1;
    let h = ((b[3] - c - z0) / res).floor() as usize + 1;
    let pad = c + 0.01;
    let free = |x: f64, z: f64| !nav.obstacles.iter().any(|o| x >= o.rect[0] - pad && x <= o.rect[2] + pad && z >= o.rect[1] - pad && z <= o.rect[3] + pad);
    let cell = |p: [f64; 2]| (((p[0] - x0) / res).round() as usize, ((p[1] - z0) / res).round() as usize);
    let (s, g) = (cell(nav.start), cell(nav.goal));
    let idx = |x: usize, z: usize| z * w + x;
    let mut best = vec![f64::INFINITY; w * h];
    let mut heap = BinaryHeap::new();
    #[derive(PartialEq)]
    struct S(f64, usize);
    impl Eq for S {}
    impl PartialOrd for S {
        fn partial_cmp(&self, o: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(o))
        }
    }
    impl Ord for S {
        fn cmp(&self, o: &Self) -> std::cmp::Ordering {
            o.0.partial_cmp(&self.0).unwrap_or(std::cmp::Ordering::Equal)
        }
    }
    best[idx(s.0, s.1)] = 0.0;
    heap.push(S(0.0, idx(s.0, s.1)));
    while let Some(S(d, u)) = heap.pop() {
        if u == idx(g.0, g.1) {
            // Grid paths overestimate true length; the start/goal snap adds at most one cell.
            return Some(d + 2.0 * res);
        }
        if d > best[u] {
            continue;
        }
        let (ux, uz) = (u % w, u / w);
        for (dx, dz) in [(-1i64, 0i64), (1, 0), (0, -1), (0, 1), (-1, -1), (-1, 1), (1, -1), (1, 1)] {
            let (nx, nz) = (ux as i64 + dx, uz as i64 + dz);
            if nx < 0 || nz < 0 || nx >= w as i64 || nz >= h as i64 {
                continue;
            }
            let (nx, nz) = (nx as usize, nz as usize);
            if !free(x0 + nx as f64 * res, z0 + nz as f64 * res) {
                continue;
            }
            let nd = d + res * if dx != 0 && dz != 0 { std::f64::consts::SQRT_2 } else { 1.0 };
            let v = idx(nx, nz);
            if nd < best[v] {
                best[v] = nd;
                heap.push(S(nd, v));
            }
        }
    }
    None
}

pub fn generate(rng: &mut Rng) -> SpatialChallenge {
    loop {
        let width = rng.range_f64(14.0, 28.0);
        let depth = rng.range_f64(16.0, 26.0);
        let robot = rng.range_f64(0.6, 1.4);
        let margin = rng.range_f64(0.05, 0.2);
        let c = robot / 2.0 + margin;
        let gates = 2 + rng.below(3);
        let mut obstacles = Vec::new();
        for gi in 0..gates {
            let z = depth * (gi as f64 + 1.0) / (gates as f64 + 1.0);
            let opening = 2.0 * c + rng.range_f64(0.4, 2.0);
            let lo = c + 0.3;
            let hi = width - c - 0.3 - opening;
            if hi <= lo {
                continue;
            }
            let x_open = rng.range_f64(lo, hi);
            let t = 0.2;
            obstacles.push(Obstacle { name: format!("Gate_{gi}_Left"), rect: [-0.5, z - t, x_open, z + t] });
            obstacles.push(Obstacle { name: format!("Gate_{gi}_Right"), rect: [x_open + opening, z - t, width + 0.5, z + t] });
        }
        // Occasional free-standing pillar to break straight lines.
        if rng.f64() < 0.5 {
            let (px, pz) = (rng.range_f64(2.0, width - 2.0), rng.range_f64(2.0, depth - 2.0));
            obstacles.push(Obstacle { name: "Pillar".into(), rect: [px - 0.4, pz - 0.4, px + 0.4, pz + 0.4] });
        }
        let first = depth / (gates as f64 + 1.0);
        let start = [rng.range_f64(c + 0.5, width - c - 0.5), first * 0.4];
        let goal = [rng.range_f64(c + 0.5, width - c - 0.5), depth - first * 0.4];
        let mut nav = Navigation { start, goal, robot_width: robot, safety_margin: margin, max_route_length: f64::INFINITY, bounds: [0.0, 0.0, width, depth], obstacles };
        // Start/goal must not sit inside a grown obstacle.
        let blocked = |p: [f64; 2]| nav.obstacles.iter().any(|o| p[0] >= o.rect[0] - c && p[0] <= o.rect[2] + c && p[1] >= o.rect[1] - c && p[1] <= o.rect[3] + c);
        if blocked(start) || blocked(goal) {
            continue;
        }
        let Some(grid_len) = grid_path_length(&nav, 0.1) else { continue };
        nav.max_route_length = (grid_len * 1.1 * 100.0).round() / 100.0;

        let k = 3 + rng.below(2);
        let need = robot + 2.0 * margin;
        let portal_gaps: Vec<f64> = (0..k).map(|_| (rng.range_f64(need - 0.6, need + 0.8) * 100.0).round() / 100.0).filter(|g| *g > 0.2).collect();
        let parts = ["Pillar_A", "Pillar_B", "Beam", "Brace", "Panel"];
        let m = 2 + rng.below(3);
        let mut targets = BTreeMap::new();
        let mut sizes = BTreeMap::new();
        for p in parts.iter().take(m) {
            targets.insert(p.to_string(), [(rng.range_f64(-20.0, 20.0) * 10.0).round() / 10.0, (rng.range_f64(0.5, 5.0) * 10.0).round() / 10.0, (rng.range_f64(-20.0, 20.0) * 10.0).round() / 10.0]);
            sizes.insert(p.to_string(), [rng.range_f64(0.5, 8.0), rng.range_f64(0.5, 4.0), rng.range_f64(0.5, 2.0)]);
        }
        return SpatialChallenge {
            navigation: nav,
            clearance: Clearance { portal_gaps, robot_width: robot, margin_each_side: margin },
            assembly: Assembly { targets, position_tolerance: 0.05, rotation_tolerance: 0.001, sizes },
        };
    }
}

pub fn procedural(n: usize, seed: u64) -> AstraSuite {
    let mut rng = Rng::new(seed ^ 0xA57A);
    let challenges: Vec<SpatialChallenge> = (0..n).map(|_| generate(&mut rng)).collect();
    let items = challenges.iter().enumerate().map(|(i, c)| Item { id: format!("astra-gen-{seed}-{i}"), task: Task::Spatial(c.clone()) }).collect();
    AstraSuite { items, challenges, source: format!("{n} procedural Astra courses (seed {seed}); scorer ported from the Space's evaluate.py") }
}

impl Suite for AstraSuite {
    fn id(&self) -> &str {
        "astra-procedural"
    }
    fn capability(&self) -> Capability {
        Capability::Spatial2d
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
        match &r.answer {
            Answer::Spatial(s) => score(&self.challenges[i], s),
            _ => 0.0,
        }
    }
    fn summarize(&self, rs: &[Response]) -> BTreeMap<String, f64> {
        let mut nav = 0.0;
        for (i, r) in rs.iter().enumerate() {
            if let Answer::Spatial(s) = &r.answer {
                let mut only_nav = s.clone();
                only_nav.passable_portals = vec![u32::MAX];
                only_nav.assembly_positions.clear();
                if score(&self.challenges[i], &only_nav) >= 0.399 {
                    nav += 1.0;
                }
            }
        }
        BTreeMap::from([("navigation_pass_rate".into(), nav / rs.len().max(1) as f64)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_courses_are_feasible_and_scorer_rejects_bad_routes() {
        let mut rng = Rng::new(3);
        for _ in 0..5 {
            let ch = generate(&mut rng);
            assert!(ch.navigation.max_route_length.is_finite());
            // A straight line from start to goal must cross the first gate wall.
            let sub = SpatialSubmission { route: vec![ch.navigation.start, ch.navigation.goal], ..Default::default() };
            let s = score(&ch, &sub);
            assert!(s < 0.4, "straight line through walls scored {s}");
        }
    }
}
