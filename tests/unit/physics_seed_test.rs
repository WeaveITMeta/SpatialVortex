mod common;

use spatial_vortex::{flux_matrix::FluxMatrixEngine, inference_engine::InferenceEngine, models::*};

/// Test reverse reasoning with Physics subject and seed 36901248751 (seeds → meanings)
#[tokio::test]
async fn test_physics_seed_1248751_reverse_reasoning() {
    println!("\n{}", "=".repeat(90));
    println!("SPATIAL VORTEX ENGINE - REVERSE REASONING TEST");
    println!("Subject: Physics | Seed Number: 36901248751");
    println!("{}\n", "=".repeat(90));

    // ============================================================================
    // PHASE 1: ENGINE INITIALIZATION
    // ============================================================================
    println!("PHASE 1: INITIALIZING SPATIAL VORTEX ENGINE");
    println!("{}\n", "-".repeat(90));

    let flux_engine = FluxMatrixEngine::new();
    let mut inference_engine = InferenceEngine::new();

    println!("[OK] Flux Matrix Engine initialized");
    println!("  Core Pattern: {:?}", flux_engine.base_pattern);
    println!("  Sacred Anchors: {:?}\n", flux_engine.sacred_positions);

    // ============================================================================
    // PHASE 2: SUBJECT MATRIX CREATION - PHYSICS
    // ============================================================================
    println!("PHASE 2: GENERATING FLUX MATRIX FOR SUBJECT 'Physics'");
    println!("{}\n", "-".repeat(90));

    let matrix = flux_engine.create_matrix("Physics".to_string()).unwrap();

    println!("[OK] Physics Matrix Created");
    println!("  Matrix ID: {}", matrix.id);
    println!("  Subject: {}", matrix.subject);
    println!("  Created: {}", matrix.created_at);
    println!();

    println!("Matrix Composition:");
    println!("  Regular Flux Nodes: {}", matrix.nodes.len());
    println!("  Sacred Guides: {}", matrix.sacred_guides.len());
    println!(
        "  Total Positions: {}\n",
        matrix.nodes.len() + matrix.sacred_guides.len()
    );

    // Show node structure
    println!("Regular Node Positions and Base Values:");
    let mut positions: Vec<_> = matrix.nodes.keys().collect();
    positions.sort();
    for pos in positions {
        let node = &matrix.nodes[pos];
        println!(
            "  Position {} -> Base Value: {} | Primary: '{}'",
            pos, node.base_value, node.semantic_index.neutral_base
        );
    }
    println!();

    println!("Sacred Guide Positions:");
    for pos in &[3, 6, 9] {
        if let Some(guide) = matrix.sacred_guides.get(pos) {
            println!("  Position {} -> {}", pos, guide.geometric_significance);
        }
    }
    println!();

    // Load matrix into inference engine
    inference_engine.update_subject_matrix(matrix.clone());
    println!("[OK] Matrix loaded into Inference Engine\n");

    // ============================================================================
    // PHASE 3: SEED NUMBER TRANSFORMATION (REVERSE REASONING BEGINS)
    // ============================================================================
    println!("PHASE 3: SEED NUMBER TRANSFORMATION - ANALYTICAL REASONING");
    println!("{}\n", "-".repeat(90));

    let seed_number = 36901248751u64;
    println!("INPUT SEED: {}", seed_number);
    println!();

    // Extract digits from seed number - each digit maps to a position
    println!("Seed Digit Extraction (Direct Position Mapping):");
    println!("{}", "-".repeat(90));

    let flux_sequence = flux_engine.seed_to_flux_sequence(seed_number);
    let seed_digits: Vec<u8> = seed_number
        .to_string()
        .chars()
        .map(|c| c.to_digit(10).unwrap() as u8)
        .collect();

    for (step, (&digit, &flux_value)) in seed_digits.iter().zip(flux_sequence.iter()).enumerate() {
        println!(
            "  Step {:2} | Seed Digit: {} -> Maps to Position: {} (Flux Value: {})",
            step + 1,
            digit,
            digit,
            flux_value
        );
    }

    println!();
    println!("GENERATED FLUX SEQUENCE: {:?}", flux_sequence);
    println!("  Length: {} positions", flux_sequence.len());
    println!();

    // ============================================================================
    // PHASE 4: FLUX VALUE TO POSITION MAPPING
    // ============================================================================
    println!("PHASE 4: MAPPING FLUX VALUES TO MATRIX POSITIONS");
    println!("{}\n", "-".repeat(90));

    println!("Flux Sequence -> Node/Guide Mapping:");
    println!("{}", "-".repeat(90));

    for (seq_idx, &flux_value) in flux_sequence.iter().enumerate() {
        // Check if it's a sacred position
        if [3, 6, 9].contains(&flux_value) {
            if let Some(guide) = matrix.sacred_guides.get(&flux_value) {
                println!(
                    "  Seq[{}] = {} -> SACRED GUIDE at Position {} | '{}'",
                    seq_idx,
                    flux_value,
                    flux_value,
                    guide
                        .divine_properties
                        .first()
                        .unwrap_or(&"Unknown".to_string())
                );
            }
        } else if let Some(position) = flux_engine.flux_value_to_position(flux_value) {
            if let Some(node) = matrix.nodes.get(&position) {
                println!(
                    "  Seq[{}] = {} -> Regular Node at Position {} | '{}'",
                    seq_idx, flux_value, position, node.semantic_index.neutral_base
                );
            }
        } else {
            println!(
                "  Seq[{}] = {} -> (Position not found in base pattern)",
                seq_idx, flux_value
            );
        }
    }
    println!();

    // ============================================================================
    // PHASE 5: SEMANTIC ASSOCIATION EXTRACTION
    // ============================================================================
    println!("PHASE 5: EXTRACTING SEMANTIC ASSOCIATIONS");
    println!("{}\n", "-".repeat(90));

    println!("Semantic Index Analysis:");
    println!("{}", "-".repeat(90));

    for (_seq_idx, &flux_value) in flux_sequence.iter().enumerate() {
        if ![3, 6, 9].contains(&flux_value) {
            if let Some(position) = flux_engine.flux_value_to_position(flux_value) {
                if let Some(node) = matrix.nodes.get(&position) {
                    println!("\n  Position {} (Flux Value: {}):", position, flux_value);
                    println!(
                        "    Primary Meaning: '{}'",
                        node.semantic_index.neutral_base
                    );
                    if !node.semantic_index.positive_associations.is_empty() {
                        println!(
                            "    Positive Associations ({}):",
                            node.semantic_index.positive_associations.len()
                        );
                        for (i, assoc) in node
                            .semantic_index
                            .positive_associations
                            .iter()
                            .take(5)
                            .enumerate()
                        {
                            println!(
                                "      {} '{}' (index: {}, confidence: {:.2})",
                                if i == 0 { "+" } else { " " },
                                assoc.word,
                                assoc.index,
                                assoc.confidence
                            );
                        }
                        if node.semantic_index.positive_associations.len() > 5 {
                            println!(
                                "       ... and {} more",
                                node.semantic_index.positive_associations.len() - 5
                            );
                        }
                    }

                    if !node.semantic_index.negative_associations.is_empty() {
                        println!(
                            "    Negative Associations ({}):",
                            node.semantic_index.negative_associations.len()
                        );
                        for (i, assoc) in node
                            .semantic_index
                            .negative_associations
                            .iter()
                            .take(3)
                            .enumerate()
                        {
                            println!(
                                "      {} '{}' (index: {}, confidence: {:.2})",
                                if i == 0 { "-" } else { " " },
                                assoc.word,
                                assoc.index,
                                assoc.confidence
                            );
                        }
                    }
                }
            }
        }
    }
    println!();

    // ============================================================================
    // PHASE 6: INFERENCE EXECUTION
    // ============================================================================
    println!("PHASE 6: EXECUTING INFERENCE ENGINE");
    println!("{}\n", "-".repeat(90));

    let seed_input = SeedInput {
        seed_numbers: vec![seed_number],
        subject_filter: SubjectFilter::Specific("Physics".to_string()),
        processing_options: ProcessingOptions {
            include_synonyms: true,
            include_antonyms: true,
            max_depth: 5,
            confidence_threshold: 0.3,
            use_sacred_guides: true,
        },
    };

    println!("Processing Configuration:");
    println!("  Subject Filter: Physics (Specific)");
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
        "  Sacred Guides: {}",
        seed_input.processing_options.use_sacred_guides
    );
    println!();

    let start_time = std::time::Instant::now();
    let result = inference_engine
        .process_seed_input(seed_input)
        .await
        .unwrap();
    let elapsed = start_time.elapsed();

    println!("[OK] Inference Complete");
    println!("  Processing Time: {:?}", elapsed);
    println!("  Inference ID: {}", result.id);
    println!();

    // ============================================================================
    // PHASE 7: INFERENCE RESULTS - REVERSE REASONING OUTPUT
    // ============================================================================
    println!("PHASE 7: INFERENCE RESULTS - REVERSE REASONING OUTPUT");
    println!("{}\n", "=".repeat(90));

    println!("OVERALL METRICS:");
    println!("{}", "-".repeat(90));
    println!(
        "  Matched Subject Matrices: {}",
        result.matched_matrices.len()
    );
    println!(
        "  Total Inferred Meanings: {}",
        result.inferred_meanings.len()
    );
    println!(
        "  Overall Confidence Score: {:.2}%",
        result.confidence_score * 100.0
    );
    println!("  Processing Duration: {} ms", result.processing_time_ms);
    println!("  Timestamp: {}", result.created_at);
    println!();

    if !result.inferred_meanings.is_empty() {
        println!("DETAILED INFERRED MEANINGS:");
        println!("{}\n", "=".repeat(90));

        for (idx, meaning) in result.inferred_meanings.iter().enumerate() {
            println!("[--- MEANING #{} {}", idx + 1, "-".repeat(77));
            println!("|");
            println!("| Subject: {}", meaning.subject);
            println!("| Node Position: {}", meaning.node_position);
            println!("| Primary Meaning: '{}'", meaning.primary_meaning);
            println!(
                "| Contextual Relevance: {:.2}%",
                meaning.contextual_relevance * 100.0
            );
            println!("|");

            let alignment_str = match &meaning.moral_alignment {
                MoralAlignment::Constructive(score) => {
                    format!("Constructive (Heaven) - Score: {:.2}", score)
                }
                MoralAlignment::Destructive(score) => {
                    format!("Destructive (Hell) - Score: {:.2}", score)
                }
                MoralAlignment::Neutral => "Neutral (Balanced)".to_string(),
            };
            println!("| Moral Alignment: {}", alignment_str);
            println!("|");

            if !meaning.semantic_associations.is_empty() {
                println!(
                    "| Semantic Associations: {} total",
                    meaning.semantic_associations.len()
                );
                println!("|");

                // Group by positive/negative index
                let positive: Vec<_> = meaning
                    .semantic_associations
                    .iter()
                    .filter(|a| a.index > 0)
                    .collect();
                let negative: Vec<_> = meaning
                    .semantic_associations
                    .iter()
                    .filter(|a| a.index < 0)
                    .collect();

                if !positive.is_empty() {
                    println!("|   POSITIVE ASSOCIATIONS (Heaven/+):");
                    for assoc in positive.iter().take(8) {
                        println!(
                            "| + '{}' [index: {}, confidence: {:.2}]",
                            assoc.word, assoc.index, assoc.confidence
                        );
                    }
                    if positive.len() > 8 {
                        println!(
                            "      ... and {} more positive associations",
                            positive.len() - 8
                        );
                    }
                    println!("|");
                }

                if !negative.is_empty() {
                    println!("|   NEGATIVE ASSOCIATIONS (Hell/-):");
                    for assoc in negative.iter().take(5) {
                        println!(
                            "| - '{}' [index: {}, confidence: {:.2}]",
                            assoc.word, assoc.index, assoc.confidence
                        );
                    }
                    if negative.len() > 5 {
                        println!(
                            "      ... and {} more negative associations",
                            negative.len() - 5
                        );
                    }
                    println!("|");
                }
            }

            println!("[{}]", "-".repeat(89));
            println!();
        }
    } else {
        println!("No semantic meanings inferred.");
        println!("  This may indicate:");
        println!("  * The flux sequence only activated sacred guides (pure geometry)");
        println!(
            "  * Confidence scores below threshold ({})",
            result.input.processing_options.confidence_threshold
        );
        println!("  * Subject matrix has no semantic associations loaded");
        println!();
    }

    // ============================================================================
    // PHASE 8: SACRED GEOMETRY ACTIVATION ANALYSIS
    // ============================================================================
    println!("PHASE 8: SACRED GEOMETRY ACTIVATION ANALYSIS");
    println!("{}\n", "=".repeat(90));

    let sacred_activations: Vec<_> = flux_sequence
        .iter()
        .enumerate()
        .filter(|(_, &val)| [3, 6, 9].contains(&val))
        .collect();

    if !sacred_activations.is_empty() {
        println!(
            "Sacred Guide Activations Detected: {} instances\n",
            sacred_activations.len()
        );

        for (seq_idx, &sacred_pos) in &sacred_activations {
            if let Some(guide) = matrix.sacred_guides.get(&sacred_pos) {
                println!("Sacred Guide at Position {}:", sacred_pos);
                println!("  Sequence Index: {}", seq_idx);
                println!("  Divine Properties:");
                for prop in &guide.divine_properties {
                    println!("    * {}", prop);
                }
                println!("  Geometric Significance: {}", guide.geometric_significance);
                println!("  Intersection Points: {}", guide.intersection_points.len());
                for intersection in &guide.intersection_points {
                    println!(
                        "    => Connects to Position {}: {} (value: {:.2})",
                        intersection.with_node,
                        intersection.significance,
                        intersection.computational_value
                    );
                }
                println!();
            }
        }
    } else {
        println!("No sacred geometry activations in this seed sequence.\n");
    }

    // ============================================================================
    // PHASE 9: REVERSE REASONING SUMMARY
    // ============================================================================
    println!("PHASE 9: REVERSE REASONING SUMMARY");
    println!("{}\n", "=".repeat(90));

    println!("COMPLETE REASONING CHAIN:");
    println!("{}", "-".repeat(90));
    println!("  1. SEED INPUT: {}", seed_number);
    println!("  2. FLUX TRANSFORMATION: {:?}", flux_sequence);
    println!(
        "  3. POSITION MAPPING: {} positions activated",
        flux_sequence.len()
    );
    println!("  4. SEMANTIC EXTRACTION: Subject 'Physics'");
    println!(
        "  5. INFERENCE OUTPUT: {} meanings generated",
        result.inferred_meanings.len()
    );
    println!(
        "  6. CONFIDENCE SCORE: {:.2}%",
        result.confidence_score * 100.0
    );
    println!();

    println!("PATTERN CHARACTERISTICS:");
    println!("{}", "-".repeat(90));

    // Analyze the sequence
    let unique_values: std::collections::HashSet<_> = flux_sequence.iter().collect();
    let sacred_count = flux_sequence
        .iter()
        .filter(|&&v| [3, 6, 9].contains(&v))
        .count();
    let regular_count = flux_sequence.len() - sacred_count;

    println!("  Unique Flux Values: {}", unique_values.len());
    println!(
        "  Sacred Activations: {} ({:.1}%)",
        sacred_count,
        (sacred_count as f32 / flux_sequence.len() as f32) * 100.0
    );
    println!(
        "  Regular Activations: {} ({:.1}%)",
        regular_count,
        (regular_count as f32 / flux_sequence.len() as f32) * 100.0
    );
    println!();

    println!("{}", "=".repeat(90));
    println!("REVERSE REASONING TEST COMPLETE");
    println!("{}\n", "=".repeat(90));

    // Assertions
    assert_eq!(
        flux_sequence.len(),
        11,
    );
    assert!(
        !result.matched_matrices.is_empty(),
        "Should match at least one matrix"
    );
    assert!(result.confidence_score >= 0.0 && result.confidence_score <= 1.0);
}
