//! Intersection Analysis System
//! 
//! Cross-references information at intersection points where segments meet.
//! Sacred anchors (3, 6, 9) and manifest nodes (1, 2, 4, 8, 7, 5) create
//! intersections with implications for interpretation and reasoning.
//! 
//! ## Concept
//! - **Intersections**: Points where multiple paths/flows cross
//! - **Cross-Reference**: Information exchange between nodes at intersection
//! - **Implications**: Derived insights from node interactions
//! - **Sacred Intersections**: 3-6-9 triangle vertices and crossings
//! - **Manifest Intersections**: Doubling/halving sequence crossings

use crate::models::{ELPTensor, FluxNode};
use crate::runtime::vortex_cycle::{FORWARD_SEQUENCE, BACKWARD_SEQUENCE, SACRED_ANCHORS};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Type of intersection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IntersectionType {
    /// Sacred anchor intersection (3, 6, or 9)
    SacredAnchor,
    
    /// Sacred triangle edge (3-6, 6-9, 3-9)
    SacredEdge,
    
    /// Manifest node (1, 2, 4, 8, 7, 5)
    ManifestNode,
    
    /// Forward-backward crossing
    BidirectionalCrossing,
    
    /// Center point (position 0)
    CenterPoint,
}

/// Intersection point in the flux matrix
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Intersection {
    /// Position where intersection occurs (0-9)
    pub position: u8,
    
    /// Type of intersection
    pub intersection_type: IntersectionType,
    
    /// Node IDs that meet at this intersection
    pub node_ids: Vec<String>,
    
    /// Cross-referenced information
    pub cross_references: Vec<CrossReference>,
    
    /// Derived implications
    pub implications: Vec<Implication>,
    
    /// Strength of intersection (0.0-1.0)
    pub strength: f64,
    
    /// When intersection was detected (not serialized)
    #[serde(skip)]
    pub detected_at: std::time::Instant,
}

impl Default for Intersection {
    fn default() -> Self {
        Self {
            position: 0,
            intersection_type: IntersectionType::CenterPoint,
            node_ids: Vec::new(),
            cross_references: Vec::new(),
            implications: Vec::new(),
            strength: 0.0,
            detected_at: std::time::Instant::now(),
        }
    }
}

/// Cross-reference between nodes at intersection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossReference {
    /// Source node ID
    pub from_node: String,
    
    /// Target node ID
    pub to_node: String,
    
    /// ELP tensor similarity
    pub tensor_similarity: f64,
    
    /// Semantic relationship type
    pub relationship: RelationshipType,
    
    /// Confidence in cross-reference (0.0-1.0)
    pub confidence: f64,
}

/// Type of relationship between nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationshipType {
    /// Similar ELP patterns
    Harmonic,
    
    /// Opposite ELP patterns
    Complementary,
    
    /// One amplifies the other
    Amplifying,
    
    /// One dampens the other
    Dampening,
    
    /// Balanced interaction
    Neutral,
}

/// Implication derived from intersection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Implication {
    /// Description of implication
    pub description: String,
    
    /// Implication category
    pub category: ImplicationCategory,
    
    /// Strength of implication (0.0-1.0)
    pub strength: f64,
    
    /// Supporting evidence
    pub evidence: Vec<String>,
}

/// Category of implication
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImplicationCategory {
    /// Semantic meaning
    Semantic,
    
    /// Structural pattern
    Structural,
    
    /// Dynamic behavior
    Dynamic,
    
    /// Sacred geometry insight
    Geometric,
    
    /// Emergent property
    Emergent,
}

/// Intersection analysis engine
pub struct IntersectionAnalyzer {
    /// Active intersections
    intersections: Arc<RwLock<HashMap<u8, Intersection>>>,
    
    /// Intersection detection threshold (0.0-1.0) - intersections below this strength are ignored
    detection_threshold: f64,
    
    /// Sacred triangle edges
    sacred_edges: Vec<(u8, u8)>,
}

impl IntersectionAnalyzer {
    /// Create new intersection analyzer
    pub fn new(detection_threshold: f64) -> Self {
        // Define sacred triangle edges
        let sacred_edges = vec![
            (3, 6), // Ethos → Pathos
            (6, 9), // Pathos → Logos
            (3, 9), // Ethos → Logos (diagonal)
        ];
        
        Self {
            intersections: Arc::new(RwLock::new(HashMap::new())),
            detection_threshold,
            sacred_edges,
        }
    }
    
    /// Detect intersections from flux nodes
    pub async fn detect_intersections(&self, nodes: &HashMap<String, FluxNode>) {
        // Group nodes by position
        let mut position_groups: HashMap<u8, Vec<(String, &FluxNode)>> = HashMap::new();
        
        for (id, node) in nodes {
            position_groups
                .entry(node.position)
                .or_insert_with(Vec::new)
                .push((id.clone(), node));
        }
        
        let mut intersections = self.intersections.write().await;
        
        // Detect intersections at each position
        for (position, group) in position_groups {
            if group.len() < 2 {
                continue; // Need at least 2 nodes for intersection
            }
            
            let intersection_type = self.classify_intersection(position);
            let node_ids: Vec<String> = group.iter().map(|(id, _)| id.clone()).collect();
            
            // Generate cross-references
            let cross_refs = self.generate_cross_references(&group);
            
            // Derive implications
            let implications = self.derive_implications(position, &cross_refs, &group);
            
            // Calculate intersection strength
            let strength = self.calculate_strength(position, &cross_refs);
            
            // USE detection_threshold to filter out weak intersections
            if strength < self.detection_threshold {
                continue; // Skip weak intersections
            }
            
            let intersection = Intersection {
                position,
                intersection_type,
                node_ids,
                cross_references: cross_refs,
                implications,
                strength,
                detected_at: std::time::Instant::now(),
            };
            
            intersections.insert(position, intersection);
        }
    }
    
    /// Classify intersection type
    fn classify_intersection(&self, position: u8) -> IntersectionType {
        if position == 0 {
            IntersectionType::CenterPoint
        } else if SACRED_ANCHORS.contains(&position) {
            IntersectionType::SacredAnchor
        } else if self.is_on_sacred_edge(position) {
            IntersectionType::SacredEdge
        } else if FORWARD_SEQUENCE.contains(&position) {
            IntersectionType::ManifestNode
        } else {
            IntersectionType::BidirectionalCrossing
        }
    }
    
    /// Check if position is on a sacred triangle edge
    fn is_on_sacred_edge(&self, position: u8) -> bool {
        // Check if position lies on any sacred edge
        // This is a simplification - could use actual geometric calculations
        for (a, b) in &self.sacred_edges {
            let positions_between = self.positions_between(*a, *b);
            if positions_between.contains(&position) {
                return true;
            }
        }
        false
    }
    
    /// Get positions between two points
    fn positions_between(&self, a: u8, b: u8) -> Vec<u8> {
        let min = a.min(b);
        let max = a.max(b);
        (min..=max).collect()
    }
    
    /// Generate cross-references between nodes at intersection
    fn generate_cross_references(&self, nodes: &[(String, &FluxNode)]) -> Vec<CrossReference> {
        let mut cross_refs = Vec::new();
        
        // All pairs of nodes
        for i in 0..nodes.len() {
            for j in (i + 1)..nodes.len() {
                let (id1, node1) = &nodes[i];
                let (id2, node2) = &nodes[j];
                
                // Extract ELP tensors from node attributes
                let tensor1 = self.extract_tensor(node1);
                let tensor2 = self.extract_tensor(node2);
                
                // Calculate similarity
                let distance = tensor1.distance(&tensor2);
                let similarity = 1.0 / (1.0 + distance);
                
                // Determine relationship type
                let relationship = self.classify_relationship(&tensor1, &tensor2);
                
                // Calculate confidence
                let confidence = similarity * 0.8 + 0.2; // Base confidence
                
                cross_refs.push(CrossReference {
                    from_node: id1.clone(),
                    to_node: id2.clone(),
                    tensor_similarity: similarity,
                    relationship,
                    confidence,
                });
            }
        }
        
        cross_refs
    }
    
    /// Extract ELP tensor from flux node
    fn extract_tensor(&self, node: &FluxNode) -> ELPTensor {
        let ethos = *node.attributes.parameters.get("ethos").unwrap_or(&0.5);
        let logos = *node.attributes.parameters.get("logos").unwrap_or(&0.5);
        let pathos = *node.attributes.parameters.get("pathos").unwrap_or(&0.5);
        
        ELPTensor::new(ethos, logos, pathos)
    }
    
    /// Classify relationship between two tensors
    fn classify_relationship(&self, t1: &ELPTensor, t2: &ELPTensor) -> RelationshipType {
        let distance = t1.distance(t2);
        
        if distance < 2.0 {
            RelationshipType::Harmonic
        } else if distance > 10.0 {
            RelationshipType::Complementary
        } else {
            // Check for amplifying/dampening patterns
            let dot_product = t1.ethos * t2.ethos + t1.logos * t2.logos + t1.pathos * t2.pathos;
            
            if dot_product > 5.0 {
                RelationshipType::Amplifying
            } else if dot_product < -5.0 {
                RelationshipType::Dampening
            } else {
                RelationshipType::Neutral
            }
        }
    }
    
    /// Derive implications from intersection
    fn derive_implications(
        &self,
        position: u8,
        cross_refs: &[CrossReference],
        nodes: &[(String, &FluxNode)],
    ) -> Vec<Implication> {
        let mut implications = Vec::new();
        
        // Sacred anchor implications
        if SACRED_ANCHORS.contains(&position) {
            let anchor_name = match position {
                3 => "Ethos (Character)",
                6 => "Pathos (Emotion)",
                9 => "Logos (Logic)",
                _ => "Unknown",
            };
            
            implications.push(Implication {
                description: format!(
                    "Sacred anchor intersection at {} - {} nodes converge at fundamental principle",
                    anchor_name, nodes.len()
                ),
                category: ImplicationCategory::Geometric,
                strength: 0.9,
                evidence: vec![
                    format!("Position {} is sacred anchor", position),
                    format!("{} nodes present", nodes.len()),
                ],
            });
        }
        
        // Harmonic resonance implications
        let harmonic_count = cross_refs.iter()
            .filter(|cr| cr.relationship == RelationshipType::Harmonic)
            .count();
        
        if harmonic_count > 0 {
            implications.push(Implication {
                description: format!(
                    "Harmonic resonance detected - {} pairs show high similarity",
                    harmonic_count
                ),
                category: ImplicationCategory::Dynamic,
                strength: (harmonic_count as f64 / cross_refs.len().max(1) as f64) * 0.8,
                evidence: vec![format!("{} harmonic pairs", harmonic_count)],
            });
        }
        
        // Complementary balance implications
        let complementary_count = cross_refs.iter()
            .filter(|cr| cr.relationship == RelationshipType::Complementary)
            .count();
        
        if complementary_count > 0 {
            implications.push(Implication {
                description: format!(
                    "Complementary balance - {} opposing forces in equilibrium",
                    complementary_count
                ),
                category: ImplicationCategory::Emergent,
                strength: (complementary_count as f64 / cross_refs.len().max(1) as f64) * 0.7,
                evidence: vec![format!("{} complementary pairs", complementary_count)],
            });
        }
        
        // Manifest node implications
        if FORWARD_SEQUENCE.contains(&position) || BACKWARD_SEQUENCE.contains(&position) {
            let sequence_name = if FORWARD_SEQUENCE.contains(&position) {
                "forward (inference)"
            } else {
                "backward (training)"
            };
            
            implications.push(Implication {
                description: format!(
                    "Manifest node in {} sequence - active propagation dynamics",
                    sequence_name
                ),
                category: ImplicationCategory::Structural,
                strength: 0.75,
                evidence: vec![
                    format!("Position {} in {} sequence", position, sequence_name),
                ],
            });
        }
        
        implications
    }
    
    /// Calculate intersection strength
    fn calculate_strength(&self, position: u8, cross_refs: &[CrossReference]) -> f64 {
        let mut strength = 0.0;
        
        // Sacred positions have inherent strength
        if SACRED_ANCHORS.contains(&position) {
            strength += 0.5;
        }
        
        // Average cross-reference confidence
        if !cross_refs.is_empty() {
            let avg_confidence: f64 = cross_refs.iter()
                .map(|cr| cr.confidence)
                .sum::<f64>() / cross_refs.len() as f64;
            strength += avg_confidence * 0.5;
        }
        
        strength.min(1.0)
    }
    
    /// Get intersection at position
    pub async fn get_intersection(&self, position: u8) -> Option<Intersection> {
        let intersections = self.intersections.read().await;
        intersections.get(&position).cloned()
    }
    
    /// Get all intersections
    pub async fn get_all_intersections(&self) -> Vec<Intersection> {
        let intersections = self.intersections.read().await;
        intersections.values().cloned().collect()
    }
    
    /// Get sacred intersections only (3, 6, 9)
    pub async fn get_sacred_intersections(&self) -> Vec<Intersection> {
        let intersections = self.intersections.read().await;
        
        intersections
            .values()
            .filter(|i| i.intersection_type == IntersectionType::SacredAnchor)
            .cloned()
            .collect()
    }
    
    /// Get manifest intersections only (1, 2, 4, 8, 7, 5)
    pub async fn get_manifest_intersections(&self) -> Vec<Intersection> {
        let intersections = self.intersections.read().await;
        
        intersections
            .values()
            .filter(|i| i.intersection_type == IntersectionType::ManifestNode)
            .cloned()
            .collect()
    }
    
    /// Analyze interdynamics between nodes at intersection
    pub async fn analyze_interdynamics(&self, position: u8) -> Option<InterdynamicsAnalysis> {
        let intersections = self.intersections.read().await;
        let intersection = intersections.get(&position)?;
        
        // Calculate network effects
        let total_nodes = intersection.node_ids.len();
        let total_connections = intersection.cross_references.len();
        let avg_similarity = if !intersection.cross_references.is_empty() {
            intersection.cross_references.iter()
                .map(|cr| cr.tensor_similarity)
                .sum::<f64>() / total_connections as f64
        } else {
            0.0
        };
        
        // Identify dominant patterns
        let mut relationship_counts: HashMap<RelationshipType, usize> = HashMap::new();
        for cr in &intersection.cross_references {
            *relationship_counts.entry(cr.relationship).or_insert(0) += 1;
        }
        
        let dominant_pattern = relationship_counts.iter()
            .max_by_key(|(_, count)| *count)
            .map(|(rel, _)| *rel);
        
        // Calculate emergent properties
        let coherence = avg_similarity * intersection.strength;
        let complexity = (total_connections as f64).log2() / (total_nodes as f64).max(1.0);
        
        Some(InterdynamicsAnalysis {
            position,
            total_nodes,
            total_connections,
            avg_similarity,
            dominant_pattern,
            coherence,
            complexity,
            implications: intersection.implications.clone(),
        })
    }
    
    /// Get statistics
    pub async fn stats(&self) -> IntersectionStats {
        let intersections = self.intersections.read().await;
        
        let total = intersections.len();
        let sacred = intersections.values()
            .filter(|i| i.intersection_type == IntersectionType::SacredAnchor)
            .count();
        let manifest = intersections.values()
            .filter(|i| i.intersection_type == IntersectionType::ManifestNode)
            .count();
        
        let avg_strength = if !intersections.is_empty() {
            intersections.values().map(|i| i.strength).sum::<f64>() / total as f64
        } else {
            0.0
        };
        
        let total_implications = intersections.values()
            .map(|i| i.implications.len())
            .sum();
        
        IntersectionStats {
            total_intersections: total,
            sacred_intersections: sacred,
            manifest_intersections: manifest,
            avg_strength,
            total_implications,
        }
    }
}

/// Interdynamics analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterdynamicsAnalysis {
    pub position: u8,
    pub total_nodes: usize,
    pub total_connections: usize,
    pub avg_similarity: f64,
    pub dominant_pattern: Option<RelationshipType>,
    pub coherence: f64,
    pub complexity: f64,
    pub implications: Vec<Implication>,
}

/// Intersection statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntersectionStats {
    pub total_intersections: usize,
    pub sacred_intersections: usize,
    pub manifest_intersections: usize,
    pub avg_strength: f64,
    pub total_implications: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{NodeAttributes, NodeState, NodeDynamics, SemanticIndex};
    use std::collections::HashMap;

    fn create_test_node(position: u8, ethos: f64, logos: f64, pathos: f64) -> FluxNode {
        let mut params = HashMap::new();
        params.insert("ethos".to_string(), ethos);
        params.insert("logos".to_string(), logos);
        params.insert("pathos".to_string(), pathos);
        
        FluxNode {
            position,
            base_value: position,
            semantic_index: SemanticIndex {
                positive_associations: vec![],
                negative_associations: vec![],
                neutral_base: format!("Node_{}", position),
                predicates: vec![],
                relations: vec![],
            },
            attributes: NodeAttributes {
                properties: HashMap::new(),
                parameters: params,
                state: NodeState {
                    active: true,
                    last_accessed: chrono::Utc::now(),
                    usage_count: 0,
                    context_stack: vec![],
                },
                dynamics: NodeDynamics::default(),
            },
            connections: vec![],
        }
    }

    #[tokio::test]
    async fn test_sacred_intersection_detection() {
        let analyzer = IntersectionAnalyzer::new(0.5);
        
        let mut nodes = HashMap::new();
        nodes.insert("node1".to_string(), create_test_node(3, 0.8, 0.5, 0.6));
        nodes.insert("node2".to_string(), create_test_node(3, 0.7, 0.5, 0.7));
        
        analyzer.detect_intersections(&nodes).await;
        
        let intersection = analyzer.get_intersection(3).await.unwrap();
        assert_eq!(intersection.intersection_type, IntersectionType::SacredAnchor);
        assert_eq!(intersection.node_ids.len(), 2);
    }

    #[tokio::test]
    async fn test_cross_reference_generation() {
        let analyzer = IntersectionAnalyzer::new(0.5);
        
        let mut nodes = HashMap::new();
        nodes.insert("node1".to_string(), create_test_node(1, 0.9, 0.5, 0.5));
        nodes.insert("node2".to_string(), create_test_node(1, 0.8, 0.6, 0.6));
        
        analyzer.detect_intersections(&nodes).await;
        
        let intersection = analyzer.get_intersection(1).await.unwrap();
        assert!(!intersection.cross_references.is_empty());
        assert!(intersection.cross_references[0].tensor_similarity > 0.0);
    }

    #[tokio::test]
    async fn test_interdynamics_analysis() {
        let analyzer = IntersectionAnalyzer::new(0.5);
        
        let mut nodes = HashMap::new();
        for i in 0..3 {
            nodes.insert(
                format!("node{}", i),
                create_test_node(6, 0.5 + i as f64 * 0.1, 0.5, 0.5)
            );
        }
        
        analyzer.detect_intersections(&nodes).await;
        
        let analysis = analyzer.analyze_interdynamics(6).await.unwrap();
        assert_eq!(analysis.total_nodes, 3);
        assert!(analysis.coherence > 0.0);
    }
}
