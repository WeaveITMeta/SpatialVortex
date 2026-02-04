//! SacredMoE Task-Specific Expert Clusters (BETR Phase 3)
//!
//! Extends SacredMoE with benchmark-targeted expert clusters.
//! Based on BETR's finding that "diagonal dominance" improves performance
//! when experts specialize in specific benchmark tasks.
//!
//! ## Key Innovation
//! - Task-specific expert clusters trained on BETR-filtered data
//! - Router uses both geometric (E8) + benchmark similarity for dispatch
//! - Each cluster optimizes for specific benchmark (MMLU, ARC, HellaSwag, etc.)
//!
//! ## Architecture
//! ```
//! Input Token
//!    ↓
//! [Geometric Router] ──E8 distance──┐
//!    ↓                               │
//! [Benchmark Router] ──cosine sim──┤→ Combined routing score
//!    ↓                               │
//! [Task Expert Cluster]             │
//!    ↓
//! Output
//! ```

use crate::ml::sacred_moe::{
    SacredMoEConfig, SacredExpert, ExpertSpecialization,
    GeometricRouter, RouterOutput, PHI, SACRED_POSITIONS,
};
use std::collections::{HashMap, HashSet};

/// Benchmark-specific expert cluster
#[derive(Debug, Clone)]
pub struct TaskExpertCluster {
    /// Benchmark this cluster targets (e.g., "MMLU", "ARC-Challenge")
    pub benchmark: String,
    /// Expert IDs in this cluster
    pub expert_ids: Vec<usize>,
    /// Target task category
    pub task_category: TaskCategory,
    /// Cluster embedding (centroid of benchmark examples)
    pub cluster_embedding: Vec<f32>,
    /// Training statistics
    pub training_stats: ClusterTrainingStats,
}

/// Task categories for clustering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskCategory {
    KnowledgeQA,      // MMLU, ARC, SciQ
    Commonsense,      // HellaSwag, PIQA, WinoGrande
    Math,             // GSM8K
    Reading,          // SQuAD, Lambada
    Code,             // HumanEval
    General,
}

impl TaskCategory {
    /// Get all task categories
    pub fn all() -> Vec<TaskCategory> {
        vec![
            TaskCategory::KnowledgeQA,
            TaskCategory::Commonsense,
            TaskCategory::Math,
            TaskCategory::Reading,
            TaskCategory::Code,
            TaskCategory::General,
        ]
    }

    /// Map benchmark name to category
    pub fn from_benchmark(benchmark: &str) -> Self {
        let lower = benchmark.to_lowercase();
        if lower.contains("mmlu") || lower.contains("arc") || lower.contains("sciq") {
            TaskCategory::KnowledgeQA
        } else if lower.contains("hellaswag") || lower.contains("piqa") || lower.contains("winogrande") {
            TaskCategory::Commonsense
        } else if lower.contains("gsm8k") || lower.contains("math") {
            TaskCategory::Math
        } else if lower.contains("squad") || lower.contains("lambada") || lower.contains("reading") {
            TaskCategory::Reading
        } else if lower.contains("humaneval") || lower.contains("code") {
            TaskCategory::Code
        } else {
            TaskCategory::General
        }
    }
}

/// Training statistics for a cluster
#[derive(Debug, Clone, Default)]
pub struct ClusterTrainingStats {
    /// Number of training examples seen
    pub examples_seen: u64,
    /// Average loss on this cluster's data
    pub avg_loss: f32,
    /// Benchmark accuracy improvement
    pub accuracy_delta: f32,
    /// Last updated timestamp
    pub last_updated: u64,
}

/// Configuration for task-specific MoE
#[derive(Debug, Clone)]
pub struct TaskSpecificMoEConfig {
    /// Base SacredMoE config
    pub base_config: SacredMoEConfig,
    /// Number of experts per task cluster
    pub experts_per_cluster: usize,
    /// Whether to enable task-specific routing
    pub task_routing_enabled: bool,
    /// Weight for geometric routing (0-1)
    pub geometric_weight: f32,
    /// Weight for benchmark routing (0-1)
    pub benchmark_weight: f32,
    /// Minimum confidence to use task routing (fallback to general otherwise)
    pub task_routing_threshold: f32,
}

impl Default for TaskSpecificMoEConfig {
    fn default() -> Self {
        Self {
            base_config: SacredMoEConfig::default(),
            experts_per_cluster: 128,  // 128 experts per task
            task_routing_enabled: true,
            geometric_weight: 0.6,     // 60% geometric, 40% benchmark
            benchmark_weight: 0.4,
            task_routing_threshold: 0.5,
        }
    }
}

/// Task-specific MoE layer with benchmark-targeted clusters
pub struct TaskSpecificSacredMoE {
    /// Configuration
    pub config: TaskSpecificMoEConfig,
    /// All experts (including task-specific clusters)
    pub experts: Vec<SacredExpert>,
    /// Task-specific clusters
    pub clusters: HashMap<TaskCategory, TaskExpertCluster>,
    /// Geometric router (E8-based)
    pub geometric_router: GeometricRouter,
    /// Benchmark router (embedding similarity)
    pub benchmark_router: BenchmarkRouter,
    /// General experts (fallback)
    pub general_expert_ids: Vec<usize>,
}

/// Benchmark-based router
pub struct BenchmarkRouter {
    /// Embedding dimension
    dim: usize,
    /// Task category centroids
    centroids: HashMap<TaskCategory, Vec<f32>>,
}

impl BenchmarkRouter {
    pub fn new(dim: usize) -> Self {
        Self {
            dim,
            centroids: HashMap::new(),
        }
    }

    /// Compute input embedding (simplified character trigram embedding)
    pub fn embed(&self, text: &str) -> Vec<f32> {
        let mut embedding = vec![0.0f32; self.dim];
        let text_lower = text.to_lowercase();
        
        let chars: Vec<char> = text_lower.chars().collect();
        for i in 0..chars.len().saturating_sub(2) {
            let trigram = format!("{}{}{}", chars[i], chars[i+1], chars[i+2]);
            let hash = self.hash_trigram(&trigram);
            embedding[hash % self.dim] += 1.0;
        }
        
        // Normalize
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            embedding.iter_mut().for_each(|x| *x /= norm);
        }
        
        embedding
    }

    fn hash_trigram(&self, trigram: &str) -> usize {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        trigram.hash(&mut hasher);
        hasher.finish() as usize
    }

    /// Route input to task category
    pub fn route(&self, input_embedding: &[f32]) -> TaskCategory {
        let mut best_category = TaskCategory::General;
        let mut best_sim = 0.0f32;

        for (category, centroid) in &self.centroids {
            let sim = cosine_similarity(input_embedding, centroid);
            if sim > best_sim {
                best_sim = sim;
                best_category = *category;
            }
        }

        best_category
    }

    /// Update centroid for a category (called during training)
    pub fn update_centroid(&mut self, category: TaskCategory, embeddings: &[Vec<f32>]) {
        if embeddings.is_empty() {
            return;
        }

        let mut centroid = vec![0.0f32; self.dim];
        for emb in embeddings {
            for (i, &v) in emb.iter().enumerate() {
                if i < self.dim {
                    centroid[i] += v;
                }
            }
        }

        // Average
        let n = embeddings.len() as f32;
        centroid.iter_mut().for_each(|v| *v /= n);

        // Normalize
        let norm: f32 = centroid.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            centroid.iter_mut().for_each(|x| *x /= norm);
        }

        self.centroids.insert(category, centroid);
    }

    /// Compute routing score for each task category
    pub fn route_scores(&self, input_embedding: &[f32]) -> Vec<(TaskCategory, f32)> {
        let mut scores: Vec<(TaskCategory, f32)> = self.centroids.iter()
            .map(|(cat, centroid)| (*cat, cosine_similarity(input_embedding, centroid)))
            .collect();

        // Sort by similarity descending
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scores
    }
}

impl TaskSpecificSacredMoE {
    /// Create new task-specific MoE with benchmark clusters
    pub fn new(config: TaskSpecificMoEConfig) -> Self {
        let total_experts = config.base_config.num_experts;
        let experts_per_cluster = config.experts_per_cluster;
        
        // Reserve experts for clusters
        let num_categories = TaskCategory::all().len();
        let reserved_experts = experts_per_cluster * num_categories;
        let general_expert_count = total_experts.saturating_sub(reserved_experts);
        
        let mut experts = Vec::with_capacity(total_experts);
        let mut clusters = HashMap::new();
        let mut general_expert_ids = Vec::new();
        
        // Create experts for each task cluster
        let categories = TaskCategory::all();
        let mut expert_id = 0;
        
        for category in categories {
            let mut cluster_expert_ids = Vec::with_capacity(experts_per_cluster);
            
            for _ in 0..experts_per_cluster {
                let group = match category {
                    TaskCategory::KnowledgeQA => 3,
                    TaskCategory::Commonsense => 6,
                    TaskCategory::Math => 9,
                    TaskCategory::Reading => 2,
                    TaskCategory::Code => 5,
                    TaskCategory::General => 1,
                };
                
                let expert = SacredExpert::new(expert_id, group, &config.base_config);
                experts.push(expert);
                cluster_expert_ids.push(expert_id);
                expert_id += 1;
            }
            
            let cluster = TaskExpertCluster {
                benchmark: format!("{:?}", category),
                expert_ids: cluster_expert_ids,
                task_category: category,
                cluster_embedding: vec![0.0f32; 384],
                training_stats: ClusterTrainingStats::default(),
            };
            
            clusters.insert(category, cluster);
        }
        
        // Create general experts (fallback)
        for _ in 0..general_expert_count {
            let group = SACRED_POSITIONS[expert_id % SACRED_POSITIONS.len()];
            let expert = SacredExpert::new(expert_id, group, &config.base_config);
            experts.push(expert);
            general_expert_ids.push(expert_id);
            expert_id += 1;
        }
        
        let geometric_router = GeometricRouter::new(&config.base_config);
        let benchmark_router = BenchmarkRouter::new(384);
        
        Self {
            config,
            experts,
            clusters,
            geometric_router,
            benchmark_router,
            general_expert_ids,
        }
    }

    /// Forward pass with task-specific routing
    pub fn forward(&mut self, input: &[f32], input_text: &str) -> Vec<f32> {
        // Get geometric routing scores
        let geo_output = self.geometric_router.route(input, &self.config.base_config);
        let geo_scores: HashMap<usize, f32> = geo_output.selected_experts.into_iter().collect();
        
        // Get benchmark routing scores
        let input_embedding = self.benchmark_router.embed(input_text);
        let task_scores = self.benchmark_router.route_scores(&input_embedding);
        
        // Determine if we use task routing or fallback to general
        let (use_task_routing, best_category) = if let Some((cat, score)) = task_scores.first() {
            (*score > self.config.task_routing_threshold, *cat)
        } else {
            (false, TaskCategory::General)
        };
        
        // Get relevant expert IDs
        let expert_ids: Vec<usize> = if use_task_routing {
            self.clusters.get(&best_category)
                .map(|c| c.expert_ids.clone())
                .unwrap_or_else(|| self.general_expert_ids.clone())
        } else {
            self.general_expert_ids.clone()
        };
        
        // Combine geometric scores with task relevance
        let mut combined_scores: Vec<(usize, f32)> = expert_ids.iter()
            .map(|&id| {
                let geo_score = geo_scores.get(&id).copied().unwrap_or(0.0);
                let task_bonus = if use_task_routing { 0.2 } else { 0.0 };
                let score = geo_score * self.config.geometric_weight + task_bonus * self.config.benchmark_weight;
                (id, score)
            })
            .collect();
        
        // Sort by score and take top-k
        combined_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let top_k = self.config.base_config.top_k.min(combined_scores.len());
        
        // Aggregate expert outputs
        let mut output = vec![0.0f32; input.len()];
        let mut total_weight = 0.0f32;
        
        for (id, weight) in combined_scores.iter().take(top_k) {
            if let Some(expert) = self.experts.get_mut(*id) {
                let expert_out = expert.forward(input);
                let boost = expert.get_sacred_boost();
                let effective_weight = weight * boost;
                
                for (i, &v) in expert_out.iter().enumerate() {
                    if i < output.len() {
                        output[i] += v * effective_weight;
                    }
                }
                total_weight += effective_weight;
            }
        }
        
        // Normalize
        if total_weight > 0.0 {
            output.iter_mut().for_each(|v| *v /= total_weight);
        }
        
        output
    }

    /// Train cluster centroids from benchmark examples
    pub fn train_clusters(&mut self, benchmark_data: &[(String, TaskCategory)]) {
        println!("[TaskSpecificMoE] Training {} clusters from {} examples",
                 self.clusters.len(), benchmark_data.len());
        
        // Group examples by category
        let mut category_embeddings: HashMap<TaskCategory, Vec<Vec<f32>>> = HashMap::new();
        
        for (text, category) in benchmark_data {
            let embedding = self.benchmark_router.embed(text);
            category_embeddings.entry(*category)
                .or_insert_with(Vec::new)
                .push(embedding);
        }
        
        // Update centroids
        for (category, embeddings) in category_embeddings {
            self.benchmark_router.update_centroid(category, &embeddings);
            
            if let Some(cluster) = self.clusters.get_mut(&category) {
                // Update cluster embedding
                if let Some(centroid) = self.benchmark_router.centroids.get(&category) {
                    cluster.cluster_embedding = centroid.clone();
                }
                cluster.training_stats.examples_seen = embeddings.len() as u64;
                cluster.training_stats.last_updated = 0; // Would use actual timestamp
            }
            
            println!("[TaskSpecificMoE] Updated {:?} centroid with {} examples",
                     category, embeddings.len());
        }
    }

    /// Get cluster statistics
    pub fn get_cluster_stats(&self) -> Vec<(TaskCategory, usize, u64)> {
        self.clusters.iter()
            .map(|(cat, cluster)| (*cat, cluster.expert_ids.len(), cluster.training_stats.examples_seen))
            .collect()
    }
}

/// Cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    
    dot / (norm_a * norm_b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_category_mapping() {
        assert_eq!(TaskCategory::from_benchmark("MMLU"), TaskCategory::KnowledgeQA);
        assert_eq!(TaskCategory::from_benchmark("HellaSwag"), TaskCategory::Commonsense);
        assert_eq!(TaskCategory::from_benchmark("GSM8K"), TaskCategory::Math);
    }

    #[test]
    fn test_benchmark_router() {
        let mut router = BenchmarkRouter::new(384);
        
        // Update centroids
        let embeddings = vec![
            vec![1.0, 0.0, 0.0],
            vec![0.9, 0.1, 0.0],
        ];
        router.update_centroid(TaskCategory::KnowledgeQA, &embeddings);
        
        // Route input
        let input = vec![0.95, 0.05, 0.0];
        let category = router.route(&input);
        
        assert_eq!(category, TaskCategory::KnowledgeQA);
    }

    #[test]
    fn test_task_specific_moe_creation() {
        let config = TaskSpecificMoEConfig::default();
        let moe = TaskSpecificSacredMoE::new(config);
        
        assert_eq!(moe.clusters.len(), 6);  // 6 task categories
        assert!(!moe.general_expert_ids.is_empty());
    }
}
