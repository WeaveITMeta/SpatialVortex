//! Common test utilities and fixtures for SpatialVortex test suite
//!
//! This module provides shared test helpers used across multiple test binaries.
//! The `#[allow(dead_code)]` attributes are necessary because Rust compiles
//! each test file as a separate binary, so functions used in one test but not
//! another will trigger false "never used" warnings.

use chrono::Utc;
use spatial_vortex::{flux_matrix::FluxMatrixEngine, models::*};
use std::collections::HashMap;

/// Create a test flux matrix for testing purposes with semantic associations
#[allow(dead_code)]
pub fn create_test_matrix(subject: &str) -> FluxMatrix {
    let engine = FluxMatrixEngine::new();
    let mut matrix = engine.create_matrix(subject.to_string()).unwrap();

    // Add meaningful semantic associations to make tests work
    let keywords = extract_keywords(subject);
    for (_position, node) in matrix.nodes.iter_mut() {
        // Add positive associations based on subject keywords
        for (i, keyword) in keywords.iter().enumerate() {
            let assoc = create_test_association(keyword, (i + 1) as i16, 0.8);
            node.semantic_index.positive_associations.push(assoc);
        }

        // Add general associations for common test patterns
        node.semantic_index
            .positive_associations
            .push(create_test_association("intelligence", 1, 0.7));
        node.semantic_index
            .positive_associations
            .push(create_test_association("reasoning", 2, 0.7));
    }

    matrix
}

/// Extract keywords from subject for semantic associations
#[allow(dead_code)]
fn extract_keywords(subject: &str) -> Vec<String> {
    subject
        .to_lowercase()
        .split_whitespace()
        .map(|s| s.to_string())
        .collect()
}

/// Create test seed input
#[allow(dead_code)]
pub fn create_test_seed_input(seeds: Vec<u64>) -> SeedInput {
    SeedInput {
        seed_numbers: seeds,
        subject_filter: SubjectFilter::All,
        processing_options: ProcessingOptions {
            include_synonyms: true,
            include_antonyms: true,
            max_depth: 5,
            confidence_threshold: 0.3,
            use_sacred_guides: true,
        },
    }
}

/// Create a test semantic association for building controlled test matrices
///
/// This helper enables precise testing of the inference path:
/// seed numbers -> flux positions -> semantic associations -> inferred meanings
#[allow(dead_code)]
pub fn create_test_association(word: &str, index: i16, confidence: f32) -> SemanticAssociation {
    SemanticAssociation {
        word: word.to_string(),
        index,
        confidence: confidence as f64,
        attributes: HashMap::new(),
    }
}

/// Assert that a flux matrix is valid
#[allow(dead_code)]
pub fn assert_valid_matrix(matrix: &FluxMatrix) {
    let engine = FluxMatrixEngine::new();
    assert!(
        engine.validate_matrix(matrix).is_ok(),
        "Matrix should be valid"
    );

    // Check that we have exactly 10 positions covered (0-9)
    let total_positions = matrix.nodes.len() + matrix.sacred_guides.len();
    assert_eq!(total_positions, 10, "Matrix should cover all 10 positions");

    // Check sacred guides are at correct positions
    for position in &[3, 6, 9] {
        assert!(
            matrix.sacred_guides.contains_key(position),
            "Sacred guide should exist at position {}",
            position
        );
    }
}

/// Create a test learning adjustment
#[allow(dead_code)]
pub fn create_test_learning_adjustment(
    adj_type: AdjustmentType,
    magnitude: f32,
) -> LearningAdjustment {
    LearningAdjustment {
        timestamp: Utc::now(),
        adjustment_type: adj_type,
        magnitude,
        rationale: "test_adjustment".to_string(),
    }
}
