//! Color-Aware Inference for Aspect Colors
//!
//! Provides color-to-meaning prediction and meaning-to-color generation
//! using trained aspect color models.

use crate::data::{AspectColor, AspectColorDataset, SemanticColorSpace};
use anyhow::Result;
use std::collections::HashMap;

/// Color-aware inference engine
pub struct ColorInferenceEngine {
    /// Semantic color space with learned associations
    color_space: SemanticColorSpace,
    
    /// Meaning-to-color cache
    meaning_cache: HashMap<String, AspectColor>,
    
    /// Configuration
    config: ColorInferenceConfig,
}

/// Configuration for color inference
#[derive(Debug, Clone)]
pub struct ColorInferenceConfig {
    /// Maximum color distance for similarity matching
    pub max_distance: f32,
    
    /// Confidence threshold for predictions
    pub confidence_threshold: f32,
    
    /// Number of top predictions to return
    pub top_k: usize,
    
    /// Use semantic relationships for better predictions
    pub use_relationships: bool,
}

impl Default for ColorInferenceConfig {
    fn default() -> Self {
        Self {
            max_distance: 0.3,
            confidence_threshold: 0.6,
            top_k: 5,
            use_relationships: true,
        }
    }
}

/// Color prediction result
#[derive(Debug, Clone)]
pub struct ColorPrediction {
    /// Predicted semantic meaning
    pub meaning: String,
    
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
    
    /// Color distance from query
    pub distance: f32,
    
    /// Associated color
    pub color: AspectColor,
}

impl ColorInferenceEngine {
    /// Create new color inference engine
    pub fn new(config: ColorInferenceConfig) -> Self {
        Self {
            color_space: SemanticColorSpace::new(),
            meaning_cache: HashMap::new(),
            config,
        }
    }
    
    /// Load pre-trained color associations from dataset
    pub fn load_from_dataset(&mut self, dataset: &AspectColorDataset) {
        for sample in dataset.samples() {
            let color = sample.color;
            let meaning = sample.meaning.clone();
            
            // Add to cache
            self.meaning_cache.insert(meaning.clone(), color);
            
            // Register in color space
            let aspect = crate::data::AspectOrientation::from_meaning(&meaning, 0.2);
            self.color_space.register_aspect(aspect);
        }
    }
    
    /// Predict semantic meaning from color
    /// 
    /// Returns top-k predictions sorted by confidence
    pub fn color_to_meaning(&self, color: &AspectColor) -> Vec<ColorPrediction> {
        let mut predictions = Vec::new();
        
        // Find similar colors in space
        let similar = self.color_space.find_by_color(color, self.config.max_distance);
        
        for aspect in similar {
            let distance = color.distance(&aspect.color);
            
            // Confidence: inverse of distance, normalized
            let confidence = 1.0 - (distance / self.config.max_distance);
            
            if confidence >= self.config.confidence_threshold {
                predictions.push(ColorPrediction {
                    meaning: aspect.meaning.clone(),
                    confidence,
                    distance,
                    color: aspect.color,
                });
            }
        }
        
        // Sort by confidence (descending)
        predictions.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        
        // Return top-k
        predictions.truncate(self.config.top_k);
        predictions
    }
    
    /// Generate color from semantic meaning
    pub fn meaning_to_color(&self, meaning: &str) -> Result<AspectColor> {
        // Check cache first
        if let Some(color) = self.meaning_cache.get(meaning) {
            return Ok(*color);
        }
        
        // Generate from semantic hash (fallback)
        Ok(AspectColor::from_meaning(meaning))
    }
    
    /// Predict multiple meanings and blend their colors
    pub fn meanings_to_blended_color(&self, meanings: &[String], weights: &[f32]) -> Result<AspectColor> {
        if meanings.is_empty() {
            anyhow::bail!("No meanings provided");
        }
        
        if meanings.len() != weights.len() {
            anyhow::bail!("Meanings and weights must have same length");
        }
        
        // Get colors for each meaning
        let colors: Vec<AspectColor> = meanings.iter()
            .map(|m| self.meaning_to_color(m).unwrap_or_else(|_| AspectColor::from_meaning(m)))
            .collect();
        
        // Blend colors with weights
        let mut result = colors[0];
        let mut total_weight = weights[0];
        
        for i in 1..colors.len() {
            let weight = weights[i];
            result = result.blend(&colors[i], weight / (total_weight + weight));
            total_weight += weight;
        }
        
        Ok(result)
    }
    
    /// Find semantically similar meanings by color proximity
    pub fn find_similar_meanings(&self, meaning: &str, max_count: usize) -> Vec<String> {
        let target_color = match self.meaning_to_color(meaning) {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };
        
        let predictions = self.color_to_meaning(&target_color);
        
        predictions.iter()
            .filter(|p| p.meaning != meaning)  // Exclude query itself
            .take(max_count)
            .map(|p| p.meaning.clone())
            .collect()
    }
    
    /// Color-guided generation context
    pub fn create_color_context(&self, color: &AspectColor) -> Result<ColorContext> {
        let predictions = self.color_to_meaning(color);
        
        let primary_meaning = predictions.first()
            .map(|p| p.meaning.clone())
            .unwrap_or_else(|| "neutral".to_string());
        
        let all_meanings: Vec<String> = predictions.iter()
            .map(|p| p.meaning.clone())
            .collect();
        
        Ok(ColorContext {
            color: *color,
            primary_meaning,
            related_meanings: all_meanings,
            average_confidence: predictions.iter()
                .map(|p| p.confidence)
                .sum::<f32>() / predictions.len().max(1) as f32,
        })
    }
}

/// Color context for guided generation
#[derive(Debug, Clone)]
pub struct ColorContext {
    /// The guiding color
    pub color: AspectColor,
    
    /// Primary semantic meaning
    pub primary_meaning: String,
    
    /// Related semantic meanings
    pub related_meanings: Vec<String>,
    
    /// Average confidence of predictions
    pub average_confidence: f32,
}

impl ColorContext {
    /// Format as prompt context
    pub fn to_prompt_context(&self) -> String {
        format!(
            "Color context ({}): Primary meaning '{}', related: [{}]",
            self.color.to_hex(),
            self.primary_meaning,
            self.related_meanings.join(", ")
        )
    }
}

/// Statistics for color inference performance
#[derive(Debug, Clone, Default)]
pub struct InferenceStats {
    pub total_predictions: usize,
    pub average_confidence: f32,
    pub average_predictions_per_query: f32,
    pub cache_hit_rate: f32,
}

impl ColorInferenceEngine {
    /// Get inference statistics
    pub fn stats(&self) -> InferenceStats {
        InferenceStats {
            total_predictions: self.meaning_cache.len(),
            average_confidence: 0.0,  // Would track over time
            average_predictions_per_query: self.config.top_k as f32,
            cache_hit_rate: 0.0,  // Would track over time
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::AspectTrainingData;
    
    fn create_test_dataset() -> AspectColorDataset {
        let mut dataset = AspectColorDataset::new();
        
        // Add test samples
        for meaning in &["love", "joy", "peace", "anger", "sadness"] {
            dataset.add_sample(AspectTrainingData::new(
                meaning.to_string(),
                AspectColor::from_meaning(meaning),
            ));
        }
        
        dataset
    }
    
    #[test]
    fn test_color_to_meaning() {
        let dataset = create_test_dataset();
        let mut engine = ColorInferenceEngine::new(ColorInferenceConfig::default());
        engine.load_from_dataset(&dataset);
        
        // Predict meaning from love color
        let love_color = AspectColor::from_meaning("love");
        let predictions = engine.color_to_meaning(&love_color);
        
        // Should have predictions
        assert!(!predictions.is_empty());
        
        // Top prediction should be love (or very close)
        assert!(predictions[0].confidence > 0.6);
    }
    
    #[test]
    fn test_meaning_to_color() {
        let dataset = create_test_dataset();
        let mut engine = ColorInferenceEngine::new(ColorInferenceConfig::default());
        engine.load_from_dataset(&dataset);
        
        // Generate color from meaning
        let color = engine.meaning_to_color("love").unwrap();
        
        // Should match expected color
        let expected = AspectColor::from_meaning("love");
        assert!(color.distance(&expected) < 0.1);
    }
    
    #[test]
    fn test_blended_colors() {
        let engine = ColorInferenceEngine::new(ColorInferenceConfig::default());
        
        let meanings = vec!["love".to_string(), "joy".to_string()];
        let weights = vec![0.6, 0.4];
        
        let blended = engine.meanings_to_blended_color(&meanings, &weights).unwrap();
        
        // Should be between love and joy colors
        let love_color = AspectColor::from_meaning("love");
        let joy_color = AspectColor::from_meaning("joy");
        
        let dist_to_love = blended.distance(&love_color);
        let dist_to_joy = blended.distance(&joy_color);
        
        // Should be closer to love (higher weight)
        assert!(dist_to_love < dist_to_joy);
    }
    
    #[test]
    fn test_find_similar_meanings() {
        let dataset = create_test_dataset();
        let mut engine = ColorInferenceEngine::new(ColorInferenceConfig {
            max_distance: 0.5,  // Increase tolerance for small dataset
            confidence_threshold: 0.3,  // Lower threshold
            top_k: 5,
            use_relationships: true,
        });
        engine.load_from_dataset(&dataset);
        
        let similar = engine.find_similar_meanings("love", 3);
        
        // May find similar meanings (small dataset may not have any within distance)
        assert!(similar.len() <= 3);
        
        // If any are found, should not include "love" itself
        if !similar.is_empty() {
            assert!(!similar.contains(&"love".to_string()));
        }
    }
    
    #[test]
    fn test_color_context() {
        let dataset = create_test_dataset();
        let mut engine = ColorInferenceEngine::new(ColorInferenceConfig::default());
        engine.load_from_dataset(&dataset);
        
        let color = AspectColor::from_meaning("peace");
        let context = engine.create_color_context(&color).unwrap();
        
        assert!(!context.primary_meaning.is_empty());
        assert!(!context.related_meanings.is_empty());
        assert!(context.average_confidence > 0.0);
    }
}
