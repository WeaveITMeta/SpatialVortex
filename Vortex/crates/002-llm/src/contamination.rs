//! Train/eval contamination measurement.
//!
//! An evaluation item is *contaminated* when any of its word 13-grams also
//! occurs in the training text (the GPT-3 paper's convention). Items shorter
//! than 13 words are checked with their full word sequence instead, at a
//! minimum of 8 words; shorter items are reported as "too short to check"
//! rather than silently counted clean.

use std::collections::HashSet;

use serde::Serialize;
use vortex_kernel::fnv1a64;

use crate::tokenize::words;

pub const N: usize = 13;
pub const MIN_CHECKABLE: usize = 8;

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct ContaminationReport {
    pub suite: String,
    pub items: usize,
    pub checkable: usize,
    pub contaminated: usize,
    pub rate: f64,
    /// Per item: true when contaminated.
    #[serde(skip)]
    pub flags: Vec<bool>,
}

fn grams(ws: &[String], n: usize) -> impl Iterator<Item = u64> + '_ {
    ws.windows(n).map(|w| fnv1a64(w.join(" ").as_bytes()))
}

/// Check eval items (each the concatenated text a model would see) against
/// training lines. Training lines are streamed, so the corpus need not fit
/// in a set.
pub fn check<'a>(suite: &str, items: &[String], training: impl Iterator<Item = &'a str>) -> ContaminationReport {
    // Per item: the set of its n-gram hashes (n = min(13, len)).
    let mut per_item: Vec<(usize, HashSet<u64>)> = Vec::new();
    let mut lengths: HashSet<usize> = HashSet::new();
    let mut checkable = 0;
    for it in items {
        let ws = words(it);
        if ws.len() < MIN_CHECKABLE {
            per_item.push((0, HashSet::new()));
            continue;
        }
        checkable += 1;
        let n = ws.len().min(N);
        lengths.insert(n);
        per_item.push((n, grams(&ws, n).collect()));
    }
    let mut index: HashSet<u64> = HashSet::new();
    for (_, set) in &per_item {
        index.extend(set.iter().copied());
    }
    let mut seen: HashSet<u64> = HashSet::new();
    for line in training {
        let ws = words(line);
        for &n in &lengths {
            for g in grams(&ws, n) {
                if index.contains(&g) {
                    seen.insert(g);
                }
            }
        }
    }
    let flags: Vec<bool> = per_item.iter().map(|(n, set)| *n > 0 && set.iter().any(|g| seen.contains(g))).collect();
    let contaminated = flags.iter().filter(|f| **f).count();
    ContaminationReport {
        suite: suite.to_string(),
        items: items.len(),
        checkable,
        contaminated,
        rate: if checkable == 0 { 0.0 } else { contaminated as f64 / checkable as f64 },
        flags,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_overlap_and_ignores_short_items() {
        let train = ["the quick brown fox jumps over the lazy dog near the river bank today", "unrelated text"];
        let items = vec![
            "quick brown fox jumps over the lazy dog near the river bank today again".to_string(),
            "a completely different sentence that shares nothing with the training data".to_string(),
            "too short".to_string(),
        ];
        let r = check("t", &items, train.iter().copied());
        assert_eq!(r.checkable, 2);
        assert_eq!(r.contaminated, 1);
    }
}
