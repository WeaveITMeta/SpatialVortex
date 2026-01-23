/// Example: Using the AI Router System
/// 
/// This example demonstrates how to use the SpatialVortex AI Router
/// for handling different types of AI requests with priority queuing.

use spatial_vortex::{
    ai_router::{AIRouter, AIRequest, RequestType},
    inference_engine::InferenceEngine,
    flux_matrix::FluxMatrixEngine,
};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🌀 SpatialVortex AI Router Example\n");

    // 1. Create and setup inference engine
    println!("📦 Setting up inference engine...");
    let mut inference_engine = InferenceEngine::new();
    
    // Create some example matrices
    let flux_engine = FluxMatrixEngine::new();
    let matrices = vec![
        flux_engine.create_matrix("Artificial Intelligence".to_string())?,
        flux_engine.create_matrix("Machine Learning".to_string())?,
        flux_engine.create_matrix("Ethics and AI".to_string())?,
    ];
    
    inference_engine.load_subject_matrices(matrices)?;
    println!("   ✅ Loaded {} matrices\n", 3);

    // 2. Create AI Router
    println!("🔧 Creating AI Router...");
    let router = AIRouter::new(inference_engine);
    println!("   ✅ Router ready\n");

    // 3. Submit different types of requests
    println!("📨 Submitting requests...\n");

    // User request (interactive)
    let user_request = AIRequest::new_user(
        "What is machine learning?".to_string(),
        "user_12345".to_string()
    );
    let user_id = router.submit_request(user_request).await?;
    println!("   👤 User request submitted: {}", user_id);

    // Machine request (API)
    let machine_request = AIRequest::new_machine(
        "Analyze dataset: [1,2,3,4,5]".to_string(),
        "api_key_abc123".to_string()
    );
    let machine_id = router.submit_request(machine_request).await?;
    println!("   🤖 Machine request submitted: {}", machine_id);

    // Compliance request (safety check)
    let compliance_request = AIRequest::new_compliance(
        "Check content: 'This is user generated content'".to_string(),
        "content_policy_v2".to_string()
    );
    let compliance_id = router.submit_request(compliance_request).await?;
    println!("   🛡️  Compliance request submitted: {}", compliance_id);

    // System request (diagnostics)
    let system_request = AIRequest::new_system(
        "Health check: inference_engine".to_string()
    );
    let system_id = router.submit_request(system_request).await?;
    println!("   ⚙️  System request submitted: {}", system_id);

    // Priority request (urgent)
    let priority_request = AIRequest::new_priority(
        "URGENT: Analyze security threat".to_string(),
        "admin_001".to_string(),
        "security_incident".to_string()
    );
    let priority_id = router.submit_request(priority_request).await?;
    println!("   🚨 Priority request submitted: {}", priority_id);

    // 4. Check pending requests
    let pending = router.pending_count().await;
    println!("\n📊 Pending requests: {}", pending);

    // 5. Show statistics before processing
    let stats_before = router.get_stats().await;
    println!("\n📈 Statistics (before processing):");
    println!("   Total requests: {}", stats_before.total_requests);
    for (request_type, count) in &stats_before.requests_by_type {
        println!("   {:?}: {}", request_type, count);
    }

    // 6. Process all requests (they will be processed in priority order)
    println!("\n⚡ Processing requests in priority order...\n");
    let responses = router.process_all().await?;

    // 7. Display responses
    for (i, response) in responses.iter().enumerate() {
        println!("─────────────────────────────────────────────────────");
        println!("Response #{} - {:?} Request", i + 1, response.request_type);
        println!("─────────────────────────────────────────────────────");
        println!("Request ID: {}", response.request_id);
        println!("Response: {}", response.response);
        
        if let Some(hash) = &response.compression_hash {
            println!("Compression Hash: {}", hash);
        }
        
        if let Some(position) = response.flux_position {
            let position_name = match position {
                0 => "Foundation",
                3 => "Creative",
                5 => "Balance",
                6 => "Sacred",
                9 => "Divine",
                _ => "Other",
            };
            println!("Flux Position: {} ({})", position, position_name);
        }
        
        if let Some(elp) = &response.elp_channels {
            println!("ELP Channels:");
            println!("   Ethos (ethics): {:.1}", elp.ethos);
            println!("   Logos (logic):  {:.1}", elp.logos);
            println!("   Pathos (emotion): {:.1}", elp.pathos);
        }
        
        println!("Confidence: {:.2}", response.confidence);
        println!("Processing Time: {}ms", response.processing_time_ms);
        println!("Model: {}", response.model_used);
        println!();
    }

    // 8. Show final statistics
    let stats_after = router.get_stats().await;
    println!("─────────────────────────────────────────────────────");
    println!("📊 Final Statistics");
    println!("─────────────────────────────────────────────────────");
    println!("Total requests processed: {}", stats_after.total_requests);
    println!("Average processing time: {}ms", stats_after.average_processing_time_ms);
    println!("Rate limit hits: {}", stats_after.rate_limit_hits);
    println!("Timeouts: {}", stats_after.timeout_count);
    println!();

    // 9. Demonstrate rate limiting
    println!("🔒 Testing rate limiting...");
    println!("   Submitting {} User requests (limit: 60/min)...", 61);
    
    let mut success_count = 0;
    let mut rate_limit_hits = 0;
    
    for i in 0..61 {
        let request = AIRequest::new_user(
            format!("Request {}", i),
            "user_ratelimit".to_string()
        );
        
        match router.submit_request(request).await {
            Ok(_) => success_count += 1,
            Err(_) => rate_limit_hits += 1,
        }
    }
    
    println!("   ✅ Successful: {}", success_count);
    println!("   ❌ Rate limited: {}", rate_limit_hits);

    let final_stats = router.get_stats().await;
    println!("   Total rate limit hits: {}", final_stats.rate_limit_hits);

    println!("\n✅ Example completed successfully!");

    Ok(())
}
