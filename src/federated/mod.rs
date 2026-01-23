//! Federated Multi-Subject Learning System
//!
//! Enables multiple Flux Matrices (subjects) to learn collaboratively
//! through shared sacred geometric space.
//!
//! # Subjects
//!
//! - **Ethical Principles**: Virtue, Duty, Honor, Integrity (Ethos channel)
//! - **Logical Concepts**: Proof, Hypothesis, Axiom, Theorem (Logos channel)
//! - **Emotional Spectrum**: Euphoria, Hope, Ecstasy, Serenity (Pathos channel)
//!
//! # Sacred Coordination
//!
//! All subjects share the sacred triangle (3-6-9) as common ground:
//! - Position 3: Integrity (Ethics) / Axiom (Logic) / Ecstasy (Emotion)
//! - Position 6: Honor (Ethics) / Theorem (Logic) / Despair (Emotion)
//! - Position 9: Virtue (Ethics) / Proof (Logic) / Euphoria (Emotion)
//!
//! # Example
//!
//! ```no_run
//! use spatial_vortex::federated::{FederatedLearner, SubjectDomain};
//!
//! let mut learner = FederatedLearner::new();
//!
//! // Add subjects
//! learner.add_subject(SubjectDomain::Ethics);
//! learner.add_subject(SubjectDomain::Logic);
//! learner.add_subject(SubjectDomain::Emotion);
//!
//! // Federated learning across all subjects
//! learner.federated_train_step(&data)?;
//! ```

pub mod subject_domain;
pub mod federated_learner;
pub mod cross_subject_inference;

// Re-exports
pub use cross_subject_inference::CrossSubjectResult;

pub use subject_domain::{SubjectDomain, SubjectMatrix};
pub use federated_learner::FederatedLearner;
pub use cross_subject_inference::CrossSubjectInference;
