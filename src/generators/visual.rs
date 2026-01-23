/// Visual Subject Generator
/// 
/// Analyzes 2D flux matrix visualizations and generates subject definitions
/// based on the spatial positions, sacred geometry intersections, and semantic patterns
/// visible in the matrix structure.

use crate::ai_integration::AIModelIntegration;
use crate::error::{Result, SpatialVortexError};
use crate::models::{FluxMatrix, FluxNode, SacredGuide};
use crate::subject_generator::{GeneratedSubject, GeneratedNode, GeneratedSacredGuide};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Visual analysis data extracted from 2D flux matrix rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FluxMatrixVisualData {
    /// Subject being visualized
    pub subject: String,
    /// Position data for each node (0-9)
    pub node_positions: HashMap<u8, NodeVisualData>,
    /// Sacred intersection points (cyan dots from 2D images)
    pub sacred_intersections: Vec<IntersectionPoint>,
    /// Flow patterns visible in the doubling sequence (1→2→4→8→7→5→1)
    pub flow_patterns: Vec<FlowLine>,
}

/// Visual data for a single node in the 2D visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeVisualData {
    /// Flux position (0-9)
    pub position: u8,
    /// X,Y coordinates in 2D space
    pub coordinates: (f32, f32),
    /// Color representation (for ELP channels)
    pub color: ColorData,
    /// Size/scale (indicates importance/activity)
    pub scale: f32,
    /// Connected nodes
    pub connections: Vec<u8>,
    /// Is this a sacred position (3, 6, 9)?
    pub is_sacred: bool,
}

/// RGB color data from visual rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorData {
    pub r: f32,  // Red channel (Pathos/Emotion)
    pub g: f32,  // Green channel (Logos/Logic)
    pub b: f32,  // Blue channel (Ethos/Character)
}

impl ColorData {
    /// Analyze which channel is dominant
    pub fn dominant_channel(&self) -> &str {
        if self.r > self.g && self.r > self.b {
            "pathos"  // Emotion-dominant
        } else if self.g > self.r && self.g > self.b {
            "logos"   // Logic-dominant
        } else if self.b > self.r && self.b > self.g {
            "ethos"   // Character-dominant
        } else {
            "balanced"
        }
    }
}

/// Sacred intersection point (cyan dots in 2D visualization)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntersectionPoint {
    /// Coordinates where sacred lines intersect
    pub coordinates: (f32, f32),
    /// Which sacred positions create this intersection (e.g., [3, 9])
    pub sacred_positions: Vec<u8>,
    /// Significance level (0.0 - 1.0)
    pub significance: f32,
}

/// Flow line in the doubling sequence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowLine {
    /// Start position
    pub from: u8,
    /// End position
    pub to: u8,
    /// Curvature (from beam tensor curviness_signed)
    pub curvature: f32,
    /// Flow intensity (0.0 - 1.0)
    pub intensity: f32,
}

/// Generator that creates subjects from visual flux matrix data
pub struct VisualSubjectGenerator {
    ai_integration: AIModelIntegration,
}

impl VisualSubjectGenerator {
    /// Create new visual subject generator
    pub fn new(ai_integration: AIModelIntegration) -> Self {
        Self { ai_integration }
    }

    /// Generate subject definition from visual flux matrix data
    pub async fn generate_from_visual_data(
        &self,
        visual_data: &FluxMatrixVisualData,
    ) -> Result<GeneratedSubject> {
        println!("[Visual Generator] Analyzing flux matrix for: {}", visual_data.subject);

        // Step 1: Analyze visual patterns
        let analysis = self.analyze_visual_patterns(visual_data);

        // Step 2: Create AI prompt with visual insights
        let prompt = self.create_visual_analysis_prompt(visual_data, &analysis);

        // Step 3: Get AI to generate subject based on visual data
        let response = self.ai_integration
            .make_subject_generation_request(&prompt)
            .await?;

        // Step 4: Parse response into GeneratedSubject
        self.parse_ai_response(response, &visual_data.subject)
    }

    /// Analyze patterns in the visual data
    fn analyze_visual_patterns(&self, visual_data: &FluxMatrixVisualData) -> VisualAnalysis {
        let mut dominant_channels = HashMap::new();
        let mut sacred_clusters = Vec::new();
        let mut flow_intensity_map = HashMap::new();

        // Analyze color dominance for each position
        for (pos, node_data) in &visual_data.node_positions {
            dominant_channels.insert(*pos, node_data.color.dominant_channel().to_string());
        }

        // Identify sacred intersection clusters
        for intersection in &visual_data.sacred_intersections {
            if intersection.significance > 0.7 {
                sacred_clusters.push(intersection.clone());
            }
        }

        // Map flow intensities
        for flow in &visual_data.flow_patterns {
            flow_intensity_map.insert((flow.from, flow.to), flow.intensity);
        }

        // Calculate node density metrics
        let total_nodes = visual_data.node_positions.len();
        let total_sacred_intersections = sacred_clusters.len();
        
        VisualAnalysis {
            dominant_channels,
            sacred_clusters,
            flow_intensity_map,
            total_nodes,
            total_sacred_intersections,
        }
    }

    /// Create AI prompt incorporating visual analysis
    fn create_visual_analysis_prompt(
        &self,
        visual_data: &FluxMatrixVisualData,
        analysis: &VisualAnalysis,
    ) -> String {
        format!(
            r#"Generate a Spatial Vortex subject definition for: "{subject}"

VISUAL ANALYSIS DATA FROM 2D FLUX MATRIX:
==========================================

1. COLOR DOMINANCE (ELP Channels):
{color_analysis}

2. SACRED INTERSECTIONS:
   - Total significant intersections: {sacred_count}
   - High-significance clusters: {cluster_positions}

3. FLOW PATTERNS (Doubling Sequence 1→2→4→8→7→5→1):
{flow_analysis}

4. NODE POSITIONS & SCALE:
{node_analysis}

TASK:
Create a subject definition with:
- 6 regular nodes (positions 1,2,4,5,7,8) with names reflecting the visual patterns
- 3 sacred guides (positions 3,6,9) representing the fundamental principles

Consider:
- Nodes with ETHOS dominance (blue): Character/moral concepts
- Nodes with LOGOS dominance (green): Logic/analytical concepts  
- Nodes with PATHOS dominance (red): Emotion/feeling concepts
- Larger scale = more fundamental/important concept
- Sacred intersections indicate key organizing principles

Return JSON format:
{{
  "nodes": [
    {{"position": 1, "name": "ConceptName"}},
    ...
  ],
  "sacred_guides": [
    {{"position": 3, "name": "PrincipleName"}},
    ...
  ]
}}

Make names specific to "{subject}" and reflect the visual structure.
"#,
            subject = visual_data.subject,
            color_analysis = self.format_color_analysis(&analysis.dominant_channels),
            sacred_count = analysis.total_sacred_intersections,
            cluster_positions = self.format_sacred_clusters(&analysis.sacred_clusters),
            flow_analysis = self.format_flow_analysis(&analysis.flow_intensity_map),
            node_analysis = self.format_node_analysis(&visual_data.node_positions),
        )
    }

    fn format_color_analysis(&self, channels: &HashMap<u8, String>) -> String {
        channels
            .iter()
            .map(|(pos, channel)| format!("   Position {}: {} dominant", pos, channel))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn format_sacred_clusters(&self, clusters: &[IntersectionPoint]) -> String {
        if clusters.is_empty() {
            "None".to_string()
        } else {
            clusters
                .iter()
                .map(|c| format!("{:?} (sig: {:.2})", c.sacred_positions, c.significance))
                .collect::<Vec<_>>()
                .join(", ")
        }
    }

    fn format_flow_analysis(&self, flows: &HashMap<(u8, u8), f32>) -> String {
        flows
            .iter()
            .map(|((from, to), intensity)| {
                format!("   {} → {}: intensity {:.2}", from, to, intensity)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn format_node_analysis(&self, nodes: &HashMap<u8, NodeVisualData>) -> String {
        let mut sorted: Vec<_> = nodes.iter().collect();
        sorted.sort_by_key(|(pos, _)| *pos);

        sorted
            .iter()
            .map(|(pos, data)| {
                format!(
                    "   Position {}: scale={:.2}, sacred={}, connections={}",
                    pos,
                    data.scale,
                    data.is_sacred,
                    data.connections.len()
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn parse_ai_response(&self, response: String, subject: &str) -> Result<GeneratedSubject> {
        let parsed: serde_json::Value = serde_json::from_str(&response)
            .map_err(|e| SpatialVortexError::AIIntegration(format!("Parse error: {}", e)))?;

        let mut nodes = Vec::new();
        if let Some(nodes_array) = parsed["nodes"].as_array() {
            for node_obj in nodes_array {
                if let (Some(pos), Some(name)) = (
                    node_obj["position"].as_u64(),
                    node_obj["name"].as_str(),
                ) {
                    nodes.push(GeneratedNode {
                        position: pos as u8,
                        name: name.to_string(),
                    });
                }
            }
        }

        let mut sacred_guides = Vec::new();
        if let Some(guides_array) = parsed["sacred_guides"].as_array() {
            for guide_obj in guides_array {
                if let (Some(pos), Some(name)) = (
                    guide_obj["position"].as_u64(),
                    guide_obj["name"].as_str(),
                ) {
                    sacred_guides.push(GeneratedSacredGuide {
                        position: pos as u8,
                        name: name.to_string(),
                    });
                }
            }
        }

        Ok(GeneratedSubject {
            name: subject.to_string(),
            nodes,
            sacred_guides,
        })
    }

    /// Convert existing FluxMatrix to visual data format
    pub fn extract_visual_data_from_matrix(matrix: &FluxMatrix) -> FluxMatrixVisualData {
        let mut node_positions = HashMap::new();
        
        // Extract visual data from regular nodes
        for (pos, node) in &matrix.nodes {
            node_positions.insert(
                *pos,
                Self::node_to_visual_data(node),
            );
        }

        // Extract visual data from sacred guides
        for (pos, guide) in &matrix.sacred_guides {
            node_positions.insert(
                *pos,
                Self::sacred_to_visual_data(*pos, guide),
            );
        }

        // Extract sacred intersections from guides
        let mut sacred_intersections = Vec::new();
        for guide in matrix.sacred_guides.values() {
            for _intersection in &guide.intersection_points {
                sacred_intersections.push(IntersectionPoint {
                    coordinates: (0.0, 0.0), // Would be calculated from actual geometry
                    sacred_positions: vec![guide.position],
                    significance: 0.8, // Default high significance
                });
            }
        }

        // Extract flow patterns from connections
        let flow_patterns = Self::extract_flow_patterns(&matrix.nodes);

        FluxMatrixVisualData {
            subject: matrix.subject.clone(),
            node_positions,
            sacred_intersections,
            flow_patterns,
        }
    }

    fn node_to_visual_data(node: &FluxNode) -> NodeVisualData {
        // Convert node to visual representation
        // In a real implementation, this would use actual rendered positions
        NodeVisualData {
            position: node.position,
            coordinates: (0.0, 0.0), // Would calculate from actual geometry
            color: ColorData { r: 0.5, g: 0.5, b: 0.5 }, // Would derive from semantic data
            scale: 1.0,
            connections: node.connections.iter().map(|c| c.target_position).collect(),
            is_sacred: false,
        }
    }

    fn sacred_to_visual_data(position: u8, _guide: &SacredGuide) -> NodeVisualData {
        NodeVisualData {
            position,
            coordinates: (0.0, 0.0),
            color: ColorData { r: 0.0, g: 1.0, b: 1.0 }, // Cyan for sacred
            scale: 1.5, // Larger for sacred nodes
            connections: vec![],
            is_sacred: true,
        }
    }

    fn extract_flow_patterns(_nodes: &HashMap<u8, FluxNode>) -> Vec<FlowLine> {
        let doubling_sequence = [1, 2, 4, 8, 7, 5];
        let mut flows = Vec::new();

        for i in 0..doubling_sequence.len() {
            let from = doubling_sequence[i];
            let to = doubling_sequence[(i + 1) % doubling_sequence.len()];

            flows.push(FlowLine {
                from,
                to,
                curvature: 0.0,
                intensity: 0.7,
            });
        }

        flows
    }
}

/// Results of visual pattern analysis
#[derive(Debug)]
struct VisualAnalysis {
    dominant_channels: HashMap<u8, String>,
    sacred_clusters: Vec<IntersectionPoint>,
    flow_intensity_map: HashMap<(u8, u8), f32>,
    /// Total number of nodes (used for density calculations by visualization renderer)
    #[allow(dead_code)]
    total_nodes: usize,
    total_sacred_intersections: usize,
}

impl VisualAnalysis {
    /// Calculate node density (nodes per position)
    /// Used by visualization system to determine rendering complexity
    #[allow(dead_code)]  // Will be used by visualization renderer
    pub fn node_density(&self) -> f32 {
        self.total_nodes as f32 / 10.0  // 10 positions (0-9)
    }
    
    /// Calculate sacred intersection ratio
    /// Used to measure geometric alignment with sacred positions (3, 6, 9)
    #[allow(dead_code)]  // Will be used for quality metrics
    pub fn sacred_ratio(&self) -> f32 {
        if self.total_nodes == 0 {
            return 0.0;
        }
        self.total_sacred_intersections as f32 / self.total_nodes as f32
    }
    
    /// Get total node count
    /// Used for scaling and performance optimization decisions
    #[allow(dead_code)]  // Will be used by adaptive rendering
    pub fn total_nodes(&self) -> usize {
        self.total_nodes
    }
}
