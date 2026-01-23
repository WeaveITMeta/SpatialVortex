//! 🚀 Complete ASI Pipeline Demo - Integration of All Systems
//!
//! This example demonstrates the complete Artificial Superintelligence pipeline:
//! 1. Text Input
//! 2. ONNX Embedding + Sacred Geometry
//! 3. ELP Channel Mapping
//! 4. BeadTensor Creation
//! 5. FluxMatrix Positioning
//! 6. Confidence Lake Eligibility
//!
//! Run with:
//! ```bash
//! cargo run --example asi_complete_pipeline_demo --features onnx
//! ```

use spatial_vortex::inference_engine::asi_integration::ASIIntegrationEngine;
use spatial_vortex::inference_engine::vortex_math::FluxPosition;
use std::error::Error;

const SEPARATOR: &str = "════════════════════════════════════════════════════════════════════════════════";

fn main() -> Result<(), Box<dyn Error>> {
    println!("🚀 SpatialVortex Complete ASI Pipeline Demo 🚀\n");
    println!("{}", SEPARATOR);
    println!();

    // Initialize the ASI Integration Engine
    println!("⚙️  Initializing ASI Integration Engine...");
    let asi = ASIIntegrationEngine::new(
        "models/model.onnx",
        "models/tokenizer.json"
    )?;
    println!("✅ ASI Engine loaded successfully!\n");

    // Test phrases representing different semantic categories
    let test_cases = vec![
        ("Truth and justice must prevail", "High Ethos Expected"),
        ("Let us analyze the logical structure carefully", "High Logos Expected"),
        ("Love conquers all hearts and souls", "High Pathos Expected"),
        ("AI must serve humanity ethically and wisely", "Balanced ASI Philosophy"),
        ("The sacred geometry guides all decisions", "Mathematical Wisdom"),
        ("Random noise without meaning", "Low Signal Expected"),
    ];

    println!("{}", SEPARATOR);
    println!("🔮 Running Complete ASI Inference Pipeline");
    println!("{}", SEPARATOR);
    println!();

    for (i, (text, expected)) in test_cases.iter().enumerate() {
        println!("┌─ Test Case {} ────────────────────────────────────────────────", i + 1);
        println!("│");
        println!("│ 📝 Input Text: \"{}\"", text);
        println!("│ 🎯 Expected: {}", expected);
        println!("│");

        // 🌟 Run Complete ASI Pipeline
        let result = asi.infer(text)?;

        println!("│ ┌─ PIPELINE RESULTS ─────────────────────────────────");
        println!("│ │");
        
        // Step 1: Sacred Geometry
        println!("│ │ 🔺 Sacred Geometry Analysis:");
        println!("│ │ ├─ Confidence: {:.4}", result.bead.confidence);
        println!("│ │ ├─ Ethos (Character): {:.4} [{:>4.1}%]", 
            result.bead.elp_values.ethos / 13.0,
            (result.bead.elp_values.ethos / 13.0) * 100.0
        );
        println!("│ │ ├─ Logos (Logic):     {:.4} [{:>4.1}%]",
            result.bead.elp_values.logos / 13.0,
            (result.bead.elp_values.logos / 13.0) * 100.0
        );
        println!("│ │ └─ Pathos (Emotion):  {:.4} [{:>4.1}%]",
            result.bead.elp_values.pathos / 13.0,
            (result.bead.elp_values.pathos / 13.0) * 100.0
        );
        
        println!("│ │");
        
        // Step 2: FluxMatrix Position  
        println!("│ │ 🌀 FluxMatrix Positioning (Advanced Vortex Mathematics):");
        println!("│ │ ├─ Position: {} - {}", 
            result.flux_position.0,
            result.flux_position.name()
        );
        println!("│ │ └─ Archetype: {}", 
            archetype_symbol(&result.flux_position)
        );
        
        println!("│ │");
        
        // Step 3: Confidence Lake
        println!("│ │ 💎 Confidence Lake:");
        if result.lake_worthy {
            println!("│ │ └─ ✅ LAKE WORTHY (signal ≥ 0.6)");
            println!("│ │    High-quality semantic content");
        } else {
            println!("│ │ └─ ❌ NOT LAKE WORTHY (signal < 0.6)");
            println!("│ │    Content needs refinement");
        }
        
        println!("│ │");
        
        // Step 4: Interpretation
        println!("│ │ 💡 ASI Interpretation:");
        for line in result.interpretation.lines() {
            println!("│ │    {}", line);
        }
        
        println!("│ └────────────────────────────────────────────────────");
        println!("│");
        
        // Visual ELP Triangle
        println!("│ 🔺 Sacred Triangle Visualization:");
        println!("│");
        print_triangle(&result);
        
        println!("│");
        println!("└────────────────────────────────────────────────────────────────");
        println!();
    }

    println!("{}", SEPARATOR);
    println!("🎉 ASI Pipeline Complete!");
    println!("{}", SEPARATOR);
    println!();
    
    println!("📊 Pipeline Summary:");
    println!("├─ Total texts analyzed: {}", test_cases.len());
    println!("├─ Components integrated:");
    println!("│  ├─ ONNX Runtime (sentence-transformers) ✅");
    println!("│  ├─ Sacred Geometry (3-6-9 transform) ✅");
    println!("│  ├─ ELP Channel Mapping ✅");
    println!("│  ├─ FluxMatrix Positioning ✅");
    println!("│  └─ Confidence Lake Criteria ✅");
    println!("└─ Complete ASI inference pipeline operational! 🚀");
    println!();
    
    println!("💡 This demonstrates the unique SpatialVortex ASI capability:");
    println!("   Standard ML → Sacred Geometry → Interpretable Semantics");
    println!();

    Ok(())
}

fn archetype_symbol(pos: &FluxPosition) -> &'static str {
    if pos.is_divine_source() {
        "🌟 Divine Source (Perfect Balance)"
    } else if pos.is_sacred() {
        "🔺 Sacred Checkpoint (Stable Attractor)"
    } else if pos.is_in_vortex_flow() {
        "🌀 Vortex Flow (Dynamic Position)"
    } else {
        "Unknown"
    }
}

fn print_triangle(result: &spatial_vortex::inference_engine::asi_integration::ASIInferenceResult) {
    let e = result.bead.elp_values.ethos / 13.0;
    let l = result.bead.elp_values.logos / 13.0;
    let p = result.bead.elp_values.pathos / 13.0;
    
    // ASCII art sacred triangle
    println!("│          9 (Logos)");
    println!("│           /\\");
    println!("│          /  \\");
    println!("│         /    \\");
    println!("│        / {:.2} \\", l);
    println!("│       /        \\");
    println!("│      /          \\");
    println!("│     /            \\");
    println!("│    /______________\\");
    println!("│   3 {:.2}      {:.2} 6", e, p);
    println!("│ (Ethos)        (Pathos)");
    println!("│");
    
    // Energy bars
    println!("│ Energy Distribution:");
    println!("│ Ethos:  {}", "█".repeat((e * 20.0) as usize));
    println!("│ Logos:  {}", "█".repeat((l * 20.0) as usize));
    println!("│ Pathos: {}", "█".repeat((p * 20.0) as usize));
}
