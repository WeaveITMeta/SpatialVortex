//! Continuous Vortex Runner - Background Learning Through Flux Dynamics
//!
//! Runs the vortex cycle continuously (resets at u64::MAX), organizing content
//! into subjects via FluxMatrix, using LadderIndex for dynamic ranking.
//!
//! At each interval:
//! - Evaluates properties, tags, attributes, parameters for each object
//! - Thinks with node title to evaluate object completely
//! - Learns through dynamics as objects flow through the loop
//!
//! ## Architecture
//! ```text
//! Source (RAG/Tools) → FluxMatrix (Subject) → Nodes (0-9) → LadderIndex (Ranking)
//!                                    ↓
//!                    Vortex Cycle: 1→2→4→8→7→5→1 (exponential)
//!                                    ↓
//!                    Sacred Checkpoints: 3, 6, 9 (anchors)
//!                                    ↓
//!                    RocksDB Storage ← Learned Patterns
//! ```

use crate::data::models::BeamTensor;
use crate::ml::calm::{CALMEngine, CALMConfig, LatentState};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// A node in the flux matrix with full attribute evaluation
#[derive(Debug, Clone)]
pub struct FluxNode {
    pub position: u8,
    pub title: String,
    pub properties: HashMap<String, String>,
    pub tags: Vec<String>,
    pub attributes: HashMap<String, f32>,
    pub parameters: HashMap<String, f32>,
    pub beam: BeamTensor,
    pub ladder_rank: f64,
    pub cycle_count: u64,
}

impl FluxNode {
    pub fn new(position: u8, title: String) -> Self {
        Self {
            position,
            title,
            properties: HashMap::new(),
            tags: Vec::new(),
            attributes: HashMap::new(),
            parameters: HashMap::new(),
            beam: BeamTensor::default(),
            ladder_rank: 0.0,
            cycle_count: 0,
        }
    }

    /// Calculate anchor proximity (3, 6, 9 are sacred)
    pub fn anchor_proximity(&self) -> f64 {
        let sacred = [3u8, 6, 9];
        let nearest = sacred.iter()
            .map(|&anchor| {
                let diff = (anchor as i32 - self.position as i32).abs();
                diff.min(10 - diff) as f64
            })
            .fold(f64::INFINITY, f64::min);
        1.0 - (nearest / 5.0)
    }
}

/// Subject organized as a FluxMatrix
#[derive(Debug, Clone)]
pub struct Subject {
    pub name: String,
    pub nodes: HashMap<u8, FluxNode>,
    pub sacred_guides: HashMap<u8, SacredGuide>,
    pub total_cycles: u64,
    pub accumulated_latent: Option<LatentState>,
}

impl Subject {
    pub fn new(name: String) -> Self {
        let mut nodes = HashMap::new();
        let mut sacred_guides = HashMap::new();

        // Create nodes at positions 0-9
        for pos in 0..=9 {
            if matches!(pos, 3 | 6 | 9) {
                sacred_guides.insert(pos, SacredGuide::new(pos, &name));
            } else {
                let title = format!("{}_{}", name, pos);
                nodes.insert(pos, FluxNode::new(pos, title));
            }
        }

        Self {
            name,
            nodes,
            sacred_guides,
            total_cycles: 0,
            accumulated_latent: None,
        }
    }

    /// Get node or sacred guide at position
    pub fn get_at_position(&self, pos: u8) -> Option<NodeOrGuide> {
        if let Some(node) = self.nodes.get(&pos) {
            Some(NodeOrGuide::Node(node.clone()))
        } else if let Some(guide) = self.sacred_guides.get(&pos) {
            Some(NodeOrGuide::Guide(guide.clone()))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub enum NodeOrGuide {
    Node(FluxNode),
    Guide(SacredGuide),
}

/// Sacred guide at positions 3, 6, 9
#[derive(Debug, Clone)]
pub struct SacredGuide {
    pub position: u8,
    pub divine_properties: Vec<String>,
    pub geometric_significance: String,
}

impl SacredGuide {
    pub fn new(position: u8, subject: &str) -> Self {
        let (props, significance) = match position {
            3 => (
                vec!["Creative Trinity".into(), "Synthesis Point".into(), "Bridge Between Realms".into()],
                format!("Ethos anchor for {}", subject)
            ),
            6 => (
                vec!["Harmonic Balance".into(), "Geometric Center".into(), "Stability Anchor".into()],
                format!("Logos anchor for {}", subject)
            ),
            9 => (
                vec!["Completion Cycle".into(), "Infinite Loop Gateway".into(), "Transcendence Portal".into()],
                format!("Pathos anchor for {}", subject)
            ),
            _ => (vec![], String::new()),
        };

        Self {
            position,
            divine_properties: props,
            geometric_significance: significance,
        }
    }
}

/// Ladder entry for dynamic ranking
#[derive(Debug, Clone)]
pub struct LadderEntry {
    pub entry_id: u64,
    pub subject_name: String,
    pub position: u8,
    pub beam: BeamTensor,
    pub rank: f64,
    pub reinforcement_weight: f64,
    pub cycle_count: u64,
    pub anchor_proximity: f64,
}

impl LadderEntry {
    pub fn new(entry_id: u64, subject_name: String, position: u8, beam: BeamTensor) -> Self {
        let anchor_proximity = {
            let sacred = [3u8, 6, 9];
            let nearest = sacred.iter()
                .map(|&anchor| {
                    let diff = (anchor as i32 - position as i32).abs();
                    diff.min(10 - diff) as f64
                })
                .fold(f64::INFINITY, f64::min);
            1.0 - (nearest / 5.0)
        };

        Self {
            entry_id,
            subject_name,
            position,
            beam,
            rank: 0.0,
            reinforcement_weight: 0.0,
            cycle_count: 0,
            anchor_proximity,
        }
    }

    /// Apply reinforcement learning
    pub fn apply_reward(&mut self, reward: f64, learning_rate: f64) {
        self.reinforcement_weight += learning_rate * reward * self.anchor_proximity;
        self.reinforcement_weight = self.reinforcement_weight.clamp(-1.0, 1.0);
    }

    /// Reset for re-entry into flux (lose rank, keep history)
    pub fn reset_for_flux(&mut self) {
        self.rank = 0.0;
        self.cycle_count += 1;
    }

    /// Calculate rank based on multiple factors
    pub fn calculate_rank(&mut self, top_entries: &[&LadderEntry]) {
        let mut rank = 0.0;

        // 1. Anchor proximity (sacred positions pull harder)
        rank += self.anchor_proximity * 100.0;

        // 2. Reinforcement weight
        rank += self.reinforcement_weight * 50.0;

        // 3. Beam similarity to top entries
        if !top_entries.is_empty() {
            let avg_sim: f64 = top_entries.iter()
                .map(|other| {
                    let dot: f32 = self.beam.digits.iter()
                        .zip(&other.beam.digits)
                        .map(|(a, b)| a * b)
                        .sum();
                    dot as f64
                })
                .sum::<f64>() / top_entries.len() as f64;
            rank += avg_sim * 30.0;
        }

        // 4. Experience bonus
        rank += (self.cycle_count as f64 + 1.0).ln() * 10.0;

        self.rank = rank;
    }

    /// Calculate rank from beam tensors (avoids borrow issues)
    pub fn calculate_rank_from_beams(&mut self, top_beams: &[BeamTensor]) {
        let mut rank = 0.0;

        // 1. Anchor proximity
        rank += self.anchor_proximity * 100.0;

        // 2. Reinforcement weight
        rank += self.reinforcement_weight * 50.0;

        // 3. Beam similarity
        if !top_beams.is_empty() {
            let avg_sim: f64 = top_beams.iter()
                .map(|other| {
                    let dot: f32 = self.beam.digits.iter()
                        .zip(&other.digits)
                        .map(|(a, b)| a * b)
                        .sum();
                    dot as f64
                })
                .sum::<f64>() / top_beams.len() as f64;
            rank += avg_sim * 30.0;
        }

        // 4. Experience bonus
        rank += (self.cycle_count as f64 + 1.0).ln() * 10.0;

        self.rank = rank;
    }
}

/// Continuous Vortex Runner
pub struct VortexRunner {
    /// Current cycle count (resets at u64::MAX)
    pub cycle: u64,
    /// Subjects organized as FluxMatrices
    pub subjects: Arc<RwLock<HashMap<String, Subject>>>,
    /// Ladder index for ranking
    pub ladder: Arc<RwLock<Vec<LadderEntry>>>,
    /// CALM engine for latent space operations
    pub calm: CALMEngine,
    /// Global accumulated latent state
    pub global_latent: Arc<RwLock<LatentState>>,
    /// Learning rate for reinforcement
    pub learning_rate: f64,
    /// Next entry ID
    next_entry_id: Arc<RwLock<u64>>,
    /// Source memory from RAG/tools
    pub source_memory: Arc<RwLock<Vec<SourceEntry>>>,
}

/// Entry from external source (RAG, tools, internet)
#[derive(Debug, Clone)]
pub struct SourceEntry {
    pub content: String,
    pub source_type: SourceType,
    pub beams: Vec<BeamTensor>,
    pub relevance: f32,
    pub cycle_learned: u64,
}

#[derive(Debug, Clone)]
pub enum SourceType {
    RAG,
    Tool,
    Internet,
    UserInput,
    Generated,
}

impl VortexRunner {
    pub fn new() -> Self {
        Self {
            cycle: 0,
            subjects: Arc::new(RwLock::new(HashMap::new())),
            ladder: Arc::new(RwLock::new(Vec::new())),
            calm: CALMEngine::new(CALMConfig::new().with_latent_dim(128)),
            global_latent: Arc::new(RwLock::new(LatentState::new(128))),
            learning_rate: 0.1,
            next_entry_id: Arc::new(RwLock::new(0)),
            source_memory: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Run one complete vortex cycle through all positions
    /// Returns the evolved latent state
    pub async fn run_cycle(&self, input_beams: &[BeamTensor]) -> LatentState {
        // Increment cycle (wraps at u64::MAX)
        let cycle = {
            let mut global = self.global_latent.write().await;
            global.step += 1;
            if global.step == usize::MAX {
                global.step = 0; // Reset at max
            }
            global.step as u64
        };

        // Encode input to latent space
        let mut latent = self.calm.encode(input_beams);

        // Vortex cycle: 1 → 2 → 4 → 8 → 7 → 5 → 1 (exponential doubling, then halving)
        let vortex_positions = [1u8, 2, 4, 8, 7, 5];

        for &pos in &vortex_positions {
            // Evolve latent at each position
            latent = self.calm.predict_next(&latent);

            // Decode to beams at this position
            let position_beams = self.calm.decode(&latent);

            // Evaluate all subjects at this position
            let subjects = self.subjects.read().await;
            for (subject_name, subject) in subjects.iter() {
                if let Some(node_or_guide) = subject.get_at_position(pos) {
                    // Think with node title
                    self.evaluate_at_position(&node_or_guide, &position_beams, &latent, cycle as u64).await;
                }
            }
        }

        // Sacred checkpoints: 3, 6, 9
        for &sacred_pos in &[3u8, 6, 9] {
            latent = self.calm.predict_next(&latent);

            // Sacred positions have stronger anchor pull
            latent.sacred_alignment = (latent.sacred_alignment + 0.1).min(1.0);

            // Update ladder rankings at sacred positions
            self.update_ladder_rankings().await;
        }

        // Accumulate into global latent
        {
            let mut global = self.global_latent.write().await;
            for (i, val) in latent.latent.iter().enumerate() {
                if i < global.latent.len() {
                    // Exponential moving average
                    global.latent[i] = global.latent[i] * 0.9 + val * 0.1;
                }
            }
            global.energy = (global.energy + latent.energy) / 2.0;
            global.sacred_alignment = global.sacred_alignment.max(latent.sacred_alignment);
        }

        latent
    }

    /// Run N cycles exponentially (2^n iterations)
    pub async fn run_exponential(&self, input_beams: &[BeamTensor], power: u32) -> LatentState {
        let iterations = 2u64.pow(power);
        let mut latent = self.calm.encode(input_beams);

        for _ in 0..iterations {
            latent = self.run_cycle(&self.calm.decode(&latent)).await;

            // Early exit if energy drops too low
            if latent.energy < 0.01 {
                break;
            }
        }

        latent
    }

    /// Evaluate object at a vortex position
    async fn evaluate_at_position(
        &self,
        node_or_guide: &NodeOrGuide,
        beams: &[BeamTensor],
        latent: &LatentState,
        cycle: u64,
    ) {
        match node_or_guide {
            NodeOrGuide::Node(node) => {
                // Think with node title
                let title_beams = self.text_to_beams(&node.title);
                let combined: Vec<BeamTensor> = beams.iter()
                    .chain(title_beams.iter())
                    .cloned()
                    .collect();

                // Encode combined for evaluation
                let eval_latent = self.calm.encode(&combined);

                // Calculate reward based on energy alignment
                let reward = eval_latent.energy - 0.5; // Positive if above average

                // Update ladder entry
                let mut ladder = self.ladder.write().await;
                for entry in ladder.iter_mut() {
                    if entry.position == node.position {
                        entry.apply_reward(reward as f64, self.learning_rate);
                    }
                }
            }
            NodeOrGuide::Guide(guide) => {
                // Sacred guides reinforce anchor proximity
                let mut ladder = self.ladder.write().await;
                for entry in ladder.iter_mut() {
                    // Entries near sacred positions get bonus
                    let distance = (entry.position as i32 - guide.position as i32).abs();
                    if distance <= 1 {
                        entry.apply_reward(0.5, self.learning_rate);
                    }
                }
            }
        }
    }

    /// Update ladder rankings
    async fn update_ladder_rankings(&self) {
        let mut ladder = self.ladder.write().await;

        // Get top entries for comparison
        ladder.sort_by(|a, b| b.rank.partial_cmp(&a.rank).unwrap_or(std::cmp::Ordering::Equal));
        // Clone top entries for comparison to avoid borrow conflict
        let top_beams: Vec<BeamTensor> = ladder.iter().take(10).map(|e| e.beam.clone()).collect();

        // Recalculate all ranks
        for entry in ladder.iter_mut() {
            entry.calculate_rank_from_beams(&top_beams);
        }

        // Re-sort
        ladder.sort_by(|a, b| b.rank.partial_cmp(&a.rank).unwrap_or(std::cmp::Ordering::Equal));
    }

    /// Add a new subject (creates FluxMatrix)
    pub async fn add_subject(&self, name: &str) -> Subject {
        let subject = Subject::new(name.to_string());
        let mut subjects = self.subjects.write().await;
        subjects.insert(name.to_string(), subject.clone());

        // Add ladder entries for each node
        let mut ladder = self.ladder.write().await;
        let mut next_id = self.next_entry_id.write().await;

        for (&pos, node) in &subject.nodes {
            let entry = LadderEntry::new(*next_id, name.to_string(), pos, node.beam.clone());
            *next_id += 1;
            ladder.push(entry);
        }

        subject
    }

    /// Learn from source (RAG, tool, internet)
    pub async fn learn_from_source(&self, content: &str, source_type: SourceType) {
        let beams = self.text_to_beams(content);
        let cycle = self.global_latent.read().await.step as u64;

        // Store in source memory
        let entry = SourceEntry {
            content: content.to_string(),
            source_type,
            beams: beams.clone(),
            relevance: 1.0,
            cycle_learned: cycle,
        };

        let mut memory = self.source_memory.write().await;
        memory.push(entry);

        // Limit memory size
        if memory.len() > 10000 {
            memory.remove(0);
        }

        // Run through vortex to integrate
        drop(memory);
        self.run_cycle(&beams).await;
    }

    /// Convert text to beams (simplified)
    fn text_to_beams(&self, text: &str) -> Vec<BeamTensor> {
        let text_lower = text.to_lowercase();
        let words: Vec<&str> = text_lower
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .collect();

        words.iter().map(|word| {
            let mut beam = BeamTensor::default();
            let hash = word.bytes().fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
            for i in 0..9 {
                beam.digits[i] = ((hash >> (i * 7)) & 0x7F) as f32 / 127.0;
            }
            let sum: f32 = beam.digits.iter().sum();
            if sum > 0.0 {
                beam.digits.iter_mut().for_each(|d| *d /= sum);
            }
            beam.confidence = 0.5;
            beam
        }).collect()
    }

    /// Get current state summary
    pub async fn state_summary(&self) -> VortexState {
        let global = self.global_latent.read().await;
        let subjects = self.subjects.read().await;
        let ladder = self.ladder.read().await;
        let memory = self.source_memory.read().await;

        VortexState {
            cycle: global.step as u64,
            energy: global.energy,
            sacred_alignment: global.sacred_alignment,
            subject_count: subjects.len(),
            ladder_entries: ladder.len(),
            source_memories: memory.len(),
            top_ranked: ladder.first().map(|e| e.subject_name.clone()),
        }
    }
}

/// Summary of vortex state
#[derive(Debug, Clone)]
pub struct VortexState {
    pub cycle: u64,
    pub energy: f32,
    pub sacred_alignment: f32,
    pub subject_count: usize,
    pub ladder_entries: usize,
    pub source_memories: usize,
    pub top_ranked: Option<String>,
}

impl Default for VortexRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_vortex_runner_creation() {
        let runner = VortexRunner::new();
        let state = runner.state_summary().await;
        assert_eq!(state.cycle, 0);
        assert_eq!(state.subject_count, 0);
    }

    #[tokio::test]
    async fn test_add_subject() {
        let runner = VortexRunner::new();
        let subject = runner.add_subject("mathematics").await;
        assert_eq!(subject.name, "mathematics");
        assert_eq!(subject.nodes.len(), 7); // 0-9 minus 3,6,9
        assert_eq!(subject.sacred_guides.len(), 3);
    }

    #[tokio::test]
    async fn test_run_cycle() {
        let runner = VortexRunner::new();
        runner.add_subject("test").await;

        let beams = runner.text_to_beams("hello world");
        let latent = runner.run_cycle(&beams).await;

        assert!(latent.energy > 0.0);
        let state = runner.state_summary().await;
        assert!(state.cycle > 0);
    }

    #[tokio::test]
    async fn test_exponential_run() {
        let runner = VortexRunner::new();
        runner.add_subject("learning").await;

        let beams = runner.text_to_beams("exponential growth");
        let latent = runner.run_exponential(&beams, 4).await; // 2^4 = 16 cycles

        let state = runner.state_summary().await;
        assert!(state.cycle >= 16);
    }

    #[tokio::test]
    async fn test_learn_from_source() {
        let runner = VortexRunner::new();
        runner.learn_from_source("The quick brown fox", SourceType::UserInput).await;

        let state = runner.state_summary().await;
        assert_eq!(state.source_memories, 1);
    }
}
