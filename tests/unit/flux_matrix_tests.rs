mod common;

use common::*;
use spatial_vortex::{flux_matrix::FluxMatrixEngine, models::*};

#[test]
fn test_flux_engine_creation() {
    let engine = FluxMatrixEngine::new();
    assert_eq!(engine.base_pattern, [1, 2, 4, 8, 7, 5, 1]);
    assert_eq!(engine.sacred_positions, [3, 6, 9]);
}

#[test]
fn test_create_basic_matrix() {
    let engine = FluxMatrixEngine::new();
    let matrix = engine.create_matrix("Test Subject".to_string()).unwrap();

    assert_eq!(matrix.subject, "Test Subject");
    assert_valid_matrix(&matrix);
}

#[test]
fn test_flux_pattern_values() {
    let engine = FluxMatrixEngine::new();

    // Positions manifest their own values (including sacred positions)
    for pos in 0u8..=9u8 {
        assert_eq!(engine.get_flux_value_at_position(pos), pos);
    }
}

#[test]
fn test_seed_to_flux_sequence() {
    let engine = FluxMatrixEngine::new();

    // Test seed number conversion
    let sequence = engine.seed_to_flux_sequence(1);
    assert_eq!(sequence.len(), 9, "Sequence should have 9 elements");

    // Verify sequence follows doubling and reduction rules
    // 1 -> 2 -> 4 -> 8 -> 16 (1+6=7) -> 14 (1+4=5) -> 10 (1+0=1) -> ...
    assert_eq!(sequence[0], 2); // 1*2=2
    assert_eq!(sequence[1], 4); // 2*2=4
    assert_eq!(sequence[2], 8); // 4*2=8
    assert_eq!(sequence[3], 7); // 8*2=16 -> 1+6=7
    assert_eq!(sequence[4], 5); // 7*2=14 -> 1+4=5
}

#[test]
fn test_digit_reduction() {
    let engine = FluxMatrixEngine::new();

    assert_eq!(engine.reduce_digits(16), 7); // 1+6=7
    assert_eq!(engine.reduce_digits(123), 6); // 1+2+3=6
    assert_eq!(engine.reduce_digits(999), 9); // 9+9+9=27 -> 2+7=9
    assert_eq!(engine.reduce_digits(5), 5); // Already single digit
}

#[test]
fn test_flux_value_to_position() {
    let engine = FluxMatrixEngine::new();

    // Test 1:1 mapping from flux values to positions
    // Flux value maps directly to its position number
    assert_eq!(engine.flux_value_to_position(0), Some(0)); // Void
    assert_eq!(engine.flux_value_to_position(1), Some(1)); // Object
    assert_eq!(engine.flux_value_to_position(2), Some(2)); // Forces
    assert_eq!(engine.flux_value_to_position(3), Some(3)); // Law (sacred)
    assert_eq!(engine.flux_value_to_position(4), Some(4)); // Value
    assert_eq!(engine.flux_value_to_position(5), Some(5)); // Unit(s)
    assert_eq!(engine.flux_value_to_position(6), Some(6)); // Anti/Dark Matter (sacred)
    assert_eq!(engine.flux_value_to_position(7), Some(7)); // Assembly
    assert_eq!(engine.flux_value_to_position(8), Some(8)); // Constraints
    assert_eq!(engine.flux_value_to_position(9), Some(9)); // Material Properties (sacred)

    // Invalid values (>9)
    assert_eq!(engine.flux_value_to_position(10), None);
    assert_eq!(engine.flux_value_to_position(99), None);
}

#[test]
fn test_sacred_guides_creation() {
    let engine = FluxMatrixEngine::new();
    let matrix = engine.create_matrix("Sacred Test".to_string()).unwrap();

    // Verify sacred guides exist at positions 3, 6, 9
    assert!(matrix.sacred_guides.contains_key(&3));
    assert!(matrix.sacred_guides.contains_key(&6));
    assert!(matrix.sacred_guides.contains_key(&9));

    // Check sacred guide 3 properties
    let guide_3 = &matrix.sacred_guides[&3];
    assert_eq!(guide_3.position, 3);
    assert!(guide_3
        .divine_properties
        .contains(&"Creative Trinity".to_string()));

    // Check sacred guide 6 properties
    let guide_6 = &matrix.sacred_guides[&6];
    assert_eq!(guide_6.position, 6);
    assert!(guide_6
        .divine_properties
        .contains(&"Harmonic Balance".to_string()));

    // Check sacred guide 9 properties
    let guide_9 = &matrix.sacred_guides[&9];
    assert_eq!(guide_9.position, 9);
    assert!(guide_9
        .divine_properties
        .contains(&"Completion Cycle".to_string()));
}

#[test]
fn test_node_connections() {
    let engine = FluxMatrixEngine::new();
    let matrix = engine.create_matrix("Connection Test".to_string()).unwrap();

    // Center (position 0) should connect to all other positions (1-9)
    if let Some(center_node) = matrix.nodes.get(&0) {
        assert_eq!(
            center_node.connections.len(),
            9,
            "Center should connect to all 9 other positions"
        );
    }

    // Regular nodes should have connections
    for (position, node) in &matrix.nodes {
        if *position != 0 {
            assert!(
                !node.connections.is_empty(),
                "Node {} should have connections",
                position
            );
        }
    }
}

#[test]
fn test_matrix_validation() {
    let engine = FluxMatrixEngine::new();
    let matrix = engine.create_matrix("Validation Test".to_string()).unwrap();

    // Valid matrix should pass validation
    assert!(engine.validate_matrix(&matrix).is_ok());
}

#[test]
fn test_learning_adjustment_update() {
    let engine = FluxMatrixEngine::new();
    let mut matrix = engine.create_matrix("Learning Test".to_string()).unwrap();

    let adjustment = create_test_learning_adjustment(AdjustmentType::ConfidenceBoost, 0.15);

    let result = engine.update_matrix_with_rl(&mut matrix, adjustment);
    assert!(result.is_ok());

    // Check that adjustment was applied
    for node in matrix.nodes.values() {
        assert!(!node.attributes.dynamics.learning_adjustments.is_empty());
    }
}

#[test]
fn test_multiple_seed_sequences() {
    let engine = FluxMatrixEngine::new();

    let seeds = [1, 888, 872, 123];
    let sequences: Vec<Vec<u8>> = seeds
        .iter()
        .map(|&seed| engine.seed_to_flux_sequence(seed))
        .collect();

    // All sequences should have 9 elements
    for seq in sequences {
        assert_eq!(seq.len(), 9);
        // All values should be single digits (0-9)
        for &value in &seq {
            assert!(value <= 9, "All flux values should be single digits");
        }
    }
}

#[test]
fn test_intersection_points() {
    let engine = FluxMatrixEngine::new();
    let matrix = engine
        .create_matrix("Intersection Test".to_string())
        .unwrap();

    // Sacred guides should have intersection points
    for (position, guide) in &matrix.sacred_guides {
        assert!(
            !guide.intersection_points.is_empty(),
            "Sacred guide {} should have intersection points",
            position
        );

        // Each sacred guide should have 2 intersection points (to the other two sacred guides)
        assert_eq!(
            guide.intersection_points.len(),
            2,
            "Sacred guide {} should have 2 intersection points",
            position
        );
    }
}
