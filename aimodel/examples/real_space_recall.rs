//! Spatial recall over **real Mountain Ascension geometry** — no engine needed.
//!
//! The journey demo ran on a synthetic spiral, which proves the machinery works
//! but says nothing about the actual world. This reads real `_instance.toml`
//! files straight off disk and asks whether the sinusoidal spatial encoding
//! actually retrieves the correct neighbour in real geometry.
//!
//! ## What is measured
//! For each probe position, the true answer is the Euclidean-nearest instance —
//! computed by brute force, so it is ground truth by construction. The encoder
//! is then asked the same question through cosine similarity over its codes.
//! Agreement is top-1 accuracy; disagreement is reported as the distance
//! penalty incurred (how much further the retrieved instance was).
//!
//! A high top-1 with a small penalty means the encoding preserves real spatial
//! structure. That is the claim the whole spatial-intelligence thesis rests on,
//! and it is checkable without Eustress running.
//!
//! ```bash
//! cargo run --release --example real_space_recall
//! ```

use std::fs;
use std::path::{Path, PathBuf};

use vortex::ml::trajectory::JourneyEncoder;

const MAX_INSTANCES: usize = 30_000;
const PROBES: usize = 300;

#[derive(Debug, Clone)]
struct Instance {
    name: String,
    class_name: String,
    pos: [f32; 3],
}

/// Minimal targeted parse: we need `[metadata] name/class_name` and
/// `[transform] position`. A full TOML parse of 130k files is far slower and
/// buys nothing here.
fn parse_instance(path: &Path) -> Option<Instance> {
    let text = fs::read_to_string(path).ok()?;
    let mut name = String::new();
    let mut class_name = String::new();
    let mut pos: Option<[f32; 3]> = None;
    let mut section = String::new();
    let mut lines = text.lines().peekable();

    while let Some(line) = lines.next() {
        let t = line.trim();
        if t.starts_with('[') {
            section = t.trim_matches(|c| c == '[' || c == ']').to_string();
            continue;
        }
        if section == "metadata" {
            if let Some(v) = t.strip_prefix("name = ") {
                name = v.trim_matches('"').to_string();
            } else if let Some(v) = t.strip_prefix("class_name = ") {
                class_name = v.trim_matches('"').to_string();
            }
        } else if section == "transform" && t.starts_with("position = [") {
            // Values sit on the following lines, one per axis.
            let mut vals = Vec::with_capacity(3);
            for _ in 0..3 {
                let l = lines.next()?.trim().trim_end_matches(',').to_string();
                vals.push(l.parse::<f32>().ok()?);
            }
            pos = Some([vals[0], vals[1], vals[2]]);
        }
    }
    pos.map(|p| Instance { name, class_name, pos: p })
}

fn collect(root: &Path, out: &mut Vec<Instance>) {
    if out.len() >= MAX_INSTANCES {
        return;
    }
    let Ok(rd) = fs::read_dir(root) else { return };
    let mut dirs: Vec<PathBuf> = Vec::new();
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            // Skip engine bookkeeping.
            if p.file_name().map_or(false, |n| n == ".eustress") {
                continue;
            }
            dirs.push(p);
        } else if p.file_name().map_or(false, |n| n == "_instance.toml") {
            if let Some(i) = parse_instance(&p) {
                out.push(i);
                if out.len() >= MAX_INSTANCES {
                    return;
                }
            }
        }
    }
    for d in dirs {
        collect(&d, out);
        if out.len() >= MAX_INSTANCES {
            return;
        }
    }
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

fn cosine(a: &[f32], b: &[f32]) -> f32 {
    let n = a.len().min(b.len());
    let (mut d, mut na, mut nb) = (0.0f32, 0.0f32, 0.0f32);
    for i in 0..n {
        d += a[i] * b[i];
        na += a[i] * a[i];
        nb += b[i] * b[i];
    }
    d / (na.sqrt() * nb.sqrt()).max(1e-8)
}

struct Lcg(u64);
impl Lcg {
    fn next_f32(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((self.0 >> 33) as f32) / (u32::MAX as f32)
    }
}

fn main() {
    let space = PathBuf::from(
        r"C:\Users\miksu\Documents\Eustress\Business & Ops\Spaces\Mountain Ascension\Workspace",
    );
    println!("Spatial recall over REAL Mountain Ascension geometry\n");
    if !space.exists() {
        eprintln!("  Space not found at {}", space.display());
        std::process::exit(1);
    }

    let t0 = std::time::Instant::now();
    let mut inst = Vec::new();
    collect(&space, &mut inst);
    println!("  instances loaded : {} (in {:.1}s)", inst.len(), t0.elapsed().as_secs_f64());
    if inst.len() < 100 {
        eprintln!("  too few instances with positions to measure");
        std::process::exit(1);
    }

    // Real extent, used to scale the encoder to this world.
    let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
    for i in &inst {
        for a in 0..3 {
            lo[a] = lo[a].min(i.pos[a]);
            hi[a] = hi[a].max(i.pos[a]);
        }
    }
    let extent = (0..3).map(|a| hi[a] - lo[a]).fold(0.0f32, f32::max);
    println!("  world extent     : {:.0} units", extent);
    println!(
        "  bounds           : x[{:.0},{:.0}] y[{:.0},{:.0}] z[{:.0},{:.0}]",
        lo[0], hi[0], lo[1], hi[1], lo[2], hi[2]
    );

    let mut classes: std::collections::HashMap<&str, usize> = Default::default();
    for i in &inst {
        *classes.entry(i.class_name.as_str()).or_default() += 1;
    }
    let mut cv: Vec<_> = classes.into_iter().collect();
    cv.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
    println!("  classes          : {}", cv.iter().take(5)
        .map(|(c, n)| format!("{c}={n}")).collect::<Vec<_>>().join(" "));

    // Band count must resolve the world's actual packing density, not just its
    // extent. The finest band has wavelength ~extent/2^(bands-1); with parts
    // sitting ~1 unit apart in a ~5000-unit world, 12 bands bottoms out at
    // ~2.4 units and cannot separate true neighbours from their neighbours'
    // neighbours. Size the bands from the density instead.
    let bands = {
        let target_resolution = 0.5f32; // units we need to tell apart
        let needed = (extent.max(1.0) / target_resolution).log2().ceil() as usize + 1;
        needed.clamp(8, 20)
    };
    println!("  bands            : {bands} (finest wavelength ~{:.2} units)",
             extent / (1u32 << (bands - 1)) as f32);
    let encoder = JourneyEncoder::new(bands, extent.max(1.0));
    let codes: Vec<Vec<f32>> = inst
        .iter()
        .map(|i| encoder.query_at_place(i.pos))
        .collect();
    println!("  encoder width    : {}\n", encoder.dim());

    // ---- top-1 retrieval vs brute-force ground truth ----
    let mut rng = Lcg(0xA5A5_1234);
    let mut hits = 0usize;
    let mut penalty_sum = 0.0f32;
    let mut true_dist_sum = 0.0f32;

    for _ in 0..PROBES {
        // Probe a real location, jittered, so queries are in-distribution but
        // not identical to any stored point.
        let src = &inst[(rng.next_f32() * (inst.len() - 1) as f32) as usize];
        let jitter = extent * 0.01;
        let q = [
            src.pos[0] + (rng.next_f32() - 0.5) * jitter,
            src.pos[1] + (rng.next_f32() - 0.5) * jitter,
            src.pos[2] + (rng.next_f32() - 0.5) * jitter,
        ];

        // Ground truth: brute-force Euclidean nearest.
        let mut best_true = (f32::MAX, 0usize);
        for (i, e) in inst.iter().enumerate() {
            let d = dist(q, e.pos);
            if d < best_true.0 {
                best_true = (d, i);
            }
        }

        // Encoder's answer.
        let qc = encoder.query_at_place(q);
        let mut best_enc = (f32::MIN, 0usize);
        for (i, c) in codes.iter().enumerate() {
            let s = cosine(&qc, c);
            if s > best_enc.0 {
                best_enc = (s, i);
            }
        }

        if best_enc.1 == best_true.1 {
            hits += 1;
        }
        true_dist_sum += best_true.0;
        penalty_sum += dist(q, inst[best_enc.1].pos) - best_true.0;
    }

    let top1 = 100.0 * hits as f32 / PROBES as f32;
    let mean_true = true_dist_sum / PROBES as f32;
    let mean_penalty = penalty_sum / PROBES as f32;

    println!("  --- top-1 retrieval vs brute-force ground truth ({PROBES} probes) ---");
    println!("    top-1 accuracy        : {top1:.1}%");
    println!("    mean true NN distance : {mean_true:.2} units");
    println!("    mean extra distance   : {mean_penalty:.2} units when it missed");

    // A concrete lookup, shown end to end.
    println!("\n  --- example: what is at this real coordinate? ---");
    let probe = inst[inst.len() / 3].pos;
    let qc = encoder.query_at_place(probe);
    let mut scored: Vec<(f32, usize)> =
        codes.iter().enumerate().map(|(i, c)| (cosine(&qc, c), i)).collect();
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    println!("    query ({:.0},{:.0},{:.0})", probe[0], probe[1], probe[2]);
    for (s, i) in scored.iter().take(4) {
        let e = &inst[*i];
        println!(
            "      sim {:.4}  d={:6.1}  {:<28} [{}]",
            s, dist(probe, e.pos), e.name.chars().take(28).collect::<String>(), e.class_name
        );
    }

    let pass = top1 >= 60.0;
    println!(
        "\n  RESULT: {}",
        if pass {
            "PASS — the encoding preserves real spatial structure"
        } else {
            "FAIL — encoding does not retrieve real neighbours reliably"
        }
    );
    if !pass {
        std::process::exit(1);
    }
}
