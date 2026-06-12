//! Win-predicate hypothesis ensemble — sensemaking formalized.
//!
//! The agent maintains competing interpretations of what "winning" means
//! for the current level, each with a posterior weight. Reward shaping
//! follows the ensemble's weighted progress, so the policy is always
//! "mapping backwards from what it currently believes the winning state
//! looks like" — and a level completion is Bayesian evidence that updates
//! every predicate's posterior toward whichever interpretations were
//! actually satisfied at the moment of victory. Posteriors persist per
//! game (and seed game-type clusters), so interpretation itself transfers.

use serde::{Deserialize, Serialize};

use eustress_vortex_grid2d::Grid2D;

/// A candidate interpretation of the level's win condition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PredicateKind {
    /// Board converges to one dominant color (clear everything else).
    UniformBoard,
    /// All cells of a specific color disappear (clear targets of color c).
    ClearColor(u8),
    /// Defect cells (differing from majority neighborhood) all fixed.
    ClearDefects,
    /// The count of distinct non-background colors drops to a minimum.
    ReduceColors,
    /// Activate-then-confirm: progress comes in click-then-confirm cycles
    /// (tn36's winning run pressed one cell 16 times between targets).
    ToggleConfirm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WinPredicate {
    pub kind: PredicateKind,
    /// Posterior weight in [0,1]; the ensemble normalizes across members.
    pub posterior: f32,
    /// How many level completions supported this predicate.
    pub confirmations: u32,
}

/// Measure a predicate's progress in [0,1] on the current grid.
fn measure(kind: &PredicateKind, grid: &Grid2D) -> f32 {
    let total = (grid.width * grid.height).max(1) as f32;
    let bg = grid.background_color();
    match kind {
        PredicateKind::UniformBoard => {
            // Fraction of cells matching the dominant color
            let mut counts = [0u32; 16];
            for row in &grid.cells {
                for &v in row {
                    counts[(v as usize).min(15)] += 1;
                }
            }
            *counts.iter().max().unwrap_or(&0) as f32 / total
        }
        PredicateKind::ClearColor(c) => {
            let remaining = grid
                .cells
                .iter()
                .flatten()
                .filter(|&&v| v == *c)
                .count() as f32;
            1.0 - (remaining / total).min(1.0) * 8.0_f32.min(total / 8.0)
        }
        PredicateKind::ClearDefects => {
            // Defect = non-bg cell whose 4-neighborhood majority differs
            let mut defects = 0u32;
            for r in 1..grid.height.saturating_sub(1) {
                for c in 1..grid.width.saturating_sub(1) {
                    let v = grid.cells[r][c];
                    if v == bg {
                        continue;
                    }
                    let n = [
                        grid.cells[r - 1][c],
                        grid.cells[r + 1][c],
                        grid.cells[r][c - 1],
                        grid.cells[r][c + 1],
                    ];
                    let same = n.iter().filter(|&&x| x == v).count();
                    if same == 0 {
                        defects += 1;
                    }
                }
            }
            1.0 / (1.0 + defects as f32 * 0.25)
        }
        PredicateKind::ReduceColors => {
            let distinct = grid.distinct_colors() as f32;
            1.0 / distinct.max(1.0)
        }
        PredicateKind::ToggleConfirm => {
            // Progress is unobservable from a single frame; neutral signal.
            // Its value comes from posterior updates at completions and the
            // re-click behavior it licenses.
            0.5
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PredicateEnsemble {
    pub predicates: Vec<WinPredicate>,
}

impl PredicateEnsemble {
    /// Instantiate candidates from the level-start grid: one ClearColor
    /// per minority color plus the structural interpretations.
    pub fn init_from_grid(grid: &Grid2D) -> Self {
        let mut predicates = vec![
            WinPredicate { kind: PredicateKind::UniformBoard, posterior: 1.0, confirmations: 0 },
            WinPredicate { kind: PredicateKind::ClearDefects, posterior: 1.0, confirmations: 0 },
            WinPredicate { kind: PredicateKind::ReduceColors, posterior: 1.0, confirmations: 0 },
            WinPredicate { kind: PredicateKind::ToggleConfirm, posterior: 0.5, confirmations: 0 },
        ];
        // Minority colors are clear-target candidates
        let mut counts = [0u32; 16];
        for row in &grid.cells {
            for &v in row {
                counts[(v as usize).min(15)] += 1;
            }
        }
        let bg = grid.background_color();
        let mut colored: Vec<(u8, u32)> = (0u8..16)
            .filter(|&c| c != bg && counts[c as usize] > 0)
            .map(|c| (c, counts[c as usize]))
            .collect();
        colored.sort_by_key(|&(_, n)| n);
        for (c, _) in colored.into_iter().take(3) {
            predicates.push(WinPredicate {
                kind: PredicateKind::ClearColor(c),
                posterior: 0.8,
                confirmations: 0,
            });
        }
        let mut e = Self { predicates };
        e.normalize();
        e
    }

    fn normalize(&mut self) {
        let sum: f32 = self.predicates.iter().map(|p| p.posterior).sum();
        if sum > 0.0 {
            for p in &mut self.predicates {
                p.posterior /= sum;
            }
        }
    }

    /// Ensemble progress: posterior-weighted predicate progress in [0,1].
    pub fn progress(&self, grid: &Grid2D) -> f32 {
        self.predicates
            .iter()
            .map(|p| p.posterior * measure(&p.kind, grid))
            .sum()
    }

    /// Bayesian-ish update on level completion: predicates that scored
    /// high at the winning frame gain posterior; the rest lose it.
    /// "Updating what we think winning is along the way."
    pub fn on_level_complete(&mut self, winning_grid: &Grid2D) {
        for p in &mut self.predicates {
            let satisfied = measure(&p.kind, winning_grid);
            if satisfied > 0.7 {
                p.posterior *= 1.0 + satisfied;
                p.confirmations += 1;
            } else {
                p.posterior *= 0.6;
            }
        }
        self.normalize();
    }

    /// Carry interpretations across a level boundary: re-derive the
    /// candidate set (ClearColor targets, counts) from the new level's
    /// first frame, but keep each surviving predicate's posterior as a
    /// prior instead of resetting it. Carried posteriors decay (x0.9,
    /// floored at 0.05 pre-normalization) so a wrong carry can recover
    /// once completions stop supporting it. Verified predicates
    /// (confirmations > 0) whose kind is absent from the fresh candidates
    /// — e.g. a confirmed ClearColor whose color is no longer a minority
    /// — are re-injected rather than dropped.
    pub fn carry_to_next_level(&mut self, new_grid: &Grid2D) {
        let mut next = Self::init_from_grid(new_grid);
        for p in &mut next.predicates {
            if let Some(prev) = self.predicates.iter().find(|q| q.kind == p.kind) {
                p.posterior = (prev.posterior * 0.9).max(0.05);
                p.confirmations = prev.confirmations;
            }
        }
        for prev in &self.predicates {
            let missing = !next.predicates.iter().any(|q| q.kind == prev.kind);
            if missing && prev.confirmations > 0 {
                next.predicates.push(WinPredicate {
                    kind: prev.kind.clone(),
                    posterior: (prev.posterior * 0.9).max(0.05),
                    confirmations: prev.confirmations,
                });
            }
        }
        next.normalize();
        *self = next;
    }

    /// The current best interpretation (highest posterior).
    pub fn best(&self) -> Option<&WinPredicate> {
        self.predicates
            .iter()
            .max_by(|a, b| a.posterior.partial_cmp(&b.posterior).unwrap_or(std::cmp::Ordering::Equal))
    }

    /// The best interpretation only when it dominates the ensemble
    /// (posterior above 0.7) — confident enough to hand to the next
    /// level as a prior.
    pub fn best_verified(&self) -> Option<&WinPredicate> {
        self.best().filter(|p| p.posterior > 0.7)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid_from(rows: Vec<Vec<u8>>) -> Grid2D {
        Grid2D::new(rows)
    }

    #[test]
    fn uniform_progress_rises_as_board_clears() {
        let messy = grid_from(vec![vec![0, 1, 2, 3], vec![0, 1, 0, 0], vec![0; 4], vec![0; 4]]);
        let clean = grid_from(vec![vec![0, 1, 0, 0], vec![0; 4], vec![0; 4], vec![0; 4]]);
        let e = PredicateEnsemble::init_from_grid(&messy);
        assert!(e.progress(&clean) > e.progress(&messy));
    }

    #[test]
    fn completion_reweights_toward_satisfied_predicates() {
        let start = grid_from(vec![vec![0, 5, 0, 5], vec![0; 4], vec![0; 4], vec![0; 4]]);
        let won = grid_from(vec![vec![0; 4], vec![0; 4], vec![0; 4], vec![0; 4]]);
        let mut e = PredicateEnsemble::init_from_grid(&start);
        let uniform_before = e
            .predicates
            .iter()
            .find(|p| p.kind == PredicateKind::UniformBoard)
            .unwrap()
            .posterior;
        e.on_level_complete(&won);
        let uniform_after = e
            .predicates
            .iter()
            .find(|p| p.kind == PredicateKind::UniformBoard)
            .unwrap()
            .posterior;
        assert!(uniform_after > uniform_before);
        assert!(e.best().unwrap().confirmations >= 1 || e.best().unwrap().posterior > 0.2);
    }

    #[test]
    fn carry_retains_priors_and_reinits_targets() {
        // Dominant, twice-confirmed UniformBoard plus a crushed loser.
        let mut e = PredicateEnsemble {
            predicates: vec![
                WinPredicate {
                    kind: PredicateKind::UniformBoard,
                    posterior: 0.999,
                    confirmations: 2,
                },
                WinPredicate {
                    kind: PredicateKind::ClearDefects,
                    posterior: 0.001,
                    confirmations: 0,
                },
            ],
        };
        let next_grid = grid_from(vec![vec![0, 1, 0, 0], vec![0; 4], vec![0; 4], vec![0; 4]]);
        let fresh = PredicateEnsemble::init_from_grid(&next_grid);
        e.carry_to_next_level(&next_grid);

        // Posterior carried (still best, stronger than a fresh init), not reset.
        let uniform = e
            .predicates
            .iter()
            .find(|p| p.kind == PredicateKind::UniformBoard)
            .unwrap();
        let fresh_uniform = fresh
            .predicates
            .iter()
            .find(|p| p.kind == PredicateKind::UniformBoard)
            .unwrap();
        assert!(uniform.posterior > fresh_uniform.posterior);
        assert_eq!(uniform.confirmations, 2);
        assert!(e.best().unwrap().kind == PredicateKind::UniformBoard);

        // Targets re-derived from the new grid: its minority color shows up.
        assert!(e
            .predicates
            .iter()
            .any(|p| p.kind == PredicateKind::ClearColor(1)));

        // Decay floor keeps the crushed predicate recoverable.
        let defects = e
            .predicates
            .iter()
            .find(|p| p.kind == PredicateKind::ClearDefects)
            .unwrap();
        assert!(defects.posterior > 0.02);

        // Still a normalized distribution.
        let sum: f32 = e.predicates.iter().map(|p| p.posterior).sum();
        assert!((sum - 1.0).abs() < 1e-4);
    }

    #[test]
    fn carry_keeps_verified_kinds_and_drops_stale_unverified_ones() {
        let mut e = PredicateEnsemble {
            predicates: vec![
                WinPredicate {
                    kind: PredicateKind::ClearColor(5),
                    posterior: 0.5,
                    confirmations: 2,
                },
                WinPredicate {
                    kind: PredicateKind::ClearColor(7),
                    posterior: 0.3,
                    confirmations: 0,
                },
                WinPredicate {
                    kind: PredicateKind::UniformBoard,
                    posterior: 0.2,
                    confirmations: 0,
                },
            ],
        };
        // New level has neither color 5 nor 7.
        let next_grid = grid_from(vec![vec![0, 1, 0, 1], vec![0; 4], vec![0; 4], vec![0; 4]]);
        e.carry_to_next_level(&next_grid);

        // Verified mechanic survives even without a fresh candidate slot.
        assert!(e
            .predicates
            .iter()
            .any(|p| p.kind == PredicateKind::ClearColor(5) && p.confirmations == 2));
        // Unverified stale target does not.
        assert!(!e
            .predicates
            .iter()
            .any(|p| p.kind == PredicateKind::ClearColor(7)));
    }

    #[test]
    fn best_verified_requires_dominant_posterior() {
        let confident = PredicateEnsemble {
            predicates: vec![
                WinPredicate {
                    kind: PredicateKind::UniformBoard,
                    posterior: 0.75,
                    confirmations: 3,
                },
                WinPredicate {
                    kind: PredicateKind::ReduceColors,
                    posterior: 0.25,
                    confirmations: 0,
                },
            ],
        };
        assert!(confident.best_verified().unwrap().kind == PredicateKind::UniformBoard);

        // A freshly initialized ensemble is spread out — nothing dominates.
        let grid = grid_from(vec![vec![0, 1, 2, 3], vec![0; 4], vec![0; 4], vec![0; 4]]);
        let fresh = PredicateEnsemble::init_from_grid(&grid);
        assert!(fresh.best_verified().is_none());

        // Empty ensemble has no best at all.
        assert!(PredicateEnsemble::default().best_verified().is_none());
    }
}
