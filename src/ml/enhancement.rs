//! Machine Learning Enhancement for Geometric Inference
//!
//! Implements ensemble learning combining rule-based inference with ML predictions
//! to achieve 95%+ accuracy target.

use crate::geometric_inference::{GeometricInferenceEngine, GeometricInput, GeometricTaskType};
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Training sample for ML model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingSample {
    /// Input features
    pub angle: f64,
    pub distance: f64,
    pub complexity: f64,
    pub task_type: String,
    
    /// Target output
    pub correct_position: u8,
    
    /// Rule-based prediction (for comparison)
    pub rule_based_prediction: u8,
    
    /// Whether rule-based was correct
    pub rule_based_correct: bool,
}

/// Simple decision tree node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecisionNode {
    /// Leaf node with predicted position
    Leaf { position: u8, confidence: f64, sample_count: usize },
    
    /// Decision node that splits on a feature
    Split {
        feature: Feature,
        threshold: f64,
        left: Box<DecisionNode>,
        right: Box<DecisionNode>,
    },
}

/// Features for decision tree splits
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Feature {
    Angle,
    Distance,
    Complexity,
    TaskType,
    RuleBasedPrediction,
}

/// Decision tree classifier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionTree {
    root: DecisionNode,
    max_depth: usize,
    min_samples_split: usize,
}

impl DecisionTree {
    /// Create new decision tree with parameters
    pub fn new(max_depth: usize, min_samples_split: usize) -> Self {
        Self {
            root: DecisionNode::Leaf { 
                position: 0, 
                confidence: 0.0, 
                sample_count: 0 
            },
            max_depth,
            min_samples_split,
        }
    }
    
    /// Train decision tree on samples
    pub fn train(&mut self, samples: &[TrainingSample]) -> Result<()> {
        if samples.is_empty() {
            return Ok(());
        }
        
        self.root = self.build_tree(samples, 0)?;
        Ok(())
    }
    
    /// Recursively build decision tree
    fn build_tree(&self, samples: &[TrainingSample], depth: usize) -> Result<DecisionNode> {
        // Stopping criteria
        if depth >= self.max_depth || samples.len() < self.min_samples_split {
            return Ok(self.create_leaf(samples));
        }
        
        // Find best split
        let best_split = self.find_best_split(samples)?;
        
        if best_split.is_none() {
            return Ok(self.create_leaf(samples));
        }
        
        let (feature, threshold, left_samples, right_samples) = best_split.unwrap();
        
        // Recursively build subtrees
        let left = Box::new(self.build_tree(&left_samples, depth + 1)?);
        let right = Box::new(self.build_tree(&right_samples, depth + 1)?);
        
        Ok(DecisionNode::Split {
            feature,
            threshold,
            left,
            right,
        })
    }
    
    /// Create leaf node from samples
    fn create_leaf(&self, samples: &[TrainingSample]) -> DecisionNode {
        // Find most common position
        let mut position_counts: HashMap<u8, usize> = HashMap::new();
        for sample in samples {
            *position_counts.entry(sample.correct_position).or_insert(0) += 1;
        }
        
        let (position, count) = position_counts
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(pos, count)| (*pos, *count))
            .unwrap_or((0, 0));
        
        let confidence = count as f64 / samples.len() as f64;
        
        DecisionNode::Leaf {
            position,
            confidence,
            sample_count: samples.len(),
        }
    }
    
    /// Find best feature and threshold to split on
    fn find_best_split(
        &self,
        samples: &[TrainingSample],
    ) -> Result<Option<(Feature, f64, Vec<TrainingSample>, Vec<TrainingSample>)>> {
        let mut best_gini = f64::INFINITY;
        let mut best_split = None;
        
        // Try all features
        for &feature in &[
            Feature::Angle,
            Feature::Distance,
            Feature::Complexity,
            Feature::RuleBasedPrediction,
        ] {
            // Try multiple thresholds
            let thresholds = self.get_candidate_thresholds(samples, feature);
            
            for threshold in thresholds {
                let (left, right) = self.split_samples(samples, feature, threshold);
                
                if left.is_empty() || right.is_empty() {
                    continue;
                }
                
                let gini = self.calculate_gini(&left, &right);
                
                if gini < best_gini {
                    best_gini = gini;
                    best_split = Some((feature, threshold, left, right));
                }
            }
        }
        
        Ok(best_split)
    }
    
    /// Get candidate thresholds for a feature
    fn get_candidate_thresholds(&self, samples: &[TrainingSample], feature: Feature) -> Vec<f64> {
        let mut values: Vec<f64> = samples
            .iter()
            .map(|s| self.get_feature_value(s, feature))
            .collect();
        
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        values.dedup();
        
        // Use midpoints between consecutive values
        values
            .windows(2)
            .map(|w| (w[0] + w[1]) / 2.0)
            .collect()
    }
    
    /// Get feature value from sample
    fn get_feature_value(&self, sample: &TrainingSample, feature: Feature) -> f64 {
        match feature {
            Feature::Angle => sample.angle,
            Feature::Distance => sample.distance,
            Feature::Complexity => sample.complexity,
            Feature::TaskType => match sample.task_type.as_str() {
                "SacredRecognition" => 0.0,
                "PositionMapping" => 1.0,
                "Transformation" => 2.0,
                "SpatialRelations" => 3.0,
                "PatternCompletion" => 4.0,
                _ => 0.0,
            },
            Feature::RuleBasedPrediction => sample.rule_based_prediction as f64,
        }
    }
    
    /// Split samples by feature and threshold
    fn split_samples(
        &self,
        samples: &[TrainingSample],
        feature: Feature,
        threshold: f64,
    ) -> (Vec<TrainingSample>, Vec<TrainingSample>) {
        let mut left = Vec::new();
        let mut right = Vec::new();
        
        for sample in samples {
            let value = self.get_feature_value(sample, feature);
            if value <= threshold {
                left.push(sample.clone());
            } else {
                right.push(sample.clone());
            }
        }
        
        (left, right)
    }
    
    /// Calculate Gini impurity for split
    fn calculate_gini(&self, left: &[TrainingSample], right: &[TrainingSample]) -> f64 {
        let total = left.len() + right.len();
        let left_gini = self.gini_impurity(left);
        let right_gini = self.gini_impurity(right);
        
        (left.len() as f64 / total as f64) * left_gini
            + (right.len() as f64 / total as f64) * right_gini
    }
    
    /// Calculate Gini impurity for samples
    fn gini_impurity(&self, samples: &[TrainingSample]) -> f64 {
        if samples.is_empty() {
            return 0.0;
        }
        
        let mut position_counts: HashMap<u8, usize> = HashMap::new();
        for sample in samples {
            *position_counts.entry(sample.correct_position).or_insert(0) += 1;
        }
        
        let total = samples.len() as f64;
        let mut gini = 1.0;
        
        for count in position_counts.values() {
            let p = *count as f64 / total;
            gini -= p * p;
        }
        
        gini
    }
    
    /// Predict position for input
    pub fn predict(&self, sample: &TrainingSample) -> (u8, f64) {
        self.predict_node(&self.root, sample)
    }
    
    /// Recursively predict using tree
    fn predict_node(&self, node: &DecisionNode, sample: &TrainingSample) -> (u8, f64) {
        match node {
            DecisionNode::Leaf { position, confidence, .. } => (*position, *confidence),
            DecisionNode::Split { feature, threshold, left, right } => {
                let value = self.get_feature_value(sample, *feature);
                if value <= *threshold {
                    self.predict_node(left, sample)
                } else {
                    self.predict_node(right, sample)
                }
            }
        }
    }
}

/// Ensemble predictor combining rule-based + ML
#[derive(Debug)]
pub struct EnsemblePredictor {
    rule_engine: GeometricInferenceEngine,
    ml_model: Option<DecisionTree>,
    training_data: Vec<TrainingSample>,
    
    /// Weight for rule-based (0.0-1.0), ML gets (1.0 - weight)
    rule_weight: f64,
}

impl Default for EnsemblePredictor {
    fn default() -> Self {
        Self::new()
    }
}

impl EnsemblePredictor {
    /// Create new ensemble predictor
    pub fn new() -> Self {
        Self {
            rule_engine: GeometricInferenceEngine::new(),
            ml_model: None,
            training_data: Vec::new(),
            rule_weight: 0.6, // Default: 60% rule-based, 40% ML
        }
    }
    
    /// Set rule-based weight (0.0 = all ML, 1.0 = all rules)
    pub fn with_rule_weight(mut self, weight: f64) -> Self {
        self.rule_weight = weight.clamp(0.0, 1.0);
        self
    }
    
    /// Add training sample
    pub fn add_training_sample(&mut self, sample: TrainingSample) {
        self.training_data.push(sample);
    }
    
    /// Train ML model on collected data
    pub fn train(&mut self) -> Result<()> {
        if self.training_data.is_empty() {
            return Ok(());
        }
        
        let mut tree = DecisionTree::new(
            10,  // max_depth
            5,   // min_samples_split
        );
        
        tree.train(&self.training_data)?;
        self.ml_model = Some(tree);
        
        Ok(())
    }
    
    /// Predict with ensemble
    pub fn predict(&self, input: &GeometricInput) -> (u8, f64) {
        // Get rule-based prediction
        let rule_position = self.rule_engine.infer_position(input);
        let rule_confidence = self.rule_engine.confidence(input, rule_position);
        
        // If no ML model trained, use rules only
        if self.ml_model.is_none() {
            return (rule_position, rule_confidence);
        }
        
        // Get ML prediction
        let sample = TrainingSample {
            angle: input.angle,
            distance: input.distance,
            complexity: input.complexity,
            task_type: format!("{:?}", input.task_type),
            correct_position: 0, // Unknown
            rule_based_prediction: rule_position,
            rule_based_correct: false,
        };
        
        let (ml_position, ml_confidence) = self.ml_model.as_ref().unwrap().predict(&sample);
        
        // Apply flow-aware correction
        let corrected_ml = self.apply_flow_correction(ml_position, input);
        
        // Ensemble: weighted voting
        if rule_position == corrected_ml {
            // Both agree - high confidence
            let combined_confidence = rule_confidence * self.rule_weight 
                + ml_confidence * (1.0 - self.rule_weight);
            return (rule_position, combined_confidence);
        }
        
        // Disagree - use confidence-weighted decision
        let rule_score = rule_confidence * self.rule_weight;
        let ml_score = ml_confidence * (1.0 - self.rule_weight);
        
        if rule_score > ml_score {
            (rule_position, rule_confidence * 0.8) // Reduce confidence due to disagreement
        } else {
            (corrected_ml, ml_confidence * 0.8)
        }
    }
    
    /// Apply flow-aware correction based on vortex math patterns
    fn apply_flow_correction(&self, position: u8, input: &GeometricInput) -> u8 {
        // Vortex flow patterns: 1→2→4→8→7→5→1 (forward)
        const FORWARD_FLOW: [u8; 6] = [1, 2, 4, 8, 7, 5];
        #[allow(dead_code)]  // Reserved for backpropagation implementation
        const BACKWARD_FLOW: [u8; 6] = [1, 5, 7, 8, 4, 2];
        
        // Sacred positions (3, 6, 9) don't participate in flow
        if position == 3 || position == 6 || position == 9 || position == 0 {
            return position;
        }
        
        // For transformation tasks, check if position follows flow
        if matches!(input.task_type, GeometricTaskType::Transformation) {
            // Check if position is in flow sequence
            if FORWARD_FLOW.contains(&position) {
                return position;
            }
            
            // Snap to nearest flow position
            return self.snap_to_flow(position);
        }
        
        position
    }
    
    /// Snap position to nearest flow sequence position
    fn snap_to_flow(&self, position: u8) -> u8 {
        const FORWARD_FLOW: [u8; 6] = [1, 2, 4, 8, 7, 5];
        
        // Find closest flow position
        FORWARD_FLOW
            .iter()
            .min_by_key(|&&flow_pos| {
                let diff = (flow_pos as i32 - position as i32).abs();
                diff.min(10 - diff) // Circular distance
            })
            .copied()
            .unwrap_or(position)
    }
    
    /// Get training data size
    pub fn training_size(&self) -> usize {
        self.training_data.len()
    }
    
    /// Get model status
    pub fn is_trained(&self) -> bool {
        self.ml_model.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_decision_tree_training() {
        let samples = vec![
            TrainingSample {
                angle: 0.0,
                distance: 5.0,
                complexity: 0.5,
                task_type: "PositionMapping".to_string(),
                correct_position: 0,
                rule_based_prediction: 0,
                rule_based_correct: true,
            },
            TrainingSample {
                angle: 180.0,
                distance: 5.0,
                complexity: 0.5,
                task_type: "PositionMapping".to_string(),
                correct_position: 5,
                rule_based_prediction: 5,
                rule_based_correct: true,
            },
        ];
        
        let mut tree = DecisionTree::new(10, 2);
        tree.train(&samples).unwrap();
        
        let (position, confidence) = tree.predict(&samples[0]);
        assert_eq!(position, 0);
        assert!(confidence > 0.0);
    }
    
    #[test]
    fn test_ensemble_prediction() {
        let mut ensemble = EnsemblePredictor::new();
        
        let input = GeometricInput {
            angle: 60.0,
            distance: 5.0,
            complexity: 0.5,
            task_type: GeometricTaskType::SacredRecognition,
        };
        
        let (position, confidence) = ensemble.predict(&input);
        assert_eq!(position, 3); // Should recognize sacred position
        assert!(confidence > 0.0);
    }
    
    #[test]
    fn test_flow_correction() {
        let ensemble = EnsemblePredictor::new();
        
        let input = GeometricInput {
            angle: 45.0,
            distance: 3.0,
            complexity: 0.7,
            task_type: GeometricTaskType::Transformation,
        };
        
        // Position 3 should stay 3 (sacred)
        assert_eq!(ensemble.apply_flow_correction(3, &input), 3);
        
        // Non-flow position should snap to flow
        let corrected = ensemble.apply_flow_correction(6, &input);
        assert_eq!(corrected, 6); // Sacred, stays put
    }
}
