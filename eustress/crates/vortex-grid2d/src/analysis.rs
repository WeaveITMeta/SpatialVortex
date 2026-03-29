//! Grid analysis — property detection for the CausalGraph.
//!
//! Detects structural properties of Grid2D instances:
//! symmetry, color distributions, object detection, dimensions, etc.
//! These properties feed into hypothesis generation.

use crate::grid::Grid2D;
use eustress_vortex_core::{Domain, Property, PropertyValue};

/// Analyzes Grid2D instances to extract properties for the CausalGraph.
pub struct GridAnalyzer;

impl GridAnalyzer {
    /// Full analysis of a grid — returns all detected properties.
    pub fn analyze(grid: &Grid2D) -> Vec<Property> {
        let mut props = Vec::new();

        // Dimensions
        props.push(prop("grid_height", PropertyValue::Int(grid.height as i64)));
        props.push(prop("grid_width", PropertyValue::Int(grid.width as i64)));
        props.push(prop("grid_square", PropertyValue::Bool(grid.height == grid.width)));

        // Colors
        let hist = grid.color_histogram();
        let distinct = grid.distinct_colors();
        let bg = grid.background_color();
        props.push(prop("distinct_colors", PropertyValue::Int(distinct as i64)));
        props.push(prop("background_color", PropertyValue::Int(bg as i64)));

        // Color distribution
        let total_cells = (grid.height * grid.width) as f64;
        if total_cells > 0.0 {
            let bg_ratio = hist[bg as usize] as f64 / total_cells;
            props.push(prop("background_ratio", PropertyValue::Float(bg_ratio)));
            props.push(prop("sparse_grid", PropertyValue::Bool(bg_ratio > 0.7)));
            props.push(prop("dense_grid", PropertyValue::Bool(bg_ratio < 0.3)));
        }

        // Symmetry
        let h_sym = Self::horizontal_symmetry(grid);
        let v_sym = Self::vertical_symmetry(grid);
        let rot_sym = Self::rotational_symmetry_180(grid);
        props.push(prop("symmetric_horizontal", PropertyValue::Bool(h_sym > 0.95)));
        props.push(prop("symmetric_vertical", PropertyValue::Bool(v_sym > 0.95)));
        props.push(prop("symmetric_rotational_180", PropertyValue::Bool(rot_sym > 0.95)));
        props.push(prop("input_asymmetric", PropertyValue::Bool(h_sym < 0.8 && v_sym < 0.8)));

        // Has unsupported objects (floating non-bg cells with bg below)
        let has_floating = Self::has_floating_objects(grid);
        props.push(prop("has_unsupported_objects", PropertyValue::Bool(has_floating)));

        // Color histogram match property (self — for comparing input vs output)
        let hist_sorted: Vec<usize> = {
            let mut h: Vec<usize> = hist.iter().copied().filter(|&c| c > 0).collect();
            h.sort_unstable();
            h
        };
        props.push(prop("color_histogram_signature", PropertyValue::String(
            format!("{:?}", hist_sorted)
        )));

        // Connected components / object count
        let obj_count = Self::count_objects(grid);
        props.push(prop("object_count", PropertyValue::Int(obj_count as i64)));
        props.push(prop("single_object", PropertyValue::Bool(obj_count == 1)));
        props.push(prop("multi_object", PropertyValue::Bool(obj_count > 1)));

        // Row/column uniformity
        let uniform_rows = grid.cells.iter()
            .filter(|row| row.iter().all(|&c| c == row[0]))
            .count();
        let uniform_cols = (0..grid.width)
            .filter(|&c| {
                let first = grid.cells[0][c];
                (1..grid.height).all(|r| grid.cells[r][c] == first)
            })
            .count();
        props.push(prop("has_uniform_rows", PropertyValue::Bool(uniform_rows > 0)));
        props.push(prop("has_uniform_cols", PropertyValue::Bool(uniform_cols > 0)));

        // Content bounding box
        let (min_r, max_r, min_c, max_c) = Self::content_bbox(grid);
        let content_h = if max_r >= min_r { max_r - min_r + 1 } else { 0 };
        let content_w = if max_c >= min_c { max_c - min_c + 1 } else { 0 };
        props.push(prop("content_height", PropertyValue::Int(content_h as i64)));
        props.push(prop("content_width", PropertyValue::Int(content_w as i64)));
        props.push(prop("content_centered", PropertyValue::Bool(
            min_r > 0 && min_c > 0 && max_r < grid.height - 1 && max_c < grid.width - 1
        )));

        // Has border (all edge cells same color)
        let has_border = Self::has_uniform_border(grid);
        props.push(prop("has_border", PropertyValue::Bool(has_border)));

        // Repeating pattern
        let h_period = Self::horizontal_period(grid);
        let v_period = Self::vertical_period(grid);
        props.push(prop("has_horizontal_period", PropertyValue::Bool(h_period.is_some())));
        props.push(prop("has_vertical_period", PropertyValue::Bool(v_period.is_some())));

        props
    }

    /// Horizontal symmetry score [0.0, 1.0].
    pub fn horizontal_symmetry(grid: &Grid2D) -> f64 {
        if grid.width <= 1 {
            return 1.0;
        }
        let total = grid.height * (grid.width / 2);
        if total == 0 {
            return 1.0;
        }
        let matching: usize = (0..grid.height)
            .map(|r| {
                (0..grid.width / 2)
                    .filter(|&c| grid.cells[r][c] == grid.cells[r][grid.width - 1 - c])
                    .count()
            })
            .sum();
        matching as f64 / total as f64
    }

    /// Vertical symmetry score [0.0, 1.0].
    pub fn vertical_symmetry(grid: &Grid2D) -> f64 {
        if grid.height <= 1 {
            return 1.0;
        }
        let total = (grid.height / 2) * grid.width;
        if total == 0 {
            return 1.0;
        }
        let matching: usize = (0..grid.height / 2)
            .map(|r| {
                (0..grid.width)
                    .filter(|&c| grid.cells[r][c] == grid.cells[grid.height - 1 - r][c])
                    .count()
            })
            .sum();
        matching as f64 / total as f64
    }

    /// 180-degree rotational symmetry score.
    pub fn rotational_symmetry_180(grid: &Grid2D) -> f64 {
        let total = grid.height * grid.width;
        if total == 0 {
            return 1.0;
        }
        let matching: usize = (0..grid.height)
            .map(|r| {
                (0..grid.width)
                    .filter(|&c| {
                        grid.cells[r][c] == grid.cells[grid.height - 1 - r][grid.width - 1 - c]
                    })
                    .count()
            })
            .sum();
        matching as f64 / total as f64
    }

    /// Check if grid has floating objects (non-bg with bg below).
    pub fn has_floating_objects(grid: &Grid2D) -> bool {
        let bg = grid.background_color();
        for r in 0..grid.height.saturating_sub(1) {
            for c in 0..grid.width {
                if grid.cells[r][c] != bg && grid.cells[r + 1][c] == bg {
                    // Check if there's no support below
                    let supported = (r + 1..grid.height).any(|r2| grid.cells[r2][c] != bg);
                    if !supported {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Count connected components of non-background cells.
    pub fn count_objects(grid: &Grid2D) -> usize {
        let bg = grid.background_color();
        let mut visited = vec![vec![false; grid.width]; grid.height];
        let mut count = 0;

        for r in 0..grid.height {
            for c in 0..grid.width {
                if grid.cells[r][c] != bg && !visited[r][c] {
                    count += 1;
                    // BFS flood
                    let mut stack = vec![(r, c)];
                    while let Some((sr, sc)) = stack.pop() {
                        if sr >= grid.height || sc >= grid.width || visited[sr][sc] || grid.cells[sr][sc] == bg {
                            continue;
                        }
                        visited[sr][sc] = true;
                        if sr > 0 { stack.push((sr - 1, sc)); }
                        if sc > 0 { stack.push((sr, sc - 1)); }
                        stack.push((sr + 1, sc));
                        stack.push((sr, sc + 1));
                    }
                }
            }
        }
        count
    }

    /// Bounding box of non-background content.
    pub fn content_bbox(grid: &Grid2D) -> (usize, usize, usize, usize) {
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
        (min_r, max_r, min_c, max_c)
    }

    /// Check if all border cells are the same color.
    pub fn has_uniform_border(grid: &Grid2D) -> bool {
        if grid.height < 2 || grid.width < 2 {
            return false;
        }
        let border_color = grid.cells[0][0];

        // Top and bottom rows
        for c in 0..grid.width {
            if grid.cells[0][c] != border_color || grid.cells[grid.height - 1][c] != border_color {
                return false;
            }
        }
        // Left and right columns
        for r in 1..grid.height - 1 {
            if grid.cells[r][0] != border_color || grid.cells[r][grid.width - 1] != border_color {
                return false;
            }
        }
        true
    }

    /// Find horizontal period (smallest repeating unit width).
    pub fn horizontal_period(grid: &Grid2D) -> Option<usize> {
        for period in 1..=grid.width / 2 {
            if grid.width % period != 0 {
                continue;
            }
            let matches = grid.cells.iter().all(|row| {
                (0..grid.width).all(|c| row[c] == row[c % period])
            });
            if matches {
                return Some(period);
            }
        }
        None
    }

    /// Find vertical period (smallest repeating unit height).
    pub fn vertical_period(grid: &Grid2D) -> Option<usize> {
        for period in 1..=grid.height / 2 {
            if grid.height % period != 0 {
                continue;
            }
            let matches = (0..grid.height).all(|r| {
                grid.cells[r] == grid.cells[r % period]
            });
            if matches {
                return Some(period);
            }
        }
        None
    }
}

fn prop(name: &str, value: PropertyValue) -> Property {
    Property {
        name: name.into(),
        domain: Domain::Grid2D,
        value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symmetry_detection() {
        // Horizontally symmetric
        let g = Grid2D::new(vec![
            vec![1, 2, 1],
            vec![3, 4, 3],
            vec![1, 2, 1],
        ]);
        assert!(GridAnalyzer::horizontal_symmetry(&g) > 0.95);
        assert!(GridAnalyzer::vertical_symmetry(&g) > 0.95);
    }

    #[test]
    fn test_object_count() {
        let g = Grid2D::new(vec![
            vec![1, 0, 2],
            vec![0, 0, 0],
            vec![3, 0, 4],
        ]);
        assert_eq!(GridAnalyzer::count_objects(&g), 4);
    }

    #[test]
    fn test_floating_objects() {
        // Cell at (0,1) is floating (nothing below)
        let g = Grid2D::new(vec![
            vec![0, 1, 0],
            vec![0, 0, 0],
            vec![0, 0, 0],
        ]);
        assert!(GridAnalyzer::has_floating_objects(&g));

        // Cell at (2,1) is on ground
        let g2 = Grid2D::new(vec![
            vec![0, 0, 0],
            vec![0, 0, 0],
            vec![0, 1, 0],
        ]);
        assert!(!GridAnalyzer::has_floating_objects(&g2));
    }

    #[test]
    fn test_horizontal_period() {
        let g = Grid2D::new(vec![
            vec![1, 2, 1, 2],
            vec![3, 4, 3, 4],
        ]);
        assert_eq!(GridAnalyzer::horizontal_period(&g), Some(2));
    }

    #[test]
    fn test_analyze_returns_properties() {
        let g = Grid2D::new(vec![
            vec![0, 1, 0],
            vec![1, 0, 1],
            vec![0, 1, 0],
        ]);
        let props = GridAnalyzer::analyze(&g);
        assert!(props.len() > 10, "Should detect many properties");
        assert!(props.iter().any(|p| p.name == "grid_square"));
        assert!(props.iter().any(|p| p.name == "distinct_colors"));
    }
}
