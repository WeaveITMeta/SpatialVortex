//! containment_benchmark — establish the FLOOR for containment-structure retrieval.
//!
//! The question this answers: given an instance in a real Eustress Space, can
//! anything retrieve its true siblings (nodes sharing a containment parent)
//! better than a 1994 ranking function over their names?
//!
//! Until a learned method clears that bar on this corpus, no claim about
//! spatial intelligence here is testable. So this measures the bar first and
//! keeps the learned arm out of it entirely.
//!
//! # The twin control is the whole experiment
//!
//! 80.7% of positioned instances share a base name with some other instance —
//! the corpus is largely prefab copies. That duplication is what produced the
//! retracted "proximity is 18x baseline" result: copies sit next to copies, so
//! ANY signal correlated with copy-ness looks like semantics.
//!
//! So every arm is scored twice:
//!
//!   RAW        every candidate in the Space is eligible.
//!   CONTROLLED candidates whose BASE NAME matches the query's are removed
//!              from the pool — both positives and negatives. What survives is
//!              retrieval that cannot be explained by "found my own twin".
//!
//! An arm that wins RAW and collapses CONTROLLED has rediscovered duplication,
//! not structure. That is the failure this harness exists to catch.
//!
//! # Arms
//!
//! * `random`   — shuffled ranking. The true floor.
//! * `class`    — rank by matching `class_name`. Structure, zero text.
//! * `char3`    — character-trigram Jaccard over names.
//! * `tfidf`    — cosine over TF-IDF of name tokens, IDF fit per Space.
//! * `bm25`     — Okapi BM25 (k1=1.2, b=0.75). The bar to beat.
//! * `proximity`— rank by Euclidean distance.
//!
//! # A prediction this harness falsified, recorded rather than deleted
//!
//! `proximity` was added as a positive control on the assumption that it was
//! the retracted 18x signal and would therefore collapse under the twin
//! control. It did not: MRR 0.578 raw -> 0.571 controlled, ~10x the floor in
//! both. The assumption was wrong, not the control.
//!
//! The retracted claim was proximity -> SEMANTIC SIMILARITY. This task asks
//! proximity -> CONTAINMENT SIBLINGHOOD, and siblings in an authored 3D scene
//! are co-located BY CONSTRUCTION: a car's wheels sit at the car. Geometry
//! predicting siblinghood is near-tautological, and measuring it only confirms
//! the scenes are authored sanely.
//!
//! The consequence is the useful part. Proximity is free and scores 0.57;
//! BM25 scores 0.11. So the bar a learned containment method must clear is
//! NOT the lexical baseline — it is "just read the coordinates". Any method
//! that beats BM25 but loses to proximity has bought nothing.
//!
//! Run:
//!   cargo run --release --example containment_benchmark -- <universe-root>...

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use vortex::ml::eustress_schema::{load_space, AiExportPolicy};

// ---------------------------------------------------------------- tokens ----

/// Split an instance name into lowercase word tokens. Handles the three
/// conventions actually present in the corpus: CamelCase, snake_case, and
/// trailing numeric suffixes (`Wall_012`).
fn tokenize(name: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut prev_lower = false;
    for ch in name.chars() {
        if ch.is_alphanumeric() {
            // a lower->upper transition starts a new CamelCase word
            if ch.is_uppercase() && prev_lower && !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
            cur.push(ch.to_ascii_lowercase());
            prev_lower = ch.is_lowercase();
        } else {
            if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
            prev_lower = false;
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out.retain(|t| !t.is_empty());
    out
}

/// The identity a prefab copy shares with its twins: the name with trailing
/// digits and separators stripped. `Wall_012`, `Wall 12` and `Wall` all reduce
/// to `wall`. This is the key the twin control removes on.
fn base_name(name: &str) -> String {
    let t = name.trim();
    let stripped = t.trim_end_matches(|c: char| c.is_ascii_digit() || c == '_' || c == '-' || c == ' ' || c == '.');
    let s = if stripped.is_empty() { t } else { stripped };
    s.to_ascii_lowercase()
}

fn trigrams(s: &str) -> HashSet<String> {
    let cs: Vec<char> = format!("  {}  ", s.to_ascii_lowercase()).chars().collect();
    let mut out = HashSet::new();
    for w in cs.windows(3) {
        out.insert(w.iter().collect::<String>());
    }
    out
}

// ------------------------------------------------------------------ data ----

struct Node {
    name: String,
    base: String,
    tokens: Vec<String>,
    class: String,
    parent: String,
    pos: [f32; 3],
}

struct SpaceData {
    label: String,
    nodes: Vec<Node>,
    /// parent path -> member indices
    groups: HashMap<String, Vec<usize>>,
    /// token -> how many nodes contain it (document frequency)
    df: HashMap<String, usize>,
    avg_len: f32,
}

impl SpaceData {
    fn build(label: String, nodes: Vec<Node>) -> Self {
        let mut groups: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, n) in nodes.iter().enumerate() {
            groups.entry(n.parent.clone()).or_default().push(i);
        }
        let mut df: HashMap<String, usize> = HashMap::new();
        for n in &nodes {
            for t in n.tokens.iter().collect::<HashSet<_>>() {
                *df.entry(t.clone()).or_insert(0) += 1;
            }
        }
        let avg_len = if nodes.is_empty() {
            1.0
        } else {
            nodes.iter().map(|n| n.tokens.len() as f32).sum::<f32>() / nodes.len() as f32
        };
        SpaceData { label, nodes, groups, df, avg_len }
    }

    fn idf(&self, term: &str) -> f32 {
        let n = self.nodes.len() as f32;
        let d = *self.df.get(term).unwrap_or(&0) as f32;
        // BM25 idf with the +0.5 smoothing, floored so common terms cannot go negative
        (((n - d + 0.5) / (d + 0.5)) + 1.0).ln().max(0.0)
    }
}

// ------------------------------------------------------------------ arms ----

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Arm {
    Random,
    Class,
    Char3,
    TfIdf,
    Bm25,
    Proximity,
}

const ARMS: [Arm; 6] = [Arm::Random, Arm::Class, Arm::Char3, Arm::TfIdf, Arm::Bm25, Arm::Proximity];

impl Arm {
    fn name(self) -> &'static str {
        match self {
            Arm::Random => "random",
            Arm::Class => "class",
            Arm::Char3 => "char3",
            Arm::TfIdf => "tfidf",
            Arm::Bm25 => "bm25",
            Arm::Proximity => "proximity",
        }
    }
}

/// Deterministic per-(query,candidate) pseudo-random score. Avoids an RNG
/// dependency and makes the random arm reproducible run to run.
fn hash_score(a: usize, b: usize) -> f32 {
    let mut h = (a as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (b as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    h ^= h >> 29;
    h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^= h >> 32;
    (h as u32) as f32 / u32::MAX as f32
}

/// Everything about the QUERY that does not change as candidates vary. Built
/// once per query; without this the idf lookups and trigram set were rebuilt
/// for every candidate, which dominated the whole run.
struct QueryCtx<'a> {
    idx: usize,
    node: &'a Node,
    /// term -> (count, idf), deduplicated
    terms: Vec<(&'a str, f32, f32)>,
    tfidf_norm: f32,
    trigrams: HashSet<String>,
}

impl<'a> QueryCtx<'a> {
    fn build(sp: &'a SpaceData, qi: usize) -> Self {
        let node = &sp.nodes[qi];
        let mut counts: HashMap<&str, f32> = HashMap::new();
        for t in &node.tokens {
            *counts.entry(t.as_str()).or_insert(0.0) += 1.0;
        }
        let mut terms: Vec<(&str, f32, f32)> =
            counts.into_iter().map(|(t, c)| (t, c, sp.idf(t))).collect();
        terms.sort_by(|a, b| a.0.cmp(b.0));
        let tfidf_norm = terms.iter().map(|(_, c, i)| (c * i) * (c * i)).sum::<f32>().sqrt();
        QueryCtx { idx: qi, node, terms, tfidf_norm, trigrams: trigrams(&node.name) }
    }
}

fn score(arm: Arm, sp: &SpaceData, q: &QueryCtx, ci: usize) -> f32 {
    let c = &sp.nodes[ci];
    match arm {
        Arm::Random => hash_score(q.idx, ci),
        Arm::Class => {
            if q.node.class == c.class { 1.0 } else { 0.0 }
        }
        Arm::Char3 => {
            let b = trigrams(&c.name);
            let inter = q.trigrams.intersection(&b).count() as f32;
            let union = q.trigrams.union(&b).count() as f32;
            if union == 0.0 { 0.0 } else { inter / union }
        }
        Arm::TfIdf => {
            let mut cv: HashMap<&str, f32> = HashMap::new();
            for t in &c.tokens { *cv.entry(t.as_str()).or_insert(0.0) += 1.0; }
            let mut dot = 0.0;
            let mut cn = 0.0;
            for (t, cf) in &cv {
                let w = cf * sp.idf(t);
                cn += w * w;
            }
            for (t, qc, qi_) in &q.terms {
                if let Some(cf) = cv.get(t) {
                    dot += (qc * qi_) * (cf * qi_);
                }
            }
            if q.tfidf_norm == 0.0 || cn == 0.0 { 0.0 } else { dot / (q.tfidf_norm * cn.sqrt()) }
        }
        Arm::Bm25 => {
            const K1: f32 = 1.2;
            const B: f32 = 0.75;
            let dl = c.tokens.len() as f32;
            let mut cf: HashMap<&str, f32> = HashMap::new();
            for t in &c.tokens { *cf.entry(t.as_str()).or_insert(0.0) += 1.0; }
            let mut s = 0.0;
            for (t, _, idf) in &q.terms {
                let f = *cf.get(*t).unwrap_or(&0.0);
                if f == 0.0 { continue; }
                let denom = f + K1 * (1.0 - B + B * dl / sp.avg_len.max(1e-6));
                s += idf * (f * (K1 + 1.0)) / denom;
            }
            s
        }
        Arm::Proximity => {
            let p = q.node.pos;
            let d = ((p[0] - c.pos[0]).powi(2) + (p[1] - c.pos[1]).powi(2) + (p[2] - c.pos[2]).powi(2)).sqrt();
            -d // nearer ranks higher
        }
    }
}

// --------------------------------------------------------------- metrics ----

#[derive(Default, Clone)]
struct Metrics {
    queries: usize,
    mrr: f64,
    hits1: usize,
    hits10: usize,
}

impl Metrics {
    fn add(&mut self, rank_of_first_hit: Option<usize>) {
        self.queries += 1;
        if let Some(r) = rank_of_first_hit {
            self.mrr += 1.0 / (r as f64 + 1.0);
            if r == 0 { self.hits1 += 1; }
            if r < 10 { self.hits10 += 1; }
        }
    }
    fn mrr(&self) -> f64 { if self.queries == 0 { 0.0 } else { self.mrr / self.queries as f64 } }
    fn h1(&self) -> f64 { if self.queries == 0 { 0.0 } else { self.hits1 as f64 / self.queries as f64 } }
    fn h10(&self) -> f64 { if self.queries == 0 { 0.0 } else { self.hits10 as f64 / self.queries as f64 } }
}

/// Candidates offered per query: exactly one true sibling plus POOL-1
/// non-siblings. Ranking the WHOLE Space instead made the metric a function of
/// sibling-group size — `Universe1/Space1` has a single group holding 3,214 of
/// its 5,246 grouped nodes, so most candidates were positives and `random`
/// scored MRR 0.52. With a fixed pool the floor is 1/POOL by construction and
/// the number means the same thing in every Space.
const POOL: usize = 100;

/// Evaluate one arm over one Space. `controlled` removes every candidate whose
/// base name equals the query's — the twin control.
fn eval(sp: &SpaceData, arm: Arm, controlled: bool, max_queries: usize) -> Metrics {
    let mut m = Metrics::default();
    // queries are nodes that HAVE at least one sibling
    let mut qs: Vec<usize> = Vec::new();
    for (parent, members) in &sp.groups {
        if parent.is_empty() || members.len() < 2 { continue; }
        qs.extend(members.iter().copied());
    }
    qs.sort_unstable();
    // deterministic subsample, evenly spread rather than a prefix
    if qs.len() > max_queries {
        let stride = qs.len() as f64 / max_queries as f64;
        qs = (0..max_queries).map(|i| qs[(i as f64 * stride) as usize]).collect();
    }

    for &qi in &qs {
        let q = &sp.nodes[qi];
        let positives: HashSet<usize> = sp.groups[&q.parent]
            .iter()
            .copied()
            .filter(|&j| j != qi)
            .filter(|&j| !controlled || sp.nodes[j].base != q.base)
            .collect();
        // A controlled query whose every sibling was a twin has nothing left to
        // find; excluding it is required, otherwise the control would score as
        // failure rather than as "not applicable".
        if positives.is_empty() { continue; }

        // ONE positive, POOL-1 negatives. The positive is chosen deterministically
        // from `positives` and the negatives are strided across the Space's
        // non-siblings, so every arm sees an identical pool for a given query.
        let pos_list: Vec<usize> = { let mut v: Vec<usize> = positives.iter().copied().collect(); v.sort_unstable(); v };
        let positive = pos_list[qi % pos_list.len()];

        let negatives: Vec<usize> = (0..sp.nodes.len())
            .filter(|&j| j != qi)
            .filter(|&j| !positives.contains(&j))
            .filter(|&j| sp.nodes[j].parent != q.parent)
            .filter(|&j| !controlled || sp.nodes[j].base != q.base)
            .collect();
        if negatives.len() < POOL - 1 { continue; }
        let stride = negatives.len() as f64 / (POOL - 1) as f64;
        let mut pool: Vec<usize> = (0..POOL - 1)
            .map(|i| negatives[(i as f64 * stride) as usize])
            .collect();
        pool.push(positive);

        // Score ONCE per candidate, then sort. Scoring inside the comparator
        // would call `score` O(n log n) times and reallocate a HashMap on every
        // one of them — that turns a seconds-long run into an hours-long one.
        let qctx = QueryCtx::build(sp, qi);
        let mut scored: Vec<(f32, usize)> =
            pool.iter().map(|&j| (score(arm, sp, &qctx, j), j)).collect();
        if scored.len() < 2 { continue; }

        // ties break on index so every arm sees the same deterministic order
        scored.sort_by(|a, b| {
            b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal).then(a.1.cmp(&b.1))
        });

        let rank = scored.iter().position(|&(_, j)| j == positive);
        m.add(rank);
    }
    m
}

// ------------------------------------------------------------------ main ----

fn collect_spaces(roots: &[PathBuf]) -> Vec<(String, PathBuf)> {
    let mut out = Vec::new();
    for r in roots {
        let spaces_dir = if r.join("Spaces").is_dir() { r.join("Spaces") } else { r.clone() };
        let Ok(rd) = std::fs::read_dir(&spaces_dir) else { continue };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                let label = format!(
                    "{}/{}",
                    r.file_name().and_then(|s| s.to_str()).unwrap_or("?"),
                    p.file_name().and_then(|s| s.to_str()).unwrap_or("?")
                );
                out.push((label, p));
            }
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("usage: containment_benchmark <universe-root>...");
        eprintln!("  e.g. containment_benchmark \"C:/Users/.../Eustress/Tucson\"");
        std::process::exit(2);
    }
    let roots: Vec<PathBuf> = args.iter().map(PathBuf::from).collect();
    let spaces = collect_spaces(&roots);

    println!("CONTAINMENT RETRIEVAL BENCHMARK");
    println!("task: given an instance, retrieve a SIBLING (same containment parent)");
    println!("policy: LegacyOwnerConsent (absent `ai` = predates the flag)\n");

    let mut loaded: Vec<SpaceData> = Vec::new();
    for (label, path) in &spaces {
        let ls = match load_space(Path::new(path), AiExportPolicy::LegacyOwnerConsent) {
            Ok(v) => v,
            Err(e) => {
                println!("  skip {:<38} load error: {}", label, e);
                continue;
            }
        };
        if ls.nodes.len() < 20 {
            println!(
                "  skip {:<38} only {} admitted (scanned {}, parse-fail {}, no-transform {}, opt-out {})",
                label, ls.nodes.len(), ls.report.files_scanned, ls.report.parse_failures,
                ls.report.excluded_no_transform, ls.report.excluded_opt_out
            );
            continue;
        }
        let nodes: Vec<Node> = ls
            .nodes
            .iter()
            .enumerate()
            .map(|(i, n)| {
                let name = n.semantic_text.clone();
                Node {
                    base: base_name(&name),
                    tokens: tokenize(&name),
                    name,
                    class: ls.classes.get(i).cloned().unwrap_or_default(),
                    parent: ls.parents.get(i).cloned().unwrap_or_default(),
                    pos: [n.position.x, n.position.y, n.position.z],
                }
            })
            .collect();
        let sd = SpaceData::build(label.clone(), nodes);
        let sib_groups = sd.groups.values().filter(|v| v.len() >= 2).count();
        if sib_groups == 0 {
            println!(
                "  skip {:<38} {} nodes but 0 sibling-groups (distinct parents: {}; sample parent: {:?})",
                label, sd.nodes.len(), sd.groups.len(),
                sd.groups.keys().next().map(|s| s.as_str()).unwrap_or("")
            );
            continue;
        }
        println!(
            "  loaded {:<38} {:6} nodes  {:5} sibling-groups  scanned {}",
            sd.label, sd.nodes.len(), sib_groups, ls.report.files_scanned
        );
        loaded.push(sd);
    }
    if loaded.is_empty() {
        eprintln!("\nno Space had >=20 admitted nodes with sibling groups — nothing to measure");
        std::process::exit(1);
    }

    // how duplicated is this corpus, really?
    let total: usize = loaded.iter().map(|s| s.nodes.len()).sum();
    let mut twin = 0usize;
    for sp in &loaded {
        let mut counts: HashMap<&str, usize> = HashMap::new();
        for n in &sp.nodes { *counts.entry(n.base.as_str()).or_insert(0) += 1; }
        twin += sp.nodes.iter().filter(|n| counts[n.base.as_str()] > 1).count();
    }
    println!(
        "\n  corpus: {} nodes across {} Spaces; {} ({:.1}%) share a base name with another node",
        total, loaded.len(), twin, 100.0 * twin as f64 / total.max(1) as f64
    );

    const MAXQ: usize = 400;
    for controlled in [false, true] {
        println!(
            "\n{}",
            if controlled {
                "== TWIN-CONTROLLED — candidates sharing the query's base name removed =="
            } else {
                "== RAW — every candidate eligible =="
            }
        );
        println!("  {:<10} {:>8} {:>8} {:>8} {:>8}   (pool of {}, 1 positive -> random floor MRR ~{:.3}, Hits@1 {:.3})",
            "arm", "MRR", "Hits@1", "Hits@10", "queries", POOL,
            (1..=POOL).map(|r| 1.0/r as f64).sum::<f64>()/POOL as f64, 1.0/POOL as f64);
        let mut rows: Vec<(Arm, Metrics)> = Vec::new();
        for arm in ARMS {
            let mut agg = Metrics::default();
            for sp in &loaded {
                let m = eval(sp, arm, controlled, MAXQ);
                agg.queries += m.queries;
                agg.mrr += m.mrr;
                agg.hits1 += m.hits1;
                agg.hits10 += m.hits10;
            }
            rows.push((arm, agg));
        }
        rows.sort_by(|a, b| b.1.mrr().partial_cmp(&a.1.mrr()).unwrap());
        for (arm, m) in &rows {
            println!(
                "  {:<10} {:>8.4} {:>8.4} {:>8.4} {:>8}",
                arm.name(), m.mrr(), m.h1(), m.h10(), m.queries
            );
        }
        if let (Some(best), Some(rnd)) = (
            rows.first(),
            rows.iter().find(|(a, _)| *a == Arm::Random),
        ) {
            let lift = if rnd.1.mrr() > 0.0 { best.1.mrr() / rnd.1.mrr() } else { f64::INFINITY };
            println!("  -> best `{}` at {:.2}x the random floor", best.0.name(), lift);
        }
    }

    println!("\nread this as: an arm that wins RAW but collapses CONTROLLED has found");
    println!("prefab duplication, not containment structure. `proximity` is in the");
    println!("table as a positive control — it is the retracted signal and SHOULD die.");
}
