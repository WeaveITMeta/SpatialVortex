//! # 012 | Vortex RSI (recursive self-improvement, gated)
//!
//! Self-improvement is where AI systems most easily fool themselves: tune on
//! the test set, keep whatever looked better on one lucky run, or write
//! "improvements" to their own code on unchecked evidence. The legacy
//! `src/asi/self_modification.rs` did the last of these (a hardcoded
//! "95/100 passed" followed by real writes to its own source). This crate
//! is the replacement, and it is deliberately narrow:
//!
//! 1. A [`Trial`] is a candidate configuration's per-item outcomes on a
//!    **development** split. The test split is never touched here.
//! 2. Each candidate is compared with the incumbent on the *same* items
//!    with an exact one-sided McNemar test, Bonferroni-corrected for the
//!    number of candidates tried.
//! 3. The resulting evidence goes to the constitution (A5). Parameters
//!    change only on `Permit`; code changes always need a human.
//! 4. Every decision, accepted or not, is appended to the RSI ledger with
//!    the incumbent kept for rollback. Writing the ledger is file I/O, so it
//!    lives in `vortex-lab` (`ledger::record`); this module only decides.

use serde::Serialize;

use crate::constitution::{ChangeKind, Constitution, Decision, ImprovementEvidence, Proposal};

#[derive(Clone, Debug, Serialize)]
pub struct Trial {
    pub name: String,
    pub config: serde_json::Value,
    /// Per-item correctness on the development split, in a fixed item order.
    pub outcomes: Vec<bool>,
}

impl Trial {
    pub fn accuracy(&self) -> f64 {
        self.outcomes.iter().filter(|x| **x).count() as f64 / self.outcomes.len().max(1) as f64
    }
}

/// One-sided exact McNemar p-value (candidate better than incumbent).
pub fn mcnemar(incumbent: &[bool], candidate: &[bool]) -> f64 {
    assert_eq!(incumbent.len(), candidate.len(), "trials must share items");
    let b = incumbent.iter().zip(candidate).filter(|(i, c)| !**i && **c).count();
    let c = incumbent.iter().zip(candidate).filter(|(i, c)| **i && !**c).count();
    let n = b + c;
    if n == 0 {
        return 1.0;
    }
    let ln_half_n = -(n as f64) * std::f64::consts::LN_2;
    let mut ln_c = 0.0f64;
    let mut total = 0.0;
    for k in 0..=n {
        if k > 0 {
            ln_c += ((n - k + 1) as f64).ln() - (k as f64).ln();
        }
        if k >= b {
            total += (ln_c + ln_half_n).exp();
        }
    }
    total.min(1.0)
}

#[derive(Clone, Debug, Serialize)]
pub struct Comparison {
    pub candidate: String,
    pub candidate_accuracy: f64,
    pub evidence: ImprovementEvidence,
    pub decision: Decision,
    pub reasons: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Selection {
    pub incumbent: Trial,
    pub chosen: Trial,
    pub changed: bool,
    pub comparisons: Vec<Comparison>,
    pub constitution: String,
    pub dev_items: usize,
}

/// Compare every candidate with the incumbent and adopt the best one the
/// constitution permits. `safety_regressions` maps candidate name to the
/// number of safety-suite items it newly fails.
pub fn select(
    constitution: &Constitution,
    incumbent: Trial,
    candidates: Vec<Trial>,
    safety_regressions: impl Fn(&Trial) -> usize,
) -> Selection {
    let k = candidates.len().max(1) as f64;
    let mut comparisons = Vec::new();
    let mut best: Option<(f64, Trial)> = None;
    for cand in candidates {
        let raw_p = mcnemar(&incumbent.outcomes, &cand.outcomes);
        let evidence = ImprovementEvidence {
            heldout_gain: cand.accuracy() - incumbent.accuracy(),
            p_value: (raw_p * k).min(1.0),
            safety_regressions: safety_regressions(&cand),
            items: cand.outcomes.len(),
        };
        let ruling = constitution.review(&Proposal::SelfModify {
            kind: ChangeKind::Parameter,
            evidence: Some(&evidence),
            human_signoff: false,
            reversible: true,
        });
        if ruling.permitted() && best.as_ref().is_none_or(|(g, _)| evidence.heldout_gain > *g) {
            best = Some((evidence.heldout_gain, cand.clone()));
        }
        comparisons.push(Comparison {
            candidate: cand.name.clone(),
            candidate_accuracy: cand.accuracy(),
            evidence,
            decision: ruling.decision,
            reasons: ruling.reasons,
        });
    }
    let dev_items = incumbent.outcomes.len();
    let (changed, chosen) = match best {
        Some((_, c)) => (true, c),
        None => (false, incumbent.clone()),
    };
    Selection { incumbent, chosen, changed, comparisons, constitution: constitution.hash().to_string(), dev_items }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn trial(name: &str, outcomes: Vec<bool>) -> Trial {
        Trial { name: name.into(), config: json!({}), outcomes }
    }

    #[test]
    fn adopts_only_significant_safe_gains() {
        let c = Constitution::v1();
        let base: Vec<bool> = (0..200).map(|i| i % 2 == 0).collect(); // 50%
        // Strong candidate: fixes 40 of the incumbent's failures, breaks none.
        let mut strong = base.clone();
        for (i, x) in strong.iter_mut().enumerate() {
            if i % 2 == 1 && i < 80 {
                *x = true;
            }
        }
        // Noise candidate: fixes 3, breaks 2.
        let mut noise = base.clone();
        noise[1] = true;
        noise[3] = true;
        noise[5] = true;
        noise[0] = false;
        noise[2] = false;
        let sel = select(&c, trial("base", base.clone()), vec![trial("noise", noise), trial("strong", strong.clone())], |_| 0);
        assert!(sel.changed);
        assert_eq!(sel.chosen.name, "strong");
        assert_eq!(sel.comparisons[0].decision, Decision::Forbid, "noise must be rejected");

        let unsafe_sel = select(&c, trial("base", base), vec![trial("strong", strong)], |_| 1);
        assert!(!unsafe_sel.changed, "a safety regression blocks adoption");
    }

    #[test]
    fn mcnemar_is_one_when_identical() {
        let v = vec![true, false, true];
        assert_eq!(mcnemar(&v, &v), 1.0);
    }

    #[test]
    fn mcnemar_matches_binomial_tail() {
        // 10 discordant pairs, all won by the candidate: p = 0.5^10.
        let (inc, cand) = (vec![false; 10], vec![true; 10]);
        assert!((mcnemar(&inc, &cand) - 0.5f64.powi(10)).abs() < 1e-12);
        // 5 discordant each way: no evidence of a gain.
        let inc: Vec<bool> = (0..10).map(|i| i < 5).collect();
        let cand: Vec<bool> = (0..10).map(|i| i >= 5).collect();
        assert!(mcnemar(&inc, &cand) > 0.5);
    }
}
