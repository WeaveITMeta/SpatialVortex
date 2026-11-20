//! Interpolated Kneser-Ney n-gram language model.
//!
//! - Orders 2-4. N-grams are packed exactly into a `u128` (32 bits per
//!   word id), so there are no hash collisions in the counts.
//! - Highest order uses raw counts; lower orders use continuation counts
//!   N1+(* h w), the defining Kneser-Ney idea: a word's lower-order weight
//!   is how many distinct contexts it follows, not how often it occurs.
//! - One absolute discount per order, D = n1 / (n1 + 2 n2), estimated from
//!   the count-of-counts.
//! - The unigram level interpolates with a uniform distribution over the
//!   vocabulary, so no in-vocabulary word gets probability zero.

use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};
use std::time::Instant;

use serde::Serialize;

use crate::LanguageModel;
use crate::tokenize::words;

pub const UNK: u32 = 0;
pub const BOS: u32 = 1;
pub const EOS: u32 = 2;

/// Multiplicative hasher for `u128` keys; far faster than SipHash for
/// counting and adequate because keys are not attacker-controlled.
#[derive(Default)]
pub struct KeyHasher(u64);

impl Hasher for KeyHasher {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        for b in bytes {
            self.0 = (self.0 ^ *b as u64).wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    fn write_u128(&mut self, k: u128) {
        let lo = k as u64;
        let hi = (k >> 64) as u64;
        let x = lo.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ hi.rotate_left(29).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
        self.0 = x ^ (x >> 32);
    }
    fn write_u32(&mut self, k: u32) {
        self.write_u128(k as u128);
    }
}

type Map<V> = HashMap<u128, V, BuildHasherDefault<KeyHasher>>;

fn pack(ids: &[u32]) -> u128 {
    ids.iter().fold(0u128, |acc, &x| (acc << 32) | x as u128)
}

fn mask(k: usize) -> u128 {
    if k >= 4 { u128::MAX } else { (1u128 << (32 * k)) - 1 }
}

#[derive(Clone, Debug, Serialize)]
pub struct NgramConfig {
    pub order: usize,
    /// Words seen fewer times than this map to `<unk>`.
    pub min_count: u32,
}

impl Default for NgramConfig {
    fn default() -> Self {
        Self { order: 3, min_count: 2 }
    }
}

#[derive(Clone, Debug, Serialize, Default)]
pub struct TrainStats {
    pub sentences: usize,
    pub tokens: usize,
    pub vocab: usize,
    pub ngrams_per_order: Vec<usize>,
    pub discounts: Vec<f64>,
    pub seconds: f64,
}

pub struct NgramLm {
    order: usize,
    vocab: HashMap<String, u32>,
    /// `counts[j]` holds (j+1)-grams: raw counts at the top order,
    /// continuation counts below it.
    counts: Vec<Map<u32>>,
    /// `ctx[j]` (j >= 1): context of j ids -> (total, distinct followers).
    ctx: Vec<Map<(u32, u32)>>,
    uni_total: f64,
    uni_types: f64,
    discounts: Vec<f64>,
    pub stats: TrainStats,
}

impl NgramLm {
    pub fn train<'a>(sentences: impl Iterator<Item = &'a str> + Clone, cfg: &NgramConfig) -> Self {
        assert!((2..=4).contains(&cfg.order), "order must be 2..=4");
        let t0 = Instant::now();
        let order = cfg.order;

        // Pass 1: vocabulary.
        let mut freq: HashMap<String, u32> = HashMap::new();
        let mut n_sent = 0usize;
        for s in sentences.clone() {
            n_sent += 1;
            for w in words(s) {
                *freq.entry(w).or_insert(0) += 1;
            }
        }
        let mut kept: Vec<(String, u32)> = freq.into_iter().filter(|(_, c)| *c >= cfg.min_count).collect();
        // Deterministic ids: by frequency, then lexicographic.
        kept.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        let mut vocab: HashMap<String, u32> = HashMap::with_capacity(kept.len() + 3);
        vocab.insert("<unk>".into(), UNK);
        vocab.insert("<s>".into(), BOS);
        vocab.insert("</s>".into(), EOS);
        for (i, (w, _)) in kept.into_iter().enumerate() {
            vocab.insert(w, i as u32 + 3);
        }

        // Pass 2: top-order counts.
        let mut counts: Vec<Map<u32>> = (0..order).map(|_| Map::default()).collect();
        let mut tokens = 0usize;
        let mut ids: Vec<u32> = Vec::new();
        for s in sentences {
            ids.clear();
            ids.extend(std::iter::repeat_n(BOS, order - 1));
            for w in words(s) {
                ids.push(*vocab.get(&w).unwrap_or(&UNK));
            }
            ids.push(EOS);
            tokens += ids.len() - (order - 1);
            for i in (order - 1)..ids.len() {
                *counts[order - 1].entry(pack(&ids[i + 1 - order..=i])).or_insert(0) += 1;
            }
        }

        // Continuation counts for lower orders.
        for j in (0..order - 1).rev() {
            let (lower, upper) = counts.split_at_mut(j + 1);
            let m = mask(j + 1);
            for key in upper[0].keys() {
                *lower[j].entry(key & m).or_insert(0) += 1;
            }
        }

        // Discounts.
        let discounts: Vec<f64> = counts
            .iter()
            .map(|m| {
                let (mut n1, mut n2) = (0f64, 0f64);
                for &c in m.values() {
                    if c == 1 {
                        n1 += 1.0;
                    } else if c == 2 {
                        n2 += 1.0;
                    }
                }
                if n1 > 0.0 && n2 > 0.0 { (n1 / (n1 + 2.0 * n2)).clamp(0.1, 0.95) } else { 0.75 }
            })
            .collect();

        // Context totals.
        let mut ctx: Vec<Map<(u32, u32)>> = (0..order).map(|_| Map::default()).collect();
        for j in 1..order {
            for (&key, &c) in counts[j].iter() {
                let e = ctx[j].entry(key >> 32).or_insert((0, 0));
                e.0 += c;
                e.1 += 1;
            }
        }
        let uni_total = counts[0].values().map(|&c| c as f64).sum();
        let uni_types = counts[0].len() as f64;

        let stats = TrainStats {
            sentences: n_sent,
            tokens,
            vocab: vocab.len(),
            ngrams_per_order: counts.iter().map(|m| m.len()).collect(),
            discounts: discounts.clone(),
            seconds: t0.elapsed().as_secs_f64(),
        };
        Self { order, vocab, counts, ctx, uni_total, uni_types, discounts, stats }
    }

    pub fn order(&self) -> usize {
        self.order
    }

    pub fn ids(&self, text: &str) -> Vec<u32> {
        words(text).into_iter().map(|w| *self.vocab.get(&w).unwrap_or(&UNK)).collect()
    }

    /// P(w | history), history = up to `order - 1` previous ids.
    pub fn prob(&self, history: &[u32], w: u32) -> f64 {
        let v = self.vocab.len() as f64;
        let c1 = *self.counts[0].get(&(w as u128)).unwrap_or(&0) as f64;
        let d0 = self.discounts[0];
        let mut p = if self.uni_total > 0.0 {
            (c1 - d0).max(0.0) / self.uni_total + d0 * self.uni_types / self.uni_total / v
        } else {
            1.0 / v
        };
        for k in 2..=self.order {
            if history.len() < k - 1 {
                break;
            }
            let h = &history[history.len() - (k - 1)..];
            let hk = pack(h);
            let Some(&(total, types)) = self.ctx[k - 1].get(&hk) else { continue };
            let c = *self.counts[k - 1].get(&((hk << 32) | w as u128)).unwrap_or(&0) as f64;
            let d = self.discounts[k - 1];
            let total = total as f64;
            p = (c - d).max(0.0) / total + d * types as f64 / total * p;
        }
        p
    }

    fn history_for(&self, context: &str) -> Vec<u32> {
        let mut h: Vec<u32> = std::iter::repeat_n(BOS, self.order - 1).collect();
        h.extend(self.ids(context));
        h
    }

    /// Perplexity over sentences (each scored from `<s>` through `</s>`).
    pub fn perplexity<'a>(&self, sentences: impl Iterator<Item = &'a str>) -> f64 {
        let (mut lp, mut n) = (0.0f64, 0usize);
        for s in sentences {
            let mut h: Vec<u32> = std::iter::repeat_n(BOS, self.order - 1).collect();
            for w in self.ids(s).into_iter().chain(std::iter::once(EOS)) {
                lp += self.prob(&h[h.len() - (self.order - 1)..], w).ln();
                n += 1;
                h.push(w);
            }
        }
        if n == 0 { f64::NAN } else { (-lp / n as f64).exp() }
    }
}

impl LanguageModel for NgramLm {
    fn name(&self) -> String {
        format!("kn{}-gram (vocab {}, {} tokens)", self.order, self.stats.vocab, self.stats.tokens)
    }

    fn logprob(&self, context: &str, continuation: &str) -> (f64, usize) {
        let mut h = self.history_for(context);
        let cont = self.ids(continuation);
        let mut lp = 0.0;
        for &w in &cont {
            lp += self.prob(&h[h.len() - (self.order - 1)..], w).ln();
            h.push(w);
        }
        (lp, cont.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ChoiceScoring;

    fn toy() -> Vec<String> {
        let mut v = Vec::new();
        for _ in 0..30 {
            v.push("the cat sat on the mat".to_string());
            v.push("the dog sat on the rug".to_string());
            v.push("plants need water and sunlight to grow".to_string());
            v.push("the sun is a star".to_string());
        }
        v
    }

    #[test]
    fn probabilities_sum_to_one() {
        let corpus = toy();
        for order in 2..=4 {
            let lm = NgramLm::train(corpus.iter().map(String::as_str), &NgramConfig { order, min_count: 1 });
            let vocab_ids: Vec<u32> = (0..lm.vocab.len() as u32).collect();
            for hist in [vec![BOS; order - 1], lm.ids("the cat").iter().rev().take(order - 1).rev().cloned().collect()] {
                let s: f64 = vocab_ids.iter().map(|&w| lm.prob(&hist, w)).sum();
                assert!((s - 1.0).abs() < 1e-6, "order {order}: sum {s}");
            }
        }
    }

    #[test]
    fn prefers_seen_continuations_and_beats_uniform() {
        let corpus = toy();
        let lm = NgramLm::train(corpus.iter().map(String::as_str), &NgramConfig { order: 3, min_count: 1 });
        let (lp_good, _) = lm.logprob("plants need", "water");
        let (lp_bad, _) = lm.logprob("plants need", "mat");
        assert!(lp_good > lp_bad);
        let ppl = lm.perplexity(["the cat sat on the rug"].into_iter());
        assert!(ppl < lm.vocab.len() as f64, "perplexity {ppl} should beat uniform");
        let (best, probs) = lm.choose("plants need", &["water and sunlight".into(), "the mat".into()], ChoiceScoring::MeanPerToken);
        assert_eq!(best, 0);
        assert!((probs.iter().sum::<f64>() - 1.0).abs() < 1e-9);
    }
}
