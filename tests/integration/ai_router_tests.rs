mod common;

use common::*;
use spatial_vortex::{
    ai_router::{AIRouter, AIRequest, RequestType},
    inference_engine::InferenceEngine,
};

#[tokio::test]
async fn test_router_creation() {
    let engine = InferenceEngine::new();
    let router = AIRouter::new(engine);
    
    let stats = router.get_stats().await;
    assert_eq!(stats.total_requests, 0);
    assert_eq!(stats.rate_limit_hits, 0);
}

#[tokio::test]
async fn test_user_request_submission() {
    let engine = InferenceEngine::new();
    let router = AIRouter::new(engine);
    
    let request = AIRequest::new_user(
        "What is AI?".to_string(),
        "user_123".to_string()
    );
    
    let request_id = router.submit_request(request).await;
    assert!(request_id.is_ok());
    
    let pending = router.pending_count().await;
    assert_eq!(pending, 1);
}

#[tokio::test]
async fn test_machine_request_submission() {
    let engine = InferenceEngine::new();
    let router = AIRouter::new(engine);
    
    let request = AIRequest::new_machine(
        "Process batch data".to_string(),
        "api_key_abc123".to_string()
    );
    
    let request_id = router.submit_request(request).await;
    assert!(request_id.is_ok());
    
    // Check metadata
    let stats = router.get_stats().await;
    assert_eq!(stats.total_requests, 1);
}

#[tokio::test]
async fn test_compliance_request() {
    let engine = InferenceEngine::new();
    let router = AIRouter::new(engine);
    
    let request = AIRequest::new_compliance(
        "Check content: 'User generated text'".to_string(),
        "content_policy_v2".to_string()
    );
    
    let request_id = router.submit_request(request).await;
    assert!(request_id.is_ok());
    
    let stats = router.get_stats().await;
    assert_eq!(*stats.requests_by_type.get(&RequestType::Compliance).unwrap_or(&0), 1);
}

#[tokio::test]
async fn test_system_request() {
    let engine = InferenceEngine::new();
    let router = AIRouter::new(engine);
    
    let request = AIRequest::new_system(
        "Health check: inference_engine".to_string()
    );
    
    let request_id = router.submit_request(request).await;
    assert!(request_id.is_ok());
}

#[tokio::test]
async fn test_priority_request() {
    let engine = InferenceEngine::new();
    let router = AIRouter::new(engine);
    
    let request = AIRequest::new_priority(
        "URGENT: Critical operation".to_string(),
        "admin_001".to_string(),
        "security_incident".to_string()
    );
    
    let request_id = router.submit_request(request).await;
    assert!(request_id.is_ok());
}

#[tokio::test]
async fn test_priority_ordering() {
    let mut engine = InferenceEngine::new();
    // Load a test matrix for processing
    let matrix = create_test_matrix("Test");
    engine.update_subject_matrix(matrix);
    
    let router = AIRouter::new(engine);
    
    // Submit requests in reverse priority order
    let machine_req = AIRequest::new_machine("M".to_string(), "key".to_string());
    let user_req = AIRequest::new_user("U".to_string(), "user".to_string());
    let compliance_req = AIRequest::new_compliance("C".to_string(), "policy".to_string());
    let priority_req = AIRequest::new_priority("P".to_string(), "user".to_string(), "urgent".to_string());
    
    router.submit_request(machine_req).await.unwrap();
    router.submit_request(user_req).await.unwrap();
    router.submit_request(compliance_req).await.unwrap();
    router.submit_request(priority_req).await.unwrap();
    
    // Process all and verify order
    let responses = router.process_all().await.unwrap();
    assert_eq!(responses.len(), 4);
    
    // Should be processed in priority order: Priority, Compliance, User, Machine
    assert_eq!(responses[0].request_type, RequestType::Priority);
    assert_eq!(responses[1].request_type, RequestType::Compliance);
    assert_eq!(responses[2].request_type, RequestType::User);
    assert_eq!(responses[3].request_type, RequestType::Machine);
}

#[tokio::test]
async fn test_rate_limiting() {
    let engine = InferenceEngine::new();
    let router = AIRouter::new(engine);
    
    // Submit requests up to limit (60 for User type)
    for i in 0..60 {
        let request = AIRequest::new_user(
            format!("Request {}", i),
            "user_123".to_string()
        );
        let result = router.submit_request(request).await;
        assert!(result.is_ok(), "Request {} should succeed", i);
    }
    
    // Next request should fail
    let request = AIRequest::new_user(
        "Request 61".to_string(),
        "user_123".to_string()
    );
    let result = router.submit_request(request).await;
    assert!(result.is_err(), "Request 61 should be rate limited");
    
    let stats = router.get_stats().await;
    assert_eq!(stats.rate_limit_hits, 1);
}

#[tokio::test]
async fn test_process_single_request() {
    let mut engine = InferenceEngine::new();
    let matrix = create_test_matrix("AI");
    engine.update_subject_matrix(matrix);
    
    let router = AIRouter::new(engine);
    
    let request = AIRequest::new_user(
        "What is consciousness?".to_string(),
        "user_456".to_string()
    );
    
    router.submit_request(request).await.unwrap();
    
    let response = router.process_next().await.unwrap();
    assert!(response.is_some());
    
    let resp = response.unwrap();
    assert!(!resp.response.is_empty());
    assert!(resp.confidence >= 0.0 && resp.confidence <= 1.0);
    assert!(resp.processing_time_ms > 0);
}

#[tokio::test]
async fn test_compression_hash_generation() {
    let mut engine = InferenceEngine::new();
    let matrix = create_test_matrix("Test");
    engine.update_subject_matrix(matrix);
    
    let router = AIRouter::new(engine);
    
    let request = AIRequest::new_user(
        "Test prompt".to_string(),
        "user_789".to_string()
    );
    
    router.submit_request(request).await.unwrap();
    let response = router.process_next().await.unwrap().unwrap();
    
    // User requests have compression enabled
    assert!(response.compression_hash.is_some());
    let hash = response.compression_hash.unwrap();
    assert_eq!(hash.len(), 24); // 12 bytes = 24 hex chars
    
    assert!(response.flux_position.is_some());
    assert_eq!(response.flux_position.unwrap(), 3); // User → Position 3 (Creative)
}

#[tokio::test]
async fn test_elp_channel_detection() {
    let mut engine = InferenceEngine::new();
    let matrix = create_test_matrix("Ethics");
    engine.update_subject_matrix(matrix);
    
    let router = AIRouter::new(engine);
    
    // Request with ethical content
    let request = AIRequest::new_user(
        "What is the right thing to do ethically?".to_string(),
        "user_ethics".to_string()
    );
    
    router.submit_request(request).await.unwrap();
    let response = router.process_next().await.unwrap().unwrap();
    
    // Should have ELP channels
    assert!(response.elp_channels.is_some());
    let elp = response.elp_channels.unwrap();
    
    // Ethos should be boosted due to "ethically" keyword
    assert!(elp.ethos > 6.0, "Ethos should be elevated: {}", elp.ethos);
}

#[tokio::test]
async fn test_statistics_tracking() {
    let mut engine = InferenceEngine::new();
    let matrix = create_test_matrix("Stats");
    engine.update_subject_matrix(matrix);
    
    let router = AIRouter::new(engine);
    
    // Submit multiple requests
    for i in 0..5 {
        let request = AIRequest::new_user(
            format!("Request {}", i),
            format!("user_{}", i)
        );
        router.submit_request(request).await.unwrap();
    }
    
    let stats_before = router.get_stats().await;
    assert_eq!(stats_before.total_requests, 5);
    
    // Process all
    router.process_all().await.unwrap();
    
    let stats_after = router.get_stats().await;
    assert!(stats_after.average_processing_time_ms > 0);
}

#[tokio::test]
async fn test_multiple_request_types() {
    let mut engine = InferenceEngine::new();
    let matrix = create_test_matrix("MultiType");
    engine.update_subject_matrix(matrix);
    
    let router = AIRouter::new(engine);
    
    // Submit one of each type
    router.submit_request(AIRequest::new_user("U".to_string(), "u1".to_string())).await.unwrap();
    router.submit_request(AIRequest::new_machine("M".to_string(), "key".to_string())).await.unwrap();
    router.submit_request(AIRequest::new_compliance("C".to_string(), "p1".to_string())).await.unwrap();
    router.submit_request(AIRequest::new_system("S".to_string())).await.unwrap();
    router.submit_request(AIRequest::new_priority("P".to_string(), "u1".to_string(), "r".to_string())).await.unwrap();
    
    let stats = router.get_stats().await;
    assert_eq!(stats.total_requests, 5);
    assert_eq!(*stats.requests_by_type.get(&RequestType::User).unwrap(), 1);
    assert_eq!(*stats.requests_by_type.get(&RequestType::Machine).unwrap(), 1);
    assert_eq!(*stats.requests_by_type.get(&RequestType::Compliance).unwrap(), 1);
    assert_eq!(*stats.requests_by_type.get(&RequestType::System).unwrap(), 1);
    assert_eq!(*stats.requests_by_type.get(&RequestType::Priority).unwrap(), 1);
}

#[tokio::test]
async fn test_empty_queue_processing() {
    let engine = InferenceEngine::new();
    let router = AIRouter::new(engine);
    
    // Try to process with empty queue
    let response = router.process_next().await.unwrap();
    assert!(response.is_none());
}

#[tokio::test]
async fn test_flux_position_mapping() {
    let mut engine = InferenceEngine::new();
    let matrix = create_test_matrix("Position");
    engine.update_subject_matrix(matrix);
    
    let router = AIRouter::new(engine);
    
    // Test each request type gets correct position
    let test_cases = vec![
        (RequestType::Priority, 9),
        (RequestType::Compliance, 6),
        (RequestType::User, 3),
        (RequestType::System, 0),
        (RequestType::Machine, 5),
    ];
    
    for (req_type, expected_position) in test_cases {
        let request = match req_type {
            RequestType::User => AIRequest::new_user("test".to_string(), "u".to_string()),
            RequestType::Machine => AIRequest::new_machine("test".to_string(), "k".to_string()),
            RequestType::Compliance => AIRequest::new_compliance("test".to_string(), "p".to_string()),
            RequestType::System => AIRequest::new_system("test".to_string()),
            RequestType::Priority => AIRequest::new_priority("test".to_string(), "u".to_string(), "r".to_string()),
        };
        
        router.submit_request(request).await.unwrap();
        let response = router.process_next().await.unwrap().unwrap();
        
        assert_eq!(
            response.flux_position.unwrap(),
            expected_position,
            "{:?} should map to position {}",
            req_type,
            expected_position
        );
    }
}

#[tokio::test]
async fn test_request_timeout_check() {
    let request = AIRequest::new_user("test".to_string(), "user".to_string());
    
    // Immediately should not be timed out
    assert!(!request.has_timed_out());
    
    // Age should be minimal
    assert!(request.age_seconds() < 2);
}

#[tokio::test]
async fn test_request_type_properties() {
    // Verify priority levels
    assert_eq!(RequestType::Priority.priority_level(), 0);
    assert_eq!(RequestType::Compliance.priority_level(), 1);
    assert_eq!(RequestType::User.priority_level(), 2);
    assert_eq!(RequestType::System.priority_level(), 3);
    assert_eq!(RequestType::Machine.priority_level(), 4);
    
    // Verify timeouts
    assert_eq!(RequestType::Priority.default_timeout(), 5);
    assert_eq!(RequestType::Compliance.default_timeout(), 10);
    assert_eq!(RequestType::User.default_timeout(), 30);
    assert_eq!(RequestType::System.default_timeout(), 60);
    assert_eq!(RequestType::Machine.default_timeout(), 120);
    
    // Verify rate limits
    assert_eq!(RequestType::Priority.rate_limit(), 100);
    assert_eq!(RequestType::Compliance.rate_limit(), 200);
    assert_eq!(RequestType::User.rate_limit(), 60);
    assert_eq!(RequestType::System.rate_limit(), 30);
    assert_eq!(RequestType::Machine.rate_limit(), 600);
}
