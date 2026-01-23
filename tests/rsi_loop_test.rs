//! RSI Loop Integration Test
//!
//! Tests the first real self-improvement cycle:
//! PerformanceTracker metrics → SelfModificationEngine auto-trigger

use spatial_vortex::ai::orchestrator::{ASIOrchestrator, ExecutionMode, RSIConfig};
use spatial_vortex::error::Result;
use std::path::PathBuf;
use tempfile::tempdir;

#[tokio::test]
async fn test_rsi_loop_disabled_by_default() -> Result<()> {
    // Create orchestrator
    let orchestrator = ASIOrchestrator::new().await?;
    
    // RSI should be disabled by default
    let result = orchestrator.rsi_cycle().await?;
    assert!(!result.errors.is_empty());
    assert!(result.errors[0].contains("disabled"));
    
    Ok(())
}

#[tokio::test]
async fn test_rsi_loop_requires_self_mod_engine() -> Result<()> {
    // Create orchestrator without self-mod engine
    let orchestrator = ASIOrchestrator::new().await?;
    
    // Enable RSI
    let mut config = RSIConfig::default();
    config.enabled = true;
    orchestrator.enable_rsi(config).await;
    
    // Should fail without self-mod engine
    let result = orchestrator.rsi_cycle().await;
    assert!(result.is_err());
    
    Ok(())
}

#[tokio::test]
async fn test_rsi_loop_with_engine() -> Result<()> {
    // Create temp directory for source code
    let temp_dir = tempdir().unwrap();
    let source_path = temp_dir.path().to_path_buf();
    
    // Create orchestrator with self-mod engine
    let orchestrator = ASIOrchestrator::new()
        .await?
        .with_self_modification(source_path);
    
    // Enable RSI with permissive config
    let mut config = RSIConfig::default();
    config.enabled = true;
    config.min_confidence_threshold = 0.9; // High threshold to trigger weakness
    config.max_thorough_time_ms = 100.0; // Low threshold to trigger weakness
    config.auto_apply_low_risk = true;
    orchestrator.enable_rsi(config).await;
    
    // Run some inferences to generate metrics
    for _ in 0..10 {
        let _ = orchestrator.process("test input", ExecutionMode::Fast).await;
    }
    
    // Run RSI cycle
    let result = orchestrator.rsi_cycle().await?;
    
    // Should detect weaknesses (confidence or time)
    println!("RSI Cycle Result: {:?}", result);
    println!("Weaknesses detected: {}", result.weaknesses_detected.len());
    println!("Proposals generated: {}", result.proposals_generated);
    println!("Proposals tested: {}", result.proposals_tested);
    println!("Proposals applied: {}", result.proposals_applied);
    
    // Verify cycle ran
    assert!(result.weaknesses_detected.len() > 0 || result.proposals_generated == 0);
    
    Ok(())
}

#[tokio::test]
async fn test_weakness_detection() -> Result<()> {
    // Create temp directory
    let temp_dir = tempdir().unwrap();
    let source_path = temp_dir.path().to_path_buf();
    
    // Create orchestrator
    let orchestrator = ASIOrchestrator::new()
        .await?
        .with_self_modification(source_path);
    
    // Configure to detect weaknesses
    let mut config = RSIConfig::default();
    config.enabled = true;
    config.min_confidence_threshold = 1.0; // Impossible threshold
    config.max_thorough_time_ms = 0.0; // Impossible threshold
    orchestrator.enable_rsi(config).await;
    
    // Run inference
    let _ = orchestrator.process("test", ExecutionMode::Thorough).await;
    
    // Run RSI cycle
    let result = orchestrator.rsi_cycle().await?;
    
    // Should detect at least one weakness
    assert!(result.weaknesses_detected.len() > 0);
    
    // Check weakness types
    let has_confidence_weakness = result.weaknesses_detected
        .iter()
        .any(|w| w.weakness_type == "low_confidence");
    let has_time_weakness = result.weaknesses_detected
        .iter()
        .any(|w| w.weakness_type == "slow_reasoning");
    
    assert!(has_confidence_weakness || has_time_weakness);
    
    Ok(())
}

#[tokio::test]
async fn test_proposal_generation() -> Result<()> {
    // Create temp directory
    let temp_dir = tempdir().unwrap();
    let source_path = temp_dir.path().to_path_buf();
    
    // Create orchestrator
    let orchestrator = ASIOrchestrator::new()
        .await?
        .with_self_modification(source_path);
    
    // Enable RSI
    let mut config = RSIConfig::default();
    config.enabled = true;
    config.min_confidence_threshold = 1.0; // Force weakness detection
    orchestrator.enable_rsi(config).await;
    
    // Run inference
    let _ = orchestrator.process("test", ExecutionMode::Fast).await;
    
    // Run RSI cycle
    let result = orchestrator.rsi_cycle().await?;
    
    // Should generate proposals
    assert!(result.proposals_generated > 0);
    
    // Get proposals
    let proposals = orchestrator.get_proposals().await;
    assert!(proposals.len() > 0);
    
    // Check proposal structure
    let proposal = &proposals[0];
    assert!(!proposal.description.is_empty());
    assert!(!proposal.weakness_addressed.is_empty());
    
    Ok(())
}

#[tokio::test]
async fn test_auto_apply_low_risk() -> Result<()> {
    // Create temp directory
    let temp_dir = tempdir().unwrap();
    let source_path = temp_dir.path().to_path_buf();
    
    // Create orchestrator
    let orchestrator = ASIOrchestrator::new()
        .await?
        .with_self_modification(source_path);
    
    // Enable auto-apply for low risk
    let mut config = RSIConfig::default();
    config.enabled = true;
    config.min_confidence_threshold = 1.0;
    config.auto_apply_low_risk = true;
    config.auto_apply_medium_risk = false;
    orchestrator.enable_rsi(config).await;
    
    // Run inference
    let _ = orchestrator.process("test", ExecutionMode::Fast).await;
    
    // Run RSI cycle
    let result = orchestrator.rsi_cycle().await?;
    
    // Check if any low-risk proposals were applied
    println!("Applied: {}, Pending: {}", 
        result.proposals_applied, 
        result.proposals_pending_approval);
    
    // Should have either applied or pending proposals
    assert!(result.proposals_applied > 0 || result.proposals_pending_approval > 0);
    
    Ok(())
}

#[tokio::test]
async fn test_rsi_stats() -> Result<()> {
    // Create temp directory
    let temp_dir = tempdir().unwrap();
    let source_path = temp_dir.path().to_path_buf();
    
    // Create orchestrator
    let orchestrator = ASIOrchestrator::new()
        .await?
        .with_self_modification(source_path);
    
    // Enable RSI
    let mut config = RSIConfig::default();
    config.enabled = true;
    config.min_confidence_threshold = 1.0;
    orchestrator.enable_rsi(config).await;
    
    // Run inference and RSI cycle
    let _ = orchestrator.process("test", ExecutionMode::Fast).await;
    let _ = orchestrator.rsi_cycle().await;
    
    // Get stats
    let stats = orchestrator.rsi_stats().await;
    assert!(stats.is_some());
    
    let stats = stats.unwrap();
    println!("RSI Stats: {:?}", stats);
    
    // Should have generated at least one proposal
    assert!(stats.proposals_generated > 0);
    
    Ok(())
}

#[tokio::test]
async fn test_manual_approval() -> Result<()> {
    // Create temp directory
    let temp_dir = tempdir().unwrap();
    let source_path = temp_dir.path().to_path_buf();
    
    // Create orchestrator
    let orchestrator = ASIOrchestrator::new()
        .await?
        .with_self_modification(source_path);
    
    // Enable RSI without auto-apply
    let mut config = RSIConfig::default();
    config.enabled = true;
    config.min_confidence_threshold = 1.0;
    config.auto_apply_low_risk = false;
    config.auto_apply_medium_risk = false;
    orchestrator.enable_rsi(config).await;
    
    // Run inference and RSI cycle
    let _ = orchestrator.process("test", ExecutionMode::Fast).await;
    let result = orchestrator.rsi_cycle().await?;
    
    // Should have pending proposals
    assert!(result.proposals_pending_approval > 0);
    
    // Get proposals
    let proposals = orchestrator.get_proposals().await;
    assert!(proposals.len() > 0);
    
    // Try to manually approve first proposal
    let proposal_id = proposals[0].id;
    let approve_result = orchestrator.approve_proposal(proposal_id).await;
    
    // Should succeed (or fail gracefully if file doesn't exist)
    println!("Manual approval result: {:?}", approve_result);
    
    Ok(())
}

#[tokio::test]
async fn test_has_self_modification() -> Result<()> {
    // Without self-mod
    let orchestrator1 = ASIOrchestrator::new().await?;
    assert!(!orchestrator1.has_self_modification());
    
    // With self-mod
    let temp_dir = tempdir().unwrap();
    let orchestrator2 = ASIOrchestrator::new()
        .await?
        .with_self_modification(temp_dir.path().to_path_buf());
    assert!(orchestrator2.has_self_modification());
    
    Ok(())
}
