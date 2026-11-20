//! # 010 | Vortex World Model
//!
//! Vortex's bridge to the domain-agnostic Eustress learning loop in
//! `eustress/crates/vortex-core` (`WorldState`, `CausalGraph`,
//! `HypothesisTree`, `solve`). Any domain that implements `WorldState` can be
//! reasoned about here; v1 wires the `Grid2D` domain for few-shot ARC.
//!
//! ## Few-shot program induction (ARC)
//! 1. For every demonstration pair, search for a DSL program that maps the
//!    input to the output (`vortex_core::solve`).
//! 2. Keep only programs that reproduce **every** demonstration exactly.
//!    This is the verification step: a program that fits one pair by
//!    accident is rejected.
//! 3. Apply the surviving program to the test input. Its output is never
//!    seen; the harness grades it afterwards.
//!
//! If nothing verifies, the closest candidate is returned with a confidence
//! that says it is a guess. The search uses `rand` inside `vortex-core`, so
//! repeated runs can differ slightly; the harness reports the seed-free
//! variance it observes rather than hiding it.

use std::time::Duration;

use eustress_vortex_core::{CausalGraph, DSLOp, WorldState, hypothesis_tree::SimulationBudget, solve};
use eustress_vortex_grid2d::Grid2D;
use vortex_kernel::{Grid, GridTask};

#[derive(Clone, Debug)]
pub struct ArcConfig {
    pub max_branch_depth: usize,
    pub max_active_branches: usize,
    pub max_total_steps: usize,
    pub time_per_pair: Duration,
}

impl Default for ArcConfig {
    fn default() -> Self {
        Self { max_branch_depth: 3, max_active_branches: 48, max_total_steps: 3_000, time_per_pair: Duration::from_millis(400) }
    }
}

#[derive(Clone, Debug)]
pub struct ArcAnswer {
    pub grid: Grid,
    /// True when the program reproduced every demonstration exactly.
    pub verified: bool,
    pub confidence: f64,
    pub program: Vec<String>,
}

fn to_grid(g: &Grid) -> Grid2D {
    Grid2D::new(g.clone())
}

fn run(program: &[DSLOp], g: &Grid2D) -> Grid2D {
    program.iter().fold(g.clone(), |cur, op| cur.apply(op))
}

pub fn solve_arc(task: &GridTask, cfg: &ArcConfig) -> ArcAnswer {
    let budget = SimulationBudget {
        max_total_steps: cfg.max_total_steps,
        max_branch_depth: cfg.max_branch_depth,
        max_active_branches: cfg.max_active_branches,
        prune_threshold: 0.05,
        time_limit: Some(cfg.time_per_pair),
    };
    let pairs: Vec<(Grid2D, Grid2D)> = task.train.iter().map(|(i, o)| (to_grid(i), to_grid(o))).collect();
    let mut causal = CausalGraph::new();

    // Identity first: some tasks need no change, and it is free to check.
    let mut candidates: Vec<Vec<DSLOp>> = vec![vec![]];
    for (inp, out) in &pairs {
        let r = solve(inp, out, &mut causal, &budget);
        if !r.program.is_empty() {
            candidates.push(r.program);
        }
    }

    let mut best: Option<(f64, &Vec<DSLOp>)> = None;
    for prog in &candidates {
        let accs: Vec<f64> = pairs.iter().map(|(i, o)| run(prog, i).cell_accuracy(o)).collect();
        let all_exact = pairs.iter().all(|(i, o)| run(prog, i).exact_match(o));
        if all_exact {
            let out = run(prog, &to_grid(&task.test_input));
            return ArcAnswer {
                grid: out.cells,
                verified: true,
                confidence: 0.7,
                program: prog.iter().map(|o| o.to_string()).collect(),
            };
        }
        let mean = accs.iter().sum::<f64>() / accs.len().max(1) as f64;
        if best.is_none_or(|(b, _)| mean > b) {
            best = Some((mean, prog));
        }
    }
    let (mean, prog) = best.expect("identity candidate always present");
    let out = run(prog, &to_grid(&task.test_input));
    ArcAnswer {
        grid: out.cells,
        verified: false,
        // A non-verified program is almost never exactly right.
        confidence: (0.05 * mean).min(0.05),
        program: prog.iter().map(|o| o.to_string()).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn induces_and_verifies_a_simple_transform() {
        // Horizontal flip, three demonstrations.
        let flip = |g: &Grid| g.iter().map(|r| r.iter().rev().cloned().collect()).collect::<Grid>();
        let ins: Vec<Grid> = vec![
            vec![vec![1, 0, 0], vec![2, 0, 0]],
            vec![vec![0, 3, 0], vec![4, 0, 5]],
            vec![vec![6, 7, 0], vec![0, 0, 8]],
        ];
        let train = ins.iter().map(|g| (g.clone(), flip(g))).collect();
        let test_input: Grid = vec![vec![9, 1, 0], vec![0, 2, 3]];
        let ans = solve_arc(&GridTask { train, test_input: test_input.clone() }, &ArcConfig::default());
        assert!(ans.verified, "program {:?}", ans.program);
        assert_eq!(ans.grid, flip(&test_input));
    }
}
