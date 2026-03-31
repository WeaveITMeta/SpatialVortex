//! Object extraction from Grid2D via connected-component analysis.
//!
//! Parses a 2D color grid into discrete objects (connected components of
//! same-color cells), each with bounding box, centroid, area, and cell set.
//! This is the foundation for object-level perception in ARC-AGI-3.

use crate::Grid2D;

// ─── Bounding Box ────────────────────────────────────────────────────────────

/// Axis-aligned bounding box in grid coordinates.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BBox {
    pub min_row: usize,
    pub min_col: usize,
    pub max_row: usize,
    pub max_col: usize,
}

impl BBox {
    pub fn width(&self) -> usize {
        self.max_col - self.min_col + 1
    }

    pub fn height(&self) -> usize {
        self.max_row - self.min_row + 1
    }

    pub fn area(&self) -> usize {
        self.width() * self.height()
    }

    pub fn center(&self) -> (f32, f32) {
        (
            (self.min_row + self.max_row) as f32 / 2.0,
            (self.min_col + self.max_col) as f32 / 2.0,
        )
    }

    /// Intersection-over-Union with another bounding box.
    pub fn iou(&self, other: &BBox) -> f32 {
        let inter_min_r = self.min_row.max(other.min_row);
        let inter_max_r = self.max_row.min(other.max_row);
        let inter_min_c = self.min_col.max(other.min_col);
        let inter_max_c = self.max_col.min(other.max_col);

        if inter_min_r > inter_max_r || inter_min_c > inter_max_c {
            return 0.0;
        }

        let inter_area = (inter_max_r - inter_min_r + 1) * (inter_max_c - inter_min_c + 1);
        let union_area = self.area() + other.area() - inter_area;

        if union_area == 0 {
            return 0.0;
        }
        inter_area as f32 / union_area as f32
    }

    /// Manhattan distance between centers.
    pub fn center_distance(&self, other: &BBox) -> f32 {
        let (r1, c1) = self.center();
        let (r2, c2) = other.center();
        (r1 - r2).abs() + (c1 - c2).abs()
    }

    fn from_cells(cells: &[(usize, usize)]) -> Self {
        let mut min_row = usize::MAX;
        let mut max_row = 0;
        let mut min_col = usize::MAX;
        let mut max_col = 0;
        for &(r, c) in cells {
            min_row = min_row.min(r);
            max_row = max_row.max(r);
            min_col = min_col.min(c);
            max_col = max_col.max(c);
        }
        Self { min_row, min_col, max_row, max_col }
    }
}

// ─── Connectivity ────────────────────────────────────────────────────────────

/// Connectivity mode for flood-fill extraction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Connectivity {
    /// 4-connected: up, down, left, right.
    Four,
    /// 8-connected: includes diagonals.
    Eight,
}

// ─── Grid Object ─────────────────────────────────────────────────────────────

/// A single extracted object: a connected component of same-color cells.
#[derive(Clone, Debug)]
pub struct GridObject {
    /// Extraction-local ID (0-based index from this frame's extraction).
    pub local_id: u32,
    /// Color of this object's cells (single-color connected component).
    pub color: u8,
    /// Cells belonging to this object: (row, col).
    pub cells: Vec<(usize, usize)>,
    /// Bounding box.
    pub bbox: BBox,
    /// Centroid (mean row, mean col).
    pub centroid: (f32, f32),
    /// Number of cells.
    pub area: usize,
}

impl GridObject {
    /// Cell overlap ratio with another object (intersection / min_area).
    pub fn cell_overlap(&self, other: &GridObject) -> f32 {
        if self.area == 0 || other.area == 0 {
            return 0.0;
        }
        // For small objects, direct set intersection is fine.
        let count = self.cells.iter()
            .filter(|c| other.cells.contains(c))
            .count();
        count as f32 / self.area.min(other.area) as f32
    }
}

// ─── Object Map ──────────────────────────────────────────────────────────────

/// Result of extracting all objects from a grid.
#[derive(Clone, Debug)]
pub struct ObjectMap {
    /// All non-background objects, sorted by area descending.
    pub objects: Vec<GridObject>,
    /// Detected background color (most common color, usually 0).
    pub background_color: u8,
    /// Per-cell label: `labels[r][c]` = index into `objects`, or `u32::MAX` for background.
    pub labels: Vec<Vec<u32>>,
    /// Grid dimensions (cached).
    pub width: usize,
    pub height: usize,
}

impl ObjectMap {
    /// Find the object at a given cell position.
    pub fn object_at(&self, row: usize, col: usize) -> Option<&GridObject> {
        if row >= self.height || col >= self.width {
            return None;
        }
        let label = self.labels[row][col];
        if label == u32::MAX {
            None
        } else {
            self.objects.get(label as usize)
        }
    }

    /// Number of non-background objects.
    pub fn object_count(&self) -> usize {
        self.objects.len()
    }
}

// ─── Object Extractor ────────────────────────────────────────────────────────

/// Extracts objects (connected components) from a Grid2D.
pub struct ObjectExtractor;

impl ObjectExtractor {
    /// Extract all non-background objects from a grid.
    ///
    /// O(width * height) — single BFS pass. For 64x64 grids: <1ms.
    pub fn extract(grid: &Grid2D, conn: Connectivity) -> ObjectMap {
        let bg = grid.background_color();
        let h = grid.height;
        let w = grid.width;
        let mut visited = vec![vec![false; w]; h];
        let mut raw_objects: Vec<GridObject> = Vec::new();

        for r in 0..h {
            for c in 0..w {
                if visited[r][c] || grid.cells[r][c] == bg {
                    continue;
                }

                let color = grid.cells[r][c];
                let mut cells = Vec::new();
                let mut stack = vec![(r, c)];
                visited[r][c] = true;

                while let Some((sr, sc)) = stack.pop() {
                    cells.push((sr, sc));

                    // 4-connected neighbors
                    let neighbors: &[(isize, isize)] = match conn {
                        Connectivity::Four => &[(-1, 0), (1, 0), (0, -1), (0, 1)],
                        Connectivity::Eight => &[
                            (-1, 0), (1, 0), (0, -1), (0, 1),
                            (-1, -1), (-1, 1), (1, -1), (1, 1),
                        ],
                    };

                    for &(dr, dc) in neighbors {
                        let nr = sr as isize + dr;
                        let nc = sc as isize + dc;
                        if nr < 0 || nc < 0 {
                            continue;
                        }
                        let nr = nr as usize;
                        let nc = nc as usize;
                        if nr < h && nc < w && !visited[nr][nc] && grid.cells[nr][nc] == color {
                            visited[nr][nc] = true;
                            stack.push((nr, nc));
                        }
                    }
                }

                let bbox = BBox::from_cells(&cells);
                let centroid = compute_centroid(&cells);
                let area = cells.len();
                raw_objects.push(GridObject {
                    local_id: 0, // assigned after sorting
                    color,
                    cells,
                    bbox,
                    centroid,
                    area,
                });
            }
        }

        // Sort by area descending (largest objects first).
        raw_objects.sort_by(|a, b| b.area.cmp(&a.area));

        // Assign local_id based on sorted order.
        for (i, obj) in raw_objects.iter_mut().enumerate() {
            obj.local_id = i as u32;
        }

        // Build label grid: map each cell to its object index.
        let mut labels = vec![vec![u32::MAX; w]; h];
        for (idx, obj) in raw_objects.iter().enumerate() {
            for &(r, c) in &obj.cells {
                labels[r][c] = idx as u32;
            }
        }

        ObjectMap {
            objects: raw_objects,
            background_color: bg,
            labels,
            width: w,
            height: h,
        }
    }
}

/// Compute centroid (mean row, mean col) from a cell set.
fn compute_centroid(cells: &[(usize, usize)]) -> (f32, f32) {
    if cells.is_empty() {
        return (0.0, 0.0);
    }
    let n = cells.len() as f32;
    let sum_r: f32 = cells.iter().map(|(r, _)| *r as f32).sum();
    let sum_c: f32 = cells.iter().map(|(_, c)| *c as f32).sum();
    (sum_r / n, sum_c / n)
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn grid(cells: Vec<Vec<u8>>) -> Grid2D {
        Grid2D::new(cells)
    }

    #[test]
    fn test_empty_grid() {
        let g = Grid2D::empty(4, 4);
        let map = ObjectExtractor::extract(&g, Connectivity::Four);
        assert_eq!(map.object_count(), 0);
        assert_eq!(map.background_color, 0);
    }

    #[test]
    fn test_single_object() {
        // Background = 0 (most common), one 3x1 red object
        let g = grid(vec![
            vec![0, 0, 0, 0],
            vec![0, 1, 1, 1],
            vec![0, 0, 0, 0],
            vec![0, 0, 0, 0],
        ]);
        let map = ObjectExtractor::extract(&g, Connectivity::Four);
        assert_eq!(map.object_count(), 1);
        assert_eq!(map.objects[0].color, 1);
        assert_eq!(map.objects[0].area, 3);
        assert_eq!(map.objects[0].bbox.min_row, 1);
        assert_eq!(map.objects[0].bbox.max_row, 1);
        assert_eq!(map.objects[0].bbox.min_col, 1);
        assert_eq!(map.objects[0].bbox.max_col, 3);
    }

    #[test]
    fn test_two_separate_objects() {
        let g = grid(vec![
            vec![0, 1, 0, 0],
            vec![0, 1, 0, 0],
            vec![0, 0, 0, 2],
            vec![0, 0, 0, 2],
        ]);
        let map = ObjectExtractor::extract(&g, Connectivity::Four);
        assert_eq!(map.object_count(), 2);
        // Both objects have area 2, sorted by area desc so order is stable
        assert!(map.objects.iter().any(|o| o.color == 1 && o.area == 2));
        assert!(map.objects.iter().any(|o| o.color == 2 && o.area == 2));
    }

    #[test]
    fn test_same_color_separate_components() {
        // Two disconnected red blobs
        let g = grid(vec![
            vec![1, 1, 0, 0],
            vec![0, 0, 0, 0],
            vec![0, 0, 1, 1],
            vec![0, 0, 0, 0],
        ]);
        let map = ObjectExtractor::extract(&g, Connectivity::Four);
        assert_eq!(map.object_count(), 2);
        assert_eq!(map.objects[0].color, 1);
        assert_eq!(map.objects[1].color, 1);
    }

    #[test]
    fn test_diagonal_connectivity() {
        // Diagonal cells: 4-connected = 2 objects, 8-connected = 1 object
        let g = grid(vec![
            vec![0, 0, 0],
            vec![0, 1, 0],
            vec![0, 0, 1],
        ]);
        let map4 = ObjectExtractor::extract(&g, Connectivity::Four);
        assert_eq!(map4.object_count(), 2);

        let map8 = ObjectExtractor::extract(&g, Connectivity::Eight);
        assert_eq!(map8.object_count(), 1);
        assert_eq!(map8.objects[0].area, 2);
    }

    #[test]
    fn test_label_grid() {
        let g = grid(vec![
            vec![0, 1, 0],
            vec![0, 1, 2],
            vec![0, 0, 2],
        ]);
        let map = ObjectExtractor::extract(&g, Connectivity::Four);
        // background cells should be u32::MAX
        assert_eq!(map.labels[0][0], u32::MAX);
        // object cells should have valid indices
        assert_ne!(map.labels[0][1], u32::MAX);
        assert_ne!(map.labels[1][2], u32::MAX);
    }

    #[test]
    fn test_object_at() {
        let g = grid(vec![
            vec![0, 1, 0],
            vec![0, 1, 0],
            vec![0, 0, 0],
        ]);
        let map = ObjectExtractor::extract(&g, Connectivity::Four);
        let obj = map.object_at(0, 1).unwrap();
        assert_eq!(obj.color, 1);
        assert!(map.object_at(0, 0).is_none());
        assert!(map.object_at(99, 99).is_none());
    }

    #[test]
    fn test_bbox_iou() {
        let a = BBox { min_row: 0, min_col: 0, max_row: 3, max_col: 3 };
        let b = BBox { min_row: 2, min_col: 2, max_row: 5, max_col: 5 };
        let iou = a.iou(&b);
        // Intersection: 2x2=4, union: 16+16-4=28
        assert!((iou - 4.0 / 28.0).abs() < 0.01);

        // No overlap
        let c = BBox { min_row: 10, min_col: 10, max_row: 12, max_col: 12 };
        assert_eq!(a.iou(&c), 0.0);

        // Perfect overlap
        assert!((a.iou(&a) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_centroid() {
        let g = grid(vec![
            vec![0, 0, 0, 0],
            vec![0, 5, 5, 0],
            vec![0, 5, 5, 0],
            vec![0, 0, 0, 0],
        ]);
        let map = ObjectExtractor::extract(&g, Connectivity::Four);
        assert_eq!(map.object_count(), 1);
        let (cr, cc) = map.objects[0].centroid;
        assert!((cr - 1.5).abs() < 0.01);
        assert!((cc - 1.5).abs() < 0.01);
    }

    #[test]
    fn test_large_grid_performance() {
        // 64x64 grid with scattered objects — should extract quickly
        let mut cells = vec![vec![0u8; 64]; 64];
        // Place a few objects
        for r in 10..15 {
            for c in 10..20 {
                cells[r][c] = 3;
            }
        }
        for r in 30..35 {
            for c in 40..45 {
                cells[r][c] = 7;
            }
        }
        cells[50][50] = 2; // single-cell object

        let g = Grid2D::new(cells);
        let map = ObjectExtractor::extract(&g, Connectivity::Four);
        assert_eq!(map.object_count(), 3);
        // Largest object first
        assert_eq!(map.objects[0].area, 50); // 5x10
        assert_eq!(map.objects[1].area, 25); // 5x5
        assert_eq!(map.objects[2].area, 1);  // single cell
    }
}
