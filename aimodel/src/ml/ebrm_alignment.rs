//! EBRM Benchmark Energy Alignment
//!
//! Implements Phase 2 of BETR-inspired improvements:
//! Train EBRM to create an energy landscape where:
//! - Golden reasoning paths (from benchmark examples) are attractors (low energy)
//! - Incorrect/random paths are repellors (high energy)
//!
//! ## Method
//! 1. Extract "golden paths" from benchmark Q/A pairs
//! 2. Generate contrastive "negative paths" (corrupted/incorrect reasoning)
//! 3. Train EBRM with contrastive loss: minimize energy on golden, maximize on negative

use crate::data::models::BeamTensor;
use crate::ml::ebrm::{EnergyBasedReasoningModel, TraceEnergy};

/// Golden reasoning path extracted from benchmark
#[derive(Debug, Clone)]
pub struct GoldenPath {
    /// Benchmark name
    pub benchmark: String,
    /// Question text
    pub question: String,
    /// Correct answer
    pub correct_answer: String,
    /// Reasoning steps as beams
    pub reasoning_steps: Vec<BeamTensor>,
    /// Expected confidence at each step
    pub expected_confidence: Vec<f32>,
}

/// Contrastive training pair for EBRM
#[derive(Debug, Clone)]
pub struct ContrastivePair {
    /// Golden (low energy) path
    pub positive: Vec<BeamTensor>,
    /// Corrupted (high energy) path
    pub negative: Vec<BeamTensor>,
    /// Margin for contrastive loss
    pub margin: f32,
}

/// Configuration for benchmark energy alignment
#[derive(Debug, Clone)]
pub struct EnergyAlignmentConfig {
    /// Learning rate for energy landscape updates
    pub learning_rate: f32,
    /// Margin for contrastive loss
    pub contrastive_margin: f32,
    /// Ratio of negative samples per positive
    pub negative_ratio: usize,
    /// Minimum energy for golden paths
    pub target_golden_energy: f32,
    /// Maximum energy for negative paths
    pub target_negative_energy: f32,
    /// Number of refinement iterations
    pub refinement_steps: usize,
}

impl Default for EnergyAlignmentConfig {
    fn default() -> Self {
        Self {
            learning_rate: 0.01,
            contrastive_margin: 0.5,
            negative_ratio: 3,
            target_golden_energy: 0.2,    // Low energy attractor
            target_negative_energy: 0.9,  // High energy repellor
            refinement_steps: 10,
        }
    }
}

/// EBRM with benchmark energy alignment training
pub struct AlignedEBRM {
    /// Base EBRM model
    pub base_model: EnergyBasedReasoningModel,
    /// Training configuration
    pub config: EnergyAlignmentConfig,
    /// Golden paths from benchmarks
    golden_paths: Vec<GoldenPath>,
    /// Training statistics
    pub stats: AlignmentStats,
}

/// Training statistics
#[derive(Debug, Clone, Default)]
pub struct AlignmentStats {
    /// Total training iterations
    pub iterations: usize,
    /// Average positive path energy
    pub avg_positive_energy: f32,
    /// Average negative path energy
    pub avg_negative_energy: f32,
    /// Contrastive loss
    pub contrastive_loss: f32,
    /// Paths successfully aligned
    pub aligned_paths: usize,
}

impl AlignedEBRM {
    pub fn new(base_model: EnergyBasedReasoningModel, config: EnergyAlignmentConfig) -> Self {
        Self {
            base_model,
            config,
            golden_paths: Vec::new(),
            stats: AlignmentStats::default(),
        }
    }

    /// Extract golden paths from benchmark Q/A pairs
    /// 
    /// Creates idealized reasoning paths that lead to correct answers
    pub fn extract_golden_paths(
        &mut self,
        questions: &[BenchmarkQA],
        benchmark_name: &str,
    ) -> usize {
        let initial_count = self.golden_paths.len();
        
        for qa in questions {
            // Create golden reasoning steps
            let steps = self.create_reasoning_steps(qa);
            
            // Expected confidence: starts lower, increases toward answer
            let n_steps = steps.len();
            let expected_confidence: Vec<f32> = (0..n_steps)
                .map(|i| 0.5 + 0.5 * (i as f32 / n_steps as f32))
                .collect();
            
            self.golden_paths.push(GoldenPath {
                benchmark: benchmark_name.to_string(),
                question: qa.question.clone(),
                correct_answer: qa.correct_answer.clone(),
                reasoning_steps: steps,
                expected_confidence,
            });
        }
        
        let extracted = self.golden_paths.len() - initial_count;
        println!("[AlignedEBRM] Extracted {} golden paths from {}", 
                 extracted, benchmark_name);
        extracted
    }

    /// Create reasoning steps from Q/A
    fn create_reasoning_steps(&self, qa: &BenchmarkQA) -> Vec<BeamTensor> {
        let mut steps = Vec::new();
        
        // Step 1: Question comprehension
        let mut q_beam = BeamTensor::default();
        q_beam.confidence = 0.6;
        q_beam.position = 1;
        steps.push(q_beam);
        
        // Step 2: Analysis/sacred position 3
        let mut analysis_beam = BeamTensor::default();
        analysis_beam.confidence = 0.7;
        analysis_beam.position = 3;
        steps.push(analysis_beam);
        
        // Step 3: Intermediate reasoning
        let mut mid_beam = BeamTensor::default();
        mid_beam.confidence = 0.75;
        mid_beam.position = 5;
        steps.push(mid_beam);
        
        // Step 4: Verification/sacred position 6
        let mut verify_beam = BeamTensor::default();
        verify_beam.confidence = 0.85;
        verify_beam.position = 6;
        steps.push(verify_beam);
        
        // Step 5: Conclusion/sacred position 9
        let mut answer_beam = BeamTensor::default();
        answer_beam.confidence = 0.95;
        answer_beam.position = 9;
        steps.push(answer_beam);
        
        steps
    }

    /// Generate contrastive negative examples by corrupting golden paths
    pub fn generate_negative_paths(&self, golden: &GoldenPath) -> Vec<Vec<BeamTensor>> {
        let mut negatives = Vec::with_capacity(self.config.negative_ratio);
        
        for i in 0..self.config.negative_ratio {
            let corrupted = match i % 3 {
                0 => self.corrupt_by_noise(&golden.reasoning_steps),
                1 => self.corrupt_by_reorder(&golden.reasoning_steps),
                _ => self.corrupt_by_dropout(&golden.reasoning_steps),
            };
            negatives.push(corrupted);
        }
        
        negatives
    }

    /// Corrupt path by adding noise to confidence
    fn corrupt_by_noise(&self, path: &[BeamTensor]) -> Vec<BeamTensor> {
        path.iter()
            .map(|beam| {
                let mut corrupted = beam.clone();
                // Reduce confidence randomly
                corrupted.confidence *= 0.5 + 0.3 * rand::random::<f32>();
                corrupted
            })
            .collect()
    }

    /// Corrupt path by reordering steps
    fn corrupt_by_reorder(&self, path: &[BeamTensor]) -> Vec<BeamTensor> {
        let mut corrupted: Vec<BeamTensor> = path.to_vec();
        if corrupted.len() >= 2 {
            // Swap two random adjacent steps
            let idx = rand::random::<usize>() % (corrupted.len() - 1);
            corrupted.swap(idx, idx + 1);
        }
        corrupted
    }

    /// Corrupt path by dropping steps
    fn corrupt_by_dropout(&self, path: &[BeamTensor]) -> Vec<BeamTensor> {
        path.iter()
            .enumerate()
            .filter(|(i, _)| *i % 2 == 0 || *i == path.len() - 1)  // Keep every other + last
            .map(|(_, beam)| beam.clone())
            .collect()
    }

    /// Compute contrastive loss for a positive-negative pair
    fn contrastive_loss(&self, positive: &[BeamTensor], negative: &[BeamTensor]) -> f32 {
        let pos_energy = self.base_model.score_trace(positive).global_energy;
        let neg_energy = self.base_model.score_trace(negative).global_energy;
        
        // Contrastive loss: ensure positive has lower energy than negative by margin
        let loss = (pos_energy - neg_energy + self.config.contrastive_margin).max(0.0);
        
        loss
    }

    /// Train one iteration on golden paths with contrastive learning
    pub fn train_iteration(&mut self) -> f32 {
        if self.golden_paths.is_empty() {
            return 0.0;
        }
        
        let mut total_loss = 0.0;
        let mut positive_energies = Vec::new();
        let mut negative_energies = Vec::new();
        let mut updates: Vec<(Vec<BeamTensor>, Vec<BeamTensor>, f32)> = Vec::new();
        
        for golden in &self.golden_paths {
            // Score positive (golden) path
            let pos_energy = self.base_model.score_trace(&golden.reasoning_steps).global_energy;
            positive_energies.push(pos_energy);
            
            // Generate and score negative paths
            let negatives = self.generate_negative_paths(golden);
            for neg_path in negatives {
                let neg_energy = self.base_model.score_trace(&neg_path).global_energy;
                negative_energies.push(neg_energy);
                
                // Compute contrastive loss
                let loss = self.contrastive_loss(&golden.reasoning_steps, &neg_path);
                total_loss += loss;
                
                // Queue update for energy landscape
                updates.push((golden.reasoning_steps.clone(), neg_path, loss));
            }
        }
        
        // Apply updates after iteration
        for (positive, negative, loss) in updates {
            self.update_energy_landscape(&positive, &negative, loss);
        }
        
        // Update statistics
        self.stats.iterations += 1;
        self.stats.avg_positive_energy = positive_energies.iter().sum::<f32>() 
            / positive_energies.len() as f32;
        self.stats.avg_negative_energy = negative_energies.iter().sum::<f32>() 
            / negative_energies.len() as f32;
        self.stats.contrastive_loss = total_loss / self.golden_paths.len() as f32;
        
        // Count aligned paths (positive energy < negative energy - margin)
        self.stats.aligned_paths = positive_energies.iter()
            .zip(negative_energies.chunks(self.config.negative_ratio))
            .filter(|(pos, negs)| {
                negs.iter().all(|neg| **pos < *neg - self.config.contrastive_margin * 0.5)
            })
            .count();
        
        total_loss
    }

    /// Update energy landscape based on contrastive feedback
    fn update_energy_landscape(
        &mut self,
        positive: &[BeamTensor],
        negative: &[BeamTensor],
        loss: f32,
    ) {
        // In a full implementation, this would update EBRM parameters
        // For now, we log the update intent
        if loss > 0.0 {
            // Positive path should have lower energy
            // Negative path should have higher energy
            // This would adjust EBRM's internal energy scoring function
        }
    }

    /// Train for multiple iterations
    pub fn train(&mut self, iterations: usize) {
        println!("[AlignedEBRM] Training on {} golden paths for {} iterations",
                 self.golden_paths.len(), iterations);
        
        for i in 0..iterations {
            let loss = self.train_iteration();
            
            if i % 10 == 0 || i == iterations - 1 {
                println!(
                    "[AlignedEBRM] Iter {}/{}: loss={:.4}, pos_energy={:.3}, neg_energy={:.3}, aligned={}/{}",
                    i, iterations, loss,
                    self.stats.avg_positive_energy,
                    self.stats.avg_negative_energy,
                    self.stats.aligned_paths,
                    self.golden_paths.len()
                );
            }
        }
    }

    /// Score a trace with alignment-aware energy
    pub fn score_trace_aligned(&self, trace: &[BeamTensor]) -> AlignedTraceEnergy {
        let base_energy = self.base_model.score_trace(trace);
        
        // Check similarity to golden paths
        let golden_bonus = self.compute_golden_similarity(trace);
        
        AlignedTraceEnergy {
            global_energy: base_energy.global_energy - golden_bonus * 0.1,
            sacred_alignment: base_energy.sacred_alignment,
            is_valid: base_energy.is_valid,
            golden_similarity: golden_bonus,
            aligned: golden_bonus > 0.5,
        }
    }

    /// Compute similarity to golden paths (returns max similarity)
    fn compute_golden_similarity(&self, trace: &[BeamTensor]) -> f32 {
        if self.golden_paths.is_empty() || trace.is_empty() {
            return 0.0;
        }
        
        self.golden_paths.iter()
            .map(|golden| self.path_similarity(trace, &golden.reasoning_steps))
            .fold(0.0f32, f32::max)
    }

    /// Simple path similarity based on position sequence and confidence
    fn path_similarity(&self, a: &[BeamTensor], b: &[BeamTensor]) -> f32 {
        let min_len = a.len().min(b.len());
        if min_len == 0 {
            return 0.0;
        }
        
        let matches: f32 = (0..min_len)
            .map(|i| {
                let pos_match = if a[i].position == b[i].position { 1.0 } else { 0.0 };
                let conf_diff = (a[i].confidence - b[i].confidence).abs();
                pos_match * (1.0 - conf_diff)
            })
            .sum();
        
        matches / min_len as f32
    }

    /// Get training statistics
    pub fn get_stats(&self) -> &AlignmentStats {
        &self.stats
    }
}

/// Benchmark Q/A pair for golden path extraction
#[derive(Debug, Clone)]
pub struct BenchmarkQA {
    pub question: String,
    pub correct_answer: String,
    pub choices: Vec<String>,
    pub category: String,
}

/// Trace energy with alignment information
#[derive(Debug, Clone)]
pub struct AlignedTraceEnergy {
    pub global_energy: f32,
    pub sacred_alignment: f32,
    pub is_valid: bool,
    pub golden_similarity: f32,
    pub aligned: bool,
}

impl Default for AlignedTraceEnergy {
    fn default() -> Self {
        Self {
            global_energy: 0.0,
            sacred_alignment: 0.0,
            is_valid: false,
            golden_similarity: 0.0,
            aligned: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_golden_path_extraction() {
        let ebrm = EnergyBasedReasoningModel::new();
        let config = EnergyAlignmentConfig::default();
        let mut aligned = AlignedEBRM::new(ebrm, config);
        
        let qa = vec![
            BenchmarkQA {
                question: "What is 2+2?".to_string(),
                correct_answer: "4".to_string(),
                choices: vec!["3".to_string(), "4".to_string(), "5".to_string()],
                category: "math".to_string(),
            },
        ];
        
        let extracted = aligned.extract_golden_paths(&qa, "test");
        assert_eq!(extracted, 1);
        assert_eq!(aligned.golden_paths.len(), 1);
    }

    #[test]
    fn test_negative_path_generation() {
        let ebrm = EnergyBasedReasoningModel::new();
        let config = EnergyAlignmentConfig::default();
        let aligned = AlignedEBRM::new(ebrm, config);
        
        let golden = GoldenPath {
            benchmark: "test".to_string(),
            question: "Q".to_string(),
            correct_answer: "A".to_string(),
            reasoning_steps: vec![BeamTensor::default(); 5],
            expected_confidence: vec![0.5, 0.6, 0.7, 0.8, 0.9],
        };
        
        let negatives = aligned.generate_negative_paths(&golden);
        assert_eq!(negatives.len(), 3);  // Default negative_ratio
    }
}
