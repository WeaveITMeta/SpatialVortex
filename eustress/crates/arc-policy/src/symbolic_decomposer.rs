//! # SymbolicActionDecomposer — Algebraic Grid Transformation Analysis
//!
//! Decomposes observed grid diffs into symbolic operations that can be
//! compared across games using derivative-signature equivalence.
//!
//! Implements the same concepts as Eustress's Symbolica resolver
//! (DerivSignature, beta-bandit EquivalenceCache) but specialized for
//! ARC grid transformations without the Bevy dependency.
//!
//! ## Decomposition Pipeline
//!
//! 1. Observe before/after grids
//! 2. Detect candidate symbolic operations:
//!    - Spatial translations (shift rows/cols)
//!    - Rotations (90°, 180°, 270°)
//!    - Reflections (horizontal, vertical)
//!    - Color remappings (color_a → color_b)
//!    - Region fills (rectangular region → single color)
//!    - Border operations
//!    - Copy/tile patterns
//! 3. Express each as an algebraic formula string
//! 4. Compare formulas across games via DerivSignature fingerprinting
//! 5. Track confidence via beta-bandit (EquivEntry)
//!
//! ## Cross-Game Transfer
//!
//! When action_3 in Game A produces "shift(rows, +1)" and action_2 in
//! Game B produces "shift(rows, +1)", the EquivalenceCache recognizes
//! them as the same underlying operation. Laws learned about shifts
//! in Game A now inform hypotheses in Game B.

use eustress_vortex_grid2d::Grid2D;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use tracing::{debug, info};

// ─────────────────────────────────────────────────────────────────────────────
// Hashing helpers (matches Eustress resolver.rs)
// ─────────────────────────────────────────────────────────────────────────────

fn hash_str(s: &str) -> u64 {
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

// ─────────────────────────────────────────────────────────────────────────────
// SymbolicOp — a single detected grid transformation
// ─────────────────────────────────────────────────────────────────────────────

/// A symbolic representation of a grid transformation.
/// Each variant is an algebraic operation that can be expressed as a formula.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SymbolicOp {
    /// Shift all content by (dr, dc). Formula: "shift(dr, dc)"
    Shift { dr: i32, dc: i32 },
    /// Rotate content by degrees. Formula: "rotate(degrees)"
    Rotate { degrees: u16 },
    /// Reflect across axis. Formula: "reflect(axis)"
    Reflect { axis: ReflectAxis },
    /// Remap one color to another. Formula: "recolor(from, to)"
    Recolor { from: u8, to: u8 },
    /// Fill a rectangular region with a color. Formula: "fill(r1,c1,r2,c2,color)"
    RegionFill { r1: usize, c1: usize, r2: usize, c2: usize, color: u8 },
    /// Draw or modify border. Formula: "border(color, width)"
    Border { color: u8, width: usize },
    /// Gravity/settle operation. Formula: "gravity(direction)"
    Gravity { direction: GravityDir },
    /// Identity (no change detected). Formula: "identity()"
    Identity,
    /// Raw patch (unrecognized pattern). Formula: "patch(n_cells)"
    RawPatch { n_cells: usize },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ReflectAxis { Horizontal, Vertical, Diagonal }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum GravityDir { Down, Up, Left, Right }

impl SymbolicOp {
    /// Express this operation as an algebraic formula string.
    /// These strings are the inputs to DerivSignature for equivalence checking.
    pub fn formula(&self) -> String {
        match self {
            SymbolicOp::Shift { dr, dc } => format!("shift({}, {})", dr, dc),
            SymbolicOp::Rotate { degrees } => format!("rotate({})", degrees),
            SymbolicOp::Reflect { axis } => format!("reflect({})", match axis {
                ReflectAxis::Horizontal => "h",
                ReflectAxis::Vertical => "v",
                ReflectAxis::Diagonal => "d",
            }),
            SymbolicOp::Recolor { from, to } => format!("recolor({}, {})", from, to),
            SymbolicOp::RegionFill { r1, c1, r2, c2, color } =>
                format!("fill({},{},{},{},{})", r1, c1, r2, c2, color),
            SymbolicOp::Border { color, width } => format!("border({}, {})", color, width),
            SymbolicOp::Gravity { direction } => format!("gravity({})", match direction {
                GravityDir::Down => "down",
                GravityDir::Up => "up",
                GravityDir::Left => "left",
                GravityDir::Right => "right",
            }),
            SymbolicOp::Identity => "identity()".into(),
            SymbolicOp::RawPatch { n_cells } => format!("patch({})", n_cells),
        }
    }

    /// Abstract category for cross-game matching.
    /// Operations with the same category are candidates for equivalence.
    pub fn category(&self) -> &'static str {
        match self {
            SymbolicOp::Shift { .. } => "spatial_displacement",
            SymbolicOp::Rotate { .. } => "spatial_rotation",
            SymbolicOp::Reflect { .. } => "spatial_reflection",
            SymbolicOp::Recolor { .. } => "color_remap",
            SymbolicOp::RegionFill { .. } => "region_fill",
            SymbolicOp::Border { .. } => "border_op",
            SymbolicOp::Gravity { .. } => "gravitational_settling",
            SymbolicOp::Identity => "identity",
            SymbolicOp::RawPatch { .. } => "raw_patch",
        }
    }

    /// Confidence in this decomposition (how well it explains the diff).
    /// Higher = more cells explained by this single operation.
    pub fn is_structural(&self) -> bool {
        !matches!(self, SymbolicOp::RawPatch { .. } | SymbolicOp::Identity)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// DerivSignature — fingerprint for formula equivalence
// ─────────────────────────────────────────────────────────────────────────────

/// Derivative-signature fingerprint for a symbolic operation.
/// Two operations with matching signatures are candidates for equivalence.
///
/// Mirrors eustress-common's DerivSignature but uses grid-specific
/// context variables (rows, cols, colors) instead of physics variables.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DerivSignature {
    /// Sorted (var_name, derivative_hash) pairs.
    entries: Vec<(String, u64)>,
    /// XOR-folded fingerprint for fast comparison.
    pub fingerprint: u64,
}

impl DerivSignature {
    /// Compute signature for a formula with respect to grid context variables.
    ///
    /// For ARC, the context variables are the operation's parameters:
    /// - For shift(dr, dc): variables are "dr", "dc"
    /// - For recolor(from, to): variables are "from", "to"
    /// - For rotate(degrees): variable is "degrees"
    pub fn compute(formula: &str, context_vars: &[&str]) -> Self {
        let mut entries: Vec<(String, u64)> = context_vars
            .iter()
            .map(|&var| {
                // Symbolic derivative: d(formula)/d(var)
                // Without Symbolica, we use deterministic string hashing
                let deriv_str = format!("d({})/d({})", formula, var);
                (var.to_string(), hash_str(&deriv_str))
            })
            .collect();
        entries.sort_by(|a, b| a.0.cmp(&b.0));

        let fingerprint = entries.iter().fold(0u64, |acc, (_, h)| acc ^ h);
        Self { entries, fingerprint }
    }

    /// Compute a category-level signature (ignores specific parameter values).
    /// This allows "shift(1, 0)" and "shift(2, 0)" to be recognized as
    /// the same type of operation.
    pub fn compute_categorical(category: &str, param_names: &[&str]) -> Self {
        let mut entries: Vec<(String, u64)> = param_names
            .iter()
            .map(|&var| {
                let deriv_str = format!("d({})/d({})", category, var);
                (var.to_string(), hash_str(&deriv_str))
            })
            .collect();
        entries.sort_by(|a, b| a.0.cmp(&b.0));

        let fingerprint = entries.iter().fold(0u64, |acc, (_, h)| acc ^ h);
        Self { entries, fingerprint }
    }

    /// Whether two signatures match (same algebraic structure).
    pub fn matches(&self, other: &Self) -> bool {
        self.fingerprint == other.fingerprint && self.entries == other.entries
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// EquivEntry — Beta-bandit confidence for operation equivalence
// ─────────────────────────────────────────────────────────────────────────────

/// Beta-bandit state for tracking whether two operations are truly equivalent.
/// Mirrors eustress-common's EquivEntry.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EquivEntry {
    pub alpha: f32,
    pub beta: f32,
}

impl Default for EquivEntry {
    fn default() -> Self {
        Self { alpha: 1.0, beta: 1.0 } // Uninformed prior
    }
}

impl EquivEntry {
    /// Mean of Beta(alpha, beta) — our confidence estimate.
    pub fn confidence(&self) -> f32 {
        self.alpha / (self.alpha + self.beta)
    }

    /// Update on episode outcome. outcome ∈ [0.0, 1.0].
    pub fn update(&mut self, outcome: f32) {
        if outcome >= 0.5 {
            self.alpha += outcome;
        } else {
            self.beta += 1.0 - outcome;
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// EquivalenceCache — cross-game operation matching
// ─────────────────────────────────────────────────────────────────────────────

/// Cache of (operation_a, operation_b) equivalence beliefs.
/// Symmetric: (a, b) and (b, a) map to the same entry.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EquivalenceCache {
    entries: HashMap<(u64, u64, u64), EquivEntry>,
    pub total_lookups: u64,
    pub total_hits: u64,
}

impl EquivalenceCache {
    fn make_key(sig_a: u64, sig_b: u64, context_hash: u64) -> (u64, u64, u64) {
        let (lo, hi) = if sig_a <= sig_b { (sig_a, sig_b) } else { (sig_b, sig_a) };
        (lo, hi, context_hash)
    }

    pub fn entry(&mut self, sig_a: u64, sig_b: u64, context_hash: u64) -> &mut EquivEntry {
        self.total_lookups += 1;
        let key = Self::make_key(sig_a, sig_b, context_hash);
        if self.entries.contains_key(&key) {
            self.total_hits += 1;
        }
        self.entries.entry(key).or_default()
    }

    pub fn get(&self, sig_a: u64, sig_b: u64, context_hash: u64) -> Option<&EquivEntry> {
        let key = Self::make_key(sig_a, sig_b, context_hash);
        self.entries.get(&key)
    }

    pub fn update(&mut self, sig_a: u64, sig_b: u64, context_hash: u64, outcome: f32) {
        let key = Self::make_key(sig_a, sig_b, context_hash);
        self.entries.entry(key).or_default().update(outcome);
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Decomposition — the actual grid diff analysis
// ─────────────────────────────────────────────────────────────────────────────

/// Result of decomposing a grid diff into symbolic operations.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Decomposition {
    /// Detected symbolic operations, ranked by coverage (cells explained).
    pub ops: Vec<(SymbolicOp, f32)>, // (op, coverage 0.0-1.0)
    /// How much of the diff is explained by the symbolic ops combined.
    pub total_coverage: f32,
    /// The diff as raw patches for anything not symbolically explained.
    pub residual_patches: Vec<(usize, usize, u8, u8)>,
}

/// Decompose the diff between two grids into symbolic operations.
///
/// Tries each detector in order of structural importance.
/// Returns the best decomposition (highest total coverage).
pub fn decompose(before: &Grid2D, after: &Grid2D) -> Decomposition {
    let rows = before.height.min(after.height);
    let cols = before.width.min(after.width);

    // Collect all changed cells
    let mut patches: Vec<(usize, usize, u8, u8)> = Vec::new();
    for r in 0..rows {
        for c in 0..cols {
            let old = before.cells[r][c];
            let new = after.cells[r][c];
            if old != new {
                patches.push((r, c, old, new));
            }
        }
    }

    if patches.is_empty() {
        return Decomposition {
            ops: vec![(SymbolicOp::Identity, 1.0)],
            total_coverage: 1.0,
            residual_patches: vec![],
        };
    }

    let total_cells = (rows * cols) as f32;
    let changed_cells = patches.len() as f32;
    let mut detected_ops: Vec<(SymbolicOp, f32)> = Vec::new();
    let mut explained = vec![false; patches.len()];

    // ── Detector 1: Color remapping ──────────────────────────────────────
    // Check if all changes can be explained by color_a → color_b substitutions.
    let remaps = detect_color_remaps(&patches);
    if !remaps.is_empty() {
        let mut remap_explained = 0usize;
        for (i, &(_, _, old, new)) in patches.iter().enumerate() {
            if remaps.iter().any(|&(f, t)| f == old && t == new) {
                if !explained[i] {
                    explained[i] = true;
                    remap_explained += 1;
                }
            }
        }
        let coverage = remap_explained as f32 / changed_cells;
        for (from, to) in remaps {
            detected_ops.push((SymbolicOp::Recolor { from, to }, coverage / detected_ops.len().max(1) as f32));
        }
        if coverage > 0.9 {
            debug!("Decompose: color remap explains {:.0}% of changes", coverage * 100.0);
        }
    }

    // ── Detector 2: Spatial shift ────────────────────────────────────────
    // Check if after ≈ shift(before, dr, dc)
    if let Some((dr, dc, coverage)) = detect_shift(before, after) {
        detected_ops.push((SymbolicOp::Shift { dr, dc }, coverage));
        if coverage > 0.8 {
            // Mark all patches as explained by shift
            for i in 0..patches.len() {
                explained[i] = true;
            }
            debug!("Decompose: shift({},{}) explains {:.0}%", dr, dc, coverage * 100.0);
        }
    }

    // ── Detector 3: Rotation ─────────────────────────────────────────────
    for &degrees in &[90u16, 180, 270] {
        if let Some(coverage) = detect_rotation(before, after, degrees) {
            if coverage > 0.7 {
                detected_ops.push((SymbolicOp::Rotate { degrees }, coverage));
                if coverage > 0.8 {
                    for i in 0..patches.len() { explained[i] = true; }
                }
                debug!("Decompose: rotate({}) explains {:.0}%", degrees, coverage * 100.0);
                break; // Only report best rotation
            }
        }
    }

    // ── Detector 4: Reflection ───────────────────────────────────────────
    for axis in &[ReflectAxis::Horizontal, ReflectAxis::Vertical] {
        if let Some(coverage) = detect_reflection(before, after, axis) {
            if coverage > 0.7 {
                detected_ops.push((SymbolicOp::Reflect { axis: axis.clone() }, coverage));
                if coverage > 0.8 {
                    for i in 0..patches.len() { explained[i] = true; }
                }
                debug!("Decompose: reflect({:?}) explains {:.0}%", axis, coverage * 100.0);
                break;
            }
        }
    }

    // ── Detector 5: Gravity / settling ───────────────────────────────────
    if let Some((dir, coverage)) = detect_gravity(before, after) {
        detected_ops.push((SymbolicOp::Gravity { direction: dir }, coverage));
        if coverage > 0.8 {
            for i in 0..patches.len() { explained[i] = true; }
        }
    }

    // ── Detector 6: Region fill ──────────────────────────────────────────
    let fills = detect_region_fills(&patches, rows, cols);
    for (fill_op, coverage) in fills {
        detected_ops.push((fill_op, coverage));
    }

    // ── Detector 7: Border ───────────────────────────────────────────────
    if let Some((border_op, coverage)) = detect_border_change(before, after, &patches) {
        detected_ops.push((border_op, coverage));
    }

    // Sort by coverage descending
    detected_ops.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Calculate total coverage (capped at 1.0)
    let total_coverage = detected_ops.iter()
        .map(|(_, c)| c)
        .sum::<f32>()
        .min(1.0);

    // Residual: unexplained patches
    let residual: Vec<(usize, usize, u8, u8)> = patches.iter()
        .enumerate()
        .filter(|(i, _)| !explained[*i])
        .map(|(_, p)| *p)
        .collect();

    // If we couldn't explain much, add RawPatch as fallback
    if total_coverage < 0.3 && !patches.is_empty() {
        detected_ops.push((SymbolicOp::RawPatch { n_cells: patches.len() }, changed_cells / total_cells));
    }

    Decomposition {
        ops: detected_ops,
        total_coverage,
        residual_patches: residual,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Individual detectors
// ─────────────────────────────────────────────────────────────────────────────

/// Detect color remapping: all changes are consistent color_a → color_b substitutions.
fn detect_color_remaps(patches: &[(usize, usize, u8, u8)]) -> Vec<(u8, u8)> {
    let mut remap: HashMap<u8, HashMap<u8, usize>> = HashMap::new();
    for &(_, _, old, new) in patches {
        *remap.entry(old).or_default().entry(new).or_default() += 1;
    }

    let mut result = Vec::new();
    for (from, targets) in &remap {
        // Each source color should map to exactly one target
        if targets.len() == 1 {
            let (&to, _count) = targets.iter().next().unwrap();
            result.push((*from, to));
        }
    }
    result.sort();
    result
}

/// Detect spatial shift: after ≈ shift(before, dr, dc).
fn detect_shift(before: &Grid2D, after: &Grid2D) -> Option<(i32, i32, f32)> {
    let rows = before.height.min(after.height);
    let cols = before.width.min(after.width);
    if rows == 0 || cols == 0 { return None; }

    // Try small shifts (-4..+4 in each direction)
    let mut best = (0i32, 0i32, 0.0f32);
    for dr in -4i32..=4 {
        for dc in -4i32..=4 {
            if dr == 0 && dc == 0 { continue; }
            let mut matching = 0usize;
            let mut total = 0usize;
            for r in 0..rows {
                for c in 0..cols {
                    let sr = r as i32 + dr;
                    let sc = c as i32 + dc;
                    if sr >= 0 && sr < rows as i32 && sc >= 0 && sc < cols as i32 {
                        total += 1;
                        if before.cells[sr as usize][sc as usize] == after.cells[r][c] {
                            matching += 1;
                        }
                    }
                }
            }
            if total > 0 {
                let coverage = matching as f32 / total as f32;
                if coverage > best.2 {
                    best = (dr, dc, coverage);
                }
            }
        }
    }

    if best.2 > 0.7 {
        Some(best)
    } else {
        None
    }
}

/// Detect rotation: after ≈ rotate(before, degrees).
fn detect_rotation(before: &Grid2D, after: &Grid2D, degrees: u16) -> Option<f32> {
    let rows = before.height;
    let cols = before.width;
    if rows != cols { return None; } // Only square grids can be purely rotated
    let n = rows;
    if n == 0 { return None; }

    let mut matching = 0usize;
    let total = n * n;

    for r in 0..n {
        for c in 0..n {
            let (nr, nc) = match degrees {
                90 => (c, n - 1 - r),
                180 => (n - 1 - r, n - 1 - c),
                270 => (n - 1 - c, r),
                _ => return None,
            };
            if nr < after.height && nc < after.width
                && before.cells[r][c] == after.cells[nr][nc]
            {
                matching += 1;
            }
        }
    }

    let coverage = matching as f32 / total as f32;
    if coverage > 0.7 { Some(coverage) } else { None }
}

/// Detect reflection: after ≈ reflect(before, axis).
fn detect_reflection(before: &Grid2D, after: &Grid2D, axis: &ReflectAxis) -> Option<f32> {
    let rows = before.height.min(after.height);
    let cols = before.width.min(after.width);
    if rows == 0 || cols == 0 { return None; }

    let mut matching = 0usize;
    let total = rows * cols;

    for r in 0..rows {
        for c in 0..cols {
            let (nr, nc) = match axis {
                ReflectAxis::Horizontal => (rows - 1 - r, c),
                ReflectAxis::Vertical => (r, cols - 1 - c),
                ReflectAxis::Diagonal => {
                    if r < cols && c < rows { (c, r) } else { continue; }
                }
            };
            if nr < after.height && nc < after.width
                && before.cells[r][c] == after.cells[nr][nc]
            {
                matching += 1;
            }
        }
    }

    let coverage = matching as f32 / total as f32;
    if coverage > 0.7 { Some(coverage) } else { None }
}

/// Detect gravity/settling: non-background cells moved toward one edge.
fn detect_gravity(before: &Grid2D, after: &Grid2D) -> Option<(GravityDir, f32)> {
    let rows = before.height.min(after.height);
    let cols = before.width.min(after.width);
    if rows == 0 || cols == 0 { return None; }

    let bg = before.background_color();

    // Check gravity-down: for each column, non-bg cells should be settled to bottom
    let mut down_match = 0usize;
    let mut down_total = 0usize;
    for c in 0..cols {
        // Collect non-bg values from before in this column
        let values: Vec<u8> = (0..rows)
            .filter_map(|r| {
                let v = before.cells[r][c];
                if v != bg { Some(v) } else { None }
            })
            .collect();

        if values.is_empty() { continue; }
        down_total += values.len();

        // Check if after has these values settled at bottom
        let start_row = rows - values.len();
        for (i, &v) in values.iter().enumerate() {
            if start_row + i < rows && after.cells[start_row + i][c] == v {
                down_match += 1;
            }
        }
    }

    if down_total > 0 {
        let coverage = down_match as f32 / down_total as f32;
        if coverage > 0.7 {
            return Some((GravityDir::Down, coverage));
        }
    }

    // Could add Up/Left/Right checks here but Down is most common in ARC
    None
}

/// Detect rectangular region fills in the diff.
fn detect_region_fills(
    patches: &[(usize, usize, u8, u8)],
    _rows: usize,
    _cols: usize,
) -> Vec<(SymbolicOp, f32)> {
    if patches.len() < 4 { return vec![]; }

    // Group patches by target color
    let mut by_color: HashMap<u8, Vec<(usize, usize)>> = HashMap::new();
    for &(r, c, _old, new) in patches {
        by_color.entry(new).or_default().push((r, c));
    }

    let mut fills = Vec::new();
    for (color, cells) in &by_color {
        if cells.len() < 4 { continue; }

        // Check if cells form a rectangle
        let min_r = cells.iter().map(|&(r, _)| r).min().unwrap();
        let max_r = cells.iter().map(|&(r, _)| r).max().unwrap();
        let min_c = cells.iter().map(|&(_, c)| c).min().unwrap();
        let max_c = cells.iter().map(|&(_, c)| c).max().unwrap();

        let rect_area = (max_r - min_r + 1) * (max_c - min_c + 1);
        let fill_ratio = cells.len() as f32 / rect_area as f32;

        if fill_ratio > 0.8 {
            let coverage = cells.len() as f32 / patches.len() as f32;
            fills.push((
                SymbolicOp::RegionFill {
                    r1: min_r, c1: min_c,
                    r2: max_r, c2: max_c,
                    color: *color,
                },
                coverage,
            ));
        }
    }

    fills
}

/// Detect border changes (cells on grid edge changed to a single color).
fn detect_border_change(
    before: &Grid2D,
    after: &Grid2D,
    patches: &[(usize, usize, u8, u8)],
) -> Option<(SymbolicOp, f32)> {
    let rows = before.height.min(after.height);
    let cols = before.width.min(after.width);
    if rows < 3 || cols < 3 { return None; }

    // Check if changes are on the border
    let border_changes: Vec<&(usize, usize, u8, u8)> = patches.iter()
        .filter(|&&(r, c, _, _)| r == 0 || r == rows - 1 || c == 0 || c == cols - 1)
        .collect();

    if border_changes.len() < 4 || border_changes.len() * 2 < patches.len() {
        return None;
    }

    // Check if all border changes go to same color
    let target_color = border_changes[0].3;
    if border_changes.iter().all(|&&(_, _, _, new)| new == target_color) {
        let coverage = border_changes.len() as f32 / patches.len() as f32;
        Some((SymbolicOp::Border { color: target_color, width: 1 }, coverage))
    } else {
        None
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SymbolicActionDecomposer — persistent cross-game operation matcher
// ─────────────────────────────────────────────────────────────────────────────

/// Persistent decomposer that learns operation equivalences across games.
///
/// Each game action is decomposed into SymbolicOps. When the same SymbolicOp
/// appears in different games, the EquivalenceCache tracks whether they're
/// truly equivalent (same effect type) via beta-bandit learning.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SymbolicActionDecomposer {
    /// Per-action decomposition history: action_id → game_id → decompositions.
    pub action_history: HashMap<u32, Vec<(String, Decomposition)>>,
    /// Cross-game operation equivalence cache.
    pub equiv_cache: EquivalenceCache,
    /// Confidence threshold for declaring two operations equivalent.
    pub confidence_threshold: f32,
    /// Known operation archetypes: category → list of (formula, game_count, confidence).
    pub archetypes: HashMap<String, Vec<ActionArchetype>>,
}

/// A recurring operation pattern observed across multiple games.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActionArchetype {
    pub formula: String,
    pub category: String,
    pub signature: DerivSignature,
    /// Number of games where this archetype was observed.
    pub game_count: u32,
    /// Beta-bandit confidence (how often this led to progress).
    pub confidence: EquivEntry,
    /// Which game action IDs produced this operation.
    pub action_ids: Vec<(String, u32)>, // (game_id, action_id)
}

impl SymbolicActionDecomposer {
    pub fn new() -> Self {
        Self {
            action_history: HashMap::new(),
            equiv_cache: EquivalenceCache::default(),
            confidence_threshold: 0.6,
            archetypes: HashMap::new(),
        }
    }

    /// Decompose an observed action effect and learn from it.
    ///
    /// Returns the decomposition and any cross-game matches found.
    pub fn observe_and_learn(
        &mut self,
        game_id: &str,
        action_id: u32,
        before: &Grid2D,
        after: &Grid2D,
        success: bool,
    ) -> (Decomposition, Vec<CrossGameMatch>) {
        let decomp = decompose(before, after);
        let mut matches = Vec::new();

        // Record this decomposition
        self.action_history
            .entry(action_id)
            .or_default()
            .push((game_id.to_string(), decomp.clone()));

        // For each structural op, check against known archetypes
        for (op, coverage) in &decomp.ops {
            if !op.is_structural() || *coverage < 0.5 { continue; }

            let formula = op.formula();
            let category = op.category().to_string();
            let param_names = param_names_for_category(&category);
            let sig = DerivSignature::compute_categorical(&category, &param_names);

            // Check existing archetypes for matches
            let archetype_matches: Vec<String> = self.archetypes
                .get(&category)
                .map(|archs| {
                    archs.iter()
                        .filter(|a| {
                            a.signature.matches(&sig)
                                && a.action_ids.iter().all(|(gid, _)| gid != game_id)
                        })
                        .map(|a| a.formula.clone())
                        .collect()
                })
                .unwrap_or_default();

            for matched_formula in &archetype_matches {
                let match_sig = DerivSignature::compute_categorical(&category, &param_names);
                let ctx_hash = hash_str(&category);

                // Update equivalence cache with outcome
                let outcome = if success { 0.8 } else { 0.2 };
                self.equiv_cache.update(
                    sig.fingerprint,
                    match_sig.fingerprint,
                    ctx_hash,
                    outcome,
                );

                let confidence = self.equiv_cache
                    .entry(sig.fingerprint, match_sig.fingerprint, ctx_hash)
                    .confidence();

                if confidence >= self.confidence_threshold {
                    matches.push(CrossGameMatch {
                        local_formula: formula.clone(),
                        matched_formula: matched_formula.clone(),
                        category: category.clone(),
                        confidence,
                    });
                }
            }

            // Update or create archetype
            let archetypes = self.archetypes.entry(category.clone()).or_default();
            if let Some(existing) = archetypes.iter_mut().find(|a| a.formula == formula) {
                if !existing.action_ids.iter().any(|(gid, aid)| gid == game_id && *aid == action_id) {
                    existing.action_ids.push((game_id.to_string(), action_id));
                    existing.game_count += 1;
                }
                let outcome = if success { 0.8 } else { 0.2 };
                existing.confidence.update(outcome);
            } else {
                let mut conf = EquivEntry::default();
                if success { conf.update(0.8); } else { conf.update(0.2); }

                archetypes.push(ActionArchetype {
                    formula: formula.clone(),
                    category: category.clone(),
                    signature: sig,
                    game_count: 1,
                    confidence: conf,
                    action_ids: vec![(game_id.to_string(), action_id)],
                });
            }
        }

        if !matches.is_empty() {
            info!(
                "SymbolicDecomposer: action_{} in {} matched {} cross-game ops",
                action_id, game_id, matches.len()
            );
        }

        (decomp, matches)
    }

    /// Given observed properties and available actions, suggest which action
    /// is most likely to produce a desired symbolic effect.
    ///
    /// This is where cross-game learning pays off: if we know "shift(down)"
    /// solves games with "has_unsupported_objects", and action_2 produces
    /// "shift(down)" in this game, suggest action_2.
    pub fn suggest_action_for_effect(
        &self,
        game_id: &str,
        desired_category: &str,
    ) -> Option<(u32, f32)> {
        let archetypes = self.archetypes.get(desired_category)?;

        // Find archetypes that matched this game's actions
        let mut best: Option<(u32, f32)> = None;
        for arch in archetypes {
            for (gid, aid) in &arch.action_ids {
                if gid == game_id {
                    let conf = arch.confidence.confidence();
                    if best.map_or(true, |(_, best_conf)| conf > best_conf) {
                        best = Some((*aid, conf));
                    }
                }
            }
        }

        // If no direct match, look for cross-game transfers
        if best.is_none() {
            for arch in archetypes {
                if arch.game_count >= 2 && arch.confidence.confidence() > self.confidence_threshold {
                    // This archetype worked in multiple games — high transfer potential
                    if let Some((_, aid)) = arch.action_ids.first() {
                        best = Some((*aid, arch.confidence.confidence() * 0.5)); // Discount for transfer
                    }
                }
            }
        }

        best
    }

    /// Get summary of known archetypes for logging.
    pub fn archetype_summary(&self) -> Vec<(String, usize, f32)> {
        self.archetypes.iter()
            .flat_map(|(_, archs)| {
                archs.iter().map(|a| {
                    (a.formula.clone(), a.game_count as usize, a.confidence.confidence())
                })
            })
            .collect()
    }
}

impl Default for SymbolicActionDecomposer {
    fn default() -> Self {
        Self::new()
    }
}

/// A cross-game match between two operations.
#[derive(Clone, Debug)]
pub struct CrossGameMatch {
    pub local_formula: String,
    pub matched_formula: String,
    pub category: String,
    pub confidence: f32,
}

/// Context variables for each operation category.
fn param_names_for_category(category: &str) -> Vec<&'static str> {
    match category {
        "spatial_displacement" => vec!["dr", "dc"],
        "spatial_rotation" => vec!["degrees"],
        "spatial_reflection" => vec!["axis"],
        "color_remap" => vec!["from", "to"],
        "region_fill" => vec!["r", "c", "color"],
        "border_op" => vec!["color", "width"],
        "gravitational_settling" => vec!["direction"],
        _ => vec!["x"],
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decompose_identity() {
        let grid = Grid2D::new(vec![vec![0, 1], vec![2, 3]]);
        let d = decompose(&grid, &grid);
        assert_eq!(d.ops.len(), 1);
        assert!(matches!(d.ops[0].0, SymbolicOp::Identity));
        assert_eq!(d.total_coverage, 1.0);
    }

    #[test]
    fn test_decompose_color_remap() {
        let before = Grid2D::new(vec![
            vec![0, 1, 1, 0],
            vec![0, 1, 1, 0],
            vec![0, 0, 0, 0],
        ]);
        let after = Grid2D::new(vec![
            vec![0, 3, 3, 0],
            vec![0, 3, 3, 0],
            vec![0, 0, 0, 0],
        ]);
        let d = decompose(&before, &after);
        let has_recolor = d.ops.iter().any(|(op, _)| matches!(op, SymbolicOp::Recolor { from: 1, to: 3 }));
        assert!(has_recolor, "Should detect recolor(1, 3): {:?}", d.ops);
    }

    #[test]
    fn test_decompose_region_fill() {
        let before = Grid2D::new(vec![
            vec![0, 0, 0, 0],
            vec![0, 1, 2, 0],
            vec![0, 3, 4, 0],
            vec![0, 0, 0, 0],
        ]);
        let after = Grid2D::new(vec![
            vec![0, 0, 0, 0],
            vec![0, 5, 5, 0],
            vec![0, 5, 5, 0],
            vec![0, 0, 0, 0],
        ]);
        let d = decompose(&before, &after);
        let has_fill = d.ops.iter().any(|(op, _)| matches!(op, SymbolicOp::RegionFill { color: 5, .. }));
        assert!(has_fill, "Should detect region fill with color 5: {:?}", d.ops);
    }

    #[test]
    fn test_decompose_rotation_180() {
        let before = Grid2D::new(vec![
            vec![1, 2, 3],
            vec![4, 5, 6],
            vec![7, 8, 9],
        ]);
        let after = Grid2D::new(vec![
            vec![9, 8, 7],
            vec![6, 5, 4],
            vec![3, 2, 1],
        ]);
        let d = decompose(&before, &after);
        let has_rot = d.ops.iter().any(|(op, _)| matches!(op, SymbolicOp::Rotate { degrees: 180 }));
        assert!(has_rot, "Should detect 180° rotation: {:?}", d.ops);
    }

    #[test]
    fn test_symbolic_op_formula() {
        assert_eq!(SymbolicOp::Shift { dr: 1, dc: -2 }.formula(), "shift(1, -2)");
        assert_eq!(SymbolicOp::Recolor { from: 3, to: 7 }.formula(), "recolor(3, 7)");
        assert_eq!(SymbolicOp::Rotate { degrees: 90 }.formula(), "rotate(90)");
    }

    #[test]
    fn test_deriv_signature_matching() {
        let sig1 = DerivSignature::compute_categorical("spatial_displacement", &["dr", "dc"]);
        let sig2 = DerivSignature::compute_categorical("spatial_displacement", &["dr", "dc"]);
        assert!(sig1.matches(&sig2));

        let sig3 = DerivSignature::compute_categorical("spatial_rotation", &["degrees"]);
        assert!(!sig1.matches(&sig3));
    }

    #[test]
    fn test_equiv_entry_bandit() {
        let mut entry = EquivEntry::default();
        assert!((entry.confidence() - 0.5).abs() < 1e-6);

        entry.update(1.0); // success
        assert!(entry.confidence() > 0.5);

        entry.update(0.0); // failure
        entry.update(0.0); // failure
        // Should still be reasonable
        assert!(entry.confidence() > 0.0);
        assert!(entry.confidence() < 1.0);
    }

    #[test]
    fn test_cross_game_learning() {
        let mut decomposer = SymbolicActionDecomposer::new();

        // Game A: action_2 produces a color remap
        let before_a = Grid2D::new(vec![vec![0, 1, 1], vec![0, 1, 0], vec![0, 0, 0]]);
        let after_a = Grid2D::new(vec![vec![0, 3, 3], vec![0, 3, 0], vec![0, 0, 0]]);
        let (decomp_a, _) = decomposer.observe_and_learn("game_a", 2, &before_a, &after_a, true);
        assert!(!decomp_a.ops.is_empty());

        // Game B: action_5 produces a similar color remap
        let before_b = Grid2D::new(vec![vec![2, 2, 0], vec![0, 2, 0], vec![0, 0, 0]]);
        let after_b = Grid2D::new(vec![vec![7, 7, 0], vec![0, 7, 0], vec![0, 0, 0]]);
        let (decomp_b, _matches) = decomposer.observe_and_learn("game_b", 5, &before_b, &after_b, true);
        assert!(!decomp_b.ops.is_empty());

        // Both should create archetypes in the "color_remap" category
        assert!(decomposer.archetypes.contains_key("color_remap"),
            "Should have color_remap archetypes: {:?}", decomposer.archetypes.keys().collect::<Vec<_>>());
    }
}
