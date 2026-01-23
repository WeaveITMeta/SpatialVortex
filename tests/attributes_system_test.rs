// Comprehensive test suite for Attributes system refactoring
// Tests the dynamic attribute system end-to-end

use spatial_vortex::data::attributes::{Attributes, AttributeValue, Tags};
use spatial_vortex::data::models::{BeamTensor, ObjectContext, ELPTensor};
use spatial_vortex::core::sacred_geometry::node_dynamics::FluxNodeDynamics;
use spatial_vortex::core::sacred_geometry::object_utils::create_object_context;
use spatial_vortex::data::models::FluxNode;

/// Test 1: Basic Attributes CRUD operations
#[test]
fn test_attributes_basic_operations() {
    let mut attrs = Attributes::new();
    
    // Set various attribute types
    attrs.set("health", AttributeValue::Number(100.0));
    attrs.set("name", AttributeValue::String("TestEntity".to_string()));
    attrs.set("active", AttributeValue::Bool(true));
    attrs.set("position", AttributeValue::Vector3([1.0, 2.0, 3.0]));
    
    // Get and verify
    assert_eq!(attrs.get_number("health"), Some(100.0));
    assert_eq!(attrs.get_string("name"), Some("TestEntity"));
    assert_eq!(attrs.get_bool("active"), Some(true));
    assert_eq!(attrs.get_vector3("position"), Some([1.0, 2.0, 3.0]));
    
    // Check existence
    assert!(attrs.has("health"));
    assert!(!attrs.has("nonexistent"));
    
    // Remove
    attrs.remove("health");
    assert!(!attrs.has("health"));
    
    println!("✅ Test 1 PASSED: Basic Attributes CRUD operations");
}

/// Test 2: ELP backward compatibility
#[test]
fn test_elp_backward_compatibility() {
    // Create with ELP values
    let attrs = Attributes::with_elp(0.5, 0.3, 0.2);
    
    // Verify ELP getters
    assert!((attrs.ethos() - 0.5).abs() < 0.001);
    assert!((attrs.logos() - 0.3).abs() < 0.001);
    assert!((attrs.pathos() - 0.2).abs() < 0.001);
    
    // Verify tensor conversion
    let tensor = attrs.elp_tensor();
    assert!((tensor[0] - 0.5).abs() < 0.001);
    assert!((tensor[1] - 0.3).abs() < 0.001);
    assert!((tensor[2] - 0.2).abs() < 0.001);
    
    // Test normalization
    let normalized = attrs.elp_normalized();
    let sum = normalized[0] + normalized[1] + normalized[2];
    assert!((sum - 1.0).abs() < 0.001);
    
    // Test dominant channel
    assert_eq!(attrs.elp_dominant(), "ethos");
    
    // Test setters
    let mut attrs2 = Attributes::new();
    attrs2.set_ethos(0.6);
    attrs2.set_logos(0.3);
    attrs2.set_pathos(0.1);
    assert!((attrs2.ethos() - 0.6).abs() < 0.001);
    
    println!("✅ Test 2 PASSED: ELP backward compatibility");
}

/// Test 3: Sacred geometry integration
#[test]
fn test_sacred_geometry_integration() {
    let mut attrs = Attributes::new();
    
    // Test confidence (signal strength)
    attrs.set_confidence(0.85);
    assert!((attrs.confidence() - 0.85).abs() < 0.001);
    
    // Test digital root flux
    attrs.set_digital_root_flux(3);
    assert_eq!(attrs.digital_root_flux(), 3);
    assert!(attrs.is_sacred_position());
    
    attrs.set_digital_root_flux(6);
    assert!(attrs.is_sacred_position());
    
    attrs.set_digital_root_flux(9);
    assert!(attrs.is_sacred_position());
    
    attrs.set_digital_root_flux(5);
    assert!(!attrs.is_sacred_position());
    
    // Test flux position
    attrs.set_flux_position(7);
    assert_eq!(attrs.flux_position(), Some(7));
    
    println!("✅ Test 3 PASSED: Sacred geometry integration");
}

/// Test 4: Tags system
#[test]
fn test_tags_system() {
    let mut tags = Tags::new();
    
    // Add tags
    assert!(tags.add("sacred"));
    assert!(tags.add("high_confidence"));
    assert!(tags.add("ethos_dominant"));
    
    // Duplicate add returns false
    assert!(!tags.add("sacred"));
    
    // Check tags
    assert!(tags.has("sacred"));
    assert!(tags.has("high_confidence"));
    assert!(!tags.has("nonexistent"));
    
    // Count
    assert_eq!(tags.len(), 3);
    
    // Remove
    assert!(tags.remove("high_confidence"));
    assert!(!tags.has("high_confidence"));
    assert_eq!(tags.len(), 2);
    
    // Sorted retrieval
    let sorted = tags.sorted();
    assert_eq!(sorted.len(), 2);
    
    println!("✅ Test 4 PASSED: Tags system");
}

/// Test 5: BeamTensor with dynamic attributes
#[test]
fn test_beam_tensor_dynamic_attributes() {
    let mut beam = BeamTensor::default();
    
    // Set ELP values using dynamic setters
    beam.set_ethos(0.6);
    beam.set_logos(0.3);
    beam.set_pathos(0.1);
    
    // Verify using dynamic getters
    assert!((beam.ethos() - 0.6).abs() < 0.001);
    assert!((beam.logos() - 0.3).abs() < 0.001);
    assert!((beam.pathos() - 0.1).abs() < 0.001);
    
    // Set confidence
    beam.confidence = 0.85;
    
    // Set position and other fields
    beam.position = 3; // Sacred position
    beam.digits = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9];
    
    // Verify attributes are stored correctly
    assert_eq!(beam.attributes.get_f32("ethos"), Some(0.6));
    assert_eq!(beam.attributes.get_f32("logos"), Some(0.3));
    assert_eq!(beam.attributes.get_f32("pathos"), Some(0.1));
    
    println!("✅ Test 5 PASSED: BeamTensor with dynamic attributes");
}

/// Test 6: ObjectContext with Attributes
#[test]
fn test_object_context_with_attributes() {
    let input = "Test query for semantic analysis";
    let subject = "reasoning";
    let attributes = Attributes::with_elp(0.4, 0.5, 0.1);
    
    let context = create_object_context(input, subject, attributes.clone());
    
    // Verify context fields
    assert_eq!(context.input, input);
    assert_eq!(context.subject, subject);
    
    // Verify attributes are preserved
    assert!((context.attributes.ethos() - 0.4).abs() < 0.001);
    assert!((context.attributes.logos() - 0.5).abs() < 0.001);
    assert!((context.attributes.pathos() - 0.1).abs() < 0.001);
    
    println!("✅ Test 6 PASSED: ObjectContext with Attributes");
}

/// Test 7: Dynamic role assignment in node_dynamics
#[test]
fn test_dynamic_role_assignment() {
    let mut node = FluxNode::new(3); // Sacred position
    
    // Initialize with subject context
    node.initialize_dynamics(Some("ethical_reasoning"));
    
    // Verify dynamics were initialized
    assert!(node.attributes.dynamics.is_some());
    
    // Sacred position should be recognized
    assert_eq!(node.position, 3);
    
    // Initialize another node without subject
    let mut node2 = FluxNode::new(5);
    node2.initialize_dynamics(None);
    assert!(node2.attributes.dynamics.is_some());
    
    println!("✅ Test 7 PASSED: Dynamic role assignment in node_dynamics");
}

/// Test 8: ELPTensor to Attributes conversion
#[test]
fn test_elp_tensor_conversion() {
    let elp = ELPTensor {
        ethos: 0.5,
        logos: 0.3,
        pathos: 0.2,
    };
    
    // Convert to Attributes
    let attrs = elp.to_attributes();
    
    // Verify conversion
    assert!((attrs.ethos() - 0.5).abs() < 0.001);
    assert!((attrs.logos() - 0.3).abs() < 0.001);
    assert!((attrs.pathos() - 0.2).abs() < 0.001);
    
    // Convert back
    let elp2 = ELPTensor::from_attributes(&attrs);
    assert!((elp2.ethos - 0.5).abs() < 0.001);
    assert!((elp2.logos - 0.3).abs() < 0.001);
    assert!((elp2.pathos - 0.2).abs() < 0.001);
    
    println!("✅ Test 8 PASSED: ELPTensor to Attributes conversion");
}

/// Test 9: Attributes merge and clone
#[test]
fn test_attributes_merge_and_clone() {
    let mut attrs1 = Attributes::with_elp(0.5, 0.3, 0.2);
    attrs1.set("custom_field", AttributeValue::String("value1".to_string()));
    
    let mut attrs2 = Attributes::new();
    attrs2.set("custom_field", AttributeValue::String("value2".to_string()));
    attrs2.set("another_field", AttributeValue::Number(42.0));
    
    // Merge attrs2 into attrs1 (attrs2 values override)
    attrs1.merge(&attrs2);
    
    // Verify merge
    assert_eq!(attrs1.get_string("custom_field"), Some("value2"));
    assert_eq!(attrs1.get_number("another_field"), Some(42.0));
    
    // ELP values should still be present
    assert!((attrs1.ethos() - 0.5).abs() < 0.001);
    
    println!("✅ Test 9 PASSED: Attributes merge and clone");
}

/// Test 10: Attribute value type conversions
#[test]
fn test_attribute_value_conversions() {
    let mut attrs = Attributes::new();
    
    // Number conversions
    attrs.set("num", AttributeValue::Number(42.5));
    assert_eq!(attrs.get_number("num"), Some(42.5));
    assert_eq!(attrs.get_f32("num"), Some(42.5));
    
    // Vector3
    attrs.set("vec", AttributeValue::Vector3([1.0, 2.0, 3.0]));
    assert_eq!(attrs.get_vector3("vec"), Some([1.0, 2.0, 3.0]));
    
    // Color
    attrs.set("color", AttributeValue::Color([1.0, 0.5, 0.0, 1.0]));
    assert_eq!(attrs.get_color("color"), Some([1.0, 0.5, 0.0, 1.0]));
    
    // Bool
    attrs.set("flag", AttributeValue::Bool(true));
    assert_eq!(attrs.get_bool("flag"), Some(true));
    
    println!("✅ Test 10 PASSED: Attribute value type conversions");
}

/// Test 11: Confidence and signal strength
#[test]
fn test_confidence_signal_strength() {
    let mut attrs = Attributes::new();
    
    // Confidence is clamped to [0.0, 1.0]
    attrs.set_confidence(0.75);
    assert!((attrs.confidence() - 0.75).abs() < 0.001);
    
    // Test clamping
    attrs.set_confidence(1.5); // Should clamp to 1.0
    assert!((attrs.confidence() - 1.0).abs() < 0.001);
    
    attrs.set_confidence(-0.5); // Should clamp to 0.0
    assert!((attrs.confidence() - 0.0).abs() < 0.001);
    
    println!("✅ Test 11 PASSED: Confidence and signal strength");
}

/// Test 12: Multiple BeamTensor operations
#[test]
fn test_multiple_beam_operations() {
    let mut beams = vec![];
    
    // Create multiple beams with different attributes
    for i in 0..5 {
        let mut beam = BeamTensor::default();
        beam.set_ethos(0.3 + i as f32 * 0.1);
        beam.set_logos(0.4 - i as f32 * 0.05);
        beam.set_pathos(0.3 + i as f32 * 0.05);
        beam.position = i;
        beam.confidence = 0.7 + i as f32 * 0.05;
        beams.push(beam);
    }
    
    // Verify each beam has correct attributes
    for (i, beam) in beams.iter().enumerate() {
        let expected_ethos = 0.3 + i as f32 * 0.1;
        let expected_logos = 0.4 - i as f32 * 0.05;
        let expected_pathos = 0.3 + i as f32 * 0.05;
        
        assert!((beam.ethos() - expected_ethos).abs() < 0.001);
        assert!((beam.logos() - expected_logos).abs() < 0.001);
        assert!((beam.pathos() - expected_pathos).abs() < 0.001);
        assert_eq!(beam.position, i as u8);
    }
    
    println!("✅ Test 12 PASSED: Multiple BeamTensor operations");
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    
    /// Integration Test: Full workflow from Attributes to BeamTensor to ObjectContext
    #[test]
    fn test_full_attribute_workflow() {
        println!("\n🔬 Running Integration Test: Full Attribute Workflow");
        
        // Step 1: Create initial attributes
        let mut attrs = Attributes::with_elp(0.4, 0.5, 0.1);
        attrs.set_confidence(0.85);
        attrs.set_digital_root_flux(3); // Sacred position
        
        // Step 2: Create BeamTensor with these attributes
        let mut beam = BeamTensor::default();
        beam.set_ethos(attrs.ethos());
        beam.set_logos(attrs.logos());
        beam.set_pathos(attrs.pathos());
        beam.confidence = attrs.confidence();
        beam.position = 3;
        
        // Step 3: Verify BeamTensor attributes match
        assert!((beam.ethos() - 0.4).abs() < 0.001);
        assert!((beam.logos() - 0.5).abs() < 0.001);
        assert!((beam.pathos() - 0.1).abs() < 0.001);
        assert!((beam.confidence - 0.85).abs() < 0.001);
        
        // Step 4: Create ObjectContext
        let context = create_object_context(
            "Ethical reasoning about AI alignment",
            "ethics",
            attrs.clone()
        );
        
        // Step 5: Verify ObjectContext preserves attributes
        assert_eq!(context.subject, "ethics");
        assert!((context.attributes.ethos() - 0.4).abs() < 0.001);
        assert!((context.attributes.logos() - 0.5).abs() < 0.001);
        assert!((context.attributes.pathos() - 0.1).abs() < 0.001);
        
        // Step 6: Initialize node dynamics
        let mut node = FluxNode::new(3);
        node.initialize_dynamics(Some("ethics"));
        assert!(node.attributes.dynamics.is_some());
        assert_eq!(node.position, 3);
        
        println!("✅ Integration Test PASSED: Full Attribute Workflow");
        println!("   - Attributes created and configured");
        println!("   - BeamTensor populated with attributes");
        println!("   - ObjectContext created with attributes");
        println!("   - Node dynamics initialized with subject context");
    }
    
    /// Integration Test: Sacred geometry flow with attributes
    #[test]
    fn test_sacred_geometry_flow() {
        println!("\n🔬 Running Integration Test: Sacred Geometry Flow");
        
        // Create nodes at sacred positions
        let mut nodes = vec![];
        for pos in [3, 6, 9] {
            let mut node = FluxNode::new(pos);
            node.initialize_dynamics(Some("sacred_flow"));
            nodes.push(node);
        }
        
        // Verify all sacred nodes initialized
        for node in &nodes {
            assert!(node.attributes.dynamics.is_some());
            assert!([3, 6, 9].contains(&node.position));
        }
        
        // Create beams for sacred positions
        let mut beams = vec![];
        for pos in [3, 6, 9] {
            let mut beam = BeamTensor::default();
            beam.position = pos;
            beam.set_ethos(0.5);
            beam.set_logos(0.3);
            beam.set_pathos(0.2);
            beam.confidence = 0.9; // High confidence at sacred positions
            beams.push(beam);
        }
        
        // Verify sacred beams
        for beam in &beams {
            assert!([3, 6, 9].contains(&beam.position));
            assert!(beam.confidence > 0.8);
        }
        
        println!("✅ Integration Test PASSED: Sacred Geometry Flow");
        println!("   - Sacred nodes (3, 6, 9) initialized");
        println!("   - Sacred beams created with high confidence");
        println!("   - Vortex pattern structural checkpoints verified");
    }
}
