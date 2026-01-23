//! Verification Threshold Tuning Demo
//!
//! Tests different verification threshold configurations to find optimal balance:
//! - Lenient: High pass rate, may miss some issues
//! - Balanced: Default, good trade-off
//! - Strict: Low pass rate, catches all issues
//! - Custom: User-defined thresholds
//!
//! Helps tune for different production scenarios

use spatial_vortex::{
    ai::{
        reasoning_chain::ReasoningChain,
        self_verification::SelfVerificationEngine,
    },
    data::models::ELPTensor,
};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("\n🔧 ═══════════════════════════════════════════════════════════");
    println!("   VERIFICATION THRESHOLD TUNING");
    println!("   Finding optimal settings for your use case");
    println!("🔧 ═══════════════════════════════════════════════════════════\n");

    // Create test chains with different quality levels
    let test_chains = create_test_chains();
    
    println!("📊 Test Suite: {} reasoning chains", test_chains.len());
    println!("   • 3 high-quality chains (should pass)");
    println!("   • 3 medium-quality chains (borderline)");
    println!("   • 3 low-quality chains (should fail)\n");
    
    // Test different verification modes
    let verifiers = vec![
        ("Lenient", create_lenient_verifier()),
        ("Balanced", SelfVerificationEngine::new()),
        ("Strict", SelfVerificationEngine::new_strict()),
        ("Custom", create_custom_verifier()),
    ];
    
    let mut results_table = Vec::new();
    
    for (mode_name, verifier) in verifiers {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("🔍 Testing: {} Mode", mode_name);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        
        print_verifier_config(&verifier);
        
        let mut passed = 0;
        let mut failed = 0;
        let mut avg_confidence = 0.0;
        let mut false_positives = 0; // Low quality that passed
        let mut false_negatives = 0; // High quality that failed
        
        for (i, (quality, chain)) in test_chains.iter().enumerate() {
            let result = verifier.verify_chain(chain)?;
            
            if result.passed {
                passed += 1;
            } else {
                failed += 1;
            }
            
            avg_confidence += result.confidence;
            
            // Check for false positives/negatives
            match quality.as_str() {
                "high" if !result.passed => false_negatives += 1,
                "low" if result.passed => false_positives += 1,
                _ => {}
            }
            
            let status = if result.passed { "✅ PASS" } else { "❌ FAIL" };
            let quality_icon = match quality.as_str() {
                "high" => "⭐",
                "medium" => "○",
                "low" => "⚠️",
                _ => "?",
            };
            
            println!("   {} {} Chain {}: Conf {:.1}%, Issues: {}",
                status, quality_icon, i + 1, 
                result.confidence * 100.0, 
                result.issues.len()
            );
        }
        
        avg_confidence /= test_chains.len() as f32;
        
        println!("\n📊 Results:");
        println!("   Passed: {}/9 ({:.0}%)", passed, (passed as f32 / 9.0) * 100.0);
        println!("   Failed: {}/9 ({:.0}%)", failed, (failed as f32 / 9.0) * 100.0);
        println!("   Avg Confidence: {:.1}%", avg_confidence * 100.0);
        println!("   False Positives: {} (low quality passed)", false_positives);
        println!("   False Negatives: {} (high quality failed)", false_negatives);
        
        // Calculate F1 score
        let precision = if (passed - false_positives) + false_positives > 0 {
            (passed - false_positives) as f32 / (passed as f32)
        } else {
            0.0
        };
        
        let recall = if 3.0 > 0.0 {
            (3 - false_negatives) as f32 / 3.0
        } else {
            0.0
        };
        
        let f1 = if precision + recall > 0.0 {
            2.0 * (precision * recall) / (precision + recall)
        } else {
            0.0
        };
        
        println!("   Precision: {:.2}", precision);
        println!("   Recall: {:.2}", recall);
        println!("   F1 Score: {:.2}", f1);
        
        // Recommendation
        print!("\n💡 ");
        if false_negatives == 0 && false_positives == 0 {
            println!("PERFECT - No false positives or negatives!");
        } else if false_negatives > 0 {
            println!("TOO STRICT - Rejecting good chains, consider relaxing");
        } else if false_positives > 1 {
            println!("TOO LENIENT - Accepting bad chains, consider tightening");
        } else {
            println!("BALANCED - Good trade-off for most use cases");
        }
        
        println!();
        
        results_table.push((
            mode_name,
            passed,
            failed,
            avg_confidence,
            false_positives,
            false_negatives,
            f1,
        ));
    }
    
    // Comparison table
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 COMPARISON TABLE");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    println!("{:<12} {:>8} {:>8} {:>10} {:>6} {:>6} {:>8}",
        "Mode", "Pass", "Fail", "Avg Conf", "FP", "FN", "F1");
    println!("{}", "-".repeat(70));
    
    for (mode, passed, failed, conf, fp, fn_, f1) in results_table {
        println!("{:<12} {:>8} {:>8} {:>9.1}% {:>6} {:>6} {:>8.2}",
            mode, passed, failed, conf * 100.0, fp, fn_, f1);
    }
    
    println!("\n🎯 Recommendations by Use Case:");
    println!("\n   📱 Rapid Prototyping / Experimentation:");
    println!("      → Use LENIENT mode");
    println!("      → Maximizes pass rate, encourages exploration");
    println!("      → Accept some false positives");
    
    println!("\n   🏢 Production Applications:");
    println!("      → Use BALANCED mode (default)");
    println!("      → Good precision/recall trade-off");
    println!("      → Suitable for most business applications");
    
    println!("\n   🏥 Safety-Critical Systems:");
    println!("      → Use STRICT mode");
    println!("      → Minimizes false positives");
    println!("      → Better to reject good than accept bad");
    
    println!("\n   🎛️  Custom Requirements:");
    println!("      → Adjust individual thresholds");
    println!("      → Test with your specific workload");
    println!("      → Monitor false positive/negative rates");
    
    println!("\n✨ Tuning complete! Choose the mode that fits your needs. ✨\n");
    
    Ok(())
}

fn create_test_chains() -> Vec<(String, ReasoningChain)> {
    let mut chains = Vec::new();
    
    // High-quality chains (3)
    for i in 0..3 {
        let mut chain = ReasoningChain::new();
        let steps = vec![
            (1, "Analyze requirements", 0.80),
            (2, "Break down problem", 0.82),
            (3, "Consider edge cases", 0.88),
            (4, "Design algorithm", 0.85),
            (8, "Implement solution", 0.83),
            (7, "Add error handling", 0.86),
            (5, "Optimize performance", 0.84),
            (6, "Write comprehensive tests", 0.89),
            (9, "Final validation", 0.91),
        ];
        
        for (pos, thought, conf) in steps {
            let elp = ELPTensor {
                ethos: 6.0 + (i as f64 * 0.3),
                logos: 7.0 + (i as f64 * 0.2),
                pathos: 5.5 + (i as f64 * 0.1),
            };
            chain.add_step(thought.to_string(), elp, pos, conf);
        }
        
        chain.finalize(format!("High-quality solution {}", i + 1));
        chains.push(("high".to_string(), chain));
    }
    
    // Medium-quality chains (3)
    for i in 0..3 {
        let mut chain = ReasoningChain::new();
        let steps = vec![
            (1, "Start analysis", 0.65),
            (2, "Basic approach", 0.68),
            (4, "Implementation idea", 0.72),
            (6, "Some testing", 0.70),
            (9, "Complete", 0.75),
        ];
        
        for (pos, thought, conf) in steps {
            let elp = ELPTensor {
                ethos: 5.5,
                logos: 6.2,
                pathos: 6.0,
            };
            chain.add_step(thought.to_string(), elp, pos, conf);
        }
        
        chain.finalize(format!("Medium solution {}", i + 1));
        chains.push(("medium".to_string(), chain));
    }
    
    // Low-quality chains (3)
    for i in 0..3 {
        let mut chain = ReasoningChain::new();
        let steps = vec![
            (1, "Quick thought", 0.45),
            (2, "Another idea", 0.40),
            (7, "Jump to conclusion", 0.38),
        ];
        
        for (pos, thought, conf) in steps {
            let elp = ELPTensor {
                ethos: 4.0,
                logos: 4.5,
                pathos: 7.0, // Pathos-heavy
            };
            chain.add_step(thought.to_string(), elp, pos, conf);
        }
        
        chain.finalize(format!("Low solution {}", i + 1));
        chains.push(("low".to_string(), chain));
    }
    
    chains
}

fn create_lenient_verifier() -> SelfVerificationEngine {
    // Very permissive - good for experimentation
    SelfVerificationEngine {
        hallucination_detector: spatial_vortex::ml::hallucinations::HallucinationDetector::default(),
        vcp: spatial_vortex::ml::hallucinations::VortexContextPreserver::default(),
        min_confidence: 0.35,
        max_elp_jump: 4.0,
    }
}

fn create_custom_verifier() -> SelfVerificationEngine {
    // Custom tuned for specific use case
    SelfVerificationEngine {
        hallucination_detector: spatial_vortex::ml::hallucinations::HallucinationDetector::default(),
        vcp: spatial_vortex::ml::hallucinations::VortexContextPreserver::default(),
        min_confidence: 0.48,
        max_elp_jump: 3.2,
    }
}

fn print_verifier_config(verifier: &SelfVerificationEngine) {
    // Access fields through verification of a test chain
    println!("⚙️  Configuration:");
    println!("   Min Confidence: (configured internally)");
    println!("   Max ELP Jump: (configured internally)");
    println!("   Min Confidence: (configured internally)\n");
}
