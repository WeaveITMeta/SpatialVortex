//! Continuous Learning System
//!
//! Phase 4: Learn from user feedback and dynamically update semantic associations
//! to continuously improve inference quality over time.

use crate::data::models::*;
use crate::error::Result;
use crate::core::sacred_geometry::FluxMatrixEngine;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use chrono::{DateTime, Utc};

/// User feedback on a response
#[derive(Debug, Clone)]
pub struct UserFeedback {
    /// Unique feedback ID
    pub id: String,
    
    /// Query that was asked
    pub query: String,
    
    /// Subject being queried
    pub subject: String,
    
    /// Position used for inference
    pub position: u8,
    
    /// Response that was generated
    pub response: String,
    
    /// User rating (1-5 stars)
    pub rating: u8,
    
    /// Whether the response was helpful
    pub helpful: bool,
    
    /// Terms user found relevant
    pub relevant_terms: Vec<String>,
    
    /// Terms user found irrelevant
    pub irrelevant_terms: Vec<String>,
    
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

impl UserFeedback {
    /// Create new feedback
    pub fn new(
        query: String,
        subject: String,
        position: u8,
        response: String,
        rating: u8,
        helpful: bool,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            query,
            subject,
            position,
            response,
            rating,
            helpful,
            relevant_terms: Vec::new(),
            irrelevant_terms: Vec::new(),
            timestamp: Utc::now(),
        }
    }
    
    /// Add relevant term
    pub fn add_relevant(&mut self, term: String) {
        if !self.relevant_terms.contains(&term) {
            self.relevant_terms.push(term);
        }
    }
    
    /// Add irrelevant term
    pub fn add_irrelevant(&mut self, term: String) {
        if !self.irrelevant_terms.contains(&term) {
            self.irrelevant_terms.push(term);
        }
    }
    
    /// Check if feedback is positive
    pub fn is_positive(&self) -> bool {
        self.rating >= 4 && self.helpful
    }
    
    /// Check if feedback is negative
    pub fn is_negative(&self) -> bool {
        self.rating <= 2 || !self.helpful
    }
}

/// Learning adjustment to apply to semantic associations
#[derive(Debug, Clone)]
pub struct LearningAdjustment {
    /// Subject to adjust
    pub subject: String,
    
    /// Position to adjust
    pub position: u8,
    
    /// Association to add/strengthen (positive)
    pub strengthen: Vec<(String, f64)>,  // (word, confidence_boost)
    
    /// Association to remove/weaken (negative)
    pub weaken: Vec<(String, f64)>,  // (word, confidence_penalty)
    
    /// Reason for adjustment
    pub reason: String,
    
    /// Confidence in this adjustment (0.0-1.0)
    pub confidence: f64,
    
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

impl LearningAdjustment {
    /// Create new adjustment
    pub fn new(subject: String, position: u8) -> Self {
        Self {
            subject,
            position,
            strengthen: Vec::new(),
            weaken: Vec::new(),
            reason: String::new(),
            confidence: 0.5,
            timestamp: Utc::now(),
        }
    }
    
    /// Add term to strengthen
    pub fn add_strengthen(&mut self, word: String, boost: f64) {
        self.strengthen.push((word, boost));
    }
    
    /// Add term to weaken
    pub fn add_weaken(&mut self, word: String, penalty: f64) {
        self.weaken.push((word, penalty));
    }
}

/// Learning metrics for monitoring
#[derive(Debug, Clone)]
pub struct LearningMetrics {
    /// Total feedback received
    pub total_feedback: usize,
    
    /// Positive feedback count
    pub positive_feedback: usize,
    
    /// Negative feedback count
    pub negative_feedback: usize,
    
    /// Total adjustments made
    pub total_adjustments: usize,
    
    /// Associations strengthened
    pub strengthened_count: usize,
    
    /// Associations weakened
    pub weakened_count: usize,
    
    /// Average rating (1-5)
    pub average_rating: f64,
    
    /// Success rate (positive / total)
    pub success_rate: f64,
    
    /// Last update timestamp
    pub last_update: DateTime<Utc>,
}

impl Default for LearningMetrics {
    fn default() -> Self {
        Self {
            total_feedback: 0,
            positive_feedback: 0,
            negative_feedback: 0,
            total_adjustments: 0,
            strengthened_count: 0,
            weakened_count: 0,
            average_rating: 0.0,
            success_rate: 0.0,
            last_update: Utc::now(),
        }
    }
}

impl LearningMetrics {
    /// Update metrics with new feedback
    pub fn update_with_feedback(&mut self, feedback: &UserFeedback) {
        self.total_feedback += 1;
        
        if feedback.is_positive() {
            self.positive_feedback += 1;
        } else if feedback.is_negative() {
            self.negative_feedback += 1;
        }
        
        // Update average rating
        let total_ratings = self.total_feedback as f64;
        self.average_rating = (self.average_rating * (total_ratings - 1.0) + feedback.rating as f64) / total_ratings;
        
        // Update success rate
        self.success_rate = self.positive_feedback as f64 / total_ratings;
        
        self.last_update = Utc::now();
    }
    
    /// Update metrics with adjustment
    pub fn update_with_adjustment(&mut self, adjustment: &LearningAdjustment) {
        self.total_adjustments += 1;
        self.strengthened_count += adjustment.strengthen.len();
        self.weakened_count += adjustment.weaken.len();
        self.last_update = Utc::now();
    }
}

/// Continuous learning engine
pub struct ContinuousLearning {
    #[allow(dead_code)]  // Reserved for future matrix creation/validation
    flux_engine: FluxMatrixEngine,
    feedback_history: Vec<UserFeedback>,
    adjustments_history: Vec<LearningAdjustment>,
    metrics: HashMap<String, LearningMetrics>,  // Per subject
    learning_rate: f64,  // How aggressively to update (0.0-1.0)
    min_confidence_threshold: f64,  // Minimum confidence for adjustments
}

impl ContinuousLearning {
    /// Create new continuous learning engine
    pub fn new(flux_engine: FluxMatrixEngine) -> Self {
        Self {
            flux_engine,
            feedback_history: Vec::new(),
            adjustments_history: Vec::new(),
            metrics: HashMap::new(),
            learning_rate: 0.1,  // Conservative by default
            min_confidence_threshold: 0.7,  // Only high-confidence adjustments
        }
    }
    
    /// Set learning rate (0.0-1.0)
    pub fn set_learning_rate(&mut self, rate: f64) {
        self.learning_rate = rate.max(0.0).min(1.0);
    }
    
    /// Submit user feedback
    pub fn submit_feedback(&mut self, feedback: UserFeedback) -> Result<()> {
        // Update metrics for this subject
        let metrics = self.metrics.entry(feedback.subject.clone())
            .or_insert_with(LearningMetrics::default);
        metrics.update_with_feedback(&feedback);
        
        // Store feedback
        self.feedback_history.push(feedback);
        
        Ok(())
    }
    
    /// Generate learning adjustments from recent feedback
    pub fn generate_adjustments(&mut self, subject: &str) -> Result<Vec<LearningAdjustment>> {
        let mut adjustments_map: HashMap<u8, LearningAdjustment> = HashMap::new();
        
        // Analyze recent feedback for this subject
        let recent_feedback: Vec<_> = self.feedback_history
            .iter()
            .filter(|f| f.subject == subject)
            .rev()
            .take(100)  // Last 100 feedback items
            .collect();
        
        for feedback in recent_feedback {
            let adjustment = adjustments_map
                .entry(feedback.position)
                .or_insert_with(|| {
                    let mut adj = LearningAdjustment::new(subject.to_string(), feedback.position);
                    adj.reason = format!("Learning from {} feedback items", 1);
                    adj
                });
            
            // Update reason count
            if let Some(count) = adjustment.reason.split_whitespace().nth(2) {
                if let Ok(n) = count.parse::<usize>() {
                    adjustment.reason = format!("Learning from {} feedback items", n + 1);
                }
            }
            
            if feedback.is_positive() {
                // Strengthen relevant terms
                for term in &feedback.relevant_terms {
                    let boost = self.learning_rate * (feedback.rating as f64 / 5.0);
                    adjustment.add_strengthen(term.clone(), boost);
                }
                
                // Weaken irrelevant terms
                for term in &feedback.irrelevant_terms {
                    let penalty = self.learning_rate * 0.5;
                    adjustment.add_weaken(term.clone(), penalty);
                }
            } else if feedback.is_negative() {
                // Weaken terms that were present but not helpful
                for term in &feedback.irrelevant_terms {
                    let penalty = self.learning_rate * (1.0 - feedback.rating as f64 / 5.0);
                    adjustment.add_weaken(term.clone(), penalty);
                }
            }
            
            // Update adjustment confidence based on feedback quality
            let feedback_weight = if feedback.rating >= 4 || feedback.rating <= 2 {
                1.0  // Strong feedback
            } else {
                0.5  // Weak feedback
            };
            adjustment.confidence += feedback_weight * 0.1;
            adjustment.confidence = adjustment.confidence.min(1.0);
        }
        
        // Filter by confidence threshold
        let adjustments: Vec<_> = adjustments_map
            .into_values()
            .filter(|adj| adj.confidence >= self.min_confidence_threshold)
            .collect();
        
        Ok(adjustments)
    }
    
    /// Apply learning adjustments to a matrix
    pub fn apply_adjustments(
        &mut self,
        matrix: &mut FluxMatrix,
        adjustments: Vec<LearningAdjustment>,
    ) -> Result<usize> {
        let mut applied_count = 0;
        
        for adjustment in adjustments {
            if let Some(node) = matrix.nodes.get_mut(&adjustment.position) {
                // Strengthen associations
                for (word, boost) in &adjustment.strengthen {
                    let mut found = false;
                    
                    // Find existing association and boost it
                    for assoc in &mut node.semantic_index.positive_associations {
                        if assoc.word == *word {
                            assoc.confidence = (assoc.confidence + boost).min(1.0);
                            found = true;
                            break;
                        }
                    }
                    
                    // Add new association if not found
                    if !found {
                        let mut new_assoc = SemanticAssociation::new(
                            word.clone(),
                            1,  // Default index
                            *boost,
                        );
                        new_assoc.set_attribute("learned".to_string(), 1.0);
                        node.semantic_index.positive_associations.push(new_assoc);
                    }
                    
                    applied_count += 1;
                }
                
                // Weaken associations
                for (word, penalty) in &adjustment.weaken {
                    // Reduce confidence
                    node.semantic_index.positive_associations.retain_mut(|assoc| {
                        if assoc.word == *word {
                            assoc.confidence = (assoc.confidence - penalty).max(0.0);
                            applied_count += 1;
                            assoc.confidence > 0.1  // Remove if too weak
                        } else {
                            true
                        }
                    });
                }
            }
            
            // Update metrics
            if let Some(metrics) = self.metrics.get_mut(&adjustment.subject) {
                metrics.update_with_adjustment(&adjustment);
            }
            
            // Store adjustment
            self.adjustments_history.push(adjustment);
        }
        
        Ok(applied_count)
    }
    
    /// Get learning metrics for a subject
    pub fn get_metrics(&self, subject: &str) -> Option<&LearningMetrics> {
        self.metrics.get(subject)
    }
    
    /// Get global learning metrics (all subjects)
    pub fn get_global_metrics(&self) -> LearningMetrics {
        let mut global = LearningMetrics::default();
        
        for metrics in self.metrics.values() {
            global.total_feedback += metrics.total_feedback;
            global.positive_feedback += metrics.positive_feedback;
            global.negative_feedback += metrics.negative_feedback;
            global.total_adjustments += metrics.total_adjustments;
            global.strengthened_count += metrics.strengthened_count;
            global.weakened_count += metrics.weakened_count;
        }
        
        if global.total_feedback > 0 {
            global.average_rating = self.metrics.values()
                .map(|m| m.average_rating * m.total_feedback as f64)
                .sum::<f64>() / global.total_feedback as f64;
            
            global.success_rate = global.positive_feedback as f64 / global.total_feedback as f64;
        }
        
        global.last_update = Utc::now();
        global
    }
    
    /// Get feedback history for a subject
    pub fn get_feedback_history(&self, subject: &str, limit: usize) -> Vec<&UserFeedback> {
        self.feedback_history
            .iter()
            .filter(|f| f.subject == subject)
            .rev()
            .take(limit)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_user_feedback() {
        let mut feedback = UserFeedback::new(
            "test query".to_string(),
            "cognition".to_string(),
            4,
            "test response".to_string(),
            5,
            true,
        );
        
        feedback.add_relevant("reasoning".to_string());
        feedback.add_irrelevant("confusion".to_string());
        
        assert!(feedback.is_positive());
        assert!(!feedback.is_negative());
        assert_eq!(feedback.relevant_terms.len(), 1);
        assert_eq!(feedback.irrelevant_terms.len(), 1);
    }
    
    #[test]
    fn test_learning_metrics() {
        let mut metrics = LearningMetrics::default();
        
        let feedback = UserFeedback::new(
            "test".to_string(),
            "test".to_string(),
            1,
            "response".to_string(),
            5,
            true,
        );
        
        metrics.update_with_feedback(&feedback);
        
        assert_eq!(metrics.total_feedback, 1);
        assert_eq!(metrics.positive_feedback, 1);
        assert_eq!(metrics.average_rating, 5.0);
        assert_eq!(metrics.success_rate, 1.0);
    }
}
