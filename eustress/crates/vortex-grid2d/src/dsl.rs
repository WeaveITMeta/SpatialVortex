//! DSL primitives for Grid2D transformation.
//!
//! ~50 operations covering the core ARC-AGI transformation vocabulary:
//! rotation, reflection, recoloring, cropping, tiling, gravity,
//! object manipulation, flood fill, etc.

use crate::grid::Grid2D;
use eustress_vortex_core::{DSLOp, Domain};

/// Grid DSL — applies named operations to Grid2D.
pub struct GridDSL;

impl GridDSL {
    /// All available DSL operations for a given grid.
    pub fn all_ops(grid: &Grid2D) -> Vec<DSLOp> {
        let mut ops = vec![
            // Rotation
            op("rotate_cw"),
            op("rotate_ccw"),
            op("rotate_180"),
            // Reflection
            op("flip_horizontal"),
            op("flip_vertical"),
            op("flip_diagonal_main"),
            op("flip_diagonal_anti"),
            // Transpose
            op("transpose"),
            // Gravity
            op("gravity_down"),
            op("gravity_up"),
            op("gravity_left"),
            op("gravity_right"),
            // Cropping
            op("crop_to_content"),
            // Tiling
            op("tile_2x2"),
            op("tile_3x3"),
            op("tile_horizontal"),
            op("tile_vertical"),
            // Fill
            op("fill_background"),
            op("invert_colors"),
            // Border
            op("add_border_1"),
            op("remove_border_1"),
            // Scale
            op("scale_2x"),
            op("scale_half"),
            // Sort
            op("sort_rows"),
            op("sort_cols"),
            // Unique
            op("deduplicate_rows"),
            op("deduplicate_cols"),
            // Mask
            op("mask_background"),
            // Shift
            op("shift_right_1"),
            op("shift_left_1"),
            op("shift_down_1"),
            op("shift_up_1"),
            // Wrap
            op("wrap_shift_right"),
            op("wrap_shift_down"),
        ];

        // Color remapping operations (recolor most frequent non-bg to each other color)
        let bg = grid.background_color();
        let hist = grid.color_histogram();
        let mut active_colors: Vec<u8> = (0..16)
            .filter(|&c| hist[c as usize] > 0 && c != bg)
            .collect();
        active_colors.sort_by_key(|&c| std::cmp::Reverse(hist[c as usize]));

        for &src in active_colors.iter().take(4) {
            for &dst in active_colors.iter().take(4) {
                if src != dst {
                    ops.push(DSLOp {
                        name: format!("recolor_{}_{}", src, dst),
                        domain: Domain::Grid2D,
                        parameters: vec![
                            serde_json::Value::from(src),
                            serde_json::Value::from(dst),
                        ],
                    });
                }
            }
        }

        // Per-cell set operations for small grids
        if grid.height <= 8 && grid.width <= 8 {
            for &color in active_colors.iter().take(3) {
                ops.push(DSLOp {
                    name: format!("flood_fill_center_{}", color),
                    domain: Domain::Grid2D,
                    parameters: vec![serde_json::Value::from(color)],
                });
            }
        }

        ops
    }

    /// Apply a named DSL operation to a grid.
    pub fn apply(grid: &Grid2D, action: &DSLOp) -> Grid2D {
        match action.name.as_str() {
            // Rotation
            "rotate_cw" => rotate_cw(grid),
            "rotate_ccw" => rotate_ccw(grid),
            "rotate_180" => rotate_180(grid),

            // Reflection
            "flip_horizontal" => flip_horizontal(grid),
            "flip_vertical" => flip_vertical(grid),
            "flip_diagonal_main" => flip_diagonal_main(grid),
            "flip_diagonal_anti" => flip_diagonal_anti(grid),
            "transpose" => transpose(grid),

            // Gravity
            "gravity_down" => gravity_down(grid),
            "gravity_up" => gravity_up(grid),
            "gravity_left" => gravity_left(grid),
            "gravity_right" => gravity_right(grid),

            // Cropping
            "crop_to_content" => crop_to_content(grid),

            // Tiling
            "tile_2x2" => tile(grid, 2, 2),
            "tile_3x3" => tile(grid, 3, 3),
            "tile_horizontal" => tile(grid, 1, 2),
            "tile_vertical" => tile(grid, 2, 1),

            // Fill
            "fill_background" => fill_background(grid),
            "invert_colors" => invert_colors(grid),

            // Border
            "add_border_1" => add_border(grid, 1),
            "remove_border_1" => remove_border(grid, 1),

            // Scale
            "scale_2x" => scale(grid, 2),
            "scale_half" => scale_half(grid),

            // Sort
            "sort_rows" => sort_rows(grid),
            "sort_cols" => sort_cols(grid),

            // Deduplicate
            "deduplicate_rows" => deduplicate_rows(grid),
            "deduplicate_cols" => deduplicate_cols(grid),

            // Mask
            "mask_background" => mask_background(grid),

            // Shift
            "shift_right_1" => shift(grid, 0, 1),
            "shift_left_1" => shift(grid, 0, -1),
            "shift_down_1" => shift(grid, 1, 0),
            "shift_up_1" => shift(grid, -1, 0),

            // Wrap
            "wrap_shift_right" => wrap_shift(grid, 0, 1),
            "wrap_shift_down" => wrap_shift(grid, 1, 0),

            // Recolor
            s if s.starts_with("recolor_") => {
                let parts: Vec<&str> = s.split('_').collect();
                if parts.len() == 3 {
                    let src = parts[1].parse::<u8>().unwrap_or(0);
                    let dst = parts[2].parse::<u8>().unwrap_or(0);
                    recolor(grid, src, dst)
                } else {
                    grid.clone()
                }
            }

            // Flood fill center
            s if s.starts_with("flood_fill_center_") => {
                let color = s.strip_prefix("flood_fill_center_")
                    .and_then(|c| c.parse::<u8>().ok())
                    .unwrap_or(0);
                flood_fill(grid, grid.height / 2, grid.width / 2, color)
            }

            _ => grid.clone(),
        }
    }
}

fn op(name: &str) -> DSLOp {
    DSLOp { name: name.into(), domain: Domain::Grid2D, parameters: vec![] }
}

// ─── Rotation ────────────────────────────────────────────────────────────

fn rotate_cw(grid: &Grid2D) -> Grid2D {
    let h = grid.height;
    let w = grid.width;
    let mut cells = vec![vec![0u8; h]; w];
    for r in 0..h {
        for c in 0..w {
            cells[c][h - 1 - r] = grid.cells[r][c];
        }
    }
    Grid2D::new(cells)
}

fn rotate_ccw(grid: &Grid2D) -> Grid2D {
    let h = grid.height;
    let w = grid.width;
    let mut cells = vec![vec![0u8; h]; w];
    for r in 0..h {
        for c in 0..w {
            cells[w - 1 - c][r] = grid.cells[r][c];
        }
    }
    Grid2D::new(cells)
}

fn rotate_180(grid: &Grid2D) -> Grid2D {
    let h = grid.height;
    let w = grid.width;
    let mut cells = vec![vec![0u8; w]; h];
    for r in 0..h {
        for c in 0..w {
            cells[h - 1 - r][w - 1 - c] = grid.cells[r][c];
        }
    }
    Grid2D::new(cells)
}

// ─── Reflection ──────────────────────────────────────────────────────────

fn flip_horizontal(grid: &Grid2D) -> Grid2D {
    let cells: Vec<Vec<u8>> = grid.cells.iter()
        .map(|row| row.iter().rev().copied().collect())
        .collect();
    Grid2D::new(cells)
}

fn flip_vertical(grid: &Grid2D) -> Grid2D {
    let cells: Vec<Vec<u8>> = grid.cells.iter().rev().cloned().collect();
    Grid2D::new(cells)
}

fn flip_diagonal_main(grid: &Grid2D) -> Grid2D {
    transpose(grid)
}

fn flip_diagonal_anti(grid: &Grid2D) -> Grid2D {
    rotate_cw(&flip_horizontal(grid))
}

fn transpose(grid: &Grid2D) -> Grid2D {
    let h = grid.height;
    let w = grid.width;
    let mut cells = vec![vec![0u8; h]; w];
    for r in 0..h {
        for c in 0..w {
            cells[c][r] = grid.cells[r][c];
        }
    }
    Grid2D::new(cells)
}

// ─── Gravity ─────────────────────────────────────────────────────────────

fn gravity_down(grid: &Grid2D) -> Grid2D {
    let bg = grid.background_color();
    let mut cells = vec![vec![bg; grid.width]; grid.height];

    for c in 0..grid.width {
        let non_bg: Vec<u8> = (0..grid.height)
            .map(|r| grid.cells[r][c])
            .filter(|&v| v != bg)
            .collect();
        let start = grid.height - non_bg.len();
        for (i, &v) in non_bg.iter().enumerate() {
            cells[start + i][c] = v;
        }
    }
    Grid2D::new(cells)
}

fn gravity_up(grid: &Grid2D) -> Grid2D {
    let bg = grid.background_color();
    let mut cells = vec![vec![bg; grid.width]; grid.height];

    for c in 0..grid.width {
        let non_bg: Vec<u8> = (0..grid.height)
            .map(|r| grid.cells[r][c])
            .filter(|&v| v != bg)
            .collect();
        for (i, &v) in non_bg.iter().enumerate() {
            cells[i][c] = v;
        }
    }
    Grid2D::new(cells)
}

fn gravity_left(grid: &Grid2D) -> Grid2D {
    let bg = grid.background_color();
    let mut cells = vec![vec![bg; grid.width]; grid.height];

    for r in 0..grid.height {
        let non_bg: Vec<u8> = grid.cells[r].iter()
            .filter(|&&v| v != bg)
            .copied()
            .collect();
        for (i, &v) in non_bg.iter().enumerate() {
            cells[r][i] = v;
        }
    }
    Grid2D::new(cells)
}

fn gravity_right(grid: &Grid2D) -> Grid2D {
    let bg = grid.background_color();
    let mut cells = vec![vec![bg; grid.width]; grid.height];

    for r in 0..grid.height {
        let non_bg: Vec<u8> = grid.cells[r].iter()
            .filter(|&&v| v != bg)
            .copied()
            .collect();
        let start = grid.width - non_bg.len();
        for (i, &v) in non_bg.iter().enumerate() {
            cells[r][start + i] = v;
        }
    }
    Grid2D::new(cells)
}

// ─── Cropping ────────────────────────────────────────────────────────────

fn crop_to_content(grid: &Grid2D) -> Grid2D {
    let bg = grid.background_color();
    let mut min_r = grid.height;
    let mut max_r = 0;
    let mut min_c = grid.width;
    let mut max_c = 0;

    for r in 0..grid.height {
        for c in 0..grid.width {
            if grid.cells[r][c] != bg {
                min_r = min_r.min(r);
                max_r = max_r.max(r);
                min_c = min_c.min(c);
                max_c = max_c.max(c);
            }
        }
    }

    if min_r > max_r || min_c > max_c {
        return grid.clone(); // All background
    }

    let cells: Vec<Vec<u8>> = (min_r..=max_r)
        .map(|r| grid.cells[r][min_c..=max_c].to_vec())
        .collect();
    Grid2D::new(cells)
}

// ─── Tiling ──────────────────────────────────────────────────────────────

fn tile(grid: &Grid2D, rows: usize, cols: usize) -> Grid2D {
    let h = grid.height * rows;
    let w = grid.width * cols;
    let mut cells = vec![vec![0u8; w]; h];

    for r in 0..h {
        for c in 0..w {
            cells[r][c] = grid.cells[r % grid.height][c % grid.width];
        }
    }
    Grid2D::new(cells)
}

// ─── Fill ────────────────────────────────────────────────────────────────

fn fill_background(grid: &Grid2D) -> Grid2D {
    let bg = grid.background_color();
    let cells: Vec<Vec<u8>> = grid.cells.iter()
        .map(|row| row.iter().map(|&c| if c == bg { 0 } else { c }).collect())
        .collect();
    Grid2D::new(cells)
}

fn invert_colors(grid: &Grid2D) -> Grid2D {
    let max_color = grid.cells.iter()
        .flat_map(|r| r.iter())
        .copied()
        .max()
        .unwrap_or(9);
    let cells: Vec<Vec<u8>> = grid.cells.iter()
        .map(|row| row.iter().map(|&c| max_color - c).collect())
        .collect();
    Grid2D::new(cells)
}

// ─── Border ──────────────────────────────────────────────────────────────

fn add_border(grid: &Grid2D, size: usize) -> Grid2D {
    let h = grid.height + 2 * size;
    let w = grid.width + 2 * size;
    let bg = grid.background_color();
    let mut cells = vec![vec![bg; w]; h];

    for r in 0..grid.height {
        for c in 0..grid.width {
            cells[r + size][c + size] = grid.cells[r][c];
        }
    }
    Grid2D::new(cells)
}

fn remove_border(grid: &Grid2D, size: usize) -> Grid2D {
    if grid.height <= 2 * size || grid.width <= 2 * size {
        return grid.clone();
    }
    let cells: Vec<Vec<u8>> = (size..grid.height - size)
        .map(|r| grid.cells[r][size..grid.width - size].to_vec())
        .collect();
    Grid2D::new(cells)
}

// ─── Scale ───────────────────────────────────────────────────────────────

fn scale(grid: &Grid2D, factor: usize) -> Grid2D {
    let h = grid.height * factor;
    let w = grid.width * factor;
    let mut cells = vec![vec![0u8; w]; h];

    for r in 0..h {
        for c in 0..w {
            cells[r][c] = grid.cells[r / factor][c / factor];
        }
    }
    Grid2D::new(cells)
}

fn scale_half(grid: &Grid2D) -> Grid2D {
    if grid.height < 2 || grid.width < 2 {
        return grid.clone();
    }
    let h = grid.height / 2;
    let w = grid.width / 2;
    let mut cells = vec![vec![0u8; w]; h];

    for r in 0..h {
        for c in 0..w {
            // Majority vote of 2x2 block
            let mut counts = [0u8; 16];
            for dr in 0..2 {
                for dc in 0..2 {
                    let v = grid.cells[r * 2 + dr][c * 2 + dc];
                    counts[v as usize & 0xF] += 1;
                }
            }
            cells[r][c] = counts.iter().enumerate()
                .max_by_key(|&(_, count)| count)
                .map_or(0, |(color, _)| color as u8);
        }
    }
    Grid2D::new(cells)
}

// ─── Sort ────────────────────────────────────────────────────────────────

fn sort_rows(grid: &Grid2D) -> Grid2D {
    let mut cells = grid.cells.clone();
    for row in &mut cells {
        row.sort();
    }
    Grid2D::new(cells)
}

fn sort_cols(grid: &Grid2D) -> Grid2D {
    let mut cells = grid.cells.clone();
    for c in 0..grid.width {
        let mut col: Vec<u8> = (0..grid.height).map(|r| cells[r][c]).collect();
        col.sort();
        for (r, &v) in col.iter().enumerate() {
            cells[r][c] = v;
        }
    }
    Grid2D::new(cells)
}

// ─── Deduplicate ─────────────────────────────────────────────────────────

fn deduplicate_rows(grid: &Grid2D) -> Grid2D {
    let mut seen = Vec::new();
    let mut cells = Vec::new();
    for row in &grid.cells {
        if !seen.contains(row) {
            seen.push(row.clone());
            cells.push(row.clone());
        }
    }
    Grid2D::new(cells)
}

fn deduplicate_cols(grid: &Grid2D) -> Grid2D {
    let t = transpose(grid);
    let deduped = deduplicate_rows(&t);
    transpose(&deduped)
}

// ─── Mask ────────────────────────────────────────────────────────────────

fn mask_background(grid: &Grid2D) -> Grid2D {
    let bg = grid.background_color();
    let cells: Vec<Vec<u8>> = grid.cells.iter()
        .map(|row| row.iter().map(|&c| if c == bg { 0 } else { 1 }).collect())
        .collect();
    Grid2D::new(cells)
}

// ─── Recolor ─────────────────────────────────────────────────────────────

fn recolor(grid: &Grid2D, src: u8, dst: u8) -> Grid2D {
    let cells: Vec<Vec<u8>> = grid.cells.iter()
        .map(|row| row.iter().map(|&c| if c == src { dst } else { c }).collect())
        .collect();
    Grid2D::new(cells)
}

// ─── Shift ───────────────────────────────────────────────────────────────

fn shift(grid: &Grid2D, dr: i32, dc: i32) -> Grid2D {
    let bg = grid.background_color();
    let h = grid.height as i32;
    let w = grid.width as i32;
    let mut cells = vec![vec![bg; grid.width]; grid.height];

    for r in 0..grid.height {
        for c in 0..grid.width {
            let nr = r as i32 + dr;
            let nc = c as i32 + dc;
            if nr >= 0 && nr < h && nc >= 0 && nc < w {
                cells[nr as usize][nc as usize] = grid.cells[r][c];
            }
        }
    }
    Grid2D::new(cells)
}

fn wrap_shift(grid: &Grid2D, dr: i32, dc: i32) -> Grid2D {
    let h = grid.height;
    let w = grid.width;
    let mut cells = vec![vec![0u8; w]; h];

    for r in 0..h {
        for c in 0..w {
            let nr = ((r as i32 + dr).rem_euclid(h as i32)) as usize;
            let nc = ((c as i32 + dc).rem_euclid(w as i32)) as usize;
            cells[nr][nc] = grid.cells[r][c];
        }
    }
    Grid2D::new(cells)
}

// ─── Flood Fill ──────────────────────────────────────────────────────────

fn flood_fill(grid: &Grid2D, start_r: usize, start_c: usize, new_color: u8) -> Grid2D {
    let mut cells = grid.cells.clone();
    let old_color = grid.get(start_r, start_c).unwrap_or(0);
    if old_color == new_color {
        return Grid2D::new(cells);
    }

    let mut stack = vec![(start_r, start_c)];
    while let Some((r, c)) = stack.pop() {
        if r >= grid.height || c >= grid.width || cells[r][c] != old_color {
            continue;
        }
        cells[r][c] = new_color;
        if r > 0 { stack.push((r - 1, c)); }
        if c > 0 { stack.push((r, c - 1)); }
        stack.push((r + 1, c));
        stack.push((r, c + 1));
    }
    Grid2D::new(cells)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_grid() -> Grid2D {
        Grid2D::new(vec![
            vec![1, 2, 3],
            vec![4, 5, 6],
            vec![7, 8, 9],
        ])
    }

    #[test]
    fn test_rotate_cw() {
        let g = test_grid();
        let r = rotate_cw(&g);
        assert_eq!(r.cells, vec![
            vec![7, 4, 1],
            vec![8, 5, 2],
            vec![9, 6, 3],
        ]);
    }

    #[test]
    fn test_rotate_180() {
        let g = test_grid();
        let r = rotate_180(&g);
        assert_eq!(r.cells, vec![
            vec![9, 8, 7],
            vec![6, 5, 4],
            vec![3, 2, 1],
        ]);
    }

    #[test]
    fn test_flip_horizontal() {
        let g = test_grid();
        let r = flip_horizontal(&g);
        assert_eq!(r.cells, vec![
            vec![3, 2, 1],
            vec![6, 5, 4],
            vec![9, 8, 7],
        ]);
    }

    #[test]
    fn test_gravity_down() {
        let g = Grid2D::new(vec![
            vec![1, 0, 0],
            vec![0, 0, 2],
            vec![0, 3, 0],
        ]);
        let r = gravity_down(&g);
        assert_eq!(r.cells, vec![
            vec![0, 0, 0],
            vec![0, 0, 0],
            vec![1, 3, 2],
        ]);
    }

    #[test]
    fn test_crop_to_content() {
        let g = Grid2D::new(vec![
            vec![0, 0, 0, 0],
            vec![0, 1, 2, 0],
            vec![0, 3, 4, 0],
            vec![0, 0, 0, 0],
        ]);
        let r = crop_to_content(&g);
        assert_eq!(r.cells, vec![
            vec![1, 2],
            vec![3, 4],
        ]);
    }

    #[test]
    fn test_tile_2x2() {
        let g = Grid2D::new(vec![vec![1, 2], vec![3, 4]]);
        let r = tile(&g, 2, 2);
        assert_eq!(r.height, 4);
        assert_eq!(r.width, 4);
        assert_eq!(r.cells[2][2], 1);
    }

    #[test]
    fn test_recolor() {
        let g = Grid2D::new(vec![vec![1, 2, 1], vec![2, 1, 2]]);
        let r = recolor(&g, 1, 5);
        assert_eq!(r.cells, vec![vec![5, 2, 5], vec![2, 5, 2]]);
    }

    #[test]
    fn test_dsl_apply() {
        let g = test_grid();
        let op = DSLOp {
            name: "rotate_cw".into(),
            domain: Domain::Grid2D,
            parameters: vec![],
        };
        let r = GridDSL::apply(&g, &op);
        assert_eq!(r.cells[0][0], 7);
    }
}
