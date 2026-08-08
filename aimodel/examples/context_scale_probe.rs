//! Measure what the context ring architecture actually scales to.
//!
//! Distinguishes three quantities that get conflated when people say
//! "context window":
//!
//! - **addressable**  — entries the buffer can represent = `cap * (rings + 1)`
//! - **view width**   — vectors attention sees per step  = `cap + rings`
//! - **full-res**     — entries readable verbatim        = `cap` (the open ring)
//!
//! Only the first can reach 1M cheaply. The third is the honest "context
//! window" in the transformer sense.

use vortex::ml::context_ring::{ContextEntry, ContextRingBuffer, ContextRingConfig};

fn mib(bytes: usize) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}

/// Resident bytes for embeddings held at full resolution.
///
/// Sealed-but-unmerged rings retain their entries; only merged rings drop them
/// in favour of a summary. So the steady-state cost is
/// `(max_rings + 1) * ring_capacity * dim * 4`.
fn resident_bytes(c: &ContextRingConfig) -> usize {
    (c.max_rings + 1) * c.ring_capacity * c.dim * 4
}

fn row(label: &str, c: &ContextRingConfig) {
    println!(
        "  {label:<26} cap={:<7} rings={:<6} addressable={:<12} view={:<7} full-res={:<7} resident={:>9.1} MiB",
        c.ring_capacity,
        c.max_rings,
        c.addressable_entries(),
        c.view_width(),
        c.ring_capacity,
        mib(resident_bytes(c))
    );
}

fn main() {
    println!("Context scaling — addressable vs view width vs full resolution\n");

    let d = ContextRingConfig::new();
    row("default", &d);

    // Several ways to reach ~1M addressable entries.
    let a = ContextRingConfig::new().with_dim(256).with_ring_capacity(1024).with_max_rings(1023);
    row("1M via 1024 x 1024", &a);

    let b = ContextRingConfig::new().with_dim(256).with_ring_capacity(4096).with_max_rings(255);
    row("1M via 4096 x 256", &b);

    let c = ContextRingConfig::new().with_dim(256).with_ring_capacity(16384).with_max_rings(63);
    row("1M via 16384 x 64", &c);

    // Beyond 1M.
    let e = ContextRingConfig::new().with_dim(256).with_ring_capacity(4096).with_max_rings(4095);
    row("16M via 4096 x 4096", &e);

    let f = ContextRingConfig::new().with_dim(64).with_ring_capacity(8192).with_max_rings(8191);
    row("67M via 8192 x 8192 d64", &f);

    // ---- The selected configuration, exercised for real ----
    let w = ContextRingConfig::wide_16m();
    println!("\n=== SELECTED: wide_16m() — live run ===");
    println!("  addressable  = {}", w.addressable_entries());
    println!("  view width   = {}", w.view_width());
    println!("  full-res     = {} (open ring, verbatim)", w.ring_capacity);
    println!("  retention    = {} (off => memory scales with rings, not entries)", w.retain_sealed_entries);
    println!("  resident     = {:.1} MiB   [vs {:.0} MiB with retention on]",
             mib(w.resident_bytes()),
             mib(ContextRingConfig::wide_16m().with_retain_sealed_entries(true).resident_bytes()));

    let mut wide = ContextRingBuffer::new(w.clone());
    let pushes = 1_000_000usize;
    let t0 = std::time::Instant::now();
    for i in 0..pushes {
        wide.push(ContextEntry::new(vec![(i % 251) as f32 * 0.004; 256]));
    }
    let elapsed = t0.elapsed();

    let wv = wide.view();
    println!("\n  pushed            = {pushes}");
    println!("  elapsed           = {:.2}s  ({:.0} entries/sec)",
             elapsed.as_secs_f64(), pushes as f64 / elapsed.as_secs_f64());
    println!("  sealed rings      = {}", wide.sealed_count());
    println!("  view vectors      = {} (recent {} + compressed {})",
             wv.len(), wv.recent.len(), wv.compressed.len());
    println!("  total represented = {}", wide.total_represented());
    println!("  headroom          = {}", wv.headroom);

    let ok_bounded = wv.len() <= w.view_width();
    let ok_complete = wide.total_represented() >= pushes;
    let ok_fullres = wv.recent.len() <= w.ring_capacity;
    println!("\n  view bounded by {}      : {}", w.view_width(), if ok_bounded { "yes" } else { "NO" });
    println!("  all {pushes} represented : {}", if ok_complete { "yes" } else { "NO" });
    println!("  open ring full-res      : {}", if ok_fullres { "yes" } else { "NO" });

    if !(ok_bounded && ok_complete && ok_fullres) {
        eprintln!("wide_16m invariants violated");
        std::process::exit(1);
    }

    println!("\nInvariant check at 1M-addressable config (1024 x 1024, dim 32)");
    // dim 32 keeps the probe fast; the invariants are dim-independent.
    let probe = ContextRingConfig::new()
        .with_dim(32)
        .with_ring_capacity(1024)
        .with_max_rings(1023)
        .with_merge_span(4)
        .with_headroom(64);
    println!("  addressable = {}", probe.addressable_entries());
    println!("  view width  = {}", probe.view_width());

    let mut buf = ContextRingBuffer::new(probe.clone());
    let n = 200_000usize; // enough to seal ~195 rings without a 1 GiB probe
    for i in 0..n {
        buf.push(ContextEntry::new(vec![(i % 97) as f32 * 0.01; 32]));
    }

    let v = buf.view();
    println!("\n  pushed              = {n}");
    println!("  sealed rings        = {}", buf.sealed_count());
    println!("  view vectors        = {}  (recent {} + compressed {})", v.len(), v.recent.len(), v.compressed.len());
    println!("  total represented   = {}", buf.total_represented());
    println!("  headroom preserved  = {}", v.headroom);

    let bounded = v.len() <= probe.view_width();
    let complete = buf.total_represented() >= n;
    println!("\n  view stayed bounded      : {}", if bounded { "yes" } else { "NO" });
    println!("  all history represented  : {}", if complete { "yes" } else { "NO" });

    println!("\n  full-resolution entries  : {} (the open ring)", buf.open_len().max(0));
    println!("  compression of deep past : {} entries -> 1 vector per ring", probe.ring_capacity);

    if !(bounded && complete) {
        std::process::exit(1);
    }
}
