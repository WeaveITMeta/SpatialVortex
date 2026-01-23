//! Response Quality Demo
//!
//! Demonstrates the improved chat response system that addresses:
//! - Context loss (returning code for greetings)
//! - Over-engineered responses (frameworks for simple questions)
//! - Information overload (massive walls of text)
//! - Formatting abuse (excessive markdown)

use spatial_vortex::core::sacred_geometry::{
    FluxMatrixEngine, MatrixGuidedInference, ResponseMode,
};

fn main() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║        Response Quality Improvement Demo                ║");
    println!("║                                                          ║");
    println!("║  Adaptive Modes · Clean Formatting · Context Awareness  ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");
    
    // Create inference system
    let flux_engine = FluxMatrixEngine::new();
    let inference = MatrixGuidedInference::new(flux_engine);
    
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // Demo 1: Greeting (should be CONCISE, not code)
    demo_greeting(&inference);
    
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // Demo 2: Simplification request (should be SHORT)
    demo_simplification(&inference);
    
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // Demo 3: Trade-offs question (should be BALANCED, not essay)
    demo_balanced_response(&inference);
    
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // Demo 4: Complex query (can be DETAILED)
    demo_detailed_response(&inference);
    
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║                   Demo Complete! ✅                      ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");
    
    println!("🎯 Key Improvements Demonstrated:");
    println!("   ✓ Greetings get friendly replies (not code!)");
    println!("   ✓ Simple requests get concise answers");
    println!("   ✓ Balanced responses for normal questions");
    println!("   ✓ Detailed only when complexity warrants it");
    println!("   ✓ Clean formatting throughout");
    println!("   ✓ No meta-commentary or 'As Vortex, I...' phrases\n");
    
    println!("   Natural conversation beats overwhelming documentation! 🌀\n");
}

fn demo_greeting(inference: &MatrixGuidedInference) {
    println!("📊 Demo 1: Greeting Detection");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    let query = "How do you do?";
    println!("User Query: \"{}\"\n", query);
    
    match inference.build_adaptive_prompt(query, "general") {
        Ok((prompt, mode)) => {
            println!("✅ Detected Mode: {:?}", mode);
            println!("\n📝 Generated System Prompt:\n");
            println!("{}", prompt);
            
            println!("\n💡 What This Prevents:");
            println!("   ❌ BEFORE: Returns Python NLTK sentiment analysis code");
            println!("   ✅ AFTER: Returns friendly greeting (1-2 sentences)");
        }
        Err(e) => println!("❌ Error: {}", e),
    }
}

fn demo_simplification(inference: &MatrixGuidedInference) {
    println!("📊 Demo 2: Simplification Request");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    let query = "Can you explain this in simpler terms?";
    println!("User Query: \"{}\"\n", query);
    
    match inference.build_adaptive_prompt(query, "cognition") {
        Ok((prompt, mode)) => {
            println!("✅ Detected Mode: {:?}", mode);
            println!("\n📝 Key Instructions from Prompt:\n");
            
            // Extract key parts
            if prompt.contains("CONCISE") {
                println!("   • Response mode: CONCISE");
                println!("   • Maximum: 2-3 sentences");
                println!("   • Direct answer only");
                println!("   • No lengthy explanations");
            }
            
            println!("\n💡 What This Prevents:");
            println!("   ❌ BEFORE: 'Simplifying Complex Concepts' framework with");
            println!("             multi-step methodology, pipes, task lists");
            println!("   ✅ AFTER: 2-3 sentence direct answer");
        }
        Err(e) => println!("❌ Error: {}", e),
    }
}

fn demo_balanced_response(inference: &MatrixGuidedInference) {
    println!("📊 Demo 3: Trade-Offs Question (Balanced)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    let query = "What are the trade-offs?";
    println!("User Query: \"{}\"\n", query);
    
    match inference.build_adaptive_prompt(query, "cognition") {
        Ok((prompt, mode)) => {
            println!("✅ Detected Mode: {:?}", mode);
            println!("\n📝 Key Instructions from Prompt:\n");
            
            if prompt.contains("BALANCED") {
                println!("   • Response mode: BALANCED");
                println!("   • Length: 2-4 short paragraphs maximum");
                println!("   • Include 1 example if helpful");
                println!("   • Bullet points ONLY for 3+ distinct items");
                println!("   • No === headers or excessive formatting");
            }
            
            println!("\n💡 What This Prevents:");
            println!("   ❌ BEFORE: 600-word essay with:");
            println!("             === headers, ### subheaders, tables,");
            println!("             multiple examples, overwhelming detail");
            println!("   ✅ AFTER: 150-200 word clear explanation");
        }
        Err(e) => println!("❌ Error: {}", e),
    }
}

fn demo_detailed_response(inference: &MatrixGuidedInference) {
    println!("📊 Demo 4: Complex Technical Query (Detailed OK)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    let query = "Explain the mathematical foundations of vortex mathematics \
                 including the doubling sequence, sacred positions 3-6-9, \
                 and how digital root reduction creates stable attractors \
                 in the flux pattern.";
    
    println!("User Query: \"{}...\" (complex technical)\n", &query[..80]);
    
    match inference.build_adaptive_prompt(query, "mathematics") {
        Ok((prompt, mode)) => {
            println!("✅ Detected Mode: {:?}", mode);
            println!("\n📝 Key Instructions from Prompt:\n");
            
            if prompt.contains("DETAILED") {
                println!("   • Response mode: DETAILED");
                println!("   • Provide comprehensive explanation");
                println!("   • Include 2-3 concrete examples");
                println!("   • Break into clear sections (max 3)");
                println!("   • Still conversational, not academic");
            }
            
            println!("\n💡 When Detailed Mode is Appropriate:");
            println!("   ✓ Complex technical question with multiple parts");
            println!("   ✓ User explicitly asks for comprehensive answer");
            println!("   ✓ Topic has high complexity (many associations)");
            println!("   ✓ Still maintains conversational tone");
        }
        Err(e) => println!("❌ Error: {}", e),
    }
}
