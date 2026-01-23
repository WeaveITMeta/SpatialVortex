//! Continuous Learning Demo
//!
//! Demonstrates Phase 4: Continuous Learning
//! - User feedback collection
//! - Dynamic semantic association updates
//! - Learning metrics tracking

use spatial_vortex::core::sacred_geometry::{
    FluxMatrixEngine, ContinuousLearning, UserFeedback,
};

fn main() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║    Continuous Learning Demonstration (Phase 4)          ║");
    println!("║                                                          ║");
    println!("║  User Feedback · Dynamic Updates · Learning Metrics     ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");
    
    // Create flux matrix engine
    let flux_engine = FluxMatrixEngine::new();
    
    // Create continuous learning system
    let mut learning_system = ContinuousLearning::new(flux_engine.clone());
    learning_system.set_learning_rate(0.15);  // Moderate learning rate
    
    println!("📚 Continuous Learning System Initialized");
    println!("   Learning Rate: 0.15");
    println!("   Min Confidence Threshold: 0.7\n");
    
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // Demo 1: Submit user feedback
    demo_1_submit_feedback(&mut learning_system);
    
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // Demo 2: Generate learning adjustments
    demo_2_generate_adjustments(&mut learning_system);
    
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // Demo 3: Apply adjustments and show metrics
    demo_3_apply_and_metrics(learning_system, flux_engine);
    
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║                   Demo Complete! ✅                      ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");
    
    println!("🎯 Key Achievements Demonstrated:");
    println!("   ✓ User feedback collection with ratings");
    println!("   ✓ Relevant/irrelevant term tracking");
    println!("   ✓ Dynamic learning adjustment generation");
    println!("   ✓ Semantic association strengthening/weakening");
    println!("   ✓ Learning metrics and success rate tracking");
    println!("   ✓ Continuous improvement over time\n");
    
    println!("   The system now LEARNS from user interactions! 🧠\n");
}

fn demo_1_submit_feedback(learning_system: &mut ContinuousLearning) {
    println!("📊 Demo 1: Submit User Feedback");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // Simulate positive feedback
    println!("✅ Submitting POSITIVE Feedback:");
    let mut feedback1 = UserFeedback::new(
        "How does reasoning work?".to_string(),
        "cognition".to_string(),
        4,  // Position 4 - Logical Cognition
        "Reasoning is a cognitive process involving logic, analysis, and deduction...".to_string(),
        5,  // 5-star rating
        true,  // Helpful
    );
    feedback1.add_relevant("reasoning".to_string());
    feedback1.add_relevant("logic".to_string());
    feedback1.add_relevant("analysis".to_string());
    feedback1.add_irrelevant("confusion".to_string());
    
    println!("   Query: \"{}\"", feedback1.query);
    println!("   Subject: {}", feedback1.subject);
    println!("   Position: {}", feedback1.position);
    println!("   Rating: ⭐⭐⭐⭐⭐ ({})", feedback1.rating);
    println!("   Helpful: {}", if feedback1.helpful { "✓ Yes" } else { "✗ No" });
    println!("   Relevant Terms: {:?}", feedback1.relevant_terms);
    println!("   Irrelevant Terms: {:?}", feedback1.irrelevant_terms);
    
    learning_system.submit_feedback(feedback1).unwrap();
    
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // Simulate another positive feedback
    println!("✅ Submitting POSITIVE Feedback #2:");
    let mut feedback2 = UserFeedback::new(
        "Explain critical thinking".to_string(),
        "cognition".to_string(),
        4,
        "Critical thinking involves analysis, evaluation, and logical reasoning...".to_string(),
        4,
        true,
    );
    feedback2.add_relevant("critical-thinking".to_string());
    feedback2.add_relevant("analysis".to_string());
    feedback2.add_relevant("evaluation".to_string());
    
    println!("   Query: \"{}\"", feedback2.query);
    println!("   Rating: ⭐⭐⭐⭐ ({})", feedback2.rating);
    println!("   Relevant Terms: {:?}", feedback2.relevant_terms);
    
    learning_system.submit_feedback(feedback2).unwrap();
    
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // Simulate negative feedback
    println!("⚠️  Submitting NEGATIVE Feedback:");
    let mut feedback3 = UserFeedback::new(
        "What is thinking?".to_string(),
        "cognition".to_string(),
        0,
        "Thinking is a mental process...".to_string(),
        2,
        false,
    );
    feedback3.add_irrelevant("mental-activity".to_string());
    feedback3.add_irrelevant("thinking-process".to_string());
    
    println!("   Query: \"{}\"", feedback3.query);
    println!("   Rating: ⭐⭐ ({})", feedback3.rating);
    println!("   Helpful: {}", if feedback3.helpful { "✓ Yes" } else { "✗ No" });
    println!("   Irrelevant Terms: {:?}", feedback3.irrelevant_terms);
    
    learning_system.submit_feedback(feedback3).unwrap();
    
    println!("\n📊 Total Feedback Submitted: 3");
    println!("   ✅ Positive: 2");
    println!("   ⚠️  Negative: 1");
}

fn demo_2_generate_adjustments(learning_system: &mut ContinuousLearning) {
    println!("📊 Demo 2: Generate Learning Adjustments");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    println!("🔧 Analyzing feedback for subject: cognition\n");
    
    match learning_system.generate_adjustments("cognition") {
        Ok(adjustments) => {
            println!("✅ Generated {} learning adjustments:\n", adjustments.len());
            
            for (i, adjustment) in adjustments.iter().enumerate() {
                println!("   Adjustment #{}", i + 1);
                println!("   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("   Position: {}", adjustment.position);
                println!("   Reason: {}", adjustment.reason);
                println!("   Confidence: {:.2}", adjustment.confidence);
                
                if !adjustment.strengthen.is_empty() {
                    println!("\n   📈 Strengthen (Boost Confidence):");
                    for (word, boost) in &adjustment.strengthen {
                        println!("      • {} (+{:.3})", word, boost);
                    }
                }
                
                if !adjustment.weaken.is_empty() {
                    println!("\n   📉 Weaken (Reduce Confidence):");
                    for (word, penalty) in &adjustment.weaken {
                        println!("      • {} (-{:.3})", word, penalty);
                    }
                }
                
                println!();
            }
            
            if adjustments.is_empty() {
                println!("   ℹ️  No high-confidence adjustments generated yet.");
                println!("   Need more feedback to reach confidence threshold (0.7)\n");
            }
        }
        Err(e) => println!("❌ Error: {}", e),
    }
}

fn demo_3_apply_and_metrics(mut learning_system: ContinuousLearning, flux_engine: FluxMatrixEngine) {
    println!("📊 Demo 3: Apply Adjustments & View Metrics");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // Get current metrics
    println!("📈 Learning Metrics - Before Adjustments:\n");
    if let Some(metrics) = learning_system.get_metrics("cognition") {
        print_metrics(metrics);
    } else {
        println!("   No metrics available yet\n");
    }
    
    // Create matrix and apply adjustments
    println!("🔧 Creating FluxMatrix and applying adjustments...\n");
    
    match flux_engine.create_matrix("cognition".to_string()) {
        Ok(mut matrix) => {
            match learning_system.generate_adjustments("cognition") {
                Ok(adjustments) => {
                    let adj_count = adjustments.len();
                    match learning_system.apply_adjustments(&mut matrix, adjustments) {
                        Ok(applied) => {
                            println!("✅ Applied {} adjustments from {} adjustment sets", applied, adj_count);
                            println!("   Matrix updated successfully!\n");
                        }
                        Err(e) => println!("❌ Error applying: {}\n", e),
                    }
                }
                Err(e) => println!("❌ Error generating: {}\n", e),
            }
        }
        Err(e) => println!("❌ Error creating matrix: {}\n", e),
    }
    
    // Show updated metrics
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    println!("📈 Learning Metrics - After Adjustments:\n");
    if let Some(metrics) = learning_system.get_metrics("cognition") {
        print_metrics(metrics);
    }
    
    // Show global metrics
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    println!("🌍 Global Learning Metrics (All Subjects):\n");
    let global = learning_system.get_global_metrics();
    print_metrics(&global);
    
    // Show feedback history
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    println!("📜 Recent Feedback History (Last 5):\n");
    let history = learning_system.get_feedback_history("cognition", 5);
    for (i, feedback) in history.iter().enumerate() {
        println!("   {}. {} - Rating: {}/5 - Helpful: {}",
            i + 1,
            feedback.query,
            feedback.rating,
            if feedback.helpful { "✓" } else { "✗" }
        );
    }
}

fn print_metrics(metrics: &spatial_vortex::core::sacred_geometry::LearningMetrics) {
    println!("   Total Feedback: {}", metrics.total_feedback);
    println!("   Positive: {} | Negative: {}", metrics.positive_feedback, metrics.negative_feedback);
    println!("   Average Rating: {:.2}/5.0 ⭐", metrics.average_rating);
    println!("   Success Rate: {:.1}%", metrics.success_rate * 100.0);
    println!("   Total Adjustments: {}", metrics.total_adjustments);
    println!("   Associations Strengthened: {}", metrics.strengthened_count);
    println!("   Associations Weakened: {}", metrics.weakened_count);
    println!("   Last Update: {}\n", metrics.last_update.format("%Y-%m-%d %H:%M:%S"));
}
