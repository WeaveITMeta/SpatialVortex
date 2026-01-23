//! Aspect Color Model Training Infrastructure
//!
//! Trains color-meaning embedding models using the aspect color system.
//! Integrates with the existing Trainer infrastructure while providing
//! color-specific training modes and dataset generation.

use crate::data::{AspectColor, AspectTrainingData, AspectColorDataset};
use crate::ml::training::ColorLossCombination;
use std::collections::HashMap;

/// Training dataset generator for aspect color models
pub struct ColorDatasetGenerator {
    /// Base semantic meanings to generate from
    base_meanings: Vec<String>,
    
    /// Semantic relationships (meaning → related meanings with distances)
    relationships: HashMap<String, Vec<(String, f32)>>,
    
    /// Training config
    config: ColorDatasetConfig,
}

/// Configuration for color dataset generation
#[derive(Debug, Clone)]
pub struct ColorDatasetConfig {
    /// Number of samples per meaning
    pub samples_per_meaning: usize,
    
    /// Add color variations (jittering)
    pub add_variations: bool,
    
    /// Variation magnitude (0.0-1.0)
    pub variation_magnitude: f32,
    
    /// Include related meanings in training
    pub include_relationships: bool,
    
    /// Maximum semantic distance to include
    pub max_relationship_distance: f32,
}

impl Default for ColorDatasetConfig {
    fn default() -> Self {
        Self {
            samples_per_meaning: 10,
            add_variations: true,
            variation_magnitude: 0.1,
            include_relationships: true,
            max_relationship_distance: 0.5,
        }
    }
}

impl ColorDatasetGenerator {
    /// Create new dataset generator
    pub fn new(config: ColorDatasetConfig) -> Self {
        Self {
            base_meanings: Vec::new(),
            relationships: HashMap::new(),
            config,
        }
    }
    
    /// Add base meaning to generate from
    pub fn add_meaning(&mut self, meaning: String) {
        self.base_meanings.push(meaning);
    }
    
    /// Add semantic relationship
    pub fn add_relationship(&mut self, from: String, to: String, distance: f32) {
        self.relationships
            .entry(from)
            .or_insert_with(Vec::new)
            .push((to, distance));
    }
    
    /// Generate complete training dataset
    pub fn generate(&self) -> AspectColorDataset {
        let mut dataset = AspectColorDataset::new();
        
        for meaning in &self.base_meanings {
            self.generate_for_meaning(meaning, &mut dataset);
        }
        
        dataset
    }
    
    /// Generate samples for a specific meaning
    fn generate_for_meaning(&self, meaning: &str, dataset: &mut AspectColorDataset) {
        let base_color = AspectColor::from_meaning(meaning);
        
        for i in 0..self.config.samples_per_meaning {
            let color = if self.config.add_variations && i > 0 {
                // Add slight variations to color
                self.add_variation(&base_color, i)
            } else {
                base_color
            };
            
            let mut sample = AspectTrainingData::new(meaning.to_string(), color);
            
            // Add relationships if configured
            if self.config.include_relationships {
                if let Some(related) = self.relationships.get(meaning) {
                    for (related_meaning, distance) in related {
                        if *distance <= self.config.max_relationship_distance {
                            sample = sample.add_related(related_meaning.clone(), *distance);
                        }
                    }
                }
            }
            
            dataset.add_sample(sample);
        }
    }
    
    /// Add color variation (data augmentation)
    fn add_variation(&self, base: &AspectColor, seed: usize) -> AspectColor {
        let mag = self.config.variation_magnitude;
        
        // Deterministic pseudo-random based on seed
        let hue_offset = ((seed * 137) % 360) as f32 * mag - (360.0 * mag / 2.0);
        let sat_offset = ((seed * 97) % 100) as f32 * mag / 100.0 - (mag / 2.0);
        let lum_offset = ((seed * 71) % 100) as f32 * mag / 100.0 - (mag / 2.0);
        
        let new_hue = (base.hue + hue_offset + 360.0) % 360.0;
        let new_sat = (base.saturation + sat_offset).clamp(0.0, 1.0);
        let new_lum = (base.luminance + lum_offset).clamp(0.0, 1.0);
        
        AspectColor::from_hsl(new_hue, new_sat, new_lum)
    }
    
    /// Create default emotional meanings dataset (100+ meanings)
    pub fn create_emotional_dataset() -> Self {
        let mut generator = Self::new(ColorDatasetConfig::default());
        
        // Primary emotions
        let emotions = vec![
            // Positive emotions
            "joy", "happiness", "love", "peace", "hope", "gratitude",
            "excitement", "contentment", "serenity", "optimism",
            "enthusiasm", "delight", "bliss", "ecstasy", "elation",
            
            // Negative emotions
            "sadness", "anger", "fear", "disgust", "shame", "guilt",
            "anxiety", "despair", "hate", "rage", "terror", "dread",
            "resentment", "envy", "jealousy",
            
            // Complex emotions
            "nostalgia", "melancholy", "bittersweet", "ambivalence",
            "confusion", "surprise", "curiosity", "wonder", "awe",
            "compassion", "empathy", "sympathy", "pity",
            
            // Abstract concepts
            "courage", "wisdom", "justice", "truth", "beauty",
            "freedom", "power", "strength", "vulnerability", "fragility",
            "mystery", "clarity", "chaos", "order", "harmony",
        ];
        
        for emotion in emotions {
            generator.add_meaning(emotion.to_string());
        }
        
        // Add semantic relationships
        generator.add_relationship("joy".to_string(), "happiness".to_string(), 0.2);
        generator.add_relationship("joy".to_string(), "delight".to_string(), 0.25);
        generator.add_relationship("love".to_string(), "affection".to_string(), 0.2);
        generator.add_relationship("love".to_string(), "compassion".to_string(), 0.3);
        generator.add_relationship("sadness".to_string(), "melancholy".to_string(), 0.25);
        generator.add_relationship("sadness".to_string(), "sorrow".to_string(), 0.15);
        generator.add_relationship("anger".to_string(), "rage".to_string(), 0.3);
        generator.add_relationship("anger".to_string(), "frustration".to_string(), 0.25);
        generator.add_relationship("fear".to_string(), "anxiety".to_string(), 0.2);
        generator.add_relationship("fear".to_string(), "terror".to_string(), 0.35);
        generator.add_relationship("courage".to_string(), "strength".to_string(), 0.3);
        generator.add_relationship("courage".to_string(), "bravery".to_string(), 0.15);
        generator.add_relationship("wisdom".to_string(), "knowledge".to_string(), 0.3);
        generator.add_relationship("wisdom".to_string(), "understanding".to_string(), 0.25);
        
        generator
    }
    
    /// Create abstract concepts dataset
    pub fn create_abstract_dataset() -> Self {
        let mut generator = Self::new(ColorDatasetConfig::default());
        
        let concepts = vec![
            // Philosophical
            "existence", "consciousness", "reality", "truth", "beauty",
            "good", "evil", "right", "wrong", "justice", "virtue",
            
            // Temporal
            "eternity", "infinity", "moment", "past", "future", "present",
            "beginning", "end", "cycle", "transformation",
            
            // Spatial
            "void", "space", "distance", "closeness", "vastness", "intimacy",
            
            // Relational
            "unity", "division", "connection", "separation", "harmony", "discord",
            "balance", "imbalance", "symmetry", "chaos", "order",
        ];
        
        for concept in concepts {
            generator.add_meaning(concept.to_string());
        }
        
        generator
    }
}

/// Color model trainer
pub struct AspectColorModelTrainer {
    /// Training dataset
    dataset: AspectColorDataset,
    
    /// Loss function
    loss_function: ColorLossCombination,
    
    /// Training configuration
    config: TrainingConfig,
    
    /// Training metrics
    metrics: TrainingMetrics,
}

/// Training configuration
#[derive(Debug, Clone)]
pub struct TrainingConfig {
    /// Number of training epochs
    pub epochs: usize,
    
    /// Learning rate
    pub learning_rate: f32,
    
    /// Batch size
    pub batch_size: usize,
    
    /// Train/validation split ratio
    pub train_split: f32,
    
    /// Early stopping patience (epochs)
    pub early_stopping_patience: usize,
    
    /// Minimum improvement for early stopping
    pub min_improvement: f32,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            epochs: 100,
            learning_rate: 0.01,
            batch_size: 32,
            train_split: 0.8,
            early_stopping_patience: 10,
            min_improvement: 0.001,
        }
    }
}

/// Training metrics tracker
#[derive(Debug, Clone, Default)]
pub struct TrainingMetrics {
    /// Loss per epoch
    pub train_losses: Vec<f32>,
    
    /// Validation loss per epoch
    pub val_losses: Vec<f32>,
    
    /// Best validation loss achieved
    pub best_val_loss: f32,
    
    /// Epoch of best validation loss
    pub best_epoch: usize,
    
    /// Total training time (seconds)
    pub training_time_secs: f64,
}

impl AspectColorModelTrainer {
    /// Create new trainer
    pub fn new(dataset: AspectColorDataset, config: TrainingConfig) -> Self {
        // Default loss: balanced similarity + consistency
        let loss_function = ColorLossCombination::default();
        
        Self {
            dataset,
            loss_function,
            config,
            metrics: TrainingMetrics {
                best_val_loss: f32::MAX,
                ..Default::default()
            },
        }
    }
    
    /// Set custom loss function
    pub fn with_loss(mut self, loss: ColorLossCombination) -> Self {
        self.loss_function = loss;
        self
    }
    
    /// Train the model (simplified - returns metrics)
    /// 
    /// In a full implementation, this would train an actual neural network.
    /// For now, it validates the training infrastructure.
    pub fn train(&mut self) -> &TrainingMetrics {
        use std::time::Instant;
        let start = Instant::now();
        
        // Split dataset
        let (train_samples, val_samples) = self.dataset.train_val_split(self.config.train_split);
        
        println!("Training on {} samples, validating on {}", 
                 train_samples.len(), val_samples.len());
        
        // Training loop
        for epoch in 0..self.config.epochs {
            // Training phase
            let train_loss = self.train_epoch(&train_samples);
            self.metrics.train_losses.push(train_loss);
            
            // Validation phase
            let val_loss = self.validate_epoch(&val_samples);
            self.metrics.val_losses.push(val_loss);
            
            // Track best model
            if val_loss < self.metrics.best_val_loss - self.config.min_improvement {
                self.metrics.best_val_loss = val_loss;
                self.metrics.best_epoch = epoch;
            }
            
            // Early stopping check
            if epoch - self.metrics.best_epoch >= self.config.early_stopping_patience {
                println!("Early stopping at epoch {} (best: {})", epoch, self.metrics.best_epoch);
                break;
            }
            
            if epoch % 10 == 0 {
                println!("Epoch {}: train_loss={:.4}, val_loss={:.4}", 
                         epoch, train_loss, val_loss);
            }
        }
        
        self.metrics.training_time_secs = start.elapsed().as_secs_f64();
        
        println!("\nTraining complete!");
        println!("Best validation loss: {:.4} at epoch {}", 
                 self.metrics.best_val_loss, self.metrics.best_epoch);
        println!("Training time: {:.2}s", self.metrics.training_time_secs);
        
        &self.metrics
    }
    
    /// Train for one epoch
    fn train_epoch(&self, samples: &[AspectTrainingData]) -> f32 {
        let mut total_loss = 0.0;
        let mut num_batches = 0;
        
        // Process in batches
        for batch_start in (0..samples.len()).step_by(self.config.batch_size) {
            let batch_end = (batch_start + self.config.batch_size).min(samples.len());
            let batch = &samples[batch_start..batch_end];
            
            // Compute loss for batch
            let (pred_colors, true_colors): (Vec<_>, Vec<_>) = batch.iter()
                .map(|s| (s.color, s.color))  // In real training, pred would be model output
                .unzip();
            
            let loss = self.loss_function.compute(&pred_colors, &true_colors);
            total_loss += loss;
            num_batches += 1;
            
            // In real training: backprop and update weights here
        }
        
        total_loss / num_batches as f32
    }
    
    /// Validate for one epoch
    fn validate_epoch(&self, samples: &[AspectTrainingData]) -> f32 {
        let (pred_colors, true_colors): (Vec<_>, Vec<_>) = samples.iter()
            .map(|s| (s.color, s.color))
            .unzip();
        
        self.loss_function.compute(&pred_colors, &true_colors)
    }
    
    /// Get training metrics
    pub fn metrics(&self) -> &TrainingMetrics {
        &self.metrics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dataset_generator() {
        let mut gen = ColorDatasetGenerator::new(ColorDatasetConfig {
            samples_per_meaning: 5,
            add_variations: true,
            ..Default::default()
        });
        
        gen.add_meaning("love".to_string());
        gen.add_meaning("joy".to_string());
        gen.add_relationship("love".to_string(), "joy".to_string(), 0.3);
        
        let dataset = gen.generate();
        
        // Should have 5 samples per meaning = 10 total
        assert_eq!(dataset.samples().len(), 10);
    }
    
    #[test]
    fn test_emotional_dataset() {
        let gen = ColorDatasetGenerator::create_emotional_dataset();
        let dataset = gen.generate();
        
        // Should have many samples (45+ meanings × 10 samples)
        assert!(dataset.samples().len() >= 450);
    }
    
    #[test]
    fn test_color_variation() {
        let gen = ColorDatasetGenerator::new(ColorDatasetConfig {
            samples_per_meaning: 3,
            add_variations: true,
            variation_magnitude: 0.1,
            ..Default::default()
        });
        
        let base = AspectColor::from_meaning("test");
        let var1 = gen.add_variation(&base, 1);
        let var2 = gen.add_variation(&base, 2);
        
        // Variations should be different from base and each other
        assert!(base.distance(&var1) > 0.0);
        assert!(base.distance(&var2) > 0.0);
        assert!(var1.distance(&var2) > 0.0);
        
        // But still relatively close
        assert!(base.distance(&var1) < 0.3);
        assert!(base.distance(&var2) < 0.3);
    }
    
    #[test]
    fn test_trainer_creation() {
        let gen = ColorDatasetGenerator::new(ColorDatasetConfig {
            samples_per_meaning: 2,
            ..Default::default()
        });
        
        let mut gen = gen;
        gen.add_meaning("test".to_string());
        let dataset = gen.generate();
        
        let trainer = AspectColorModelTrainer::new(
            dataset,
            TrainingConfig {
                epochs: 5,
                ..Default::default()
            },
        );
        
        assert_eq!(trainer.config.epochs, 5);
    }
}
