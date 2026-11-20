//! Running suites: items are answered in parallel, graded, timed, and
//! summarised.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use vortex_kernel::{Mechanism, Response};

use crate::report::SuiteResult;
use crate::stats::{percentile, wilson};
use crate::{Agent, Suite};

pub fn run_suite(agent: &dyn Agent, suite: &dyn Suite, threads: usize) -> (SuiteResult, Vec<Response>) {
    let items = suite.items();
    let n = items.len();
    let next = AtomicUsize::new(0);
    let threads = threads.max(1).min(n.max(1));
    let mut slots: Vec<Option<(Response, f64)>> = vec![None; n];
    let t0 = Instant::now();
    std::thread::scope(|s| {
        let handles: Vec<_> = (0..threads)
            .map(|_| {
                let next = &next;
                s.spawn(move || {
                    let mut local = Vec::new();
                    loop {
                        let i = next.fetch_add(1, Ordering::Relaxed);
                        if i >= n {
                            break;
                        }
                        let t = Instant::now();
                        let r = agent.respond(&items[i].task);
                        local.push((i, r, t.elapsed().as_secs_f64() * 1000.0));
                    }
                    local
                })
            })
            .collect();
        for h in handles {
            for (i, r, ms) in h.join().expect("agent thread panicked") {
                slots[i] = Some((r, ms));
            }
        }
    });
    let wall = t0.elapsed().as_secs_f64();

    let mut responses = Vec::with_capacity(n);
    let mut latencies = Vec::with_capacity(n);
    let (mut score, mut brier, mut abstain) = (0.0, 0.0, 0usize);
    let mut mechanisms = std::collections::BTreeMap::new();
    let mut failed_ids = Vec::new();
    for (i, slot) in slots.into_iter().enumerate() {
        let (r, ms) = slot.expect("every item answered");
        let g = suite.grade(i, &r).clamp(0.0, 1.0);
        score += g;
        brier += (r.confidence - g).powi(2);
        if r.is_abstain() {
            abstain += 1;
        }
        if g < 0.999 {
            failed_ids.push(items[i].id.clone());
        }
        *mechanisms.entry(r.mechanism).or_insert(0usize) += 1;
        latencies.push(ms);
        responses.push(r);
    }
    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let acc = if n == 0 { 0.0 } else { score / n as f64 };
    let (lo, hi) = wilson(acc, n, 1.96);
    let chance = suite.chance();
    let normalized = if chance < 1.0 { (acc - chance) / (1.0 - chance) } else { 0.0 };
    let dominant = mechanisms
        .iter()
        .filter(|(m, _)| **m != Mechanism::None)
        .max_by_key(|(_, c)| **c)
        .map(|(m, _)| *m)
        .unwrap_or(Mechanism::None);
    let result = SuiteResult {
        id: suite.id().to_string(),
        capability: suite.capability(),
        source: suite.source(),
        sealed: suite.sealed(),
        n,
        accuracy: acc,
        ci95: (lo, hi),
        chance,
        normalized,
        above_chance: n > 0 && lo > chance,
        abstain_rate: if n == 0 { 0.0 } else { abstain as f64 / n as f64 },
        brier: if n == 0 { 0.0 } else { brier / n as f64 },
        latency_ms_mean: if n == 0 { 0.0 } else { latencies.iter().sum::<f64>() / n as f64 },
        latency_ms_p95: percentile(&latencies, 0.95),
        wall_seconds: wall,
        mechanism: dominant,
        mechanisms,
        extra: suite.summarize(&responses),
        failed_ids: if suite.sealed() { vec![] } else { failed_ids.into_iter().take(50).collect() },
        contamination_rate: None,
    };
    (result, responses)
}

pub fn run_all(agent: &dyn Agent, suites: &[Box<dyn Suite>], threads: usize, mut progress: impl FnMut(&SuiteResult)) -> Vec<SuiteResult> {
    suites
        .iter()
        .map(|s| {
            let (r, _) = run_suite(agent, s.as_ref(), threads);
            progress(&r);
            r
        })
        .collect()
}
