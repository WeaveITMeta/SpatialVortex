/// Flux Matrix Visualization Module
/// 
/// Supports 2D (plotters) and 3D (Bevy) rendering

#[cfg(all(feature = "bevy_support", not(target_arch = "wasm32")))]
pub mod bevy_3d;
#[cfg(all(feature = "bevy_support", not(target_arch = "wasm32")))]
pub mod bevy_shapes;
#[cfg(all(feature = "bevy_support", feature = "voice", not(target_arch = "wasm32")))]
pub mod voice_3d;
#[cfg(not(target_arch = "wasm32"))]
pub mod dynamic_color_renderer;

/// 2D Flux Matrix Visualization
/// 
/// Provides matplotlib-compatible data structures for visualizing:
/// - Position mapping (0-9) on 2D plane
/// - Sacred geometry (3-6-9 pattern)
/// - Data point inference
/// - Flow directions and intersections
/// - ELP tensor calculations
/// - Node relationships and dynamics
///
/// ## Vortex Math Coordinate System
/// - X-Axis (3→6): Ethos to Pathos (Character → Emotion)
/// - Y-Axis (6→9): Pathos to Logos (Emotion → Logic)
/// - Y-Axis (3→9): Ethos to Logos (Character → Logic, diagonal)
/// - Sacred triangle vertices (3, 6, 9) form the measurement framework
/// - All values normalized to ±13 unit scale

use crate::models::FluxNode;
use crate::lock_free_flux::LockFreeFluxMatrix;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 2D coordinate for flux position
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

impl Point2D {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    
    pub fn distance_to(&self, other: &Point2D) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

/// Position layout on 2D plane
/// Maps 0-9 positions to geometric coordinates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FluxLayout {
    /// Position coordinates (0-9)
    pub positions: HashMap<u8, Point2D>,
    
    /// Sacred triangle vertices (3, 6, 9)
    pub sacred_triangle: [Point2D; 3],
    
    /// Center point of the pattern
    pub center: Point2D,
    
    /// Radius of the outer circle
    pub radius: f64,
}

impl Default for FluxLayout {
    fn default() -> Self {
        Self::circular_layout()
    }
}

impl FluxLayout {
    /// Create circular layout (positions around a circle)
    pub fn circular_layout() -> Self {
        let radius = 1.0;
        let center = Point2D::new(0.0, 0.0);
        let mut positions = HashMap::new();
        
        // Arrange 0-9 in a circle, starting from top (12 o'clock)
        // Position 0 at top, clockwise
        for i in 0..10u8 {
            let angle = std::f64::consts::PI / 2.0 - (i as f64) * 2.0 * std::f64::consts::PI / 10.0;
            positions.insert(i, Point2D::new(
                center.x + radius * angle.cos(),
                center.y + radius * angle.sin(),
            ));
        }
        
        // Sacred triangle (3, 6, 9)
        let sacred_triangle = [
            positions[&3],  // Top-right
            positions[&6],  // Bottom-left
            positions[&9],  // Left
        ];
        
        Self {
            positions,
            sacred_triangle,
            center,
            radius,
        }
    }
    
    /// Create sacred geometry layout (emphasizes 3-6-9)
    /// Vortex Math pattern: positions 0-9 arranged in circle
    /// - Position 9 at top (12 o'clock)
    /// - Clockwise: 1, 2, 3, 4, 5, 6, 7, 8
    /// - Sacred triangle: 3-6-9 (equilateral, 120° apart)
    pub fn sacred_geometry_layout() -> Self {
        let radius = 8.125; // Scaled to fit -13 to 13 coordinate system
        let y_shift = 2.6; // Adjusted slightly to center intersection at position 0
        let center = Point2D::new(0.0, y_shift); // Center also shifts up
        let mut positions = HashMap::new();
        
        // Vortex Math: 9 positions (1-9), with 0 optionally at center or merged with 9
        // Position 9 at top (90°), then clockwise 40° apart
        let angle_offset = std::f64::consts::PI / 2.0; // Start at top (90°)
        let angle_step = 2.0 * std::f64::consts::PI / 9.0; // 40° between positions
        
        // Position 9 at top (index 0 in our iteration)
        let pos_order = [9, 1, 2, 3, 4, 5, 6, 7, 8];
        
        for (i, pos) in pos_order.iter().enumerate() {
            let angle = angle_offset - (i as f64) * angle_step; // Clockwise from top
            positions.insert(*pos, Point2D::new(
                center.x + radius * angle.cos(),
                center.y + radius * angle.sin(),
            ));
        }
        
        // Position 0 at original center (now below the shifted circle)
        positions.insert(0, Point2D::new(0.0, 0.0));
        
        // Sacred triangle vertices: 3-6-9
        // - 9 at top (12 o'clock / 90°)
        // - 3 at right (4 o'clock / -30°)  
        // - 6 at left (8 o'clock / 210°)
        let sacred_triangle = [
            positions[&3],  // Right
            positions[&6],  // Bottom-left
            positions[&9],  // Top
        ];
        
        Self {
            positions,
            sacred_triangle,
            center,
            radius,
        }
    }
}

/// Data point on the flux matrix with inference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FluxDataPoint {
    /// Unique identifier
    pub id: String,
    
    /// Inferred position (0-9)
    pub position: u8,
    
    /// 2D coordinates on the plot
    pub coords: Point2D,
    
    /// Gold/reference position (0-9) - ground truth for accuracy measurement
    /// Used in realtime data processing pipeline for geometric reasoning evaluation
    pub gold_position: Option<u8>,
    
    /// Gold position coordinates (if gold_position is Some)
    pub gold_coords: Option<Point2D>,
    
    /// ELP tensor values
    pub ethos: f64,
    pub logos: f64,
    pub pathos: f64,
    
    /// Raw properties from data
    pub properties: HashMap<String, f64>,
    
    /// Distance to sacred positions
    pub sacred_distances: HashMap<u8, f64>,
    
    /// Flow direction (angle in radians)
    pub flow_direction: f64,
    
    /// Is this at a sacred position?
    pub is_sacred: bool,
    
    /// Judgment result at this position
    pub judgment: String,
}

impl FluxDataPoint {
    /// Create from FluxNode
    pub fn from_flux_node(node: &FluxNode, layout: &FluxLayout) -> Self {
        let position = node.position;
        let coords = layout.positions.get(&position).copied().unwrap_or(layout.center);
        
        // Extract ELP from parameters
        let ethos = node.attributes.parameters.get("ethos").copied().unwrap_or(0.5);
        let logos = node.attributes.parameters.get("logos").copied().unwrap_or(0.5);
        let pathos = node.attributes.parameters.get("pathos").copied().unwrap_or(0.5);
        
        // Calculate distances to sacred positions
        let mut sacred_distances = HashMap::new();
        for &sacred_pos in &[3, 6, 9] {
            if let Some(sacred_coords) = layout.positions.get(&sacred_pos) {
                sacred_distances.insert(sacred_pos, coords.distance_to(sacred_coords));
            }
        }
        
        // Initialize gold_position as None (set explicitly when evaluating accuracy)
        let gold_position = None;
        let gold_coords = None;
        
        // Calculate flow direction (towards center)
        let flow_direction = (coords.y - layout.center.y).atan2(coords.x - layout.center.x);
        
        let is_sacred = [3, 6, 9].contains(&position);
        
        // Judgment based on ELP entropy
        let entropy = (ethos + logos + pathos) / 3.0;
        let judgment = if entropy > 0.7 {
            "Reverse".to_string()
        } else if entropy < 0.3 {
            "Stabilize".to_string()
        } else {
            "Allow".to_string()
        };
        
        Self {
            id: node.semantic_index.neutral_base.clone(),
            position,
            coords,
            gold_position,
            gold_coords,
            ethos,
            logos,
            pathos,
            properties: node.attributes.parameters.clone(),
            sacred_distances,
            flow_direction,
            is_sacred,
            judgment,
        }
    }
    
    /// Set gold/reference position for accuracy evaluation
    /// Returns self for method chaining
    pub fn with_gold_position(mut self, gold_pos: u8, layout: &FluxLayout) -> Self {
        self.gold_position = Some(gold_pos);
        self.gold_coords = layout.positions.get(&gold_pos).copied();
        self
    }
    
    /// Calculate accuracy: distance between predicted and gold position
    /// Returns None if no gold position is set
    pub fn calculate_positional_accuracy(&self) -> Option<f64> {
        match (self.gold_coords, self.gold_position) {
            (Some(gold_coords), Some(_)) => {
                let distance = self.coords.distance_to(&gold_coords);
                // Convert distance to accuracy score (0.0-1.0)
                // Closer = higher accuracy. Max distance ~= 16 units
                Some((1.0 - (distance / 16.0)).max(0.0))
            }
            _ => None,
        }
    }
    
    /// Calculate position error: raw distance to gold position
    /// Returns None if no gold position is set
    pub fn calculate_position_error(&self) -> Option<f64> {
        match self.gold_coords {
            Some(gold_coords) => Some(self.coords.distance_to(&gold_coords)),
            None => None,
        }
    }
    
    /// Check if prediction matches gold position exactly
    pub fn is_exact_match(&self) -> Option<bool> {
        self.gold_position.map(|gold_pos| self.position == gold_pos)
    }
    
    /// Calculate tensor magnitude
    pub fn tensor_magnitude(&self) -> f64 {
        (self.ethos.powi(2) + self.logos.powi(2) + self.pathos.powi(2)).sqrt()
    }
    
    /// Get dominant ELP channel
    pub fn dominant_channel(&self) -> &str {
        if self.ethos > self.logos && self.ethos > self.pathos {
            "Ethos"
        } else if self.logos > self.pathos {
            "Logos"
        } else {
            "Pathos"
        }
    }
}

/// Flow line between two positions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowLine {
    pub from_pos: u8,
    pub to_pos: u8,
    pub from_coords: Point2D,
    pub to_coords: Point2D,
    pub strength: f64,  // 0.0 to 1.0
    pub is_sacred: bool,  // Connects to sacred position
}

/// Complete visualization data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FluxVisualization {
    /// Layout configuration
    pub layout: FluxLayout,
    
    /// Data points to plot
    pub data_points: Vec<FluxDataPoint>,
    
    /// Flow lines between positions
    pub flow_lines: Vec<FlowLine>,
    
    /// Sacred geometry elements
    pub sacred_elements: SacredGeometry,
    
    /// Metadata
    pub title: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Sacred geometry visualization elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SacredGeometry {
    /// Triangle connecting 3-6-9
    pub triangle_vertices: Vec<Point2D>,
    
    /// Circle through sacred positions
    pub sacred_circle_center: Point2D,
    pub sacred_circle_radius: f64,
    
    /// Vortex center (0,0)
    pub vortex_center: Point2D,
}

impl FluxVisualization {
    /// Create visualization from FluxMatrix
    pub fn from_flux_matrix(
        matrix: &LockFreeFluxMatrix,
        layout: FluxLayout,
        title: String,
    ) -> Self {
        let mut data_points = Vec::new();
        let mut flow_lines = Vec::new();
        
        // Extract all nodes
        for pos in 0..10u8 {
            if let Some(versioned_node) = matrix.get(pos) {
                let data_point = FluxDataPoint::from_flux_node(&versioned_node.node, &layout);
                data_points.push(data_point);
            }
        }
        
        // Generate flow lines - Vortex Math star pattern
        // Pattern: Each position connects to positions that create the internal star
        // The doubling pattern: 1→2, 2→4, 4→8, 8→7, 7→5, 5→1, etc.
        let vortex_connections = vec![
            (1, 2), (2, 4), (4, 8), (8, 7), (7, 5), (5, 1), // Inner hexagon
            (1, 5), (5, 7), (7, 8), (8, 4), (4, 2), (2, 1), // Return connections
            (3, 6), (6, 9), (9, 3), // Sacred triangle
            (3, 9), (9, 6), (6, 3), // Sacred triangle reverse
            // Additional star connections
            (1, 4), (2, 5), (4, 7), (5, 8), (7, 2), (8, 1),
        ];
        
        for (from_pos, to_pos) in vortex_connections {
            if let (Some(from_coords), Some(to_coords)) = (
                layout.positions.get(&from_pos),
                layout.positions.get(&to_pos),
            ) {
                let is_sacred = [3, 6, 9].contains(&from_pos) && [3, 6, 9].contains(&to_pos);
                
                flow_lines.push(FlowLine {
                    from_pos,
                    to_pos,
                    from_coords: *from_coords,
                    to_coords: *to_coords,
                    strength: if is_sacred { 1.0 } else { 0.3 },
                    is_sacred,
                });
            }
        }
        
        // Sacred geometry
        let sacred_triangle_vertices = layout.sacred_triangle.to_vec();
        
        // Calculate sacred circle (circumscribed around triangle)
        let sacred_circle_center = layout.center;
        let sacred_circle_radius = layout.positions.get(&3)
            .map(|p| p.distance_to(&sacred_circle_center))
            .unwrap_or(layout.radius);
        
        let sacred_elements = SacredGeometry {
            triangle_vertices: sacred_triangle_vertices,
            sacred_circle_center,
            sacred_circle_radius,
            vortex_center: layout.center,
        };
        
        Self {
            layout,
            data_points,
            flow_lines,
            sacred_elements,
            title,
            timestamp: chrono::Utc::now(),
        }
    }
    
    /// Export as JSON for matplotlib/Python
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
    
    /// Export as matplotlib-compatible Python dict
    pub fn to_matplotlib_dict(&self) -> HashMap<String, serde_json::Value> {
        let mut dict = HashMap::new();
        
        // Position coordinates
        let mut pos_x = Vec::new();
        let mut pos_y = Vec::new();
        let mut pos_labels = Vec::new();
        
        for (pos, coords) in &self.layout.positions {
            pos_x.push(coords.x);
            pos_y.push(coords.y);
            pos_labels.push(pos.to_string());
        }
        
        dict.insert("position_x".to_string(), serde_json::json!(pos_x));
        dict.insert("position_y".to_string(), serde_json::json!(pos_y));
        dict.insert("position_labels".to_string(), serde_json::json!(pos_labels));
        
        // Data points
        let mut data_x = Vec::new();
        let mut data_y = Vec::new();
        let mut data_labels = Vec::new();
        let mut data_colors = Vec::new();
        let mut data_sizes = Vec::new();
        
        for point in &self.data_points {
            data_x.push(point.coords.x);
            data_y.push(point.coords.y);
            data_labels.push(point.id.clone());
            
            // Color by dominant channel
            let color = match point.dominant_channel() {
                "Ethos" => "red",
                "Logos" => "blue",
                "Pathos" => "green",
                _ => "gray",
            };
            data_colors.push(color);
            
            // Size by tensor magnitude
            data_sizes.push(point.tensor_magnitude() * 100.0);
        }
        
        dict.insert("data_x".to_string(), serde_json::json!(data_x));
        dict.insert("data_y".to_string(), serde_json::json!(data_y));
        dict.insert("data_labels".to_string(), serde_json::json!(data_labels));
        dict.insert("data_colors".to_string(), serde_json::json!(data_colors));
        dict.insert("data_sizes".to_string(), serde_json::json!(data_sizes));
        
        // Sacred triangle
        let triangle_x: Vec<f64> = self.sacred_elements.triangle_vertices.iter().map(|p| p.x).collect();
        let triangle_y: Vec<f64> = self.sacred_elements.triangle_vertices.iter().map(|p| p.y).collect();
        
        dict.insert("sacred_triangle_x".to_string(), serde_json::json!(triangle_x));
        dict.insert("sacred_triangle_y".to_string(), serde_json::json!(triangle_y));
        
        // Flow lines
        let mut flow_x = Vec::new();
        let mut flow_y = Vec::new();
        
        for line in &self.flow_lines {
            flow_x.push(vec![line.from_coords.x, line.to_coords.x]);
            flow_y.push(vec![line.from_coords.y, line.to_coords.y]);
        }
        
        dict.insert("flow_x".to_string(), serde_json::json!(flow_x));
        dict.insert("flow_y".to_string(), serde_json::json!(flow_y));
        
        dict.insert("title".to_string(), serde_json::json!(self.title));
        
        dict
    }
}

/// Analysis of position intersections and dynamics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionAnalysis {
    pub position: u8,
    pub is_sacred: bool,
    pub sacred_proximity: f64,  // Distance to nearest sacred position
    pub nearest_sacred: u8,      // Which sacred position (3, 6, or 9)
    pub flow_convergence: f64,   // How many flows intersect here
    pub tensor_intensity: f64,   // ELP magnitude at this position
    pub judgment_type: String,   // Allow/Reverse/Stabilize
}

impl PositionAnalysis {
    pub fn analyze(point: &FluxDataPoint, layout: &FluxLayout) -> Self {
        let is_sacred = [3, 6, 9].contains(&point.position);
        
        // Find nearest sacred position
        let (nearest_sacred, sacred_proximity) = if is_sacred {
            (point.position, 0.0)
        } else {
            point.sacred_distances.iter()
                .min_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .map(|(pos, dist)| (*pos, *dist))
                .unwrap_or((3, 1.0))
        };
        
        // Flow convergence (how "central" this position is)
        let flow_convergence = 1.0 - point.coords.distance_to(&layout.center) / layout.radius;
        
        let tensor_intensity = point.tensor_magnitude();
        
        Self {
            position: point.position,
            is_sacred,
            sacred_proximity,
            nearest_sacred,
            flow_convergence,
            tensor_intensity,
            judgment_type: point.judgment.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_circular_layout() {
        let layout = FluxLayout::circular_layout();
        
        // Should have 10 positions
        assert_eq!(layout.positions.len(), 10);
        
        // Sacred triangle should have 3 vertices
        assert_eq!(layout.sacred_triangle.len(), 3);
        
        // All positions should be roughly same distance from center
        for (pos, coords) in &layout.positions {
            let dist = coords.distance_to(&layout.center);
            assert!((dist - layout.radius).abs() < 0.01, 
                "Position {} at distance {} from center", pos, dist);
        }
    }
    
    #[test]
    fn test_sacred_geometry_layout() {
        let layout = FluxLayout::sacred_geometry_layout();
        
        assert_eq!(layout.positions.len(), 10);
        
        // Sacred positions should form equilateral triangle
        let p3 = layout.positions[&3];
        let p6 = layout.positions[&6];
        let p9 = layout.positions[&9];
        
        let d36 = p3.distance_to(&p6);
        let d69 = p6.distance_to(&p9);
        let d93 = p9.distance_to(&p3);
        
        assert!((d36 - d69).abs() < 0.01);
        assert!((d69 - d93).abs() < 0.01);
    }
    
    #[test]
    fn test_point_2d_distance() {
        let p1 = Point2D::new(0.0, 0.0);
        let p2 = Point2D::new(3.0, 4.0);
        
        assert!((p1.distance_to(&p2) - 5.0).abs() < 0.001);
    }
}
