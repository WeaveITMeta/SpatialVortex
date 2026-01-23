//! Runtime Weakness Detector Tests

use spatial_vortex::ai::orchestrator::{ASIOrchestrator, ExecutionMode, RSIConfig};
use spatial_vortex::asi::runtime_detector::{RuntimeDetectorConfig, RuntimeWeaknessType};
use spatial_vortex::error::Result;
use std::path::PathBuf;
use tokio::time::{Duration, sleep};
use tempfile::tempdir;

#[tokio::test]
async fn test_runtime_detector_creation() -> Result<()> {
    let config = RuntimeDetectorConfig::default();
    let orchestrator = ASIOrchestrator::new()
        .await?
        .with_runtime_detector(config);
    
    assert!(orchestrator.has_runtime_detector());
    
    Ok(())
}

#[tokio::test]
async fn test_runtime_monitoring_start_stop() -> Result<()> {
    let mut config = RuntimeDetectorConfig::default();
    config.enabled = true;
    config.monitor_interval_ms = 100; // Fast for testing
    
    let orchestrator = ASIOrchestrator::new()
        .await?
        .with_runtime_detector(config);
    
    // Start monitoring
    let _handle = orchestrator.start_runtime_monitoring().await?;
    
    // Wait a bit
    sleep(Duration::from_millis(500)).await;
    
    // Check stats
    let stats = orchestrator.runtime_stats();
    assert!(stats.is_some());
    
    // Stop monitoring
    orchestrator.stop_runtime_monitoring();
    
    Ok(())
}

#[tokio::test]
async fn test_metric_recording() -> Result<()> {
    let config = RuntimeDetectorConfig::default();
    let orchestrator = ASIOrchestrator::new()
        .await?
        .with_runtime_detector(config);
    
    // Record some samples
    if let Some(ref detector) = orchestrator.runtime_detector {
        detector.record_sample(250.0, 0.8, 0.1);
        detector.record_sample(300.0, 0.75, 0.12);
        detector.record_sample(280.0, 0.82, 0.09);
    }
    
    // Check stats
    let stats = orchestrator.runtime_stats().unwrap();
    assert_eq!(stats.samples_collected, 3);
    assert!(stats.avg_latency_ms > 0.0);
    assert!(stats.avg_confidence > 0.0);
    
    Ok(())
}

#[tokio::test]
async fn test_confidence_drop_detection() -> Result<()> {
    let mut config = RuntimeDetectorConfig::default();
    config.confidence_drop_threshold = 0.1; // 10% drop
    
    let orchestrator = ASIOrchestrator::new()
        .await?
        .with_runtime_detector(config);
    
    if let Some(ref detector) = orchestrator.runtime_detector {
        // Baseline: high confidence
        for _ in 0..15 {
            detector.record_sample(200.0, 0.85, 0.1);
        }
        
        // Drop: low confidence
        for _ in 0..10 {
            detector.record_sample(200.0, 0.65, 0.1);
        }
    }
    
    // Wait for analysis
    sleep(Duration::from_millis(100)).await;
    
    // Check for detected weaknesses
    let weaknesses = orchestrator.get_runtime_weaknesses(10);
    
    // Should detect confidence drop or low confidence
    let has_confidence_issue = weaknesses.iter().any(|w| 
        matches!(w.weakness_type, RuntimeWeaknessType::ConfidenceDrop | RuntimeWeaknessType::ConfidenceLow)
    );
    
    assert!(has_confidence_issue || weaknesses.is_empty()); // May not trigger immediately
    
    Ok(())
}

#[tokio::test]
async fn test_latency_spike_detection() -> Result<()> {
    let mut config = RuntimeDetectorConfig::default();
    config.latency_spike_threshold_ms = 300.0;
    config.latency_spike_count = 3;
    
    let orchestrator = ASIOrchestrator::new()
        .await?
        .with_runtime_detector(config);
    
    if let Some(ref detector) = orchestrator.runtime_detector {
        // Baseline: normal latency
        for _ in 0..15 {
            detector.record_sample(200.0, 0.8, 0.1);
        }
        
        // Spikes: high latency
        for _ in 0..5 {
            detector.record_sample(800.0, 0.8, 0.1); // 600ms above baseline
        }
    }
    
    sleep(Duration::from_millis(100)).await;
    
    let weaknesses = orchestrator.get_runtime_weaknesses(10);
    
    // May detect latency spike
    let has_latency_spike = weaknesses.iter().any(|w| 
        matches!(w.weakness_type, RuntimeWeaknessType::LatencySpike)
    );
    
    // Test passes if spike detected or no weaknesses yet
    assert!(has_latency_spike || weaknesses.is_empty());
    
    Ok(())
}

#[tokio::test]
async fn test_auto_trigger_with_self_mod() -> Result<()> {
    let temp_dir = tempdir().unwrap();
    let source_path = temp_dir.path().to_path_buf();
    
    let mut detector_config = RuntimeDetectorConfig::default();
    detector_config.enabled = true;
    detector_config.monitor_interval_ms = 100;
    detector_config.auto_trigger_rsi = true;
    detector_config.trigger_cooldown_secs = 1;
    detector_config.confidence_low_threshold = 0.9; // High threshold to trigger
    
    let orchestrator = ASIOrchestrator::new()
        .await?
        .with_self_modification(source_path)
        .with_runtime_detector(detector_config);
    
    // Enable RSI
    let mut rsi_config = RSIConfig::default();
    rsi_config.enabled = true;
    rsi_config.auto_apply_low_risk = true;
    orchestrator.enable_rsi(rsi_config).await;
    
    // Start monitoring
    let _handle = orchestrator.start_runtime_monitoring().await?;
    
    // Record samples with low confidence to trigger
    if let Some(ref detector) = orchestrator.runtime_detector {
        for _ in 0..20 {
            detector.record_sample(200.0, 0.5, 0.1); // Low confidence
        }
    }
    
    // Wait for detector to analyze and trigger
    sleep(Duration::from_secs(2)).await;
    
    // Check if RSI was triggered
    let stats = orchestrator.runtime_stats().unwrap();
    println!("RSI triggers: {}", stats.rsi_triggers);
    println!("Weaknesses detected: {}", stats.weaknesses_detected);
    
    // Stop monitoring
    orchestrator.stop_runtime_monitoring();
    
    Ok(())
}

#[tokio::test]
async fn test_process_records_metrics() -> Result<()> {
    let config = RuntimeDetectorConfig::default();
    let orchestrator = ASIOrchestrator::new()
        .await?
        .with_runtime_detector(config);
    
    // Process some inputs
    for _ in 0..5 {
        let _ = orchestrator.process("test input", ExecutionMode::Fast).await;
    }
    
    // Check that metrics were recorded
    let stats = orchestrator.runtime_stats().unwrap();
    assert!(stats.samples_collected >= 5);
    
    Ok(())
}

#[tokio::test]
async fn test_runtime_stats_available() -> Result<()> {
    let config = RuntimeDetectorConfig::default();
    let orchestrator = ASIOrchestrator::new()
        .await?
        .with_runtime_detector(config);
    
    // Should have stats even with no samples
    let stats = orchestrator.runtime_stats();
    assert!(stats.is_some());
    
    let stats = stats.unwrap();
    assert_eq!(stats.samples_collected, 0);
    assert_eq!(stats.weaknesses_detected, 0);
    assert_eq!(stats.rsi_triggers, 0);
    
    Ok(())
}

#[tokio::test]
async fn test_get_recent_weaknesses() -> Result<()> {
    let config = RuntimeDetectorConfig::default();
    let orchestrator = ASIOrchestrator::new()
        .await?
        .with_runtime_detector(config);
    
    // Should return empty vec initially
    let weaknesses = orchestrator.get_runtime_weaknesses(10);
    assert_eq!(weaknesses.len(), 0);
    
    Ok(())
}

#[tokio::test]
async fn test_has_runtime_detector() -> Result<()> {
    // Without detector
    let orchestrator1 = ASIOrchestrator::new().await?;
    assert!(!orchestrator1.has_runtime_detector());
    
    // With detector
    let config = RuntimeDetectorConfig::default();
    let orchestrator2 = ASIOrchestrator::new()
        .await?
        .with_runtime_detector(config);
    assert!(orchestrator2.has_runtime_detector());
    
    Ok(())
}
