//! Writing Gate — Gatekeeper Vetting ML Proposals Before Trait Commitment
//!
//! Table of Contents:
//! 1. TraitProposal — A proposed trait update from any ML subsystem
//! 2. GatePolicy — Rules governing what proposals are accepted
//! 3. WritingGate — The gatekeeper that vets all proposals
//! 4. ProposalVerdict — Accept/reject/defer decision with reasoning
//! 5. GateMetrics — Acceptance rates, rejection reasons, throughput
//!
//! Architecture:
//! No ML subsystem (supervised, unsupervised, RL, federated) can directly
//! mutate traits. All outputs are funneled through the WritingGate, which
//! vets proposals against global policies, human feedback thresholds,
//! consistency checks, and RL-learned acceptance criteria. This prevents
//! overfitting by treating deep learning as a proposal engine, not a
//! state overlord.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::storage::trait_ledger::{TraitValue, TraitDelta, ProvenanceSource, TraitLedger};
use crate::ml::rl_actor_critic::{
    QTable, DiscreteAction, PathCurator, EvidenceEvent, EvidenceType,
    StateTransition, TransitionReward,
};

// =============================================================================
// 1. TraitProposal — A proposed trait update from any ML subsystem
// =============================================================================

/// A proposal to update a trait value, submitted by an ML subsystem.
/// Must pass through the WritingGate before commitment to the ledger.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitProposal {
    /// Which trait to update
    pub trait_name: String,
    /// Proposed new value
    pub proposed_value: TraitValue,
    /// Delta from current value (computed by proposer)
    pub delta: Option<TraitDelta>,
    /// Source subsystem that generated this proposal
    pub source: ProvenanceSource,
    /// Confidence of the proposing subsystem
    pub confidence: f64,
    /// Evidence supporting this proposal (event IDs)
    pub supporting_evidence: Vec<u64>,
    /// Human-readable justification
    pub justification: String,
    /// Priority (higher = more urgent)
    pub priority: u32,
    /// Author (subsystem identifier)
    pub author: String,
}

// =============================================================================
// 2. GatePolicy — Rules governing what proposals are accepted
// =============================================================================

/// Policy rules for the writing gate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatePolicy {
    /// Minimum confidence to auto-accept
    pub min_confidence_auto_accept: f64,
    /// Minimum confidence to consider (below = auto-reject)
    pub min_confidence_threshold: f64,
    /// Maximum delta magnitude to auto-accept (larger changes need more scrutiny)
    pub max_auto_accept_magnitude: f64,
    /// Minimum evidence events required for acceptance
    pub min_evidence_count: usize,
    /// Maximum contradiction rate for the trait (above = reject)
    pub max_contradiction_rate: f64,
    /// Whether to use RL-learned acceptance policy
    pub use_rl_policy: bool,
    /// Whether human feedback is required for high-impact changes
    pub require_human_for_high_impact: bool,
    /// High-impact threshold (delta magnitude)
    pub high_impact_threshold: f64,
    /// Source-specific trust levels
    pub source_trust: HashMap<String, f64>,
    /// Cool-down period between writes to same trait (ms)
    pub write_cooldown_ms: u64,
}

impl Default for GatePolicy {
    fn default() -> Self {
        let mut source_trust = HashMap::new();
        source_trust.insert("HumanFeedback".to_string(), 1.0);
        source_trust.insert("SupervisedInit".to_string(), 0.9);
        source_trust.insert("FederatedAggregation".to_string(), 0.8);
        source_trust.insert("ReinforcementLearning".to_string(), 0.7);
        source_trust.insert("UnsupervisedDiscovery".to_string(), 0.6);
        source_trust.insert("StructuredPrediction".to_string(), 0.7);
        source_trust.insert("MetaLearning".to_string(), 0.8);

        Self {
            min_confidence_auto_accept: 0.85,
            min_confidence_threshold: 0.3,
            max_auto_accept_magnitude: 0.5,
            min_evidence_count: 1,
            max_contradiction_rate: 0.2,
            use_rl_policy: true,
            require_human_for_high_impact: false,
            high_impact_threshold: 2.0,
            source_trust,
            write_cooldown_ms: 100,
        }
    }
}

// =============================================================================
// 3. ProposalVerdict — Accept/reject/defer decision with reasoning
// =============================================================================

/// The gate's decision on a proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalVerdict {
    /// Whether the proposal was accepted
    pub accepted: bool,
    /// Decision type
    pub decision: VerdictDecision,
    /// Reasons for the decision
    pub reasons: Vec<String>,
    /// Adjusted confidence (gate may modify confidence)
    pub adjusted_confidence: f64,
    /// Adjusted delta (gate may reduce magnitude)
    pub adjusted_value: Option<TraitValue>,
}

/// Types of verdict decisions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VerdictDecision {
    /// Accepted as-is
    Accept,
    /// Accepted with reduced magnitude
    AcceptReduced,
    /// Rejected: insufficient confidence
    RejectLowConfidence,
    /// Rejected: too high contradiction rate
    RejectContradiction,
    /// Rejected: insufficient evidence
    RejectInsufficientEvidence,
    /// Rejected: cooldown period active
    RejectCooldown,
    /// Rejected: RL policy says no
    RejectByPolicy,
    /// Deferred to human review
    DeferToHuman,
}

// =============================================================================
// 4. WritingGate — The gatekeeper that vets all proposals
// =============================================================================

/// The Writing Gate: all ML proposals must pass through here.
/// Prevents any subsystem from directly mutating traits.
pub struct WritingGate {
    /// Gate policy
    policy: GatePolicy,
    /// Last write timestamps per trait (for cooldown)
    last_write_times: HashMap<String, u64>,
    /// Proposal history (for audit)
    proposal_log: Vec<(TraitProposal, ProposalVerdict)>,
    /// Maximum log size
    max_log_size: usize,
    /// Metrics
    metrics: GateMetrics,
}

impl WritingGate {
    /// Create a new writing gate with default policy
    pub fn new() -> Self {
        Self::with_policy(GatePolicy::default())
    }

    /// Create with custom policy
    pub fn with_policy(policy: GatePolicy) -> Self {
        Self {
            policy,
            last_write_times: HashMap::new(),
            proposal_log: Vec::new(),
            max_log_size: 10000,
            metrics: GateMetrics::new(),
        }
    }

    /// Submit a proposal for vetting. Returns verdict.
    /// If accepted, the caller should commit to the ledger.
    pub fn vet_proposal(
        &mut self,
        proposal: &TraitProposal,
        ledger: &TraitLedger,
        curator: Option<&PathCurator>,
    ) -> ProposalVerdict {
        self.metrics.total_proposals += 1;
        let mut reasons = Vec::new();

        // Check 1: Minimum confidence threshold
        if proposal.confidence < self.policy.min_confidence_threshold {
            let verdict = ProposalVerdict {
                accepted: false,
                decision: VerdictDecision::RejectLowConfidence,
                reasons: vec![format!(
                    "Confidence {:.3} below threshold {:.3}",
                    proposal.confidence, self.policy.min_confidence_threshold
                )],
                adjusted_confidence: proposal.confidence,
                adjusted_value: None,
            };
            self.log_verdict(proposal, &verdict);
            self.metrics.rejected_low_confidence += 1;
            return verdict;
        }

        // Check 2: Contradiction rate for this trait
        let contradiction_rate = ledger.contradiction_rate(&proposal.trait_name);
        if contradiction_rate > self.policy.max_contradiction_rate {
            let verdict = ProposalVerdict {
                accepted: false,
                decision: VerdictDecision::RejectContradiction,
                reasons: vec![format!(
                    "Trait '{}' contradiction rate {:.3} exceeds max {:.3}",
                    proposal.trait_name, contradiction_rate, self.policy.max_contradiction_rate
                )],
                adjusted_confidence: proposal.confidence,
                adjusted_value: None,
            };
            self.log_verdict(proposal, &verdict);
            self.metrics.rejected_contradiction += 1;
            return verdict;
        }

        // Check 3: Evidence count
        if proposal.supporting_evidence.len() < self.policy.min_evidence_count {
            reasons.push(format!(
                "Only {} evidence events (min {})",
                proposal.supporting_evidence.len(), self.policy.min_evidence_count
            ));
            // Don't reject yet — may still pass on confidence
        }

        // Check 4: Cooldown
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        if let Some(&last_write) = self.last_write_times.get(&proposal.trait_name) {
            if now_ms - last_write < self.policy.write_cooldown_ms {
                let verdict = ProposalVerdict {
                    accepted: false,
                    decision: VerdictDecision::RejectCooldown,
                    reasons: vec![format!(
                        "Cooldown active for trait '{}' ({}ms remaining)",
                        proposal.trait_name,
                        self.policy.write_cooldown_ms - (now_ms - last_write)
                    )],
                    adjusted_confidence: proposal.confidence,
                    adjusted_value: None,
                };
                self.log_verdict(proposal, &verdict);
                self.metrics.rejected_cooldown += 1;
                return verdict;
            }
        }

        // Check 5: Delta magnitude
        let magnitude = proposal.delta.as_ref()
            .map(|d| d.magnitude())
            .unwrap_or(0.0);

        // Check 6: High-impact changes may need human review
        if self.policy.require_human_for_high_impact && magnitude > self.policy.high_impact_threshold {
            let verdict = ProposalVerdict {
                accepted: false,
                decision: VerdictDecision::DeferToHuman,
                reasons: vec![format!(
                    "High-impact change (magnitude {:.3} > threshold {:.3}) requires human review",
                    magnitude, self.policy.high_impact_threshold
                )],
                adjusted_confidence: proposal.confidence,
                adjusted_value: None,
            };
            self.log_verdict(proposal, &verdict);
            self.metrics.deferred_to_human += 1;
            return verdict;
        }

        // Check 7: RL-learned acceptance policy
        if self.policy.use_rl_policy {
            if let Some(curator) = curator {
                let state = self.proposal_to_state(proposal, magnitude, contradiction_rate);
                let action = curator.q_table.select_action(QTable::hash_state(&state, 0.1));
                match action {
                    DiscreteAction::Reject => {
                        let verdict = ProposalVerdict {
                            accepted: false,
                            decision: VerdictDecision::RejectByPolicy,
                            reasons: vec!["RL policy rejected this proposal".to_string()],
                            adjusted_confidence: proposal.confidence,
                            adjusted_value: None,
                        };
                        self.log_verdict(proposal, &verdict);
                        self.metrics.rejected_by_policy += 1;
                        return verdict;
                    }
                    DiscreteAction::AcceptReduced => {
                        // Accept with halved magnitude
                        let reduced = self.reduce_magnitude(&proposal.proposed_value, 0.5);
                        reasons.push("RL policy: accepted with reduced magnitude".to_string());
                        let verdict = ProposalVerdict {
                            accepted: true,
                            decision: VerdictDecision::AcceptReduced,
                            reasons,
                            adjusted_confidence: proposal.confidence * 0.8,
                            adjusted_value: Some(reduced),
                        };
                        self.log_verdict(proposal, &verdict);
                        self.last_write_times.insert(proposal.trait_name.clone(), now_ms);
                        self.metrics.accepted_reduced += 1;
                        return verdict;
                    }
                    DiscreteAction::DeferToHuman => {
                        let verdict = ProposalVerdict {
                            accepted: false,
                            decision: VerdictDecision::DeferToHuman,
                            reasons: vec!["RL policy deferred to human".to_string()],
                            adjusted_confidence: proposal.confidence,
                            adjusted_value: None,
                        };
                        self.log_verdict(proposal, &verdict);
                        self.metrics.deferred_to_human += 1;
                        return verdict;
                    }
                    DiscreteAction::TriggerRollback => {
                        let verdict = ProposalVerdict {
                            accepted: false,
                            decision: VerdictDecision::RejectByPolicy,
                            reasons: vec!["RL policy triggered rollback instead".to_string()],
                            adjusted_confidence: proposal.confidence,
                            adjusted_value: None,
                        };
                        self.log_verdict(proposal, &verdict);
                        self.metrics.rejected_by_policy += 1;
                        return verdict;
                    }
                    DiscreteAction::Accept => {
                        // Fall through to auto-accept logic
                    }
                }
            }
        }

        // Check 8: Source trust adjustment
        let source_key = format!("{:?}", proposal.source);
        let trust = self.policy.source_trust.get(&source_key).copied().unwrap_or(0.5);
        let adjusted_confidence = proposal.confidence * trust;

        // Final decision: auto-accept if confidence and magnitude pass
        if adjusted_confidence >= self.policy.min_confidence_auto_accept
            && magnitude <= self.policy.max_auto_accept_magnitude
        {
            reasons.push(format!("Auto-accepted: confidence {:.3}, magnitude {:.3}", adjusted_confidence, magnitude));
            let verdict = ProposalVerdict {
                accepted: true,
                decision: VerdictDecision::Accept,
                reasons,
                adjusted_confidence,
                adjusted_value: None,
            };
            self.log_verdict(proposal, &verdict);
            self.last_write_times.insert(proposal.trait_name.clone(), now_ms);
            self.metrics.accepted += 1;
            return verdict;
        }

        // Accept with reduced magnitude for medium-confidence proposals
        if adjusted_confidence >= self.policy.min_confidence_threshold {
            let reduction = (adjusted_confidence / self.policy.min_confidence_auto_accept).min(1.0);
            let reduced = self.reduce_magnitude(&proposal.proposed_value, reduction);
            reasons.push(format!(
                "Accepted with reduction factor {:.3} (confidence {:.3})",
                reduction, adjusted_confidence
            ));
            let verdict = ProposalVerdict {
                accepted: true,
                decision: VerdictDecision::AcceptReduced,
                reasons,
                adjusted_confidence,
                adjusted_value: Some(reduced),
            };
            self.log_verdict(proposal, &verdict);
            self.last_write_times.insert(proposal.trait_name.clone(), now_ms);
            self.metrics.accepted_reduced += 1;
            return verdict;
        }

        // Should not reach here, but reject as fallback
        let verdict = ProposalVerdict {
            accepted: false,
            decision: VerdictDecision::RejectLowConfidence,
            reasons: vec!["Fallback rejection".to_string()],
            adjusted_confidence,
            adjusted_value: None,
        };
        self.log_verdict(proposal, &verdict);
        self.metrics.rejected_low_confidence += 1;
        verdict
    }

    /// Submit proposal and auto-commit to ledger if accepted
    pub fn submit_and_commit(
        &mut self,
        proposal: TraitProposal,
        ledger: &mut TraitLedger,
        curator: Option<&PathCurator>,
    ) -> ProposalVerdict {
        let verdict = self.vet_proposal(&proposal, ledger, curator);

        if verdict.accepted {
            let value = verdict.adjusted_value.clone()
                .unwrap_or(proposal.proposed_value.clone());
            ledger.write_trait(
                &proposal.trait_name,
                value,
                &proposal.author,
                &proposal.justification,
                ProvenanceSource::WritingGate,
            );
        }

        verdict
    }

    /// Convert proposal to state vector for RL
    fn proposal_to_state(&self, proposal: &TraitProposal, magnitude: f64, contradiction_rate: f64) -> Vec<f64> {
        vec![
            proposal.confidence,
            magnitude,
            contradiction_rate,
            proposal.supporting_evidence.len() as f64 / 10.0,
            proposal.priority as f64 / 10.0,
        ]
    }

    /// Reduce magnitude of a trait value by a factor
    fn reduce_magnitude(&self, value: &TraitValue, factor: f64) -> TraitValue {
        match value {
            TraitValue::Scalar(v) => TraitValue::Scalar(v * factor),
            TraitValue::Vector(v) => TraitValue::Vector(v.iter().map(|x| x * factor).collect()),
            other => other.clone(),
        }
    }

    /// Log a verdict for audit
    fn log_verdict(&mut self, proposal: &TraitProposal, verdict: &ProposalVerdict) {
        if self.proposal_log.len() >= self.max_log_size {
            self.proposal_log.remove(0);
        }
        self.proposal_log.push((proposal.clone(), verdict.clone()));
    }

    /// Get gate metrics
    pub fn metrics(&self) -> &GateMetrics {
        &self.metrics
    }

    /// Get recent proposal log
    pub fn recent_proposals(&self, n: usize) -> &[(TraitProposal, ProposalVerdict)] {
        let start = self.proposal_log.len().saturating_sub(n);
        &self.proposal_log[start..]
    }

    /// Get acceptance rate
    pub fn acceptance_rate(&self) -> f64 {
        if self.metrics.total_proposals == 0 {
            return 0.0;
        }
        (self.metrics.accepted + self.metrics.accepted_reduced) as f64
            / self.metrics.total_proposals as f64
    }
}

// =============================================================================
// 5. GateMetrics — Acceptance rates, rejection reasons, throughput
// =============================================================================

/// Metrics tracking the writing gate's behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateMetrics {
    /// Total proposals received
    pub total_proposals: u64,
    /// Accepted as-is
    pub accepted: u64,
    /// Accepted with reduced magnitude
    pub accepted_reduced: u64,
    /// Rejected: low confidence
    pub rejected_low_confidence: u64,
    /// Rejected: contradiction rate
    pub rejected_contradiction: u64,
    /// Rejected: insufficient evidence
    pub rejected_insufficient_evidence: u64,
    /// Rejected: cooldown
    pub rejected_cooldown: u64,
    /// Rejected: RL policy
    pub rejected_by_policy: u64,
    /// Deferred to human
    pub deferred_to_human: u64,
}

impl GateMetrics {
    pub fn new() -> Self {
        Self {
            total_proposals: 0,
            accepted: 0,
            accepted_reduced: 0,
            rejected_low_confidence: 0,
            rejected_contradiction: 0,
            rejected_insufficient_evidence: 0,
            rejected_cooldown: 0,
            rejected_by_policy: 0,
            deferred_to_human: 0,
        }
    }

    /// Total rejections
    pub fn total_rejections(&self) -> u64 {
        self.rejected_low_confidence
            + self.rejected_contradiction
            + self.rejected_insufficient_evidence
            + self.rejected_cooldown
            + self.rejected_by_policy
    }
}
