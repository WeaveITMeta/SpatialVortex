use crate::error::{Result, SpatialVortexError};
use crate::models::*;
use crate::core::sacred_geometry::pattern_coherence::{PatternCoherenceTracker, CoherenceMetrics};
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

/// Core Flux Matrix implementation based on the pattern: 1, 2, 4, 8, 7, 5, 1
/// With sacred guides at positions 3, 6, 9
#[derive(Clone, Debug)]
pub struct FluxMatrixEngine {
    /// Base flux pattern that repeats infinitely
    pub base_pattern: [u8; 7],
    /// Sacred guide positions with special properties
    pub sacred_positions: [u8; 3],
    /// Pattern coherence tracker for real-time 3-6-9 monitoring
    pub pattern_tracker: PatternCoherenceTracker,
    /// Current position in vortex cycle
    current_position: u8,
}

impl Default for FluxMatrixEngine {
    fn default() -> Self {
        Self {
            base_pattern: [1, 2, 4, 8, 7, 5, 1], // Core discovery pattern
            sacred_positions: [3, 6, 9],         // Divine intersection points
            pattern_tracker: PatternCoherenceTracker::new(100),
            current_position: 1,
        }
    }
}

impl FluxMatrixEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Generate a flux matrix for a given subject
    /// Checks for predefined subject definitions first, then creates generic matrix
    pub fn create_matrix(&self, subject: String) -> Result<FluxMatrix> {
        // Check if we have curated definition with populated semantics
        if let Some(subject_def) = crate::subject_definitions::get_subject_definition(&subject) {
            return crate::subject_definitions::create_matrix_from_curated(subject_def, self);
        }

        // Fallback to generic matrix creation
        let matrix_id = Uuid::new_v4();
        let now = Utc::now();

        let mut nodes = HashMap::new();
        let mut sacred_guides = HashMap::new();

        // Create the 10 positions (0-9: center, regular nodes, sacred guides at 3, 6, 9)
        for position in 0..=9 {
            if self.sacred_positions.contains(&position) {
                // Create sacred guide
                let guide = self.create_sacred_guide(position, &subject)?;
                sacred_guides.insert(position, guide);
            } else {
                // Create regular flux node
                let node = self.create_flux_node(position, &subject)?;
                nodes.insert(position, node);
            }
        }

        Ok(FluxMatrix {
            id: matrix_id,
            subject,
            nodes,
            sacred_guides,
            created_at: now,
            updated_at: now,
        })
    }

    /// Create a flux matrix from a subject definition
    /// Semantic associations are empty by default and should be populated dynamically via AI/API
    pub fn create_matrix_from_definition(
        &self,
        subject_def: crate::generators::subjects::SubjectDefinition,
    ) -> Result<FluxMatrix> {
        let matrix_id = Uuid::new_v4();
        let now = Utc::now();
        let subject = subject_def.name.clone();

        let mut nodes = HashMap::new();
        let mut sacred_guides = HashMap::new();

        // Create nodes from subject definition
        // Semantic associations will be fetched dynamically when needed
        for node_def in &subject_def.nodes {
            let base_value = self.get_flux_value_at_position(node_def.position);

            let semantic_index = SemanticIndex {
                positive_associations: Vec::new(), // Populated dynamically via AI/API
                negative_associations: Vec::new(), // Populated dynamically via AI/API
                neutral_base: node_def.name.clone(),
                predicates: Vec::new(),
                relations: Vec::new(),
            };

            let mut node = FluxNode {
                position: node_def.position,
                base_value,
                semantic_index,
                attributes: NodeAttributes {
                    properties: HashMap::new(),
                    parameters: HashMap::new(),
                    state: NodeState {
                        active: true,
                        last_accessed: now,
                        usage_count: 0,
                        context_stack: Vec::new(),
                    },
                    dynamics: NodeDynamics::default(),
                },
                connections: self.create_node_connections(node_def.position),
            };

            // Initialize dynamics with loop and order awareness
            use crate::core::sacred_geometry::node_dynamics::FluxNodeDynamics;
            node.initialize_dynamics(None);

            nodes.insert(node_def.position, node);
        }

        // Create sacred guides from subject definition
        // Sacred guides use their name directly without additional properties
        for sacred_def in &subject_def.sacred_guides {
            let mut intersection_points = self.create_intersection_points(sacred_def.position);

            // Update significance for subject context
            for intersection in &mut intersection_points {
                intersection.significance =
                    format!("{} - {}", sacred_def.name, intersection.significance);
            }

            let guide = SacredGuide {
                position: sacred_def.position,
                divine_properties: vec![sacred_def.name.clone()], // Use name as primary property
                intersection_points,
                geometric_significance: format!(
                    "Sacred {} in {}: Fundamental organizing principle",
                    sacred_def.name, subject
                ),
            };

            sacred_guides.insert(sacred_def.position, guide);
        }

        Ok(FluxMatrix {
            id: matrix_id,
            subject,
            nodes,
            sacred_guides,
            created_at: now,
            updated_at: now,
        })
    }

    /// Populate semantic associations for a matrix dynamically using AI/API
    /// 
    /// NOTE: Semantic associations are now pre-populated in curated definitions.
    /// This method is kept for backward compatibility but is deprecated.
    #[cfg(not(target_arch = "wasm32"))]
    #[deprecated(note = "Use curated subject definitions with pre-populated semantics instead")]
    pub async fn populate_semantic_associations(
        &self,
        _matrix: &mut FluxMatrix,
        _ai_integration: &crate::ai_integration::AIModelIntegration,
    ) -> Result<()> {
        // Semantic associations are now pre-populated in subject_definitions
        // No dynamic population needed
        Ok(())
    }

    /// Create a flux node at given position
    pub fn create_flux_node(&self, position: u8, subject: &str) -> Result<FluxNode> {
        let base_value = self.get_flux_value_at_position(position);

        let semantic_index = SemanticIndex {
            positive_associations: Vec::new(), // Will be populated by ML/AI
            negative_associations: Vec::new(),
            neutral_base: format!("{}_node_{}", subject, position),
            predicates: Vec::new(),
            relations: Vec::new(),
        };

        let attributes = NodeAttributes {
            properties: HashMap::new(),
            parameters: HashMap::new(),
            state: NodeState {
                active: true,
                last_accessed: Utc::now(),
                usage_count: 0,
                context_stack: Vec::new(),
            },
            dynamics: NodeDynamics::default(),
        };

        let mut node = FluxNode {
            position,
            base_value,
            semantic_index,
            attributes,
            connections: self.create_node_connections(position),
        };
        
        // Initialize dynamics with loop and order awareness
        use crate::core::sacred_geometry::node_dynamics::FluxNodeDynamics;
        node.initialize_dynamics(None);

        Ok(node)
    }

    /// Create a sacred guide node (3, 6, 9)
    pub fn create_sacred_guide(&self, position: u8, subject: &str) -> Result<SacredGuide> {
        if !self.sacred_positions.contains(&position) {
            return Err(SpatialVortexError::InvalidFluxMatrix(format!(
                "Position {} is not a sacred guide position",
                position
            )));
        }

        let divine_properties = match position {
            3 => vec![
                "Creative Trinity".to_string(),
                "Synthesis Point".to_string(),
                "Bridge Between Realms".to_string(),
            ],
            6 => vec![
                "Harmonic Balance".to_string(),
                "Geometric Center".to_string(),
                "Stability Anchor".to_string(),
            ],
            9 => vec![
                "Completion Cycle".to_string(),
                "Infinite Loop Gateway".to_string(),
                "Transcendence Portal".to_string(),
            ],
            _ => vec!["Unknown Sacred Property".to_string()],
        };

        let intersection_points = self.create_intersection_points(position);
        let geometric_significance = format!(
            "Sacred {} forms triangular patterns with divine computation for {}",
            position, subject
        );

        Ok(SacredGuide {
            position,
            divine_properties,
            intersection_points,
            geometric_significance,
        })
    }

    /// Get flux pattern value at given position
    /// Maps positions to flux pattern: [1,2,4,8,7,5,1] with sacred positions (3,6,9) manifesting themselves
    /// Position mapping: 0→0(void), 1→1, 2→2, 3→3(sacred), 4→4, 5→5, 6→6(sacred), 7→7, 8→2(loops), 9→9(sacred)
    pub fn get_flux_value_at_position(&self, position: u8) -> u8 {
        match position {
            0 => 0,  // Center/void
            3 | 6 | 9 => position,  // Sacred guides manifest themselves
            1..=7 => self.base_pattern[(position - 1) as usize],  // Regular positions use pattern
            8 => self.base_pattern[1],  // Position 8 loops back to pattern[1] = 2
            _ => 0,  // Safety fallback for out of bounds
        }
    }
    /// Create connections between nodes following flux pattern
    pub fn create_node_connections(&self, position: u8) -> Vec<NodeConnection> {
        let mut connections = Vec::new();

        match position {
            0 => {
                // Center node connects to all other positions (1-9)
                for pos in 1..=9 {
                    let conn_type = if self.sacred_positions.contains(&pos) {
                        ConnectionType::Sacred
                    } else {
                        ConnectionType::Geometric
                    };
                    connections.push(NodeConnection {
                        target_position: pos,
                        connection_type: conn_type,
                        weight: 1.0,
                        bidirectional: true,
                    });
                }
            }
            pos if self.sacred_positions.contains(&pos) => {
                // Sacred guides have special connection patterns
                connections.push(NodeConnection {
                    target_position: 0,
                    connection_type: ConnectionType::Sacred,
                    weight: 1.5,
                    bidirectional: true,
                });

                // Connect to other sacred guides
                for &sacred_pos in &self.sacred_positions {
                    if sacred_pos != pos {
                        connections.push(NodeConnection {
                            target_position: sacred_pos,
                            connection_type: ConnectionType::Sacred,
                            weight: 1.2,
                            bidirectional: true,
                        });
                    }
                }
            }
            _ => {
                // Regular nodes follow sequential pattern
                let next = if position == 8 { 1 } else { position + 1 };
                let prev = if position == 1 { 8 } else { position - 1 };

                connections.push(NodeConnection {
                    target_position: next,
                    connection_type: ConnectionType::Sequential,
                    weight: 1.0,
                    bidirectional: false,
                });

                connections.push(NodeConnection {
                    target_position: prev,
                    connection_type: ConnectionType::Sequential,
                    weight: 0.8,
                    bidirectional: false,
                });

                // Connect to center
                connections.push(NodeConnection {
                    target_position: 0,
                    connection_type: ConnectionType::Geometric,
                    weight: 0.5,
                    bidirectional: true,
                });
            }
        }

        connections
    }

    /// Create intersection points for sacred guides
    pub fn create_intersection_points(&self, sacred_position: u8) -> Vec<IntersectionPoint> {
        let mut intersections = Vec::new();

        match sacred_position {
            3 => {
                intersections.push(IntersectionPoint {
                    with_node: 6,
                    significance: "Trinity-Harmony Bridge".to_string(),
                    computational_value: 3.6,
                });
                intersections.push(IntersectionPoint {
                    with_node: 9,
                    significance: "Creation-Completion Arc".to_string(),
                    computational_value: 3.9,
                });
            }
            6 => {
                intersections.push(IntersectionPoint {
                    with_node: 3,
                    significance: "Harmony-Trinity Bridge".to_string(),
                    computational_value: 6.3,
                });
                intersections.push(IntersectionPoint {
                    with_node: 9,
                    significance: "Balance-Transcendence Arc".to_string(),
                    computational_value: 6.9,
                });
            }
            9 => {
                intersections.push(IntersectionPoint {
                    with_node: 3,
                    significance: "Completion-Creation Loop".to_string(),
                    computational_value: 9.3,
                });
                intersections.push(IntersectionPoint {
                    with_node: 6,
                    significance: "Transcendence-Balance Arc".to_string(),
                    computational_value: 9.6,
                });
            }
            _ => {}
        }

        intersections
    }

    /// Convert seed number to flux pattern sequence
    /// Deterministic 9-step sequence via doubling + digit reduction
    pub fn seed_to_flux_sequence(&self, seed: u64) -> Vec<u8> {
        let mut seq = Vec::with_capacity(9);
        let mut current = seed;
        for _ in 0..9 {
            current = current * 2;
            let reduced = self.reduce_digits(current) as u8;
            seq.push(reduced);
            current = reduced as u64;
        }
        seq
    }

    /// Reduce multi-digit numbers by summing digits (key flux operation)
    pub fn reduce_digits(&self, mut number: u64) -> u64 {
        while number > 9 {
            let mut sum = 0;
            while number > 0 {
                sum += number % 10;
                number /= 10;
            }
            number = sum;
        }
        number
    }

    /// Find node position from flux value
    /// Direct 1:1 mapping: flux value maps directly to matrix position
    /// Positions 0-9 correspond directly to their flux values
    pub fn flux_value_to_position(&self, value: u8) -> Option<u8> {
        if value <= 9 {
            Some(value)
        } else {
            None
        }
    }

    /// Validate flux matrix integrity
    pub fn validate_matrix(&self, matrix: &FluxMatrix) -> Result<bool> {
        // Check all positions 0-9 are covered
        let mut covered_positions = Vec::new();

        for position in matrix.nodes.keys() {
            covered_positions.push(*position);
        }

        for position in matrix.sacred_guides.keys() {
            covered_positions.push(*position);
        }

        covered_positions.sort();
        let expected: Vec<u8> = (0..=9).collect();

        if covered_positions != expected {
            return Err(SpatialVortexError::InvalidFluxMatrix(
                "Matrix does not cover all required positions 0-9".to_string(),
            ));
        }

        // Validate sacred guides are at correct positions
        for position in matrix.sacred_guides.keys() {
            if !self.sacred_positions.contains(position) {
                return Err(SpatialVortexError::InvalidFluxMatrix(format!(
                    "Sacred guide at invalid position: {}",
                    position
                )));
            }
        }

        Ok(true)
    }

    /// Update matrix with reinforcement learning data
    pub fn update_matrix_with_rl(
        &self,
        matrix: &mut FluxMatrix,
        adjustment: LearningAdjustment,
    ) -> Result<()> {
        let now = Utc::now();
        matrix.updated_at = now;

        // Apply adjustment to relevant nodes
        for node in matrix.nodes.values_mut() {
            node.attributes
                .dynamics
                .learning_adjustments
                .push(adjustment.clone());
            node.attributes.state.last_accessed = now;
        }

        Ok(())
    }

    /// Attach a predicate to a node (Subject-Predicate model)
    pub fn add_predicate_to_node(
        &self,
        matrix: &mut FluxMatrix,
        position: u8,
        predicate: Predicate,
    ) -> Result<()> {
        let now = Utc::now();
        if let Some(node) = matrix.nodes.get_mut(&position) {
            node.semantic_index.predicates.push(predicate);
            node.attributes.state.last_accessed = now;
            node.attributes.state.usage_count += 1;
            matrix.updated_at = now;
            Ok(())
        } else {
            Err(SpatialVortexError::InvalidFluxMatrix(format!(
                "No node at position {}",
                position
            )))
        }
    }

    /// Retrieve a sorted predicate ladder for a node
    /// Sort order: ladder_rank asc, weight desc, index asc
    pub fn get_predicate_ladder(
        &self,
        matrix: &FluxMatrix,
        position: u8,
    ) -> Result<Vec<Predicate>> {
        if let Some(node) = matrix.nodes.get(&position) {
            let mut preds = node.semantic_index.predicates.clone();
            preds.sort_by(|a, b| {
                a.ladder_rank
                    .cmp(&b.ladder_rank)
                    .then_with(|| b.weight.partial_cmp(&a.weight).unwrap_or(std::cmp::Ordering::Equal))
                    .then(a.index.cmp(&b.index))
            });
            Ok(preds)
        } else {
            Err(SpatialVortexError::InvalidFluxMatrix(format!(
                "No node at position {}",
                position
            )))
        }
    }

    /// Attach a relation triple (predicate+object) to a node
    pub fn add_relation_to_node(
        &self,
        matrix: &mut FluxMatrix,
        position: u8,
        relation: Relation,
    ) -> Result<()> {
        let now = Utc::now();
        if let Some(node) = matrix.nodes.get_mut(&position) {
            node.semantic_index.relations.push(relation);
            node.attributes.state.last_accessed = now;
            node.attributes.state.usage_count += 1;
            matrix.updated_at = now;
            Ok(())
        } else {
            Err(SpatialVortexError::InvalidFluxMatrix(format!(
                "No node at position {}",
                position
            )))
        }
    }

    /// Retrieve sorted relations for a node
    /// Sort order: ladder_rank asc, weight desc, index asc
    pub fn get_relations_for_node(
        &self,
        matrix: &FluxMatrix,
        position: u8,
    ) -> Result<Vec<Relation>> {
        if let Some(node) = matrix.nodes.get(&position) {
            let mut rels = node.semantic_index.relations.clone();
            rels.sort_by(|a, b| {
                a.ladder_rank
                    .cmp(&b.ladder_rank)
                    .then_with(|| b.weight.partial_cmp(&a.weight).unwrap_or(std::cmp::Ordering::Equal))
                    .then(a.index.cmp(&b.index))
            });
            Ok(rels)
        } else {
            Err(SpatialVortexError::InvalidFluxMatrix(format!(
                "No node at position {}",
                position
            )))
        }
    }
    
    /// Calculate flux position from ELP (Ethos, Logos, Pathos) values
    ///
    /// Maps ELP coordinates to the 10-position flux matrix (0-9).
    /// Uses sacred geometry principles to determine optimal position.
    ///
    /// # Algorithm
    ///
    /// 1. Normalize ELP values to sum to 1.0
    /// 2. Calculate weighted position based on ELP triangle
    /// 3. Apply digital root reduction
    /// 4. Return position 0-9
    ///
    /// # Arguments
    ///
    /// * `ethos` - Ethos (character/moral) value (0.0-9.0)
    /// * `logos` - Logos (logic/reason) value (0.0-9.0)
    /// * `pathos` - Pathos (emotion/feeling) value (0.0-9.0)
    ///
    /// # Returns
    ///
    /// Flux position (0-9) where:
    /// - 0: Center/void
    /// - 1, 2, 4, 5, 7, 8: Regular vortex positions
    /// - 3, 6, 9: Sacred positions
    pub fn calculate_position_from_elp(&self, ethos: f32, logos: f32, pathos: f32) -> u8 {
        // Normalize ELP to sum to 1.0
        let sum = ethos + logos + pathos;
        let (e, l, p) = if sum > 0.0 {
            (ethos / sum, logos / sum, pathos / sum)
        } else {
            (1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0)
        };
        
        // Calculate weighted position (0.0 - 9.0 range)
        // High ethos → positions 1-3
        // High logos → positions 4-6
        // High pathos → positions 7-9
        // Balanced → position 0
        let raw_position = (e * 3.0 + l * 6.0 + p * 9.0).round() as u8;
        
        // Clamp to 0-9 range
        raw_position.min(9)
    }
    
    /// Find best flux position based on semantic similarity
    /// 
    /// # Arguments
    /// * `input` - Input text to match
    /// * `subject` - Subject domain (e.g., "consciousness", "ethics")
    /// 
    /// # Returns
    /// * `(position, confidence)` - Best matching position and confidence score
    pub fn find_best_position(
        &self,
        input: &str,
        subject: &str,
    ) -> Result<(u8, f32)> {
        // Get or create matrix for subject
        let matrix = self.create_matrix(subject.to_string())?;
        
        let mut best_position = 0;
        let mut best_score = 0.0;
        
        // Score each regular node's semantic fit
        for (position, node) in &matrix.nodes {
            let score = self.calculate_semantic_similarity(input, node);
            
            if score > best_score {
                best_score = score;
                best_position = *position;
            }
        }
        
        // Also check sacred guides (with 2.0x boost for stronger attraction)
        for (position, guide) in &matrix.sacred_guides {
            let score = self.calculate_sacred_similarity(input, guide);
            
            // Sacred positions attract high-quality patterns
            // Increased from 1.5x to 2.0x to better compete with strong regular matches
            let boosted_score = score * 2.0;
            
            if boosted_score > best_score {
                best_score = boosted_score;
                best_position = *position;
            }
        }
        
        // If no good match, use ELP fallback with neutral values
        if best_score < 0.3 {
            best_position = 0;  // Center position
            best_score = 0.3;
        }
        
        Ok((best_position, best_score))
    }
    
    /// Calculate semantic similarity between input and node
    fn calculate_semantic_similarity(
        &self,
        input: &str,
        node: &FluxNode,
    ) -> f32 {
        let input_lower = input.to_lowercase();
        let input_words: Vec<&str> = input_lower.split_whitespace().collect();
        let mut total_score = 0.0;
        let mut matches = 0;
        
        // Check positive associations (Heaven)
        for assoc in &node.semantic_index.positive_associations {
            let assoc_lower = assoc.word.to_lowercase();
            if input_words.iter().any(|w| w.contains(&assoc_lower) || assoc_lower.contains(w)) {
                total_score += assoc.confidence as f32;
                matches += 1;
            }
        }
        
        // Check negative associations (Hell) - penalize if present
        for assoc in &node.semantic_index.negative_associations {
            let assoc_lower = assoc.word.to_lowercase();
            if input_words.iter().any(|w| w.contains(&assoc_lower) || assoc_lower.contains(w)) {
                total_score -= assoc.confidence as f32 * 0.5;
            }
        }
        
        // Check neutral base (core meaning)
        let neutral_lower = node.semantic_index.neutral_base.to_lowercase();
        if input_words.iter().any(|w| w.contains(&neutral_lower) || neutral_lower.contains(w)) {
            total_score += 2.0;
            matches += 1;
        }
        
        // Normalize by number of matches
        if matches > 0 {
            (total_score / matches as f32).max(0.0).min(1.0)
        } else {
            0.0
        }
    }
    
    /// Calculate similarity with sacred guide
    fn calculate_sacred_similarity(
        &self,
        input: &str,
        guide: &SacredGuide,
    ) -> f32 {
        let input_lower = input.to_lowercase();
        let mut score: f32 = 0.0;
        
        // Bonus for fundamental/philosophical keywords (Position 9 boosters)
        let fundamental_keywords = ["nature", "essence", "fundamental", "ultimate", "divine", "absolute"];
        for keyword in &fundamental_keywords {
            if input_lower.contains(keyword) {
                score += 0.8;  // Strong boost for philosophical depth
            }
        }
        
        // Bonus for unity/integration keywords (Position 3 boosters)
        let unity_keywords = ["integrat", "unif", "whole", "brings together", "complete"];
        for keyword in &unity_keywords {
            if input_lower.contains(keyword) {
                score += 0.9;  // Very strong boost for unity queries
            }
        }
        
        // Bonus for heart/core keywords (Position 6 boosters)
        let heart_keywords = ["heart of", "core of", "center of", "soul of", "felt"];
        for keyword in &heart_keywords {
            if input_lower.contains(keyword) {
                score += 0.9;  // Very strong boost for emotional core queries
            }
        }
        
        // Check divine properties
        for property in &guide.divine_properties {
            let property_lower = property.to_lowercase();
            if input_lower.contains(&property_lower) {
                score += 1.5;
            }
        }
        
        // Check intersection points
        for intersection in &guide.intersection_points {
            let sig_lower = intersection.significance.to_lowercase();
            if input_lower.contains(&sig_lower) {
                score += 1.0;
            }
        }
        
        // Check geometric significance
        let geo_lower = guide.geometric_significance.to_lowercase();
        if input_lower.contains(&geo_lower) {
            score += 1.2;
        }
        
        score.min(1.0)
    }
    
    /// Validate position using 3-6-9 pattern coherence
    /// 
    /// # Returns
    /// * `(position, adjusted_confidence, is_sacred)` - Validated position with adjustments
    pub fn validate_position_coherence(
        &self,
        position: u8,
        confidence: f32,
    ) -> (u8, f32, bool) {
        let is_sacred = self.sacred_positions.contains(&position);
        
        if is_sacred {
            // Sacred positions get confidence boost
            let boosted_confidence = (confidence * 1.15).min(1.0);
            return (position, boosted_confidence, true);
        }
        
        // Check if position follows vortex flow pattern
        let in_vortex_flow = self.base_pattern.contains(&position);
        
        if !in_vortex_flow && position != 0 {
            // Position not in pattern - find nearest valid position
            let nearest = self.find_nearest_vortex_position(position);
            let adjusted_confidence = confidence * 0.9;
            return (nearest, adjusted_confidence, false);
        }
        
        (position, confidence, false)
    }
    
    /// Find nearest position in vortex flow
    fn find_nearest_vortex_position(&self, position: u8) -> u8 {
        let mut min_distance = u8::MAX;
        let mut nearest = 1;
        
        for &flow_pos in &self.base_pattern {
            let distance = if position > flow_pos {
                position - flow_pos
            } else {
                flow_pos - position
            };
            
            if distance < min_distance {
                min_distance = distance;
                nearest = flow_pos;
            }
        }
        
        nearest
    }
}
