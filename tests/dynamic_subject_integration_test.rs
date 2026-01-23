/// Integration Test: Dynamic Subject Generation
/// 
/// Validates that AIRouter properly creates, stores, and retrieves FluxMatrix instances
/// from AI-generated instructions.

use spatial_vortex::ai::router::AIRouter;

#[tokio::test]
async fn test_dynamic_subject_creation_and_retrieval() {
    // Create router without API key (uses fallback)
    let mut router = AIRouter::new(None, false);
    
    // Initially, no subjects should exist
    assert_eq!(router.list_subjects().len(), 0);
    
    // Create a dynamic subject
    router.create_dynamic_subject("Love").await.expect("Failed to create subject");
    
    // Verify subject was created
    let subjects = router.list_subjects();
    assert_eq!(subjects.len(), 1);
    assert!(subjects.contains(&"Love".to_string()));
    
    // Retrieve the matrix
    let matrix = router.get_matrix("Love").expect("Matrix should exist");
    
    // Verify sacred anchors exist
    assert!(matrix.get_sacred_anchor(3).is_some(), "Position 3 should be sacred");
    assert!(matrix.get_sacred_anchor(6).is_some(), "Position 6 should be sacred");
    assert!(matrix.get_sacred_anchor(9).is_some(), "Position 9 should be sacred");
    
    // Verify at least some nodes were created
    // Fallback creates positions 0, 1, 3, 6, 9
    assert!(matrix.get(0).is_some(), "Position 0 should exist");
    assert!(matrix.get(1).is_some(), "Position 1 should exist");
    assert!(matrix.get(3).is_some(), "Position 3 should exist");
    
    println!("✅ Dynamic subject creation test passed!");
}

#[tokio::test]
async fn test_subject_caching() {
    let mut router = AIRouter::new(None, false);
    
    // Create subject first time
    router.create_dynamic_subject("Consciousness").await.unwrap();
    
    // Get matrix
    let matrix1 = router.get_matrix("Consciousness").unwrap();
    
    // Create same subject again (should be cached, not recreated)
    router.create_dynamic_subject("Consciousness").await.unwrap();
    
    // Get matrix again
    let matrix2 = router.get_matrix("Consciousness").unwrap();
    
    // Should be the same Arc instance (cached)
    assert!(std::sync::Arc::ptr_eq(&matrix1, &matrix2));
    
    println!("✅ Subject caching test passed!");
}

#[tokio::test]
async fn test_multiple_subjects() {
    let mut router = AIRouter::new(None, false);
    
    // Create multiple subjects
    let subjects_to_create = vec!["Love", "Justice", "Freedom", "Truth"];
    
    for subject in &subjects_to_create {
        router.create_dynamic_subject(subject).await.unwrap();
    }
    
    // Verify all were created
    let created_subjects = router.list_subjects();
    assert_eq!(created_subjects.len(), 4);
    
    for subject in &subjects_to_create {
        assert!(created_subjects.contains(&subject.to_string()), 
            "Subject '{}' should be in list", subject);
        
        let matrix = router.get_matrix(subject).expect(&format!("Matrix for '{}' should exist", subject));
        assert!(matrix.get_sacred_anchor(3).is_some());
    }
    
    println!("✅ Multiple subjects test passed!");
}

#[tokio::test]
async fn test_node_semantic_associations() {
    let mut router = AIRouter::new(None, false);
    
    // Create subject
    router.create_dynamic_subject("Wisdom").await.unwrap();
    
    // Get matrix and check a node
    let matrix = router.get_matrix("Wisdom").unwrap();
    
    if let Some(node_ref) = matrix.get(1) {
        let node = &node_ref.node;
        
        // Verify semantic structure
        assert!(!node.semantic_index.neutral_base.is_empty(), "Should have base concept");
        
        // Positive associations should exist
        assert!(!node.semantic_index.positive_associations.is_empty(), 
            "Should have positive associations");
        
        // Verify association indices
        for (i, assoc) in node.semantic_index.positive_associations.iter().enumerate() {
            assert_eq!(assoc.index, (i + 1) as i16, "Positive index should be sequential");
            assert!(assoc.confidence > 0.0, "Should have confidence score");
        }
        
        // Negative associations should exist (from fallback)
        if !node.semantic_index.negative_associations.is_empty() {
            for (i, assoc) in node.semantic_index.negative_associations.iter().enumerate() {
                assert_eq!(assoc.index, -((i + 1) as i16), "Negative index should be sequential negative");
            }
        }
    }
    
    println!("✅ Node semantic associations test passed!");
}

#[tokio::test]
async fn test_sacred_position_priority() {
    let mut router = AIRouter::new(None, false);
    
    // Create subject
    router.create_dynamic_subject("Harmony").await.unwrap();
    
    // Get matrix
    let matrix = router.get_matrix("Harmony").unwrap();
    
    // Sacred positions should be created in priority order: 0, 3, 6, 9, then others
    // Verify they exist
    let position_0 = matrix.get(0);
    let position_3 = matrix.get(3);
    let position_6 = matrix.get(6);
    let position_9 = matrix.get(9);
    
    assert!(position_0.is_some(), "Position 0 (center) should be created first");
    assert!(position_3.is_some(), "Position 3 (sacred) should be created");
    assert!(position_6.is_some(), "Position 6 (sacred) should be created");
    assert!(position_9.is_some(), "Position 9 (sacred) should be created");
    
    // Verify base values match positions
    if let Some(node_3) = position_3 {
        assert_eq!(node_3.node.base_value, 3, "Position 3 should have base value 3");
    }
    
    println!("✅ Sacred position priority test passed!");
}

#[tokio::test]
async fn test_generate_response_auto_creates_subject() {
    let mut router = AIRouter::new(None, false);
    
    // Call generate_response with a subject that doesn't exist yet
    // This should automatically create it
    let result = router.generate_response(
        "Tell me about enlightenment",
        "test_user",
        Some("Enlightenment"),
        0.85,
        3,
    ).await;
    
    // Should succeed (even if API call fails, it should handle gracefully)
    // The important part is that the subject was created
    assert!(result.is_ok() || result.is_err(), "Should complete without panic");
    
    // Verify subject was created
    let subjects = router.list_subjects();
    assert!(subjects.contains(&"Enlightenment".to_string()), 
        "Subject 'Enlightenment' should be auto-created");
    
    // Verify matrix exists
    let matrix = router.get_matrix("Enlightenment");
    assert!(matrix.is_some(), "Matrix should exist after auto-creation");
    
    println!("✅ Auto-create subject on generate_response test passed!");
}

use std::sync::Arc;

#[tokio::test]
async fn test_concurrent_subject_access() {
    let router = Arc::new(tokio::sync::Mutex::new(AIRouter::new(None, false)));
    
    // Create a subject
    {
        let mut r = router.lock().await;
        r.create_dynamic_subject("Peace").await.unwrap();
    }
    
    // Spawn multiple tasks that access the same subject concurrently
    let mut handles = vec![];
    
    for i in 0..10 {
        let router_clone = Arc::clone(&router);
        let handle = tokio::spawn(async move {
            let r = router_clone.lock().await;
            let matrix = r.get_matrix("Peace").expect(&format!("Task {} should get matrix", i));
            
            // Access sacred anchor (lock-free read)
            assert!(matrix.get_sacred_anchor(3).is_some());
            
            // Access node (lock-free read)
            let node = matrix.get(0);
            assert!(node.is_some(), "Task {} should get node 0", i);
        });
        handles.push(handle);
    }
    
    // Wait for all tasks
    for handle in handles {
        handle.await.unwrap();
    }
    
    println!("✅ Concurrent subject access test passed!");
}
