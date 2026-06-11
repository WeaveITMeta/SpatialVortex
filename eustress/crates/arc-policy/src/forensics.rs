//! Death forensics — every GAME_OVER becomes a named, reusable rule.
//!
//! Dying is the densest learning signal an interactive game emits, but only
//! if the death is EXPLAINED. An unexplained GAME_OVER teaches "something
//! bad happened"; an explained one teaches "color-5 cells kill on contact"
//! or "this game runs on a fixed energy budget" — knowledge that A* path
//! costs and click selection can consume directly. This is designing for
//! error: instead of merely penalizing the last action (which smears blame
//! across whatever we happened to be doing), each death generates a cause
//! HYPOTHESIS, and only reproduced hypotheses harden into hazard knowledge.
//!
//! The epistemic discipline mirrors the ledger: one death near a red cell
//! is correlation (we may have died of a timer while standing next to
//! something red); two deaths attributed to the same color is reproduction,
//! and only then do that color's cells start costing extra in pathfinding.
//! `ledger_claim` exposes each cause as a claim id so the EpistemicLedger
//! can track reproducibility with the same Observed→Law lifecycle as every
//! other causal belief.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use eustress_vortex_grid2d::Grid2D;
use tracing::debug;

/// A color must be implicated in at least this many deaths before its
/// cells are treated as hazards. One death is a hypothesis; a reproduced
/// death is knowledge (same philosophy as the ledger's correlation gate —
/// we refuse to pay pathfinding costs for coincidences).
const HAZARD_CONFIRMED_AT: u32 = 2;

/// A life length only counts as evidence of a fixed timer when typical
/// lives are at least this long. Very short lives (instant deaths) cluster
/// tightly for OTHER reasons — usually spawning next to a hazard — and
/// would otherwise masquerade as a schedule.
const TIMER_MIN_LIFE: u32 = 10;

/// How close to the typical life length a death must land to be blamed on
/// the clock rather than on contact.
const TIMER_TOLERANCE: u32 = 2;

/// Why did we die? Each variant is a falsifiable hypothesis about the
/// game's kill mechanic, not a certainty — reproduction (via the ledger
/// and `hazard_colors` counts) is what upgrades it to actionable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DeathCause {
    /// The avatar was adjacent to a non-background, non-self color when it
    /// died — contact with that color is the suspected kill mechanic.
    HazardContact { color: u8 },
    /// The death landed on the same step count as previous deaths: the
    /// game has a fixed energy/time budget and we ran it out.
    TimerExpired,
    /// We pressed submit (ACTION7) and the game judged our state wrong.
    /// The submit itself was the error, not anything spatial.
    BadSubmit,
    /// No heuristic fired. Recorded anyway — even unexplained deaths
    /// contribute their step counts to the timer estimate.
    Unknown,
}

/// One death, frozen at the moment it happened, so later analysis can
/// re-litigate the classification as more context arrives.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeathRecord {
    pub cause: DeathCause,
    /// Steps survived in the life that ended here.
    pub step: u32,
    /// The action string in flight when the game ended ("3", "6:12,40", "7").
    pub last_action: String,
}

/// The accumulating case file. Serializes into the knowledge snapshot so
/// hazard knowledge survives across episodes and sessions — dying to the
/// same color twice in one life or across ten lives is the same evidence.
#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct DeathForensics {
    pub deaths: Vec<DeathRecord>,
    /// color → number of deaths attributed to contact with that color.
    pub hazard_colors: HashMap<u8, u32>,
}

impl DeathForensics {
    /// Classify a GAME_OVER and record the verdict.
    ///
    /// Heuristics fire in precision order — most specific evidence wins:
    ///
    /// 1. **BadSubmit** — `last_action` starts with "7". Submitting is an
    ///    explicit judgment request; if the game ends right after, the
    ///    submit was wrong. Nothing spatial should take blame for it.
    /// 2. **TimerExpired** — `steps_this_life` lands within ±2 of
    ///    `typical_life_len` (when known and ≥10). Fixed-budget energy
    ///    games kill on schedule; deaths that cluster on a step count are
    ///    the schedule, and any nearby colored cell is a bystander.
    /// 3. **HazardContact** — the avatar's 4-neighborhood in `prev_grid`
    ///    (the last frame BEFORE the death screen — `last_grid` is often a
    ///    wipe/overlay that destroys the spatial evidence) contains a
    ///    non-background color that is not the avatar's own. The most
    ///    frequent such neighbor is the suspect; ties break to the lowest
    ///    color id so repeated identical deaths accuse the same color.
    /// 4. **Unknown** — nothing fired. Still recorded: step counts feed
    ///    the timer estimate even when the cause is opaque.
    pub fn classify(
        &mut self,
        prev_grid: &Grid2D,
        _last_grid: &Grid2D,
        last_action: &str,
        avatar_cell: Option<(usize, usize)>,
        steps_this_life: u32,
        typical_life_len: Option<u32>,
    ) -> DeathCause {
        let cause = Self::diagnose(
            prev_grid,
            last_action,
            avatar_cell,
            steps_this_life,
            typical_life_len,
        );
        if let DeathCause::HazardContact { color } = cause {
            *self.hazard_colors.entry(color).or_insert(0) += 1;
        }
        debug!(
            ?cause,
            step = steps_this_life,
            action = last_action,
            "death classified"
        );
        self.deaths.push(DeathRecord {
            cause: cause.clone(),
            step: steps_this_life,
            last_action: last_action.to_string(),
        });
        cause
    }

    /// The pure classification, separated so recording can never diverge
    /// from diagnosis.
    fn diagnose(
        prev_grid: &Grid2D,
        last_action: &str,
        avatar_cell: Option<(usize, usize)>,
        steps_this_life: u32,
        typical_life_len: Option<u32>,
    ) -> DeathCause {
        // 1. Submit-then-die: the submit was the mistake.
        if last_action.starts_with('7') {
            return DeathCause::BadSubmit;
        }

        // 2. Death on schedule: a fixed budget, not a collision.
        if let Some(typical) = typical_life_len {
            if typical >= TIMER_MIN_LIFE
                && steps_this_life.abs_diff(typical) <= TIMER_TOLERANCE
            {
                return DeathCause::TimerExpired;
            }
        }

        // 3. Contact: a foreign color touching the avatar when it died.
        if let Some((r, c)) = avatar_cell {
            let bg = prev_grid.background_color();
            let own = prev_grid.get(r, c).unwrap_or(bg);

            let mut neighbors: Vec<(usize, usize)> = Vec::with_capacity(4);
            if r > 0 {
                neighbors.push((r - 1, c));
            }
            if c > 0 {
                neighbors.push((r, c - 1));
            }
            neighbors.push((r + 1, c));
            neighbors.push((r, c + 1));

            let mut counts: HashMap<u8, u32> = HashMap::new();
            for (nr, nc) in neighbors {
                if let Some(color) = prev_grid.get(nr, nc) {
                    if color != bg && color != own {
                        *counts.entry(color).or_insert(0) += 1;
                    }
                }
            }
            if !counts.is_empty() {
                let mut ranked: Vec<(u8, u32)> = counts.into_iter().collect();
                // Most frequent first; ties break to the lowest color id so
                // the accusation is deterministic across identical deaths.
                ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
                return DeathCause::HazardContact { color: ranked[0].0 };
            }
        }

        // 4. We don't know — and we say so, rather than inventing a story.
        DeathCause::Unknown
    }

    /// The claim id for a death cause, in the epistemic ledger's
    /// vocabulary. The ledger then tracks reproducibility: "death →
    /// hazard_color_5" observed once is correlation; predicted in advance
    /// ("stepping there will kill us") and confirmed, it becomes law.
    pub fn ledger_claim(&self, cause: &DeathCause) -> String {
        match cause {
            DeathCause::HazardContact { color } => format!("death->hazard_color_{}", color),
            DeathCause::TimerExpired => "death->timer_expired".to_string(),
            DeathCause::BadSubmit => "death->bad_submit".to_string(),
            DeathCause::Unknown => "death->unknown".to_string(),
        }
    }

    /// Cells belonging to confirmed-hazard colors (≥2 attributed deaths),
    /// for A* to price and click selection to avoid. Input is plain
    /// (color, cells) pairs rather than tracker types so this module stays
    /// decoupled from ObjectTracker internals — callers project whatever
    /// object source they have.
    pub fn hazard_cells(
        &self,
        tracker_objects: &[(u8, Vec<(usize, usize)>)],
    ) -> HashSet<(i32, i32)> {
        let mut out = HashSet::new();
        for (color, cells) in tracker_objects {
            if self.hazard_colors.get(color).copied().unwrap_or(0) >= HAZARD_CONFIRMED_AT {
                for &(r, c) in cells {
                    out.insert((r as i32, c as i32));
                }
            }
        }
        out
    }

    /// Median life length across recorded deaths — the input to the
    /// TimerExpired heuristic on the NEXT death. Median, not mean, because
    /// one early hazard death must not drag the schedule estimate around.
    /// None under 3 deaths: two points cannot distinguish a schedule from
    /// a coincidence.
    pub fn typical_life_len(&self) -> Option<u32> {
        if self.deaths.len() < 3 {
            return None;
        }
        let mut steps: Vec<u32> = self.deaths.iter().map(|d| d.step).collect();
        steps.sort_unstable();
        let n = steps.len();
        Some(if n % 2 == 1 {
            steps[n / 2]
        } else {
            (steps[n / 2 - 1] + steps[n / 2]) / 2
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An all-background field — nothing to blame.
    fn open_field() -> Grid2D {
        Grid2D::new(vec![vec![0; 6]; 6])
    }

    /// Avatar (color 3) at (2,2) with arbitrary extra cells painted in.
    fn field_with(cells: &[(usize, usize, u8)]) -> Grid2D {
        let mut g = open_field();
        g.set(2, 2, 3);
        for &(r, c, color) in cells {
            g.set(r, c, color);
        }
        g
    }

    #[test]
    fn submit_deaths_blame_the_submit() {
        let mut f = DeathForensics::default();
        // Even standing on top of a hazard, action 7 takes the blame.
        let g = field_with(&[(2, 3, 5)]);
        let cause = f.classify(&g, &open_field(), "7", Some((2, 2)), 4, None);
        assert_eq!(cause, DeathCause::BadSubmit);
        assert_eq!(f.deaths.len(), 1);
        assert_eq!(f.deaths[0].last_action, "7");
        // No hazard accusation was recorded for the adjacent color.
        assert!(f.hazard_colors.is_empty());
    }

    #[test]
    fn deaths_on_schedule_are_timer_deaths() {
        let mut f = DeathForensics::default();
        // Typical life 20, died at 19 — within tolerance, even though a
        // hazard-looking neighbor was present. The schedule wins.
        let g = field_with(&[(2, 3, 5)]);
        let cause = f.classify(&g, &open_field(), "3", Some((2, 2)), 19, Some(20));
        assert_eq!(cause, DeathCause::TimerExpired);
        assert!(f.hazard_colors.is_empty());

        // Outside ±2 the timer heuristic does NOT fire.
        let cause = f.classify(&g, &open_field(), "3", Some((2, 2)), 25, Some(20));
        assert_ne!(cause, DeathCause::TimerExpired);
    }

    #[test]
    fn short_typical_lives_never_indicate_a_timer() {
        let mut f = DeathForensics::default();
        // typical=5 < 10: instant-death clustering, not a schedule.
        let cause = f.classify(&open_field(), &open_field(), "1", None, 5, Some(5));
        assert_eq!(cause, DeathCause::Unknown);
    }

    #[test]
    fn adjacent_foreign_color_is_a_hazard_suspect() {
        let mut f = DeathForensics::default();
        let g = field_with(&[(2, 3, 5)]);
        let cause = f.classify(&g, &open_field(), "4", Some((2, 2)), 7, None);
        assert_eq!(cause, DeathCause::HazardContact { color: 5 });
        assert_eq!(f.hazard_colors.get(&5), Some(&1));
    }

    #[test]
    fn most_frequent_neighbor_color_wins() {
        let mut f = DeathForensics::default();
        // Two color-5 neighbors, one color-4 neighbor: 5 is the suspect.
        let g = field_with(&[(1, 2, 5), (3, 2, 5), (2, 1, 4)]);
        let cause = f.classify(&g, &open_field(), "1", Some((2, 2)), 7, None);
        assert_eq!(cause, DeathCause::HazardContact { color: 5 });
    }

    #[test]
    fn neighbor_ties_break_to_lowest_color() {
        let mut f = DeathForensics::default();
        let g = field_with(&[(1, 2, 5), (3, 2, 4)]);
        let cause = f.classify(&g, &open_field(), "1", Some((2, 2)), 7, None);
        assert_eq!(cause, DeathCause::HazardContact { color: 4 });
    }

    #[test]
    fn own_color_neighbors_are_not_hazards() {
        let mut f = DeathForensics::default();
        // A multi-cell avatar: neighbors share its color 3.
        let g = field_with(&[(1, 2, 3), (3, 2, 3)]);
        let cause = f.classify(&g, &open_field(), "1", Some((2, 2)), 7, None);
        assert_eq!(cause, DeathCause::Unknown);
    }

    #[test]
    fn border_avatars_clip_the_neighborhood() {
        let mut f = DeathForensics::default();
        let mut g = open_field();
        g.set(0, 0, 3); // avatar in the corner
        g.set(0, 1, 9); // hazard to its right
        let cause = f.classify(&g, &open_field(), "2", Some((0, 0)), 3, None);
        assert_eq!(cause, DeathCause::HazardContact { color: 9 });
    }

    #[test]
    fn no_avatar_and_no_pattern_is_unknown() {
        let mut f = DeathForensics::default();
        let cause = f.classify(&open_field(), &open_field(), "1", None, 12, None);
        assert_eq!(cause, DeathCause::Unknown);
        assert_eq!(f.deaths.len(), 1);
        assert_eq!(f.deaths[0].step, 12);
    }

    #[test]
    fn ledger_claims_name_each_cause() {
        let f = DeathForensics::default();
        assert_eq!(
            f.ledger_claim(&DeathCause::HazardContact { color: 5 }),
            "death->hazard_color_5"
        );
        assert_eq!(f.ledger_claim(&DeathCause::TimerExpired), "death->timer_expired");
        assert_eq!(f.ledger_claim(&DeathCause::BadSubmit), "death->bad_submit");
        assert_eq!(f.ledger_claim(&DeathCause::Unknown), "death->unknown");
    }

    #[test]
    fn hazard_cells_require_reproduction() {
        let mut f = DeathForensics::default();
        let g = field_with(&[(2, 3, 5)]);
        // First death blamed on color 5: hypothesis only — no hazard cells.
        f.classify(&g, &open_field(), "4", Some((2, 2)), 7, None);
        let objects: Vec<(u8, Vec<(usize, usize)>)> = vec![
            (5, vec![(2, 3), (4, 4)]),
            (4, vec![(0, 5)]),
        ];
        assert!(f.hazard_cells(&objects).is_empty());

        // Second death on the same color: reproduced — cells become hazards.
        f.classify(&g, &open_field(), "4", Some((2, 2)), 9, None);
        let cells = f.hazard_cells(&objects);
        assert!(cells.contains(&(2, 3)));
        assert!(cells.contains(&(4, 4)));
        // Color 4 has zero attributed deaths: its cells stay free.
        assert!(!cells.contains(&(0, 5)));
        assert_eq!(cells.len(), 2);
    }

    #[test]
    fn median_life_length_needs_three_deaths() {
        let mut f = DeathForensics::default();
        let g = open_field();
        f.classify(&g, &g, "1", None, 10, None);
        f.classify(&g, &g, "1", None, 30, None);
        assert_eq!(f.typical_life_len(), None);

        f.classify(&g, &g, "1", None, 20, None);
        assert_eq!(f.typical_life_len(), Some(20));

        // Even count: mean of the two middles. An outlier-resistant
        // estimate — the 10-step early death does not drag it to the mean.
        f.classify(&g, &g, "1", None, 40, None);
        assert_eq!(f.typical_life_len(), Some(25));
    }

    #[test]
    fn forensics_survive_a_serde_roundtrip() {
        let mut f = DeathForensics::default();
        let g = field_with(&[(2, 3, 5)]);
        f.classify(&g, &open_field(), "4", Some((2, 2)), 7, None);
        f.classify(&g, &open_field(), "7", Some((2, 2)), 9, None);

        let json = serde_json::to_string(&f).expect("serialize");
        let back: DeathForensics = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.deaths, f.deaths);
        assert_eq!(back.hazard_colors.get(&5), Some(&1));
    }
}
