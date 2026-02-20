//! Learning-to-Rank — Learned Ranking Models for Sliding Indexes
//!
//! Table of Contents:
//! 1. RankableItem — An item in a sliding index with features
//! 2. LambdaRank — LambdaMART-inspired pairwise ranking model
//! 3. SlidingIndex — Dynamic priority queue with learned ordering
//! 4. IndexMode — Ascending/descending traversal modes
//! 5. RankTrainer — Online training from pairwise preferences
//! 6. RankMetrics — NDCG, MAP, and other ranking quality metrics
//!
//! Architecture:
//! Learning-to-rank (LTR) embeds directly into sliding indexes, treating
//! them as dynamic priority queues where events, hypotheses, or evidence
//! are scored and reordered via learned models. This eliminates the need
//! for transformer-style attention by leveraging index operations:
//! ascending sorts for building from foundational to derived traits,
//! descending for urgency-driven reasoning. Inference becomes a sequence
//! of rank-guided traversals rather than holistic computations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// =============================================================================
// 1. RankableItem — An item in a sliding index with features
// =============================================================================

/// An item that can be ranked in a sliding index.
/// Contains features used by the ranking model to compute scores.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankableItem {
    /// Unique item ID
    pub id: String,
    /// Feature vector for ranking
    pub features: Vec<f64>,
    /// Current rank score (computed by model)
    pub rank_score: f64,
    /// Ground-truth relevance (for training, 0-4 scale)
    pub relevance: Option<f64>,
    /// Timestamp (for recency features)
    pub timestamp_ms: u64,
    /// Item type tag
    pub item_type: ItemType,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// Types of items that can be ranked
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ItemType {
    /// Evidence event
    Evidence,
    /// Hypothesis
    Hypothesis,
    /// Trait update proposal
    TraitProposal,
    /// Inference path
    InferencePath,
    /// Knowledge fact
    KnowledgeFact,
    /// Expert output
    ExpertOutput,
}

impl RankableItem {
    /// Create a new rankable item
    pub fn new(id: &str, features: Vec<f64>, item_type: ItemType) -> Self {
        Self {
            id: id.to_string(),
            features,
            rank_score: 0.0,
            relevance: None,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            item_type,
            metadata: HashMap::new(),
        }
    }

    /// Create with standard features: [recency, trustworthiness, causal_impact,
    /// confidence, magnitude, evidence_count, contradiction_rate, source_trust]
    pub fn with_standard_features(
        id: &str,
        recency: f64,
        trustworthiness: f64,
        causal_impact: f64,
        confidence: f64,
        magnitude: f64,
        evidence_count: f64,
        contradiction_rate: f64,
        source_trust: f64,
        item_type: ItemType,
    ) -> Self {
        Self::new(
            id,
            vec![recency, trustworthiness, causal_impact, confidence,
                 magnitude, evidence_count, contradiction_rate, source_trust],
            item_type,
        )
    }
}

// =============================================================================
// 2. LambdaRank — LambdaMART-inspired pairwise ranking model
// =============================================================================

/// LambdaRank: a pairwise learning-to-rank model inspired by LambdaMART.
/// Uses gradient-boosted decision stumps for fast, interpretable ranking.
/// Each "tree" is a single feature threshold (decision stump) for efficiency.
pub struct LambdaRank {
    /// Ensemble of decision stumps (feature_idx, threshold, left_score, right_score)
    stumps: Vec<DecisionStump>,
    /// Learning rate for boosting
    learning_rate: f64,
    /// Number of features
    num_features: usize,
    /// Maximum number of stumps
    max_stumps: usize,
    /// Training iterations completed
    iterations: u64,
    /// Feature importance scores
    feature_importance: Vec<f64>,
}

/// A single decision stump (weak learner)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DecisionStump {
    /// Which feature to split on
    feature_idx: usize,
    /// Threshold value
    threshold: f64,
    /// Score for items where feature <= threshold
    left_score: f64,
    /// Score for items where feature > threshold
    right_score: f64,
    /// Weight of this stump in the ensemble
    weight: f64,
}

impl LambdaRank {
    /// Create a new LambdaRank model
    pub fn new(num_features: usize) -> Self {
        Self {
            stumps: Vec::new(),
            learning_rate: 0.1,
            num_features,
            max_stumps: 100,
            iterations: 0,
            feature_importance: vec![0.0; num_features],
        }
    }

    /// Set learning rate
    pub fn with_learning_rate(mut self, lr: f64) -> Self {
        self.learning_rate = lr;
        self
    }

    /// Score a single item using the ensemble
    pub fn score(&self, features: &[f64]) -> f64 {
        let mut total = 0.0;
        for stump in &self.stumps {
            let feature_val = features.get(stump.feature_idx).copied().unwrap_or(0.0);
            let stump_score = if feature_val <= stump.threshold {
                stump.left_score
            } else {
                stump.right_score
            };
            total += stump.weight * stump_score;
        }
        total
    }

    /// Score and rank a list of items (returns indices sorted by score descending)
    pub fn rank(&self, items: &[RankableItem]) -> Vec<usize> {
        let mut scored: Vec<(usize, f64)> = items.iter()
            .enumerate()
            .map(|(i, item)| (i, self.score(&item.features)))
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().map(|(i, _)| i).collect()
    }

    /// Train on pairwise preferences: item_a should rank higher than item_b
    pub fn train_pairwise(&mut self, items: &mut [RankableItem]) {
        if items.len() < 2 || self.stumps.len() >= self.max_stumps {
            return;
        }

        // Compute current scores
        for item in items.iter_mut() {
            item.rank_score = self.score(&item.features);
        }

        // Find best stump to add (greedy)
        let mut best_stump: Option<DecisionStump> = None;
        let mut best_gain = f64::NEG_INFINITY;

        for feat_idx in 0..self.num_features {
            // Try median as threshold
            let mut vals: Vec<f64> = items.iter()
                .map(|item| item.features.get(feat_idx).copied().unwrap_or(0.0))
                .collect();
            vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let threshold = vals[vals.len() / 2];

            // Compute pairwise lambda gradients
            let (left_grad, right_grad, gain) = self.compute_lambda_gradients(
                items, feat_idx, threshold,
            );

            if gain > best_gain {
                best_gain = gain;
                best_stump = Some(DecisionStump {
                    feature_idx: feat_idx,
                    threshold,
                    left_score: left_grad * self.learning_rate,
                    right_score: right_grad * self.learning_rate,
                    weight: 1.0,
                });
            }
        }

        if let Some(stump) = best_stump {
            // Update feature importance
            self.feature_importance[stump.feature_idx] += best_gain.abs();
            self.stumps.push(stump);
            self.iterations += 1;
        }
    }

    /// Compute lambda gradients for a candidate split
    fn compute_lambda_gradients(
        &self,
        items: &[RankableItem],
        feature_idx: usize,
        threshold: f64,
    ) -> (f64, f64, f64) {
        let mut left_gradient = 0.0;
        let mut right_gradient = 0.0;
        let mut left_count = 0.0;
        let mut right_count = 0.0;

        // Pairwise comparisons
        for i in 0..items.len() {
            for j in (i + 1)..items.len() {
                let rel_i = items[i].relevance.unwrap_or(0.0);
                let rel_j = items[j].relevance.unwrap_or(0.0);

                if (rel_i - rel_j).abs() < 1e-10 {
                    continue; // Same relevance, no gradient
                }

                // Lambda: gradient of pairwise loss
                let score_diff = items[i].rank_score - items[j].rank_score;
                let sigma = 1.0; // Sigmoid steepness
                let lambda = -sigma / (1.0 + (sigma * score_diff).exp());

                // NDCG delta (simplified)
                let ndcg_delta = (rel_i - rel_j).abs();
                let lambda_scaled = lambda * ndcg_delta;

                // Assign to left/right based on feature threshold
                let feat_i = items[i].features.get(feature_idx).copied().unwrap_or(0.0);
                let feat_j = items[j].features.get(feature_idx).copied().unwrap_or(0.0);

                if feat_i <= threshold {
                    left_gradient += lambda_scaled;
                    left_count += 1.0;
                } else {
                    right_gradient += lambda_scaled;
                    right_count += 1.0;
                }

                if feat_j <= threshold {
                    left_gradient -= lambda_scaled;
                    left_count += 1.0;
                } else {
                    right_gradient -= lambda_scaled;
                    right_count += 1.0;
                }
            }
        }

        // Normalize
        if left_count > 0.0 { left_gradient /= left_count; }
        if right_count > 0.0 { right_gradient /= right_count; }

        let gain = left_gradient.abs() + right_gradient.abs();
        (left_gradient, right_gradient, gain)
    }

    /// Get feature importance (normalized)
    pub fn feature_importance(&self) -> Vec<f64> {
        let total: f64 = self.feature_importance.iter().sum();
        if total > 0.0 {
            self.feature_importance.iter().map(|&v| v / total).collect()
        } else {
            vec![1.0 / self.num_features as f64; self.num_features]
        }
    }

    /// Get model statistics
    pub fn stats(&self) -> LambdaRankStats {
        LambdaRankStats {
            num_stumps: self.stumps.len(),
            iterations: self.iterations,
            feature_importance: self.feature_importance(),
        }
    }
}

/// LambdaRank model statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LambdaRankStats {
    pub num_stumps: usize,
    pub iterations: u64,
    pub feature_importance: Vec<f64>,
}

// =============================================================================
// 3. SlidingIndex — Dynamic priority queue with learned ordering
// =============================================================================

/// A sliding index: a dynamic priority queue where items are scored
/// and reordered by a learned ranking model. Supports ascending and
/// descending traversal modes for different reasoning strategies.
pub struct SlidingIndex {
    /// Items in the index
    items: Vec<RankableItem>,
    /// Ranking model
    ranker: LambdaRank,
    /// Maximum index size
    max_size: usize,
    /// Current traversal mode
    mode: IndexMode,
    /// Whether index needs re-ranking
    dirty: bool,
    /// Statistics
    stats: IndexStats,
}

/// Traversal mode for the sliding index
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum IndexMode {
    /// Ascending: foundational → derived (e.g., most trusted to least)
    Ascending,
    /// Descending: urgent → routine (e.g., most anomalous first)
    Descending,
    /// Interleaved: alternate between ascending and descending
    Interleaved,
}

/// Index statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IndexStats {
    /// Total items inserted
    pub total_inserts: u64,
    /// Total items evicted (overflow)
    pub total_evictions: u64,
    /// Total re-rankings performed
    pub total_rerankings: u64,
    /// Total traversals
    pub total_traversals: u64,
    /// Average items per traversal
    pub avg_traversal_length: f64,
}

impl SlidingIndex {
    /// Create a new sliding index
    pub fn new(num_features: usize, max_size: usize) -> Self {
        Self {
            items: Vec::new(),
            ranker: LambdaRank::new(num_features),
            max_size,
            mode: IndexMode::Descending,
            dirty: false,
            stats: IndexStats::default(),
        }
    }

    /// Set traversal mode
    pub fn with_mode(mut self, mode: IndexMode) -> Self {
        self.mode = mode;
        self
    }

    /// Insert an item into the index
    pub fn insert(&mut self, mut item: RankableItem) {
        // Score the item
        item.rank_score = self.ranker.score(&item.features);

        // Evict lowest-ranked if at capacity
        if self.items.len() >= self.max_size {
            // Find and remove lowest-scored item
            if let Some((min_idx, _)) = self.items.iter().enumerate()
                .min_by(|(_, a), (_, b)| a.rank_score.partial_cmp(&b.rank_score)
                    .unwrap_or(std::cmp::Ordering::Equal))
            {
                if item.rank_score > self.items[min_idx].rank_score {
                    self.items.swap_remove(min_idx);
                    self.stats.total_evictions += 1;
                } else {
                    return; // New item is worse than all existing
                }
            }
        }

        self.items.push(item);
        self.dirty = true;
        self.stats.total_inserts += 1;
    }

    /// Re-rank all items using the current model
    pub fn rerank(&mut self) {
        for item in &mut self.items {
            item.rank_score = self.ranker.score(&item.features);
        }

        // Sort based on mode
        match self.mode {
            IndexMode::Ascending => {
                self.items.sort_by(|a, b| a.rank_score.partial_cmp(&b.rank_score)
                    .unwrap_or(std::cmp::Ordering::Equal));
            }
            IndexMode::Descending | IndexMode::Interleaved => {
                self.items.sort_by(|a, b| b.rank_score.partial_cmp(&a.rank_score)
                    .unwrap_or(std::cmp::Ordering::Equal));
            }
        }

        self.dirty = false;
        self.stats.total_rerankings += 1;
    }

    /// Traverse the index in current mode, returning items in order
    pub fn traverse(&mut self, limit: usize) -> Vec<&RankableItem> {
        if self.dirty {
            self.rerank();
        }

        self.stats.total_traversals += 1;
        let n = self.items.len().min(limit);
        self.stats.avg_traversal_length = {
            let total = self.stats.total_traversals as f64;
            (self.stats.avg_traversal_length * (total - 1.0) + n as f64) / total
        };

        match self.mode {
            IndexMode::Ascending | IndexMode::Descending => {
                self.items.iter().take(n).collect()
            }
            IndexMode::Interleaved => {
                // Alternate: first, last, second, second-to-last, ...
                let mut result = Vec::with_capacity(n);
                let mut lo = 0;
                let mut hi = self.items.len().saturating_sub(1);
                let mut from_front = true;
                while result.len() < n && lo <= hi {
                    if from_front {
                        result.push(&self.items[lo]);
                        lo += 1;
                    } else {
                        result.push(&self.items[hi]);
                        if hi == 0 { break; }
                        hi -= 1;
                    }
                    from_front = !from_front;
                }
                result
            }
        }
    }

    /// Train the ranking model on current items (requires relevance labels)
    pub fn train(&mut self) {
        let mut items_clone = self.items.clone();
        self.ranker.train_pairwise(&mut items_clone);
        // Re-score with updated model
        self.dirty = true;
    }

    /// Update relevance label for an item (from feedback)
    pub fn set_relevance(&mut self, item_id: &str, relevance: f64) {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == item_id) {
            item.relevance = Some(relevance);
        }
    }

    /// Get item count
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get index statistics
    pub fn stats(&self) -> &IndexStats {
        &self.stats
    }

    /// Get ranking model statistics
    pub fn ranker_stats(&self) -> LambdaRankStats {
        self.ranker.stats()
    }

    /// Get top-K items by score
    pub fn top_k(&mut self, k: usize) -> Vec<&RankableItem> {
        if self.dirty {
            self.rerank();
        }
        self.items.iter().take(k).collect()
    }

    /// Remove an item by ID
    pub fn remove(&mut self, item_id: &str) -> Option<RankableItem> {
        if let Some(pos) = self.items.iter().position(|i| i.id == item_id) {
            Some(self.items.swap_remove(pos))
        } else {
            None
        }
    }
}

// =============================================================================
// 4. RankTrainer — Online training from pairwise preferences
// =============================================================================

/// Online trainer that collects pairwise preferences and periodically
/// updates the ranking model.
pub struct RankTrainer {
    /// Collected pairwise preferences: (better_id, worse_id)
    preferences: Vec<(String, String)>,
    /// Maximum preferences to store
    max_preferences: usize,
    /// Training frequency (train every N preferences)
    train_every: usize,
    /// Total training rounds
    training_rounds: u64,
}

impl RankTrainer {
    pub fn new(train_every: usize) -> Self {
        Self {
            preferences: Vec::new(),
            max_preferences: 10000,
            train_every,
            training_rounds: 0,
        }
    }

    /// Record a pairwise preference: item_a is better than item_b
    pub fn record_preference(&mut self, better_id: &str, worse_id: &str) {
        if self.preferences.len() >= self.max_preferences {
            self.preferences.remove(0);
        }
        self.preferences.push((better_id.to_string(), worse_id.to_string()));
    }

    /// Check if training should be triggered
    pub fn should_train(&self) -> bool {
        self.preferences.len() >= self.train_every
    }

    /// Apply preferences to items (set relevance based on win/loss ratio)
    pub fn apply_preferences(&self, items: &mut [RankableItem]) {
        let mut win_counts: HashMap<String, u64> = HashMap::new();
        let mut loss_counts: HashMap<String, u64> = HashMap::new();

        for (better, worse) in &self.preferences {
            *win_counts.entry(better.clone()).or_insert(0) += 1;
            *loss_counts.entry(worse.clone()).or_insert(0) += 1;
        }

        for item in items.iter_mut() {
            let wins = *win_counts.get(&item.id).unwrap_or(&0) as f64;
            let losses = *loss_counts.get(&item.id).unwrap_or(&0) as f64;
            let total = wins + losses;
            if total > 0.0 {
                // Relevance = win rate scaled to 0-4
                item.relevance = Some(4.0 * wins / total);
            }
        }
    }

    /// Get training statistics
    pub fn stats(&self) -> RankTrainerStats {
        RankTrainerStats {
            preferences_collected: self.preferences.len(),
            training_rounds: self.training_rounds,
        }
    }
}

/// Rank trainer statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankTrainerStats {
    pub preferences_collected: usize,
    pub training_rounds: u64,
}

// =============================================================================
// 5. RankMetrics — NDCG, MAP, and other ranking quality metrics
// =============================================================================

/// Computes ranking quality metrics
pub struct RankMetrics;

impl RankMetrics {
    /// Normalized Discounted Cumulative Gain at position k
    pub fn ndcg_at_k(ranked_items: &[&RankableItem], k: usize) -> f64 {
        let dcg = Self::dcg_at_k(ranked_items, k);
        
        // Ideal DCG: sort by relevance descending
        let mut ideal: Vec<f64> = ranked_items.iter()
            .map(|item| item.relevance.unwrap_or(0.0))
            .collect();
        ideal.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        
        let idcg: f64 = ideal.iter().take(k).enumerate()
            .map(|(i, &rel)| (2.0f64.powf(rel) - 1.0) / (i as f64 + 2.0).log2())
            .sum();

        if idcg > 0.0 { dcg / idcg } else { 0.0 }
    }

    /// Discounted Cumulative Gain at position k
    pub fn dcg_at_k(ranked_items: &[&RankableItem], k: usize) -> f64 {
        ranked_items.iter().take(k).enumerate()
            .map(|(i, item)| {
                let rel = item.relevance.unwrap_or(0.0);
                (2.0f64.powf(rel) - 1.0) / (i as f64 + 2.0).log2()
            })
            .sum()
    }

    /// Mean Average Precision
    pub fn mean_average_precision(ranked_items: &[&RankableItem], relevance_threshold: f64) -> f64 {
        let mut relevant_count = 0;
        let mut precision_sum = 0.0;

        for (i, item) in ranked_items.iter().enumerate() {
            let rel = item.relevance.unwrap_or(0.0);
            if rel >= relevance_threshold {
                relevant_count += 1;
                precision_sum += relevant_count as f64 / (i + 1) as f64;
            }
        }

        let total_relevant = ranked_items.iter()
            .filter(|item| item.relevance.unwrap_or(0.0) >= relevance_threshold)
            .count();

        if total_relevant > 0 {
            precision_sum / total_relevant as f64
        } else {
            0.0
        }
    }

    /// Mean Reciprocal Rank
    pub fn mrr(ranked_items: &[&RankableItem], relevance_threshold: f64) -> f64 {
        for (i, item) in ranked_items.iter().enumerate() {
            if item.relevance.unwrap_or(0.0) >= relevance_threshold {
                return 1.0 / (i + 1) as f64;
            }
        }
        0.0
    }
}
