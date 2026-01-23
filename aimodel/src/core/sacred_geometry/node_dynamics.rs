//! Dynamic Node Evaluation and Intelligence
//!
//! Loop-aware, order-conscious, and object-relative evaluation for FluxNode.

use chrono::Utc;

use crate::data::models::{
    AttributeChannel, ConfidenceSnapshot, ELPChannel, FluxNode, InteractionPattern, InteractionType,
    ObjectContext, OrderRole, VortexPosition,
};

/// Result of evaluating an object at a node
#[derive(Debug, Clone)]
pub struct EvaluationResult {
    pub should_accept: bool,
    pub confidence: f32,
    pub fit_score: f32,
    pub suggested_adjustments: Vec<String>,
}

/// Extensions for FluxNode to support dynamic evaluation
pub trait FluxNodeDynamics {
    /// Initialize node dynamics based on position in the matrix and subject context
    /// Roles and channels are determined by the subject's journey through the vortex pattern
    fn initialize_dynamics(&mut self, subject_context: Option<&str>);
    
    /// Evaluate object relative to this node's position and role
    fn evaluate_object(&mut self, object: &ObjectContext) -> EvaluationResult;
    
    /// Advance vortex position tracking
    fn advance_vortex_position(&mut self);
    
    /// Calculate semantic fit score
    fn calculate_semantic_fit(&self, object: &ObjectContext) -> f32;
    
    /// Calculate attribute fit score based on channel alignment
    fn calculate_attribute_fit(&self, object: &ObjectContext) -> f32;
    
    /// Calculate position fit score
    fn calculate_position_fit(&self, object: &ObjectContext) -> f32;
    
    /// Record interaction pattern
    fn record_interaction(&mut self, object: &ObjectContext);
    
    /// Update stability based on fit score
    fn update_stability(&mut self, fit_score: f32);
    
    /// Suggest adjustments based on evaluation
    fn suggest_adjustments(&self) -> Vec<String>;
    
    /// Process the object flowing through this node, modifying it based on node attributes and governors
    fn process_flow(&self, object: &mut ObjectContext);
}

impl FluxNodeDynamics for FluxNode {
    fn initialize_dynamics(&mut self, subject_context: Option<&str>) {
        // Roles and channels are NOT hardcoded - they're determined by:
        // 1. The subject's semantic meaning
        // 2. The predefined sequence for achieving the goal
        // 3. The vortex flow pattern (1→2→4→8→7→5→1)
        // 4. Sacred checkpoints (3, 6, 9) for validation
        
        // Sacred positions (3, 6, 9) are structural guides, not semantic roles
        let is_sacred = matches!(self.position, 3 | 6 | 9);
        self.attributes.dynamics.is_sacred = is_sacred;
        self.attributes.dynamics.sacred_multiplier = if is_sacred { 2.0 } else { 1.0 };

        // Initialize vortex position tracking
        self.advance_vortex_position();
        
        // Set initial sequence confidence
        self.attributes.dynamics.sequence_confidence = 0.5;
        
        // OrderRole and AttributeChannel should be set by the subject domain
        // based on what the node represents in the context of achieving a goal
        // Default to neutral if no context provided
        if subject_context.is_none() {
            self.attributes.dynamics.order_role = OrderRole::Center;
            self.attributes.dynamics.attribute_channel = AttributeChannel::Neutral;
        } 
    }

    /// Process object flow - Apply attributes and Governor influence
    fn process_flow(&self, object: &mut ObjectContext) {
        // 1. Apply Node's inherent attributes (Channel influence)
        // Nodes impart their "flavor" to the object via attribute modulation
        apply_channel_influence(&mut object.attributes, &self.attributes.dynamics.attribute_channel, 0.5);
        
        // 2. Apply Governor Influence (3, 6, 9)
        // Governors facilitate realtime dynamics based on geometric proximity on the circle
        // 3 governs neighbors 2 and 4
        // 6 governs neighbors 5 and 7
        // 9 governs neighbors 8 and 1
        let governor_pos = match self.position {
            2 | 4 => 3, // Governed by Creative Trinity (3)
            5 | 7 => 6, // Governed by Harmonic Balance (6)
            8 | 1 => 9, // Governed by Divine Completion (9)
            _ => 0,     // Self-governed (3, 6, 9) or Center (0)
        };
        
        if governor_pos != 0 {
            // In a full graph system, we would look up the Governor node here.
            // Since we are in a single node context, we simulate the Governor's "Name/Idea" influence.
            
            let governor_influence = match governor_pos {
                3 => ("Creative Trinity", 0.2, 0.1, 0.1), // Boost Ethos
                6 => ("Harmonic Balance", 0.1, 0.1, 0.2), // Boost Pathos/Balance
                9 => ("Divine Completion", 0.1, 0.2, 0.1), // Boost Logos/Wisdom
                _ => ("Unknown", 0.0, 0.0, 0.0),
            };
            
            // "Realtime dynamics for its own represented idea via its name"
            // We simulate this by adding a semantic tag to the object
            let tag = format!("Governed by {}: {}", governor_pos, governor_influence.0);
            if !object.keywords.contains(&tag) {
                object.keywords.push(tag);
            }
            
            // Apply modulation via attributes
            modulate_attributes(&mut object.attributes, governor_influence.1, governor_influence.2, governor_influence.3);
        }
        
        // 3. Ensure conservation of energy (normalization not strictly required here but good for stability)
        // We allow values to grow (accumulation of wisdom) but clamp to reasonable bounds
        clamp_attributes(&mut object.attributes, 13.0);
    }

    fn evaluate_object(&mut self, object: &ObjectContext) -> EvaluationResult {
        // 1. Calculate fit scores first (immutable borrows)
        let semantic_fit = self.calculate_semantic_fit(object);
        let attribute_fit = self.calculate_attribute_fit(object);
        let position_fit = self.calculate_position_fit(object);
        let fit_score = (semantic_fit + attribute_fit + position_fit) / 3.0;
        
        // 2. Update current object context and apply adjustments (scoped borrow)
        let (confidence, final_fit_score) = {
            let dynamics = &mut self.attributes.dynamics;
            dynamics.current_object = Some(object.clone());
            dynamics.last_evaluation = Some(Utc::now());
            dynamics.object_fit_score = fit_score;
            
            // 3. Apply role-based adjustments
            let mut confidence = semantic_fit;
            
            match dynamics.order_role {
                OrderRole::SacredEthos | OrderRole::SacredPathos | OrderRole::SacredLogos => {
                    // Sacred positions: Boost if object contains sacred keywords
                    let is_fundamental = object.keywords.iter()
                        .any(|k| ["fundamental", "ultimate", "essence", "divine", "absolute"]
                            .contains(&k.to_lowercase().as_str()));
                    
                    if is_fundamental {
                        confidence *= dynamics.sacred_multiplier;
                    }
                },
                OrderRole::Power => {
                    // Logos position: Boost for logical/rational queries
                    if object.attributes.logos() > object.attributes.ethos() 
                        && object.attributes.logos() > object.attributes.pathos() {
                        confidence *= 1.2;
                    }
                },
                OrderRole::Change => {
                    // Pathos position: Boost for emotional queries
                    if object.attributes.pathos() > object.attributes.ethos() 
                        && object.attributes.pathos() > object.attributes.logos() {
                        confidence *= 1.2;
                    }
                },
                OrderRole::Beginning => {
                    // Ethos position: Boost for identity/character queries
                    if object.attributes.ethos() > object.attributes.logos() 
                        && object.attributes.ethos() > object.attributes.pathos() {
                        confidence *= 1.15;
                    }
                },
                _ => {}
            }
            
            // Clamp confidence to [0.0, 1.0]
            confidence = confidence.min(1.0).max(0.0);
            dynamics.object_confidence = confidence;
            
            // 4. Store confidence snapshot
            let order_role_str = format!("{:?}", dynamics.order_role);
            dynamics.confidence_history.push(ConfidenceSnapshot {
                timestamp: Utc::now(),
                confidence,
                object_type: object.subject.clone(),
                adjustment_applied: Some(order_role_str),
            });
            
            // Keep history limited to last 100 evaluations
            if dynamics.confidence_history.len() > 100 {
                dynamics.confidence_history.drain(0..50);
            }
            
            (confidence, dynamics.object_fit_score)
        }; // dynamics borrow ends here
        
        // 5. Record interaction (now we can mutably borrow self again)
        self.record_interaction(object);
        
        // 6. Update stability
        self.update_stability(final_fit_score);
        
        EvaluationResult {
            should_accept: confidence > 0.6,
            confidence,
            fit_score: final_fit_score,
            suggested_adjustments: self.suggest_adjustments(),
        }
    }
    
    fn advance_vortex_position(&mut self) {
        let dynamics = &mut self.attributes.dynamics;
        
        dynamics.vortex_position = match self.position {
            1 => VortexPosition::Position1,
            2 => VortexPosition::Position2,
            4 => VortexPosition::Position4,
            8 => VortexPosition::Position8,
            7 => VortexPosition::Position7,
            5 => {
                // Completed loop - increment iteration
                dynamics.loop_iteration += 1;
                VortexPosition::Position5
            },
            _ => VortexPosition::LoopComplete,
        };
        
        // Update sequence confidence based on loop consistency
        if dynamics.loop_iteration > 0 {
            // Confidence improves with each completed loop
            dynamics.sequence_confidence = 1.0 - (1.0 / (dynamics.loop_iteration as f32 + 1.0));
        }
    }
    
    fn calculate_semantic_fit(&self, object: &ObjectContext) -> f32 {
        // Semantic fit based on keyword matches
        // Higher semantic_matches = better fit
        let base_fit = (object.semantic_matches as f32 / 10.0).min(1.0);
        
        // Adjust based on position role
        let role_modifier = match self.attributes.dynamics.order_role {
            OrderRole::SacredEthos | OrderRole::SacredPathos | OrderRole::SacredLogos => 1.1,
            OrderRole::Center => 0.9, // Center is more general
            _ => 1.0,
        };
        
        (base_fit * role_modifier).min(1.0)
    }
    
    fn calculate_attribute_fit(&self, object: &ObjectContext) -> f32 {
        let channel = &self.attributes.dynamics.attribute_channel;
        
        match channel {
            AttributeChannel::Ethos => {
                // Higher ethos = better fit
                normalize_attribute_value(object.attributes.ethos())
            },
            AttributeChannel::Logos => {
                // Higher logos = better fit
                normalize_attribute_value(object.attributes.logos())
            },
            AttributeChannel::Pathos => {
                // Higher pathos = better fit
                normalize_attribute_value(object.attributes.pathos())
            },
            AttributeChannel::Mixed => {
                // Average of all three channels
                let avg = (normalize_attribute_value(object.attributes.ethos()) 
                         + normalize_attribute_value(object.attributes.logos())
                         + normalize_attribute_value(object.attributes.pathos())) / 3.0;
                avg
            },
            AttributeChannel::Neutral => {
                // Neutral doesn't prefer any channel
                0.5
            },
        }
    }
    
    fn calculate_position_fit(&self, object: &ObjectContext) -> f32 {
        // Position fit based on query characteristics matching position role
        match self.attributes.dynamics.order_role {
            OrderRole::Beginning => {
                // Good for "what is", "who am I" queries
                if object.query.to_lowercase().contains("what is")
                    || object.query.to_lowercase().contains("who")
                    || object.query.to_lowercase().contains("self") {
                    0.9
                } else {
                    0.6
                }
            },
            OrderRole::Power => {
                // Good for "how does", "why" queries (analytical)
                if object.query.to_lowercase().contains("how")
                    || object.query.to_lowercase().contains("why")
                    || object.query.to_lowercase().contains("logic") {
                    0.9
                } else {
                    0.6
                }
            },
            OrderRole::Wisdom => {
                // Good for "what should", "meaning" queries
                if object.query.to_lowercase().contains("should")
                    || object.query.to_lowercase().contains("meaning")
                    || object.query.to_lowercase().contains("understand") {
                    0.9
                } else {
                    0.6
                }
            },
            OrderRole::SacredEthos | OrderRole::SacredPathos | OrderRole::SacredLogos => {
                // Sacred positions: Good for fundamental questions
                if object.keywords.iter().any(|k| 
                    ["fundamental", "ultimate", "nature", "essence"].contains(&k.to_lowercase().as_str())
                ) {
                    0.95
                } else {
                    0.7
                }
            },
            _ => 0.7, // Default moderate fit
        }
    }
    
    fn record_interaction(&mut self, _object: &ObjectContext) {
        let dynamics = &mut self.attributes.dynamics;
        let current_time = Utc::now();
        
        // Determine interaction type based on position
        let interaction_type = if dynamics.is_sacred {
            InteractionType::SacredCheckpoint
        } else if matches!(dynamics.vortex_position, 
            VortexPosition::Position1 | VortexPosition::Position2 | 
            VortexPosition::Position4 | VortexPosition::Position8 |
            VortexPosition::Position7 | VortexPosition::Position5) {
            InteractionType::VortexFlow
        } else {
            InteractionType::CrossSubject
        };
        
        // Find or create interaction pattern for this subject
        if let Some(pattern) = dynamics.interaction_patterns.iter_mut()
            .find(|p| p.with_position == self.position) {
            // Update existing pattern
            pattern.frequency += 1;
            pattern.avg_confidence = (pattern.avg_confidence * (pattern.frequency as f32 - 1.0) 
                                     + dynamics.object_confidence) / pattern.frequency as f32;
            pattern.last_interaction = current_time;
        } else {
            // Create new pattern
            dynamics.interaction_patterns.push(InteractionPattern {
                with_position: self.position,
                interaction_type,
                frequency: 1,
                avg_confidence: dynamics.object_confidence,
                last_interaction: current_time,
            });
        }
        
        // Keep only recent patterns (last 50)
        if dynamics.interaction_patterns.len() > 50 {
            dynamics.interaction_patterns.sort_by(|a, b| 
                b.last_interaction.cmp(&a.last_interaction)
            );
            dynamics.interaction_patterns.truncate(50);
        }
    }
    
    fn update_stability(&mut self, fit_score: f32) {
        let dynamics = &mut self.attributes.dynamics;
        
        // Stability improves with good fits, decreases with poor fits
        let adjustment = if fit_score > 0.7 {
            0.02 // Slowly increase stability with good matches
        } else if fit_score < 0.4 {
            -0.05 // Decrease stability faster with poor matches
        } else {
            0.0
        };
        
        dynamics.stability_index = (dynamics.stability_index + adjustment).clamp(0.0, 1.0);
        
        // Sacred positions have minimum stability of 0.6
        if dynamics.is_sacred && dynamics.stability_index < 0.6 {
            dynamics.stability_index = 0.6;
        }
    }
    
    fn suggest_adjustments(&self) -> Vec<String> {
        let mut suggestions = Vec::new();
        let dynamics = &self.attributes.dynamics;
        
        // Suggest based on stability
        if dynamics.stability_index < 0.4 {
            suggestions.push("Consider re-evaluating semantic associations".to_string());
        }
        
        // Suggest based on confidence
        if dynamics.object_confidence < 0.5 {
            suggestions.push(format!(
                "Low confidence ({:.2}) - may not be best position for this query",
                dynamics.object_confidence
            ));
        }
        
        // Suggest based on fit score
        if dynamics.object_fit_score < 0.6 {
            suggestions.push("Object characteristics don't align well with position role".to_string());
        }
        
        // Suggest loop continuation
        if matches!(dynamics.vortex_position, VortexPosition::Position5) {
            suggestions.push(format!(
                "Loop iteration {} complete - consider sacred checkpoint",
                dynamics.loop_iteration
            ));
        }
        
        suggestions
    }
}

/// Apply channel influence to attributes based on channel type
fn apply_channel_influence(attributes: &mut crate::data::attributes::Attributes, channel: &AttributeChannel, amount: f32) {
    use crate::data::attributes::AttributeValue;
    
    match channel {
        AttributeChannel::Ethos => {
            let current = attributes.ethos();
            attributes.set("ethos", AttributeValue::Number((current + amount) as f64));
        },
        AttributeChannel::Logos => {
            let current = attributes.logos();
            attributes.set("logos", AttributeValue::Number((current + amount) as f64));
        },
        AttributeChannel::Pathos => {
            let current = attributes.pathos();
            attributes.set("pathos", AttributeValue::Number((current + amount) as f64));
        },
        AttributeChannel::Mixed => {
            let mixed_amount = amount * 0.4; // Distribute evenly
            let ethos = attributes.ethos();
            let logos = attributes.logos();
            let pathos = attributes.pathos();
            attributes.set("ethos", AttributeValue::Number((ethos + mixed_amount) as f64));
            attributes.set("logos", AttributeValue::Number((logos + mixed_amount) as f64));
            attributes.set("pathos", AttributeValue::Number((pathos + mixed_amount) as f64));
        },
        _ => {},
    }
}

/// Modulate attributes by specific amounts for each channel
fn modulate_attributes(attributes: &mut crate::data::attributes::Attributes, ethos_delta: f32, logos_delta: f32, pathos_delta: f32) {
    use crate::data::attributes::AttributeValue;
    
    let ethos = attributes.ethos() + ethos_delta;
    let logos = attributes.logos() + logos_delta;
    let pathos = attributes.pathos() + pathos_delta;
    
    attributes.set("ethos", AttributeValue::Number(ethos as f64));
    attributes.set("logos", AttributeValue::Number(logos as f64));
    attributes.set("pathos", AttributeValue::Number(pathos as f64));
}

/// Clamp all attribute channels to maximum value
fn clamp_attributes(attributes: &mut crate::data::attributes::Attributes, max_value: f32) {
    use crate::data::attributes::AttributeValue;
    
    let ethos = attributes.ethos().min(max_value);
    let logos = attributes.logos().min(max_value);
    let pathos = attributes.pathos().min(max_value);
    
    attributes.set("ethos", AttributeValue::Number(ethos as f64));
    attributes.set("logos", AttributeValue::Number(logos as f64));
    attributes.set("pathos", AttributeValue::Number(pathos as f64));
}

/// Helper: Normalize attribute value from [-13, +13] to [0, 1]
fn normalize_attribute_value(value: f32) -> f32 {
    ((value + 13.0) / 26.0).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::models::{NodeAttributes, NodeState, SemanticIndex};
    use std::collections::HashMap;

    fn create_test_node(position: u8) -> FluxNode {
        FluxNode {
            position,
            base_value: position,
            semantic_index: SemanticIndex {
                positive_associations: Vec::new(),
                negative_associations: Vec::new(),
                neutral_base: format!("test_{}", position),
                predicates: Vec::new(),
                relations: Vec::new(),
            },
            attributes: NodeAttributes {
                properties: HashMap::new(),
                parameters: HashMap::new(),
                state: NodeState {
                    active: true,
                    last_accessed: Utc::now(),
                    usage_count: 0,
                    context_stack: Vec::new(),
                },
                dynamics: Default::default(),
            },
            connections: Vec::new(),
        }
    }

    #[test]
    fn test_initialize_dynamics() {
        let mut node = create_test_node(3);
        node.initialize_dynamics(None);

        // Sacred position is structural, not semantic
        assert!(node.attributes.dynamics.is_sacred);
        assert_eq!(node.attributes.dynamics.sacred_multiplier, 2.0);
        
        // Roles should be neutral without subject context
        assert_eq!(node.attributes.dynamics.order_role, OrderRole::Center);
        assert_eq!(node.attributes.dynamics.attribute_channel, AttributeChannel::Neutral);
    }

    #[test]
    fn test_vortex_position_advancement() {
        let mut node = create_test_node(5);
        node.initialize_dynamics(None);

        assert_eq!(node.attributes.dynamics.vortex_position, VortexPosition::Position5);
        assert_eq!(node.attributes.dynamics.loop_iteration, 1);
    }

    #[test]
    fn test_sacred_position_detection() {
        // Test that sacred positions (3, 6, 9) are correctly identified
        let sacred_positions = vec![3, 6, 9];
        let non_sacred_positions = vec![0, 1, 2, 4, 5, 7, 8];
        
        for pos in sacred_positions {
            let mut node = create_test_node(pos);
            node.initialize_dynamics(None);
            assert!(node.attributes.dynamics.is_sacred, "Position {} should be sacred", pos);
            assert_eq!(node.attributes.dynamics.sacred_multiplier, 2.0);
        }
        
        for pos in non_sacred_positions {
            let mut node = create_test_node(pos);
            node.initialize_dynamics(None);
            assert!(!node.attributes.dynamics.is_sacred, "Position {} should not be sacred", pos);
            assert_eq!(node.attributes.dynamics.sacred_multiplier, 1.0);
        }
    }

    #[test]
    fn test_process_flow_governance() {
        use crate::data::models::ObjectContext;
        use crate::data::attributes::Attributes;
        
        // Create a node at Position 1 in vortex flow
        // Governed by Position 9 (sacred checkpoint) due to geometric proximity
        let mut node = create_test_node(1);
        node.initialize_dynamics(None);
        
        // Set a channel for testing (normally set by subject context)
        node.attributes.dynamics.attribute_channel = AttributeChannel::Ethos;
        
        let mut attributes = Attributes::with_elp(1.0, 1.0, 1.0);
        let mut object = ObjectContext {
            query: "Test Query".to_string(),
            subject: "Test Subject".to_string(),
            attributes,
            keywords: vec![],
            semantic_matches: 0,
            timestamp: Utc::now(),
        };
        
        // Apply flow dynamics
        node.process_flow(&mut object);
        
        // Check 1: Node 1 is Ethos channel, so Ethos should increase by 0.5
        // Check 2: Governor 9 adds 0.1 Ethos, 0.2 Logos, 0.1 Pathos
        // Total Ethos change: +0.5 + 0.1 = +0.6
        // Total Logos change: +0.2
        assert!((object.attributes.ethos() - 1.6).abs() < 0.001);
        assert!((object.attributes.logos() - 1.2).abs() < 0.001);
        
        // Check 3: Tag added
        assert!(object.keywords.iter().any(|k| k.contains("Governed by 9")));
        assert!(object.keywords.iter().any(|k| k.contains("Divine Completion")));
    }
}
