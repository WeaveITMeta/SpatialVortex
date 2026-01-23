//! First Principles Reasoning Demo
//!
//! Demonstrates truth detection, lie detection, sarcasm detection, and uncertainty analysis.

use spatial_vortex::agents::first_principles::{FirstPrinciplesReasoner, TruthAssessment};

fn main() {
    println!("🧠 First Principles Reasoning Demo\n");
    println!("═══════════════════════════════════════════════════════════════\n");
    
    let reasoner = FirstPrinciplesReasoner::new();
    
    // Test 1: Logical Contradiction
    println!("📋 Test 1: Logical Contradiction\n");
    let result1 = reasoner.analyze("The sky is blue. The sky is not blue.");
    print_result(&result1);
    
    // Test 2: Sarcasm Detection
    println!("\n📋 Test 2: Sarcasm Detection\n");
    let result2 = reasoner.analyze("Oh great, another rainy day. Just what I needed.");
    print_result(&result2);
    
    // Test 3: Opinion vs Fact
    println!("\n📋 Test 3: Opinion Detection\n");
    let result3 = reasoner.analyze("I think chocolate ice cream is the best flavor.");
    print_result(&result3);
    
    // Test 4: Deception Pattern
    println!("\n📋 Test 4: Deception Detection\n");
    let result4 = reasoner.analyze("Everyone always uses this product. Literally everyone, every single time, without exception.");
    print_result(&result4);
    
    // Test 5: Simple Truth
    println!("\n📋 Test 5: Simple Truth\n");
    let result5 = reasoner.analyze("Water is composed of hydrogen and oxygen.");
    print_result(&result5);
    
    // Test 6: Exaggeration
    println!("\n📋 Test 6: Exaggeration/Sarcasm\n");
    let result6 = reasoner.analyze("Yeah right, that's absolutely perfect. Totally what I wanted.");
    print_result(&result6);
    
    // Test 7: Self-Contradictory
    println!("\n📋 Test 7: Self-Contradiction\n");
    let result7 = reasoner.analyze("This statement is false.");
    print_result(&result7);
    
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("\n✅ Demo complete! First principles reasoning operational.");
}

fn print_result(result: &spatial_vortex::agents::first_principles::FirstPrinciplesResult) {
    println!("Statement: \"{}\"", result.statement);
    println!();
    
    match &result.truth_assessment {
        TruthAssessment::True { certainty } => {
            println!("✅ Assessment: TRUE");
            println!("   Certainty: {:.0}%", certainty * 100.0);
        }
        TruthAssessment::False { certainty } => {
            println!("❌ Assessment: FALSE");
            println!("   Certainty: {:.0}%", certainty * 100.0);
        }
        TruthAssessment::PartiallyTrue { true_percentage } => {
            println!("⚠️  Assessment: PARTIALLY TRUE");
            println!("   Accuracy: {:.0}%", true_percentage * 100.0);
        }
        TruthAssessment::Uncertain { ambiguity_score } => {
            println!("❓ Assessment: UNCERTAIN");
            println!("   Ambiguity: {:.0}%", ambiguity_score * 100.0);
        }
        TruthAssessment::Sarcastic { intended_meaning, confidence } => {
            println!("😏 Assessment: SARCASTIC/IRONIC");
            println!("   Confidence: {:.0}%", confidence * 100.0);
            println!("   Intended: {}", intended_meaning);
        }
        TruthAssessment::Deceptive { deception_type, confidence } => {
            println!("🚨 Assessment: DECEPTIVE");
            println!("   Type: {:?}", deception_type);
            println!("   Confidence: {:.0}%", confidence * 100.0);
        }
        TruthAssessment::Opinion { perspective } => {
            println!("💭 Assessment: OPINION");
            println!("   Perspective: {}", perspective);
        }
    }
    
    println!();
    println!("ELP Signature:");
    println!("   Ethos (Character): {:.1}/9", result.elp_signature.ethos);
    println!("   Logos (Logic):     {:.1}/9", result.elp_signature.logos);
    println!("   Pathos (Emotion):  {:.1}/9", result.elp_signature.pathos);
    
    println!();
    println!("Overall Confidence: {:.0}%", result.confidence * 100.0);
    
    if !result.reasoning_steps.is_empty() {
        println!();
        println!("Reasoning Steps:");
        for (i, step) in result.reasoning_steps.iter().enumerate() {
            println!("   {}. {}", i + 1, step.description);
            println!("      Premise: {}", step.premise);
            println!("      Conclusion: {}", step.conclusion);
        }
    }
    
    if !result.axioms_applied.is_empty() {
        println!();
        println!("Axioms Applied:");
        for axiom in &result.axioms_applied {
            println!("   - {}", axiom);
        }
    }
    
    println!("\n───────────────────────────────────────────────────────────────");
}
