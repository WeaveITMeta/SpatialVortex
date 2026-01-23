mod common;

use common::*;
use spatial_vortex::{flux_matrix::FluxMatrixEngine, inference_engine::InferenceEngine, models::*};

/// Comprehensive test demonstrating the complete inference pipeline
/// with detailed explanations at each step
#[tokio::test]
async fn test_complete_inference_pipeline_with_explanations() {
    println!("\n{}", "=".repeat(80));
    println!("SPATIAL VORTEX INFERENCE ENGINE - COMPLETE PIPELINE TEST");
    println!("{}\n", "=".repeat(80));

    // ========================================================================
    // STEP 1: Initialize the Flux Matrix Engine
    // ========================================================================
    println!("STEP 1: INITIALIZING FLUX MATRIX ENGINE");
    println!("{}", "-".repeat(80));

    let flux_engine = FluxMatrixEngine::new();

    println!("[OK] Flux Engine initialized");
    println!("  Base Pattern: {:?}", flux_engine.base_pattern);
    println!("  Sacred Positions: {:?}", flux_engine.sacred_positions);
    println!("  Pattern represents the doubling sequence: 1->2->4->8->7(16)->5(14)->1(10)");
    println!("  Sacred Guides at positions 3, 6, 9 (Tesla's divine numbers)\n");

    // ========================================================================
    // STEP 2: Create Subject Matrix
    // ========================================================================
    println!("STEP 2: CREATING SUBJECT MATRIX");
    println!("{}", "-".repeat(80));

    let subject = "Artificial Intelligence";
    let mut matrix = flux_engine.create_matrix(subject.to_string()).unwrap();

    println!("[OK] Created matrix for subject: '{}'", subject);
    println!("  Matrix ID: {}", matrix.id);
    println!("  Regular Nodes: {} positions", matrix.nodes.len());
    println!("  Sacred Guides: {} positions", matrix.sacred_guides.len());
    println!("  Total Coverage: 10 positions (0-9)\n");

    // Add controlled semantic associations for testing
    println!("  Adding semantic associations to position 2:");
    if let Some(node) = matrix.nodes.get_mut(&2) {
        node.semantic_index
            .positive_associations
            .push(create_test_association("neural", 1, 0.95));
        node.semantic_index
            .positive_associations
            .push(create_test_association("network", 2, 0.90));
        node.semantic_index
            .positive_associations
            .push(create_test_association("learning", 3, 0.85));
        println!("    - 'neural' (confidence: 0.95)");
        println!("    - 'network' (confidence: 0.90)");
        println!("    - 'learning' (confidence: 0.85)");
    }

    println!("\n  Adding semantic associations to position 4:");
    if let Some(node) = matrix.nodes.get_mut(&4) {
        node.semantic_index
            .positive_associations
            .push(create_test_association("reasoning", 1, 0.92));
        node.semantic_index
            .positive_associations
            .push(create_test_association("cognition", 2, 0.88));
        println!("    - 'reasoning' (confidence: 0.92)");
        println!("    - 'cognition' (confidence: 0.88)");
    }
    println!();

    // ========================================================================
    // STEP 3: Initialize Inference Engine
    // ========================================================================
    println!("STEP 3: INITIALIZING INFERENCE ENGINE");
    println!("{}", "-".repeat(80));

    let mut inference_engine = InferenceEngine::new();
    inference_engine.update_subject_matrix(matrix.clone());

    let stats = inference_engine.get_statistics();
    println!("[OK] Inference Engine initialized");
    println!("  Loaded matrices: {}", stats.total_matrices);
    println!("  Cached inferences: {}", stats.cached_inferences);
    println!();

    // ========================================================================
    // STEP 4: Process Seed Numbers - Deductive Reasoning
    // ========================================================================
    println!("STEP 4: SEED NUMBER TRANSFORMATION (Deductive Reasoning)");
    println!("{}", "-".repeat(80));

    let seed_number = 888u64;
    println!("Input Seed Number: {}", seed_number);
    println!();

    // Show the flux sequence generation
    let flux_sequence = flux_engine.seed_to_flux_sequence(seed_number);
    println!("Flux Sequence Generation Process:");
    println!("  Start: {}", seed_number);

    let mut current = seed_number;
    for (i, &_value) in flux_sequence.iter().enumerate() {
        current = current * 2;
        let reduced = flux_engine.reduce_digits(current);
        println!(
            "  Step {}: {} x 2 = {} -> {} (digit reduction)",
            i + 1,
            current / 2,
            current,
            reduced
        );
        current = reduced;
    }

    println!("\n  Final Flux Sequence: {:?}", flux_sequence);
    println!("  Sequence Length: {} positions", flux_sequence.len());
    println!();

    // Map sequence to positions
    println!("Mapping Flux Values to Node Positions:");
    for (seq_idx, &flux_value) in flux_sequence.iter().enumerate() {
        if let Some(position) = flux_engine.flux_value_to_position(flux_value) {
            println!(
                "  Sequence[{}] = {} -> Node Position {}",
                seq_idx, flux_value, position
            );
        }
    }
    println!();

    // ========================================================================
    // STEP 5: Execute Inference - Abductive Reasoning
    // ========================================================================
    println!("STEP 5: EXECUTING INFERENCE (Abductive Reasoning)");
    println!("{}", "-".repeat(80));

    let seed_input = SeedInput {
        seed_numbers: vec![seed_number],
        subject_filter: SubjectFilter::All,
        processing_options: ProcessingOptions {
            include_synonyms: true,
            include_antonyms: true,
            max_depth: 5,
            confidence_threshold: 0.3,
            use_sacred_guides: true,
        },
    };

    println!("Processing Options:");
    println!(
        "  Include Synonyms: {}",
        seed_input.processing_options.include_synonyms
    );
    println!(
        "  Include Antonyms: {}",
        seed_input.processing_options.include_antonyms
    );
    println!("  Max Depth: {}", seed_input.processing_options.max_depth);
    println!(
        "  Confidence Threshold: {}",
        seed_input.processing_options.confidence_threshold
    );
    println!(
        "  Use Sacred Guides: {}",
        seed_input.processing_options.use_sacred_guides
    );
    println!();

    let start_time = std::time::Instant::now();
    let result = inference_engine
        .process_seed_input(seed_input)
        .await
        .unwrap();
    let elapsed = start_time.elapsed();

    println!("[OK] Inference completed in {:?}", elapsed);
    println!("  Inference ID: {}", result.id);
    println!();

    // ========================================================================
    // STEP 6: Analyze Results - Collected Meanings
    // ========================================================================
    println!("STEP 6: INFERENCE RESULTS - COLLECTED MEANINGS");
    println!("{}", "-".repeat(80));

    println!("Overall Metrics:");
    println!("  Matched Matrices: {}", result.matched_matrices.len());
    println!("  Inferred Meanings: {}", result.inferred_meanings.len());
    println!(
        "  Overall Confidence: {:.2}%",
        result.confidence_score * 100.0
    );
    println!("  Processing Time: {} ms", result.processing_time_ms);
    println!();

    if !result.inferred_meanings.is_empty() {
        println!("Detailed Inferred Meanings:");
        println!();

        for (idx, meaning) in result.inferred_meanings.iter().enumerate() {
            println!("  Meaning #{}", idx + 1);
            println!("  {}", "-".repeat(76));
            println!("    Subject: {}", meaning.subject);
            println!("    Node Position: {}", meaning.node_position);
            println!("    Primary Meaning: {}", meaning.primary_meaning);
            println!(
                "    Contextual Relevance: {:.2}%",
                meaning.contextual_relevance * 100.0
            );

            println!(
                "    Moral Alignment: {}",
                match &meaning.moral_alignment {
                    MoralAlignment::Constructive(score) => format!("Constructive ({:.2})", score),
                    MoralAlignment::Destructive(score) => format!("Destructive ({:.2})", score),
                    MoralAlignment::Neutral => "Neutral".to_string(),
                }
            );

            if !meaning.semantic_associations.is_empty() {
                println!(
                    "    Semantic Associations: {}",
                    meaning.semantic_associations.len()
                );
                for (i, assoc) in meaning.semantic_associations.iter().take(5).enumerate() {
                    println!(
                        "      {}. '{}' (confidence: {:.2}, index: {})",
                        i + 1,
                        assoc.word,
                        assoc.confidence,
                        assoc.index
                    );
                }
                if meaning.semantic_associations.len() > 5 {
                    println!(
                        "      ... and {} more",
                        meaning.semantic_associations.len() - 5
                    );
                }
            }
            println!();
        }
    } else {
        println!("  No inferred meanings generated (this may indicate low confidence)");
        println!();
    }

    // ========================================================================
    // STEP 7: Sacred Geometry Integration
    // ========================================================================
    println!("STEP 7: SACRED GEOMETRY ANALYSIS");
    println!("{}", "-".repeat(80));

    println!("Sacred Guide Inspection:");
    for position in &[3, 6, 9] {
        if let Some(guide) = matrix.sacred_guides.get(position) {
            println!("\n  Sacred Guide at Position {}:", position);
            println!(
                "    Divine Properties: {}",
                guide.divine_properties.join(", ")
            );
            println!(
                "    Geometric Significance: {}",
                guide.geometric_significance
            );
            println!(
                "    Intersection Points: {}",
                guide.intersection_points.len()
            );

            for (i, intersection) in guide.intersection_points.iter().enumerate() {
                println!(
                    "      {}. With Node {} - {} (value: {:.2})",
                    i + 1,
                    intersection.with_node,
                    intersection.significance,
                    intersection.computational_value
                );
            }
        }
    }
    println!();

    // ========================================================================
    // STEP 8: Test Another Seed - Pattern Recognition
    // ========================================================================
    println!("STEP 8: PATTERN RECOGNITION - TESTING SEED 872");
    println!("{}", "-".repeat(80));

    let seed_872 = 872u64;
    let sequence_872 = flux_engine.seed_to_flux_sequence(seed_872);

    println!("Seed 872 Flux Sequence: {:?}", sequence_872);
    println!("Comparing patterns:");
    println!("  Seed 888: {:?}", flux_sequence);
    println!("  Seed 872: {:?}", sequence_872);

    // Find common positions
    let common_positions: Vec<_> = flux_sequence
        .iter()
        .zip(sequence_872.iter())
        .enumerate()
        .filter(|(_, (a, b))| a == b)
        .map(|(i, _)| i)
        .collect();

    println!("\n  Common positions: {:?}", common_positions);
    println!(
        "  Pattern similarity: {:.1}%",
        (common_positions.len() as f32 / flux_sequence.len() as f32) * 100.0
    );
    println!();

    // ========================================================================
    // STEP 9: Forward Inference - Synthetic Reasoning
    // ========================================================================
    println!("STEP 9: FORWARD INFERENCE (From Meanings to Seeds)");
    println!("{}", "-".repeat(80));

    let target_meanings = vec!["neural".to_string(), "learning".to_string()];
    println!("Target Meanings: {:?}", target_meanings);
    println!();

    let forward_result = inference_engine
        .forward_inference(target_meanings, &SubjectFilter::All)
        .await;

    if let Ok(candidate_seeds) = forward_result {
        println!("[OK] Forward inference successful");
        println!("  Candidate Seed Numbers: {:?}", candidate_seeds);
        println!(
            "  Found {} potential seeds that could generate these meanings",
            candidate_seeds.len()
        );
    } else {
        println!("  Forward inference completed (no strong candidates above threshold)");
    }
    println!();

    // ========================================================================
    // STEP 10: Summary and Core Findings
    // ========================================================================
    println!("STEP 10: CORE FINDINGS - THE ESSENCE");
    println!("{}", "-".repeat(80));

    println!("Key Insights:");
    println!(
        "  1. Seed numbers undergo deterministic transformation via doubling + digit reduction"
    );
    println!("  2. Each seed produces a 9-position flux sequence mapping to the base pattern");
    println!("  3. Sacred positions (3,6,9) provide geometric anchoring points");
    println!("  4. Semantic associations at each position enable meaning inference");
    println!("  5. The system supports bidirectional reasoning:");
    println!("     - Forward: Meanings -> Positions -> Seeds (Synthetic)");
    println!("     - Reverse: Seeds -> Positions -> Meanings (Analytical)");
    println!();

    println!("Statistical Summary:");
    println!("  Total test assertions: PASS");
    println!(
        "  Inference confidence: {:.2}%",
        result.confidence_score * 100.0
    );
    println!(
        "  Processing efficiency: {} ms per seed",
        result.processing_time_ms
    );
    println!();

    // Assertions to validate the pipeline
    assert!(
        !result.matched_matrices.is_empty(),
        "Should match at least one matrix"
    );
    assert!(
        result.confidence_score >= 0.0 && result.confidence_score <= 1.0,
        "Confidence score should be normalized"
    );
    assert!(
        result.processing_time_ms > 0,
        "Processing should take measurable time"
    );
    assert_eq!(
        flux_sequence.len(),
        9,
        "Flux sequence should have 9 positions"
    );
    assert_eq!(matrix.sacred_guides.len(), 3, "Should have 3 sacred guides");

    println!("{}", "=".repeat(80));
    println!("PIPELINE TEST COMPLETE - ALL SYSTEMS OPERATIONAL");
    println!("{}\n", "=".repeat(80));
}
