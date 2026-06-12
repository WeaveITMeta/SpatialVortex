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
//! Three real classifiers fire in precision order — overlap (we touched
//! it), drained counter (the clock ran out), fatal click (we chose wrong) —
//! and only when none fire do we admit `Unknown`. Evidence richer than a
//! single frozen frame comes from `observe_step`, the per-step feed that
//! lets the classifier see trajectories: a bar draining toward empty, the
//! click issued one step before the fatal frame.
//!
//! The epistemic discipline mirrors the ledger: one death blamed on a color
//! is correlation (we may have died of a timer while standing on something
//! red); two deaths attributed to the same color is reproduction, and only
//! then do that color's cells start costing extra in pathfinding.
//! `ledger_claim` exposes each cause as a claim id ("hazard:color5") so the
//! EpistemicLedger can track reproducibility with the same Observed→Law
//! lifecycle as every other causal belief.

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

/// A cell-group must be seen strictly shrinking at least this many times
/// before its emptying reads as a drained counter. Two ticks could be an
/// object leaving the screen twice; three mirrors the ledger's
/// correlation-at-3 gate (a trend, not an accident).
const TIMER_MIN_SHRINKS: u32 = 3;

/// Why did we die? Each variant is a falsifiable hypothesis about the
/// game's kill mechanic, not a certainty — reproduction (via the ledger
/// and `hazard_colors` counts) is what upgrades it to actionable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DeathCause {
    /// In the pre-death frame the avatar's own cell, or the cell its last
    /// move targeted, held a foreign non-background color — we overlapped
    /// the thing that killed us. Overlap, not adjacency: the old
    /// 4-neighborhood test accused bystanders we never touched.
    ContactHazard { color: u8 },
    /// A cell-group that only ever shrank during this life (a counter, an
    /// energy bar) was empty in the pre-death frame — or, for games whose
    /// budget is not rendered, the death landed on the same step count as
    /// previous deaths.
    TimerExpiry,
    /// The death came within one step of a click action: the click itself
    /// was in flight, or was the action immediately before it. Click games
    /// kill via bad choices, not geometry.
    FatalClick,
    /// No classifier fired. Recorded anyway — even unexplained deaths
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
    /// The action string in flight when the game ended ("3", "6:12,40").
    pub last_action: String,
}

/// Cell-count trajectory of one color across the current life. A color
/// whose group only ever shrinks is a counter candidate; a single growth
/// disqualifies it for the rest of the life (bars that regenerate are
/// health, not clocks).
#[derive(Debug, Clone)]
struct GroupTrend {
    /// Cell count at the most recent observation.
    last: u32,
    /// Number of strict decreases observed.
    shrinks: u32,
    /// True once the count has ever increased.
    grew: bool,
}

/// The accumulating case file. `deaths` and `hazard_colors` serialize into
/// the knowledge snapshot so hazard knowledge survives across episodes and
/// sessions — dying to the same color twice in one life or across ten
/// lives is the same evidence. The per-life evidence (trends, action
/// history, avatar color) is `serde(skip)`: a snapshot must not resurrect
/// another session's half-finished life.
#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct DeathForensics {
    pub deaths: Vec<DeathRecord>,
    /// color → number of deaths attributed to contact with that color.
    pub hazard_colors: HashMap<u8, u32>,
    /// color → cell-count trajectory over the current life.
    #[serde(skip)]
    trends: HashMap<u8, GroupTrend>,
    /// Last avatar color reported via `observe_step` this life. Lets the
    /// avatar-cell overlap check distinguish "that cell IS the avatar"
    /// from "something foreign is sitting where the avatar should be".
    #[serde(skip)]
    avatar_color: Option<u8>,
    /// The two most recent actions seen via `observe_step`, newest first —
    /// enough history to ask "was a click issued within one step of this
    /// death?" regardless of whether the fatal action itself was observed.
    #[serde(skip)]
    last_seen_action: Option<String>,
    #[serde(skip)]
    prior_action: Option<String>,
}

/// Conventional ARC-AGI-3 directional layout (1=up, 2=down, 3=left,
/// 4=right). Games that remap directions will sometimes point this at the
/// wrong cell; the reproduction gate (`HAZARD_CONFIRMED_AT`) is what keeps
/// such one-off misreads from ever pricing cells in A*.
fn move_delta(action: &str) -> Option<(i32, i32)> {
    match action {
        "1" => Some((-1, 0)),
        "2" => Some((1, 0)),
        "3" => Some((0, -1)),
        "4" => Some((0, 1)),
        _ => None,
    }
}

fn is_click(action: &str) -> bool {
    action.starts_with('6')
}

impl DeathForensics {
    /// Per-step evidence feed. Call once per decision with the frame the
    /// action produced, the action string, and the avatar's color when one
    /// is tracked (`None` means "not seen this step" — the last known
    /// color persists, because losing track of the avatar does not change
    /// what color it is). This is what lets classification see
    /// TRAJECTORIES — a counter draining toward empty, the click issued
    /// just before the fatal frame — instead of a single frozen frame.
    pub fn observe_step(&mut self, grid: &Grid2D, action: &str, avatar_color: Option<u8>) {
        if avatar_color.is_some() {
            self.avatar_color = avatar_color;
        }
        self.prior_action = self.last_seen_action.take();
        self.last_seen_action = Some(action.to_string());

        let hist = grid.color_histogram();
        for (&color, trend) in self.trends.iter_mut() {
            let now = hist[color as usize & 0xF] as u32;
            if now < trend.last {
                trend.shrinks += 1;
            }
            if now > trend.last {
                trend.grew = true;
            }
            trend.last = now;
        }
        // Colors first appearing now start their trajectory here — a
        // counter that spawns mid-life and then drains still counts.
        for (color, &n) in hist.iter().enumerate() {
            if n > 0 {
                self.trends
                    .entry(color as u8)
                    .or_insert(GroupTrend { last: n as u32, shrinks: 0, grew: false });
            }
        }
    }

    /// Forget all per-life evidence. `classify` calls this automatically —
    /// a death ends the life — but level advances and new episodes are
    /// also life boundaries and must reset us too: a counter that drained
    /// across a level seam is two half-stories, not a schedule.
    pub fn reset_life(&mut self) {
        self.trends.clear();
        self.avatar_color = None;
        self.last_seen_action = None;
        self.prior_action = None;
    }

    /// Classify a GAME_OVER, record the verdict, and end the life (per-life
    /// evidence is consumed by the verdict and cleared).
    ///
    /// Classifiers fire in precision order — most specific evidence wins:
    ///
    /// 1. **ContactHazard** — in `prev_grid` (the last frame BEFORE the
    ///    death screen — `last_grid` is often a wipe/overlay that destroys
    ///    the spatial evidence) the avatar's cell, or the destination of
    ///    its last move, held a non-background color that is not the
    ///    avatar's own. We touched the suspect; emit its color.
    /// 2. **TimerExpiry** — a cell-group that only ever shrank this life
    ///    (≥3 observed ticks, fed via `observe_step`) is empty in
    ///    `prev_grid`: an on-screen counter ran out. Fallback for games
    ///    with no rendered budget: `steps_this_life` lands within ±2 of
    ///    `typical_life_len` (when known and ≥10) — deaths that cluster on
    ///    a step count are a schedule, and nearby colors are bystanders.
    /// 3. **FatalClick** — the death came within one step of a click
    ///    (`last_action` is the click, or the click was the action seen
    ///    immediately before it).
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
        let cause = self.diagnose(
            prev_grid,
            last_action,
            avatar_cell,
            steps_this_life,
            typical_life_len,
        );
        if let DeathCause::ContactHazard { color } = cause {
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
        self.reset_life();
        cause
    }

    /// The pure-read diagnosis, separated so recording can never diverge
    /// from it.
    fn diagnose(
        &self,
        prev_grid: &Grid2D,
        last_action: &str,
        avatar_cell: Option<(usize, usize)>,
        steps_this_life: u32,
        typical_life_len: Option<u32>,
    ) -> DeathCause {
        // 1. Contact: we overlapped the killer — on our own cell, or on
        // the cell our last move walked into.
        if let Some((r, c)) = avatar_cell {
            let bg = prev_grid.background_color();
            // Without an externally observed avatar color, the centroid
            // cell IS the avatar by assumption, so only the move
            // destination can incriminate anything.
            let own = self
                .avatar_color
                .unwrap_or_else(|| prev_grid.get(r, c).unwrap_or(bg));
            let foreign = |cell: Option<u8>| cell.filter(|&color| color != bg && color != own);

            if let Some(color) = foreign(prev_grid.get(r, c)) {
                return DeathCause::ContactHazard { color };
            }
            if let Some((dr, dc)) = move_delta(last_action) {
                let (nr, nc) = (r as i32 + dr, c as i32 + dc);
                if nr >= 0 && nc >= 0 {
                    if let Some(color) = foreign(prev_grid.get(nr as usize, nc as usize)) {
                        return DeathCause::ContactHazard { color };
                    }
                }
            }
        }

        // 2. Timer: an on-screen counter drained to nothing. The final
        // tick to zero may only be visible here (the death arrived before
        // the next observe_step), so it counts as one more shrink.
        let hist = prev_grid.color_histogram();
        for (&color, trend) in &self.trends {
            let now = hist[color as usize & 0xF] as u32;
            let shrinks = trend.shrinks + u32::from(now < trend.last);
            if !trend.grew && now == 0 && shrinks >= TIMER_MIN_SHRINKS {
                return DeathCause::TimerExpiry;
            }
        }
        // Fallback: death on schedule in a game with no rendered budget.
        if let Some(typical) = typical_life_len {
            if typical >= TIMER_MIN_LIFE
                && steps_this_life.abs_diff(typical) <= TIMER_TOLERANCE
            {
                return DeathCause::TimerExpiry;
            }
        }

        // 3. Fatal click: the click in flight, or the one issued a single
        // step earlier (some games kill on the frame AFTER the choice).
        if is_click(last_action) {
            return DeathCause::FatalClick;
        }
        // If the fatal action itself was already observed, the click must
        // be one observation further back.
        let before_fatal = if self.last_seen_action.as_deref() == Some(last_action) {
            self.prior_action.as_deref()
        } else {
            self.last_seen_action.as_deref()
        };
        if before_fatal.is_some_and(is_click) {
            return DeathCause::FatalClick;
        }

        // 4. We don't know — and we say so, rather than inventing a story.
        DeathCause::Unknown
    }

    /// The claim id for a death cause, in the epistemic ledger's
    /// vocabulary. The ledger then tracks reproducibility: "hazard:color5"
    /// observed once is correlation; predicted in advance ("stepping there
    /// will kill us") and confirmed, it becomes law. Every cause — even
    /// Unknown — gets a claim so unexplained deaths stay countable.
    pub fn ledger_claim(&self, cause: &DeathCause) -> String {
        match cause {
            DeathCause::ContactHazard { color } => format!("hazard:color{}", color),
            DeathCause::TimerExpiry => "hazard:timer".to_string(),
            DeathCause::FatalClick => "hazard:click".to_string(),
            DeathCause::Unknown => "hazard:unknown".to_string(),
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
    /// TimerExpiry fallback on the NEXT death. Median, not mean, because
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

    /// A field whose color-9 "counter" occupies the first `n` cells of
    /// row 0 — shrinking n models a bar draining.
    fn counter_field(n: usize) -> Grid2D {
        let mut g = open_field();
        for c in 0..n {
            g.set(0, c, 9);
        }
        g
    }

    #[test]
    fn overlap_at_move_destination_is_contact() {
        let mut f = DeathForensics::default();
        // Hazard color 5 sits at (2,3); the fatal action was "4" (right),
        // whose destination from (2,2) is exactly that cell.
        let g = field_with(&[(2, 3, 5)]);
        let cause = f.classify(&g, &open_field(), "4", Some((2, 2)), 7, None);
        assert_eq!(cause, DeathCause::ContactHazard { color: 5 });
        assert_eq!(f.hazard_colors.get(&5), Some(&1));
    }

    #[test]
    fn overlap_at_avatar_cell_needs_known_avatar_color() {
        // Color 7 sits where the avatar centroid is. Without an observed
        // avatar color, that cell is assumed to BE the avatar — no verdict.
        let mut g = open_field();
        g.set(2, 2, 7);

        let mut f = DeathForensics::default();
        let cause = f.classify(&g, &open_field(), "1", Some((2, 2)), 7, None);
        assert_eq!(cause, DeathCause::Unknown);

        // Once observe_step has reported the avatar as color 3, color 7
        // on the avatar's cell is recognizably foreign: it overlapped us.
        let mut f = DeathForensics::default();
        f.observe_step(&g, "1", Some(3));
        let cause = f.classify(&g, &open_field(), "1", Some((2, 2)), 7, None);
        assert_eq!(cause, DeathCause::ContactHazard { color: 7 });
    }

    #[test]
    fn own_and_background_destinations_are_not_contact() {
        let mut f = DeathForensics::default();
        // Destination holds the avatar's own color (multi-cell avatar).
        let g = field_with(&[(2, 3, 3)]);
        let cause = f.classify(&g, &open_field(), "4", Some((2, 2)), 7, None);
        assert_eq!(cause, DeathCause::Unknown);

        // Destination is plain background.
        let g = field_with(&[]);
        let cause = f.classify(&g, &open_field(), "4", Some((2, 2)), 7, None);
        assert_eq!(cause, DeathCause::Unknown);
        assert!(f.hazard_colors.is_empty());
    }

    #[test]
    fn adjacency_alone_is_not_contact() {
        let mut f = DeathForensics::default();
        // Color 5 is adjacent above, but the fatal move went right into
        // open background — the old 4-neighborhood test would have accused
        // the bystander; overlap semantics must not.
        let g = field_with(&[(1, 2, 5)]);
        let cause = f.classify(&g, &open_field(), "4", Some((2, 2)), 7, None);
        assert_eq!(cause, DeathCause::Unknown);
        assert!(f.hazard_colors.is_empty());
    }

    #[test]
    fn moves_off_grid_clip_safely() {
        let mut f = DeathForensics::default();
        let mut g = open_field();
        g.set(0, 0, 3); // avatar in the corner
        g.set(1, 0, 9); // hazard below it
        // Moving up from (0,0) leaves the grid: no destination, no verdict.
        let cause = f.classify(&g, &open_field(), "1", Some((0, 0)), 3, None);
        assert_eq!(cause, DeathCause::Unknown);
        // Moving down lands on the hazard.
        let cause = f.classify(&g, &open_field(), "2", Some((0, 0)), 3, None);
        assert_eq!(cause, DeathCause::ContactHazard { color: 9 });
    }

    #[test]
    fn contact_beats_timer_and_click() {
        let mut f = DeathForensics::default();
        let g = field_with(&[(2, 3, 5)]);
        // Schedule evidence present (died exactly on the typical step) and
        // overlap evidence present: the overlap is more specific and wins.
        let cause = f.classify(&g, &open_field(), "4", Some((2, 2)), 20, Some(20));
        assert_eq!(cause, DeathCause::ContactHazard { color: 5 });
    }

    #[test]
    fn draining_group_reaching_zero_is_timer() {
        let mut f = DeathForensics::default();
        // Color 9 ticks 3 → 2 → 1 (two observed shrinks), then the
        // pre-death frame shows 0 — the final tick counts, total 3.
        f.observe_step(&counter_field(3), "1", None);
        f.observe_step(&counter_field(2), "1", None);
        f.observe_step(&counter_field(1), "1", None);
        let cause = f.classify(&counter_field(0), &open_field(), "1", None, 14, None);
        assert_eq!(cause, DeathCause::TimerExpiry);
    }

    #[test]
    fn regrowing_group_never_reads_as_timer() {
        let mut f = DeathForensics::default();
        // 2 → 3 → 2 → 1: it grew once, so it is a health bar or scenery,
        // not a clock — even though it ends empty.
        f.observe_step(&counter_field(2), "1", None);
        f.observe_step(&counter_field(3), "1", None);
        f.observe_step(&counter_field(2), "1", None);
        f.observe_step(&counter_field(1), "1", None);
        let cause = f.classify(&counter_field(0), &open_field(), "1", None, 14, None);
        assert_eq!(cause, DeathCause::Unknown);
    }

    #[test]
    fn too_few_ticks_never_read_as_timer() {
        let mut f = DeathForensics::default();
        // Only one observed shrink plus the final tick: two strict
        // decreases could be an object leaving the screen, not a counter.
        f.observe_step(&counter_field(2), "1", None);
        f.observe_step(&counter_field(1), "1", None);
        let cause = f.classify(&counter_field(0), &open_field(), "1", None, 14, None);
        assert_eq!(cause, DeathCause::Unknown);
    }

    #[test]
    fn deaths_on_schedule_are_timer_deaths() {
        let mut f = DeathForensics::default();
        // No drained counter on screen, but typical life is 20 and we died
        // at 19 — within tolerance: the schedule takes the blame.
        let cause = f.classify(&open_field(), &open_field(), "3", None, 19, Some(20));
        assert_eq!(cause, DeathCause::TimerExpiry);

        // Outside ±2 the schedule fallback does NOT fire.
        let cause = f.classify(&open_field(), &open_field(), "3", None, 25, Some(20));
        assert_ne!(cause, DeathCause::TimerExpiry);
    }

    #[test]
    fn short_typical_lives_never_indicate_a_timer() {
        let mut f = DeathForensics::default();
        // typical=5 < 10: instant-death clustering, not a schedule.
        let cause = f.classify(&open_field(), &open_field(), "1", None, 5, Some(5));
        assert_eq!(cause, DeathCause::Unknown);
    }

    #[test]
    fn timer_beats_click() {
        let mut f = DeathForensics::default();
        f.observe_step(&counter_field(3), "6:1,1", None);
        f.observe_step(&counter_field(2), "6:1,1", None);
        f.observe_step(&counter_field(1), "6:1,1", None);
        // The counter ran out on the same step a click was in flight: the
        // drained counter is the stronger story.
        let cause = f.classify(&counter_field(0), &open_field(), "6:1,1", None, 14, None);
        assert_eq!(cause, DeathCause::TimerExpiry);
    }

    #[test]
    fn click_death_is_fatal_click() {
        let mut f = DeathForensics::default();
        let cause = f.classify(&open_field(), &open_field(), "6:12,40", None, 7, None);
        assert_eq!(cause, DeathCause::FatalClick);
        // A click accuses no color — geometry took no part in this.
        assert!(f.hazard_colors.is_empty());
    }

    #[test]
    fn death_one_step_after_click_is_fatal_click() {
        // The click armed the kill; the next (non-click) action only
        // revealed it. Both call patterns must work: fatal action already
        // observed, and fatal action seen only by classify.
        let mut f = DeathForensics::default();
        f.observe_step(&open_field(), "6:3,4", None);
        f.observe_step(&open_field(), "1", None);
        let cause = f.classify(&open_field(), &open_field(), "1", None, 7, None);
        assert_eq!(cause, DeathCause::FatalClick);

        let mut f = DeathForensics::default();
        f.observe_step(&open_field(), "6:3,4", None);
        let cause = f.classify(&open_field(), &open_field(), "1", None, 7, None);
        assert_eq!(cause, DeathCause::FatalClick);
    }

    #[test]
    fn death_two_steps_after_click_is_not_fatal_click() {
        let mut f = DeathForensics::default();
        f.observe_step(&open_field(), "6:3,4", None);
        f.observe_step(&open_field(), "1", None);
        f.observe_step(&open_field(), "2", None);
        let cause = f.classify(&open_field(), &open_field(), "2", None, 7, None);
        assert_eq!(cause, DeathCause::Unknown);
    }

    #[test]
    fn no_evidence_is_unknown() {
        let mut f = DeathForensics::default();
        let cause = f.classify(&open_field(), &open_field(), "1", None, 12, None);
        assert_eq!(cause, DeathCause::Unknown);
        assert_eq!(f.deaths.len(), 1);
        assert_eq!(f.deaths[0].step, 12);
    }

    #[test]
    fn reset_life_clears_per_life_evidence() {
        let mut f = DeathForensics::default();
        // Build a full timer trend AND an observed avatar color…
        let mut g = open_field();
        g.set(2, 2, 7);
        f.observe_step(&counter_field(4), "1", Some(3));
        f.observe_step(&counter_field(3), "1", Some(3));
        f.observe_step(&counter_field(2), "1", Some(3));
        f.observe_step(&counter_field(1), "1", Some(3));
        f.reset_life();
        // …then forget it all: no drained counter, and color 7 on the
        // avatar cell is once again assumed to BE the avatar.
        let cause = f.classify(&g, &open_field(), "1", Some((2, 2)), 7, None);
        assert_eq!(cause, DeathCause::Unknown);
    }

    #[test]
    fn classify_ends_the_life() {
        let mut f = DeathForensics::default();
        f.observe_step(&counter_field(4), "1", None);
        f.observe_step(&counter_field(3), "1", None);
        f.observe_step(&counter_field(2), "1", None);
        f.observe_step(&counter_field(1), "1", None);
        let cause = f.classify(&counter_field(0), &open_field(), "1", None, 14, None);
        assert_eq!(cause, DeathCause::TimerExpiry);
        // The trend died with the life: the next death sees no counter.
        let cause = f.classify(&counter_field(0), &open_field(), "1", None, 14, None);
        assert_eq!(cause, DeathCause::Unknown);
    }

    #[test]
    fn ledger_claims_name_each_cause() {
        let f = DeathForensics::default();
        assert_eq!(
            f.ledger_claim(&DeathCause::ContactHazard { color: 4 }),
            "hazard:color4"
        );
        assert_eq!(f.ledger_claim(&DeathCause::TimerExpiry), "hazard:timer");
        assert_eq!(f.ledger_claim(&DeathCause::FatalClick), "hazard:click");
        assert_eq!(f.ledger_claim(&DeathCause::Unknown), "hazard:unknown");
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
        f.classify(&g, &open_field(), "6:1,1", None, 9, None);

        let json = serde_json::to_string(&f).expect("serialize");
        let back: DeathForensics = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.deaths, f.deaths);
        assert_eq!(back.hazard_colors.get(&5), Some(&1));
    }
}
