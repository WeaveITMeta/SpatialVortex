//! Backward chaining — "define what winning looks like, map backwards
//! to the current state, follow the path."
//!
//! The two halves of the agent's knowledge meet here. The predicate
//! ensemble (predicates.rs) says WHERE we are going — its weighted
//! progress is the best current guess at "how close is this grid to a
//! winning grid". The epistemic ledger (epistemic.rs) says WHAT WE CAN
//! ACTUALLY DO — but only claims at Law strength, because a plan built
//! on correlations is a wish, while a plan built on verified
//! interventions is a procedure. This module composes the two: chain
//! Laws whose simulated effects move the grid toward the predicate.
//!
//! Two honesty notes, so nobody mistakes this for more than it is:
//!
//! 1. **The simulation is partial.** A Law's effect signature records
//!    only the FIRST non-ambient cell change plus a magnitude
//!    ("d(r,c:old->new)xN" means N cells changed, starting with this
//!    one). Cell-wise identity of the other N-1 changes was never
//!    stored, so simulation applies only the first change. Progress
//!    estimates are therefore a lower-fidelity sketch of the real
//!    post-click grid — good enough to rank operators, not good enough
//!    to promise the score.
//!
//! 2. **The search is greedy-forward, not true regression.** We do not
//!    regress goal conditions through operator preconditions (classical
//!    backward chaining); we greedily apply verified operators and keep
//!    the ones the predicate scores higher. The predicate DEFINES the
//!    direction — that is the "backward" part — but the construction
//!    walks forward. True goal regression is future work, and worth
//!    doing once effect signatures capture full deltas.
//!
//! A Law is only applied where its recorded precondition holds (the
//! target cell currently shows the `old` color). A Law verified under
//! one board state says nothing about boards where its precondition is
//! absent — replaying it there would be exactly the unverified
//! extrapolation the ledger exists to prevent.

use serde::{Deserialize, Serialize};
use tracing::debug;

use eustress_vortex_grid2d::Grid2D;

use crate::epistemic::EpistemicLedger;
use crate::predicates::PredicateEnsemble;

/// Minimum progress gain for a step to count as "improvement" — guards
/// against chaining on f32 rounding noise.
const MIN_GAIN: f32 = 1e-6;

/// One verified step in a plan: which Law to fire, where to click, what
/// we predict will happen (in the ledger's effect vocabulary, so the
/// executor can `declare_intent` with it verbatim), and how much we
/// trust the Law (its per-claim IRA).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlannedStep {
    /// Ledger claim id, e.g. "click(3,7)->d(2,5:4->0)x1".
    pub claim_id: String,
    /// Click coordinates (x, y) as the game API expects them.
    pub click: (u32, u32),
    /// Expected effect signature, e.g. "d(2,5:4->0)x1" — pass this to
    /// `EpistemicLedger::declare_intent` before executing the click so
    /// the plan itself becomes an intervention that re-tests its Laws.
    pub expected_effect: String,
    /// The Law's representation accuracy at planning time.
    pub ira: f32,
}

/// A Law parsed into an executable operator.
struct LawOperator {
    claim_id: String,
    click: (u32, u32),
    /// First-change effect: cells[row][col] goes old -> new.
    row: usize,
    col: usize,
    old: u8,
    new: u8,
    /// Total cells the original effect touched (we can only simulate 1).
    magnitude: u32,
    /// Effect substring of the claim id (the intent vocabulary).
    effect: String,
    ira: f32,
}

/// Parse a click-Law claim id of the form
/// "click(x,y)->d(r,c:old->new)xN" (see world_model.rs, CLICK MECHANIC
/// LEARNING — this module must stay in lock-step with that format).
///
/// Returns None for "->none" effects (a verified no-op is real
/// knowledge, but it cannot move the grid anywhere), for non-click
/// claims (e.g. keyboard-action claims), and for anything malformed.
fn parse_law_claim(id: &str) -> Option<(String, (u32, u32), usize, usize, u8, u8, u32)> {
    let rest = id.strip_prefix("click(")?;
    let close = rest.find(')')?;
    let (coords, after) = rest.split_at(close);
    let (x, y) = coords.split_once(',')?;
    let x: u32 = x.trim().parse().ok()?;
    let y: u32 = y.trim().parse().ok()?;
    let effect = after.strip_prefix(")->")?;
    if effect == "none" {
        return None;
    }
    let body = effect.strip_prefix("d(")?;
    let close = body.find(')')?;
    let (inner, tail) = body.split_at(close);
    let magnitude: u32 = tail.strip_prefix(")x")?.trim().parse().ok()?;
    let (rc, change) = inner.split_once(':')?;
    let (r, c) = rc.split_once(',')?;
    let r: usize = r.trim().parse().ok()?;
    let c: usize = c.trim().parse().ok()?;
    let (old, new) = change.split_once("->")?;
    let old: u8 = old.trim().parse().ok()?;
    let new: u8 = new.trim().parse().ok()?;
    Some((effect.to_string(), (x, y), r, c, old, new, magnitude))
}

/// Build a plan of verified steps whose simulated application increases
/// ensemble progress, greedily and without reusing a Law.
///
/// At each step every unused, precondition-satisfied Law is simulated
/// (first-change only — see module doc) against the working grid; the
/// Law with the largest progress gain is committed and the working grid
/// advances. Construction stops when no remaining Law improves progress
/// by more than rounding noise, or at `max_len`.
///
/// Ties on progress go to the higher-IRA Law (equally useful on paper,
/// trust the better-verified one), then to claim-id order so plans are
/// deterministic for a given ledger.
pub fn plan(
    grid: &Grid2D,
    ensemble: &PredicateEnsemble,
    ledger: &EpistemicLedger,
    max_len: usize,
) -> Vec<PlannedStep> {
    let mut ops: Vec<LawOperator> = ledger
        .laws()
        .filter_map(|(id, claim)| {
            parse_law_claim(id).map(|(effect, click, row, col, old, new, magnitude)| {
                LawOperator {
                    claim_id: id.clone(),
                    click,
                    row,
                    col,
                    old,
                    new,
                    magnitude,
                    effect,
                    ira: claim.ira,
                }
            })
        })
        .collect();
    // Stable iteration order -> deterministic tie-breaks (HashMap order
    // would otherwise leak into the plan).
    ops.sort_by(|a, b| a.claim_id.cmp(&b.claim_id));

    let mut current = grid.clone();
    let mut score = ensemble.progress(&current);
    let mut used = vec![false; ops.len()];
    let mut steps: Vec<PlannedStep> = Vec::new();

    while steps.len() < max_len {
        let mut best: Option<(usize, f32, Grid2D)> = None;
        for (i, op) in ops.iter().enumerate() {
            if used[i] {
                continue; // Rule: never the same claim twice in one plan.
            }
            // Precondition: the Law was verified with the target cell
            // showing `old`. Anywhere else its effect is unverified.
            if current.get(op.row, op.col) != Some(op.old) {
                continue;
            }
            let mut sim = current.clone();
            sim.set(op.row, op.col, op.new);
            let s = ensemble.progress(&sim);
            if s - score <= MIN_GAIN {
                continue;
            }
            let replace = match &best {
                None => true,
                Some((bi, bs, _)) => {
                    s > *bs + MIN_GAIN
                        || ((s - *bs).abs() <= MIN_GAIN && op.ira > ops[*bi].ira)
                }
            };
            if replace {
                best = Some((i, s, sim));
            }
        }
        let Some((i, s, sim)) = best else {
            break; // No verified operator moves us closer to "winning".
        };
        let op = &ops[i];
        debug!(
            "  PLAN step {}: {} (gain {:.4} -> {:.4}, ira {:.2}, effect touches {} cells, 1 simulated)",
            steps.len(),
            op.claim_id,
            score,
            s,
            op.ira,
            op.magnitude,
        );
        steps.push(PlannedStep {
            claim_id: op.claim_id.clone(),
            click: op.click,
            expected_effect: op.effect.clone(),
            ira: op.ira,
        });
        used[i] = true;
        current = sim;
        score = s;
    }
    steps
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::epistemic::ClaimStatus;

    /// Drive a claim to Law via declared interventions that hold —
    /// the only path the ledger allows (correlation never promotes).
    fn promote_law(ledger: &mut EpistemicLedger, claim_id: &str, effect: &str) {
        for _ in 0..6 {
            ledger.declare_intent(claim_id, effect);
            let (_, held) = ledger.resolve_intent(effect).unwrap();
            assert!(held);
        }
        assert_eq!(ledger.claims[claim_id].status, ClaimStatus::Law);
    }

    /// 4x4 board, background 0, two single-cell minority colors.
    /// Clearing 5 also fixes the board's only interior defect, so the
    /// ensemble strictly prefers it over clearing 3.
    fn test_grid() -> Grid2D {
        Grid2D::new(vec![
            vec![0, 3, 0, 0],
            vec![0, 0, 5, 0],
            vec![0, 0, 0, 0],
            vec![0, 0, 0, 0],
        ])
    }

    fn clear5_id() -> &'static str {
        "click(2,2)->d(1,2:5->0)x1"
    }
    fn clear3_id() -> &'static str {
        "click(1,1)->d(0,1:3->0)x1"
    }

    #[test]
    fn parses_click_law_claim() {
        let (effect, click, r, c, old, new, n) =
            parse_law_claim("click(12,34)->d(2,5:7->0)x3").unwrap();
        assert_eq!(effect, "d(2,5:7->0)x3");
        assert_eq!(click, (12, 34));
        assert_eq!((r, c), (2, 5));
        assert_eq!((old, new), (7, 0));
        assert_eq!(n, 3);
    }

    #[test]
    fn parser_rejects_none_and_malformed() {
        assert!(parse_law_claim("click(1,2)->none").is_none());
        assert!(parse_law_claim("action_2->avatar_down").is_none());
        assert!(parse_law_claim("click(1,2)->d(2:3->4)x1").is_none());
        assert!(parse_law_claim("click(1,2)->d(2,3:3->4)").is_none());
        assert!(parse_law_claim("click(1,2)").is_none());
    }

    #[test]
    fn plan_orders_by_progress_gain() {
        let grid = test_grid();
        let ensemble = PredicateEnsemble::init_from_grid(&grid);
        let mut ledger = EpistemicLedger::default();
        promote_law(&mut ledger, clear3_id(), "d(0,1:3->0)x1");
        promote_law(&mut ledger, clear5_id(), "d(1,2:5->0)x1");

        let steps = plan(&grid, &ensemble, &ledger, 10);
        assert_eq!(steps.len(), 2);
        // Clearing 5 helps ClearDefects + ClearColor(5) + uniformity;
        // clearing 3 lacks the defect contribution — greedy must take
        // the 5 first.
        assert_eq!(steps[0].claim_id, clear5_id());
        assert_eq!(steps[1].claim_id, clear3_id());
        assert_eq!(steps[0].click, (2, 2));
        assert_eq!(steps[1].click, (1, 1));
        assert_eq!(steps[0].expected_effect, "d(1,2:5->0)x1");
        assert!(steps[0].ira > 0.8, "Law IRA should carry into the step");
    }

    #[test]
    fn plan_progress_is_monotone_under_simulation() {
        let grid = test_grid();
        let ensemble = PredicateEnsemble::init_from_grid(&grid);
        let mut ledger = EpistemicLedger::default();
        promote_law(&mut ledger, clear3_id(), "d(0,1:3->0)x1");
        promote_law(&mut ledger, clear5_id(), "d(1,2:5->0)x1");

        let steps = plan(&grid, &ensemble, &ledger, 10);
        let mut sim = grid.clone();
        let mut last = ensemble.progress(&sim);
        for step in &steps {
            let (_, _, r, c, _, new, _) = parse_law_claim(&step.claim_id).unwrap();
            sim.set(r, c, new);
            let p = ensemble.progress(&sim);
            assert!(p > last, "each planned step must raise progress");
            last = p;
        }
    }

    #[test]
    fn plan_respects_max_len() {
        let grid = test_grid();
        let ensemble = PredicateEnsemble::init_from_grid(&grid);
        let mut ledger = EpistemicLedger::default();
        promote_law(&mut ledger, clear3_id(), "d(0,1:3->0)x1");
        promote_law(&mut ledger, clear5_id(), "d(1,2:5->0)x1");

        let steps = plan(&grid, &ensemble, &ledger, 1);
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].claim_id, clear5_id(), "truncation keeps the best step");
        assert!(plan(&grid, &ensemble, &ledger, 0).is_empty());
    }

    #[test]
    fn plan_skips_none_effects_and_harmful_laws() {
        let grid = test_grid();
        let ensemble = PredicateEnsemble::init_from_grid(&grid);
        let mut ledger = EpistemicLedger::default();
        // A verified no-op: real knowledge, useless for chaining.
        promote_law(&mut ledger, "click(7,7)->none", "none");
        // A verified law that DAMAGES progress (paints a bg cell).
        promote_law(&mut ledger, "click(0,0)->d(3,3:0->7)x1", "d(3,3:0->7)x1");
        promote_law(&mut ledger, clear5_id(), "d(1,2:5->0)x1");

        let steps = plan(&grid, &ensemble, &ledger, 10);
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].claim_id, clear5_id());
    }

    #[test]
    fn plan_requires_precondition_to_hold() {
        let grid = test_grid();
        let ensemble = PredicateEnsemble::init_from_grid(&grid);
        let mut ledger = EpistemicLedger::default();
        // Setting (0,1) to 0 WOULD raise progress, but this Law was
        // verified with old=9 there — the board shows 3, so replaying
        // it would be unverified extrapolation.
        promote_law(&mut ledger, "click(4,4)->d(0,1:9->0)x1", "d(0,1:9->0)x1");

        assert!(plan(&grid, &ensemble, &ledger, 10).is_empty());
    }

    #[test]
    fn plan_uses_laws_only_never_correlations() {
        let grid = test_grid();
        let ensemble = PredicateEnsemble::init_from_grid(&grid);
        let mut ledger = EpistemicLedger::default();
        // Heavily observed, never verified: stays Correlated, and a
        // plan must not be built on correlation.
        for _ in 0..50 {
            ledger.observe_cooccurrence(clear5_id());
        }
        assert!(plan(&grid, &ensemble, &ledger, 10).is_empty());
        assert!(plan(&grid, &ensemble, &EpistemicLedger::default(), 10).is_empty());
    }

    #[test]
    fn plan_never_reuses_a_claim() {
        let grid = test_grid();
        let ensemble = PredicateEnsemble::init_from_grid(&grid);
        let mut ledger = EpistemicLedger::default();
        promote_law(&mut ledger, clear3_id(), "d(0,1:3->0)x1");
        promote_law(&mut ledger, clear5_id(), "d(1,2:5->0)x1");

        let steps = plan(&grid, &ensemble, &ledger, 100);
        let mut ids: Vec<&str> = steps.iter().map(|s| s.claim_id.as_str()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), steps.len(), "no claim may appear twice");
    }
}
