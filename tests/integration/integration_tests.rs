// Integration tests for the complete SpatialVortex system
mod common;

use common::*;
use spatial_vortex::{flux_matrix::FluxMatrixEngine, inference_engine::InferenceEngine, models::*};

#[tokio::test]
async fn test_end_to_end_inference_flow() {
    // Create flux engine
    let flux_engine = FluxMatrixEngine::new();

    // Create inference engine
    let mut inference_engine = InferenceEngine::new();

    // Generate a matrix
    let matrix = flux_engine
        .create_matrix("End-to-End Test".to_string())
        .unwrap();
    assert_valid_matrix(&matrix);

    // Load matrix into inference engine
    inference_engine.update_subject_matrix(matrix);

    // Process seed input
    let seed_input = create_test_seed_input(vec![888, 872]);
    let result = inference_engine
        .process_seed_input(seed_input)
        .await
        .unwrap();

    // Verify results
    assert!(!result.inferred_meanings.is_empty());
    assert!(result.confidence_score >= 0.0);
    assert!(result.processing_time_ms > 0);
}

#[tokio::test]
async fn test_multiple_subject_matrices() {
    let flux_engine = FluxMatrixEngine::new();
    let mut inference_engine = InferenceEngine::new();

    // Create multiple subject matrices
    let subjects = vec!["Mathematics", "Physics", "Computer Science", "Philosophy"];

    for subject in subjects {
        let matrix = flux_engine.create_matrix(subject.to_string()).unwrap();
        inference_engine.update_subject_matrix(matrix);
    }

    // Process with filter for all subjects
    let seed_input = SeedInput {
        seed_numbers: vec![123],
        subject_filter: SubjectFilter::All,
        processing_options: ProcessingOptions {
            include_synonyms: true,
            include_antonyms: true,
            max_depth: 5,
            confidence_threshold: 0.3,
            use_sacred_guides: true,
        },
    };

    let result = inference_engine
        .process_seed_input(seed_input)
        .await
        .unwrap();
    assert_eq!(result.matched_matrices.len(), 4);
}

#[tokio::test]
async fn test_sacred_geometry_integration() {
    let flux_engine = FluxMatrixEngine::new();
    let matrix = flux_engine
        .create_matrix("Sacred Geometry Test".to_string())
        .unwrap();

    // Verify sacred guides exist
    assert_eq!(matrix.sacred_guides.len(), 3);

    // Verify sacred guide positions
    assert!(matrix.sacred_guides.contains_key(&3));
    assert!(matrix.sacred_guides.contains_key(&6));
    assert!(matrix.sacred_guides.contains_key(&9));

    // Verify sacred guides have intersection points
    for guide in matrix.sacred_guides.values() {
        assert!(!guide.intersection_points.is_empty());
        assert!(!guide.divine_properties.is_empty());
    }
}

#[tokio::test]
async fn test_flux_pattern_consistency() {
    let flux_engine = FluxMatrixEngine::new();

    // Test the core flux pattern mapping to positions
    // Position 0 = 0 (void), then regular nodes, skipping sacred positions 3, 6, 9
    let test_cases = vec![
        (0, 0), // Void
        (1, 1), // pattern[0]
        (2, 2), // pattern[1]
        (3, 0), // Sacred guide (no flux value)
        (4, 4), // pattern[2]
        (5, 8), // pattern[3]
        (6, 0), // Sacred guide (no flux value)
        (7, 7), // pattern[4]
        (8, 5), // pattern[5]
        (9, 0), // Sacred guide (no flux value)
    ];

    for (position, expected_value) in test_cases {
        let actual_value = flux_engine.get_flux_value_at_position(position);
        assert_eq!(
            actual_value, expected_value,
            "Flux value at position {} should be {}",
            position, expected_value
        );
    }
}

#[tokio::test]
async fn test_seed_number_transformation() {
    let flux_engine = FluxMatrixEngine::new();

    // Test various seed numbers
    let seeds = vec![1, 10, 100, 888, 872, 999];

    for seed in seeds {
        let sequence = flux_engine.seed_to_flux_sequence(seed);

        // All sequences should have 9 elements
        assert_eq!(sequence.len(), 9);

        // All values should be single digits
        for &value in &sequence {
            assert!(value <= 9, "Seed {}: value {} should be ≤ 9", seed, value);
        }
    }
}

#[tokio::test]
async fn test_moral_alignment_system() {
    let flux_engine = FluxMatrixEngine::new();
    let mut inference_engine = InferenceEngine::new();

    let matrix = flux_engine
        .create_matrix("Moral Alignment Test".to_string())
        .unwrap();
    inference_engine.update_subject_matrix(matrix);

    let seed_input = create_test_seed_input(vec![500]);
    let result = inference_engine
        .process_seed_input(seed_input)
        .await
        .unwrap();

    // Count moral alignments
    let mut has_constructive = false;
    let mut has_destructive = false;
    let mut has_neutral = false;

    for inference in &result.inferred_meanings {
        match &inference.moral_alignment {
            MoralAlignment::Constructive(_) => has_constructive = true,
            MoralAlignment::Destructive(_) => has_destructive = true,
            MoralAlignment::Neutral => has_neutral = true,
        }
    }

    // At least one type of moral alignment should exist
    assert!(has_constructive || has_destructive || has_neutral);
}

#[tokio::test]
async fn test_learning_adjustments() {
    let flux_engine = FluxMatrixEngine::new();
    let mut matrix = flux_engine
        .create_matrix("Learning Test".to_string())
        .unwrap();

    // Create various learning adjustments
    let adjustments = vec![
        create_test_learning_adjustment(AdjustmentType::ConfidenceBoost, 0.1),
        create_test_learning_adjustment(AdjustmentType::SemanticRefinement, 0.2),
        create_test_learning_adjustment(AdjustmentType::ConnectionStrengthening, 0.15),
    ];

    for adjustment in adjustments {
        let result = flux_engine.update_matrix_with_rl(&mut matrix, adjustment);
        assert!(result.is_ok());
    }

    // Verify adjustments were applied
    for node in matrix.nodes.values() {
        assert_eq!(node.attributes.dynamics.learning_adjustments.len(), 3);
    }
}

#[tokio::test]
async fn test_bidirectional_inference() {
    let flux_engine = FluxMatrixEngine::new();
    let mut inference_engine = InferenceEngine::new();

    let matrix = flux_engine
        .create_matrix("Bidirectional Test".to_string())
        .unwrap();
    inference_engine.update_subject_matrix(matrix);

    // Reverse inference: seed -> meanings
    let seed_input = create_test_seed_input(vec![100]);
    let reverse_result = inference_engine
        .process_seed_input(seed_input)
        .await
        .unwrap();

    assert!(!reverse_result.inferred_meanings.is_empty());

    // Forward inference: meanings -> seeds
    let target_meanings = vec!["test".to_string()];
    let forward_result = inference_engine
        .forward_inference(target_meanings, &SubjectFilter::All)
        .await;

    assert!(forward_result.is_ok());
}

#[tokio::test]
async fn test_confidence_threshold_filtering() {
    let flux_engine = FluxMatrixEngine::new();
    let mut inference_engine = InferenceEngine::new();

    let matrix = flux_engine
        .create_matrix("Threshold Test".to_string())
        .unwrap();
    inference_engine.update_subject_matrix(matrix);

    // Test with high confidence threshold
    let high_threshold_input = SeedInput {
        seed_numbers: vec![250],
        subject_filter: SubjectFilter::All,
        processing_options: ProcessingOptions {
            include_synonyms: true,
            include_antonyms: true,
            max_depth: 5,
            confidence_threshold: 0.9, // High threshold
            use_sacred_guides: true,
        },
    };

    let high_result = inference_engine
        .process_seed_input(high_threshold_input)
        .await
        .unwrap();

    // Test with low confidence threshold
    let low_threshold_input = SeedInput {
        seed_numbers: vec![250],
        subject_filter: SubjectFilter::All,
        processing_options: ProcessingOptions {
            include_synonyms: true,
            include_antonyms: true,
            max_depth: 5,
            confidence_threshold: 0.1, // Low threshold
            use_sacred_guides: true,
        },
    };

    let low_result = inference_engine
        .process_seed_input(low_threshold_input)
        .await
        .unwrap();

    // Lower threshold should potentially include more associations
    // (though in test data they might be equal)
    assert!(low_result.inferred_meanings.len() >= high_result.inferred_meanings.len());
}

#[tokio::test]
async fn test_matrix_versioning() {
    let flux_engine = FluxMatrixEngine::new();

    // Create multiple versions of the same subject
    let v1 = flux_engine
        .create_matrix("Versioned Subject".to_string())
        .unwrap();
    let v2 = flux_engine
        .create_matrix("Versioned Subject".to_string())
        .unwrap();

    // They should have different IDs but same subject
    assert_ne!(v1.id, v2.id);
    assert_eq!(v1.subject, v2.subject);
}
