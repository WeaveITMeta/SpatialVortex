//! Shortest collision-free route on the XZ plane.
//!
//! The robot (width `w`, margin `m`) is reduced to a point by growing every
//! obstacle by `w/2 + m` (configuration space). The shortest path among
//! convex obstacles bends only at their corners, so the search graph is
//! {start, goal, grown corners}, connected where the straight segment is
//! free, solved with Dijkstra. Corners are pushed a hair further out so the
//! route never *touches* a grown wall; touching counts as a collision.

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use vortex_kernel::spatial::Navigation;

/// Extra clearance kept beyond the required margin, in metres.
const SLACK: f64 = 2e-3;

pub fn length(route: &[[f64; 2]]) -> f64 {
    route.windows(2).map(|w| dist(w[0], w[1])).sum()
}

fn dist(a: [f64; 2], b: [f64; 2]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
}

/// Slab test over the closed segment; touching the rectangle counts as a hit.
/// Identical semantics to the Astra evaluator.
pub fn segment_hits_rect(a: [f64; 2], b: [f64; 2], r: [f64; 4]) -> bool {
    let (mut lo_t, mut hi_t) = (0.0f64, 1.0f64);
    for axis in 0..2 {
        let d = b[axis] - a[axis];
        let (lo, hi) = (r[axis], r[axis + 2]);
        if d.abs() < 1e-12 {
            if a[axis] < lo || a[axis] > hi {
                return false;
            }
        } else {
            let (t0, t1) = ((lo - a[axis]) / d, (hi - a[axis]) / d);
            let (t0, t1) = if t0 <= t1 { (t0, t1) } else { (t1, t0) };
            lo_t = lo_t.max(t0);
            hi_t = hi_t.min(t1);
            if lo_t > hi_t {
                return false;
            }
        }
    }
    true
}

fn grow(r: [f64; 4], c: f64) -> [f64; 4] {
    [r[0] - c, r[1] - c, r[2] + c, r[3] + c]
}

#[derive(PartialEq)]
struct State(f64, usize);
impl Eq for State {}
impl PartialOrd for State {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}
impl Ord for State {
    fn cmp(&self, o: &Self) -> Ordering {
        o.0.partial_cmp(&self.0).unwrap_or(Ordering::Equal).then_with(|| o.1.cmp(&self.1))
    }
}

/// Shortest valid route from start to goal, or `None` if none exists.
pub fn plan(nav: &Navigation) -> Option<Vec<[f64; 2]>> {
    let c = nav.robot_width / 2.0 + nav.safety_margin;
    let check: Vec<[f64; 4]> = nav.obstacles.iter().map(|o| grow(o.rect, c + SLACK / 2.0)).collect();
    let corner_rects: Vec<[f64; 4]> = nav.obstacles.iter().map(|o| grow(o.rect, c + SLACK)).collect();
    let b = nav.bounds;
    let inside = |p: [f64; 2]| p[0] >= b[0] + c && p[0] <= b[2] - c && p[1] >= b[1] + c && p[1] <= b[3] - c;
    let free_point = |p: [f64; 2]| inside(p) && !check.iter().any(|r| p[0] >= r[0] && p[0] <= r[2] && p[1] >= r[1] && p[1] <= r[3]);

    let mut nodes = vec![nav.start, nav.goal];
    for r in &corner_rects {
        for p in [[r[0], r[1]], [r[0], r[3]], [r[2], r[1]], [r[2], r[3]]] {
            if free_point(p) {
                nodes.push(p);
            }
        }
    }
    let n = nodes.len();
    let visible = |i: usize, j: usize| !check.iter().any(|r| segment_hits_rect(nodes[i], nodes[j], *r));

    let mut best = vec![f64::INFINITY; n];
    let mut prev = vec![usize::MAX; n];
    let mut heap = BinaryHeap::new();
    best[0] = 0.0;
    heap.push(State(0.0, 0));
    while let Some(State(d, u)) = heap.pop() {
        if u == 1 {
            break;
        }
        if d > best[u] {
            continue;
        }
        for v in 0..n {
            if v == u {
                continue;
            }
            let nd = d + dist(nodes[u], nodes[v]);
            if nd < best[v] && visible(u, v) {
                best[v] = nd;
                prev[v] = u;
                heap.push(State(nd, v));
            }
        }
    }
    if !best[1].is_finite() {
        return None;
    }
    let mut route = vec![nodes[1]];
    let mut cur = 1;
    while cur != 0 {
        cur = prev[cur];
        route.push(nodes[cur]);
    }
    route.reverse();
    Some(route)
}

#[cfg(test)]
mod tests {
    use super::*;
    use vortex_kernel::spatial::Obstacle;

    fn astra() -> Navigation {
        // The real Astra Spatial Reasoning Lab navigation challenge.
        Navigation {
            start: [-15.0, -7.0],
            goal: [-5.0, 7.0],
            robot_width: 1.0,
            safety_margin: 0.1,
            max_route_length: 50.0,
            bounds: [-17.8, -9.8, -2.2, 9.8],
            obstacles: vec![
                Obstacle { name: "Gate_A_Left".into(), rect: [-18.0, -3.2, -7.0, -2.8] },
                Obstacle { name: "Gate_A_Right".into(), rect: [-4.0, -3.2, -2.0, -2.8] },
                Obstacle { name: "Gate_B_Left".into(), rect: [-18.0, 2.8, -16.0, 3.2] },
                Obstacle { name: "Gate_B_Right".into(), rect: [-13.0, 2.8, -2.0, 3.2] },
            ],
        }
    }

    #[test]
    fn solves_astra_without_touching_walls() {
        let nav = astra();
        let route = plan(&nav).expect("route exists");
        let c = nav.robot_width / 2.0 + nav.safety_margin;
        for w in route.windows(2) {
            for o in &nav.obstacles {
                assert!(!segment_hits_rect(w[0], w[1], grow(o.rect, c)), "segment {w:?} hits {}", o.name);
            }
        }
        let len = length(&route);
        assert!(len <= nav.max_route_length, "length {len}");
        assert_eq!(route[0], nav.start);
        assert_eq!(*route.last().unwrap(), nav.goal);
    }

    #[test]
    fn reports_impossible() {
        let mut nav = astra();
        nav.obstacles.push(Obstacle { name: "Wall".into(), rect: [-18.0, -0.2, -2.0, 0.2] });
        assert!(plan(&nav).is_none());
    }
}
