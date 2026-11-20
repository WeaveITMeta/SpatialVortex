//! Train/eval contamination measurement.
//!
//! An evaluation item is *contaminated* when any of its word 13-grams also
//! occurs in the training text (the GPT-3 paper's convention). Items shorter
//! than 13 words are checked with their full word sequence instead, at a
//! minimum of 8 words; shorter items are reported as "too short to check"
//! rather than silently counted clean.

use std::collections::HashSet;

use serde::Serialize;

use super::tokenize::words;

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

/// FNV-1a 64 of the words joined by single spaces, without building the
/// joined string. Identical to `fnv1a64(ws.join(" ").as_bytes())`.
fn gram_hash(ws: &[String]) -> u64 {
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for (i, w) in ws.iter().enumerate() {
        if i > 0 {
            h = (h ^ b' ' as u64).wrapping_mul(PRIME);
        }
        for b in w.as_bytes() {
            h = (h ^ *b as u64).wrapping_mul(PRIME);
        }
    }
    h
}

fn grams(ws: &[String], n: usize) -> impl Iterator<Item = u64> + '_ {
    ws.windows(n).map(gram_hash)
}

/// Check eval items (each the concatenated text a model would see) against
/// training lines. Training lines are streamed, so the corpus need not fit
/// in a set.
pub fn check<'a>(suite: &str, items: &[String], training: impl Iterator<Item = &'a str>) -> ContaminationReport {
    check_many(&[(suite, items)], training).pop().expect("one suite in, one report out")
}

/// [`check`] for several suites in ONE pass over the training text. Reading
/// and tokenising the corpus is the whole cost, so checking every suite
/// together costs about what checking one does. Reports come back in the
/// order the suites were given.
pub fn check_many<'a>(suites: &[(&str, &[String])], training: impl Iterator<Item = &'a str>) -> Vec<ContaminationReport> {
    // Per suite, per item: the set of its n-gram hashes (n = min(13, len)).
    let mut per_suite: Vec<Vec<(usize, HashSet<u64>)>> = Vec::with_capacity(suites.len());
    let mut lengths: Vec<usize> = Vec::new();
    let mut index: HashSet<u64> = HashSet::new();
    for (_, items) in suites {
        let mut per_item = Vec::with_capacity(items.len());
        for it in *items {
            let ws = words(it);
            if ws.len() < MIN_CHECKABLE {
                per_item.push((0, HashSet::new()));
                continue;
            }
            let n = ws.len().min(N);
            if !lengths.contains(&n) {
                lengths.push(n);
            }
            let set: HashSet<u64> = grams(&ws, n).collect();
            index.extend(set.iter().copied());
            per_item.push((n, set));
        }
        per_suite.push(per_item);
    }
    let mut seen: HashSet<u64> = HashSet::new();
    if !index.is_empty() {
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
    }
    suites
        .iter()
        .zip(per_suite)
        .map(|((suite, items), per_item)| {
            let flags: Vec<bool> = per_item.iter().map(|(n, set)| *n > 0 && set.iter().any(|g| seen.contains(g))).collect();
            let checkable = per_item.iter().filter(|(n, _)| *n > 0).count();
            let contaminated = flags.iter().filter(|f| **f).count();
            ContaminationReport {
                suite: suite.to_string(),
                items: items.len(),
                checkable,
                contaminated,
                rate: if checkable == 0 { 0.0 } else { contaminated as f64 / checkable as f64 },
                flags,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use vortex_kernel::fnv1a64;

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

    #[test]
    fn gram_hash_equals_fnv_of_the_joined_words() {
        for text in ["one", "two words", "the quick brown fox jumps over the lazy dog", "caf\u{e9} na\u{ef}ve r\u{e9}sum\u{e9} 42"] {
            let ws = words(text);
            assert_eq!(gram_hash(&ws), fnv1a64(ws.join(" ").as_bytes()), "{text}");
        }
        assert_eq!(gram_hash(&[]), fnv1a64(b""));
    }

    #[test]
    fn one_pass_over_many_suites_matches_one_pass_each() {
        let train = [
            "the quick brown fox jumps over the lazy dog near the river bank today",
            "plants need water and sunlight to grow tall and strong every single day",
            "unrelated text",
        ];
        let a = vec![
            "quick brown fox jumps over the lazy dog near the river bank today again".to_string(),
            "a completely different sentence that shares nothing with the training data".to_string(),
            "too short".to_string(),
        ];
        // Shorter than 13 words, so it is checked at its own length (9).
        let b = vec!["plants need water and sunlight to grow tall and".to_string(), "nine words that never appear in any training line".to_string()];
        let c: Vec<String> = vec![];
        let together = check_many(&[("a", &a), ("b", &b), ("c", &c)], train.iter().copied());
        let apart = vec![check("a", &a, train.iter().copied()), check("b", &b, train.iter().copied()), check("c", &c, train.iter().copied())];
        assert_eq!(together, apart);
        assert_eq!((together[0].contaminated, together[1].contaminated, together[2].items), (1, 1, 0));
        assert_eq!(together[1].flags, vec![true, false]);
    }
}
