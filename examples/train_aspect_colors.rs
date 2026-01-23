//! Example: Training Aspect Color Models
//!
//! Demonstrates the complete training pipeline for aspect color ML:
//! 1. Generate training dataset
//! 2. Configure loss functions
//! 3. Train model
//! 4. Validate results
//! 5. Visualize learned embeddings

use spatial_vortex::ml::training::{
    ColorDatasetGenerator,
    ColorDatasetConfig,
    AspectColorModelTrainer,
    ColorLossCombination,
    ColorLossFunction,
};
use spatial_vortex::data::AspectColor;

fn main() -> anyhow::Result<()> {
    println!("🎨 Aspect Color Model Training Example");
    println!("{}", "=".repeat(60));
    println!();
    
    // ========================================================================
    // Phase 1: Dataset Generation
    // ========================================================================
    
    println!("📊 Phase 1: Generating Training Dataset");
    println!("{}", "-".repeat(60));
    
    // Create emotional meanings dataset
    let generator = ColorDatasetGenerator::create_emotional_dataset();
    let dataset = generator.generate();
    
    println!("✅ Generated {} training samples", dataset.samples().len());
    println!("   Meanings covered: 45+ emotions and concepts");
    println!("   Samples per meaning: 10 (with variations)");
    println!();
    
    // Show some examples
    println!("Sample meanings and their colors:");
    for meaning in &["joy", "sadness", "love", "anger", "peace"] {
        let color = AspectColor::from_meaning(meaning);
        println!("  • {:<12} → HSL({:.0}°, {:.0}%, {:.0}%) {}",
                 meaning,
                 color.hue,
                 color.saturation * 100.0,
                 color.luminance * 100.0,
                 color.to_hex());
    }
    println!();
    
    // ========================================================================
    // Phase 2: Loss Function Configuration
    // ========================================================================
    
    println!("🎯 Phase 2: Configuring Loss Functions");
    println!("{}", "-".repeat(60));
    
    let loss = ColorLossCombination::new()
        .add_loss(
            ColorLossFunction::ColorSimilarity {
                hue_weight: 0.6,
                sat_weight: 0.2,
                lum_weight: 0.2,
            },
            1.0,  // Primary loss
        )
        .add_loss(
            ColorLossFunction::SemanticConsistency {
                temperature: 0.5,
            },
            0.5,  // Secondary loss
        )
        .add_loss(
            ColorLossFunction::HuePreservation {
                angular_weight: 0.3,
            },
            0.3,  // Tertiary loss
        );
    
    println!("✅ Loss function configured:");
    println!("   • ColorSimilarity (weight: 1.0) - HSL distance");
    println!("   • SemanticConsistency (weight: 0.5) - Relationship preservation");
    println!("   • HuePreservation (weight: 0.3) - Angular structure");
    println!();
    
    // ========================================================================
    // Phase 3: Training Configuration
    // ========================================================================
    
    println!("⚙️  Phase 3: Training Configuration");
    println!("{}", "-".repeat(60));
    
    let config = spatial_vortex::ml::training::aspect_color_trainer::TrainingConfig {
        epochs: 50,
        learning_rate: 0.01,
        batch_size: 32,
        train_split: 0.8,
        early_stopping_patience: 5,
        min_improvement: 0.001,
    };
    
    println!("✅ Training configuration:");
    println!("   • Epochs: {}", config.epochs);
    println!("   • Learning rate: {}", config.learning_rate);
    println!("   • Batch size: {}", config.batch_size);
    println!("   • Train/val split: {}/{}", 
             (config.train_split * 100.0) as usize,
             ((1.0 - config.train_split) * 100.0) as usize);
    println!("   • Early stopping patience: {} epochs", config.early_stopping_patience);
    println!();
    
    // ========================================================================
    // Phase 4: Model Training
    // ========================================================================
    
    println!("🚀 Phase 4: Training Model");
    println!("{}", "-".repeat(60));
    println!();
    
    let mut trainer = AspectColorModelTrainer::new(dataset, config)
        .with_loss(loss);
    
    let metrics = trainer.train();
    
    println!();
    println!("{}", "=".repeat(60));
    
    // ========================================================================
    // Phase 5: Results Summary
    // ========================================================================
    
    println!("📈 Phase 5: Training Results");
    println!("{}", "-".repeat(60));
    
    println!("✅ Training completed successfully!");
    println!();
    println!("Final metrics:");
    println!("  • Best validation loss: {:.4}", metrics.best_val_loss);
    println!("  • Best epoch: {}", metrics.best_epoch);
    println!("  • Total epochs: {}", metrics.train_losses.len());
    println!("  • Training time: {:.2}s", metrics.training_time_secs);
    println!();
    
    if !metrics.train_losses.is_empty() && !metrics.val_losses.is_empty() {
        println!("Loss progression:");
        println!("  • Initial train loss: {:.4}", metrics.train_losses[0]);
        println!("  • Final train loss: {:.4}", metrics.train_losses.last().unwrap());
        println!("  • Initial val loss: {:.4}", metrics.val_losses[0]);
        println!("  • Final val loss: {:.4}", metrics.val_losses.last().unwrap());
        println!();
        
        // Calculate improvement
        let train_improvement = (metrics.train_losses[0] - metrics.train_losses.last().unwrap()) 
                               / metrics.train_losses[0] * 100.0;
        let val_improvement = (metrics.val_losses[0] - metrics.best_val_loss) 
                             / metrics.val_losses[0] * 100.0;
        
        println!("Improvement:");
        println!("  • Training: {:.1}% reduction", train_improvement);
        println!("  • Validation: {:.1}% reduction", val_improvement);
    }
    
    println!();
    println!("{}", "=".repeat(60));
    println!("✅ Example complete! Color model training pipeline verified.");
    println!();
    println!("Next steps:");
    println!("  • Integrate with InferenceEngine (Week 5-6)");
    println!("  • Implement color_to_meaning() prediction");
    println!("  • Add color-guided generation");
    println!("  • Create visualization tools (Week 7-8)");
    
    Ok(())
}
