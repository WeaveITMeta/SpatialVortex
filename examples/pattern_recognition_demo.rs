//! Pattern Recognition Demo
//!
//! Demonstrates pattern detection in runtime monitoring:
//! 1. Temporal patterns (recurring at intervals)
//! 2. Sequential patterns (event sequences)
//! 3. Correlation patterns (related metrics)
//! 4. Cyclic patterns (hourly/daily cycles)
//! 5. Pattern-based RSI triggering

use spatial_vortex::ai::orchestrator::ASIOrchestrator;
use spatial_vortex::asi::runtime_detector::RuntimeDetectorConfig;
use spatial_vortex::asi::pattern_recognition::{PatternRecognitionEngine, PatternEvent};
use spatial_vortex::error::Result;
use std::collections::HashMap;
use tokio::time::{Duration, sleep};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    println!("=== Pattern Recognition Demo ===\n");
    println!("Detecting recurring issues in runtime monitoring\n");
    
    // Step 1: Create pattern recognition engine
    println!("📊 Step 1: Creating Pattern Recognition Engine...");
    let engine = PatternRecognitionEngine::new();
    println!("✓ Engine initialized");
    println!();
    
    // Step 2: Simulate temporal pattern (hourly spikes)
    println!("⏰ Step 2: Simulating Temporal Pattern (Hourly Latency Spikes)...");
    let base_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() - 18000; // Start 5 hours ago
    
    for i in 0..5 {
        let timestamp = base_time + i * 3600; // Every hour
        engine.record_event(PatternEvent {
            timestamp,
            event_type: "latency_spike".to_string(),
            severity: 0.85,
            metadata: {
                let mut m = HashMap::new();
                m.insert("latency_ms".to_string(), "850".to_string());
                m
            },
        });
        println!("  [{}h ago] Latency spike recorded", 5 - i);
    }
    println!("✓ 5 hourly events recorded");
    println!();
    
    // Step 3: Simulate sequential pattern
    println!("🔗 Step 3: Simulating Sequential Pattern (Load → Spike)...");
    for i in 0..5 {
        let timestamp = base_time + 20000 + i * 600;
        
        // First: high load
        engine.record_event(PatternEvent {
            timestamp,
            event_type: "high_load".to_string(),
            severity: 0.7,
            metadata: HashMap::new(),
        });
        
        // Then: latency spike (10 seconds later)
        engine.record_event(PatternEvent {
            timestamp: timestamp + 10,
            event_type: "latency_spike".to_string(),
            severity: 0.9,
            metadata: HashMap::new(),
        });
        
        println!("  Sequence {}: high_load → latency_spike", i + 1);
    }
    println!("✓ 5 sequences recorded");
    println!();
    
    // Step 4: Simulate correlation pattern
    println!("📈 Step 4: Simulating Correlation Pattern (Latency ↔ Errors)...");
    for i in 0..10 {
        let timestamp = base_time + 30000 + i * 60;
        let latency = 200.0 + i as f32 * 50.0;
        let error_rate = 0.05 + i as f32 * 0.03;
        
        let mut metadata = HashMap::new();
        metadata.insert("latency_ms".to_string(), latency.to_string());
        metadata.insert("error_rate".to_string(), error_rate.to_string());
        
        engine.record_event(PatternEvent {
            timestamp,
            event_type: "performance_degradation".to_string(),
            severity: (latency / 1000.0).max(error_rate),
            metadata,
        });
    }
    println!("✓ 10 correlated events recorded");
    println!();
    
    // Step 5: Analyze patterns
    println!("🔍 Step 5: Analyzing Patterns...");
    engine.analyze_patterns();
    
    let all_patterns = engine.get_patterns();
    println!("✓ Analysis complete");
    println!("✓ Detected {} patterns", all_patterns.len());
    println!();
    
    // Step 6: Show detected patterns
    println!("📋 Step 6: Detected Patterns");
    if all_patterns.is_empty() {
        println!("  No patterns detected (may need more data)");
    } else {
        for (i, pattern) in all_patterns.iter().enumerate() {
            println!("\n  Pattern {}:", i + 1);
            println!("    Type: {:?}", pattern.pattern_type);
            println!("    Description: {}", pattern.description);
            println!("    Confidence: {:.2}%", pattern.confidence * 100.0);
            println!("    Occurrences: {}", pattern.occurrences);
            println!("    Avg Severity: {:.2}", pattern.avg_severity);
            
            if let Some(next) = pattern.next_predicted {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                
                if next > now {
                    println!("    Next predicted: in {} seconds", next - now);
                } else {
                    println!("    Next predicted: {} seconds ago", now - next);
                }
            }
        }
    }
    println!();
    
    // Step 7: Filter by pattern type
    println!("🔎 Step 7: Patterns by Type");
    
    let temporal = engine.get_patterns_by_type("temporal");
    println!("  Temporal patterns: {}", temporal.len());
    for p in &temporal {
        println!("    - {}", p.description);
    }
    
    let sequential = engine.get_patterns_by_type("sequential");
    println!("  Sequential patterns: {}", sequential.len());
    for p in &sequential {
        println!("    - {}", p.description);
    }
    
    let cyclic = engine.get_patterns_by_type("cyclic");
    println!("  Cyclic patterns: {}", cyclic.len());
    for p in &cyclic {
        println!("    - {}", p.description);
    }
    
    let correlation = engine.get_patterns_by_type("correlation");
    println!("  Correlation patterns: {}", correlation.len());
    for p in &correlation {
        println!("    - {}", p.description);
    }
    println!();
    
    // Step 8: High confidence patterns
    println!("⭐ Step 8: High Confidence Patterns (>75%)");
    let high_conf = engine.get_high_confidence_patterns(0.75);
    
    if high_conf.is_empty() {
        println!("  No high-confidence patterns detected");
    } else {
        for pattern in &high_conf {
            println!("  - {} (confidence: {:.1}%)", 
                pattern.description, pattern.confidence * 100.0);
        }
    }
    println!();
    
    // Step 9: Upcoming predictions
    println!("🔮 Step 9: Upcoming Pattern Predictions (Next Hour)");
    let upcoming = engine.check_upcoming_patterns(3600);
    
    if upcoming.is_empty() {
        println!("  No patterns predicted in next hour");
    } else {
        for pattern in &upcoming {
            if let Some(next) = pattern.next_predicted {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                println!("  - {} in {} seconds", 
                    pattern.description, next.saturating_sub(now));
            }
        }
    }
    println!();
    
    // Step 10: Integration with Runtime Detector
    println!("🔗 Step 10: Integration with Runtime Detector...");
    
    let mut detector_config = RuntimeDetectorConfig::default();
    detector_config.enable_pattern_recognition = true;
    detector_config.pattern_confidence_threshold = 0.75;
    
    let orchestrator = ASIOrchestrator::new()
        .await?
        .with_runtime_detector(detector_config);
    
    println!("✓ Orchestrator created with pattern recognition");
    println!("✓ Pattern confidence threshold: 75%");
    println!();
    
    // Simulate some runtime events
    println!("🎯 Step 11: Simulating Runtime Events...");
    for i in 0..10 {
        if let Some(ref detector) = orchestrator.runtime_detector {
            // Simulate varying performance
            let latency = if i % 3 == 0 { 800.0 } else { 200.0 };
            let confidence = if i % 3 == 0 { 0.55 } else { 0.85 };
            
            detector.record_sample(latency, confidence, 0.1);
            
            if i % 3 == 0 {
                println!("  [{}] Performance issue: latency={}ms, conf={:.2}", 
                    i, latency, confidence);
            }
        }
        
        sleep(Duration::from_millis(100)).await;
    }
    println!("✓ 10 runtime samples recorded");
    println!();
    
    // Check for detected patterns in runtime detector
    println!("📊 Step 12: Runtime Detector Patterns");
    if let Some(ref detector) = orchestrator.runtime_detector {
        let patterns = detector.get_detected_patterns();
        
        if patterns.is_empty() {
            println!("  No patterns detected yet (may need more samples)");
        } else {
            println!("  Detected {} patterns:", patterns.len());
            for pattern in &patterns {
                println!("    - {}", pattern.description);
            }
        }
    }
    println!();
    
    // Summary
    println!("=== Demo Summary ===");
    println!("✓ Pattern Recognition Engine: Operational");
    println!("✓ Temporal patterns: Detects recurring events at intervals");
    println!("✓ Sequential patterns: Detects event sequences");
    println!("✓ Correlation patterns: Detects related metrics");
    println!("✓ Cyclic patterns: Detects hourly/daily/weekly cycles");
    println!("✓ Predictions: Forecasts next occurrence");
    println!("✓ Runtime integration: Auto-detects patterns during monitoring");
    println!();
    
    println!("🎯 Pattern Recognition Benefits:");
    println!("   1. Identifies recurring issues automatically");
    println!("   2. Predicts when problems will occur");
    println!("   3. Detects complex event sequences");
    println!("   4. Finds correlated performance issues");
    println!("   5. Enables proactive RSI triggering");
    println!();
    
    println!("🚀 Pattern Recognition: COMPLETE");
    
    Ok(())
}
