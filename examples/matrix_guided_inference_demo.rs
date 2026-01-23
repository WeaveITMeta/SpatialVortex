//! Matrix-Guided Inference Demo
//!
//! Demonstrates Phase 3: Matrix-Guided Inference
//! - Extract semantic knowledge from FluxMatrix positions
//! - Build enhanced prompts with position-specific context
//! - Analyze response quality using semantic associations

use spatial_vortex::core::sacred_geometry::{
    FluxMatrixEngine, MatrixGuidedInference, EnhancementMode,
};
use spatial_vortex::subject_definitions;

fn main() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║     Matrix-Guided Inference Demonstration (Phase 3)     ║");
    println!("║                                                          ║");
    println!("║  Semantic Enhancement · Quality Analysis · Smart Prompts║");
    println!("╚══════════════════════════════════════════════════════════╝\n");
    
    // Create flux matrix engine
    let flux_engine = FluxMatrixEngine::new();
    
    // Load subject definitions from Phase 2
    println!("📚 Loading Subject Definitions from Phase 2...\n");
    
    // Load cognition subject
    let cognition_def = subject_definitions::cognition::definition();
    println!("✓ Loaded: {} ({} nodes)", cognition_def.name, cognition_def.nodes.len());
    
    // Load consciousness subject
    let consciousness_def = subject_definitions::consciousness::definition();
    println!("✓ Loaded: {} ({} nodes)", consciousness_def.name, consciousness_def.nodes.len());
    
    // Load ethics subject
    let ethics_def = subject_definitions::ethics::definition();
    println!("✓ Loaded: {} ({} nodes)\n", ethics_def.name, ethics_def.nodes.len());
    
    // Create matrix-guided inference engine
    let mgi = MatrixGuidedInference::new(flux_engine.clone());
    
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // Demo 1: Extract context for a cognition query
    demo_1_context_extraction(&mgi);
    
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // Demo 2: Build enhanced prompts with different modes
    demo_2_prompt_enhancement(&mgi);
    
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // Demo 3: Analyze response quality
    demo_3_quality_analysis(&mgi);
    
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║                   Demo Complete! ✅                      ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");
    
    println!("🎯 Key Achievements Demonstrated:");
    println!("   ✓ Context extraction from semantic associations");
    println!("   ✓ Multi-mode prompt enhancement (Full/Summary/Minimal)");
    println!("   ✓ Response quality analysis with metrics");
    println!("   ✓ Sacred position detection and enhancement");
    println!("   ✓ ELP-aware contextual prompts\n");
    
    println!("   Matrix-Guided Inference transforms prompts from GENERIC → CONTEXTUAL!\n");
}

fn demo_1_context_extraction(mgi: &MatrixGuidedInference) {
    println!("📊 Demo 1: Context Extraction from FluxMatrix");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    let query = "How does logical reasoning work?";
    let subject = "cognition";
    
    println!("Query: \"{}\"\n", query);
    
    match mgi.extract_context(query, subject) {
        Ok(context) => {
            println!("🔧 Extracted Matrix Context:");
            println!("   Subject: {}", context.subject);
            println!("   Position: {}", context.position);
            println!("   Neutral Base: {}", context.neutral_base);
            println!("   Is Sacred: {}", if context.is_sacred { "🌟 YES" } else { "No" });
            println!("   ELP: Ethos={:.1}, Logos={:.1}, Pathos={:.1}",
                context.elp_context.ethos,
                context.elp_context.logos,
                context.elp_context.pathos
            );
            
            if !context.positive_associations.is_empty() {
                println!("\n   📈 Top Positive Associations:");
                for (i, assoc) in context.positive_associations.iter().take(5).enumerate() {
                    println!("      {}. {} (confidence: {:.2})",
                        i + 1, assoc.word, assoc.confidence
                    );
                }
            }
            
            if !context.negative_associations.is_empty() {
                println!("\n   ⚠️  Negative Associations (to avoid):");
                for (i, assoc) in context.negative_associations.iter().take(3).enumerate() {
                    println!("      {}. {}", i + 1, assoc.word);
                }
            }
            
            if !context.related_subjects.is_empty() {
                println!("\n   🔗 Related Subjects: {}", context.related_subjects.join(", "));
            }
        }
        Err(e) => println!("❌ Error: {}", e),
    }
}

fn demo_2_prompt_enhancement(mgi: &MatrixGuidedInference) {
    println!("📊 Demo 2: Prompt Enhancement Modes");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    let query = "Explain consciousness";
    let subject = "consciousness";
    
    println!("Original Query: \"{}\"\n", query);
    
    // Test Minimal mode
    println!("🟦 MINIMAL Mode (Top 3 associations only):");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    match mgi.enhance_query(query, subject, EnhancementMode::Minimal) {
        Ok(enhanced) => println!("{}\n", enhanced),
        Err(e) => println!("❌ Error: {}\n", e),
    }
    
    // Test Summary mode
    println!("🟨 SUMMARY Mode (Context + Top associations):");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    match mgi.enhance_query(query, subject, EnhancementMode::Summary) {
        Ok(enhanced) => println!("{}\n", enhanced),
        Err(e) => println!("❌ Error: {}\n", e),
    }
    
    // Test Full mode
    println!("🟩 FULL Mode (Complete semantic context):");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    match mgi.enhance_query(query, subject, EnhancementMode::Full) {
        Ok(enhanced) => {
            // Show first 500 chars
            let preview = if enhanced.len() > 500 {
                format!("{}...\n[{} more characters]", &enhanced[..500], enhanced.len() - 500)
            } else {
                enhanced
            };
            println!("{}\n", preview);
        }
        Err(e) => println!("❌ Error: {}\n", e),
    }
}

fn demo_3_quality_analysis(mgi: &MatrixGuidedInference) {
    println!("📊 Demo 3: Response Quality Analysis");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    let query = "What is the nature of thinking?";
    let subject = "cognition";
    
    // Simulate a good response (mentions positive associations)
    let good_response = "Thinking involves reasoning, logic, and analysis. \
        It's a process of cognition that includes metacognition and reflection. \
        Through critical thinking, we engage in deduction and problem-solving.";
    
    // Simulate a poor response (mentions negative associations)
    let poor_response = "Thinking is when you're not being illogical or fallacious. \
        It's the opposite of confusion and stupidity.";
    
    println!("Query: \"{}\"\n", query);
    
    // Analyze good response
    println!("✅ Analyzing GOOD Response:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    match mgi.analyze_response_quality(query, good_response, subject) {
        Ok(analysis) => print_quality_analysis(&analysis, good_response),
        Err(e) => println!("❌ Error: {}", e),
    }
    
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // Analyze poor response
    println!("⚠️  Analyzing POOR Response:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    match mgi.analyze_response_quality(query, poor_response, subject) {
        Ok(analysis) => print_quality_analysis(&analysis, poor_response),
        Err(e) => println!("❌ Error: {}", e),
    }
}

fn print_quality_analysis(analysis: &spatial_vortex::core::sacred_geometry::ResponseQualityAnalysis, response: &str) {
    println!("Response: \"{}\"\n", response);
    
    println!("📊 Quality Metrics:");
    println!("   Overall Score: {:.3}", analysis.quality_score);
    println!("   Quality Level: {}", analysis.quality_level);
    println!("   Position Used: {}{}", 
        analysis.position,
        if analysis.is_sacred { " 🌟 (Sacred)" } else { "" }
    );
    println!("   Positive Matches: {}", analysis.positive_matches);
    println!("   Negative Matches: {} {}", 
        analysis.negative_matches,
        if analysis.negative_matches > 0 { "⚠️" } else { "✓" }
    );
    
    if !analysis.positive_matched.is_empty() {
        println!("\n   ✅ Matched Positive Associations:");
        for (i, word) in analysis.positive_matched.iter().enumerate() {
            println!("      {}. {}", i + 1, word);
        }
    }
    
    if !analysis.negative_matched.is_empty() {
        println!("\n   ⚠️  Matched Negative Associations:");
        for (i, word) in analysis.negative_matched.iter().enumerate() {
            println!("      {}. {}", i + 1, word);
        }
    }
}
