//! Demonstration of Dynamic ELP with RL Gradient Optimization
//!
//! Shows the evolution from static to dynamic ELP attributes with
//! reinforcement learning for gradient optimization and next-word prediction.

use spatial_vortex::data::{DynamicELP, FluxSubject, AttributeState};
use spatial_vortex::ml::rl_gradient_optimizer::{RLGradientOptimizer, GradientState};
use spatial_vortex::core::sacred_geometry::flux_matrix::FluxMatrixEngine;
use spatial_vortex::storage::confidence_lake::PostgresConfidenceLake;
use nalgebra::DVector;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🌟 Dynamic ELP with RL Gradient Optimization Demo");
    println!("{}", "=".repeat(60));
    println!();
    
    // Initialize components
    let flux_engine = FluxMatrixEngine::new();
    let mut rl_optimizer = RLGradientOptimizer::new(0.001, 64);
    let confidence_lake = Arc::new(PostgresConfidenceLake::new(":memory:").await?);
    
    println!("📊 Demonstrating Dynamic ELP Evolution\n");
    
    // Phase 1: Static to Dynamic ELP Transformation
    println!("Phase 1: Creating Dynamic ELP from Sacred Positions");
    println!("{}", "-".repeat(40));
    
    // Create subjects from each sacred position
    let ethos_subject = FluxSubject::from_sacred_position(3, "ethical context");
    let logos_subject = FluxSubject::from_sacred_position(6, "logical analysis");
    let pathos_subject = FluxSubject::from_sacred_position(9, "emotional narrative");
    
    // Generate dynamic ELP for each
    let mut ethos_elp = DynamicELP::from_subject(&ethos_subject, 3);
    let mut logos_elp = DynamicELP::from_subject(&logos_subject, 6);
    let mut pathos_elp = DynamicELP::from_subject(&pathos_subject, 9);
    
    println!("Position 3 (Ethos): {}", format_elp_state(&ethos_elp));
    println!("Position 6 (Logos): {}", format_elp_state(&logos_elp));
    println!("Position 9 (Pathos): {}", format_elp_state(&pathos_elp));
    println!();
    
    // Phase 2: Comparative Signal Dynamics
    println!("Phase 2: Applying Comparative Signal Dynamics");
    println!("{}", "-".repeat(40));
    
    // Simulate input signals and adjust
    let signals = vec![
        (0.3, "Low signal - needs boost"),
        (0.7, "Strong signal - maintain balance"),
        (0.95, "Very strong - risk of dominance"),
    ];
    
    for (signal, desc) in signals {
        println!("\nApplying signal {:.2}: {}", signal, desc);
        
        // Adjust each ELP based on signal
        ethos_elp.adjust(signal, 1.0);
        logos_elp.adjust(signal, 1.0);
        pathos_elp.adjust(signal, 1.0);
        
        // Show harmony scores
        println!("  Ethos harmony: {:.3}", ethos_elp.harmony_score());
        println!("  Logos harmony: {:.3}", logos_elp.harmony_score());
        println!("  Pathos harmony: {:.3}", pathos_elp.harmony_score());
        
        // Check for critical states
        if pathos_elp.state == AttributeState::Critical {
            println!("  ⚠️ CRITICAL: Pathos dominance detected - applying correction");
        }
    }
    
    // Phase 3: RL Gradient Optimization
    println!("\n\nPhase 3: RL-Driven Gradient Optimization");
    println!("{}", "-".repeat(40));
    
    // Create gradient states for each ELP
    let gradient_states = vec![
        GradientState {
            gradients: DVector::from_element(10, 0.5),
            elp: ethos_elp.clone(),
            flux_position: 3,
            confidence: 0.8,
            cumulative_reward: 0.0,
        },
        GradientState {
            gradients: DVector::from_element(10, 0.6),
            elp: logos_elp.clone(),
            flux_position: 6,
            confidence: 0.7,
            cumulative_reward: 0.0,
        },
        GradientState {
            gradients: DVector::from_element(10, 0.7),
            elp: pathos_elp.clone(),
            flux_position: 9,
            confidence: 0.6,
            cumulative_reward: 0.0,
        },
    ];
    
    // Process each state through RL optimizer
    for (i, state) in gradient_states.iter().enumerate() {
        println!("\nProcessing Position {} State:", state.flux_position);
        
        // Select action based on state
        let action = rl_optimizer.select_action(state);
        println!("  Selected Action: {:?}", action);
        
        // Simulate forward propagation
        let input = DVector::from_element(10, 0.5 + i as f32 * 0.1);
        let forward_result = rl_optimizer.forward_propagation(&input, state)?;
        println!("  Forward prop output norm: {:.3}", forward_result.norm());
        
        // Simulate backward propagation with halving
        let gradients = DVector::from_element(10, 1.0);
        let backward_result = rl_optimizer.backward_propagation(gradients, state)?;
        println!("  Backward prop output norm: {:.3}", backward_result.norm());
        
        // Calculate reward
        let reward = rl_optimizer.calculate_reward(state, "predicted", "target");
        println!("  Reward: {:.3} ({})", 
            reward.immediate,
            if reward.is_positive { "Positive" } else { "Negative" }
        );
        
        if reward.should_halve {
            println!("  📉 Halving sequence should be applied");
        }
    }
    
    // Phase 4: Knowledge Retention Decision
    println!("\n\nPhase 4: Knowledge Retention in Confidence Lake");
    println!("{}", "-".repeat(40));
    
    let threshold = 0.6;
    println!("Retention threshold: {:.2}\n", threshold);
    
    for elp in [&ethos_elp, &logos_elp, &pathos_elp] {
        let importance = elp.importance();
        let should_retain = elp.should_retain(threshold);
        
        println!("Position {}: Importance={:.3}, Retain={}",
            elp.dominant_value,
            importance,
            if should_retain { "✅ YES" } else { "❌ NO" }
        );
        
        if should_retain {
            // Store in Confidence Lake
            use spatial_vortex::ai::orchestrator::ASIOutput;
            
            let output = ASIOutput {
                result: format!("Dynamic ELP State at position {}", elp.dominant_value),
                elp: elp.to_static_elp(),
                flux_position: elp.dominant_value,
                confidence: importance,
                confidence: importance,
                is_sacred: [3, 6, 9].contains(&elp.dominant_value),
                mode: spatial_vortex::ai::orchestrator::ExecutionMode::Comprehensive,
                consensus_used: false,
                processing_time_ms: 10,
            };
            
            let id = confidence_lake.store_diamond(&output).await?;
            println!("  💎 Stored in Confidence Lake with ID: {}", id);
        }
    }
    
    // Phase 5: Halving Sequence Demonstration
    println!("\n\nPhase 5: Halving Sequence in Backward Flow");
    println!("{}", "-".repeat(40));
    
    let halving_sequence = vec![1, 5, 7, 8, 4, 2, 1];
    println!("Backward flow sequence: {:?}\n", halving_sequence);
    
    for position in halving_sequence {
        let mut test_state = gradient_states[0].clone();
        test_state.flux_position = position;
        test_state.confidence = 0.4;  // Low signal to trigger halving
        
        let gradients = DVector::from_element(10, 2.0);
        let result = rl_optimizer.backward_propagation(gradients.clone(), &test_state)?;
        
        let reduction_factor = result.norm() / gradients.norm();
        println!("Position {}: Gradient reduced to {:.2}% of original",
            position,
            reduction_factor * 100.0
        );
    }
    
    // Final Summary
    println!("\n{}", "=".repeat(60));
    println!("\n✨ Demo Complete - Key Innovations Demonstrated:\n");
    
    println!("1. **Dynamic ELP Attributes**:");
    println!("   - Derived from sacred 3-6-9 positions");
    println!("   - 3D vectors with comparative dynamics");
    println!("   - State tracking (Stable/Fluctuating/Emergent/Critical)");
    
    println!("\n2. **Harmony Enforcement**:");
    println!("   - Automatic pathos dominance prevention");
    println!("   - Emergency ethos/logos boosting");
    println!("   - Balance maintained through adjustments");
    
    println!("\n3. **RL Gradient Optimization**:");
    println!("   - PPO-style policy updates");
    println!("   - Sacred position rewards (3=1.5x, 6=1.3x, 9=1.2x)");
    println!("   - Halving sequence for backward propagation");
    
    println!("\n4. **Knowledge Retention**:");
    println!("   - Importance-based filtering");
    println!("   - Confidence Lake storage for high-value states");
    println!("   - Trajectory tracking for learning");
    
    println!("\n5. **Technical Correctness**:");
    println!("   - Chain rule in forward propagation");
    println!("   - Exponential reduction functions");
    println!("   - Gradient clipping for failing sequences");
    
    println!("\n🚀 The system now supports both inference AND training!");
    println!("   Static ELP → Dynamic ELP → RL Optimization → AGI/ASI Path");
    
    Ok(())
}

fn format_elp_state(elp: &DynamicELP) -> String {
    format!(
        "E={:.2}, L={:.2}, P={:.2} | State={:?} | Importance={:.3}",
        elp.ethos.norm(),
        elp.logos.norm(),
        elp.pathos.norm(),
        elp.state,
        elp.importance()
    )
}
