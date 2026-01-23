//! Hallucination Detection Demo
//! 
//! Demonstrates the Vortex Context Preserver (VCP) framework for detecting and mitigating
//! hallucinations in time series-like BeamTensor sequences.
//!
//! Run with:
//! cargo run --example hallucination_demo

use spatial_vortex::hallucinations::{VortexContextPreserver, HallucinationDetector, SignalSubspace};
use spatial_vortex::models::BeamTensor;

fn main() {
    println!("🌀 Vortex Context Preserver (VCP) - Hallucination Detection Demo\n");
    
    // Demo 1: Basic Signal Subspace Analysis
    println!("═══ Demo 1: Signal Subspace Analysis ═══");
    demo_signal_subspace();
    println!();
    
    // Demo 2: Hallucination Detection
    println!("═══ Demo 2: Hallucination Detection ═══");
    demo_hallucination_detection();
    println!();
    
    // Demo 3: Sacred Position Interventions
    println!("═══ Demo 3: Sacred Position Interventions ═══");
    demo_sacred_interventions();
    println!();
    
    // Demo 4: Vortex vs. Linear Comparison
    println!("═══ Demo 4: Vortex vs. Linear Transformer ═══");
    demo_vortex_vs_linear();
    println!();
    
    println!("✅ All demos completed!");
}

fn demo_signal_subspace() {
    // Create a sequence of BeamTensors with varying distributions
    let mut beams = Vec::new();
    
    for i in 0..10 {
        let mut beam = BeamTensor::default();
        
        // Create peaked distributions
        beam.digits = [0.0; 9];
        beam.digits[i % 9] = 0.7;
        beam.digits[(i + 1) % 9] = 0.2;
        beam.digits[(i + 2) % 9] = 0.1;
        
        beam.position = ((i % 9) + 1) as u8;
        beam.confidence = 0.8;
        
        beams.push(beam);
    }
    
    // Compute signal subspace
    let subspace = SignalSubspace::from_beam_tensors(&beams, 5);
    
    println!("Sequence length: {}", beams.len());
    println!("Subspace rank: {}", subspace.rank);
    println!("Signal strength: {:.3}", subspace.strength);
    println!("Top singular values: {:?}", 
        subspace.singular_values.iter()
            .take(3)
            .map(|v| format!("{:.3}", v))
            .collect::<Vec<_>>()
    );
    
    // Project a beam onto subspace
    let projected = subspace.project(&beams[0]);
    println!("\nOriginal beam distribution: {:?}", 
        beams[0].digits.iter()
            .map(|v| format!("{:.2}", v))
            .collect::<Vec<_>>()
    );
    println!("Projected distribution: {:?}",
        projected.iter()
            .map(|v| format!("{:.2}", v))
            .collect::<Vec<_>>()
    );
}

fn demo_hallucination_detection() {
    let detector = HallucinationDetector::default();
    
    // Create stable context
    let mut context = Vec::new();
    for _ in 0..5 {
        let mut beam = BeamTensor::default();
        beam.ethos = 5.0;
        beam.logos = 5.0;
        beam.pathos = 5.0;
        beam.confidence = 0.8;
        context.push(beam);
    }
    
    // Test Case 1: Normal forecast (no hallucination)
    let mut normal_forecast = Vec::new();
    for _ in 0..3 {
        let mut beam = BeamTensor::default();
        beam.ethos = 5.2;  // Slight variation
        beam.logos = 4.8;
        beam.pathos = 5.1;
        normal_forecast.push(beam);
    }
    
    let result1 = detector.detect_hallucination(&context, &normal_forecast);
    println!("Test Case 1: Normal Forecast");
    println!("  Hallucination: {}", result1.is_hallucination);
    println!("  Signal strength: {:.3}", result1.confidence);
    println!("  Dynamics divergence: {:.3}", result1.dynamics_divergence);
    println!("  Confidence score: {:.3}", result1.confidence_score);
    
    // Test Case 2: Hallucinated forecast (divergent dynamics)
    let mut hallucinated_forecast = Vec::new();
    for _ in 0..3 {
        let mut beam = BeamTensor::default();
        beam.ethos = 1.0;  // Drastically different
        beam.logos = 9.0;
        beam.pathos = 2.0;
        hallucinated_forecast.push(beam);
    }
    
    let result2 = detector.detect_hallucination(&context, &hallucinated_forecast);
    println!("\nTest Case 2: Hallucinated Forecast");
    println!("  Hallucination: {}", result2.is_hallucination);
    println!("  Signal strength: {:.3}", result2.confidence);
    println!("  Dynamics divergence: {:.3}", result2.dynamics_divergence);
    println!("  Confidence score: {:.3}", result2.confidence_score);
}

fn demo_sacred_interventions() {
    let cascade = VortexContextPreserver::default();
    
    // Create beam sequence passing through flux pattern
    let flux_pattern = [1, 2, 4, 8, 7, 5, 1, 2, 4];  // Including sacred positions
    let mut beams = Vec::new();
    
    for pos in flux_pattern {
        let mut beam = BeamTensor::default();
        beam.position = pos;
        beam.confidence = 0.6;
        beam.digits = [0.1; 9];
        beam.digits[pos as usize - 1] = 0.5;  // Peak at position
        beams.push(beam);
    }
    
    // Add sacred positions
    for sacred_pos in [3, 6, 9] {
        let mut beam = BeamTensor::default();
        beam.position = sacred_pos;
        beam.confidence = 0.6;
        beams.push(beam);
    }
    
    println!("Before interventions:");
    for (i, beam) in beams.iter().enumerate() {
        if matches!(beam.position, 3 | 6 | 9) {
            println!("  Beam {}: Position {} (SACRED), confidence={:.3}", 
                i, beam.position, beam.confidence);
        }
    }
    
    // Apply interventions
    let _results = cascade.process_with_interventions(&mut beams, true);
    
    println!("\nAfter interventions:");
    for (i, beam) in beams.iter().enumerate() {
        if matches!(beam.position, 3 | 6 | 9) {
            println!("  Beam {}: Position {} (SACRED), confidence={:.3} (+15% boost), confidence={:.3}", 
                i, beam.position, beam.confidence, beam.confidence);
        }
    }
}

fn demo_vortex_vs_linear() {
    let cascade = VortexContextPreserver::new(0.5, 5, 1.5);
    
    // Initial context
    let mut initial = Vec::new();
    for i in 0..5 {
        let mut beam = BeamTensor::default();
        beam.position = ((i % 9) + 1) as u8;
        beam.confidence = 0.8;
        beam.confidence = 0.7;
        initial.push(beam);
    }
    
    println!("Initial signal strength: {:.3}", initial[0].confidence);
    
    // Compare propagation over 20 steps
    let (vortex_strength, linear_strength) = cascade.compare_propagation_methods(
        &initial,
        20,
    );
    
    println!("\nAfter 20 propagation steps:");
    println!("  Vortex (cyclic + interventions): {:.3}", vortex_strength);
    println!("  Linear (no cycles): {:.3}", linear_strength);
    
    let improvement = ((vortex_strength - linear_strength) / linear_strength) * 100.0;
    println!("\n🎯 Vortex improvement: {:.1}%", improvement);
    
    if vortex_strength > linear_strength {
        println!("✅ Vortex propagation preserves context better!");
        println!("   Sacred position interventions reduce hallucination risk.");
    } else {
        println!("⚠️  Results may vary - try different sequence lengths.");
    }
}
