//! Formal Verification Demo
//!
//! Demonstrates the formal logic engine for SpatialVortex,
//! including theorem proving and verification of sacred geometry.
//!
//! Run with:
//! ```bash
//! cargo run --example formal_verification_demo --features formal-verification
//! ```

use spatial_vortex::core::formal_logic::{FormalLogicEngine, Axiom};

fn main() -> Result<(), String> {
    println!("🔷 Formal Logic Engine for SpatialVortex 🔷");
    println!("{}", "=".repeat(60));
    println!();
    
    // Create formal logic engine
    let mut engine = FormalLogicEngine::new()?;
    
    // Display all axioms
    demo_axioms(&engine);
    
    // Prove theorems
    demo_vortex_cycling(&mut engine)?;
    demo_sacred_exclusion(&mut engine)?;
    
    // Verify transformations
    demo_verification(&mut engine)?;
    
    // Check consistency
    demo_consistency(&mut engine)?;
    
    println!();
    println!("{}", "=".repeat(60));
    println!("✅ All formal verifications complete!");
    println!();
    
    Ok(())
}

fn demo_axioms(engine: &FormalLogicEngine) {
    println!("📜 AXIOMS OF SACRED GEOMETRY");
    println!("{}", "-".repeat(60));
    println!();
    
    for (i, axiom) in engine.axioms().iter().enumerate() {
        println!("{}. {}", i + 1, axiom.name());
        println!("   {}", axiom.description());
        println!();
    }
}

fn demo_vortex_cycling(engine: &mut FormalLogicEngine) -> Result<(), String> {
    println!("🔄 THEOREM: VORTEX CYCLING");
    println!("{}", "-".repeat(60));
    println!();
    
    let theorem = engine.prove_vortex_cycling()?;
    
    println!("Theorem: {}", theorem.name);
    println!("Statement: {}", theorem.statement);
    println!();
    
    println!("Proof:");
    for (i, step) in theorem.proof_steps.iter().enumerate() {
        println!("  {}", step);
    }
    
    println!();
    if theorem.proven {
        println!("✅ PROVEN: Theorem is mathematically correct");
    } else {
        println!("❌ NOT PROVEN: Verification failed");
    }
    
    println!();
    Ok(())
}

fn demo_sacred_exclusion(engine: &mut FormalLogicEngine) -> Result<(), String> {
    println!("🔺 THEOREM: SACRED EXCLUSION");
    println!("{}", "-".repeat(60));
    println!();
    
    let theorem = engine.prove_sacred_exclusion()?;
    
    println!("Theorem: {}", theorem.name);
    println!("Statement: {}", theorem.statement);
    println!();
    
    println!("Proof:");
    for step in &theorem.proof_steps {
        println!("  {}", step);
    }
    
    println!();
    if theorem.proven {
        println!("✅ PROVEN: 3, 6, 9 never appear in vortex flow");
    } else {
        println!("❌ NOT PROVEN: Verification failed");
    }
    
    println!();
    Ok(())
}

fn demo_verification(engine: &mut FormalLogicEngine) -> Result<(), String> {
    println!("🔍 VERIFICATION: TRANSFORMATION CORRECTNESS");
    println!("{}", "-".repeat(60));
    println!();
    
    // Test 1: Valid transformation
    println!("Test 1: Valid Transformation");
    let input = vec![0.5; 384];
    let result = engine.verify_transformation(
        &input,
        0.75,  // signal
        0.33,  // ethos
        0.33,  // logos
        0.34,  // pathos
    )?;
    
    println!("  Signal: 0.75");
    println!("  Ethos: 0.33, Logos: 0.33, Pathos: 0.34");
    println!("  Sum: {:.2}", 0.33 + 0.33 + 0.34);
    
    if result.holds() {
        println!("  ✅ VALID: {}", result.explanation);
    } else {
        println!("  ❌ INVALID");
        for violation in &result.violations {
            println!("    - {}", violation);
        }
    }
    println!();
    
    // Test 2: Invalid transformation (ELP doesn't sum to 1)
    println!("Test 2: Invalid Transformation (ELP Conservation Violation)");
    let result = engine.verify_transformation(
        &input,
        0.75,
        0.5,   // ethos
        0.5,   // logos
        0.5,   // pathos (sum = 1.5, not 1.0!)
    )?;
    
    println!("  Signal: 0.75");
    println!("  Ethos: 0.5, Logos: 0.5, Pathos: 0.5");
    println!("  Sum: {:.2}", 0.5 + 0.5 + 0.5);
    
    if result.holds() {
        println!("  ✅ VALID");
    } else {
        println!("  ❌ INVALID: {}", result.explanation);
        for violation in &result.violations {
            println!("    - {}", violation);
        }
    }
    println!();
    
    // Test 3: ELP Conservation directly
    println!("Test 3: ELP Conservation Law");
    let result = engine.verify_elp_conservation(0.33, 0.33, 0.34)?;
    
    println!("  {}", result.explanation);
    if result.holds() {
        println!("  ✅ CONSERVATION LAW SATISFIED");
    } else {
        println!("  ❌ CONSERVATION LAW VIOLATED");
    }
    println!();
    
    // Test 4: Sacred Exclusion Verification
    println!("Test 4: Sacred Exclusion Verification");
    let result = engine.verify_sacred_exclusion()?;
    
    println!("  {}", result.explanation);
    if result.is_valid() {
        println!("  ✅ SACRED POSITIONS PRESERVED");
    } else {
        println!("  ❌ SACRED EXCLUSION VIOLATED");
    }
    println!();
    
    Ok(())
}

fn demo_consistency(engine: &mut FormalLogicEngine) -> Result<(), String> {
    println!("🧮 SYSTEM CONSISTENCY CHECK");
    println!("{}", "-".repeat(60));
    println!();
    
    println!("Checking logical consistency of all axioms...");
    
    let consistent = engine.check_consistency()?;
    
    if consistent {
        println!("✅ CONSISTENT: System has no logical contradictions");
        println!();
        println!("This proves:");
        println!("  • All axioms are mutually compatible");
        println!("  • No contradictions exist");
        println!("  • Sacred geometry is logically sound");
        println!("  • Vortex mathematics is well-founded");
    } else {
        println!("❌ INCONSISTENT: System contains contradictions");
        println!();
        println!("This would indicate:");
        println!("  • Axioms conflict with each other");
        println!("  • Theoretical foundation needs revision");
    }
    
    println!();
    Ok(())
}
