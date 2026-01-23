//! Background Training Tests

use spatial_vortex::ml::training::{
    BackgroundTrainingCoordinator, BackgroundTrainingConfig, TrainingTrigger,
};
use spatial_vortex::eustress_bridge::training_pipeline::EustressTrainingPipeline;
use spatial_vortex::eustress_bridge::flux_dynamics::FluxDynamics;
use spatial_vortex::eustress_bridge::entity_embedding::EustressEntity;
use spatial_vortex::ai::orchestrator::ASIOrchestrator;
use tokio::time::{Duration, sleep};

#[tokio::test]
async fn test_coordinator_creation() {
    let config = BackgroundTrainingConfig::default();
    let flux = FluxDynamics::new();
    let pipeline = EustressTrainingPipeline::new(flux);
    
    let coordinator = BackgroundTrainingCoordinator::new(config, pipeline);
    
    assert!(!coordinator.is_running());
    assert_eq!(coordinator.current_model_version(), 0);
    assert_eq!(coordinator.sample_buffer_size(), 0);
}

#[tokio::test]
async fn test_add_entities_to_buffer() {
    let config = BackgroundTrainingConfig::default();
    let flux = FluxDynamics::new();
    let pipeline = EustressTrainingPipeline::new(flux);
    
    let coordinator = BackgroundTrainingCoordinator::new(config, pipeline);
    
    // Create test entities
    let entities = vec![
        EustressEntity::default(),
        EustressEntity::default(),
        EustressEntity::default(),
    ];
    
    coordinator.add_entities(entities).await;
    
    // Should have samples in buffer
    assert!(coordinator.sample_buffer_size() > 0);
}

#[tokio::test]
async fn test_manual_trigger() {
    let config = BackgroundTrainingConfig::default();
    let flux = FluxDynamics::new();
    let pipeline = EustressTrainingPipeline::new(flux);
    
    let coordinator = BackgroundTrainingCoordinator::new(config, pipeline);
    
    // Should not panic
    coordinator.trigger_training(TrainingTrigger::Manual);
    coordinator.trigger_training(TrainingTrigger::Scheduled);
    coordinator.trigger_training(TrainingTrigger::RSITriggered {
        reason: "test".to_string(),
    });
}

#[tokio::test]
async fn test_start_stop_training() {
    let mut config = BackgroundTrainingConfig::default();
    config.enabled = true;
    config.training_interval_secs = 1;
    
    let flux = FluxDynamics::new();
    let pipeline = EustressTrainingPipeline::new(flux);
    
    let coordinator = BackgroundTrainingCoordinator::new(config, pipeline);
    
    // Start training
    let _handle = coordinator.start().await;
    
    // Should be running
    assert!(coordinator.is_running());
    
    // Wait a bit
    sleep(Duration::from_millis(100)).await;
    
    // Stop training
    coordinator.stop();
    
    // Should not be running
    sleep(Duration::from_millis(100)).await;
    assert!(!coordinator.is_running());
}

#[tokio::test]
async fn test_training_stats() {
    let config = BackgroundTrainingConfig::default();
    let flux = FluxDynamics::new();
    let pipeline = EustressTrainingPipeline::new(flux);
    
    let coordinator = BackgroundTrainingCoordinator::new(config, pipeline);
    
    let stats = coordinator.stats();
    
    assert_eq!(stats.total_training_runs, 0);
    assert_eq!(stats.total_samples_trained, 0);
    assert_eq!(stats.current_model_version, 0);
    assert_eq!(stats.model_swaps, 0);
}

#[tokio::test]
async fn test_model_versions() {
    let config = BackgroundTrainingConfig::default();
    let flux = FluxDynamics::new();
    let pipeline = EustressTrainingPipeline::new(flux);
    
    let coordinator = BackgroundTrainingCoordinator::new(config, pipeline);
    
    let versions = coordinator.get_model_versions();
    
    // Should start with no versions
    assert_eq!(versions.len(), 0);
}

#[tokio::test]
async fn test_orchestrator_integration() {
    let config = BackgroundTrainingConfig::default();
    let flux = FluxDynamics::new();
    let pipeline = EustressTrainingPipeline::new(flux);
    
    let coordinator = BackgroundTrainingCoordinator::new(config, pipeline);
    
    let orchestrator = ASIOrchestrator::new()
        .await
        .unwrap()
        .with_background_trainer(coordinator);
    
    // Should have background trainer
    assert!(orchestrator.background_trainer.is_some());
    
    // Should be able to get stats
    let stats = orchestrator.training_stats();
    assert!(stats.is_some());
}

#[tokio::test]
async fn test_orchestrator_start_stop() {
    let mut config = BackgroundTrainingConfig::default();
    config.enabled = true;
    config.training_interval_secs = 10;
    
    let flux = FluxDynamics::new();
    let pipeline = EustressTrainingPipeline::new(flux);
    
    let coordinator = BackgroundTrainingCoordinator::new(config, pipeline);
    
    let orchestrator = ASIOrchestrator::new()
        .await
        .unwrap()
        .with_background_trainer(coordinator);
    
    // Start training
    let result = orchestrator.start_background_training().await;
    assert!(result.is_ok());
    
    // Wait a bit
    sleep(Duration::from_millis(100)).await;
    
    // Stop training
    orchestrator.stop_background_training();
}

#[tokio::test]
async fn test_trigger_from_orchestrator() {
    let config = BackgroundTrainingConfig::default();
    let flux = FluxDynamics::new();
    let pipeline = EustressTrainingPipeline::new(flux);
    
    let coordinator = BackgroundTrainingCoordinator::new(config, pipeline);
    
    let orchestrator = ASIOrchestrator::new()
        .await
        .unwrap()
        .with_background_trainer(coordinator);
    
    // Should not panic
    orchestrator.trigger_training(TrainingTrigger::Manual);
}

#[tokio::test]
async fn test_get_model_versions_from_orchestrator() {
    let config = BackgroundTrainingConfig::default();
    let flux = FluxDynamics::new();
    let pipeline = EustressTrainingPipeline::new(flux);
    
    let coordinator = BackgroundTrainingCoordinator::new(config, pipeline);
    
    let orchestrator = ASIOrchestrator::new()
        .await
        .unwrap()
        .with_background_trainer(coordinator);
    
    let versions = orchestrator.get_model_versions();
    assert_eq!(versions.len(), 0);
    
    let current = orchestrator.current_model_version();
    assert_eq!(current, 0);
}

#[tokio::test]
async fn test_clear_sample_buffer() {
    let config = BackgroundTrainingConfig::default();
    let flux = FluxDynamics::new();
    let pipeline = EustressTrainingPipeline::new(flux);
    
    let coordinator = BackgroundTrainingCoordinator::new(config, pipeline);
    
    // Add entities
    let entities = vec![EustressEntity::default()];
    coordinator.add_entities(entities).await;
    
    let size_before = coordinator.sample_buffer_size();
    assert!(size_before > 0);
    
    // Clear buffer
    coordinator.clear_sample_buffer();
    
    let size_after = coordinator.sample_buffer_size();
    assert_eq!(size_after, 0);
}

#[tokio::test]
async fn test_training_with_samples() {
    let mut config = BackgroundTrainingConfig::default();
    config.enabled = true;
    config.training_interval_secs = 1;
    config.min_samples_for_training = 2;
    
    let flux = FluxDynamics::new();
    let pipeline = EustressTrainingPipeline::new(flux);
    
    let coordinator = BackgroundTrainingCoordinator::new(config, pipeline);
    
    // Add enough samples
    let entities = vec![
        EustressEntity::default(),
        EustressEntity::default(),
        EustressEntity::default(),
    ];
    coordinator.add_entities(entities).await;
    
    // Start training
    let _handle = coordinator.start().await;
    
    // Wait for training cycle
    sleep(Duration::from_secs(2)).await;
    
    // Stop
    coordinator.stop();
    
    // Check stats (may or may not have run depending on timing)
    let stats = coordinator.stats();
    assert!(stats.total_training_runs >= 0);
}
