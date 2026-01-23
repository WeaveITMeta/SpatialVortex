//! ASI Core Module - Autonomous Superintelligent System
//!
//! This module implements the missing integration layer that transforms
//! SpatialVortex from a sophisticated reasoning framework into a true
//! autonomous intelligence system.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                      ASI CORE LOOP                              │
//! │                                                                 │
//! │  ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐    │
//! │  │ PERCEIVE │ → │  THINK   │ → │   ACT    │ → │  LEARN   │    │
//! │  │ sensors  │   │ reason   │   │  tools   │   │ improve  │    │
//! │  └────┬─────┘   └────┬─────┘   └────┬─────┘   └────┬─────┘    │
//! │       │              │              │              │           │
//! │       └──────────────┴──────────────┴──────────────┘           │
//! │                          ↓                                     │
//! │                  PERSISTENT MEMORY                             │
//! │                          ↓                                     │
//! │                  SELF-IMPROVEMENT                              │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Key Components
//!
//! - **ASICore**: The main autonomous loop
//! - **WorldInterface**: Sensors and actuators for real-world interaction
//! - **PersistentIdentity**: Long-term memory and self-model
//! - **SelfModification**: Code generation and self-improvement
//! - **GoalManager**: Autonomous goal selection and pursuit

pub mod core;
pub mod world_interface;
pub mod identity;
pub mod goal_manager;
pub mod self_modification;
pub mod bootstrap;
pub mod runtime_detector;
pub mod pattern_recognition;
pub mod task_pattern_tracker;
pub mod pre_production_trainer;
pub mod ai_task_generator;
pub mod consensus_task_validator;
pub mod deep_task_validator;
pub mod rsi_closure;

pub use core::{ASICore, ASIConfig, ASIState, ASIMode};
pub use bootstrap::{ASIBuilder, create_minimal_asi, create_dev_asi, create_full_asi};
pub use world_interface::{
    Sensor, Actuator, Observation, Action, ActionResult,
    FileSystemSensor, ShellActuator, TimeSensor,
};
pub use identity::{PersistentIdentity, SelfModel, EpisodicMemory, KnowledgeEntry};
pub use goal_manager::{GoalManager, AutonomousGoal, GoalPriority, GoalStatus};
pub use self_modification::{SelfModificationEngine, CodePatch, ImprovementProposal};
pub use runtime_detector::{
    RuntimeWeaknessDetector, RuntimeDetectorConfig, RuntimeWeakness, 
    RuntimeWeaknessType, RuntimeStats, RSITriggerEvent,
};
pub use pattern_recognition::{
    PatternRecognitionEngine, PatternType, CycleType, DetectedPattern, PatternEvent,
};
pub use task_pattern_tracker::{
    TaskPatternTracker, TaskTrackerConfig, TaskAttempt, TaskResult, TaskCategory,
    ErrorType, FailurePattern, TaskStats,
};
pub use pre_production_trainer::{
    PreProductionTrainer, TrainingScenario, TrainingMetrics, ValidationBenchmark,
    ProductionReadiness, create_default_scenarios, create_default_benchmarks,
};
pub use ai_task_generator::{
    AITaskGenerator, TaskGenerationStrategy, GeneratedTask, PredictedFailureMode,
    TaskMetadata, TaskGenerationStats,
};
pub use consensus_task_validator::{
    ConsensusTaskValidator, TaskValidationResult, QualityScores,
};
pub use deep_task_validator::{
    DeepTaskValidator, DeepValidationResult, MindMapAnalysis, FluxAnalysis,
    DeepQuality, DiscussionSpace, Perspective,
};
pub use rsi_closure::{
    RSIClosureCoordinator, RSIStatus, GapAnalysis, ComponentStatus, RSILevel,
};
