//! 🌟 ONNX + Sacred Geometry Integration Demo 🌟
//!
//! This example demonstrates the innovative combination of:
//! - Standard ML embeddings (sentence-transformers)
//! - Sacred geometry (3-6-9 positions)
//! - Vortex mathematics (1→2→4→8→7→5→1)
//! - ELP channel analysis (Ethos, Logos, Pathos)
//!
//! **This is what makes SpatialVortex ASI unique!**
//!
//! Run with:
//! ```bash
//! cargo run --example onnx_sacred_geometry_demo --features onnx
//! ```

use spatial_vortex::inference_engine::onnx_runtime::OnnxInferenceEngine;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("🌟 SpatialVortex ASI - ONNX + Sacred Geometry Integration 🌟\n");

    // Initialize the inference engine
    println!("Loading ONNX model and tokenizer...");
    let engine = OnnxInferenceEngine::new(
        "models/model.onnx",
        "models/tokenizer.json"
    )?;
    println!("✅ Model loaded successfully!\n");
    println!("Model dimension: {}\n", engine.embedding_dim());

    // Test sentences with different semantic characteristics
    let test_sentences = vec![
        ("Truth and justice prevail", "High Ethos (Character)"),
        ("Let us analyze the logical structure", "High Logos (Logic)"),
        ("Love conquers all hearts", "High Pathos (Emotion)"),
        ("Balanced wisdom guides action", "Balanced ELP"),
        ("AI must serve humanity ethically", "ASI Philosophy"),
    ];

    println!("─".repeat(80));
    println!("🔮 Analyzing Sentences through Sacred Geometry 🔮");
    println!("─".repeat(80));
    println!();

    for (text, expected) in test_sentences {
        println!("📝 Text: \"{}\"", text);
        println!("   Expected: {}", expected);
        
        // 🌟 INNOVATION: Embed with Sacred Geometry transformation
        let (embedding, confidence, ethos, logos, pathos) = 
            engine.embed_with_sacred_geometry(text)?;
        
        println!();
        println!("   📊 Sacred Geometry Analysis:");
        println!("   ├─ Confidence:   {:.4} {}", 
            confidence,
            confidence_indicator(confidence)
        );
        println!("   └─ Embedding dim:     {}", embedding.len());
        
        println!();
        println!("   🎭 ELP Channel Distribution:");
        println!("   ├─ Ethos (Character): {:.4} {}", 
            ethos,
            bar_chart(ethos, 30)
        );
        println!("   ├─ Logos (Logic):     {:.4} {}", 
            logos,
            bar_chart(logos, 30)
        );
        println!("   └─ Pathos (Emotion):  {:.4} {}", 
            pathos,
            bar_chart(pathos, 30)
        );
        
        println!();
        println!("   🔺 Sacred Triangle Status:");
        println!("   Position 3 (Ethos):  {} = {:.4}", "●".repeat((ethos * 10.0) as usize), ethos);
        println!("   Position 6 (Pathos): {} = {:.4}", "●".repeat((pathos * 10.0) as usize), pathos);
        println!("   Position 9 (Logos):  {} = {:.4}", "●".repeat((logos * 10.0) as usize), logos);
        
        println!();
        println!("   ✨ Interpretation:");
        println!("   {}", interpret_elp(ethos, logos, pathos));
        
        println!();
        println!("─".repeat(80));
        println!();
    }

    println!("🎉 Sacred Geometry Integration Complete!");
    println!();
    println!("💡 Key Innovation:");
    println!("   Standard ML embeddings (sentence-transformers) are transformed");
    println!("   through sacred geometry (3-6-9 positions) and mapped to ELP channels.");
    println!("   This combines cutting-edge ML with mathematical foundations!");
    println!();
    println!("🚀 This is what makes SpatialVortex ASI unique!");

    Ok(())
}

/// Signal strength indicator
fn confidence_indicator(strength: f32) -> &'static str {
    match strength {
        s if s >= 0.7 => "⭐ Very Strong",
        s if s >= 0.5 => "✅ Strong",
        s if s >= 0.3 => "⚡ Moderate",
        _ => "⚠️  Weak",
    }
}

/// Simple bar chart visualization
fn bar_chart(value: f32, max_width: usize) -> String {
    let filled = ((value * max_width as f32) as usize).min(max_width);
    let empty = max_width - filled;
    format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
}

/// Interpret ELP channel distribution
fn interpret_elp(ethos: f32, logos: f32, pathos: f32) -> String {
    let max = ethos.max(logos).max(pathos);
    let balance = (ethos - 0.33).abs() + (logos - 0.33).abs() + (pathos - 0.33).abs();
    
    if balance < 0.15 {
        "Balanced across all three channels - harmonious sacred geometry!".to_string()
    } else if ethos == max {
        format!("Ethos-dominant ({:.1}%) - Strong character/ethical content", ethos * 100.0)
    } else if logos == max {
        format!("Logos-dominant ({:.1}%) - Strong logical/analytical content", logos * 100.0)
    } else {
        format!("Pathos-dominant ({:.1}%) - Strong emotional/empathetic content", pathos * 100.0)
    }
}
