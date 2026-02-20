//! Geometric Inference Engine
//!
//! # Table of Contents
//! 1. Core rule-based inference (position → ELP attributes)
//! 2. Observation recording for ML training data collection
//! 3. Forward propagation: cluster-enhanced position → attributes
//! 4. Backward propagation: regression-learned confidence → weights
//! 5. Semantic subspace reduction via PCA
//! 6. Bi-directional propagation combining forward + backward
//!
//! # Architecture
//! - **Rule-based baseline**: Hardcoded sacred geometry mappings (always available)
//! - **ML augmentation** (behind `linfa-ml` feature):
//!   - K-Means clustering on flux position feature vectors
//!   - PCA dimensionality reduction on semantic subspaces
//!   - Linear regression for backward chain weight learning
//!   - Bi-directional propagation fusing rule-based + ML signals

use crate::data::attributes::Attributes;

#[cfg(feature = "linfa-ml")]
use linfa::prelude::*;
#[cfg(feature = "linfa-ml")]
use linfa_clustering::KMeans;
#[cfg(feature = "linfa-ml")]
use linfa_reduction::Pca;
#[cfg(feature = "linfa-ml")]
use linfa_linear::LinearRegression;
#[cfg(feature = "linfa-ml")]
use ndarray::{Array1, Array2, Axis, ArrayBase, OwnedRepr, Dim};

// =========================================================================
// Observation: training data for ML models
// =========================================================================

/// A single inference observation for ML training
/// Records position, ELP state, confidence, and whether the inference was correct
#[derive(Debug, Clone)]
pub struct InferenceObservation {
    /// Vortex position (1-9)
    pub position: u8,
    /// ELP tensor at time of inference
    pub elp: [f32; 3],
    /// Semantic embedding (variable-length, truncated/padded to fixed dim)
    pub embedding: Vec<f32>,
    /// Confidence of the inference
    pub confidence: f32,
    /// Whether the inference was correct (ground truth)
    pub correct: bool,
    /// Expert weights used during this inference
    pub expert_weights: Vec<f32>,
}

// =========================================================================
// Cluster assignment result
// =========================================================================

/// Result of cluster-based forward propagation
#[derive(Debug, Clone)]
pub struct ClusterInference {
    /// Cluster ID this position was assigned to
    pub cluster_id: usize,
    /// ELP attributes inferred from cluster centroid
    pub attributes: Attributes,
    /// Confidence based on distance to centroid (closer = higher)
    pub confidence: f32,
}

// =========================================================================
// Reduced embedding result
// =========================================================================

/// Result of PCA-reduced semantic subspace
#[derive(Debug, Clone)]
pub struct ReducedEmbedding {
    /// Reduced-dimension vector
    pub components: Vec<f32>,
    /// Explained variance ratio per component
    pub explained_variance: Vec<f32>,
    /// Original dimensionality
    pub original_dim: usize,
}

// =========================================================================
// Regression-learned weights
// =========================================================================

/// Backward chain weights learned via linear regression
#[derive(Debug, Clone)]
pub struct LearnedWeights {
    /// Per-expert weights learned from observation history
    pub expert_weights: Vec<f32>,
    /// Intercept (bias) term
    pub intercept: f32,
    /// R² score of the regression fit
    pub r_squared: f32,
}

// =========================================================================
// Bi-directional propagation result
// =========================================================================

/// Combined forward + backward propagation result
#[derive(Debug, Clone)]
pub struct BiDirectionalResult {
    /// Forward: position → cluster → attributes
    pub forward_attrs: Attributes,
    /// Forward confidence (cluster proximity)
    pub forward_confidence: f32,
    /// Backward: learned expert weights from regression
    pub backward_weights: Vec<f32>,
    /// Backward confidence (regression R²)
    pub backward_confidence: f32,
    /// Fused confidence: weighted combination of forward + backward
    pub fused_confidence: f32,
}

// =========================================================================
// GeometricInferenceEngine
// =========================================================================

/// ML-augmented geometric inference engine
///
/// Combines rule-based sacred geometry with classical ML:
/// - **Forward**: Position features → K-Means clusters → ELP attributes
/// - **Backward**: Observation history → Linear regression → expert weights
/// - **Reduction**: High-dim embeddings → PCA → compact subspace
#[derive(Debug, Clone)]
pub struct GeometricInferenceEngine {
    /// Confidence threshold for committing an inference
    confidence_threshold: f32,
    /// Observation history for ML training
    observations: Vec<InferenceObservation>,
    /// Maximum observations to retain (ring buffer)
    max_observations: usize,
    /// Number of clusters for K-Means
    n_clusters: usize,
    /// Number of PCA components for reduction
    n_pca_components: usize,
    /// Fixed embedding dimension (pad/truncate to this)
    embedding_dim: usize,
    /// Cached cluster centroids (position features → ELP)
    #[cfg(feature = "linfa-ml")]
    cluster_centroids: Option<Array2<f64>>,
    /// Cached cluster labels
    #[cfg(feature = "linfa-ml")]
    cluster_labels: Option<Vec<usize>>,
    /// Cached regression weights
    #[cfg(feature = "linfa-ml")]
    regression_weights: Option<Array1<f64>>,
    /// Cached regression intercept
    #[cfg(feature = "linfa-ml")]
    regression_intercept: f32,
    /// Cached regression R²
    #[cfg(feature = "linfa-ml")]
    regression_r_squared: f32,
    /// Cached PCA model explained variance
    #[cfg(feature = "linfa-ml")]
    pca_explained_variance: Option<Vec<f32>>,
}

impl Default for GeometricInferenceEngine {
    fn default() -> Self {
        Self {
            confidence_threshold: 0.6,
            observations: Vec::with_capacity(1024),
            max_observations: 4096,
            n_clusters: 6,       // 6 vortex flow positions + sacred groupings
            n_pca_components: 8, // Reduce semantic space to 8 principal components
            embedding_dim: 64,   // Fixed embedding dimension for ML inputs
            #[cfg(feature = "linfa-ml")]
            cluster_centroids: None,
            #[cfg(feature = "linfa-ml")]
            cluster_labels: None,
            #[cfg(feature = "linfa-ml")]
            regression_weights: None,
            #[cfg(feature = "linfa-ml")]
            regression_intercept: 0.0,
            #[cfg(feature = "linfa-ml")]
            regression_r_squared: 0.0,
            #[cfg(feature = "linfa-ml")]
            pca_explained_variance: None,
        }
    }
}

impl GeometricInferenceEngine {
    pub fn new() -> Self { Self::default() }

    /// Configure cluster count for K-Means
    pub fn with_clusters(mut self, n: usize) -> Self {
        self.n_clusters = n.max(2);
        self
    }

    /// Configure PCA component count
    pub fn with_pca_components(mut self, n: usize) -> Self {
        self.n_pca_components = n.max(1);
        self
    }

    /// Configure embedding dimension
    pub fn with_embedding_dim(mut self, dim: usize) -> Self {
        self.embedding_dim = dim.max(4);
        self
    }

    // =====================================================================
    // 1. Core rule-based inference (always available, no feature gate)
    // =====================================================================

    /// Infer ELP attributes from vortex position using sacred geometry rules
    ///
    /// Position mapping:
    /// - 1, 3 → Ethos-dominant (identity, unity)
    /// - 4, 9 → Logos-dominant (logic, completion)
    /// - 5, 6 → Pathos-dominant (emotion, heart)
    /// - 2, 7, 8 → Balanced (flow positions)
    pub fn infer_from_position(&self, position: u8) -> Attributes {
        let mut attrs = Attributes::new();
        match position {
            1 | 3 => attrs.set_ethos(0.8),
            4 | 9 => attrs.set_logos(0.8),
            5 | 6 => attrs.set_pathos(0.8),
            _ => {
                attrs.set_ethos(0.33);
                attrs.set_logos(0.33);
                attrs.set_pathos(0.34);
            }
        }
        attrs.set_digital_root_flux(position);
        attrs
    }

    /// Check if confidence meets the commit threshold
    pub fn meets_threshold(&self, confidence: f32) -> bool {
        confidence >= self.confidence_threshold
    }

    /// Set the confidence threshold
    pub fn set_threshold(&mut self, threshold: f32) {
        self.confidence_threshold = threshold.clamp(0.0, 1.0);
    }

    // =====================================================================
    // 2. Observation recording (always available — collects training data)
    // =====================================================================

    /// Record an inference observation for ML training
    pub fn observe(&mut self, obs: InferenceObservation) {
        if self.observations.len() >= self.max_observations {
            // Ring buffer: drop oldest 25% when full
            let drop_count = self.max_observations / 4;
            self.observations.drain(..drop_count);
        }
        self.observations.push(obs);
    }

    /// Number of observations collected
    pub fn observation_count(&self) -> usize {
        self.observations.len()
    }

    /// Clear all observations
    pub fn clear_observations(&mut self) {
        self.observations.clear();
    }

    /// Build a feature vector from a position for clustering
    /// Features: [position_norm, is_sacred, is_flow, ethos, logos, pathos, sin(pos), cos(pos)]
    fn position_features(&self, position: u8) -> Vec<f32> {
        let pos_norm = position as f32 / 9.0;
        let is_sacred = if matches!(position, 3 | 6 | 9) { 1.0 } else { 0.0 };
        let is_flow = if matches!(position, 1 | 2 | 4 | 5 | 7 | 8) { 1.0 } else { 0.0 };
        let attrs = self.infer_from_position(position);
        let angle = std::f32::consts::TAU * (position as f32 / 9.0);
        vec![
            pos_norm,
            is_sacred,
            is_flow,
            attrs.ethos(),
            attrs.logos(),
            attrs.pathos(),
            angle.sin(),
            angle.cos(),
        ]
    }

    /// Pad or truncate an embedding to the fixed dimension
    fn normalize_embedding(&self, embedding: &[f32]) -> Vec<f32> {
        let mut result = vec![0.0f32; self.embedding_dim];
        let copy_len = embedding.len().min(self.embedding_dim);
        result[..copy_len].copy_from_slice(&embedding[..copy_len]);
        result
    }

    // =====================================================================
    // 3. Forward propagation: K-Means clustering on flux positions
    //    (linfa-ml feature only)
    // =====================================================================

    /// Train K-Means clusters on observed position feature vectors
    /// Returns number of clusters fitted
    #[cfg(feature = "linfa-ml")]
    pub fn fit_position_clusters(&mut self) -> usize {
        if self.observations.len() < self.n_clusters * 2 {
            return 0; // Not enough data
        }

        // Build feature matrix from observations (f64 for linfa)
        let feature_dim = 8; // position_features() returns 8 features
        let n_samples = self.observations.len();
        let mut data = Array2::<f64>::zeros((n_samples, feature_dim));

        for (i, obs) in self.observations.iter().enumerate() {
            let features = self.position_features(obs.position);
            for (j, &f) in features.iter().enumerate() {
                data[[i, j]] = f as f64;
            }
        }

        // Fit K-Means (linfa 0.7 uses Dataset with targets for clustering)
        let dataset = DatasetBase::from(data.clone());
        let model = KMeans::params(self.n_clusters)
            .max_n_iterations(100)
            .tolerance(1e-4)
            .fit(&dataset);

        match model {
            Ok(model) => {
                let dataset_pred = model.predict(DatasetBase::from(data.clone()));
                self.cluster_centroids = Some(model.centroids().clone());
                self.cluster_labels = Some(dataset_pred.targets().to_vec());
                self.n_clusters
            }
            Err(_) => 0,
        }
    }

    /// Forward propagation: infer attributes from position via cluster assignment
    /// Falls back to rule-based if clusters not fitted
    #[cfg(feature = "linfa-ml")]
    pub fn forward_propagate(&self, position: u8) -> ClusterInference {
        let centroids = match &self.cluster_centroids {
            Some(c) => c,
            None => {
                // Fallback to rule-based
                return ClusterInference {
                    cluster_id: position as usize % self.n_clusters,
                    attributes: self.infer_from_position(position),
                    confidence: 0.5,
                };
            }
        };

        let features: Vec<f64> = self.position_features(position)
            .iter().map(|&v| v as f64).collect();
        let feature_arr = Array1::from_vec(features);

        // Find nearest centroid
        let mut best_cluster = 0;
        let mut best_dist = f64::MAX;
        for (ci, centroid) in centroids.axis_iter(Axis(0)).enumerate() {
            let diff = &feature_arr - &centroid.to_owned();
            let dist: f64 = diff.iter().map(|x| x * x).sum::<f64>().sqrt();
            if dist < best_dist {
                best_dist = dist;
                best_cluster = ci;
            }
        }

        // Compute cluster-average ELP from observations in this cluster
        let labels = self.cluster_labels.as_ref().unwrap();
        let mut ethos_sum = 0.0f32;
        let mut logos_sum = 0.0f32;
        let mut pathos_sum = 0.0f32;
        let mut count = 0usize;
        let mut correct_count = 0usize;

        for (i, &label) in labels.iter().enumerate() {
            if label == best_cluster {
                let obs = &self.observations[i];
                ethos_sum += obs.elp[0];
                logos_sum += obs.elp[1];
                pathos_sum += obs.elp[2];
                if obs.correct { correct_count += 1; }
                count += 1;
            }
        }

        let mut attrs = if count > 0 {
            let mut a = Attributes::new();
            a.set_ethos(ethos_sum / count as f32);
            a.set_logos(logos_sum / count as f32);
            a.set_pathos(pathos_sum / count as f32);
            a
        } else {
            self.infer_from_position(position)
        };
        attrs.set_digital_root_flux(position);

        // Confidence: inverse distance (closer to centroid = higher confidence)
        // Blended with cluster accuracy
        let dist_conf = 1.0 / (1.0 + best_dist as f32);
        let acc_conf = if count > 0 { correct_count as f32 / count as f32 } else { 0.5 };
        let confidence = 0.6 * dist_conf + 0.4 * acc_conf;

        ClusterInference {
            cluster_id: best_cluster,
            attributes: attrs,
            confidence,
        }
    }

    /// Forward propagation fallback (no linfa)
    #[cfg(not(feature = "linfa-ml"))]
    pub fn forward_propagate(&self, position: u8) -> ClusterInference {
        ClusterInference {
            cluster_id: position as usize % 6,
            attributes: self.infer_from_position(position),
            confidence: 0.5,
        }
    }

    // =====================================================================
    // 4. Backward propagation: linear regression on confidence → weights
    //    (linfa-ml feature only)
    // =====================================================================

    /// Train linear regression: expert_weights → confidence (for correct observations)
    /// Returns R² score of the fit
    #[cfg(feature = "linfa-ml")]
    pub fn fit_backward_regression(&mut self) -> f32 {
        // Filter to observations with expert weights
        let valid: Vec<&InferenceObservation> = self.observations.iter()
            .filter(|o| !o.expert_weights.is_empty() && o.correct)
            .collect();

        if valid.len() < 10 {
            return 0.0; // Not enough data
        }

        let n_features = valid[0].expert_weights.len();
        let n_samples = valid.len();

        // Build feature matrix (expert weights) and target vector (confidence) in f64
        let mut x = Array2::<f64>::zeros((n_samples, n_features));
        let mut y = Array1::<f64>::zeros(n_samples);

        for (i, obs) in valid.iter().enumerate() {
            for (j, &w) in obs.expert_weights.iter().take(n_features).enumerate() {
                x[[i, j]] = w as f64;
            }
            y[i] = obs.confidence as f64;
        }

        // linfa-linear Dataset: records = x, targets = y
        let dataset = linfa::DatasetBase::new(x.clone(), y.clone());
        let model = LinearRegression::default().fit(&dataset);

        match model {
            Ok(model) => {
                let pred_dataset = model.predict(DatasetBase::from(x.clone()));
                let predictions = pred_dataset.targets();
                // Compute R²
                let y_mean: f64 = y.mean().unwrap_or(0.0);
                let ss_tot: f64 = y.iter()
                    .map(|&yi: &f64| (yi - y_mean).powi(2))
                    .sum();
                let ss_res: f64 = y.iter()
                    .zip(predictions.iter())
                    .map(|(&yi, &pi): (&f64, &f64)| (yi - pi).powi(2))
                    .sum();
                let r_squared = if ss_tot > 0.0 { 1.0 - (ss_res / ss_tot) } else { 0.0 };

                // Store weights
                let params = model.params();
                self.regression_weights = Some(params.clone());
                self.regression_intercept = model.intercept() as f32;
                self.regression_r_squared = r_squared as f32;

                r_squared as f32
            }
            Err(_) => 0.0,
        }
    }

    /// Backward propagation: predict optimal expert weights from learned regression
    #[cfg(feature = "linfa-ml")]
    pub fn backward_propagate(&self, n_experts: usize) -> LearnedWeights {
        match &self.regression_weights {
            Some(weights) => {
                let mut expert_weights = vec![1.0f32; n_experts];
                for (i, &w) in weights.iter().enumerate() {
                    if i < n_experts {
                        // Clamp to reasonable range (f64 → f32)
                        expert_weights[i] = (w as f32).clamp(0.1, 5.0);
                    }
                }
                LearnedWeights {
                    expert_weights,
                    intercept: self.regression_intercept,
                    r_squared: self.regression_r_squared,
                }
            }
            None => {
                // No regression fitted — return uniform weights
                LearnedWeights {
                    expert_weights: vec![1.0; n_experts],
                    intercept: 0.0,
                    r_squared: 0.0,
                }
            }
        }
    }

    /// Backward propagation fallback (no linfa)
    #[cfg(not(feature = "linfa-ml"))]
    pub fn backward_propagate(&self, n_experts: usize) -> LearnedWeights {
        LearnedWeights {
            expert_weights: vec![1.0; n_experts],
            intercept: 0.0,
            r_squared: 0.0,
        }
    }

    // =====================================================================
    // 5. Semantic subspace reduction via PCA
    //    (linfa-ml feature only)
    // =====================================================================

    /// Reduce high-dimensional embeddings to a compact subspace via PCA
    #[cfg(feature = "linfa-ml")]
    pub fn reduce_embeddings(&mut self, embeddings: &[Vec<f32>]) -> Vec<ReducedEmbedding> {
        if embeddings.is_empty() {
            return Vec::new();
        }

        let dim = self.embedding_dim;
        let n_samples = embeddings.len();
        let n_components = self.n_pca_components.min(dim).min(n_samples);

        // Build matrix with normalized embeddings (f64 for linfa)
        let mut data = Array2::<f64>::zeros((n_samples, dim));
        for (i, emb) in embeddings.iter().enumerate() {
            let normed = self.normalize_embedding(emb);
            for (j, &v) in normed.iter().enumerate() {
                data[[i, j]] = v as f64;
            }
        }

        // Fit PCA
        let dataset = DatasetBase::from(data);
        let model = Pca::params(n_components).fit(&dataset);

        match model {
            Ok(model) => {
                let transformed = model.transform(dataset);
                let explained = model.explained_variance_ratio()
                    .iter().map(|&v| v as f32).collect::<Vec<_>>();
                self.pca_explained_variance = Some(explained.clone());

                transformed.records().axis_iter(Axis(0))
                    .map(|row| ReducedEmbedding {
                        components: row.iter().map(|&v| v as f32).collect(),
                        explained_variance: explained.clone(),
                        original_dim: dim,
                    })
                    .collect()
            }
            Err(_) => Vec::new(),
        }
    }

    /// Reduce embeddings fallback (no linfa) — identity truncation
    #[cfg(not(feature = "linfa-ml"))]
    pub fn reduce_embeddings(&mut self, embeddings: &[Vec<f32>]) -> Vec<ReducedEmbedding> {
        embeddings.iter().map(|emb| {
            let normed = self.normalize_embedding(emb);
            let components = normed[..self.n_pca_components.min(normed.len())].to_vec();
            ReducedEmbedding {
                components,
                explained_variance: Vec::new(),
                original_dim: emb.len(),
            }
        }).collect()
    }

    // =====================================================================
    // 6. Bi-directional propagation (combines forward + backward)
    // =====================================================================

    /// Full bi-directional propagation:
    /// - Forward: position → cluster → ELP attributes
    /// - Backward: observation history → regression → expert weights
    /// - Fusion: weighted combination of both signals
    pub fn propagate(&self, position: u8, n_experts: usize) -> BiDirectionalResult {
        let forward = self.forward_propagate(position);
        let backward = self.backward_propagate(n_experts);

        // Fuse: weight forward confidence by cluster quality, backward by R²
        let forward_weight = 0.6;
        let backward_weight = 0.4;
        let fused = forward_weight * forward.confidence
            + backward_weight * backward.r_squared.max(0.1);

        BiDirectionalResult {
            forward_attrs: forward.attributes,
            forward_confidence: forward.confidence,
            backward_weights: backward.expert_weights,
            backward_confidence: backward.r_squared,
            fused_confidence: fused.clamp(0.0, 1.0),
        }
    }

    /// Retrain all ML models from current observation history
    /// Call periodically (e.g., every N observations) to update models
    /// Returns (n_clusters_fitted, regression_r_squared)
    #[cfg(feature = "linfa-ml")]
    pub fn retrain(&mut self) -> (usize, f32) {
        let clusters = self.fit_position_clusters();
        let r_squared = self.fit_backward_regression();
        (clusters, r_squared)
    }

    /// Retrain fallback (no linfa) — no-op
    #[cfg(not(feature = "linfa-ml"))]
    pub fn retrain(&mut self) -> (usize, f32) {
        (0, 0.0)
    }

    /// Check if ML models have been trained
    #[cfg(feature = "linfa-ml")]
    pub fn is_ml_trained(&self) -> bool {
        self.cluster_centroids.is_some() || self.regression_weights.is_some()
    }

    /// Check if ML models have been trained (no linfa)
    #[cfg(not(feature = "linfa-ml"))]
    pub fn is_ml_trained(&self) -> bool {
        false
    }
}
