//! Curiosity Engine for AGI
//!
//! Enables intrinsic motivation through:
//! - Curiosity-driven exploration (actively seek novel information)
//! - Information gain maximization
//! - Exploration/exploitation balance
//! - Active hypothesis testing

use crate::data::models::ELPTensor;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet, BinaryHeap};
use std::cmp::Ordering;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeGap {
    pub id: Uuid,
    pub domain: String,
    pub description: String,
    pub uncertainty: f32,
    pub information_gain: f32,
    pub elp_relevance: ELPTensor,
    pub exploration_count: u32,
    pub last_explored: Option<DateTime<Utc>>,
}

impl PartialEq for KnowledgeGap {
    fn eq(&self, other: &Self) -> bool { self.id == other.id }
}

impl Eq for KnowledgeGap {}

impl PartialOrd for KnowledgeGap {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl Ord for KnowledgeGap {
    fn cmp(&self, other: &Self) -> Ordering {
        self.information_gain.partial_cmp(&other.information_gain).unwrap_or(Ordering::Equal)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hypothesis {
    pub id: Uuid,
    pub statement: String,
    pub confidence: f32,
    pub evidence_for: Vec<Evidence>,
    pub evidence_against: Vec<Evidence>,
    pub status: HypothesisStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HypothesisStatus { Proposed, Testing, Confirmed, Refuted, Uncertain }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub source: String,
    pub description: String,
    pub strength: f32,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorationAction {
    pub id: Uuid,
    pub action_type: ActionType,
    pub target: String,
    pub expected_gain: f32,
    pub actual_gain: Option<f32>,
    pub executed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    Query { question: String },
    Experiment { hypothesis_id: Uuid },
    DeepDive { topic: String },
    CrossReference { topics: Vec<String> },
}

pub struct CuriosityEngine {
    pub knowledge_gaps: BinaryHeap<KnowledgeGap>,
    pub hypotheses: HashMap<Uuid, Hypothesis>,
    pub explored_topics: HashSet<String>,
    pub exploration_history: Vec<ExplorationAction>,
    pub surprise_threshold: f32,
    pub exploration_rate: f32,
    pub stats: CuriosityStats,
}

#[derive(Debug, Clone, Default)]
pub struct CuriosityStats {
    pub gaps_identified: u64,
    pub gaps_filled: u64,
    pub hypotheses_tested: u64,
    pub hypotheses_confirmed: u64,
    pub total_information_gain: f64,
    pub exploration_actions: u64,
}

impl Default for CuriosityEngine {
    fn default() -> Self { Self::new() }
}

impl CuriosityEngine {
    pub fn new() -> Self {
        Self {
            knowledge_gaps: BinaryHeap::new(),
            hypotheses: HashMap::new(),
            explored_topics: HashSet::new(),
            exploration_history: Vec::new(),
            surprise_threshold: 0.5,
            exploration_rate: 0.3,
            stats: CuriosityStats::default(),
        }
    }
    
    /// Identify a knowledge gap from high uncertainty
    pub fn identify_gap(&mut self, domain: &str, description: &str, uncertainty: f32, elp: &ELPTensor) {
        let info_gain = self.estimate_information_gain(domain, uncertainty);
        
        let gap = KnowledgeGap {
            id: Uuid::new_v4(),
            domain: domain.to_string(),
            description: description.to_string(),
            uncertainty,
            information_gain: info_gain,
            elp_relevance: elp.clone(),
            exploration_count: 0,
            last_explored: None,
        };
        
        self.knowledge_gaps.push(gap);
        self.stats.gaps_identified += 1;
        tracing::debug!("Knowledge gap identified: {} (gain: {:.2})", description, info_gain);
    }
    
    fn estimate_information_gain(&self, domain: &str, uncertainty: f32) -> f32 {
        let novelty = if self.explored_topics.contains(domain) { 0.5 } else { 1.0 };
        let urgency = uncertainty;
        novelty * urgency
    }
    
    /// Get the most valuable knowledge gap to explore
    pub fn get_most_curious(&mut self) -> Option<KnowledgeGap> {
        self.knowledge_gaps.pop()
    }
    
    /// Decide whether to explore or exploit
    pub fn should_explore(&self) -> bool {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        rng.gen::<f32>() < self.exploration_rate
    }
    
    /// Generate an exploration action for a knowledge gap
    pub fn generate_exploration(&self, gap: &KnowledgeGap) -> ExplorationAction {
        let action_type = if gap.exploration_count == 0 {
            ActionType::Query { question: format!("What is {}?", gap.description) }
        } else if gap.exploration_count < 3 {
            ActionType::DeepDive { topic: gap.domain.clone() }
        } else {
            ActionType::CrossReference { topics: vec![gap.domain.clone(), "related".to_string()] }
        };
        
        ExplorationAction {
            id: Uuid::new_v4(),
            action_type,
            target: gap.description.clone(),
            expected_gain: gap.information_gain,
            actual_gain: None,
            executed_at: None,
        }
    }
    
    /// Record exploration result
    pub fn record_exploration(&mut self, mut action: ExplorationAction, actual_gain: f32, filled: bool) {
        action.actual_gain = Some(actual_gain);
        action.executed_at = Some(Utc::now());
        
        if let ActionType::DeepDive { ref topic } = action.action_type {
            self.explored_topics.insert(topic.clone());
        }
        
        self.exploration_history.push(action);
        self.stats.exploration_actions += 1;
        self.stats.total_information_gain += actual_gain as f64;
        
        if filled { self.stats.gaps_filled += 1; }
    }
    
    /// Propose a hypothesis based on observations
    pub fn propose_hypothesis(&mut self, statement: &str, initial_confidence: f32) -> Hypothesis {
        let hypothesis = Hypothesis {
            id: Uuid::new_v4(),
            statement: statement.to_string(),
            confidence: initial_confidence,
            evidence_for: Vec::new(),
            evidence_against: Vec::new(),
            status: HypothesisStatus::Proposed,
            created_at: Utc::now(),
        };
        
        self.hypotheses.insert(hypothesis.id, hypothesis.clone());
        hypothesis
    }
    
    /// Add evidence to a hypothesis
    pub fn add_evidence(&mut self, hypothesis_id: Uuid, evidence: Evidence, supports: bool) {
        if let Some(h) = self.hypotheses.get_mut(&hypothesis_id) {
            let weight = evidence.strength;
            
            if supports {
                h.evidence_for.push(evidence);
                h.confidence = (h.confidence + weight * 0.1).min(1.0);
            } else {
                h.evidence_against.push(evidence);
                h.confidence = (h.confidence - weight * 0.1).max(0.0);
            }
            
            // Update status based on confidence
            h.status = match h.confidence {
                c if c > 0.8 => HypothesisStatus::Confirmed,
                c if c < 0.2 => HypothesisStatus::Refuted,
                _ => HypothesisStatus::Testing,
            };
        }
    }
    
    /// Test a hypothesis by generating an experiment
    pub fn test_hypothesis(&mut self, hypothesis_id: Uuid) -> Option<ExplorationAction> {
        if let Some(h) = self.hypotheses.get_mut(&hypothesis_id) {
            h.status = HypothesisStatus::Testing;
            self.stats.hypotheses_tested += 1;
            
            Some(ExplorationAction {
                id: Uuid::new_v4(),
                action_type: ActionType::Experiment { hypothesis_id },
                target: h.statement.clone(),
                expected_gain: 0.5,
                actual_gain: None,
                executed_at: None,
            })
        } else {
            None
        }
    }
    
    /// Get statistics
    pub fn get_stats(&self) -> &CuriosityStats {
        &self.stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_knowledge_gap() {
        let mut engine = CuriosityEngine::new();
        let elp = ELPTensor { ethos: 5.0, logos: 7.0, pathos: 3.0 };
        engine.identify_gap("physics", "quantum entanglement", 0.9, &elp);
        assert_eq!(engine.stats.gaps_identified, 1);
        
        let gap = engine.get_most_curious();
        assert!(gap.is_some());
    }
    
    #[test]
    fn test_hypothesis() {
        let mut engine = CuriosityEngine::new();
        let h = engine.propose_hypothesis("Vortex math improves reasoning", 0.5);
        
        let evidence = Evidence {
            source: "experiment".to_string(),
            description: "40% better context".to_string(),
            strength: 0.8,
            timestamp: Utc::now(),
        };
        
        engine.add_evidence(h.id, evidence, true);
        let updated = engine.hypotheses.get(&h.id).unwrap();
        assert!(updated.confidence > 0.5);
    }
    
    #[test]
    fn test_exploration() {
        let mut engine = CuriosityEngine::new();
        let elp = ELPTensor { ethos: 5.0, logos: 7.0, pathos: 3.0 };
        engine.identify_gap("math", "calculus", 0.8, &elp);
        
        let gap = engine.get_most_curious().unwrap();
        let action = engine.generate_exploration(&gap);
        
        engine.record_exploration(action, 0.7, true);
        assert_eq!(engine.stats.gaps_filled, 1);
    }
}
