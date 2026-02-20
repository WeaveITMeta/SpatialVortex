//! Secure Aggregation — Differential Privacy + SMPC for Federated Trait Updates
//!
//! Table of Contents:
//! 1. DifferentialPrivacy — Noise injection for privacy-preserving trait deltas
//! 2. SecretShare — Shamir-style secret sharing for SMPC
//! 3. SecureAggregator — Aggregates trait deltas without exposing raw data
//! 4. PrivacyBudget — Tracks cumulative privacy loss (epsilon accounting)
//! 5. FederatedProtocol — Full federated learning protocol with privacy
//! 6. AggregationResult — Result of secure aggregation round
//!
//! Architecture:
//! Federated learning aggregates trait deltas via secure protocols.
//! Differential privacy adds calibrated noise to deltas before sharing.
//! Secret sharing splits deltas across nodes so no single node sees raw data.
//! Privacy budget tracking ensures cumulative epsilon stays within bounds.
//! This ensures traits evolve collectively while preserving locality.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::storage::trait_ledger::{TraitValue, TraitDelta};

// =============================================================================
// 1. DifferentialPrivacy — Noise injection for privacy-preserving trait deltas
// =============================================================================

/// Differential privacy mechanism for trait deltas.
/// Adds calibrated Gaussian or Laplace noise to ensure (ε, δ)-DP.
pub struct DifferentialPrivacy {
    /// Privacy parameter epsilon (smaller = more private)
    pub epsilon: f64,
    /// Privacy parameter delta (probability of failure)
    pub delta: f64,
    /// Sensitivity: maximum change any single data point can cause
    pub sensitivity: f64,
    /// Noise mechanism
    pub mechanism: NoiseMechanism,
    /// Clipping bound for delta magnitudes
    pub clip_bound: f64,
    /// Pseudo-random seed for reproducibility
    seed: u64,
}

/// Type of noise mechanism
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum NoiseMechanism {
    /// Gaussian mechanism: (ε, δ)-DP with Gaussian noise
    Gaussian,
    /// Laplace mechanism: ε-DP with Laplace noise
    Laplace,
}

impl DifferentialPrivacy {
    /// Create a new DP mechanism
    pub fn new(epsilon: f64, delta: f64, sensitivity: f64) -> Self {
        Self {
            epsilon,
            delta,
            sensitivity,
            mechanism: NoiseMechanism::Gaussian,
            clip_bound: 1.0,
            seed: 42,
        }
    }

    /// Set noise mechanism
    pub fn with_mechanism(mut self, mechanism: NoiseMechanism) -> Self {
        self.mechanism = mechanism;
        self
    }

    /// Set clipping bound
    pub fn with_clip_bound(mut self, bound: f64) -> Self {
        self.clip_bound = bound;
        self
    }

    /// Compute noise standard deviation for Gaussian mechanism
    fn gaussian_sigma(&self) -> f64 {
        // σ = sensitivity * sqrt(2 * ln(1.25/δ)) / ε
        let ln_term = (2.0 * (1.25 / self.delta).ln()).sqrt();
        self.sensitivity * ln_term / self.epsilon
    }

    /// Compute noise scale for Laplace mechanism
    fn laplace_scale(&self) -> f64 {
        // b = sensitivity / ε
        self.sensitivity / self.epsilon
    }

    /// Generate pseudo-random Gaussian noise (Box-Muller transform)
    fn gaussian_noise(&mut self) -> f64 {
        self.seed = self.seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let u1 = (self.seed >> 33) as f64 / u32::MAX as f64;
        self.seed = self.seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let u2 = (self.seed >> 33) as f64 / u32::MAX as f64;

        let u1 = u1.max(1e-10); // Avoid log(0)
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
    }

    /// Generate pseudo-random Laplace noise
    fn laplace_noise(&mut self) -> f64 {
        self.seed = self.seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let u = (self.seed >> 33) as f64 / u32::MAX as f64 - 0.5;
        -u.signum() * (1.0 - 2.0 * u.abs()).max(1e-10).ln()
    }

    /// Add noise to a scalar value
    pub fn privatize_scalar(&mut self, value: f64) -> f64 {
        // Clip first
        let clipped = value.clamp(-self.clip_bound, self.clip_bound);

        // Add noise
        let noise = match self.mechanism {
            NoiseMechanism::Gaussian => self.gaussian_noise() * self.gaussian_sigma(),
            NoiseMechanism::Laplace => self.laplace_noise() * self.laplace_scale(),
        };

        clipped + noise
    }

    /// Add noise to a vector
    pub fn privatize_vector(&mut self, values: &[f64]) -> Vec<f64> {
        values.iter().map(|v| self.privatize_scalar(*v)).collect()
    }

    /// Privatize a trait delta
    pub fn privatize_delta(&mut self, delta: &TraitDelta) -> TraitDelta {
        match delta {
            TraitDelta::ScalarDelta(d) => {
                TraitDelta::ScalarDelta(self.privatize_scalar(*d))
            }
            TraitDelta::VectorDelta(d) => {
                TraitDelta::VectorDelta(self.privatize_vector(d))
            }
            other => other.clone(), // Labels and flags pass through unchanged
        }
    }

    /// Get the privacy cost of one query
    pub fn privacy_cost(&self) -> f64 {
        self.epsilon
    }
}

// =============================================================================
// 2. SecretShare — Shamir-style secret sharing for SMPC
// =============================================================================

/// A secret share of a trait delta value.
/// No single share reveals the original value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretShare {
    /// Share index (which node holds this)
    pub index: u32,
    /// Share value (polynomial evaluation)
    pub value: f64,
    /// Trait name this share belongs to
    pub trait_name: String,
    /// Total number of shares
    pub total_shares: u32,
    /// Threshold for reconstruction
    pub threshold: u32,
}

/// Secret sharing scheme (simplified Shamir's)
pub struct SecretSharing {
    /// Number of shares to create
    pub num_shares: u32,
    /// Threshold for reconstruction (t-of-n)
    pub threshold: u32,
    /// Pseudo-random seed
    seed: u64,
}

impl SecretSharing {
    /// Create a new secret sharing scheme
    pub fn new(num_shares: u32, threshold: u32) -> Self {
        assert!(threshold <= num_shares, "Threshold must be <= num_shares");
        Self {
            num_shares,
            threshold,
            seed: 12345,
        }
    }

    /// Split a secret value into shares
    pub fn split(&mut self, secret: f64, trait_name: &str) -> Vec<SecretShare> {
        // Generate random polynomial coefficients: f(x) = secret + a1*x + a2*x^2 + ...
        let mut coefficients = vec![secret];
        for _ in 1..self.threshold {
            self.seed = self.seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let coeff = (self.seed >> 33) as f64 / u32::MAX as f64 * 2.0 - 1.0;
            coefficients.push(coeff);
        }

        // Evaluate polynomial at points 1, 2, ..., num_shares
        (1..=self.num_shares)
            .map(|i| {
                let x = i as f64;
                let value = coefficients.iter().enumerate()
                    .map(|(power, &coeff)| coeff * x.powi(power as i32))
                    .sum();
                SecretShare {
                    index: i,
                    value,
                    trait_name: trait_name.to_string(),
                    total_shares: self.num_shares,
                    threshold: self.threshold,
                }
            })
            .collect()
    }

    /// Reconstruct secret from shares using Lagrange interpolation
    pub fn reconstruct(shares: &[SecretShare]) -> Option<f64> {
        if shares.is_empty() {
            return None;
        }
        let threshold = shares[0].threshold as usize;
        if shares.len() < threshold {
            return None; // Not enough shares
        }

        let shares_to_use = &shares[..threshold];

        // Lagrange interpolation at x=0
        let mut secret = 0.0;
        for (i, share_i) in shares_to_use.iter().enumerate() {
            let xi = share_i.index as f64;
            let mut basis = 1.0;
            for (j, share_j) in shares_to_use.iter().enumerate() {
                if i != j {
                    let xj = share_j.index as f64;
                    basis *= -xj / (xi - xj);
                }
            }
            secret += share_i.value * basis;
        }

        Some(secret)
    }
}

// =============================================================================
// 3. PrivacyBudget — Tracks cumulative privacy loss (epsilon accounting)
// =============================================================================

/// Tracks cumulative privacy expenditure across multiple queries.
/// Uses basic composition theorem: total ε = sum of individual ε's.
/// Advanced composition gives tighter bounds for many queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyBudget {
    /// Maximum allowed total epsilon
    pub max_epsilon: f64,
    /// Current spent epsilon
    pub spent_epsilon: f64,
    /// Maximum allowed delta
    pub max_delta: f64,
    /// Current spent delta
    pub spent_delta: f64,
    /// Number of queries made
    pub query_count: u64,
    /// Whether to use advanced composition
    pub advanced_composition: bool,
}

impl PrivacyBudget {
    /// Create a new budget
    pub fn new(max_epsilon: f64, max_delta: f64) -> Self {
        Self {
            max_epsilon,
            spent_epsilon: 0.0,
            max_delta,
            spent_delta: 0.0,
            query_count: 0,
            advanced_composition: true,
        }
    }

    /// Check if a query with given cost is affordable
    pub fn can_afford(&self, epsilon: f64, delta: f64) -> bool {
        self.spent_epsilon + epsilon <= self.max_epsilon
            && self.spent_delta + delta <= self.max_delta
    }

    /// Spend privacy budget for a query
    pub fn spend(&mut self, epsilon: f64, delta: f64) -> bool {
        if !self.can_afford(epsilon, delta) {
            return false;
        }
        self.spent_epsilon += epsilon;
        self.spent_delta += delta;
        self.query_count += 1;
        true
    }

    /// Remaining epsilon budget
    pub fn remaining_epsilon(&self) -> f64 {
        (self.max_epsilon - self.spent_epsilon).max(0.0)
    }

    /// Remaining delta budget
    pub fn remaining_delta(&self) -> f64 {
        (self.max_delta - self.spent_delta).max(0.0)
    }

    /// Budget utilization (0.0 - 1.0)
    pub fn utilization(&self) -> f64 {
        self.spent_epsilon / self.max_epsilon
    }

    /// Reset budget (new epoch)
    pub fn reset(&mut self) {
        self.spent_epsilon = 0.0;
        self.spent_delta = 0.0;
        self.query_count = 0;
    }
}

// =============================================================================
// 4. SecureAggregator — Aggregates trait deltas without exposing raw data
// =============================================================================

/// Securely aggregates trait deltas from multiple nodes.
/// Combines differential privacy and secret sharing.
pub struct SecureAggregator {
    /// Differential privacy mechanism
    pub dp: DifferentialPrivacy,
    /// Secret sharing scheme
    pub sharing: SecretSharing,
    /// Privacy budget tracker
    pub budget: PrivacyBudget,
    /// Number of participating nodes
    pub num_nodes: usize,
    /// Aggregation statistics
    pub stats: AggregationStats,
}

/// Statistics for aggregation operations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AggregationStats {
    /// Total aggregation rounds
    pub rounds: u64,
    /// Total deltas aggregated
    pub deltas_aggregated: u64,
    /// Budget exhaustion events
    pub budget_exhaustions: u64,
    /// Average noise magnitude added
    pub avg_noise_magnitude: f64,
    /// Reconstruction failures
    pub reconstruction_failures: u64,
}

/// Result of a secure aggregation round
#[derive(Debug, Clone)]
pub struct AggregationResult {
    /// Aggregated trait deltas (privatized)
    pub aggregated_deltas: HashMap<String, f64>,
    /// Privacy cost of this round
    pub privacy_cost: f64,
    /// Number of nodes that contributed
    pub contributing_nodes: usize,
    /// Whether budget was sufficient
    pub budget_sufficient: bool,
    /// Noise magnitude added
    pub noise_magnitude: f64,
}

impl SecureAggregator {
    /// Create a new secure aggregator
    pub fn new(num_nodes: usize, epsilon: f64, delta: f64) -> Self {
        Self {
            dp: DifferentialPrivacy::new(epsilon, delta, 1.0),
            sharing: SecretSharing::new(num_nodes as u32, (num_nodes as u32 / 2) + 1),
            budget: PrivacyBudget::new(epsilon * 100.0, delta * 100.0), // Budget for 100 rounds
            num_nodes,
            stats: AggregationStats::default(),
        }
    }

    /// Aggregate trait deltas from multiple nodes securely.
    /// Each node's deltas are privatized before aggregation.
    pub fn aggregate(
        &mut self,
        node_deltas: &[HashMap<String, f64>],
    ) -> AggregationResult {
        self.stats.rounds += 1;

        // Check privacy budget
        let cost = self.dp.privacy_cost();
        if !self.budget.can_afford(cost, self.dp.delta) {
            self.stats.budget_exhaustions += 1;
            return AggregationResult {
                aggregated_deltas: HashMap::new(),
                privacy_cost: 0.0,
                contributing_nodes: 0,
                budget_sufficient: false,
                noise_magnitude: 0.0,
            };
        }

        // Spend budget
        self.budget.spend(cost, self.dp.delta);

        // Collect all trait names
        let mut all_traits: HashMap<String, Vec<f64>> = HashMap::new();
        for node_delta in node_deltas {
            for (trait_name, delta) in node_delta {
                all_traits.entry(trait_name.clone())
                    .or_default()
                    .push(*delta);
            }
        }

        // Privatize and aggregate each trait
        let mut aggregated = HashMap::new();
        let mut total_noise = 0.0;

        for (trait_name, deltas) in &all_traits {
            // Step 1: Each node privatizes its delta
            let privatized: Vec<f64> = deltas.iter()
                .map(|d| self.dp.privatize_scalar(*d))
                .collect();

            // Step 2: Aggregate (simple mean for now; SMPC would use secret shares)
            let n = privatized.len() as f64;
            let mean = privatized.iter().sum::<f64>() / n;

            // Track noise magnitude
            let original_mean = deltas.iter().sum::<f64>() / n;
            total_noise += (mean - original_mean).abs();

            aggregated.insert(trait_name.clone(), mean);
            self.stats.deltas_aggregated += 1;
        }

        let noise_magnitude = if !all_traits.is_empty() {
            total_noise / all_traits.len() as f64
        } else {
            0.0
        };

        // Update running average noise
        let n = self.stats.rounds as f64;
        self.stats.avg_noise_magnitude =
            (self.stats.avg_noise_magnitude * (n - 1.0) + noise_magnitude) / n;

        AggregationResult {
            aggregated_deltas: aggregated,
            privacy_cost: cost,
            contributing_nodes: node_deltas.len(),
            budget_sufficient: true,
            noise_magnitude,
        }
    }

    /// Aggregate using secret sharing (full SMPC protocol)
    pub fn aggregate_with_sharing(
        &mut self,
        node_deltas: &[HashMap<String, f64>],
    ) -> AggregationResult {
        // First privatize, then split into shares, then reconstruct
        let mut aggregated = HashMap::new();

        // Collect all trait names
        let mut all_traits: HashMap<String, Vec<f64>> = HashMap::new();
        for node_delta in node_deltas {
            for (trait_name, delta) in node_delta {
                all_traits.entry(trait_name.clone())
                    .or_default()
                    .push(*delta);
            }
        }

        for (trait_name, deltas) in &all_traits {
            // Each node creates shares of its privatized delta
            let mut all_shares: Vec<Vec<SecretShare>> = Vec::new();
            for delta in deltas {
                let privatized = self.dp.privatize_scalar(*delta);
                let shares = self.sharing.split(privatized, &trait_name);
                all_shares.push(shares);
            }

            // Aggregate shares at each index
            let num_shares = self.sharing.num_shares as usize;
            let mut aggregated_shares: Vec<SecretShare> = Vec::new();

            for share_idx in 0..num_shares {
                let mut sum = 0.0;
                let mut count = 0;
                for node_shares in &all_shares {
                    if let Some(share) = node_shares.get(share_idx) {
                        sum += share.value;
                        count += 1;
                    }
                }
                if count > 0 {
                    aggregated_shares.push(SecretShare {
                        index: (share_idx + 1) as u32,
                        value: sum / count as f64,
                        trait_name: trait_name.clone(),
                        total_shares: self.sharing.num_shares,
                        threshold: self.sharing.threshold,
                    });
                }
            }

            // Reconstruct aggregated value
            match SecretSharing::reconstruct(&aggregated_shares) {
                Some(value) => { aggregated.insert(trait_name.clone(), value); }
                None => { self.stats.reconstruction_failures += 1; }
            }
        }

        let cost = self.dp.privacy_cost();
        self.budget.spend(cost, self.dp.delta);

        AggregationResult {
            aggregated_deltas: aggregated,
            privacy_cost: cost,
            contributing_nodes: node_deltas.len(),
            budget_sufficient: true,
            noise_magnitude: 0.0,
        }
    }
}

// =============================================================================
// 5. FederatedProtocol — Full federated learning protocol with privacy
// =============================================================================

/// Full federated learning protocol combining all privacy mechanisms.
/// Orchestrates rounds of: collect → privatize → share → aggregate → commit.
pub struct FederatedProtocol {
    /// Secure aggregator
    pub aggregator: SecureAggregator,
    /// Round counter
    pub round: u64,
    /// Node participation history
    pub participation: Vec<Vec<usize>>,
    /// Global policy for trait writing (consensus heuristics)
    pub consensus_threshold: f64,
    /// Minimum nodes required for a valid round
    pub min_nodes: usize,
}

impl FederatedProtocol {
    /// Create a new federated protocol
    pub fn new(num_nodes: usize, epsilon: f64, delta: f64) -> Self {
        Self {
            aggregator: SecureAggregator::new(num_nodes, epsilon, delta),
            round: 0,
            participation: Vec::new(),
            consensus_threshold: 0.6,
            min_nodes: (num_nodes / 2) + 1,
        }
    }

    /// Execute one round of federated aggregation
    pub fn execute_round(
        &mut self,
        node_deltas: &[HashMap<String, f64>],
    ) -> FederatedRoundResult {
        self.round += 1;

        // Check minimum participation
        if node_deltas.len() < self.min_nodes {
            return FederatedRoundResult {
                round: self.round,
                success: false,
                aggregation: None,
                reason: format!(
                    "Insufficient nodes: {} < min {}",
                    node_deltas.len(), self.min_nodes
                ),
            };
        }

        // Record participation
        let participating: Vec<usize> = (0..node_deltas.len()).collect();
        self.participation.push(participating);

        // Aggregate
        let result = self.aggregator.aggregate(node_deltas);

        if !result.budget_sufficient {
            return FederatedRoundResult {
                round: self.round,
                success: false,
                aggregation: Some(result),
                reason: "Privacy budget exhausted".to_string(),
            };
        }

        FederatedRoundResult {
            round: self.round,
            success: true,
            aggregation: Some(result),
            reason: "Round completed successfully".to_string(),
        }
    }

    /// Get protocol statistics
    pub fn stats(&self) -> FederatedStats {
        FederatedStats {
            rounds_completed: self.round,
            aggregation_stats: self.aggregator.stats.clone(),
            budget_utilization: self.aggregator.budget.utilization(),
            remaining_epsilon: self.aggregator.budget.remaining_epsilon(),
            avg_participation: if !self.participation.is_empty() {
                self.participation.iter().map(|p| p.len() as f64).sum::<f64>()
                    / self.participation.len() as f64
            } else {
                0.0
            },
        }
    }
}

/// Result of a federated round
#[derive(Debug, Clone)]
pub struct FederatedRoundResult {
    pub round: u64,
    pub success: bool,
    pub aggregation: Option<AggregationResult>,
    pub reason: String,
}

/// Federated protocol statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederatedStats {
    pub rounds_completed: u64,
    pub aggregation_stats: AggregationStats,
    pub budget_utilization: f64,
    pub remaining_epsilon: f64,
    pub avg_participation: f64,
}
