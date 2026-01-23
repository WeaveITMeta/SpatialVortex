use spatial_vortex::{
    compression::{compress_text, ELPChannels},
    flux_matrix::FluxMatrixEngine,
    inference_engine::InferenceEngine,
    models::{InferenceInput, ProcessingOptions, SubjectFilter, SeedInput},
};

/// Load a subject matrix using the engine
fn load_test_matrix(subject: &str) -> spatial_vortex::models::FluxMatrix {
    let engine = FluxMatrixEngine::new();
    engine.create_matrix(subject.to_string()).expect("create_matrix")
}

/// Helper to create test processing options
fn create_test_options() -> ProcessingOptions {
    ProcessingOptions {
        include_synonyms: true,
        include_antonyms: false,
        max_depth: 3,
        confidence_threshold: 0.5,
        use_sacred_guides: true,
    }
}

#[tokio::test]
async fn test_compress_and_infer() {
    let mut engine = InferenceEngine::new();

    // Load a test matrix
    let matrix = load_test_matrix("Consciousness");
    engine.update_subject_matrix(matrix);

    // Create a compression hash for a philosophical query
    let text = "What is consciousness?";
    let elp = ELPChannels::new(8.5, 8.0, 7.0); // High ethos/logos, lower pathos
    let hash = compress_text(text, 1001, 9, elp); // Position 9 = divine

    println!("Compressed '{}' to hash: {}", text, hash.to_hex());
    println!("Position: {}", hash.flux_position());
    println!("ELP: E:{:.1} L:{:.1} P:{:.1}", 
        hash.elp_channels().ethos,
        hash.elp_channels().logos,
        hash.elp_channels().pathos
    );
    println!("Is sacred: {}", hash.is_sacred());

    // Create inference input with compression hash
    let input = InferenceInput {
        compression_hashes: vec![hash.to_hex()],
        seed_numbers: vec![],
        subject_filter: SubjectFilter::Specific("Consciousness".to_string()),
        processing_options: create_test_options(),
    };

    // Process inference
    let result = engine.process_inference(input).await;
    assert!(result.is_ok(), "Inference processing should succeed");

    let inference_result = result.unwrap();
    
    // Verify hash metadata was extracted
    assert!(inference_result.hash_metadata.is_some());
    let metadata = inference_result.hash_metadata.as_ref().unwrap();
    assert_eq!(metadata.len(), 1);
    assert_eq!(metadata[0].flux_position, 9);
    assert!(metadata[0].is_sacred);
    assert_eq!(metadata[0].elp_channels.ethos, 8.0); // Compressed to nibbles

    // Verify inferences were generated
    assert!(!inference_result.inferred_meanings.is_empty());
    println!("Generated {} inferences", inference_result.inferred_meanings.len());

    // Check that sacred position boosted confidence
    let avg_relevance: f32 = inference_result.inferred_meanings.iter()
        .map(|i| i.contextual_relevance)
        .sum::<f32>() / inference_result.inferred_meanings.len() as f32;
    println!("Average relevance (with sacred boost): {:.2}", avg_relevance);
}

#[tokio::test]
async fn test_multiple_hashes() {
    let mut engine = InferenceEngine::new();

    // Load matrices
    let matrix1 = load_test_matrix("AI Ethics");
    let matrix2 = load_test_matrix("Machine Learning");
    engine.update_subject_matrix(matrix1);
    engine.update_subject_matrix(matrix2);

    // Create multiple hashes with different characteristics
    let hash1 = compress_text(
        "What is ethical AI?",
        1001,
        9,  // Position 9 = divine
        ELPChannels::new(9.0, 7.0, 5.0), // High ethos
    );

    let hash2 = compress_text(
        "How does machine learning work?",
        1002,
        3,  // Position 3 = creative
        ELPChannels::new(5.0, 9.0, 4.0), // High logos
    );

    let hash3 = compress_text(
        "I feel inspired by AI",
        1003,
        6,  // Position 6 = sacred
        ELPChannels::new(4.0, 5.0, 9.0), // High pathos
    );

    let input = InferenceInput {
        compression_hashes: vec![hash1.to_hex(), hash2.to_hex(), hash3.to_hex()],
        seed_numbers: vec![],
        subject_filter: SubjectFilter::All,
        processing_options: create_test_options(),
    };

    let result = engine.process_inference(input).await.unwrap();

    // Verify all three hashes were processed
    assert_eq!(result.hash_metadata.as_ref().unwrap().len(), 3);

    // Check positions
    let positions: Vec<u8> = result.hash_metadata.as_ref().unwrap()
        .iter()
        .map(|m| m.flux_position)
        .collect();
    assert_eq!(positions, vec![9, 3, 6]);

    // Check dominant channels
    let metadata = result.hash_metadata.as_ref().unwrap();
    assert!(metadata[0].elp_channels.ethos > 7.0); // Hash 1: High ethos
    assert!(metadata[1].elp_channels.logos > 7.0); // Hash 2: High logos
    assert!(metadata[2].elp_channels.pathos > 7.0); // Hash 3: High pathos

    println!("Processed {} hashes successfully", positions.len());
}

#[tokio::test]
async fn test_sacred_position_boost() {
    let mut engine = InferenceEngine::new();
    let matrix = load_test_matrix("Spirituality");
    engine.update_subject_matrix(matrix);

    // Create hash at sacred position
    let sacred_hash = compress_text(
        "Divine transcendence",
        1001,
        9,  // Position 9 = divine (sacred)
        ELPChannels::new(9.0, 9.0, 9.0),
    );

    // Create hash at non-sacred position
    let normal_hash = compress_text(
        "Regular thought",
        1002,
        5,  // Position 5 = not sacred
        ELPChannels::new(5.0, 5.0, 5.0),
    );

    // Process sacred hash
    let sacred_input = InferenceInput {
        compression_hashes: vec![sacred_hash.to_hex()],
        seed_numbers: vec![],
        subject_filter: SubjectFilter::Specific("Spirituality".to_string()),
        processing_options: create_test_options(),
    };
    let sacred_result = engine.process_inference(sacred_input).await.unwrap();

    // Process normal hash
    let normal_input = InferenceInput {
        compression_hashes: vec![normal_hash.to_hex()],
        seed_numbers: vec![],
        subject_filter: SubjectFilter::Specific("Spirituality".to_string()),
        processing_options: create_test_options(),
    };
    let normal_result = engine.process_inference(normal_input).await.unwrap();

    // Sacred position should have higher average relevance
    let sacred_avg = sacred_result.inferred_meanings.iter()
        .map(|i| i.contextual_relevance)
        .sum::<f32>() / sacred_result.inferred_meanings.len() as f32;

    let normal_avg = normal_result.inferred_meanings.iter()
        .map(|i| i.contextual_relevance)
        .sum::<f32>() / normal_result.inferred_meanings.len() as f32;

    println!("Sacred position (9) average relevance: {:.3}", sacred_avg);
    println!("Normal position (5) average relevance: {:.3}", normal_avg);

    assert!(sacred_avg > normal_avg, 
        "Sacred position should boost relevance: {} vs {}", sacred_avg, normal_avg);
}

#[tokio::test]
async fn test_backward_compatibility() {
    let mut engine = InferenceEngine::new();
    let matrix = load_test_matrix("Legacy Test");
    engine.update_subject_matrix(matrix);

    // Test old SeedInput still works
    #[allow(deprecated)]
    let seed_input = SeedInput {
        seed_numbers: vec![888, 872],
        subject_filter: SubjectFilter::Specific("Legacy Test".to_string()),
        processing_options: create_test_options(),
    };

    #[allow(deprecated)]
    let result = engine.process_seed_input(seed_input).await;
    assert!(result.is_ok(), "Legacy seed input should still work");

    let inference_result = result.unwrap();
    assert!(inference_result.hash_metadata.is_none()); // No hash metadata for legacy
    assert!(!inference_result.inferred_meanings.is_empty());
}

#[tokio::test]
async fn test_invalid_hash_handling() {
    let mut engine = InferenceEngine::new();
    let matrix = load_test_matrix("Error Test");
    engine.update_subject_matrix(matrix);

    // Create input with one valid and one invalid hash
    let valid_hash = compress_text("Valid", 1001, 5, ELPChannels::new(5.0, 5.0, 5.0));
    
    let input = InferenceInput {
        compression_hashes: vec![
            valid_hash.to_hex(),
            "invalid_hex_string".to_string(), // Invalid
            "toolong123456789012345678901234567890".to_string(), // Too long
        ],
        seed_numbers: vec![],
        subject_filter: SubjectFilter::Specific("Error Test".to_string()),
        processing_options: create_test_options(),
    };

    let result = engine.process_inference(input).await;
    assert!(result.is_ok(), "Should process valid hashes and skip invalid ones");

    let inference_result = result.unwrap();
    // Should only have metadata for the valid hash
    assert_eq!(inference_result.hash_metadata.as_ref().unwrap().len(), 1);
}

#[tokio::test]
async fn test_hash_to_rgb_color() {
    // Test ELP channel to RGB conversion
    let hash = compress_text(
        "Test",
        1001,
        5,
        ELPChannels::new(9.0, 5.0, 3.0), // High E, medium L, low P
    );

    let (r, g, b) = hash.rgb_color();
    println!("RGB from ELP(9,5,3): R:{} G:{} B:{}", r, g, b);

    // Blue (ethos) should be highest
    assert!(b > g && b > r, "Blue (ethos) should be highest");

    // High ethos should give strong blue
    assert!(b > 200, "High ethos should produce strong blue: {}", b);
}
