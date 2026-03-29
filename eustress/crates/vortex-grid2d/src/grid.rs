//! Grid2D — the core ARC grid type implementing WorldState.

use eustress_vortex_core::{
    DSLOp, Delta, Property, Score, WorldState,
};
use serde::{Deserialize, Serialize};

/// A 2D grid of cells, each with a color value 0–15.
/// This is the fundamental representation for ARC-AGI-3 tasks.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Grid2D {
    /// Row-major storage: cells[row][col].
    pub cells: Vec<Vec<u8>>,
    pub height: usize,
    pub width: usize,
}

impl Grid2D {
    pub fn new(cells: Vec<Vec<u8>>) -> Self {
        let height = cells.len();
        let width = cells.first().map_or(0, |r| r.len());
        Self { cells, height, width }
    }

    pub fn empty(height: usize, width: usize) -> Self {
        Self {
            cells: vec![vec![0; width]; height],
            height,
            width,
        }
    }

    pub fn get(&self, row: usize, col: usize) -> Option<u8> {
        self.cells.get(row).and_then(|r| r.get(col)).copied()
    }

    pub fn set(&mut self, row: usize, col: usize, value: u8) {
        if row < self.height && col < self.width {
            self.cells[row][col] = value;
        }
    }

    /// Cell-level accuracy against another grid.
    pub fn cell_accuracy(&self, other: &Grid2D) -> f64 {
        if self.height != other.height || self.width != other.width {
            return 0.0;
        }
        let total = (self.height * self.width) as f64;
        if total == 0.0 {
            return 1.0;
        }
        let matching = self.cells.iter().zip(other.cells.iter())
            .flat_map(|(r1, r2)| r1.iter().zip(r2.iter()))
            .filter(|(a, b)| a == b)
            .count() as f64;
        matching / total
    }

    /// Exact grid equality.
    pub fn exact_match(&self, other: &Grid2D) -> bool {
        self.cells == other.cells
    }

    /// Color histogram: count of each color 0–15.
    pub fn color_histogram(&self) -> [usize; 16] {
        let mut hist = [0usize; 16];
        for row in &self.cells {
            for &cell in row {
                hist[cell as usize & 0xF] += 1;
            }
        }
        hist
    }

    /// Count of distinct non-background colors.
    pub fn distinct_colors(&self) -> usize {
        let hist = self.color_histogram();
        hist.iter().filter(|&&c| c > 0).count()
    }

    /// Background color (most frequent).
    pub fn background_color(&self) -> u8 {
        let hist = self.color_histogram();
        hist.iter().enumerate()
            .max_by_key(|&(_, count)| count)
            .map_or(0, |(color, _)| color as u8)
    }

    /// Count of cells that differ between two grids.
    pub fn cells_changed(&self, other: &Grid2D) -> usize {
        if self.height != other.height || self.width != other.width {
            return self.height * self.width;
        }
        self.cells.iter().zip(other.cells.iter())
            .flat_map(|(r1, r2)| r1.iter().zip(r2.iter()))
            .filter(|(a, b)| a != b)
            .count()
    }

    /// Serialize to JSON array (ARC format).
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(&self.cells).unwrap_or(serde_json::Value::Null)
    }

    /// Parse from JSON array.
    pub fn from_json(value: &serde_json::Value) -> Option<Self> {
        let cells: Vec<Vec<u8>> = serde_json::from_value(value.clone()).ok()?;
        Some(Self::new(cells))
    }
}

impl WorldState for Grid2D {
    fn analyze(&self) -> Vec<Property> {
        crate::analysis::GridAnalyzer::analyze(self)
    }

    fn available_actions(&self) -> Vec<DSLOp> {
        crate::dsl::GridDSL::all_ops(self)
    }

    fn apply(&self, action: &DSLOp) -> Self {
        crate::dsl::GridDSL::apply(self, action)
    }

    fn score_against(&self, goal: &Self) -> Score {
        let accuracy = self.cell_accuracy(goal);
        Score {
            exact_match: self.exact_match(goal),
            accuracy,
            details: serde_json::json!({
                "cells_correct": (accuracy * (self.height * self.width) as f64) as usize,
                "total_cells": self.height * self.width,
                "dimensions_match": self.height == goal.height && self.width == goal.width,
            }),
        }
    }

    fn diff(&self, other: &Self) -> Vec<Delta> {
        let changed = self.cells_changed(other);
        let total = self.height * self.width;
        if changed == 0 {
            return vec![];
        }
        vec![Delta {
            kind: "cell_changes".into(),
            description: format!(
                "{}/{} cells changed ({:.0}%)",
                changed, total, changed as f64 / total.max(1) as f64 * 100.0
            ),
            magnitude: changed as f64,
        }]
    }

    fn to_iggy_payload(&self) -> Vec<u8> {
        serde_json::to_vec(&self.cells).unwrap_or_default()
    }

    fn summary(&self) -> String {
        format!(
            "Grid2D({}x{}, {} colors, bg={})",
            self.height, self.width, self.distinct_colors(), self.background_color()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_grid() -> Grid2D {
        Grid2D::new(vec![
            vec![0, 1, 2],
            vec![3, 4, 5],
            vec![6, 7, 8],
        ])
    }

    #[test]
    fn test_basic_properties() {
        let g = test_grid();
        assert_eq!(g.height, 3);
        assert_eq!(g.width, 3);
        assert_eq!(g.get(1, 1), Some(4));
        assert_eq!(g.distinct_colors(), 9);
    }

    #[test]
    fn test_cell_accuracy() {
        let g1 = test_grid();
        let g2 = test_grid();
        assert_eq!(g1.cell_accuracy(&g2), 1.0);

        let mut g3 = test_grid();
        g3.set(0, 0, 9);
        assert!((g1.cell_accuracy(&g3) - 8.0 / 9.0).abs() < 1e-6);
    }

    #[test]
    fn test_score_against() {
        let g1 = test_grid();
        let g2 = test_grid();
        let score = g1.score_against(&g2);
        assert!(score.exact_match);
        assert_eq!(score.accuracy, 1.0);
    }
}
