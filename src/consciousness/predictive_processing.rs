//! Predictive Processing - Minimize surprise through prediction
//!
//! Implements the Free Energy Principle: consciousness as constantly predicting
//! and updating a model of reality. Surprise = learning signal.

use super::thought::Thought;
use serde::{Deserialize, Serialize};

/// Predictive processor that learns from prediction errors
#[derive(Debug)]
pub struct PredictiveProcessor {
    /// Internal model of "what should happen next"
    world_model: WorldModel,
    
    /// Tracks prediction accuracy over time
    prediction_history: Vec<PredictionResult>,
    
    /// Current surprise level (prediction error)
    current_surprise: f64,
    
    /// Learning rate for model updates
    learning_rate: f64,
}

/// Internal model of expected patterns
#[derive(Debug, Clone)]
struct WorldModel {
    /// Expected ELP distribution for next thought
    expected_elp: (f64, f64, f64),
    
    /// Expected confidence level
    expected_confidence: f64,
    
    /// Expected flux position sequence
    expected_position: u8,
    
    /// Confidence in predictions
    model_confidence: f64,
}

/// Result of a prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionResult {
    /// What was predicted
    pub prediction: String,
    
    /// What actually happened
    pub actual: String,
    
    /// Surprise level (0.0 = perfect prediction, 1.0 = completely wrong)
    pub surprise: f64,
    
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Surprise signal for learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurpriseSignal {
    /// How unexpected was this? (0.0-1.0)
    pub magnitude: f64,
    
    /// Which dimension had highest surprise?
    pub source: SurpriseSource,
    
    /// Should we update the model?
    pub requires_learning: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum SurpriseSource {
    EthosUnexpected,    // Moral dimension surprised us
    LogosUnexpected,    // Logical dimension surprised us
    PathosUnexpected,   // Emotional dimension surprised us
    ConfidenceShift,    // Confidence was different than expected
    PositionJump,       // Unexpected position in vortex cycle
    ContentNovel,       // Content was entirely new
}

impl PredictiveProcessor {
    pub fn new() -> Self {
        Self {
            world_model: WorldModel::default(),
            prediction_history: Vec::new(),
            current_surprise: 0.0,
            learning_rate: 0.1,
        }
    }
    
    /// Predict next thought based on current model
    pub fn predict_next(&self) -> Thought {
        Thought::new(
            "predicted_thought".to_string(),
            "predictor".to_string(),
            super::thought::ThoughtPriority::Medium,
        )
        .with_elp(
            self.world_model.expected_elp.0,
            self.world_model.expected_elp.1,
            self.world_model.expected_elp.2,
        )
        .with_confidence(self.world_model.expected_confidence)
        .with_flux_position(self.world_model.expected_position)
    }
    
    /// Observe actual thought and compute surprise
    pub fn observe_actual(&mut self, actual: &Thought) -> SurpriseSignal {
        let prediction = self.predict_next();
        
        // Compute surprise for each dimension
        let ethos_surprise = (actual.ethos - prediction.ethos).abs();
        let logos_surprise = (actual.logos - prediction.logos).abs();
        let pathos_surprise = (actual.pathos - prediction.pathos).abs();
        let confidence_surprise = (actual.confidence - prediction.confidence).abs();
        
        // Total surprise (mean of all dimensions)
        let total_surprise = (ethos_surprise + logos_surprise + pathos_surprise + confidence_surprise) / 4.0;
        
        // Determine primary source of surprise
        let source = if ethos_surprise > logos_surprise && ethos_surprise > pathos_surprise {
            SurpriseSource::EthosUnexpected
        } else if logos_surprise > pathos_surprise {
            SurpriseSource::LogosUnexpected
        } else {
            SurpriseSource::PathosUnexpected
        };
        
        self.current_surprise = total_surprise;
        
        // Record prediction result
        self.prediction_history.push(PredictionResult {
            prediction: format!("E:{:.2}/L:{:.2}/P:{:.2}", 
                prediction.ethos, prediction.logos, prediction.pathos),
            actual: format!("E:{:.2}/L:{:.2}/P:{:.2}", 
                actual.ethos, actual.logos, actual.pathos),
            surprise: total_surprise,
            timestamp: chrono::Utc::now(),
        });
        
        // Keep only recent history
        if self.prediction_history.len() > 50 {
            self.prediction_history.remove(0);
        }
        
        // Update model if surprise is significant
        if total_surprise > 0.2 {
            self.update_model(actual);
        }
        
        SurpriseSignal {
            magnitude: total_surprise,
            source,
            requires_learning: total_surprise > 0.3,
        }
    }
    
    /// Update world model based on prediction error
    fn update_model(&mut self, actual: &Thought) {
        let lr = self.learning_rate;
        
        // Update expected ELP (weighted average with actual)
        self.world_model.expected_elp.0 = 
            (1.0 - lr) * self.world_model.expected_elp.0 + lr * actual.ethos;
        self.world_model.expected_elp.1 = 
            (1.0 - lr) * self.world_model.expected_elp.1 + lr * actual.logos;
        self.world_model.expected_elp.2 = 
            (1.0 - lr) * self.world_model.expected_elp.2 + lr * actual.pathos;
        
        // Update expected confidence
        self.world_model.expected_confidence = 
            (1.0 - lr) * self.world_model.expected_confidence + lr * actual.confidence;
        
        // Update position expectation
        self.world_model.expected_position = actual.flux_position;
        
        // Increase model confidence as we learn
        self.world_model.model_confidence = 
            (self.world_model.model_confidence + 0.05).min(0.95);
    }
    
    /// Get current surprise level
    pub fn current_surprise(&self) -> f64 {
        self.current_surprise
    }
    
    /// Get model confidence (how sure is the model?)
    pub fn model_confidence(&self) -> f64 {
        self.world_model.model_confidence
    }
    
    /// Get prediction accuracy over recent history
    pub fn prediction_accuracy(&self) -> f64 {
        if self.prediction_history.is_empty() {
            return 0.5;
        }
        
        let recent_surprise: f64 = self.prediction_history
            .iter()
            .rev()
            .take(10)
            .map(|p| p.surprise)
            .sum::<f64>() / 10.0_f64.min(self.prediction_history.len() as f64);
        
        // Accuracy = 1 - surprise
        1.0 - recent_surprise
    }
    
    /// Get total learning (how much has the model improved?)
    pub fn total_learning(&self) -> f64 {
        if self.prediction_history.len() < 2 {
            return 0.0;
        }
        
        let early_surprise: f64 = self.prediction_history
            .iter()
            .take(5)
            .map(|p| p.surprise)
            .sum::<f64>() / 5.0_f64.min(self.prediction_history.len() as f64);
        
        let recent_surprise: f64 = self.prediction_history
            .iter()
            .rev()
            .take(5)
            .map(|p| p.surprise)
            .sum::<f64>() / 5.0;
        
        // Learning = reduction in surprise
        (early_surprise - recent_surprise).max(0.0)
    }
    
    /// Generate prediction report
    pub fn prediction_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("=== Predictive Processing Report ===\n\n");
        
        report.push_str(&format!("Current Surprise: {:.1}%\n", self.current_surprise * 100.0));
        report.push_str(&format!("Model Confidence: {:.1}%\n", self.model_confidence() * 100.0));
        report.push_str(&format!("Prediction Accuracy: {:.1}%\n", self.prediction_accuracy() * 100.0));
        report.push_str(&format!("Total Learning: {:.1}%\n\n", self.total_learning() * 100.0));
        
        report.push_str("Expected Next Thought (ELP):\n");
        report.push_str(&format!("  Ethos:  {:.1}%\n", self.world_model.expected_elp.0 * 100.0));
        report.push_str(&format!("  Logos:  {:.1}%\n", self.world_model.expected_elp.1 * 100.0));
        report.push_str(&format!("  Pathos: {:.1}%\n\n", self.world_model.expected_elp.2 * 100.0));
        
        if !self.prediction_history.is_empty() {
            report.push_str("Recent Predictions:\n");
            for result in self.prediction_history.iter().rev().take(3) {
                report.push_str(&format!("  Predicted: {} → Actual: {} (surprise: {:.1}%)\n",
                    result.prediction,
                    result.actual,
                    result.surprise * 100.0
                ));
            }
        }
        
        report
    }
}

impl Default for PredictiveProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for WorldModel {
    fn default() -> Self {
        Self {
            expected_elp: (0.33, 0.33, 0.34),
            expected_confidence: 0.5,
            expected_position: 1,
            model_confidence: 0.3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consciousness::thought::ThoughtPriority;
    
    #[test]
    fn test_predictor_creation() {
        let predictor = PredictiveProcessor::new();
        assert_eq!(predictor.current_surprise(), 0.0);
        assert!(predictor.model_confidence() < 0.5);
    }
    
    #[test]
    fn test_prediction_learning() {
        let mut predictor = PredictiveProcessor::new();
        
        // Feed consistent pattern
        for _ in 0..10 {
            let thought = Thought::new(
                "test".to_string(),
                "agent".to_string(),
                ThoughtPriority::Medium,
            ).with_elp(0.7, 0.2, 0.1)
             .with_confidence(0.8);
            
            predictor.observe_actual(&thought);
        }
        
        // Model should have learned the pattern
        assert!(predictor.model_confidence() > 0.3);
        assert!(predictor.prediction_accuracy() > 0.5);
    }
}
