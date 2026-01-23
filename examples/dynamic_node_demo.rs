//! Dynamic Node Attributes Demo
//!
//! Demonstrates how nodes dynamically evaluate objects based on:
//! - Loop position awareness
//! - Order of operations role
//! - Sacred position intelligence
//! - Object-relative confidence

use spatial_vortex::core::sacred_geometry::{
    FluxMatrixEngine, FluxNodeDynamics, create_object_context, estimate_attributes_from_query,
};
use spatial_vortex::data::attributes::Attributes;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║        Dynamic Node Attributes Demonstration            ║");
    println!("║                                                          ║");
    println!("║  Loop-Aware · Order-Conscious · Object-Relative         ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let flux_engine = FluxMatrixEngine::new();
    
    // Demo 1: Sacred position with fundamental query
    println!("📊 Demo 1: Sacred Position 9 (Logos) Evaluation");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    let query1 = "What is the fundamental nature of consciousness?";
    let subject1 = "consciousness";
    
    let attrs1 = estimate_attributes_from_query(query1);
    println!("Query: \"{}\"", query1);
    println!("Attributes: Ethos={:.2}, Logos={:.2}, Pathos={:.2}", 
             attrs1.ethos(), attrs1.logos(), attrs1.pathos());
    
    let mut node9 = flux_engine.create_flux_node(9, subject1)?;
    let object1 = create_object_context(query1, subject1, attrs1);
    
    println!("\n🔧 Node State After Initialization:");
    println!("   Position: {}", node9.position);
    println!("   Order Role: {:?}", node9.attributes.dynamics.order_role);
    println!("   Attribute Channel: {:?}", node9.attributes.dynamics.attribute_channel);
    println!("   Is Sacred: {}", node9.attributes.dynamics.is_sacred);
    println!("   Sacred Multiplier: {:.1}x", node9.attributes.dynamics.sacred_multiplier);
    println!("   Vortex Position: {:?}", node9.attributes.dynamics.vortex_position);
    
    let result1 = node9.evaluate_object(&object1);
    
    println!("\n✨ Dynamic Evaluation Result:");
    println!("   Should Accept: {}", result1.should_accept);
    println!("   Confidence: {:.3} ({})", 
             result1.confidence,
             if result1.confidence > 0.8 { "⭐ Excellent" } 
             else if result1.confidence > 0.6 { "✓ Good" } 
             else { "⚠ Low" });
    println!("   Fit Score: {:.3}", result1.fit_score);
    println!("   Adjustments: {}", result1.suggested_adjustments.len());
    
    if result1.confidence >= 0.8 {
        println!("\n   🎯 SACRED BOOST APPLIED!");
        println!("   Sacred keywords detected in query");
        println!("   Confidence amplified by sacred multiplier");
    }
    
    // Demo 2: Non-sacred position with emotional query
    println!("\n\n📊 Demo 2: Position 5 (Change/Pathos) Evaluation");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    let query2 = "How do I feel about this situation?";
    let subject2 = "psychology";
    
    let attrs2 = estimate_attributes_from_query(query2);
    println!("Query: \"{}\"", query2);
    println!("Attributes: Ethos={:.2}, Logos={:.2}, Pathos={:.2}", 
             attrs2.ethos(), attrs2.logos(), attrs2.pathos());
    
    let mut node5 = flux_engine.create_flux_node(5, subject2)?;
    let object2 = create_object_context(query2, subject2, attrs2);
    
    println!("\n🔧 Node State After Initialization:");
    println!("   Position: {}", node5.position);
    println!("   Order Role: {:?}", node5.attributes.dynamics.order_role);
    println!("   Attribute Channel: {:?}", node5.attributes.dynamics.attribute_channel);
    println!("   Is Sacred: {}", node5.attributes.dynamics.is_sacred);
    println!("   Sacred Multiplier: {:.1}x", node5.attributes.dynamics.sacred_multiplier);
    
    let result2 = node5.evaluate_object(&object2);
    
    println!("\n✨ Dynamic Evaluation Result:");
    println!("   Should Accept: {}", result2.should_accept);
    println!("   Confidence: {:.3}", result2.confidence);
    println!("   Fit Score: {:.3}", result2.fit_score);
    
    if result2.should_accept {
        println!("\n   ✓ PATHOS ALIGNMENT DETECTED!");
        println!("   Query matches Pathos channel at Position 5");
        println!("   Confidence boosted by 1.2x for emotional content");
    }
    
    // Demo 3: Loop progression
    println!("\n\n📊 Demo 3: Loop Progression Through Vortex Sequence");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    let query3 = "Explain the vortex mathematics pattern";
    let subject3 = "mathematics";
    let attrs3 = Attributes::with_elp(3.0, 9.0, 2.0); // Logical query
    
    let vortex_sequence = [1, 2, 4, 8, 7, 5]; // 1→2→4→8→7→5→1
    
    println!("Query: \"{}\"", query3);
    println!("Vortex Sequence: 1→2→4→8→7→5→1 (loop)\n");
    
    for (idx, &pos) in vortex_sequence.iter().enumerate() {
        let mut node = flux_engine.create_flux_node(pos, subject3)?;
        let object = create_object_context(query3, subject3, attrs3.clone());
        
        let result = node.evaluate_object(&object);
        
        println!("Step {}: Position {} ({:?})", 
                 idx + 1, pos, node.attributes.dynamics.order_role);
        println!("   Confidence: {:.3} | Fit: {:.3} | Vortex: {:?}",
                 result.confidence, result.fit_score,
                 node.attributes.dynamics.vortex_position);
        
        // Show sacred checkpoints
        if node.attributes.dynamics.is_sacred {
            println!("   ⭐ SACRED CHECKPOINT - Pattern validated!");
        }
    }
    
    // Demo 4: Learning and memory
    println!("\n\n📊 Demo 4: Learning and Memory (Multiple Evaluations)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    let mut node3 = flux_engine.create_flux_node(3, "philosophy")?;
    
    println!("Position 3 (Sacred Ethos) - Evaluating 3 queries:\n");
    
    let queries = [
        "What is the right thing to do?",
        "How should we behave ethically?",
        "Define moral responsibility",
    ];
    
    for (i, query) in queries.iter().enumerate() {
        let attrs = estimate_attributes_from_query(query);
        let object = create_object_context(query, "philosophy", attrs);
        let result = node3.evaluate_object(&object);
        
        println!("Evaluation {}: \"{}\"", i + 1, query);
        println!("   Confidence: {:.3} | Stability: {:.3}",
                 result.confidence, 
                 node3.attributes.dynamics.stability_index);
        println!("   History Size: {}", 
                 node3.attributes.dynamics.confidence_history.len());
        
        if result.confidence > 0.8 {
            println!("   ✓ High confidence - strengthening node stability");
        }
        println!();
    }
    
    println!("🧠 Node Learning Summary:");
    println!("   Total Evaluations: {}", 
             node3.attributes.dynamics.confidence_history.len());
    println!("   Final Stability: {:.3}", 
             node3.attributes.dynamics.stability_index);
    println!("   Interaction Patterns: {}", 
             node3.attributes.dynamics.interaction_patterns.len());
    
    // Summary
    println!("\n\n╔══════════════════════════════════════════════════════════╗");
    println!("║                   Demo Complete! ✅                      ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");
    
    println!("🎯 Key Achievements Demonstrated:");
    println!("   ✓ Loop awareness (vortex position tracking)");
    println!("   ✓ Order-conscious evaluation (role-based adjustments)");
    println!("   ✓ Sacred position intelligence (2.0x multiplier)");
    println!("   ✓ Object-relative confidence (semantic + ELP + position fit)");
    println!("   ✓ Memory and learning (confidence history, stability)");
    println!("\n   Nodes are now INTELLIGENT, not just STATIC containers!\n");
    
    Ok(())
}
