//! FluxTransformer - Multi-Layer FluxMatrix Processing
//!
//! Implements transformer-like architecture using stacked FluxMatrices.
//! Each layer refines understanding through the sacred geometry pattern.
//!
//! ## Architecture
//!
//! ```text
//! Input
//!   ↓
//! Layer 1 (Subject Matrix) → Position 1 + ELP₁
//!   ↓
//! Layer 2 (Refined Context) → Position 2 + ELP₂
//!   ↓
//! Layer 3 (Deep Pattern) → Position 3 + ELP₃
//!   ↓
//! Output (Integrated Understanding)
//! ```
//!
//! ## Key Concepts
//!
//! - **Layer Attention**: Each layer attends to previous layer's sacred positions
//! - **Pattern Resonance**: 3-6-9 positions amplify across layers
//! - **Vortex Flow**: Information flows through 1→2→4→8→7→5→1 pattern
//! - **Sacred Checkpoints**: Positions 3, 6, 9 serve as cross-layer anchors

use crate::data::models::*;
use crate::data::attributes::Attributes;
use crate::error::{Result, SpatialVortexError};
use crate::core::sacred_geometry::flux_matrix::FluxMatrixEngine;
use crate::core::sacred_geometry::node_dynamics::FluxNodeDynamics;
use crate::core::sacred_geometry::object_utils::create_object_context;
use std::collections::HashMap;

/// Multi-layer FluxMatrix transformer
#[derive(Clone, Debug)]
pub struct FluxTransformer {
    /// Number of layers
    num_layers: usize,
    
    /// Flux engine for matrix operations
    flux_engine: FluxMatrixEngine,
    
    /// Whether to use sacred attention (focus on 3-6-9)
    sacred_attention: bool,
    
    /// Whether to use residual connections
    residual_connections: bool,
}

impl FluxTransformer {
    /// Create new transformer with specified number of layers
    pub fn new(num_layers: usize) -> Self {
        Self {
            num_layers,
            flux_engine: FluxMatrixEngine::new(),
            sacred_attention: true,
            residual_connections: true,
        }
    }
    
    /// Process input through multiple FluxMatrix layers
    /// 
    /// # Arguments
    /// * `input` - Input text
    /// * `subject` - Subject domain
    /// 
    /// # Returns
    /// * `TransformerOutput` - Multi-layer processed result
    pub fn process(&self, input: &str, subject: &str) -> Result<TransformerOutput> {
        let mut layers = Vec::new();
        let mut current_input = input.to_string();
        let mut cumulative_elp = ELPTensor { ethos: 0.0, logos: 0.0, pathos: 0.0 };
        
        // Process through each layer
        for layer_idx in 0..self.num_layers {
            let layer_result = self.process_layer(
                &current_input,
                subject,
                layer_idx,
                &cumulative_elp,
                if layer_idx > 0 { Some(&layers) } else { None },
            )?;
            
            // Accumulate ELP (residual connection)
            if self.residual_connections {
                cumulative_elp.ethos += layer_result.elp.ethos;
                cumulative_elp.logos += layer_result.elp.logos;
                cumulative_elp.pathos += layer_result.elp.pathos;
            } else {
                cumulative_elp = layer_result.elp.clone();
            }
            
            // Update input for next layer (use position knowledge)
            current_input = self.create_refined_input(&current_input, &layer_result);
            
            layers.push(layer_result);
        }
        
        // Integrate all layers
        let final_output = self.integrate_layers(&layers, &cumulative_elp)?;
        
        Ok(TransformerOutput {
            layers,
            final_position: final_output.position,
            final_elp: cumulative_elp,
            final_confidence: final_output.confidence,
            sacred_resonance: final_output.sacred_resonance,
            pattern_coherence: final_output.pattern_coherence,
        })
    }
    
    /// Process a single layer with dynamic node evaluation
    fn process_layer(
        &self,
        input: &str,
        subject: &str,
        layer_idx: usize,
        cumulative_elp: &ELPTensor,
        previous_layers: Option<&Vec<LayerOutput>>,
    ) -> Result<LayerOutput> {
        // Find best position for this layer
        let (position, semantic_conf) = self.flux_engine
            .find_best_position(input, subject)?;
        
        // Validate with pattern coherence
        let (validated_pos, adjusted_conf, is_sacred) = self.flux_engine
            .validate_position_coherence(position, semantic_conf);
        
        // Calculate layer ELP (influenced by position and previous context)
        let layer_elp = self.calculate_layer_elp(
            validated_pos,
            cumulative_elp,
            layer_idx,
        );
        
        // DYNAMIC EVALUATION: Create object context and evaluate with node
        let attributes = Attributes::with_elp(layer_elp.ethos as f32, layer_elp.logos as f32, layer_elp.pathos as f32);
        let object = create_object_context(input, subject, attributes);
        
        // Get the node at this position and evaluate dynamically
        let mut node_confidence = adjusted_conf;
        if let Ok(mut node) = self.flux_engine.create_flux_node(validated_pos, subject) {
            let eval_result = node.evaluate_object(&object);
            
            // Use dynamic confidence from node evaluation
            node_confidence = eval_result.confidence;
            
            // Advance vortex position for next evaluation
            node.advance_vortex_position();
        }
        
        // Apply attention to previous layers' sacred positions
        let attention_boost = if self.sacred_attention && previous_layers.is_some() {
            self.calculate_sacred_attention(validated_pos, previous_layers.unwrap())
        } else {
            1.0
        };
        
        // Calculate pattern resonance across vortex flow
        let pattern_resonance = self.calculate_pattern_resonance(
            validated_pos,
            previous_layers,
        );
        
        Ok(LayerOutput {
            layer_index: layer_idx,
            position: validated_pos,
            is_sacred,
            semantic_confidence: semantic_conf,
            adjusted_confidence: node_confidence * attention_boost,
            elp: layer_elp,
            attention_weights: self.get_attention_weights(validated_pos, previous_layers),
            pattern_resonance,
        })
    }
    
    /// Calculate sacred attention weights from previous layers
    /// 
    /// Sacred positions in previous layers get higher attention weight
    fn calculate_sacred_attention(
        &self,
        current_position: u8,
        previous_layers: &[LayerOutput],
    ) -> f32 {
        let mut attention_sum = 1.0;
        
        for prev_layer in previous_layers {
            if prev_layer.is_sacred {
                // Sacred positions in previous layers amplify current position
                // Especially if current position is also sacred (resonance)
                let resonance_multiplier = if [3, 6, 9].contains(&current_position) {
                    1.5  // Sacred-to-sacred resonance
                } else {
                    1.2  // Sacred-to-regular amplification
                };
                
                attention_sum += prev_layer.adjusted_confidence * resonance_multiplier;
            }
        }
        
        (attention_sum / (previous_layers.len() as f32 + 1.0)).min(2.0)  // Cap at 2x boost
    }
    
    /// Calculate ELP for current layer based on position and context
    fn calculate_layer_elp(
        &self,
        position: u8,
        cumulative_elp: &ELPTensor,
        _layer_idx: usize,
    ) -> ELPTensor {
        // Base ELP from position
        let base_elp = match position {
            0 => ELPTensor { ethos: 0.0, logos: 0.0, pathos: 0.0 },
            1 => ELPTensor { ethos: 5.0, logos: 3.0, pathos: 2.0 },
            2 => ELPTensor { ethos: 3.0, logos: 4.0, pathos: 3.0 },
            3 => ELPTensor { ethos: 9.0, logos: 6.0, pathos: 3.0 },
            4 => ELPTensor { ethos: 4.0, logos: 6.0, pathos: 3.0 },
            5 => ELPTensor { ethos: 3.0, logos: 3.0, pathos: 7.0 },
            6 => ELPTensor { ethos: 3.0, logos: 6.0, pathos: 9.0 },
            7 => ELPTensor { ethos: 5.0, logos: 7.0, pathos: 4.0 },
            8 => ELPTensor { ethos: 6.0, logos: 6.0, pathos: 5.0 },
            9 => ELPTensor { ethos: 6.0, logos: 9.0, pathos: 6.0 },
            _ => ELPTensor { ethos: 0.0, logos: 0.0, pathos: 0.0 },
        };
        
        // Blend with cumulative ELP (residual connection)
        let blend_factor = 0.3;  // 30% previous, 70% current
        ELPTensor {
            ethos: base_elp.ethos * (1.0 - blend_factor) + cumulative_elp.ethos * blend_factor,
            logos: base_elp.logos * (1.0 - blend_factor) + cumulative_elp.logos * blend_factor,
            pathos: base_elp.pathos * (1.0 - blend_factor) + cumulative_elp.pathos * blend_factor,
        }
    }
    
    /// Calculate pattern resonance through vortex flow
    fn calculate_pattern_resonance(
        &self,
        position: u8,
        previous_layers: Option<&Vec<LayerOutput>>,
    ) -> f32 {
        if previous_layers.is_none() {
            return 1.0;
        }
        
        let vortex_flow = [1, 2, 4, 8, 7, 5];
        let mut resonance = 1.0;
        
        // Check if current position follows vortex flow from previous
        if let Some(layers) = previous_layers {
            if let Some(last_layer) = layers.last() {
                if let Some(current_idx) = vortex_flow.iter().position(|&p| p == position) {
                    if let Some(prev_idx) = vortex_flow.iter().position(|&p| p == last_layer.position) {
                        // Check if following forward flow
                        if (prev_idx + 1) % vortex_flow.len() == current_idx {
                            resonance += 0.5;  // Forward flow bonus
                        }
                    }
                }
            }
        }
        
        resonance
    }
    
    /// Get attention weights for all positions
    fn get_attention_weights(
        &self,
        current_position: u8,
        previous_layers: Option<&Vec<LayerOutput>>,
    ) -> HashMap<u8, f32> {
        let mut weights = HashMap::new();
        
        // Self-attention weight
        weights.insert(current_position, 1.0);
        
        if let Some(layers) = previous_layers {
            for layer in layers {
                // Sacred positions get higher weight
                let weight = if layer.is_sacred { 1.5 } else { 1.0 };
                *weights.entry(layer.position).or_insert(0.0) += weight;
            }
        }
        
        // Normalize weights
        let sum: f32 = weights.values().sum();
        if sum > 0.0 {
            for value in weights.values_mut() {
                *value /= sum;
            }
        }
        
        weights
    }
    
    /// Create refined input for next layer
    fn create_refined_input(
        &self,
        original_input: &str,
        _layer_result: &LayerOutput,
    ) -> String {
        // For now, return original input
        // TODO: Enhance with layer insights
        original_input.to_string()
    }
    
    /// Integrate all layers into final output
    fn integrate_layers(
        &self,
        layers: &[LayerOutput],
        _cumulative_elp: &ELPTensor,
    ) -> Result<IntegratedOutput> {
        // Find strongest layer (highest confidence)
        let strongest_layer = layers
            .iter()
            .max_by(|a, b| a.adjusted_confidence.partial_cmp(&b.adjusted_confidence).unwrap())
            .ok_or_else(|| SpatialVortexError::Processing("No layers to integrate".into()))?;
        
        // Calculate sacred resonance (how many sacred positions hit)
        let sacred_count = layers.iter().filter(|l| l.is_sacred).count();
        let sacred_resonance = (sacred_count as f32 / layers.len() as f32) * 2.0;  // 0-2 range
        
        // Calculate overall pattern coherence
        let avg_resonance: f32 = layers.iter()
            .map(|l| l.pattern_resonance)
            .sum::<f32>() / layers.len() as f32;
        
        // Final confidence is weighted average with emphasis on sacred layers
        let final_confidence = layers.iter()
            .map(|l| {
                let weight = if l.is_sacred { 1.5 } else { 1.0 };
                l.adjusted_confidence * weight
            })
            .sum::<f32>() / layers.iter()
            .map(|l| if l.is_sacred { 1.5 } else { 1.0 })
            .sum::<f32>();
        
        Ok(IntegratedOutput {
            position: strongest_layer.position,
            confidence: final_confidence.min(1.0),
            sacred_resonance: sacred_resonance.min(2.0),
            pattern_coherence: avg_resonance,
        })
    }
}

/// Output from a single transformer layer
#[derive(Clone, Debug)]
pub struct LayerOutput {
    pub layer_index: usize,
    pub position: u8,
    pub is_sacred: bool,
    pub semantic_confidence: f32,
    pub adjusted_confidence: f32,
    pub elp: ELPTensor,
    pub attention_weights: HashMap<u8, f32>,
    pub pattern_resonance: f32,
}

/// Integrated output from all layers
#[derive(Clone, Debug)]
struct IntegratedOutput {
    position: u8,
    confidence: f32,
    sacred_resonance: f32,
    pattern_coherence: f32,
}

/// Final transformer output
#[derive(Clone, Debug)]
pub struct TransformerOutput {
    pub layers: Vec<LayerOutput>,
    pub final_position: u8,
    pub final_elp: ELPTensor,
    pub final_confidence: f32,
    pub sacred_resonance: f32,  // 0-2: measures sacred position hits
    pub pattern_coherence: f32,  // 0-2: measures vortex flow alignment
}
