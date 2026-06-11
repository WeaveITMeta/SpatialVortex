//! The epistemic ledger — causal claims with a scientific lifecycle.
//!
//! Every action→effect relationship is a CLAIM that advances through:
//!
//!   Observed(n)   — it happened after we acted (correlation only)
//!   Correlated    — it happened repeatedly (still only correlation)
//!   Verified(k)   — we PREDICTED it in advance, intervened, and the
//!                   prediction held — k independent times
//!   Law           — verification strong enough to write as truth;
//!                   frozen unless reality contradicts it
//!
//! The crucial distinction is intervention-with-prediction versus mere
//! co-occurrence: "the bar drained when I clicked" is correlation;
//! "I said clicking (3,7) would toggle it to blue, clicked, and it did —
//! five times" is verified causation. Per-claim IRA is the verification
//! metric, and demotion happens only on prediction failure — conclusions
//! are written as truth and changed only when the environment disagrees.
//! Promotion is bounded and earned (the 2GB-knowledge-file lesson).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Lifecycle state of a causal claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaimStatus {
    Observed,
    Correlated,
    Verified,
    Law,
}

/// Machine-readable intent: what we expect an action to do, declared
/// BEFORE acting. Comparing outcome to intent (not just to prediction)
/// separates "my model was wrong" from "my plan was wrong".
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Intent {
    /// The claim this action tests, e.g. "click(3,7)→toggle_blue" or
    /// "action_2→avatar_down".
    pub claim_id: String,
    /// Expected effect signature (same vocabulary as observed effects).
    pub expected_effect: String,
}

/// One causal claim with its evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalClaim {
    pub status: ClaimStatus,
    /// Times the effect co-occurred with the action (any circumstance).
    pub observations: u32,
    /// Times a DECLARED prediction of this effect held (intervention).
    pub verifications: u32,
    /// Times a declared prediction FAILED.
    pub contradictions: u32,
    /// Per-claim representation accuracy (EMA of prediction success).
    pub ira: f32,
}

impl Default for CausalClaim {
    fn default() -> Self {
        Self {
            status: ClaimStatus::Observed,
            observations: 0,
            verifications: 0,
            contradictions: 0,
            ira: 0.5,
        }
    }
}

const CORRELATED_AT: u32 = 3; // co-occurrences before "correlated"
const VERIFIED_AT: u32 = 2; // held predictions before "verified"
const LAW_AT: u32 = 5; // held predictions before "law"
const LAW_MIN_IRA: f32 = 0.8;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EpistemicLedger {
    pub claims: HashMap<String, CausalClaim>,
    /// Intent declared for the in-flight action, if any.
    #[serde(skip)]
    pending_intent: Option<Intent>,
}

impl EpistemicLedger {
    /// Record a mere co-occurrence: action happened, effect followed.
    /// Advances Observed → Correlated, never further — correlation alone
    /// can never make a Law.
    pub fn observe_cooccurrence(&mut self, claim_id: &str) {
        let c = self.claims.entry(claim_id.to_string()).or_default();
        c.observations += 1;
        if c.status == ClaimStatus::Observed && c.observations >= CORRELATED_AT {
            c.status = ClaimStatus::Correlated;
        }
    }

    /// Declare intent BEFORE acting: this action is an intervention
    /// testing `claim_id`, predicting `expected_effect`.
    pub fn declare_intent(&mut self, claim_id: &str, expected_effect: &str) {
        self.pending_intent = Some(Intent {
            claim_id: claim_id.to_string(),
            expected_effect: expected_effect.to_string(),
        });
    }

    /// Score the outcome of the declared intervention. Returns the intent
    /// and whether it held, so callers can write the semantic record.
    pub fn resolve_intent(&mut self, observed_effect: &str) -> Option<(Intent, bool)> {
        let intent = self.pending_intent.take()?;
        let held = observed_effect == intent.expected_effect;
        let c = self.claims.entry(intent.claim_id.clone()).or_default();
        if held {
            c.verifications += 1;
            c.ira = c.ira * 0.8 + 0.2;
        } else {
            c.contradictions += 1;
            c.ira *= 0.7;
        }
        // Promotion: only interventions advance past Correlated
        c.status = match (c.verifications, c.ira) {
            (v, ira) if v >= LAW_AT && ira >= LAW_MIN_IRA => ClaimStatus::Law,
            (v, _) if v >= VERIFIED_AT => ClaimStatus::Verified,
            _ if c.observations >= CORRELATED_AT => ClaimStatus::Correlated,
            _ => ClaimStatus::Observed,
        };
        // Demotion on contradiction: Laws fall back to Verified, and a
        // claim contradicted more than verified collapses to Observed.
        if !held {
            if c.status == ClaimStatus::Law {
                c.status = ClaimStatus::Verified;
            }
            if c.contradictions > c.verifications + 1 {
                c.status = ClaimStatus::Observed;
                c.verifications = 0;
            }
        }
        Some((intent, held))
    }

    /// Laws only — what we are willing to plan with as truth.
    pub fn laws(&self) -> impl Iterator<Item = (&String, &CausalClaim)> {
        self.claims.iter().filter(|(_, c)| c.status == ClaimStatus::Law)
    }

    /// Claims worth testing next: correlated but unverified — the
    /// scientific frontier. Sorted by observation count (most-seen first).
    pub fn frontier(&self, limit: usize) -> Vec<String> {
        let mut f: Vec<(&String, u32)> = self
            .claims
            .iter()
            .filter(|(_, c)| c.status == ClaimStatus::Correlated)
            .map(|(id, c)| (id, c.observations))
            .collect();
        f.sort_by(|a, b| b.1.cmp(&a.1));
        f.into_iter().take(limit).map(|(id, _)| id.clone()).collect()
    }

    pub fn reset_pending(&mut self) {
        self.pending_intent = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correlation_alone_never_makes_law() {
        let mut l = EpistemicLedger::default();
        for _ in 0..100 {
            l.observe_cooccurrence("click(0,0)→bar_drain");
        }
        assert_eq!(l.claims["click(0,0)→bar_drain"].status, ClaimStatus::Correlated);
    }

    #[test]
    fn predicted_interventions_promote_to_law() {
        let mut l = EpistemicLedger::default();
        for _ in 0..6 {
            l.declare_intent("click(3,7)→toggle_blue", "toggle_blue");
            let (_, held) = l.resolve_intent("toggle_blue").unwrap();
            assert!(held);
        }
        assert_eq!(l.claims["click(3,7)→toggle_blue"].status, ClaimStatus::Law);
        assert_eq!(l.laws().count(), 1);
    }

    #[test]
    fn contradiction_demotes_law() {
        let mut l = EpistemicLedger::default();
        for _ in 0..6 {
            l.declare_intent("c", "x");
            l.resolve_intent("x");
        }
        assert_eq!(l.claims["c"].status, ClaimStatus::Law);
        l.declare_intent("c", "x");
        let (_, held) = l.resolve_intent("y").unwrap();
        assert!(!held);
        assert_eq!(l.claims["c"].status, ClaimStatus::Verified);
    }

    #[test]
    fn frontier_surfaces_correlated_unverified() {
        let mut l = EpistemicLedger::default();
        for _ in 0..5 {
            l.observe_cooccurrence("a");
        }
        for _ in 0..3 {
            l.observe_cooccurrence("b");
        }
        let f = l.frontier(2);
        assert_eq!(f, vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn intent_resolution_reports_mismatch() {
        let mut l = EpistemicLedger::default();
        l.declare_intent("claim", "expected");
        let (intent, held) = l.resolve_intent("something_else").unwrap();
        assert_eq!(intent.expected_effect, "expected");
        assert!(!held);
    }
}
