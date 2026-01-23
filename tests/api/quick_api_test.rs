//! Quick API smoke tests
//!
//! Fast tests to verify the server is running and responding

use reqwest::Client;
use serde_json::json;

const API_BASE: &str = "http://localhost:8080/api/v1";

#[tokio::test]
async fn smoke_test_health() {
    let client = Client::new();
    let response = client
        .get(&format!("{}/health", API_BASE))
        .send()
        .await;
    
    assert!(response.is_ok(), "Server not responding");
    assert_eq!(response.unwrap().status(), 200);
    println!("✅ Server is running");
}

#[tokio::test]
async fn smoke_test_inference() {
    let client = Client::new();
    let response = client
        .post(&format!("{}/inference/reverse", API_BASE))
        .json(&json!({
            "seed_numbers": [3, 6, 9],
            "subject_filter": "all",
            "include_synonyms": true
        }))
        .send()
        .await;
    
    assert!(response.is_ok(), "Inference endpoint not responding");
    assert_eq!(response.unwrap().status(), 200);
    println!("✅ Inference engine is working");
}

#[tokio::test]
async fn smoke_test_database() {
    let client = Client::new();
    let response = client
        .get(&format!("{}/health", API_BASE))
        .send()
        .await
        .expect("Health check failed");
    
    let body: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse response");
    
    assert_eq!(body["database_status"], "healthy");
    println!("✅ Database is connected");
}

#[tokio::test]
async fn smoke_test_cache() {
    let client = Client::new();
    let response = client
        .get(&format!("{}/health", API_BASE))
        .send()
        .await
        .expect("Health check failed");
    
    let body: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse response");
    
    assert_eq!(body["cache_status"], "healthy");
    println!("✅ Cache is connected");
}
