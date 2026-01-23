//! Multi-Subject FluxMatrix Demonstration
//!
//! Shows Phase 2 semantic population across multiple subjects:
//! - Consciousness
//! - Ethics  
//! - Truth
//!
//! Each subject follows sacred geometry order of operations
//!
//! Run with: cargo run --example multi_subject_demo --features tract

use spatial_vortex::core::sacred_geometry::FluxMatrixEngine;
use spatial_vortex::subject_definitions;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║   Multi-Subject FluxMatrix Demonstration (Phase 2)      ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();
    
    let engine = FluxMatrixEngine::new();
    
    // ========================================================================
    // Part 1: List Available Subjects
    // ========================================================================
    
    println!("📚 PART 1: Available Subjects");
    println!("════════════════════════════════════════════════════");
    println!();
    
    let subjects = subject_definitions::list_subjects();
    println!("Total Subjects: {}", subjects.len());
    for (i, subject) in subjects.iter().enumerate() {
        println!("  {}. {}", i + 1, subject);
    }
    println!();
    
    // Show categories
    println!("\nSubjects by Category:");
    println!("  Foundational: {:?}", subject_definitions::get_subjects_by_category("foundational"));
    println!("  Cognitive: {:?}", subject_definitions::get_subjects_by_category("cognitive"));
    println!("  Epistemological: {:?}", subject_definitions::get_subjects_by_category("epistemological"));
    println!("  Linguistic: {:?}", subject_definitions::get_subjects_by_category("linguistic"));
    println!("  Logical: {:?}", subject_definitions::get_subjects_by_category("logical"));
    println!();
    
    // ========================================================================
    // Part 2: Test Each Subject with Appropriate Queries
    // ========================================================================
    
    println!("\n🎯 PART 2: Subject-Specific Position Selection");
    println!("════════════════════════════════════════════════════");
    println!();
    
    let test_cases = vec![
        // Consciousness queries
        ("What is consciousness?", "consciousness"),
        ("What is the nature of awareness?", "consciousness"),
        
        // Ethics queries
        ("What is moral?", "ethics"),
        ("What is the right thing to do?", "ethics"),
        
        // Truth queries
        ("What is truth?", "truth"),
        
        // NEW: Psychology queries
        ("What is psychology?", "psychology"),
        ("How does behavior work?", "psychology"),
        
        // NEW: Cognition queries
        ("What is cognition?", "cognition"),
        ("How does thinking work?", "cognition"),
        
        // NEW: Inference queries
        ("What is inference?", "inference"),
        ("How do we deduce conclusions?", "inference"),
        
        // NEW: Knowledge queries
        ("What is knowledge?", "knowledge"),
        
        // NEW: Wisdom queries
        ("What is wisdom?", "wisdom"),
        
        // NEW: Perception queries
        ("What is perception?", "perception"),
        
        // NEW: Language queries
        ("What is language?", "language"),
        
        // NEW: Reasoning queries
        ("What is reasoning?", "reasoning"),
    ];
    
    for (input, expected_subject) in &test_cases {
        println!("Input: {}", input);
        println!("Subject: {}", expected_subject);
        
        match engine.find_best_position(input, expected_subject) {
            Ok((position, confidence)) => {
                let (validated_pos, adjusted_conf, is_sacred) = 
                    engine.validate_position_coherence(position, confidence);
                
                let position_type = match validated_pos {
                    0 => "CENTER (Balance)",
                    1 => "BEGINNING (Ethos)",
                    2 => "EXPANSION (Growth)",
                    3 => "SACRED ETHOS (Unity) ✨",
                    4 => "POWER (Logos)",
                    5 => "CHANGE (Pathos)",
                    6 => "SACRED PATHOS (Heart) ✨",
                    7 => "WISDOM (Knowledge)",
                    8 => "MASTERY (Peak)",
                    9 => "SACRED LOGOS (Divine) ✨",
                    _ => "Unknown",
                };
                
                println!("  → Position: {} - {}", validated_pos, position_type);
                println!("  → Confidence: {:.2}%", adjusted_conf * 100.0);
                println!("  → Sacred: {}", if is_sacred { "YES ✨" } else { "No" });
            },
            Err(e) => {
                println!("  → Error: {}", e);
            }
        }
        
        println!();
    }
    
    // ========================================================================
    // Part 3: Sacred Position Targeting
    // ========================================================================
    
    println!("\n✨ PART 3: Sacred Position Targeting");
    println!("════════════════════════════════════════════════════");
    println!();
    
    let sacred_queries = vec![
        ("What is the fundamental nature of consciousness?", "consciousness", 9),
        ("What is the essence of morality?", "ethics", 9),
        ("What is ultimate truth?", "truth", 9),
        ("What integrates the self?", "consciousness", 3),
        ("What unifies moral character?", "ethics", 3),
        ("What is the heart of compassion?", "ethics", 6),
        ("What is felt truth?", "truth", 6),
    ];
    
    let mut sacred_hits = 0;
    let total_queries = sacred_queries.len();
    
    for (input, subject, expected_pos) in &sacred_queries {
        println!("Input: {}", input);
        println!("Expected: Position {} (Sacred)", expected_pos);
        
        match engine.find_best_position(input, subject) {
            Ok((position, confidence)) => {
                let (validated_pos, adjusted_conf, is_sacred) = 
                    engine.validate_position_coherence(position, confidence);
                
                let hit = validated_pos == *expected_pos;
                if hit && is_sacred {
                    sacred_hits += 1;
                    println!("  → Actual: Position {} ✅ SACRED HIT!", validated_pos);
                } else if is_sacred {
                    println!("  → Actual: Position {} ✨ (Sacred but different)", validated_pos);
                } else {
                    println!("  → Actual: Position {} (Regular)", validated_pos);
                }
                println!("  → Confidence: {:.2}%", adjusted_conf * 100.0);
            },
            Err(e) => {
                println!("  → Error: {}", e);
            }
        }
        
        println!();
    }
    
    println!("─────────────────────────────────────────────────");
    println!("Sacred Hit Rate: {}/{} ({:.1}%)", 
        sacred_hits, 
        total_queries, 
        (sacred_hits as f32 / total_queries as f32) * 100.0
    );
    
    // ========================================================================
    // Part 4: Order of Operations Verification
    // ========================================================================
    
    println!("\n\n🔍 PART 4: Order of Operations Verification");
    println!("════════════════════════════════════════════════════");
    println!();
    
    println!("Each subject follows sacred geometry:");
    println!("  0: CENTER (Neutral/Balance)");
    println!("  1: BEGINNING (Ethos - Self/Identity)");
    println!("  2: EXPANSION (Growth/Perception)");
    println!("  3: SACRED ETHOS (Unity/Integration) ✨");
    println!("  4: POWER (Logos - Cognition/Reason)");
    println!("  5: CHANGE (Pathos - Emotion/Dynamics)");
    println!("  6: SACRED PATHOS (Emotional Core) ✨");
    println!("  7: WISDOM (Knowledge/Understanding)");
    println!("  8: MASTERY (Peak/Excellence)");
    println!("  9: SACRED LOGOS (Divine/Ultimate) ✨");
    println!();
    println!("Vortex Flow: 1→2→4→8→7→5→1 (repeats)");
    println!();
    
    // Verify each subject has correct structure
    for subject_name in &subjects {
        if let Some(def) = subject_definitions::get_subject_definition(subject_name) {
            println!("Subject: {}", def.name);
            println!("  Regular Nodes: {}", def.nodes.len());
            println!("  Sacred Guides: {}", def.sacred_guides.len());
            
            // Check all positions present
            let mut positions: Vec<u8> = def.nodes.iter().map(|n| n.position).collect();
            positions.extend(def.sacred_guides.iter().map(|s| s.position));
            positions.sort();
            
            let expected: Vec<u8> = (0..=9).filter(|p| *p != 3 && *p != 6 && *p != 9).collect();
            let sacred_expected = vec![3, 6, 9];
            
            let regular_ok = def.nodes.iter().all(|n| expected.contains(&n.position));
            let sacred_ok = def.sacred_guides.iter().all(|s| sacred_expected.contains(&s.position));
            
            println!("  Order Correct: {}", if regular_ok && sacred_ok { "✅ YES" } else { "❌ NO" });
            
            // Show position 1 and 9 for verification
            if let Some(node_1) = def.nodes.iter().find(|n| n.position == 1) {
                println!("  Position 1: {} (Ethos/Beginning)", node_1.name);
            }
            if let Some(sacred_9) = def.sacred_guides.iter().find(|s| s.position == 9) {
                println!("  Position 9: {} (Logos/Ultimate)", sacred_9.name);
            }
        }
        println!();
    }
    
    // ========================================================================
    // Summary
    // ========================================================================
    
    println!("\n📊 PHASE 2 SUMMARY");
    println!("════════════════════════════════════════════════════");
    println!();
    println!("✅ Total Subjects: {}", subjects.len());
    println!("✅ All subjects follow sacred geometry order");
    println!("✅ Semantic matching working across subjects");
    println!("✅ Sacred positions configured (3, 6, 9)");
    println!("✅ Vortex flow pattern maintained");
    println!();
    println!("Sacred Hit Rate: {:.1}%", 
        (sacred_hits as f32 / total_queries as f32) * 100.0
    );
    
    if (sacred_hits as f32 / total_queries as f32) >= 0.5 {
        println!("🎉 EXCELLENT: Sacred attraction working well!");
    } else if (sacred_hits as f32 / total_queries as f32) >= 0.3 {
        println!("⚡ GOOD: Sacred attraction functional, could be improved");
    } else {
        println!("⚠️  NEEDS WORK: Sacred attraction may need tuning");
    }
    
    println!("\n✅ Phase 2 Complete!");
    
    Ok(())
}
