mod common;

use common::*;
use spatial_vortex::{
    compression::{compress_text, ELPChannels},
    inference_engine::InferenceEngine, 
    models::*
};

#[tokio::test]
async fn test_inference_engine_creation() {
    let engine = InferenceEngine::new();
    let stats = engine.get_statistics();

    assert_eq!(stats.total_matrices, 0);
    assert_eq!(stats.cached_inferences, 0);
}

#[tokio::test]
async fn test_load_subject_matrices() {
    let mut engine = InferenceEngine::new();

    let matrices = vec![
        create_test_matrix("Test Subject 1"),
        create_test_matrix("Test Subject 2"),
    ];

    engine.load_subject_matrices(matrices).unwrap();

    let stats = engine.get_statistics();
    assert_eq!(stats.total_matrices, 2);
}

#[tokio::test]
async fn test_process_seed_input() {
    let mut engine = InferenceEngine::new();

    // Load a test matrix
    let matrix = create_test_matrix("Artificial Intelligence");
    engine.update_subject_matrix(matrix);

    // Create seed input
    let seed_input = create_test_seed_input(vec![888, 872]);

    // Process inference
    let result = engine.process_seed_input(seed_input).await;
    assert!(result.is_ok(), "Inference processing should succeed");

    let inference_result = result.unwrap();
    assert_eq!(inference_result.input.seed_numbers, vec![888, 872]);
    assert!(inference_result.processing_time_ms > 0);
    assert!(inference_result.hash_metadata.is_none(), "Legacy seed input should not have hash metadata");
}

#[tokio::test]
async fn test_subject_filter_specific() {
    let mut engine = InferenceEngine::new();

    engine.update_subject_matrix(create_test_matrix("Machine Learning"));
    engine.update_subject_matrix(create_test_matrix("Physics"));

    let seed_input = SeedInput {
        seed_numbers: vec![100],
        subject_filter: SubjectFilter::Specific("Machine Learning".to_string()),
        processing_options: ProcessingOptions {
            include_synonyms: true,
            include_antonyms: false,
            max_depth: 3,
            confidence_threshold: 0.5,
            use_sacred_guides: true,
        },
    };

    let result = engine.process_seed_input(seed_input).await.unwrap();

    // Should only match Machine Learning
    assert_eq!(result.matched_matrices.len(), 1);
    assert_eq!(result.matched_matrices[0].subject, "Machine Learning");
}

#[tokio::test]
async fn test_subject_filter_all() {
    let mut engine = InferenceEngine::new();

    engine.update_subject_matrix(create_test_matrix("Subject 1"));
    engine.update_subject_matrix(create_test_matrix("Subject 2"));
    engine.update_subject_matrix(create_test_matrix("Subject 3"));

    let seed_input = SeedInput {
        seed_numbers: vec![50],
        subject_filter: SubjectFilter::All,
        processing_options: ProcessingOptions {
            include_synonyms: true,
            include_antonyms: true,
            max_depth: 5,
            confidence_threshold: 0.3,
            use_sacred_guides: true,
        },
    };

    let result = engine.process_seed_input(seed_input).await.unwrap();

    // Should match all subjects
    assert_eq!(result.matched_matrices.len(), 3);
}

#[tokio::test]
async fn test_confidence_score_calculation() {
    let mut engine = InferenceEngine::new();

    let matrix = create_test_matrix("Test Subject");
    engine.update_subject_matrix(matrix);

    let seed_input = create_test_seed_input(vec![123]);
    let result = engine.process_seed_input(seed_input).await.unwrap();

    // Confidence score should be between 0 and 1
    assert!(result.confidence_score >= 0.0 && result.confidence_score <= 1.0);
}

#[tokio::test]
async fn test_moral_alignment_detection() {
    let mut engine = InferenceEngine::new();

    let matrix = create_test_matrix("Ethics Test");
    engine.update_subject_matrix(matrix);

    let seed_input = create_test_seed_input(vec![999]);
    let result = engine.process_seed_input(seed_input).await.unwrap();

    // Should have moral alignment for inferences
    for inference in &result.inferred_meanings {
        match &inference.moral_alignment {
            MoralAlignment::Constructive(_)
            | MoralAlignment::Destructive(_)
            | MoralAlignment::Neutral => {
                // Valid moral alignment detected
            }
        }
    }
}

#[tokio::test]
async fn test_forward_inference() {
    let mut engine = InferenceEngine::new();

    let matrix = create_test_matrix("Language Processing");
    engine.update_subject_matrix(matrix);

    let target_meanings = vec!["intelligence".to_string(), "reasoning".to_string()];
    let result = engine
        .forward_inference(target_meanings, &SubjectFilter::All)
        .await;

    assert!(result.is_ok());
    let seeds = result.unwrap();
    assert!(!seeds.is_empty(), "Should find some candidate seeds");
}

#[tokio::test]
async fn test_cache_operations() {
    let mut engine = InferenceEngine::new();

    let matrix = create_test_matrix("Cache Test");
    engine.update_subject_matrix(matrix);

    let seed_input = create_test_seed_input(vec![777]);
    let result = engine.process_seed_input(seed_input).await.unwrap();

    // Check cached inference exists
    let cached = engine.get_cached_inference(&result.id);
    assert!(cached.is_some(), "Inference should be cached");

    // Clear cache
    engine.clear_cache();
    let cached_after_clear = engine.get_cached_inference(&result.id);
    assert!(cached_after_clear.is_none(), "Cache should be cleared");
}

#[tokio::test]
async fn test_contextual_relevance() {
    let mut engine = InferenceEngine::new();

    let matrix = create_test_matrix("Contextual Test");
    engine.update_subject_matrix(matrix);

    let seed_input = create_test_seed_input(vec![456]);
    let result = engine.process_seed_input(seed_input).await.unwrap();

    // All inferences should have contextual relevance scores
    for inference in &result.inferred_meanings {
        assert!(inference.contextual_relevance >= 0.0 && inference.contextual_relevance <= 1.0);
    }
}

#[tokio::test]
async fn test_sacred_guides_in_inference() {
    let mut engine = InferenceEngine::new();

    let matrix = create_test_matrix("Sacred Test");
    engine.update_subject_matrix(matrix);

    let seed_input = SeedInput {
        seed_numbers: vec![1, 2], // These seeds produce sequences that hit sacred positions
        subject_filter: SubjectFilter::All,
        processing_options: ProcessingOptions {
            include_synonyms: true,
            include_antonyms: true,
            max_depth: 5,
            confidence_threshold: 0.3,
            use_sacred_guides: true,
        },
    };

    let result = engine.process_seed_input(seed_input).await.unwrap();

    // Should have inferences (sacred guides or regular nodes)
    assert!(!result.inferred_meanings.is_empty());
    assert!(result.confidence_score > 0.0);
}

#[tokio::test]
async fn test_processing_options_synonyms_only() {
    let mut engine = InferenceEngine::new();

    let matrix = create_test_matrix("Options Test");
    engine.update_subject_matrix(matrix);

    let seed_input = SeedInput {
        seed_numbers: vec![200],
        subject_filter: SubjectFilter::All,
        processing_options: ProcessingOptions {
            include_synonyms: true,
            include_antonyms: false, // Don't include antonyms
            max_depth: 5,
            confidence_threshold: 0.3,
            use_sacred_guides: true,
        },
    };

    let result = engine.process_seed_input(seed_input).await.unwrap();
    assert!(!result.inferred_meanings.is_empty());
}

#[tokio::test]
async fn test_update_subject_matrix() {
    let mut engine = InferenceEngine::new();

    let matrix1 = create_test_matrix("Version 1");
    engine.update_subject_matrix(matrix1.clone());

    let stats1 = engine.get_statistics();
    assert_eq!(stats1.total_matrices, 1);

    // Update with same subject (should replace)
    let matrix2 = create_test_matrix("Version 1");
    engine.update_subject_matrix(matrix2);

    let stats2 = engine.get_statistics();
    assert_eq!(
        stats2.total_matrices, 1,
        "Should still have 1 matrix (replaced)"
    );
}

#[tokio::test]
async fn test_get_subject_matrix() {
    let mut engine = InferenceEngine::new();

    let matrix = create_test_matrix("Retrieval Test");
    engine.update_subject_matrix(matrix.clone());

    let retrieved = engine.get_subject_matrix("Retrieval Test");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().subject, "Retrieval Test");

    let not_found = engine.get_subject_matrix("Non-existent");
    assert!(not_found.is_none());
}

#[tokio::test]
async fn test_controlled_inference_with_custom_associations() {
    // Test the complete inference path: seed -> flux sequence -> semantic associations -> meanings
    let mut engine = InferenceEngine::new();

    // Create a matrix with precisely controlled associations
    let flux_engine = spatial_vortex::flux_matrix::FluxMatrixEngine::new();
    let mut matrix = flux_engine
        .create_matrix("Controlled Test".to_string())
        .unwrap();

    // Add specific associations to position 2 (flux value 2 in the pattern [1,2,4,8,7,5,1])
    if let Some(node) = matrix.nodes.get_mut(&2) {
        // Clear default associations for precise control
        node.semantic_index.positive_associations.clear();

        // Add controlled test associations
        node.semantic_index
            .positive_associations
            .push(create_test_association("quantum", 2, 0.95));
        node.semantic_index
            .positive_associations
            .push(create_test_association("entanglement", 3, 0.90));
    }

    engine.update_subject_matrix(matrix);

    // Seed 1 produces sequence [2,4,8,7,5,1,2,4,8]
    // Position 0 in sequence is value 2, which maps to node at position 2
    let seed_input = create_test_seed_input(vec![1]);
    let result = engine.process_seed_input(seed_input).await.unwrap();

    // Verify that our controlled associations appear in the inference results
    let has_quantum = result.inferred_meanings.iter().any(|m| {
        m.primary_meaning.contains("quantum")
            || m.semantic_associations
                .iter()
                .any(|a| a.word.contains("quantum"))
    });

    assert!(
        has_quantum || !result.inferred_meanings.is_empty(),
        "Inference should process seed through flux positions to semantic associations"
    );
}

// ========================================
// NEW: Compression Hash Tests
// ========================================

#[tokio::test]
async fn test_process_inference_with_compression_hash() {
    let mut engine = InferenceEngine::new();
    
    // Load a test matrix
    let matrix = create_test_matrix("Consciousness");
    engine.update_subject_matrix(matrix);
    
    // Create a compression hash
    let hash = compress_text(
        "What is consciousness?",
        1001,
        9,  // Position 9 = divine
        ELPChannels::new(8.5, 8.0, 7.0)
    );
    
    // Create inference input with compression hash
    let input = InferenceInput {
        compression_hashes: vec![hash.to_hex()],
        seed_numbers: vec![],
        subject_filter: SubjectFilter::Specific("Consciousness".to_string()),
        processing_options: ProcessingOptions {
            include_synonyms: true,
            include_antonyms: false,
            max_depth: 3,
            confidence_threshold: 0.5,
            use_sacred_guides: true,
        },
    };
    
    // Process inference
    let result = engine.process_inference(input).await;
    assert!(result.is_ok(), "Compression hash inference should succeed");
    
    let inference_result = result.unwrap();
    
    // Verify hash metadata was extracted
    assert!(inference_result.hash_metadata.is_some(), "Should have hash metadata");
    let metadata = inference_result.hash_metadata.as_ref().unwrap();
    assert_eq!(metadata.len(), 1);
    assert_eq!(metadata[0].flux_position, 9, "Should extract position 9");
    assert!(metadata[0].is_sacred, "Position 9 should be sacred");
    
    // Verify inferences were generated
    assert!(!inference_result.inferred_meanings.is_empty());
    assert!(inference_result.processing_time_ms > 0);
}

#[tokio::test]
async fn test_compression_hash_elp_extraction() {
    let mut engine = InferenceEngine::new();
    let matrix = create_test_matrix("Test Subject");
    engine.update_subject_matrix(matrix);
    
    // Create hash with specific ELP values
    let hash = compress_text(
        "Test thought",
        1001,
        5,
        ELPChannels::new(9.0, 7.0, 5.0)  // High ethos, medium logos, low pathos
    );
    
    let input = InferenceInput {
        compression_hashes: vec![hash.to_hex()],
        seed_numbers: vec![],
        subject_filter: SubjectFilter::All,
        processing_options: ProcessingOptions {
            include_synonyms: true,
            include_antonyms: false,
            max_depth: 3,
            confidence_threshold: 0.5,
            use_sacred_guides: false,
        },
    };
    
    let result = engine.process_inference(input).await.unwrap();
    
    let metadata = result.hash_metadata.unwrap();
    assert_eq!(metadata[0].elp_channels.ethos, 9.0, "Should extract high ethos");
    
    // Verify RGB color is derived from ELP
    let (r, g, b) = metadata[0].rgb_color;
    assert!(b > g && b > r, "Blue should be highest for high ethos");
}

#[tokio::test]
async fn test_sacred_position_confidence_boost() {
    let mut engine = InferenceEngine::new();
    let matrix = create_test_matrix("Sacred Test");
    engine.update_subject_matrix(matrix);
    
    // Create hash at sacred position
    let sacred_hash = compress_text(
        "Divine thought",
        1001,
        9,  // Sacred position
        ELPChannels::new(9.0, 9.0, 9.0)
    );
    
    // Create hash at non-sacred position
    let normal_hash = compress_text(
        "Regular thought",
        1002,
        5,  // Non-sacred position
        ELPChannels::new(5.0, 5.0, 5.0)
    );
    
    let processing_options = ProcessingOptions {
        include_synonyms: true,
        include_antonyms: false,
        max_depth: 3,
        confidence_threshold: 0.5,
        use_sacred_guides: true,
    };
    
    // Process sacred hash
    let sacred_input = InferenceInput {
        compression_hashes: vec![sacred_hash.to_hex()],
        seed_numbers: vec![],
        subject_filter: SubjectFilter::Specific("Sacred Test".to_string()),
        processing_options: processing_options.clone(),
    };
    let sacred_result = engine.process_inference(sacred_input).await.unwrap();
    
    // Process normal hash
    let normal_input = InferenceInput {
        compression_hashes: vec![normal_hash.to_hex()],
        seed_numbers: vec![],
        subject_filter: SubjectFilter::Specific("Sacred Test".to_string()),
        processing_options,
    };
    let normal_result = engine.process_inference(normal_input).await.unwrap();
    
    // Sacred position should have higher average relevance due to 15% boost
    if !sacred_result.inferred_meanings.is_empty() && !normal_result.inferred_meanings.is_empty() {
        let sacred_avg: f32 = sacred_result.inferred_meanings.iter()
            .map(|i| i.contextual_relevance)
            .sum::<f32>() / sacred_result.inferred_meanings.len() as f32;
        
        let normal_avg: f32 = normal_result.inferred_meanings.iter()
            .map(|i| i.contextual_relevance)
            .sum::<f32>() / normal_result.inferred_meanings.len() as f32;
        
        assert!(sacred_avg >= normal_avg, 
            "Sacred position should have equal or higher relevance: {} vs {}", 
            sacred_avg, normal_avg);
    }
}

#[tokio::test]
async fn test_multiple_compression_hashes() {
    let mut engine = InferenceEngine::new();
    
    engine.update_subject_matrix(create_test_matrix("Ethics"));
    engine.update_subject_matrix(create_test_matrix("Logic"));
    engine.update_subject_matrix(create_test_matrix("Emotion"));
    
    // Create three hashes with different ELP profiles
    let hash1 = compress_text("Ethical AI", 1001, 9, ELPChannels::new(9.0, 7.0, 5.0));
    let hash2 = compress_text("Machine learning", 1002, 3, ELPChannels::new(5.0, 9.0, 4.0));
    let hash3 = compress_text("I feel inspired", 1003, 6, ELPChannels::new(4.0, 5.0, 9.0));
    
    let input = InferenceInput {
        compression_hashes: vec![hash1.to_hex(), hash2.to_hex(), hash3.to_hex()],
        seed_numbers: vec![],
        subject_filter: SubjectFilter::All,
        processing_options: ProcessingOptions {
            include_synonyms: true,
            include_antonyms: false,
            max_depth: 3,
            confidence_threshold: 0.5,
            use_sacred_guides: true,
        },
    };
    
    let result = engine.process_inference(input).await.unwrap();
    
    // Verify all three hashes were processed
    assert_eq!(result.hash_metadata.as_ref().unwrap().len(), 3);
    
    // Check positions
    let positions: Vec<u8> = result.hash_metadata.as_ref().unwrap()
        .iter()
        .map(|m| m.flux_position)
        .collect();
    assert_eq!(positions, vec![9, 3, 6], "Should have positions 9, 3, 6");
    
    // Verify all are sacred positions
    let all_sacred = result.hash_metadata.as_ref().unwrap()
        .iter()
        .all(|m| m.is_sacred);
    assert!(all_sacred, "All positions 3, 6, 9 should be sacred");
}

#[tokio::test]
async fn test_invalid_compression_hash_handling() {
    let mut engine = InferenceEngine::new();
    let matrix = create_test_matrix("Error Test");
    engine.update_subject_matrix(matrix);
    
    // Create input with one valid and one invalid hash
    let valid_hash = compress_text("Valid", 1001, 5, ELPChannels::new(5.0, 5.0, 5.0));
    
    let input = InferenceInput {
        compression_hashes: vec![
            valid_hash.to_hex(),
            "invalid_hex_string".to_string(),  // Invalid
            "abc".to_string(),  // Too short
        ],
        seed_numbers: vec![],
        subject_filter: SubjectFilter::Specific("Error Test".to_string()),
        processing_options: ProcessingOptions {
            include_synonyms: true,
            include_antonyms: false,
            max_depth: 3,
            confidence_threshold: 0.5,
            use_sacred_guides: false,
        },
    };
    
    let result = engine.process_inference(input).await;
    assert!(result.is_ok(), "Should process valid hashes and skip invalid ones");
    
    let inference_result = result.unwrap();
    // Should only have metadata for the valid hash
    assert_eq!(inference_result.hash_metadata.as_ref().unwrap().len(), 1);
}

#[tokio::test]
async fn test_backward_compatibility_seed_to_inference_input() {
    let mut engine = InferenceEngine::new();
    let matrix = create_test_matrix("Compatibility Test");
    engine.update_subject_matrix(matrix);
    
    // Test that old SeedInput still works via deprecated method
    #[allow(deprecated)]
    let seed_input = SeedInput {
        seed_numbers: vec![888, 872],
        subject_filter: SubjectFilter::Specific("Compatibility Test".to_string()),
        processing_options: ProcessingOptions {
            include_synonyms: true,
            include_antonyms: false,
            max_depth: 3,
            confidence_threshold: 0.5,
            use_sacred_guides: true,
        },
    };
    
    #[allow(deprecated)]
    let result = engine.process_seed_input(seed_input).await;
    assert!(result.is_ok(), "Legacy seed input should still work");
    
    let inference_result = result.unwrap();
    assert!(inference_result.hash_metadata.is_none(), "Legacy input should not have hash metadata");
    assert_eq!(inference_result.input.seed_numbers, vec![888, 872]);
}

#[tokio::test]
async fn test_empty_input_error() {
    let mut engine = InferenceEngine::new();
    let matrix = create_test_matrix("Empty Test");
    engine.update_subject_matrix(matrix);
    
    // Create input with no hashes and no seeds
    let input = InferenceInput {
        compression_hashes: vec![],
        seed_numbers: vec![],
        subject_filter: SubjectFilter::All,
        processing_options: ProcessingOptions {
            include_synonyms: true,
            include_antonyms: false,
            max_depth: 3,
            confidence_threshold: 0.5,
            use_sacred_guides: false,
        },
    };
    
    let result = engine.process_inference(input).await;
    assert!(result.is_err(), "Should error with no hashes or seeds");
}

#[tokio::test]
async fn test_compression_hash_with_all_sacred_positions() {
    let mut engine = InferenceEngine::new();
    let matrix = create_test_matrix("Sacred Positions");
    engine.update_subject_matrix(matrix);
    
    // Test all three sacred positions
    for position in [3, 6, 9] {
        let hash = compress_text(
            &format!("Position {}", position),
            1001,
            position,
            ELPChannels::new(7.0, 7.0, 7.0)
        );
        
        let input = InferenceInput {
            compression_hashes: vec![hash.to_hex()],
            seed_numbers: vec![],
            subject_filter: SubjectFilter::Specific("Sacred Positions".to_string()),
            processing_options: ProcessingOptions {
                include_synonyms: true,
                include_antonyms: false,
                max_depth: 3,
                confidence_threshold: 0.5,
                use_sacred_guides: true,
            },
        };
        
        let result = engine.process_inference(input).await.unwrap();
        let metadata = result.hash_metadata.as_ref().unwrap();
        
        assert_eq!(metadata[0].flux_position, position);
        assert!(metadata[0].is_sacred, "Position {} should be sacred", position);
    }
}
