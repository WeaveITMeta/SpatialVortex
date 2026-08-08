//! Reason through an object's journey and recall what happened.
//!
//! Simulates an object ascending a spiral mountain path over a long timeline,
//! records the journey into ring-buffered spatial-temporal memory, then asks it
//! questions after the fact:
//!
//!   - where was it at time t?
//!   - what happened near this place?
//!   - what events fell between two times?
//!
//! Recall error is measured against ground truth and against a chance baseline
//! (guessing a uniformly random recorded waypoint). Beating chance by a wide
//! margin is the claim; the numbers below are the evidence.
//!
//! Run:
//! ```bash
//! cargo run --release --example journey_recall
//! ```

use vortex::ml::context_ring::ContextRingConfig;
use vortex::ml::trajectory::{JourneyEncoder, JourneyMemory, Waypoint};

const STEPS: usize = 100_000;
const WORLD: f32 = 512.0;

/// Deterministic LCG so the run is reproducible.
struct Lcg(u64);
impl Lcg {
    fn next_f32(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((self.0 >> 33) as f32) / (u32::MAX as f32)
    }
}

/// Ground-truth path: a spiral ascent. Deterministic in t, so any query time
/// has an exact answer to score against.
fn truth(t: f32) -> [f32; 3] {
    let turns = 6.0;
    let ang = t * turns * std::f32::consts::TAU;
    let radius = 200.0 * (1.0 - 0.6 * t); // spirals inward while climbing
    [radius * ang.cos(), t * 300.0, radius * ang.sin()]
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

fn main() {
    println!("Journey recall — spatial-temporal memory over a long path\n");

    let encoder = JourneyEncoder::new(10, WORLD);
    // Deep past compressed; only rings' summaries stay resident.
    let cfg = ContextRingConfig::new()
        .with_ring_capacity(2048)
        .with_max_rings(2047)
        .with_headroom(128)
        .with_retain_sealed_entries(false);
    let mut mem = JourneyMemory::new(encoder.clone(), cfg);

    // Landmarks the object passes, recorded as events.
    let landmarks: Vec<(f32, &str)> = vec![
        (0.08, "left base camp"),
        (0.22, "crossed the rope bridge"),
        (0.37, "sheltered from a storm"),
        (0.51, "found the cairn"),
        (0.66, "traversed the ridge"),
        (0.79, "passed the ice field"),
        (0.94, "reached the summit"),
    ];

    let t0 = std::time::Instant::now();
    let mut next_landmark = 0usize;
    for i in 0..STEPS {
        let t = i as f32 / (STEPS - 1) as f32;
        let p = truth(t);
        if next_landmark < landmarks.len() && t >= landmarks[next_landmark].0 {
            mem.observe(Waypoint::new(t, p).with_event(landmarks[next_landmark].1));
            next_landmark += 1;
        } else {
            mem.observe(Waypoint::new(t, p));
        }
    }
    let ingest = t0.elapsed();

    println!("  waypoints recorded : {}", mem.len());
    println!("  ingest             : {:.2}s ({:.0}/sec)", ingest.as_secs_f64(), STEPS as f64 / ingest.as_secs_f64());
    println!("  path length        : {:.0} units", mem.path_length());
    println!("  context view       : {} vectors", mem.context().len());
    println!("  ring represented   : {}", mem.ring().total_represented());
    println!("  encoder width      : {}", encoder.dim());

    // ---- Q1: where was it at time t? ----
    println!("\n  --- where was it at time t? ---");
    let probes = [0.05f32, 0.17, 0.33, 0.48, 0.62, 0.77, 0.91];
    let mut err_sum = 0.0;
    for &t in &probes {
        let w = mem.recall_at_time(t).expect("recall");
        let want = truth(t);
        let e = dist(w.pos, want);
        err_sum += e;
        println!(
            "    t={t:.2}  recalled ({:7.1},{:7.1},{:7.1})  truth ({:7.1},{:7.1},{:7.1})  err {:6.2}",
            w.pos[0], w.pos[1], w.pos[2], want[0], want[1], want[2], e
        );
    }
    let mean_err = err_sum / probes.len() as f32;

    // Chance baseline: guess a random recorded waypoint.
    let mut rng = Lcg(0x30C0FFEE);
    let mut chance_sum = 0.0;
    let n_chance = 400;
    for _ in 0..n_chance {
        let t = rng.next_f32();
        let idx = (rng.next_f32() * (mem.len() - 1) as f32) as usize;
        chance_sum += dist(mem.waypoints()[idx].pos, truth(t));
    }
    let chance_err = chance_sum / n_chance as f32;

    println!("\n    mean recall error : {mean_err:.2} units");
    println!("    chance baseline   : {chance_err:.2} units");
    println!("    better than chance: {:.0}x", chance_err / mean_err.max(1e-6));

    // ---- Q2: what happened near this place? ----
    println!("\n  --- what happened near this place? ---");
    let mut place_hits = 0usize;
    for &(lt, name) in landmarks.iter() {
        let p = truth(lt);
        let w = mem.recall_event_at_place(p).expect("an event");
        let label = w.event.as_deref().unwrap_or("");
        let hit = label == name;
        if hit {
            place_hits += 1;
        }
        let verdict = if hit { "correct".to_string() } else { format!("MISS (wanted: {name})") };
        println!(
            "    near ({:7.1},{:7.1},{:7.1})  ->  t={:.2}  {label:<24} {verdict}",
            p[0], p[1], p[2], w.t
        );
    }
    println!("    place-query accuracy: {place_hits}/{}", landmarks.len());

    // ---- Q3: what happened between two times? ----
    println!("\n  --- what happened between t=0.30 and t=0.70? ---");
    for w in mem.events_between(0.30, 0.70) {
        println!("    t={:.2}  {}", w.t, w.event.as_deref().unwrap_or(""));
    }

    // ---- verdict ----
    let recall_good = mean_err < chance_err * 0.1;
    let events_kept = mem.events_between(0.0, 1.0).len() == landmarks.len();
    let places_good = place_hits == landmarks.len();
    println!("\n  time recall beats chance >10x : {}", if recall_good { "yes" } else { "NO" });
    println!("  all {} events retained         : {}", landmarks.len(), if events_kept { "yes" } else { "NO" });
    println!("  place queries all correct      : {}", if places_good { "yes" } else { "NO" });
    let pass = recall_good && events_kept && places_good;
    println!(
        "\n  RESULT: {}",
        if pass { "PASS — the journey is queryable long after the fact" } else { "FAIL" }
    );
    if !pass {
        std::process::exit(1);
    }
}
