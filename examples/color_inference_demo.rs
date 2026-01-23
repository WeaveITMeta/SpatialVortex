//! Example: Color-Aware Inference
//!
//! Demonstrates the complete color inference pipeline:
//! 1. Load trained color model
//! 2. Predict meanings from colors
//! 3. Generate colors from meanings
//! 4. Find similar semantic concepts
//! 5. Color-guided generation context

use spatial_vortex::ml::inference::{ColorInferenceEngine, ColorInferenceConfig};
use spatial_vortex::ml::training::{ColorDatasetGenerator, ColorDatasetConfig};
use spatial_vortex::data::AspectColor;

fn main() -> anyhow::Result<()> {
    println!("🎨 Color-Aware Inference Demo");
    println!("{}", "=".repeat(70));
    println!();
    
    // ========================================================================
    // Phase 1: Load Trained Model
    // ========================================================================
    
    println!("📊 Phase 1: Loading Trained Color Model");
    println!("{}", "-".repeat(70));
    
    // Generate training dataset
    let generator = ColorDatasetGenerator::create_emotional_dataset();
    let dataset = generator.generate();
    
    // Create inference engine
    let mut engine = ColorInferenceEngine::new(ColorInferenceConfig {
        max_distance: 0.3,
        confidence_threshold: 0.5,
        top_k: 5,
        use_relationships: true,
    });
    
    // Load color associations
    engine.load_from_dataset(&dataset);
    
    println!("✅ Loaded {} color-meaning associations", dataset.samples().len());
    println!();
    
    // ========================================================================
    // Phase 2: Color → Meaning Prediction
    // ========================================================================
    
    println!("🔮 Phase 2: Predicting Meanings from Colors");
    println!("{}", "-".repeat(70));
    
    let test_colors = vec![
        ("Red (passionate)", AspectColor::from_hsl(0.0, 0.8, 0.5)),
        ("Blue (calm)", AspectColor::from_hsl(210.0, 0.7, 0.5)),
        ("Green (growth)", AspectColor::from_hsl(120.0, 0.6, 0.5)),
        ("Yellow (joyful)", AspectColor::from_hsl(55.0, 0.9, 0.6)),
        ("Purple (mysterious)", AspectColor::from_hsl(280.0, 0.7, 0.5)),
    ];
    
    for (name, color) in &test_colors {
        println!("\n🎨 {}", name);
        println!("   Color: {}", color.to_hex());
        
        let predictions = engine.color_to_meaning(color);
        
        if predictions.is_empty() {
            println!("   No predictions found");
        } else {
            println!("   Top predictions:");
            for (i, pred) in predictions.iter().take(3).enumerate() {
                println!("   {}. {} (confidence: {:.2}, distance: {:.3})",
                         i + 1, pred.meaning, pred.confidence, pred.distance);
            }
        }
    }
    
    println!();
    
    // ========================================================================
    // Phase 3: Meaning → Color Generation
    // ========================================================================
    
    println!("🌈 Phase 3: Generating Colors from Meanings");
    println!("{}", "-".repeat(70));
    
    let test_meanings = vec![
        "love", "joy", "peace", "courage", "wisdom",
        "sadness", "anger", "fear", "mystery", "hope",
    ];
    
    println!("\nMeaning → Color mappings:");
    for meaning in &test_meanings {
        let color = engine.meaning_to_color(meaning)?;
        println!("  • {:<12} → {} HSL({:.0}°, {:.0}%, {:.0}%)",
                 meaning,
                 color.to_hex(),
                 color.hue,
                 color.saturation * 100.0,
                 color.luminance * 100.0);
    }
    
    println!();
    
    // ========================================================================
    // Phase 4: Blended Colors
    // ========================================================================
    
    println!("🎨 Phase 4: Blending Multiple Meanings");
    println!("{}", "-".repeat(70));
    
    let blends = vec![
        (vec!["love".to_string(), "joy".to_string()], vec![0.6, 0.4], "Loving Joy"),
        (vec!["courage".to_string(), "wisdom".to_string()], vec![0.5, 0.5], "Wise Courage"),
        (vec!["peace".to_string(), "hope".to_string()], vec![0.7, 0.3], "Hopeful Peace"),
    ];
    
    for (meanings, weights, description) in &blends {
        let blended = engine.meanings_to_blended_color(meanings, weights)?;
        println!("\n{}", description);
        println!("  Meanings: {} + {}", meanings[0], meanings[1]);
        println!("  Weights: {:.0}% + {:.0}%", weights[0] * 100.0, weights[1] * 100.0);
        println!("  Result: {} HSL({:.0}°, {:.0}%, {:.0}%)",
                 blended.to_hex(),
                 blended.hue,
                 blended.saturation * 100.0,
                 blended.luminance * 100.0);
    }
    
    println!();
    
    // ========================================================================
    // Phase 5: Find Similar Meanings
    // ========================================================================
    
    println!("🔍 Phase 5: Finding Similar Semantic Concepts");
    println!("{}", "-".repeat(70));
    
    let query_meanings = vec!["love", "anger", "peace", "courage"];
    
    for meaning in &query_meanings {
        let similar = engine.find_similar_meanings(meaning, 3);
        println!("\n\"{}\" is similar to:", meaning);
        for (i, sim) in similar.iter().enumerate() {
            println!("  {}. {}", i + 1, sim);
        }
    }
    
    println!();
    
    // ========================================================================
    // Phase 6: Color-Guided Generation Context
    // ========================================================================
    
    println!("📝 Phase 6: Color-Guided Generation Context");
    println!("{}", "-".repeat(70));
    
    let guide_colors = vec![
        ("Peaceful Blue", AspectColor::from_hsl(210.0, 0.6, 0.5)),
        ("Energetic Red", AspectColor::from_hsl(5.0, 0.8, 0.5)),
        ("Balanced Green", AspectColor::from_hsl(120.0, 0.6, 0.5)),
    ];
    
    for (name, color) in &guide_colors {
        println!("\n{}", name);
        let context = engine.create_color_context(color)?;
        
        println!("  Primary meaning: {}", context.primary_meaning);
        println!("  Related meanings: [{}]", context.related_meanings.join(", "));
        println!("  Confidence: {:.2}", context.average_confidence);
        println!("\n  Prompt context:");
        println!("  {}", context.to_prompt_context());
    }
    
    println!();
    
    // ========================================================================
    // Phase 7: Performance Statistics
    // ========================================================================
    
    println!("📊 Phase 7: Inference Statistics");
    println!("{}", "-".repeat(70));
    
    let stats = engine.stats();
    println!("\nStatistics:");
    println!("  • Total predictions loaded: {}", stats.total_predictions);
    println!("  • Top-k per query: {}", stats.average_predictions_per_query);
    println!();
    
    // ========================================================================
    // Summary
    // ========================================================================
    
    println!("{}", "=".repeat(70));
    println!("✅ Color Inference Demo Complete!");
    println!();
    println!("Capabilities demonstrated:");
    println!("  ✅ Color → Meaning prediction");
    println!("  ✅ Meaning → Color generation");
    println!("  ✅ Multi-meaning color blending");
    println!("  ✅ Semantic similarity search");
    println!("  ✅ Color-guided generation context");
    println!();
    println!("Next steps:");
    println!("  • Integrate with full InferenceEngine");
    println!("  • Add to ASI Orchestrator");
    println!("  • Performance benchmarking");
    println!("  • Production deployment");
    
    Ok(())
}
