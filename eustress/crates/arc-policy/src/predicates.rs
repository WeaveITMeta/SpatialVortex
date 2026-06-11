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

    /// The current best interpretation (highest posterior).
    pub fn best(&self) -> Option<&WinPredicate> {
        self.predicates
            .iter()
            .max_by(|a, b| a.posterior.partial_cmp(&b.posterior).unwrap_or(std::cmp::Ordering::Equal))
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
}
