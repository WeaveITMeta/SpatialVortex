//! ARC-AGI Vortex Evaluation Harness
//!
//! End-to-end ARC solver using the vortex machinery:
//!   GridEncoder → TransformInduction → CausalModel → SymbolResolver → GridPredict
//!
//! Usage:
//!   cargo run --bin arc-eval --release -- --data-dir ../benchmarks/data --limit 50
//!   cargo run --bin arc-eval --release -- --data-dir ../benchmarks/data --tasks all

use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Instant;

use vortex::ml::grid_encoder::{
    ArcGrid, GridEncoder, GridEncoderConfig, EncodedGrid, TransformEncoding,
    extract_spatial_relations, EMBED_DIM,
};
use vortex::ml::causal_model::{CausalModel, SymbolicDerivative};
use vortex::ml::symbol_resolver::SymbolResolver;

// Eustress Vortex integration — domain-agnostic solve loop + Grid2D DSL
use eustress_vortex_core::{
    solve as vortex_solve, CausalGraph, DSLOp,
    WorldState as VortexWorldState,
};
use eustress_vortex_core::hypothesis_tree::SimulationBudget;
use eustress_vortex_grid2d::{Grid2D, GridDSL, GridAnalyzer};

// =============================================================================
// CLI
// =============================================================================

#[derive(Parser, Debug)]
#[command(
    name = "arc-eval",
    about = "ARC-AGI evaluation using SpatialVortex vortex machinery",
    version = "0.1.0"
)]
struct Args {
    /// Path to benchmarks/data directory
    #[arg(long, default_value = "../benchmarks/data")]
    data_dir: String,

    /// Limit number of tasks (0 = all)
    #[arg(long, default_value_t = 0)]
    limit: usize,

    /// Output JSON path
    #[arg(long, default_value = "arc_eval_results.json")]
    output: String,

    /// Verbose per-task output
    #[arg(long, default_value_t = false)]
    verbose: bool,

    /// Which dataset: "training" or "evaluation"
    #[arg(long, default_value = "training")]
    split: String,
}

// =============================================================================
// ARC Task structures
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ArcExample {
    input: Vec<Vec<u8>>,
    output: Vec<Vec<u8>>,
}

#[derive(Debug, Clone)]
struct ArcTask {
    id: String,
    train: Vec<ArcExample>,
    test: Vec<ArcExample>,
}

// =============================================================================
// Transform Rule — the induced pattern from training examples
// =============================================================================

#[derive(Debug, Clone)]
struct TransformRule {
    /// Mean cycled delta embedding from training pairs
    mean_delta: Vec<f32>,
    /// Size change pattern: (input_h, input_w) → (output_h, output_w)
    size_changes: Vec<((usize, usize), (usize, usize))>,
    /// Colour mapping: input_colour → output_colour (if consistent)
    colour_map: HashMap<u8, u8>,
    /// Whether the transform preserves grid size
    size_preserving: bool,
    /// Output size if consistent across examples
    consistent_output_size: Option<(usize, usize)>,
    /// Scale factor (if output is scaled version of input)
    scale_factor: Option<(usize, usize)>,
    /// Detected transformation type
    transform_type: TransformType,
    /// Per-cell transformation patterns (for small grids)
    cell_patterns: Vec<CellPattern>,
    /// Spatial relation triples from training examples
    relation_triples: Vec<(String, String, String)>,
    /// Embedding similarity between training deltas (consistency measure)
    delta_consistency: f32,
}

#[derive(Debug, Clone, PartialEq)]
enum TransformType {
    /// Direct colour remapping (e.g., swap red↔blue)
    ColourRemap,
    /// Scale the grid by a factor
    Scale,
    /// Tile/repeat the input
    Tile,
    /// Rotate or flip
    Geometric,
    /// Fill regions based on rules
    Fill,
    /// Object manipulation (move, copy, etc.)
    ObjectManip,
    /// Complex/unknown
    Complex,
}

#[derive(Debug, Clone)]
struct CellPattern {
    /// For each output cell, how it relates to input cells
    row: usize,
    col: usize,
    source: CellSource,
}

#[derive(Debug, Clone)]
enum CellSource {
    /// Direct copy from input[r][c]
    Copy(usize, usize),
    /// Constant colour
    Constant(u8),
    /// Remapped from input[r][c] through colour_map
    Remap(usize, usize),
    /// Unknown
    Unknown,
}

// =============================================================================
// Vortex ARC Solver
// =============================================================================

struct VortexArcSolver {
    encoder: GridEncoder,
    causal_model: CausalModel,
    symbol_resolver: SymbolResolver,
    solved: usize,
    attempted: usize,
    /// Eustress CausalGraph — persists across tasks for cross-task learning
    vortex_graph: CausalGraph,
}

impl VortexArcSolver {
    fn new() -> Self {
        Self {
            encoder: GridEncoder::new(GridEncoderConfig::default()),
            causal_model: CausalModel::with_physics_laws(),
            symbol_resolver: SymbolResolver::new(),
            solved: 0,
            attempted: 0,
            vortex_graph: CausalGraph::with_physics_laws(),
        }
    }

    /// Solve a single ARC task: learn the pattern from train examples, predict test outputs.
    #[allow(unused_mut)]
    fn solve_task(&mut self, task: &ArcTask, verbose: bool) -> Vec<Vec<Vec<u8>>> {
        self.attempted += 1;
        let mut predictions = Vec::new();

        // Step 1: Encode all training transform pairs
        let transforms: Vec<TransformEncoding> = task.train.iter().map(|ex| {
            let input_grid = make_grid(&ex.input);
            let output_grid = make_grid(&ex.output);
            self.encoder.encode_transform(&input_grid, &output_grid)
        }).collect();

        // Step 2: Induce transformation rule
        let rule = self.induce_rule(task, &transforms);

        if verbose {
            println!("  Transform type: {:?}", rule.transform_type);
            println!("  Size preserving: {}", rule.size_preserving);
            if let Some((sh, sw)) = rule.scale_factor {
                println!("  Scale factor: {}x{}", sh, sw);
            }
            println!("  Colour map: {:?}", rule.colour_map);
            println!("  Delta consistency: {:.3}", rule.delta_consistency);
        }

        // Step 3: Add discovered causal rules to the model
        self.register_causal_rules(&rule, &task.id);

        // Step 4: Predict each test output
        for (test_idx, test_ex) in task.test.iter().enumerate() {
            let test_input = make_grid(&test_ex.input);
            let predicted = self.predict_output(&test_input, &rule, task, verbose);

            if verbose {
                let correct = &test_ex.output;
                let is_match = predicted == *correct;
                println!("  Test {}: {} ({}x{} → {}x{})",
                    test_idx,
                    if is_match { "CORRECT" } else { "WRONG" },
                    test_input.height, test_input.width,
                    predicted.len(), predicted.first().map_or(0, |r| r.len()),
                );
            }

            predictions.push(predicted);
        }

        predictions
    }

    /// Induce a transformation rule from training examples.
    fn induce_rule(&self, task: &ArcTask, transforms: &[TransformEncoding]) -> TransformRule {
        let d = EMBED_DIM;

        // ── Mean cycled delta ────────────────────────────────────────────
        let mut mean_delta = vec![0.0f32; d];
        if !transforms.is_empty() {
            for t in transforms {
                for dd in 0..d {
                    mean_delta[dd] += t.cycled_delta[dd];
                }
            }
            let inv = 1.0 / transforms.len() as f32;
            for dd in 0..d {
                mean_delta[dd] *= inv;
            }
        }

        // ── Delta consistency (pairwise cosine) ──────────────────────────
        let delta_consistency = if transforms.len() >= 2 {
            let mut total_sim = 0.0f32;
            let mut count = 0;
            for i in 0..transforms.len() {
                for j in i+1..transforms.len() {
                    total_sim += cosine_sim(&transforms[i].cycled_delta, &transforms[j].cycled_delta);
                    count += 1;
                }
            }
            if count > 0 { total_sim / count as f32 } else { 0.0 }
        } else {
            1.0
        };

        // ── Size analysis ────────────────────────────────────────────────
        let size_changes: Vec<_> = task.train.iter().map(|ex| {
            let ih = ex.input.len();
            let iw = ex.input.first().map_or(0, |r| r.len());
            let oh = ex.output.len();
            let ow = ex.output.first().map_or(0, |r| r.len());
            ((ih, iw), (oh, ow))
        }).collect();

        let size_preserving = size_changes.iter().all(|((ih, iw), (oh, ow))| ih == oh && iw == ow);

        let consistent_output_size = if size_changes.len() > 0 {
            let first = size_changes[0].1;
            if size_changes.iter().all(|(_, o)| *o == first) {
                Some(first)
            } else {
                None
            }
        } else {
            None
        };

        // Check for scale factor
        let scale_factor = if !size_preserving && !size_changes.is_empty() {
            let ((ih, iw), (oh, ow)) = size_changes[0];
            if ih > 0 && iw > 0 && oh % ih == 0 && ow % iw == 0 {
                let sh = oh / ih;
                let sw = ow / iw;
                if size_changes.iter().all(|((ih2, iw2), (oh2, ow2))| {
                    *ih2 > 0 && *iw2 > 0 && oh2 / ih2 == sh && ow2 / iw2 == sw
                }) {
                    Some((sh, sw))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // ── Colour mapping ───────────────────────────────────────────────
        let colour_map = self.induce_colour_map(&task.train);

        // ── Detect transform type ────────────────────────────────────────
        let transform_type = self.classify_transform(
            &task.train, size_preserving, &colour_map, scale_factor,
        );

        // ── Cell patterns (for size-preserving transforms) ───────────────
        let cell_patterns = if size_preserving && task.train.len() >= 2 {
            self.induce_cell_patterns(&task.train)
        } else {
            Vec::new()
        };

        // ── Spatial relations from training ──────────────────────────────
        let mut relation_triples = Vec::new();
        for ex in &task.train {
            let grid = make_grid(&ex.input);
            relation_triples.extend(extract_spatial_relations(&grid, 4));
        }

        TransformRule {
            mean_delta,
            size_changes,
            colour_map,
            size_preserving,
            consistent_output_size,
            scale_factor,
            transform_type,
            cell_patterns,
            relation_triples,
            delta_consistency,
        }
    }

    /// Induce a colour mapping from training examples.
    fn induce_colour_map(&self, examples: &[ArcExample]) -> HashMap<u8, u8> {
        // For each input colour, track what output colour it maps to at the same position
        let mut mappings: HashMap<u8, HashMap<u8, usize>> = HashMap::new();

        for ex in examples {
            let ih = ex.input.len();
            let iw = ex.input.first().map_or(0, |r| r.len());
            let oh = ex.output.len();
            let ow = ex.output.first().map_or(0, |r| r.len());

            if ih == oh && iw == ow {
                for r in 0..ih {
                    for c in 0..iw {
                        let ic = ex.input[r][c];
                        let oc = ex.output[r][c];
                        *mappings.entry(ic).or_default().entry(oc).or_insert(0) += 1;
                    }
                }
            }
        }

        let mut colour_map = HashMap::new();
        for (ic, counts) in &mappings {
            if let Some((&oc, _)) = counts.iter().max_by_key(|(_, &ct)| ct) {
                colour_map.insert(*ic, oc);
            }
        }
        colour_map
    }

    /// Classify the transformation type based on structural analysis.
    fn classify_transform(
        &self,
        examples: &[ArcExample],
        size_preserving: bool,
        colour_map: &HashMap<u8, u8>,
        scale_factor: Option<(usize, usize)>,
    ) -> TransformType {
        if let Some(_) = scale_factor {
            // Check if it's tiling (repeating input) vs scaling (enlarging cells)
            if examples.len() > 0 {
                let ex = &examples[0];
                let ih = ex.input.len();
                let iw = ex.input.first().map_or(0, |r| r.len());
                let oh = ex.output.len();
                let ow = ex.output.first().map_or(0, |r| r.len());

                if oh > 0 && ow > 0 && ih > 0 && iw > 0 {
                    // Check tiling: output[r][c] == input[r % ih][c % iw]?
                    let is_tile = (0..oh).all(|r| {
                        (0..ow).all(|c| {
                            ex.output[r][c] == ex.input[r % ih][c % iw]
                        })
                    });
                    if is_tile { return TransformType::Tile; }

                    // Check scaling: each input cell expands to a block
                    let sh = oh / ih;
                    let sw = ow / iw;
                    let is_scale = (0..ih).all(|r| {
                        (0..iw).all(|c| {
                            let expected = ex.input[r][c];
                            (0..sh).all(|dr| {
                                (0..sw).all(|dc| {
                                    ex.output[r * sh + dr][c * sw + dc] == expected
                                })
                            })
                        })
                    });
                    if is_scale { return TransformType::Scale; }
                }
            }
            return TransformType::Scale;
        }

        if size_preserving {
            // Check if pure colour remap
            let is_pure_remap = examples.iter().all(|ex| {
                let ih = ex.input.len();
                let iw = ex.input.first().map_or(0, |r| r.len());
                (0..ih).all(|r| {
                    (0..iw).all(|c| {
                        colour_map.get(&ex.input[r][c]).copied().unwrap_or(ex.input[r][c]) == ex.output[r][c]
                    })
                })
            });
            if is_pure_remap { return TransformType::ColourRemap; }

            // Check geometric transforms (rotation, flip)
            for ex in examples {
                if is_rotation_90(&ex.input, &ex.output) { return TransformType::Geometric; }
                if is_horizontal_flip(&ex.input, &ex.output) { return TransformType::Geometric; }
                if is_vertical_flip(&ex.input, &ex.output) { return TransformType::Geometric; }
            }
        }

        // Check fill patterns
        if size_preserving && self.is_fill_pattern(examples) {
            return TransformType::Fill;
        }

        TransformType::Complex
    }

    /// Check if the transform is a fill pattern (flood fill, region colouring, etc.)
    fn is_fill_pattern(&self, examples: &[ArcExample]) -> bool {
        // Heuristic: if most cells stay the same and only specific regions change
        for ex in examples {
            let ih = ex.input.len();
            let iw = ex.input.first().map_or(0, |r| r.len());
            if ih != ex.output.len() { return false; }
            let mut same = 0;
            let mut diff = 0;
            for r in 0..ih {
                for c in 0..iw {
                    if ex.input[r][c] == ex.output[r][c] { same += 1; } else { diff += 1; }
                }
            }
            // Fill pattern: majority of cells unchanged, minority changed
            if diff == 0 || diff > same { return false; }
        }
        true
    }

    /// Induce per-cell patterns from size-preserving transforms.
    fn induce_cell_patterns(&self, examples: &[ArcExample]) -> Vec<CellPattern> {
        if examples.is_empty() { return Vec::new(); }

        let oh = examples[0].output.len();
        let ow = examples[0].output.first().map_or(0, |r| r.len());

        let mut patterns = Vec::new();

        for r in 0..oh {
            for c in 0..ow {
                // Check if output[r][c] is always a constant
                let first_val = examples[0].output[r][c];
                let all_same = examples.iter().all(|ex| {
                    ex.output.get(r).and_then(|row| row.get(c)).copied() == Some(first_val)
                });
                if all_same && examples.len() >= 2 {
                    patterns.push(CellPattern {
                        row: r, col: c,
                        source: CellSource::Constant(first_val),
                    });
                    continue;
                }

                // Check if output[r][c] always copies from input[r][c]
                let copies_self = examples.iter().all(|ex| {
                    let iv = ex.input.get(r).and_then(|row| row.get(c)).copied();
                    let ov = ex.output.get(r).and_then(|row| row.get(c)).copied();
                    iv == ov
                });
                if copies_self {
                    patterns.push(CellPattern {
                        row: r, col: c,
                        source: CellSource::Copy(r, c),
                    });
                    continue;
                }

                // Check all possible source positions
                let ih = examples[0].input.len();
                let iw = examples[0].input.first().map_or(0, |r| r.len());
                let mut found_source = false;
                for sr in 0..ih {
                    for sc in 0..iw {
                        let matches = examples.iter().all(|ex| {
                            let iv = ex.input.get(sr).and_then(|row| row.get(sc)).copied();
                            let ov = ex.output.get(r).and_then(|row| row.get(c)).copied();
                            iv == ov
                        });
                        if matches {
                            patterns.push(CellPattern {
                                row: r, col: c,
                                source: CellSource::Copy(sr, sc),
                            });
                            found_source = true;
                            break;
                        }
                    }
                    if found_source { break; }
                }

                if !found_source {
                    patterns.push(CellPattern {
                        row: r, col: c,
                        source: CellSource::Unknown,
                    });
                }
            }
        }

        patterns
    }

    /// Register discovered causal rules from a transform into the causal model.
    fn register_causal_rules(&mut self, rule: &TransformRule, task_id: &str) {
        // Add a causal edge: input_embedding → output_embedding with the transform delta
        let delta_magnitude: f32 = rule.mean_delta.iter().map(|x| x * x).sum::<f32>().sqrt();

        if delta_magnitude > 0.01 {
            self.causal_model.add_discovered_rule(
                &format!("{}_input", task_id),
                &format!("{}_output", task_id),
                SymbolicDerivative::Constant(delta_magnitude as f64),
                &format!("arc_transform_{}", task_id),
            );
        }

        // Add colour mapping rules
        for (&ic, &oc) in &rule.colour_map {
            if ic != oc {
                self.causal_model.add_discovered_rule(
                    &format!("colour_{}", ic),
                    &format!("colour_{}", oc),
                    SymbolicDerivative::Constant(1.0),
                    &format!("colour_remap_{}_to_{}", ic, oc),
                );
            }
        }
    }

    /// Use the Eustress Vortex DSL to find programs that generalize.
    /// Enumerates single DSL ops and 2-op compositions, verifies against training.
    /// Also uses GridAnalyzer properties to guide search + CausalGraph for learning.
    fn solve_with_vortex(
        &mut self,
        task: &ArcTask,
        test_input: &ArcGrid,
        verbose: bool,
    ) -> Option<Vec<Vec<u8>>> {
        if task.train.is_empty() { return None; }

        let sample_grid = Grid2D::new(task.train[0].input.clone());
        let all_ops = GridDSL::all_ops(&sample_grid);

        // ── Property-guided analysis ─────────────────────────────────────
        let _props = GridAnalyzer::analyze(&sample_grid);

        // ── Single DSL ops ───────────────────────────────────────────────
        for op in &all_ops {
            let all_match = task.train.iter().all(|ex| {
                let input = Grid2D::new(ex.input.clone());
                GridDSL::apply(&input, op).cells == ex.output
            });
            if all_match {
                let test_grid = Grid2D::new(test_input.cells.clone());
                let result = GridDSL::apply(&test_grid, op);
                if verbose {
                    println!("  [VORTEX] VERIFIED single DSL op: {}", op.name);
                }
                return Some(result.cells);
            }
        }

        // ── Two-op compositions (structural × all, small grids only) ──
        let max_cells = task.train.iter()
            .map(|ex| ex.input.len() * ex.input.first().map_or(0, |r| r.len()))
            .max().unwrap_or(0);
        if max_cells <= 225 {
            let structural_ops: Vec<&DSLOp> = all_ops.iter()
                .filter(|op| !op.name.starts_with("recolor_") && !op.name.starts_with("flood_fill_"))
                .collect();

            for op1 in &structural_ops {
                for op2 in &all_ops {
                    let all_match = task.train.iter().all(|ex| {
                        let input = Grid2D::new(ex.input.clone());
                        let mid = GridDSL::apply(&input, op1);
                        GridDSL::apply(&mid, op2).cells == ex.output
                    });
                    if all_match {
                        let test_grid = Grid2D::new(test_input.cells.clone());
                        let mid = GridDSL::apply(&test_grid, op1);
                        let result = GridDSL::apply(&mid, op2);
                        if verbose {
                            println!("  [VORTEX] VERIFIED 2-op: {} → {}", op1.name, op2.name);
                        }
                        return Some(result.cells);
                    }
                }
            }
        }

        // ── Separator-based region operations ────────────────────────────
        if let Some(result) = self.try_separator_strategies(test_input, task, verbose) {
            return Some(result);
        }

        // ── Object-level strategies ──────────────────────────────────────
        if let Some(result) = self.try_object_strategies(test_input, task, verbose) {
            return Some(result);
        }

        None
    }

    /// Detect separator lines and try region operations (overlay, XOR, select).
    fn try_separator_strategies(
        &self,
        test_input: &ArcGrid,
        task: &ArcTask,
        verbose: bool,
    ) -> Option<Vec<Vec<u8>>> {
        let regions_ops = ["overlay_or", "overlay_and", "overlay_xor",
                           "select_first", "select_last", "majority_vote",
                           "diff_regions", "overlay_last_wins", "overlay_first_wins",
                           "mask_first_with_second", "mask_second_with_first",
                           "subtract", "count_nonzero"];

        // Try splitting by horizontal and vertical separator lines
        for sep_color in 0u8..=9 {
            let h_seps = find_separator_rows(&task.train[0].input, sep_color);
            let v_seps = find_separator_cols(&task.train[0].input, sep_color);

            // Try horizontal split + region operations
            if !h_seps.is_empty() {
                for op_name in &regions_ops {
                    // Try direct operation
                    if let Some(result) = try_region_op_h(task, test_input, sep_color, op_name, None) {
                        if verbose { println!("  [VORTEX] VERIFIED h-sep({}) + {}", sep_color, op_name); }
                        return Some(result);
                    }
                    // Try with recolor: all non-bg → constant color
                    for recolor in 1u8..=9 {
                        if let Some(result) = try_region_op_h(task, test_input, sep_color, op_name, Some(recolor)) {
                            if verbose { println!("  [VORTEX] VERIFIED h-sep({}) + {} + recolor({})", sep_color, op_name, recolor); }
                            return Some(result);
                        }
                    }
                }
            }

            // Try vertical split + region operations
            if !v_seps.is_empty() {
                for op_name in &regions_ops {
                    if let Some(result) = try_region_op_v(task, test_input, sep_color, op_name, None) {
                        if verbose { println!("  [VORTEX] VERIFIED v-sep({}) + {}", sep_color, op_name); }
                        return Some(result);
                    }
                    for recolor in 1u8..=9 {
                        if let Some(result) = try_region_op_v(task, test_input, sep_color, op_name, Some(recolor)) {
                            if verbose { println!("  [VORTEX] VERIFIED v-sep({}) + {} + recolor({})", sep_color, op_name, recolor); }
                            return Some(result);
                        }
                    }
                }
            }
        }

        None
    }

    /// Object-level strategies: extract objects, match by properties, transform.
    fn try_object_strategies(
        &self,
        test_input: &ArcGrid,
        task: &ArcTask,
        verbose: bool,
    ) -> Option<Vec<Vec<u8>>> {
        // Strategy: Recolor objects by size (e.g., smallest object gets color X)
        // Strategy: Fill enclosed regions with the object's border color
        // Strategy: Copy/move objects to fill holes
        // Strategy: Remove the most/least common object

        // ── Recolor by object size ranking ───────────────────────────────
        // Detect if output recolors objects based on their size order
        if let Some(result) = try_recolor_by_size(test_input, task, verbose) {
            return Some(result);
        }

        // ── Fill enclosed with nearest border color ──────────────────────
        if let Some(result) = try_fill_enclosed_per_object(test_input, task, verbose) {
            return Some(result);
        }

        // ── Keep only objects of a specific color ────────────────────────
        if let Some(result) = try_keep_color_objects(test_input, task, verbose) {
            return Some(result);
        }

        // ── Replace each object with its bounding-box filled version ─────
        if let Some(result) = try_fill_object_bbox(test_input, task, verbose) {
            return Some(result);
        }

        None
    }

    /// Predict the output grid given a test input and the induced transformation rule.
    ///
    /// Universal approach: try ALL strategies, verify each against training data,
    /// and use the first one that perfectly reproduces all training outputs.
    fn predict_output(
        &mut self,
        test_input: &ArcGrid,
        rule: &TransformRule,
        task: &ArcTask,
        verbose: bool,
    ) -> Vec<Vec<u8>> {
        // ── Phase 0: Eustress Vortex DSL solve ──────────────────────────
        // Use vortex-core's universal solve loop + Grid2D DSL to find
        // composable programs that generalize across training examples.
        if let Some(result) = self.solve_with_vortex(task, test_input, verbose) {
            return result;
        }

        // ── Phase 1: Training-verified strategy search ───────────────────
        // Try every strategy; if one perfectly reproduces ALL training outputs, use it.
        let strategy_names = [
            // Geometric (fast, exact)
            "identity",
            "rotate_90", "rotate_180", "rotate_270",
            "h_flip", "v_flip",
            "transpose", "anti_transpose",
            // Colour
            "colour_remap",
            // Size-changing
            "scale", "tile",
            // Structural
            "gravity_down", "gravity_up", "gravity_left", "gravity_right",
            "sort_rows_by_color", "sort_cols_by_color",
            "crop_to_content",
            // Cell-level
            "cell_patterns",
            "fill_neighbours",
            "fill_3x3",
            "fill_enclosed",
            // Composite: geometric + colour remap
            "rotate_90_remap", "rotate_180_remap", "rotate_270_remap",
            "h_flip_remap", "v_flip_remap", "transpose_remap",
            // Mirror fill: copy one half to the other
            "mirror_left_to_right", "mirror_right_to_left",
            "mirror_top_to_bottom", "mirror_bottom_to_top",
            // Reverse
            "reverse_rows", "reverse_cols",
            // Analogical reasoning
            "analogical_5x5",
        ];

        for name in &strategy_names {
            let all_match = task.train.iter().all(|ex| {
                let input = make_grid(&ex.input);
                let pred = self.apply_strategy(name, &input, rule, task);
                pred == ex.output
            });
            if all_match {
                if verbose {
                    println!("  [VERIFIED] Strategy '{}' matches all training examples", name);
                }
                return self.apply_strategy(name, test_input, rule, task);
            }
        }

        // ── Phase 1.5: Dynamic extraction strategies (for shrink/crop tasks) ──
        // These generate candidates based on training data structure.
        if let Some(result) = self.try_extraction_strategies(test_input, rule, task, verbose) {
            return result;
        }

        // ── Phase 2: Partial-match scoring ───────────────────────────────
        // No strategy perfectly matches all training. Find the best partial match.
        let mut best_candidate = test_input.cells.clone();
        let mut best_score: usize = 0;

        for name in &strategy_names {
            let mut total_correct_cells = 0usize;
            let mut total_cells = 0usize;
            let mut all_sizes_match = true;

            for ex in &task.train {
                let input = make_grid(&ex.input);
                let pred = self.apply_strategy(name, &input, rule, task);
                let oh = ex.output.len();
                let ow = ex.output.first().map_or(0, |r| r.len());

                if pred.len() != oh || pred.first().map_or(0, |r| r.len()) != ow {
                    all_sizes_match = false;
                    continue;
                }

                for r in 0..oh {
                    for c in 0..ow {
                        total_cells += 1;
                        if pred[r][c] == ex.output[r][c] {
                            total_correct_cells += 1;
                        }
                    }
                }
            }

            // Only consider strategies that produce the right output dimensions
            if all_sizes_match && total_correct_cells > best_score {
                best_score = total_correct_cells;
                best_candidate = self.apply_strategy(name, test_input, rule, task);
            }
        }

        if verbose && best_score > 0 {
            println!("  [PARTIAL] Best partial match: {} cells correct", best_score);
        }

        // ── Phase 3: Embedding NN fallback ──────────────────────────────
        // If no strategy even partially matches, try the embedding-based predictor
        // as a last resort (slower but captures latent structure).
        if best_score == 0 {
            return self.predict_by_embedding_nn(test_input, rule, task);
        }

        best_candidate
    }

    /// Dynamic extraction strategies for shrink/crop tasks.
    /// These generate candidates based on training data structure rather than fixed transforms.
    fn try_extraction_strategies(
        &self,
        test_input: &ArcGrid,
        rule: &TransformRule,
        task: &ArcTask,
        verbose: bool,
    ) -> Option<Vec<Vec<u8>>> {
        // Only apply to size-changing tasks
        if rule.size_preserving { return None; }

        // ── Strategy A: Extract bounding box of each non-bg color ───────
        // For each color, crop to its bounding box and check training
        for color in 1u8..=15 {
            let extractor = |grid: &[Vec<u8>]| -> Vec<Vec<u8>> {
                extract_color_bbox(grid, color)
            };
            let all_match = task.train.iter().all(|ex| {
                extractor(&ex.input) == ex.output
            });
            if all_match {
                if verbose { println!("  [VERIFIED] extract_color_bbox({})", color); }
                return Some(extractor(&test_input.cells));
            }
        }

        // ── Strategy B: Extract interior of framed rectangle ────────────
        // Look for a rectangular border of one color; extract the inside
        for border_color in 0u8..=15 {
            let extractor = |grid: &[Vec<u8>]| -> Vec<Vec<u8>> {
                extract_framed_interior(grid, border_color)
            };
            let all_match = task.train.iter().all(|ex| {
                extractor(&ex.input) == ex.output
            });
            if all_match {
                if verbose { println!("  [VERIFIED] extract_framed_interior({})", border_color); }
                return Some(extractor(&test_input.cells));
            }
        }

        // ── Strategy C: Subgrid enumeration (for consistent output sizes) ──
        // If all training outputs are the same size, enumerate all subgrids of
        // that size from each training input and find a consistent extraction rule.
        if let Some((oh, ow)) = rule.consistent_output_size {
            // Try: is the output always at the same relative position?
            // Find which (row_offset, col_offset) produces the right output for all training
            let positions = find_consistent_subgrid_position(task, oh, ow);
            if let Some((dr, dc)) = positions {
                let result = extract_subgrid(&test_input.cells, dr, dc, oh, ow);
                if verbose { println!("  [VERIFIED] subgrid at ({}, {}) size {}x{}", dr, dc, oh, ow); }
                return Some(result);
            }

            // Try: output is the subgrid containing the most non-bg cells
            let extractor = |grid: &[Vec<u8>]| -> Vec<Vec<u8>> {
                extract_densest_subgrid(grid, oh, ow)
            };
            let all_match = task.train.iter().all(|ex| {
                extractor(&ex.input) == ex.output
            });
            if all_match {
                if verbose { println!("  [VERIFIED] densest_subgrid({}x{})", oh, ow); }
                return Some(extractor(&test_input.cells));
            }
        }

        // ── Strategy D: Extract unique/non-repeating subgrid ────────────
        // For tiled inputs, extract the unique tile unit
        {
            let extractor = |grid: &[Vec<u8>]| -> Vec<Vec<u8>> {
                extract_tile_unit(grid)
            };
            let all_match = task.train.iter().all(|ex| {
                extractor(&ex.input) == ex.output
            });
            if all_match && !task.train.is_empty() {
                if verbose { println!("  [VERIFIED] extract_tile_unit"); }
                return Some(extractor(&test_input.cells));
            }
        }

        // ── Strategy E: Extract each color's bbox and check all ─────────
        // Some tasks output the bbox of the LARGEST non-bg object
        {
            let extractor = |grid: &[Vec<u8>]| -> Vec<Vec<u8>> {
                extract_largest_object_bbox(grid)
            };
            let all_match = task.train.iter().all(|ex| {
                extractor(&ex.input) == ex.output
            });
            if all_match {
                if verbose { println!("  [VERIFIED] extract_largest_object_bbox"); }
                return Some(extractor(&test_input.cells));
            }
        }

        // ── Strategy F: Extract smallest object bbox ────────────────────
        {
            let extractor = |grid: &[Vec<u8>]| -> Vec<Vec<u8>> {
                extract_smallest_object_bbox(grid)
            };
            let all_match = task.train.iter().all(|ex| {
                extractor(&ex.input) == ex.output
            });
            if all_match {
                if verbose { println!("  [VERIFIED] extract_smallest_object_bbox"); }
                return Some(extractor(&test_input.cells));
            }
        }

        None
    }

    /// Apply a named strategy to an input grid.
    fn apply_strategy(
        &self,
        name: &str,
        input: &ArcGrid,
        rule: &TransformRule,
        task: &ArcTask,
    ) -> Vec<Vec<u8>> {
        match name {
            "identity" => input.cells.clone(),
            "rotate_90" => rotate_90(&input.cells),
            "rotate_180" => rotate_180(&input.cells),
            "rotate_270" => rotate_270(&input.cells),
            "h_flip" => horizontal_flip(&input.cells),
            "v_flip" => vertical_flip(&input.cells),
            "transpose" => transpose(&input.cells),
            "anti_transpose" => anti_transpose(&input.cells),
            "colour_remap" => self.predict_colour_remap(input, rule),
            "scale" => self.predict_scale(input, rule),
            "tile" => self.predict_tile(input, rule),
            "gravity_down" => gravity(&input.cells, GravityDir::Down),
            "gravity_up" => gravity(&input.cells, GravityDir::Up),
            "gravity_left" => gravity(&input.cells, GravityDir::Left),
            "gravity_right" => gravity(&input.cells, GravityDir::Right),
            "sort_rows_by_color" => sort_rows_by_color(&input.cells),
            "sort_cols_by_color" => sort_cols_by_color(&input.cells),
            "crop_to_content" => crop_to_content(&input.cells),
            "cell_patterns" => self.predict_by_cell_patterns(input, rule, task),
            "fill_neighbours" => self.predict_fill(input, rule, task),
            "fill_3x3" => self.predict_fill_3x3(input, task),
            "fill_enclosed" => fill_enclosed(&input.cells),
            "rotate_90_remap" => self.predict_colour_remap(&make_grid(&rotate_90(&input.cells)), rule),
            "rotate_180_remap" => self.predict_colour_remap(&make_grid(&rotate_180(&input.cells)), rule),
            "rotate_270_remap" => self.predict_colour_remap(&make_grid(&rotate_270(&input.cells)), rule),
            "h_flip_remap" => self.predict_colour_remap(&make_grid(&horizontal_flip(&input.cells)), rule),
            "v_flip_remap" => self.predict_colour_remap(&make_grid(&vertical_flip(&input.cells)), rule),
            "transpose_remap" => self.predict_colour_remap(&make_grid(&transpose(&input.cells)), rule),
            "mirror_left_to_right" => mirror_lr(&input.cells, true),
            "mirror_right_to_left" => mirror_lr(&input.cells, false),
            "mirror_top_to_bottom" => mirror_tb(&input.cells, true),
            "mirror_bottom_to_top" => mirror_tb(&input.cells, false),
            "reverse_rows" => input.cells.iter().map(|r| r.iter().rev().copied().collect()).collect(),
            "reverse_cols" => input.cells.iter().rev().cloned().collect(),
            "analogical_5x5" => self.predict_analogical(input, task, 5),
            _ => input.cells.clone(),
        }
    }

    fn predict_colour_remap(&self, input: &ArcGrid, rule: &TransformRule) -> Vec<Vec<u8>> {
        let mut output = vec![vec![0u8; input.width]; input.height];
        for r in 0..input.height {
            for c in 0..input.width {
                let ic = input.at(r, c);
                output[r][c] = rule.colour_map.get(&ic).copied().unwrap_or(ic);
            }
        }
        output
    }

    fn predict_scale(&self, input: &ArcGrid, rule: &TransformRule) -> Vec<Vec<u8>> {
        if let Some((sh, sw)) = rule.scale_factor {
            let oh = input.height * sh;
            let ow = input.width * sw;
            let mut output = vec![vec![0u8; ow]; oh];
            for r in 0..input.height {
                for c in 0..input.width {
                    let val = input.at(r, c);
                    for dr in 0..sh {
                        for dc in 0..sw {
                            output[r * sh + dr][c * sw + dc] = val;
                        }
                    }
                }
            }
            output
        } else {
            // Fallback: copy input
            input.cells.clone()
        }
    }

    fn predict_tile(&self, input: &ArcGrid, rule: &TransformRule) -> Vec<Vec<u8>> {
        if let Some((oh, ow)) = rule.consistent_output_size {
            let mut output = vec![vec![0u8; ow]; oh];
            for r in 0..oh {
                for c in 0..ow {
                    output[r][c] = input.at(r % input.height, c % input.width);
                }
            }
            output
        } else if let Some((sh, sw)) = rule.scale_factor {
            let oh = input.height * sh;
            let ow = input.width * sw;
            let mut output = vec![vec![0u8; ow]; oh];
            for r in 0..oh {
                for c in 0..ow {
                    output[r][c] = input.at(r % input.height, c % input.width);
                }
            }
            output
        } else {
            input.cells.clone()
        }
    }

    // predict_geometric removed — universal predict_output tries all transforms now
    fn _predict_geometric_unused(&self, input: &ArcGrid, task: &ArcTask) -> Vec<Vec<u8>> {
        if task.train.is_empty() { return input.cells.clone(); }
        let ex = &task.train[0];

        if is_rotation_90(&ex.input, &ex.output) {
            return rotate_90(&input.cells);
        }
        if is_horizontal_flip(&ex.input, &ex.output) {
            return horizontal_flip(&input.cells);
        }
        if is_vertical_flip(&ex.input, &ex.output) {
            return vertical_flip(&input.cells);
        }
        if is_rotation_180(&ex.input, &ex.output) {
            return rotate_180(&input.cells);
        }
        if is_rotation_270(&ex.input, &ex.output) {
            return rotate_270(&input.cells);
        }
        if is_transpose(&ex.input, &ex.output) {
            return transpose(&input.cells);
        }

        input.cells.clone()
    }

    fn predict_fill(&self, input: &ArcGrid, rule: &TransformRule, task: &ArcTask) -> Vec<Vec<u8>> {
        // Try cell patterns first
        let result = self.predict_by_cell_patterns(input, rule, task);
        if !rule.cell_patterns.iter().any(|p| matches!(p.source, CellSource::Unknown)) {
            return result;
        }

        // Fallback: learn which cells change and what they change to
        // Use majority voting across training examples
        if task.train.is_empty() { return input.cells.clone(); }

        let mut output = input.cells.clone();
        let ex = &task.train[0];

        // Find the "change rule": when input has colour X at a cell, and certain
        // neighbours have colour Y, the output becomes Z
        let mut change_rules: HashMap<(u8, Vec<u8>), u8> = HashMap::new();

        for train_ex in &task.train {
            let ih = train_ex.input.len();
            let iw = train_ex.input.first().map_or(0, |r| r.len());
            for r in 0..ih {
                for c in 0..iw {
                    if train_ex.input[r][c] != train_ex.output[r][c] {
                        let ic = train_ex.input[r][c];
                        let neighbours = get_neighbours(&train_ex.input, r, c);
                        change_rules.insert((ic, neighbours), train_ex.output[r][c]);
                    }
                }
            }
        }

        // Apply change rules
        for r in 0..input.height {
            for c in 0..input.width {
                let ic = input.at(r, c);
                let neighbours = get_neighbours(&input.cells, r, c);
                if let Some(&new_val) = change_rules.get(&(ic, neighbours)) {
                    output[r][c] = new_val;
                }
            }
        }

        output
    }

    fn predict_by_cell_patterns(
        &self, input: &ArcGrid, rule: &TransformRule, task: &ArcTask,
    ) -> Vec<Vec<u8>> {
        if rule.cell_patterns.is_empty() {
            // Don't fall through to slow embedding NN — just return input
            return input.cells.clone();
        }

        // Determine output size
        let (oh, ow) = if let Some(size) = rule.consistent_output_size {
            size
        } else if rule.size_preserving {
            (input.height, input.width)
        } else {
            // Infer from scale or training examples
            if let Some((sh, sw)) = rule.scale_factor {
                (input.height * sh, input.width * sw)
            } else {
                (input.height, input.width)
            }
        };

        let mut output = vec![vec![0u8; ow]; oh];
        let mut has_unknown = false;

        for pattern in &rule.cell_patterns {
            if pattern.row >= oh || pattern.col >= ow { continue; }
            match &pattern.source {
                CellSource::Constant(v) => {
                    output[pattern.row][pattern.col] = *v;
                }
                CellSource::Copy(sr, sc) => {
                    output[pattern.row][pattern.col] = input.at(*sr, *sc);
                }
                CellSource::Remap(sr, sc) => {
                    let ic = input.at(*sr, *sc);
                    output[pattern.row][pattern.col] =
                        rule.colour_map.get(&ic).copied().unwrap_or(ic);
                }
                CellSource::Unknown => {
                    has_unknown = true;
                    output[pattern.row][pattern.col] = input.at(pattern.row, pattern.col);
                }
            }
        }

        output
    }

    /// Analogical cell transfer: for each test cell, find the most similar
    /// training cell (by NxN context window) and use its output value.
    /// Works for local, context-dependent transformations.
    fn predict_analogical(&self, input: &ArcGrid, task: &ArcTask, window: usize) -> Vec<Vec<u8>> {
        if !task.train.iter().all(|ex| ex.input.len() == ex.output.len()
            && ex.input.first().map_or(0, |r| r.len()) == ex.output.first().map_or(0, |r| r.len()))
        {
            return input.cells.clone(); // Only works for same-size transforms
        }

        // Build lookup: context_hash → output_value (from ALL training examples)
        let half = (window / 2) as i32;
        let mut context_map: HashMap<Vec<u8>, u8> = HashMap::new();

        for ex in &task.train {
            let ih = ex.input.len();
            let iw = ex.input.first().map_or(0, |r| r.len());
            for r in 0..ih {
                for c in 0..iw {
                    let ctx = get_nxn_context(&ex.input, r, c, half);
                    let out_val = ex.output[r][c];
                    context_map.insert(ctx, out_val);
                }
            }
        }

        // Apply to test input
        let mut output = input.cells.clone();
        for r in 0..input.height {
            for c in 0..input.width {
                let ctx = get_nxn_context(&input.cells, r, c, half);
                if let Some(&val) = context_map.get(&ctx) {
                    output[r][c] = val;
                }
                // If no exact match, keep input value (identity fallback)
            }
        }
        output
    }

    /// Fill prediction using 3x3 neighbourhood context.
    /// For each cell, uses a 3x3 window around it (9 values) as the key.
    fn predict_fill_3x3(&self, input: &ArcGrid, task: &ArcTask) -> Vec<Vec<u8>> {
        // Learn 3x3 → output_value mappings from training
        let mut rules: HashMap<Vec<u8>, u8> = HashMap::new();

        for ex in &task.train {
            let ih = ex.input.len();
            let iw = ex.input.first().map_or(0, |r| r.len());
            let oh = ex.output.len();
            let ow = ex.output.first().map_or(0, |r| r.len());
            if ih != oh || iw != ow { continue; }

            for r in 0..ih {
                for c in 0..iw {
                    if ex.input[r][c] != ex.output[r][c] {
                        let key = get_3x3_context(&ex.input, r, c);
                        rules.insert(key, ex.output[r][c]);
                    }
                }
            }
        }

        let mut output = input.cells.clone();
        for r in 0..input.height {
            for c in 0..input.width {
                let key = get_3x3_context(&input.cells, r, c);
                if let Some(&val) = rules.get(&key) {
                    output[r][c] = val;
                }
            }
        }
        output
    }

    /// Last-resort predictor: use embedding nearest-neighbour matching.
    /// Encode the test input, compute embedding + mean_delta, find the closest
    /// training output embedding, and use that training output as a template.
    fn predict_by_embedding_nn(
        &self, test_input: &ArcGrid, rule: &TransformRule, task: &ArcTask,
    ) -> Vec<Vec<u8>> {
        let test_enc = self.encoder.encode(test_input);

        // Predicted output embedding = test_input_embedding + mean_delta
        let d = EMBED_DIM;
        let mut predicted_emb = vec![0.0f32; d];
        for dd in 0..d {
            predicted_emb[dd] = test_enc.embedding[dd] + rule.mean_delta[dd];
        }

        // Find best matching training output
        let mut best_idx = 0;
        let mut best_sim = f32::NEG_INFINITY;

        for (i, ex) in task.train.iter().enumerate() {
            let out_grid = make_grid(&ex.output);
            let out_enc = self.encoder.encode(&out_grid);
            let sim = cosine_sim(&predicted_emb, &out_enc.embedding);
            if sim > best_sim {
                best_sim = sim;
                best_idx = i;
            }
        }

        // Use the matched training output as template, but try to adapt it
        let template = &task.train[best_idx].output;
        let template_input = &task.train[best_idx].input;

        // If test input is same size as template input, try cell-wise transfer
        if test_input.height == template_input.len()
            && test_input.width == template_input.first().map_or(0, |r| r.len())
        {
            let oh = template.len();
            let ow = template.first().map_or(0, |r| r.len());
            let mut output = vec![vec![0u8; ow]; oh];

            for r in 0..oh {
                for c in 0..ow {
                    // Where template_output differs from template_input,
                    // apply same difference pattern to test
                    let ti = template_input.get(r).and_then(|row| row.get(c)).copied().unwrap_or(0);
                    let to = template.get(r).and_then(|row| row.get(c)).copied().unwrap_or(0);

                    if r < test_input.height && c < test_input.width {
                        if ti == to {
                            // Cell unchanged: copy from test input
                            output[r][c] = test_input.at(r, c);
                        } else {
                            // Cell changed: apply the same transformation
                            let test_val = test_input.at(r, c);
                            if test_val == ti {
                                output[r][c] = to;
                            } else {
                                output[r][c] = rule.colour_map.get(&test_val).copied().unwrap_or(to);
                            }
                        }
                    } else {
                        output[r][c] = to;
                    }
                }
            }
            output
        } else {
            // Different sizes: just return the template (best we can do)
            template.clone()
        }
    }

    // predict_complex removed — universal predict_output handles all types now
}

// =============================================================================
// Geometric transform helpers
// =============================================================================

fn rotate_90(grid: &Vec<Vec<u8>>) -> Vec<Vec<u8>> {
    let h = grid.len();
    let w = grid.first().map_or(0, |r| r.len());
    let mut out = vec![vec![0u8; h]; w];
    for r in 0..h {
        for c in 0..w {
            out[c][h - 1 - r] = grid[r][c];
        }
    }
    out
}

fn rotate_180(grid: &Vec<Vec<u8>>) -> Vec<Vec<u8>> {
    let h = grid.len();
    let w = grid.first().map_or(0, |r| r.len());
    let mut out = vec![vec![0u8; w]; h];
    for r in 0..h {
        for c in 0..w {
            out[h - 1 - r][w - 1 - c] = grid[r][c];
        }
    }
    out
}

fn rotate_270(grid: &Vec<Vec<u8>>) -> Vec<Vec<u8>> {
    let h = grid.len();
    let w = grid.first().map_or(0, |r| r.len());
    let mut out = vec![vec![0u8; h]; w];
    for r in 0..h {
        for c in 0..w {
            out[w - 1 - c][r] = grid[r][c];
        }
    }
    out
}

fn horizontal_flip(grid: &Vec<Vec<u8>>) -> Vec<Vec<u8>> {
    grid.iter().map(|row| row.iter().rev().copied().collect()).collect()
}

fn vertical_flip(grid: &Vec<Vec<u8>>) -> Vec<Vec<u8>> {
    grid.iter().rev().cloned().collect()
}

fn transpose(grid: &Vec<Vec<u8>>) -> Vec<Vec<u8>> {
    let h = grid.len();
    let w = grid.first().map_or(0, |r| r.len());
    let mut out = vec![vec![0u8; h]; w];
    for r in 0..h {
        for c in 0..w {
            out[c][r] = grid[r][c];
        }
    }
    out
}

fn is_rotation_90(input: &Vec<Vec<u8>>, output: &Vec<Vec<u8>>) -> bool {
    let h = input.len();
    let w = input.first().map_or(0, |r| r.len());
    if output.len() != w || output.first().map_or(0, |r| r.len()) != h { return false; }
    (0..h).all(|r| (0..w).all(|c| output[c][h - 1 - r] == input[r][c]))
}

fn is_rotation_180(input: &Vec<Vec<u8>>, output: &Vec<Vec<u8>>) -> bool {
    let h = input.len();
    let w = input.first().map_or(0, |r| r.len());
    if output.len() != h || output.first().map_or(0, |r| r.len()) != w { return false; }
    (0..h).all(|r| (0..w).all(|c| output[h - 1 - r][w - 1 - c] == input[r][c]))
}

fn is_rotation_270(input: &Vec<Vec<u8>>, output: &Vec<Vec<u8>>) -> bool {
    let h = input.len();
    let w = input.first().map_or(0, |r| r.len());
    if output.len() != w || output.first().map_or(0, |r| r.len()) != h { return false; }
    (0..h).all(|r| (0..w).all(|c| output[w - 1 - c][r] == input[r][c]))
}

fn is_horizontal_flip(input: &Vec<Vec<u8>>, output: &Vec<Vec<u8>>) -> bool {
    let h = input.len();
    let w = input.first().map_or(0, |r| r.len());
    if output.len() != h || output.first().map_or(0, |r| r.len()) != w { return false; }
    (0..h).all(|r| (0..w).all(|c| output[r][w - 1 - c] == input[r][c]))
}

fn is_vertical_flip(input: &Vec<Vec<u8>>, output: &Vec<Vec<u8>>) -> bool {
    let h = input.len();
    let w = input.first().map_or(0, |r| r.len());
    if output.len() != h || output.first().map_or(0, |r| r.len()) != w { return false; }
    (0..h).all(|r| (0..w).all(|c| output[h - 1 - r][c] == input[r][c]))
}

fn is_transpose(input: &Vec<Vec<u8>>, output: &Vec<Vec<u8>>) -> bool {
    let h = input.len();
    let w = input.first().map_or(0, |r| r.len());
    if output.len() != w || output.first().map_or(0, |r| r.len()) != h { return false; }
    (0..h).all(|r| (0..w).all(|c| output[c][r] == input[r][c]))
}

// =============================================================================
// Extraction strategies for shrink/crop tasks
// =============================================================================

/// Extract the bounding box of all cells with the given color.
fn extract_color_bbox(grid: &[Vec<u8>], color: u8) -> Vec<Vec<u8>> {
    let h = grid.len();
    let w = grid.first().map_or(0, |r| r.len());
    let mut min_r = h; let mut max_r = 0;
    let mut min_c = w; let mut max_c = 0;
    for r in 0..h {
        for c in 0..w {
            if grid[r][c] == color {
                min_r = min_r.min(r); max_r = max_r.max(r);
                min_c = min_c.min(c); max_c = max_c.max(c);
            }
        }
    }
    if min_r > max_r { return grid.to_vec(); }
    (min_r..=max_r).map(|r| grid[r][min_c..=max_c].to_vec()).collect()
}

/// Find rectangular border of `border_color` and extract its interior.
fn extract_framed_interior(grid: &[Vec<u8>], border_color: u8) -> Vec<Vec<u8>> {
    let h = grid.len();
    let w = grid.first().map_or(0, |r| r.len());

    // Find the largest rectangle of border_color cells
    // Strategy: find contiguous top row of border_color, bottom row, left/right cols
    for top in 0..h {
        for left in 0..w {
            // Check if (top, left) starts a border frame
            if grid[top][left] != border_color { continue; }

            // Find right extent of top border
            let mut right = left;
            while right < w && grid[top][right] == border_color { right += 1; }
            right -= 1;
            if right <= left + 1 { continue; }

            // Find bottom extent of left border
            let mut bottom = top;
            while bottom < h && grid[bottom][left] == border_color { bottom += 1; }
            bottom -= 1;
            if bottom <= top + 1 { continue; }

            // Verify right column and bottom row are border_color
            let right_ok = (top..=bottom).all(|r| grid[r][right] == border_color);
            let bottom_ok = (left..=right).all(|c| grid[bottom][c] == border_color);

            if right_ok && bottom_ok && bottom > top + 1 && right > left + 1 {
                // Extract interior
                let interior: Vec<Vec<u8>> = (top+1..bottom)
                    .map(|r| grid[r][left+1..right].to_vec())
                    .collect();
                if !interior.is_empty() && !interior[0].is_empty() {
                    return interior;
                }
            }
        }
    }
    grid.to_vec()
}

/// Extract a subgrid at the given position.
fn extract_subgrid(grid: &[Vec<u8>], row: usize, col: usize, h: usize, w: usize) -> Vec<Vec<u8>> {
    (row..row+h).map(|r| {
        if r < grid.len() {
            let row_data = &grid[r];
            (col..col+w).map(|c| if c < row_data.len() { row_data[c] } else { 0 }).collect()
        } else {
            vec![0; w]
        }
    }).collect()
}

/// Find a consistent (row, col) position that extracts the correct output from all training inputs.
fn find_consistent_subgrid_position(task: &ArcTask, oh: usize, ow: usize) -> Option<(usize, usize)> {
    if task.train.is_empty() { return None; }

    // Use the smallest input dimensions to bound the search
    let max_dr = task.train.iter()
        .map(|ex| ex.input.len().saturating_sub(oh))
        .min().unwrap_or(0);
    let max_dc = task.train.iter()
        .map(|ex| ex.input.first().map_or(0, |r| r.len()).saturating_sub(ow))
        .min().unwrap_or(0);

    for dr in 0..=max_dr {
        for dc in 0..=max_dc {
            let all_match = task.train.iter().all(|ex| {
                let h = ex.input.len();
                let w = ex.input.first().map_or(0, |r| r.len());
                if dr + oh > h || dc + ow > w { return false; }
                let extracted: Vec<Vec<u8>> = (dr..dr+oh)
                    .map(|r| ex.input[r][dc..dc+ow].to_vec())
                    .collect();
                extracted == ex.output
            });
            if all_match { return Some((dr, dc)); }
        }
    }
    None
}

/// Extract the subgrid of given size that contains the most non-background cells.
fn extract_densest_subgrid(grid: &[Vec<u8>], oh: usize, ow: usize) -> Vec<Vec<u8>> {
    let h = grid.len();
    let w = grid.first().map_or(0, |r| r.len());
    if oh > h || ow > w || oh == 0 || ow == 0 { return grid.to_vec(); }

    // Background = most frequent
    let mut hist = [0usize; 16];
    for row in grid { for &c in row { hist[(c as usize) & 0xF] += 1; } }
    let bg = hist.iter().enumerate().max_by_key(|(_, &c)| c).map_or(0, |(i, _)| i as u8);

    let mut best_count = 0;
    let mut best_r = 0;
    let mut best_c = 0;

    for dr in 0..=h-oh {
        for dc in 0..=w-ow {
            let count: usize = (dr..dr+oh)
                .map(|r| {
                    let row = &grid[r];
                    (dc..dc+ow).filter(|&c| c < row.len() && row[c] != bg).count()
                })
                .sum();
            if count > best_count {
                best_count = count;
                best_r = dr;
                best_c = dc;
            }
        }
    }

    extract_subgrid(grid, best_r, best_c, oh, ow)
}

/// Extract the smallest repeating tile unit from a grid.
fn extract_tile_unit(grid: &[Vec<u8>]) -> Vec<Vec<u8>> {
    let h = grid.len();
    let w = grid.first().map_or(0, |r| r.len());

    for th in 1..=h/2 {
        if h % th != 0 { continue; }
        for tw in 1..=w/2 {
            if w % tw != 0 { continue; }
            let tile: Vec<Vec<u8>> = (0..th).map(|r| grid[r][0..tw].to_vec()).collect();
            let is_tile = (0..h).all(|r| (0..w).all(|c| grid[r][c] == tile[r % th][c % tw]));
            if is_tile {
                return tile;
            }
        }
    }
    grid.to_vec()
}

/// Extract the bounding box of the largest connected non-bg object.
fn extract_largest_object_bbox(grid: &[Vec<u8>]) -> Vec<Vec<u8>> {
    let objects = find_objects(grid);
    if objects.is_empty() { return grid.to_vec(); }
    let largest = objects.iter().max_by_key(|o| o.len()).unwrap();
    let min_r = largest.iter().map(|&(r,_)| r).min().unwrap();
    let max_r = largest.iter().map(|&(r,_)| r).max().unwrap();
    let min_c = largest.iter().map(|&(_,c)| c).min().unwrap();
    let max_c = largest.iter().map(|&(_,c)| c).max().unwrap();
    (min_r..=max_r).map(|r| grid[r][min_c..=max_c].to_vec()).collect()
}

/// Extract the bounding box of the smallest connected non-bg object.
fn extract_smallest_object_bbox(grid: &[Vec<u8>]) -> Vec<Vec<u8>> {
    let objects = find_objects(grid);
    if objects.is_empty() { return grid.to_vec(); }
    let smallest = objects.iter().min_by_key(|o| o.len()).unwrap();
    let min_r = smallest.iter().map(|&(r,_)| r).min().unwrap();
    let max_r = smallest.iter().map(|&(r,_)| r).max().unwrap();
    let min_c = smallest.iter().map(|&(_,c)| c).min().unwrap();
    let max_c = smallest.iter().map(|&(_,c)| c).max().unwrap();
    (min_r..=max_r).map(|r| grid[r][min_c..=max_c].to_vec()).collect()
}

/// Find connected components of non-background cells.
fn find_objects(grid: &[Vec<u8>]) -> Vec<Vec<(usize, usize)>> {
    let h = grid.len();
    let w = grid.first().map_or(0, |r| r.len());

    let mut hist = [0usize; 16];
    for row in grid { for &c in row { hist[(c as usize) & 0xF] += 1; } }
    let bg = hist.iter().enumerate().max_by_key(|(_, &c)| c).map_or(0, |(i, _)| i as u8);

    let mut visited = vec![vec![false; w]; h];
    let mut objects = Vec::new();

    for r in 0..h {
        for c in 0..w {
            if grid[r][c] != bg && !visited[r][c] {
                let mut obj = Vec::new();
                let mut stack = vec![(r, c)];
                while let Some((sr, sc)) = stack.pop() {
                    if sr >= h || sc >= w || visited[sr][sc] || grid[sr][sc] == bg { continue; }
                    visited[sr][sc] = true;
                    obj.push((sr, sc));
                    if sr > 0 { stack.push((sr-1, sc)); }
                    if sc > 0 { stack.push((sr, sc-1)); }
                    stack.push((sr+1, sc));
                    stack.push((sr, sc+1));
                }
                if !obj.is_empty() { objects.push(obj); }
            }
        }
    }
    objects
}

// =============================================================================
// Mirror and other transform helpers
// =============================================================================

/// Mirror left half to right (left_to_right=true) or right to left.
fn mirror_lr(grid: &[Vec<u8>], left_to_right: bool) -> Vec<Vec<u8>> {
    let h = grid.len();
    let w = grid.first().map_or(0, |r| r.len());
    let mut out = grid.to_vec();
    for r in 0..h {
        for c in 0..w / 2 {
            if left_to_right {
                out[r][w - 1 - c] = out[r][c];
            } else {
                out[r][c] = out[r][w - 1 - c];
            }
        }
    }
    out
}

/// Mirror top half to bottom (top_to_bottom=true) or bottom to top.
fn mirror_tb(grid: &[Vec<u8>], top_to_bottom: bool) -> Vec<Vec<u8>> {
    let h = grid.len();
    let mut out = grid.to_vec();
    for r in 0..h / 2 {
        if top_to_bottom {
            out[h - 1 - r] = out[r].clone();
        } else {
            out[r] = out[h - 1 - r].clone();
        }
    }
    out
}

fn anti_transpose(grid: &Vec<Vec<u8>>) -> Vec<Vec<u8>> {
    let h = grid.len();
    let w = grid.first().map_or(0, |r| r.len());
    let mut out = vec![vec![0u8; h]; w];
    for r in 0..h {
        for c in 0..w {
            out[w - 1 - c][h - 1 - r] = grid[r][c];
        }
    }
    out
}

#[derive(Clone, Copy)]
enum GravityDir { Down, Up, Left, Right }

fn gravity(grid: &[Vec<u8>], dir: GravityDir) -> Vec<Vec<u8>> {
    let h = grid.len();
    let w = grid.first().map_or(0, |r| r.len());
    if h == 0 || w == 0 { return grid.to_vec(); }

    // Detect background color (most frequent)
    let mut hist = [0usize; 16];
    for row in grid {
        for &c in row { hist[(c as usize) & 0xF] += 1; }
    }
    let bg = hist.iter().enumerate().max_by_key(|(_, &c)| c).map_or(0, |(i, _)| i as u8);

    let mut out = vec![vec![bg; w]; h];

    match dir {
        GravityDir::Down => {
            for c in 0..w {
                let mut write = h;
                for r in (0..h).rev() {
                    if grid[r][c] != bg {
                        write -= 1;
                        out[write][c] = grid[r][c];
                    }
                }
            }
        }
        GravityDir::Up => {
            for c in 0..w {
                let mut write = 0;
                for r in 0..h {
                    if grid[r][c] != bg {
                        out[write][c] = grid[r][c];
                        write += 1;
                    }
                }
            }
        }
        GravityDir::Left => {
            for r in 0..h {
                let mut write = 0;
                for c in 0..w {
                    if grid[r][c] != bg {
                        out[r][write] = grid[r][c];
                        write += 1;
                    }
                }
            }
        }
        GravityDir::Right => {
            for r in 0..h {
                let mut write = w;
                for c in (0..w).rev() {
                    if grid[r][c] != bg {
                        write -= 1;
                        out[r][write] = grid[r][c];
                    }
                }
            }
        }
    }
    out
}

fn sort_rows_by_color(grid: &[Vec<u8>]) -> Vec<Vec<u8>> {
    grid.iter().map(|row| {
        let mut sorted = row.clone();
        sorted.sort();
        sorted
    }).collect()
}

fn sort_cols_by_color(grid: &[Vec<u8>]) -> Vec<Vec<u8>> {
    let h = grid.len();
    let w = grid.first().map_or(0, |r| r.len());
    let mut out = grid.to_vec();
    for c in 0..w {
        let mut col: Vec<u8> = (0..h).map(|r| grid[r][c]).collect();
        col.sort();
        for r in 0..h {
            out[r][c] = col[r];
        }
    }
    out
}

fn crop_to_content(grid: &[Vec<u8>]) -> Vec<Vec<u8>> {
    let h = grid.len();
    let w = grid.first().map_or(0, |r| r.len());
    if h == 0 || w == 0 { return grid.to_vec(); }

    // Background = most frequent
    let mut hist = [0usize; 16];
    for row in grid {
        for &c in row { hist[(c as usize) & 0xF] += 1; }
    }
    let bg = hist.iter().enumerate().max_by_key(|(_, &c)| c).map_or(0, |(i, _)| i as u8);

    let mut min_r = h; let mut max_r = 0;
    let mut min_c = w; let mut max_c = 0;
    for r in 0..h {
        for c in 0..w {
            if grid[r][c] != bg {
                min_r = min_r.min(r); max_r = max_r.max(r);
                min_c = min_c.min(c); max_c = max_c.max(c);
            }
        }
    }

    if min_r > max_r { return grid.to_vec(); }
    (min_r..=max_r).map(|r| grid[r][min_c..=max_c].to_vec()).collect()
}

fn fill_enclosed(grid: &[Vec<u8>]) -> Vec<Vec<u8>> {
    let h = grid.len();
    let w = grid.first().map_or(0, |r| r.len());
    if h == 0 || w == 0 { return grid.to_vec(); }

    // Background = most frequent
    let mut hist = [0usize; 16];
    for row in grid {
        for &c in row { hist[(c as usize) & 0xF] += 1; }
    }
    let bg = hist.iter().enumerate().max_by_key(|(_, &c)| c).map_or(0, |(i, _)| i as u8);

    // Flood fill from edges to find exterior bg cells
    let mut exterior = vec![vec![false; w]; h];
    let mut stack: Vec<(usize, usize)> = Vec::new();
    // Seed from borders
    for c in 0..w {
        if grid[0][c] == bg { stack.push((0, c)); }
        if grid[h-1][c] == bg { stack.push((h-1, c)); }
    }
    for r in 1..h.saturating_sub(1) {
        if grid[r][0] == bg { stack.push((r, 0)); }
        if grid[r][w-1] == bg { stack.push((r, w-1)); }
    }
    while let Some((r, c)) = stack.pop() {
        if r >= h || c >= w || exterior[r][c] || grid[r][c] != bg { continue; }
        exterior[r][c] = true;
        if r > 0 { stack.push((r-1, c)); }
        if c > 0 { stack.push((r, c-1)); }
        stack.push((r+1, c));
        stack.push((r, c+1));
    }

    // Find the fill color: most common non-bg color
    let fill_color = {
        let mut h2 = [0usize; 16];
        for row in grid {
            for &c in row {
                if c != bg { h2[(c as usize) & 0xF] += 1; }
            }
        }
        h2.iter().enumerate().max_by_key(|(_, &c)| c).map_or(bg, |(i, _)| i as u8)
    };

    // Fill interior bg cells
    let mut out = grid.to_vec();
    for r in 0..h {
        for c in 0..w {
            if grid[r][c] == bg && !exterior[r][c] {
                out[r][c] = fill_color;
            }
        }
    }
    out
}

/// 3x3 neighbourhood context for a cell (uses 255 for out-of-bounds).
fn get_3x3_context(grid: &[Vec<u8>], r: usize, c: usize) -> Vec<u8> {
    let h = grid.len();
    let w = grid.first().map_or(0, |row| row.len());
    let mut ctx = Vec::with_capacity(9);
    for dr in [-1i32, 0, 1] {
        for dc in [-1i32, 0, 1] {
            let nr = r as i32 + dr;
            let nc = c as i32 + dc;
            if nr >= 0 && nr < h as i32 && nc >= 0 && nc < w as i32 {
                ctx.push(grid[nr as usize][nc as usize]);
            } else {
                ctx.push(255);
            }
        }
    }
    ctx
}

/// NxN neighbourhood context for a cell (uses 255 for out-of-bounds).
fn get_nxn_context(grid: &[Vec<u8>], r: usize, c: usize, half: i32) -> Vec<u8> {
    let h = grid.len();
    let w = grid.first().map_or(0, |row| row.len());
    let mut ctx = Vec::with_capacity(((2 * half + 1) * (2 * half + 1)) as usize);
    for dr in -half..=half {
        for dc in -half..=half {
            let nr = r as i32 + dr;
            let nc = c as i32 + dc;
            if nr >= 0 && nr < h as i32 && nc >= 0 && nc < w as i32 {
                ctx.push(grid[nr as usize][nc as usize]);
            } else {
                ctx.push(255);
            }
        }
    }
    ctx
}

// =============================================================================
// Separator detection + region operations
// =============================================================================

/// Try a region operation with horizontal split, optionally recoloring output.
fn try_region_op_h(
    task: &ArcTask, test_input: &ArcGrid, sep_color: u8, op_name: &str, recolor: Option<u8>,
) -> Option<Vec<Vec<u8>>> {
    let all_match = task.train.iter().all(|ex| {
        let regions = split_by_rows(&ex.input, sep_color);
        if regions.len() < 2 { return false; }
        let mut result = combine_regions(&regions, op_name);
        if let Some(rc) = recolor {
            recolor_nonbg(&mut result, rc, &ex.input);
        }
        result == ex.output
    });
    if !all_match { return None; }
    let regions = split_by_rows(&test_input.cells, sep_color);
    if regions.len() < 2 { return None; }
    let mut result = combine_regions(&regions, op_name);
    if let Some(rc) = recolor {
        recolor_nonbg(&mut result, rc, &test_input.cells);
    }
    Some(result)
}

/// Try a region operation with vertical split, optionally recoloring output.
fn try_region_op_v(
    task: &ArcTask, test_input: &ArcGrid, sep_color: u8, op_name: &str, recolor: Option<u8>,
) -> Option<Vec<Vec<u8>>> {
    let all_match = task.train.iter().all(|ex| {
        let regions = split_by_cols(&ex.input, sep_color);
        if regions.len() < 2 { return false; }
        let mut result = combine_regions(&regions, op_name);
        if let Some(rc) = recolor {
            recolor_nonbg(&mut result, rc, &ex.input);
        }
        result == ex.output
    });
    if !all_match { return None; }
    let regions = split_by_cols(&test_input.cells, sep_color);
    if regions.len() < 2 { return None; }
    let mut result = combine_regions(&regions, op_name);
    if let Some(rc) = recolor {
        recolor_nonbg(&mut result, rc, &test_input.cells);
    }
    Some(result)
}

/// Replace all non-background cells with a single color.
fn recolor_nonbg(grid: &mut Vec<Vec<u8>>, new_color: u8, original: &[Vec<u8>]) {
    // Detect bg from original grid
    let mut hist = [0usize; 16];
    for row in original { for &c in row { hist[c as usize & 0xF] += 1; } }
    let bg = hist.iter().enumerate().max_by_key(|(_, &c)| c).map_or(0, |(i, _)| i as u8);

    for row in grid.iter_mut() {
        for cell in row.iter_mut() {
            if *cell != bg {
                *cell = new_color;
            }
        }
    }
}

/// Find rows that are entirely one color (separator lines).
fn find_separator_rows(grid: &[Vec<u8>], color: u8) -> Vec<usize> {
    grid.iter().enumerate()
        .filter(|(_, row)| !row.is_empty() && row.iter().all(|&c| c == color))
        .map(|(i, _)| i)
        .collect()
}

/// Find columns that are entirely one color.
fn find_separator_cols(grid: &[Vec<u8>], color: u8) -> Vec<usize> {
    let h = grid.len();
    let w = grid.first().map_or(0, |r| r.len());
    (0..w).filter(|&c| (0..h).all(|r| grid[r][c] == color)).collect()
}

/// Split grid into regions by removing separator rows.
fn split_by_rows(grid: &[Vec<u8>], sep_color: u8) -> Vec<Vec<Vec<u8>>> {
    let mut regions = Vec::new();
    let mut current: Vec<Vec<u8>> = Vec::new();

    for row in grid {
        if !row.is_empty() && row.iter().all(|&c| c == sep_color) {
            if !current.is_empty() {
                regions.push(current.clone());
                current.clear();
            }
        } else {
            current.push(row.clone());
        }
    }
    if !current.is_empty() {
        regions.push(current);
    }
    regions
}

/// Split grid into regions by removing separator columns.
fn split_by_cols(grid: &[Vec<u8>], sep_color: u8) -> Vec<Vec<Vec<u8>>> {
    let h = grid.len();
    let w = grid.first().map_or(0, |r| r.len());

    // Find separator column positions
    let seps: Vec<usize> = (0..w)
        .filter(|&c| (0..h).all(|r| grid[r][c] == sep_color))
        .collect();

    if seps.is_empty() { return vec![grid.to_vec()]; }

    let mut regions = Vec::new();
    let mut boundaries = vec![0];
    for &s in &seps {
        boundaries.push(s);
        boundaries.push(s + 1);
    }
    boundaries.push(w);

    for chunk in boundaries.chunks(2) {
        if chunk.len() == 2 && chunk[0] < chunk[1] {
            let region: Vec<Vec<u8>> = grid.iter()
                .map(|row| row[chunk[0]..chunk[1]].to_vec())
                .collect();
            if !region.is_empty() && !region[0].is_empty() {
                regions.push(region);
            }
        }
    }
    regions
}

/// Combine regions using the specified operation.
fn combine_regions(regions: &[Vec<Vec<u8>>], op: &str) -> Vec<Vec<u8>> {
    if regions.is_empty() { return vec![]; }

    // All regions must be the same size
    let h = regions[0].len();
    let w = regions[0].first().map_or(0, |r| r.len());
    if !regions.iter().all(|r| r.len() == h && r.first().map_or(0, |row| row.len()) == w) {
        return regions[0].clone();
    }

    // Detect background color from first region
    let mut hist = [0usize; 16];
    for row in &regions[0] { for &c in row { hist[c as usize & 0xF] += 1; } }
    let bg = hist.iter().enumerate().max_by_key(|(_, &c)| c).map_or(0, |(i, _)| i as u8);

    match op {
        "overlay_or" => {
            // Non-bg wins: last region's non-bg cell overwrites
            let mut out = vec![vec![bg; w]; h];
            for region in regions {
                for r in 0..h {
                    for c in 0..w {
                        if region[r][c] != bg {
                            out[r][c] = region[r][c];
                        }
                    }
                }
            }
            out
        }
        "overlay_and" => {
            // Cell is non-bg only if ALL regions have non-bg there
            let mut out = vec![vec![bg; w]; h];
            for r in 0..h {
                for c in 0..w {
                    if regions.iter().all(|region| region[r][c] != bg) {
                        out[r][c] = regions[0][r][c]; // Use first region's color
                    }
                }
            }
            out
        }
        "overlay_xor" => {
            // Cell is non-bg if EXACTLY ONE region has non-bg there
            let mut out = vec![vec![bg; w]; h];
            for r in 0..h {
                for c in 0..w {
                    let non_bg: Vec<_> = regions.iter()
                        .filter(|region| region[r][c] != bg)
                        .collect();
                    if non_bg.len() == 1 {
                        out[r][c] = non_bg[0][r][c];
                    }
                }
            }
            out
        }
        "select_first" => regions[0].clone(),
        "select_last" => regions.last().unwrap().clone(),
        "majority_vote" => {
            // Each cell gets the most common color across regions
            let mut out = vec![vec![bg; w]; h];
            for r in 0..h {
                for c in 0..w {
                    let mut votes = [0u8; 16];
                    for region in regions {
                        votes[region[r][c] as usize & 0xF] += 1;
                    }
                    out[r][c] = votes.iter().enumerate()
                        .max_by_key(|(_, &v)| v)
                        .map_or(bg, |(i, _)| i as u8);
                }
            }
            out
        }
        "diff_regions" => {
            // Show cells that differ between regions (first two)
            if regions.len() < 2 { return regions[0].clone(); }
            let mut out = vec![vec![bg; w]; h];
            for r in 0..h {
                for c in 0..w {
                    if regions[0][r][c] != regions[1][r][c] {
                        out[r][c] = if regions[1][r][c] != bg {
                            regions[1][r][c]
                        } else {
                            regions[0][r][c]
                        };
                    }
                }
            }
            out
        }
        "overlay_last_wins" => {
            // Last region's non-bg cells override everything
            let mut out = regions[0].clone();
            for region in regions.iter().skip(1) {
                for r in 0..h {
                    for c in 0..w {
                        if region[r][c] != bg {
                            out[r][c] = region[r][c];
                        }
                    }
                }
            }
            out
        }
        "overlay_first_wins" => {
            // First region's non-bg cells take priority, fill rest from later
            let mut out = vec![vec![bg; w]; h];
            for region in regions.iter().rev() {
                for r in 0..h {
                    for c in 0..w {
                        if region[r][c] != bg {
                            out[r][c] = region[r][c];
                        }
                    }
                }
            }
            out
        }
        "mask_first_with_second" => {
            // Use region[0] as content, region[1] as mask (non-bg in mask → keep content)
            if regions.len() < 2 { return regions[0].clone(); }
            let mut out = vec![vec![bg; w]; h];
            for r in 0..h {
                for c in 0..w {
                    if regions[1][r][c] != bg {
                        out[r][c] = regions[0][r][c];
                    }
                }
            }
            out
        }
        "mask_second_with_first" => {
            if regions.len() < 2 { return regions[0].clone(); }
            let mut out = vec![vec![bg; w]; h];
            for r in 0..h {
                for c in 0..w {
                    if regions[0][r][c] != bg {
                        out[r][c] = regions[1][r][c];
                    }
                }
            }
            out
        }
        "subtract" => {
            // First region minus second: keep cells from first only where second is bg
            if regions.len() < 2 { return regions[0].clone(); }
            let mut out = regions[0].clone();
            for r in 0..h {
                for c in 0..w {
                    if regions[1][r][c] != bg {
                        out[r][c] = bg;
                    }
                }
            }
            out
        }
        "count_nonzero" => {
            // Cell value = count of regions where cell is non-bg
            let mut out = vec![vec![0u8; w]; h];
            for r in 0..h {
                for c in 0..w {
                    out[r][c] = regions.iter()
                        .filter(|reg| reg[r][c] != bg)
                        .count() as u8;
                }
            }
            out
        }
        _ => regions[0].clone(),
    }
}

// =============================================================================
// Object-level strategies
// =============================================================================

/// Try to recolor objects based on their size ranking.
fn try_recolor_by_size(
    test_input: &ArcGrid,
    task: &ArcTask,
    verbose: bool,
) -> Option<Vec<Vec<u8>>> {
    // For each training example, find objects and their colors in input/output
    // Check if output recolors objects based on size order

    // Learn: for each size rank (smallest=0, next=1, ...), what color does it become?
    let mut rank_to_color: Option<Vec<u8>> = None;

    for ex in &task.train {
        let input_objects = find_objects(&ex.input);
        let output_objects = find_objects(&ex.output);

        if input_objects.len() != output_objects.len() || input_objects.len() < 2 {
            return None;
        }

        // Sort input objects by size
        let mut sorted: Vec<(usize, &Vec<(usize, usize)>)> = input_objects.iter()
            .enumerate().collect();
        sorted.sort_by_key(|(_, obj)| obj.len());

        // For each object, find its output color (the dominant non-bg color at its positions)
        let mut hist_out = [0usize; 16];
        for row in &ex.output { for &c in row { hist_out[c as usize & 0xF] += 1; } }
        let bg = hist_out.iter().enumerate().max_by_key(|(_, &c)| c).map_or(0, |(i, _)| i as u8);

        let mut this_rank_colors = Vec::new();
        for (_, obj) in &sorted {
            let mut color_hist = [0usize; 16];
            for &(r, c) in *obj {
                if r < ex.output.len() && c < ex.output[r].len() {
                    color_hist[ex.output[r][c] as usize & 0xF] += 1;
                }
            }
            let out_color = color_hist.iter().enumerate()
                .filter(|&(i, _)| i as u8 != bg)
                .max_by_key(|(_, &c)| c)
                .map_or(bg, |(i, _)| i as u8);
            this_rank_colors.push(out_color);
        }

        match &rank_to_color {
            None => rank_to_color = Some(this_rank_colors),
            Some(existing) => {
                if *existing != this_rank_colors { return None; }
            }
        }
    }

    let rank_colors = rank_to_color?;

    // Apply to test input
    let test_objects = find_objects(&test_input.cells);
    if test_objects.len() != rank_colors.len() { return None; }

    let mut sorted: Vec<(usize, &Vec<(usize, usize)>)> = test_objects.iter()
        .enumerate().collect();
    sorted.sort_by_key(|(_, obj)| obj.len());

    let mut output = test_input.cells.clone();
    for (rank, (_, obj)) in sorted.iter().enumerate() {
        for &(r, c) in *obj {
            if r < output.len() && c < output[r].len() {
                output[r][c] = rank_colors[rank];
            }
        }
    }

    if verbose {
        println!("  [VORTEX] VERIFIED recolor_by_size: {:?}", rank_colors);
    }
    Some(output)
}

/// Fill enclosed regions per-object with the enclosing object's color.
fn try_fill_enclosed_per_object(
    test_input: &ArcGrid,
    task: &ArcTask,
    verbose: bool,
) -> Option<Vec<Vec<u8>>> {
    // Strategy: for each enclosed region (bg cells surrounded by non-bg),
    // fill with the most common adjacent non-bg color.
    let fill_fn = |grid: &[Vec<u8>]| -> Vec<Vec<u8>> {
        let h = grid.len();
        let w = grid.first().map_or(0, |r| r.len());
        if h == 0 || w == 0 { return grid.to_vec(); }

        let mut hist = [0usize; 16];
        for row in grid { for &c in row { hist[c as usize & 0xF] += 1; } }
        let bg = hist.iter().enumerate().max_by_key(|(_, &c)| c).map_or(0, |(i, _)| i as u8);

        // Flood fill from edges to find exterior
        let mut exterior = vec![vec![false; w]; h];
        let mut stack: Vec<(usize, usize)> = Vec::new();
        for c in 0..w {
            if grid[0][c] == bg { stack.push((0, c)); }
            if grid[h-1][c] == bg { stack.push((h-1, c)); }
        }
        for r in 1..h.saturating_sub(1) {
            if grid[r][0] == bg { stack.push((r, 0)); }
            if grid[r][w-1] == bg { stack.push((r, w-1)); }
        }
        while let Some((r, c)) = stack.pop() {
            if r >= h || c >= w || exterior[r][c] || grid[r][c] != bg { continue; }
            exterior[r][c] = true;
            if r > 0 { stack.push((r-1, c)); }
            if c > 0 { stack.push((r, c-1)); }
            stack.push((r+1, c));
            stack.push((r, c+1));
        }

        // For each interior bg cell, find nearest non-bg color
        let mut out = grid.to_vec();
        for r in 0..h {
            for c in 0..w {
                if grid[r][c] == bg && !exterior[r][c] {
                    // Find adjacent non-bg colors
                    let mut adj_colors = [0u16; 16];
                    for dr in -1i32..=1 {
                        for dc in -1i32..=1 {
                            let nr = r as i32 + dr;
                            let nc = c as i32 + dc;
                            if nr >= 0 && nr < h as i32 && nc >= 0 && nc < w as i32 {
                                let v = grid[nr as usize][nc as usize];
                                if v != bg { adj_colors[v as usize & 0xF] += 1; }
                            }
                        }
                    }
                    let fill = adj_colors.iter().enumerate()
                        .max_by_key(|(_, &c)| c)
                        .filter(|(_, &c)| c > 0)
                        .map_or(bg, |(i, _)| i as u8);
                    out[r][c] = fill;
                }
            }
        }
        out
    };

    let all_match = task.train.iter().all(|ex| fill_fn(&ex.input) == ex.output);
    if all_match {
        if verbose { println!("  [VORTEX] VERIFIED fill_enclosed_per_object"); }
        return Some(fill_fn(&test_input.cells));
    }
    None
}

/// Keep only objects of a specific color, remove everything else.
fn try_keep_color_objects(
    test_input: &ArcGrid,
    task: &ArcTask,
    verbose: bool,
) -> Option<Vec<Vec<u8>>> {
    // For each possible "keep" color, check if output equals input with only that color kept
    for keep_color in 1u8..=9 {
        let filter_fn = |grid: &[Vec<u8>]| -> Vec<Vec<u8>> {
            let mut hist = [0usize; 16];
            for row in grid { for &c in row { hist[c as usize & 0xF] += 1; } }
            let bg = hist.iter().enumerate().max_by_key(|(_, &c)| c).map_or(0, |(i, _)| i as u8);

            grid.iter().map(|row| {
                row.iter().map(|&c| if c == keep_color { c } else { bg }).collect()
            }).collect()
        };

        let all_match = task.train.iter().all(|ex| {
            ex.input.len() == ex.output.len() && filter_fn(&ex.input) == ex.output
        });
        if all_match {
            if verbose { println!("  [VORTEX] VERIFIED keep_color({})", keep_color); }
            return Some(filter_fn(&test_input.cells));
        }
    }
    None
}

/// Replace each object with its bounding box filled with the object's color.
fn try_fill_object_bbox(
    test_input: &ArcGrid,
    task: &ArcTask,
    verbose: bool,
) -> Option<Vec<Vec<u8>>> {
    let fill_fn = |grid: &[Vec<u8>]| -> Vec<Vec<u8>> {
        let mut out = grid.to_vec();
        let objects = find_objects(grid);
        for obj in &objects {
            if obj.is_empty() { continue; }
            let min_r = obj.iter().map(|&(r, _)| r).min().unwrap();
            let max_r = obj.iter().map(|&(r, _)| r).max().unwrap();
            let min_c = obj.iter().map(|&(_, c)| c).min().unwrap();
            let max_c = obj.iter().map(|&(_, c)| c).max().unwrap();
            // Dominant color of this object
            let mut hist = [0usize; 16];
            for &(r, c) in obj { hist[grid[r][c] as usize & 0xF] += 1; }
            let color = hist.iter().enumerate().max_by_key(|(_, &c)| c).map_or(0, |(i, _)| i as u8);
            for r in min_r..=max_r {
                for c in min_c..=max_c {
                    out[r][c] = color;
                }
            }
        }
        out
    };

    let all_match = task.train.iter().all(|ex| {
        ex.input.len() == ex.output.len() && fill_fn(&ex.input) == ex.output
    });
    if all_match {
        if verbose { println!("  [VORTEX] VERIFIED fill_object_bbox"); }
        return Some(fill_fn(&test_input.cells));
    }
    None
}

fn get_neighbours(grid: &Vec<Vec<u8>>, r: usize, c: usize) -> Vec<u8> {
    let h = grid.len();
    let w = grid.first().map_or(0, |row| row.len());
    let mut n = Vec::with_capacity(4);
    if r > 0 { n.push(grid[r-1][c]); } else { n.push(255); }
    if r + 1 < h { n.push(grid[r+1][c]); } else { n.push(255); }
    if c > 0 { n.push(grid[r][c-1]); } else { n.push(255); }
    if c + 1 < w { n.push(grid[r][c+1]); } else { n.push(255); }
    n
}

// =============================================================================
// Utility
// =============================================================================

fn make_grid(cells: &Vec<Vec<u8>>) -> ArcGrid {
    let height = cells.len();
    let width = cells.first().map_or(0, |r| r.len());
    ArcGrid {
        cells: cells.clone(),
        height,
        width,
    }
}

fn cosine_sim(a: &[f32], b: &[f32]) -> f32 {
    let mut dot = 0.0f32;
    let mut na = 0.0f32;
    let mut nb = 0.0f32;
    for i in 0..a.len().min(b.len()) {
        dot += a[i] * b[i];
        na += a[i] * a[i];
        nb += b[i] * b[i];
    }
    let denom = na.sqrt() * nb.sqrt();
    if denom < 1e-9 { 0.0 } else { dot / denom }
}

// =============================================================================
// Data Loading
// =============================================================================

fn load_arc_tasks(data_dir: &str, split: &str) -> Result<Vec<ArcTask>> {
    // Try arc-agi-3 first, then arc-agi-2, then arc v1
    let paths_to_try = vec![
        format!("{}/arc-agi-3/arc_agi_3_evaluation.json", data_dir),
        format!("{}/arc-agi-2/arc_agi_training.json", data_dir),
        format!("{}/arc/ARC-V1-Feb2018-2/ARC-Challenge/{}.jsonl", data_dir, split),
    ];

    for path in &paths_to_try {
        if Path::new(path).exists() {
            println!("[DATA] Loading from: {}", path);
            let content = fs::read_to_string(path)?;

            if path.ends_with(".jsonl") {
                // JSONL format (ARC v1)
                let mut tasks = Vec::new();
                for line in content.lines() {
                    let v: serde_json::Value = serde_json::from_str(line)?;
                    if let Some(id) = v.get("id").and_then(|v| v.as_str()) {
                        if let Some(task) = parse_arc_task(id, &v) {
                            tasks.push(task);
                        }
                    }
                }
                return Ok(tasks);
            }

            // JSON dict format (ARC-AGI)
            let data: HashMap<String, serde_json::Value> = serde_json::from_str(&content)?;
            let mut tasks: Vec<ArcTask> = data.into_iter()
                .filter_map(|(id, v)| parse_arc_task(&id, &v))
                .collect();
            tasks.sort_by(|a, b| a.id.cmp(&b.id));
            return Ok(tasks);
        }
    }

    Err(anyhow::anyhow!(
        "No ARC data found. Tried: {:?}",
        paths_to_try
    ))
}

fn parse_arc_task(id: &str, value: &serde_json::Value) -> Option<ArcTask> {
    let train = value.get("train").and_then(|v| v.as_array())?;
    let test = value.get("test").and_then(|v| v.as_array())?;

    let parse_examples = |arr: &Vec<serde_json::Value>| -> Vec<ArcExample> {
        arr.iter().filter_map(|ex| {
            let input = ex.get("input").and_then(|v| v.as_array())?;
            let output = ex.get("output").and_then(|v| v.as_array())?;
            let input_grid: Vec<Vec<u8>> = input.iter().map(|row| {
                row.as_array().map(|r| {
                    r.iter().map(|v| v.as_u64().unwrap_or(0) as u8).collect()
                }).unwrap_or_default()
            }).collect();
            let output_grid: Vec<Vec<u8>> = output.iter().map(|row| {
                row.as_array().map(|r| {
                    r.iter().map(|v| v.as_u64().unwrap_or(0) as u8).collect()
                }).unwrap_or_default()
            }).collect();
            Some(ArcExample { input: input_grid, output: output_grid })
        }).collect()
    };

    Some(ArcTask {
        id: id.to_string(),
        train: parse_examples(train),
        test: parse_examples(test),
    })
}

// =============================================================================
// Results
// =============================================================================

#[derive(Serialize)]
struct EvalResult {
    timestamp: String,
    dataset: String,
    total_tasks: usize,
    correct_tasks: usize,
    correct_tests: usize,
    total_tests: usize,
    task_accuracy: f64,
    test_accuracy: f64,
    time_secs: f64,
    per_task: Vec<TaskScore>,
}

#[derive(Serialize)]
struct TaskScore {
    task_id: String,
    correct: bool,
    tests_correct: usize,
    tests_total: usize,
    transform_type: String,
}

// =============================================================================
// Main
// =============================================================================

fn main() -> Result<()> {
    let args = Args::parse();
    let start = Instant::now();

    println!("+===============================================================+");
    println!("|         ARC-AGI VORTEX EVALUATION                             |");
    println!("|         GridEncoder + CausalModel + SymbolResolver            |");
    println!("+===============================================================+");
    println!("| Data dir:  {}", args.data_dir);
    println!("| Split:     {}", args.split);
    println!("| Limit:     {}", if args.limit == 0 { "all".to_string() } else { args.limit.to_string() });
    println!("+===============================================================+\n");

    // Load tasks
    let all_tasks = load_arc_tasks(&args.data_dir, &args.split)?;
    let tasks = if args.limit > 0 {
        &all_tasks[..args.limit.min(all_tasks.len())]
    } else {
        &all_tasks
    };
    println!("[LOADED] {} tasks ({} available)\n", tasks.len(), all_tasks.len());

    // Run solver
    let mut solver = VortexArcSolver::new();
    let mut correct_tasks = 0usize;
    let mut correct_tests = 0usize;
    let mut total_tests = 0usize;
    let mut per_task = Vec::new();

    for (i, task) in tasks.iter().enumerate() {
        let task_start = Instant::now();
        let predictions = solver.solve_task(task, args.verbose);

        let mut task_correct = 0;
        let task_total = task.test.len();

        for (j, pred) in predictions.iter().enumerate() {
            total_tests += 1;
            if j < task.test.len() && *pred == task.test[j].output {
                correct_tests += 1;
                task_correct += 1;
            }
        }

        let all_correct = task_correct == task_total && task_total > 0;
        if all_correct { correct_tasks += 1; }

        let task_time = task_start.elapsed().as_secs_f64();
        let status = if all_correct { "OK" } else { "FAIL" };

        if args.verbose || i % 50 == 0 || all_correct {
            println!(
                "[{:4}/{:4}] {} {} ({}/{} tests, {:.1}ms)",
                i + 1, tasks.len(), status, task.id,
                task_correct, task_total, task_time * 1000.0,
            );
        }

        per_task.push(TaskScore {
            task_id: task.id.clone(),
            correct: all_correct,
            tests_correct: task_correct,
            tests_total: task_total,
            transform_type: format!("{:?}", solver.induce_rule(task, &[]).transform_type),
        });
    }

    let elapsed = start.elapsed().as_secs_f64();
    let task_acc = if tasks.is_empty() { 0.0 } else { correct_tasks as f64 / tasks.len() as f64 * 100.0 };
    let test_acc = if total_tests == 0 { 0.0 } else { correct_tests as f64 / total_tests as f64 * 100.0 };

    // Print summary
    println!("\n+===============================================================+");
    println!("|  ARC-AGI RESULTS                                              |");
    println!("+===============================================================+");
    println!("| Tasks solved:  {}/{} ({:.1}%)", correct_tasks, tasks.len(), task_acc);
    println!("| Tests correct: {}/{} ({:.1}%)", correct_tests, total_tests, test_acc);
    println!("| Time:          {:.1}s ({:.1}ms/task)", elapsed, elapsed * 1000.0 / tasks.len().max(1) as f64);
    println!("+===============================================================+");

    // Break down by transform type
    let mut by_type: HashMap<String, (usize, usize)> = HashMap::new();
    for t in &per_task {
        let entry = by_type.entry(t.transform_type.clone()).or_insert((0, 0));
        entry.1 += 1;
        if t.correct { entry.0 += 1; }
    }
    println!("\nBy transform type:");
    let mut type_vec: Vec<_> = by_type.into_iter().collect();
    type_vec.sort_by_key(|(_, (c, _))| std::cmp::Reverse(*c));
    for (ttype, (correct, total)) in &type_vec {
        let pct = if *total > 0 { *correct as f64 / *total as f64 * 100.0 } else { 0.0 };
        println!("  {:20} {}/{} ({:.1}%)", ttype, correct, total, pct);
    }

    // Save results
    let result = EvalResult {
        timestamp: chrono::Utc::now().to_rfc3339(),
        dataset: args.split.clone(),
        total_tasks: tasks.len(),
        correct_tasks,
        correct_tests,
        total_tests,
        task_accuracy: task_acc,
        test_accuracy: test_acc,
        time_secs: elapsed,
        per_task,
    };

    let json = serde_json::to_string_pretty(&result)?;
    fs::write(&args.output, &json)?;
    println!("\n[SAVED] Results to {}", args.output);

    Ok(())
}
