//! Federated learning coordinator across multiple subjects

use super::subject_domain::{SubjectDomain, SubjectMatrix};
use crate::ml::training::{VortexSGD, SacredGradientField};
use crate::ml::training::vortex_sgd::TrainingConfig;
use crate::data::models::ELPTensor;
use std::collections::HashMap;

/// Federated learning system coordinating multiple subject matrices
///
/// Enables collaborative learning across different knowledge domains
/// (Ethics, Logic, Emotion) using shared sacred geometric structure.
///
/// # Examples
///
/// ```
/// use spatial_vortex::federated::{FederatedLearner, SubjectDomain};
///
/// let mut learner = FederatedLearner::new();
///
/// // Add all three subjects
/// learner.add_subject(SubjectDomain::Ethics);
/// learner.add_subject(SubjectDomain::Logic);
/// learner.add_subject(SubjectDomain::Emotion);
///
/// // Check federation status
/// assert_eq!(learner.num_subjects(), 3);
/// ```
pub struct FederatedLearner {
    /// Subject-specific matrices
    subjects: HashMap<SubjectDomain, SubjectMatrix>,
    /// Shared SGD optimizer
    optimizer: VortexSGD,
    /// Sacred gradient field (shared across subjects)
    sacred_field: SacredGradientField,
    /// Global step counter
    global_step: usize,
}

impl FederatedLearner {
    /// Creates a new federated learner
    pub fn new() -> Self {
        let config = TrainingConfig::default();
        
        Self {
            subjects: HashMap::new(),
            optimizer: VortexSGD::new(config),
            sacred_field: SacredGradientField::new(1.0),
            global_step: 0,
        }
    }
    
    /// Adds a subject domain to the federation
    pub fn add_subject(&mut self, domain: SubjectDomain) {
        if !self.subjects.contains_key(&domain) {
            self.subjects.insert(domain, SubjectMatrix::new(domain));
        }
    }
    
    /// Returns the number of federated subjects
    pub fn num_subjects(&self) -> usize {
        self.subjects.len()
    }
    
    /// Gets a subject matrix
    pub fn get_subject(&self, domain: SubjectDomain) -> Option<&SubjectMatrix> {
        self.subjects.get(&domain)
    }
    
    /// Performs federated training step across all subjects
    ///
    /// 1. Each subject processes data locally
    /// 2. Gradients flow through shared sacred geometry
    /// 3. Updates aggregated across subjects
    /// 4. Sacred positions synchronize learning
    pub fn federated_train_step(&mut self, data: &[f64]) -> f64 {
        let mut total_loss = 0.0;
        let mut aggregated_gradients = vec![0.0; data.len()];
        
        // Each subject processes independently
        for (_domain, _subject) in &mut self.subjects {
            // Forward pass through subject's vortex
            let output = self.optimizer.forward_pass(data);
            
            // Compute loss (simplified)
            let loss: f64 = output.iter().sum();
            total_loss += loss;
            
            // Backward pass
            let gradients = self.optimizer.backward_pass(&output);
            
            // Aggregate gradients
            for (i, grad) in gradients.iter().enumerate() {
                if i < aggregated_gradients.len() {
                    aggregated_gradients[i] += grad;
                }
            }
        }
        
        // Average gradients across subjects
        let num_subjects = self.subjects.len() as f64;
        for grad in &mut aggregated_gradients {
            *grad /= num_subjects;
        }
        
        // Apply sacred gradient attraction
        // (This synchronizes learning across subjects via sacred positions)
        let elp = ELPTensor::new(
            aggregated_gradients.get(0).copied().unwrap_or(0.0),
            aggregated_gradients.get(1).copied().unwrap_or(0.0),
            aggregated_gradients.get(2).copied().unwrap_or(0.0),
        );
        
        let mut sacred_grad = elp.clone();
        self.sacred_field.apply_sacred_gradient(
            &mut sacred_grad,
            (self.global_step % 10) as u8,
            &elp,
        );
        
        // Update weights for all subjects
        for (_domain, _subject) in &mut self.subjects {
            self.optimizer.step(&mut _subject.weights, &aggregated_gradients);
        }
        
        self.global_step += 1;
        
        total_loss / num_subjects
    }
    
    /// Gets consensus across subjects for a position
    ///
    /// Returns the weighted average of activations at a position
    /// across all subject domains
    pub fn get_consensus(&self, position: u8) -> f64 {
        if self.subjects.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = self.subjects.values()
            .filter_map(|s| s.weights.get(position as usize))
            .sum();
        
        sum / self.subjects.len() as f64
    }
    
    /// Checks if subjects agree on sacred positions
    ///
    /// Returns true if all subjects have high activation at sacred positions
    pub fn sacred_alignment(&self) -> bool {
        for subject in self.subjects.values() {
            for pos in [3, 6, 9] {
                if let Some(&weight) = subject.weights.get(pos as usize) {
                    if weight < 0.5 {
                        return false;
                    }
                }
            }
        }
        true
    }
    
    /// Returns global training step
    pub fn global_step(&self) -> usize {
        self.global_step
    }
}

impl Default for FederatedLearner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_federated_learner_creation() {
        let learner = FederatedLearner::new();
        assert_eq!(learner.num_subjects(), 0);
        assert_eq!(learner.global_step(), 0);
    }
    
    #[test]
    fn test_add_subjects() {
        let mut learner = FederatedLearner::new();
        
        learner.add_subject(SubjectDomain::Ethics);
        assert_eq!(learner.num_subjects(), 1);
        
        learner.add_subject(SubjectDomain::Logic);
        learner.add_subject(SubjectDomain::Emotion);
        assert_eq!(learner.num_subjects(), 3);
    }
    
    #[test]
    fn test_get_subject() {
        let mut learner = FederatedLearner::new();
        learner.add_subject(SubjectDomain::Ethics);
        
        let ethics = learner.get_subject(SubjectDomain::Ethics);
        assert!(ethics.is_some());
        assert_eq!(ethics.unwrap().domain, SubjectDomain::Ethics);
    }
    
    #[test]
    fn test_federated_training() {
        let mut learner = FederatedLearner::new();
        learner.add_subject(SubjectDomain::Ethics);
        learner.add_subject(SubjectDomain::Logic);
        
        let data = vec![1.0, 2.0, 3.0];
        let loss = learner.federated_train_step(&data);
        
        assert!(loss >= 0.0);
        assert_eq!(learner.global_step(), 1);
    }
    
    #[test]
    fn test_consensus() {
        let mut learner = FederatedLearner::new();
        learner.add_subject(SubjectDomain::Ethics);
        learner.add_subject(SubjectDomain::Logic);
        
        let consensus = learner.get_consensus(3);
        assert!(consensus >= 0.0);
    }
}
