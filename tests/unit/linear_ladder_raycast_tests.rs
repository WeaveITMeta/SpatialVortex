// Linear ladder raycast test (pure Rust, no Bevy)
// Concept: Anchor at (0,0,0). Spawn 20 nodes at (0, y*2, 0) for y=1..20.
// Step a ray up along +Y in 2.0-unit increments and detect hits with a simple AABB closeness.
// Expect: we "see" all 20 nodes in order.

#[derive(Clone, Copy, Debug, PartialEq)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3 {
    fn new(x: f32, y: f32, z: f32) -> Self { Self { x, y, z } }
    fn add(&self, other: Vec3) -> Self { Self::new(self.x + other.x, self.y + other.y, self.z + other.z) }
    fn mul_scalar(&self, s: f32) -> Self { Self::new(self.x * s, self.y * s, self.z * s) }
}

#[test]
fn test_linear_ladder_raycast() {
    // Anchor at origin
    let anchor = Vec3::new(0.0, 0.0, 0.0);

    // Generate 20 nodes: (0, 2, 0), (0, 4, 0), ..., (0, 40, 0)
    let mut ids: Vec<usize> = Vec::new();
    let mut positions: Vec<Vec3> = Vec::new();
    for y in 1..=20 {
        ids.push(y as usize);
        positions.push(Vec3::new(0.0, (y as f32) * 2.0, 0.0));
    }

    // Ray parameters
    let dir = Vec3::new(0.0, 1.0, 0.0);
    let max_dist = 40.0;

    // Simple stepping ray: every 2 units
    let mut ray_hits: Vec<usize> = Vec::new();
    let mut dist = 2.0f32;
    while dist <= max_dist + 1e-3 {
        let end = anchor.add(dir.mul_scalar(dist));
        // Find closest node within a small AABB (1.5 studs threshold like the spec)
        let mut best: Option<(usize, f32)> = None;
        for (i, pos) in positions.iter().enumerate() {
            let dy = (end.y - pos.y).abs();
            let dx = (end.x - pos.x).abs();
            let dz = (end.z - pos.z).abs();
            if dy < 1.5 && dx < 1.5 && dz < 1.5 {
                let score = dy; // closeness on Y is enough here
                match best {
                    None => best = Some((ids[i], score)),
                    Some((_, bscore)) if score < bscore => best = Some((ids[i], score)),
                    _ => {}
                }
            }
        }
        if let Some((hit_id, diff)) = best {
            if diff < 1.0 { // accept close hits
                ray_hits.push(hit_id);
            }
        }
        dist += 2.0;
    }

    // We should see all 20 nodes in order (1..=20)
    assert_eq!(ray_hits.len(), 20, "Ray should hit 20 nodes; got {}", ray_hits.len());
    assert_eq!(ray_hits[0], ids[0]);
    assert_eq!(ray_hits[19], ids[19]);
}
