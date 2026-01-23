//! Transfer Learning Module for AGI
//!
//! Enables cross-domain knowledge application:
//! - Domain abstraction (extract transferable principles)
//! - Analogical reasoning (map concepts between domains)
//! - Knowledge distillation (compress domain expertise)
//! - Skill composition (combine learned abilities)
//!
//! ## Architecture
//!
//! ```text
//! Source Domain → Abstract Principles → Target Domain
//!       ↓              ↓                    ↓
//!   Concrete       Transferable         Applied
//!   Knowledge      Patterns             Knowledge
//! ```

use crate::data::models::ELPTensor;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ============================================================================
// Core Transfer Learning Structures
// ============================================================================

/// A domain of knowledge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Domain {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub elp_profile: ELPTensor,
    pub concepts: Vec<Concept>,
    pub principles: Vec<Principle>,
    pub skills: Vec<Skill>,
    pub created_at: DateTime<Utc>,
}

/// A concept within a domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Concept {
    pub id: Uuid,
    pub name: String,
    pub definition: String,
    pub abstraction_level: f32, // 0.0 = concrete, 1.0 = abstract
    pub elp_weights: ELPTensor,
    pub related_concepts: Vec<Uuid>,
}

/// An abstract principle that can transfer between domains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Principle {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub abstraction: String, // Abstract form of the principle
    pub source_domains: Vec<Uuid>,
    pub applicability_score: f32,
    pub elp_signature: ELPTensor,
}

/// A learned skill that can be composed or transferred
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub proficiency: f32, // 0.0-1.0
    pub prerequisites: Vec<Uuid>,
    pub composable_with: Vec<Uuid>,
    pub domain_specificity: f32, // 0.0 = general, 1.0 = domain-specific
}

/// An analogy mapping between domains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Analogy {
    pub id: Uuid,
    pub source_domain: Uuid,
    pub target_domain: Uuid,
    pub concept_mappings: Vec<ConceptMapping>,
    pub structural_similarity: f32,
    pub transfer_success_rate: f32,
    pub created_at: DateTime<Utc>,
}

/// Mapping between concepts in different domains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptMapping {
    pub source_concept: Uuid,
    pub target_concept: Uuid,
    pub mapping_type: MappingType,
    pub confidence: f32,
    pub explanation: String,
}

/// Types of concept mappings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MappingType {
    /// Direct equivalence
    Equivalent,
    /// Source is more general
    Generalization,
    /// Source is more specific
    Specialization,
    /// Structural similarity
    Analogous,
    /// Functional similarity
    Functional,
    /// Causal relationship preserved
    Causal,
}

// ============================================================================
// Transfer Learning Engine
// ============================================================================

/// Main transfer learning system
pub struct TransferLearningEngine {
    /// Known domains
    pub domains: HashMap<Uuid, Domain>,
    
    /// Discovered analogies
    pub analogies: Vec<Analogy>,
    
    /// Abstract principles library
    pub principles: HashMap<Uuid, Principle>,
    
    /// Transfer history for learning
    pub transfer_history: Vec<TransferAttempt>,
    
    /// Statistics
    pub stats: TransferStats,
}

/// Record of a transfer attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferAttempt {
    pub id: Uuid,
    pub source_domain: Uuid,
    pub target_domain: Uuid,
    pub principle_used: Option<Uuid>,
    pub analogy_used: Option<Uuid>,
    pub success: bool,
    pub effectiveness: f32,
    pub timestamp: DateTime<Utc>,
}

/// Statistics for transfer learning
#[derive(Debug, Clone, Default)]
pub struct TransferStats {
    pub domains_learned: u64,
    pub principles_extracted: u64,
    pub analogies_discovered: u64,
    pub transfers_attempted: u64,
    pub transfers_successful: u64,
    pub avg_transfer_effectiveness: f32,
}

impl Default for TransferLearningEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TransferLearningEngine {
    pub fn new() -> Self {
        Self {
            domains: HashMap::new(),
            analogies: Vec::new(),
            principles: HashMap::new(),
            transfer_history: Vec::new(),
            stats: TransferStats::default(),
        }
    }
    
    /// Register a new domain
    pub fn register_domain(&mut self, name: &str, description: &str, elp: &ELPTensor) -> Uuid {
        let domain = Domain {
            id: Uuid::new_v4(),
            name: name.to_string(),
            description: description.to_string(),
            elp_profile: elp.clone(),
            concepts: Vec::new(),
            principles: Vec::new(),
            skills: Vec::new(),
            created_at: Utc::now(),
        };
        
        let id = domain.id;
        self.domains.insert(id, domain);
        self.stats.domains_learned += 1;
        
        tracing::info!("Registered domain: {} ({})", name, id);
        id
    }
    
    /// Add a concept to a domain
    pub fn add_concept(
        &mut self,
        domain_id: Uuid,
        name: &str,
        definition: &str,
        abstraction_level: f32,
        elp: &ELPTensor,
    ) -> Option<Uuid> {
        let concept = Concept {
            id: Uuid::new_v4(),
            name: name.to_string(),
            definition: definition.to_string(),
            abstraction_level,
            elp_weights: elp.clone(),
            related_concepts: Vec::new(),
        };
        
        let concept_id = concept.id;
        
        if let Some(domain) = self.domains.get_mut(&domain_id) {
            domain.concepts.push(concept);
            Some(concept_id)
        } else {
            None
        }
    }
    
    /// Add a skill to a domain
    pub fn add_skill(
        &mut self,
        domain_id: Uuid,
        name: &str,
        description: &str,
        proficiency: f32,
        domain_specificity: f32,
    ) -> Option<Uuid> {
        let skill = Skill {
            id: Uuid::new_v4(),
            name: name.to_string(),
            description: description.to_string(),
            proficiency,
            prerequisites: Vec::new(),
            composable_with: Vec::new(),
            domain_specificity,
        };
        
        let skill_id = skill.id;
        
        if let Some(domain) = self.domains.get_mut(&domain_id) {
            domain.skills.push(skill);
            Some(skill_id)
        } else {
            None
        }
    }
    
    /// Extract abstract principle from domain knowledge
    pub fn extract_principle(
        &mut self,
        domain_id: Uuid,
        name: &str,
        description: &str,
        abstraction: &str,
    ) -> Option<Uuid> {
        let (elp_profile, domain_name) = {
            let domain = self.domains.get(&domain_id)?;
            (domain.elp_profile.clone(), domain.name.clone())
        };
        
        let principle = Principle {
            id: Uuid::new_v4(),
            name: name.to_string(),
            description: description.to_string(),
            abstraction: abstraction.to_string(),
            source_domains: vec![domain_id],
            applicability_score: 0.5, // Initial score
            elp_signature: elp_profile,
        };
        
        let principle_id = principle.id;
        self.principles.insert(principle_id, principle.clone());
        
        // Add to domain
        if let Some(d) = self.domains.get_mut(&domain_id) {
            d.principles.push(principle);
        }
        
        self.stats.principles_extracted += 1;
        tracing::info!("Extracted principle: {} from domain {}", name, domain_name);
        
        Some(principle_id)
    }
    
    /// Discover analogy between two domains
    pub fn discover_analogy(&mut self, source_id: Uuid, target_id: Uuid) -> Option<Analogy> {
        let source = self.domains.get(&source_id)?;
        let target = self.domains.get(&target_id)?;
        
        // Calculate structural similarity based on ELP profiles
        let elp_similarity = self.calculate_elp_similarity(&source.elp_profile, &target.elp_profile);
        
        // Find concept mappings
        let mappings = self.find_concept_mappings(source, target);
        
        if mappings.is_empty() {
            return None;
        }
        
        let avg_confidence = mappings.iter().map(|m| m.confidence).sum::<f32>() / mappings.len() as f32;
        
        let analogy = Analogy {
            id: Uuid::new_v4(),
            source_domain: source_id,
            target_domain: target_id,
            concept_mappings: mappings,
            structural_similarity: elp_similarity,
            transfer_success_rate: avg_confidence,
            created_at: Utc::now(),
        };
        
        self.analogies.push(analogy.clone());
        self.stats.analogies_discovered += 1;
        
        tracing::info!("Discovered analogy: {} → {} (similarity: {:.2})", 
            source.name, target.name, elp_similarity);
        
        Some(analogy)
    }
    
    /// Transfer knowledge from source to target domain
    pub fn transfer_knowledge(
        &mut self,
        source_id: Uuid,
        target_id: Uuid,
        principle_id: Option<Uuid>,
    ) -> TransferResult {
        self.stats.transfers_attempted += 1;
        
        let source = match self.domains.get(&source_id) {
            Some(d) => d.clone(),
            None => return TransferResult::failed("Source domain not found"),
        };
        
        let target = match self.domains.get(&target_id) {
            Some(d) => d.clone(),
            None => return TransferResult::failed("Target domain not found"),
        };
        
        // Find or create analogy
        let analogy = self.analogies.iter()
            .find(|a| a.source_domain == source_id && a.target_domain == target_id)
            .cloned()
            .or_else(|| self.discover_analogy(source_id, target_id));
        
        // Calculate transfer effectiveness
        let effectiveness = self.calculate_transfer_effectiveness(
            &source,
            &target,
            analogy.as_ref(),
            principle_id.and_then(|id| self.principles.get(&id)),
        );
        
        let success = effectiveness > 0.5;
        
        // Record attempt
        let attempt = TransferAttempt {
            id: Uuid::new_v4(),
            source_domain: source_id,
            target_domain: target_id,
            principle_used: principle_id,
            analogy_used: analogy.as_ref().map(|a| a.id),
            success,
            effectiveness,
            timestamp: Utc::now(),
        };
        
        self.transfer_history.push(attempt);
        
        if success {
            self.stats.transfers_successful += 1;
            
            // Update average effectiveness
            let total = self.stats.transfers_attempted as f32;
            self.stats.avg_transfer_effectiveness = 
                (self.stats.avg_transfer_effectiveness * (total - 1.0) + effectiveness) / total;
            
            // Transfer applicable skills
            let transferred_skills = self.transfer_skills(&source, &target);
            
            TransferResult {
                success: true,
                effectiveness,
                transferred_concepts: analogy.as_ref()
                    .map(|a| a.concept_mappings.len())
                    .unwrap_or(0),
                transferred_skills: transferred_skills.len(),
                new_principles: Vec::new(),
                explanation: format!(
                    "Successfully transferred knowledge from {} to {} with {:.0}% effectiveness",
                    source.name, target.name, effectiveness * 100.0
                ),
            }
        } else {
            TransferResult {
                success: false,
                effectiveness,
                transferred_concepts: 0,
                transferred_skills: 0,
                new_principles: Vec::new(),
                explanation: format!(
                    "Transfer from {} to {} failed (effectiveness: {:.0}%)",
                    source.name, target.name, effectiveness * 100.0
                ),
            }
        }
    }
    
    /// Compose multiple skills into a new compound skill
    pub fn compose_skills(&mut self, skill_ids: &[Uuid], new_name: &str, domain_id: Uuid) -> Option<Skill> {
        let skills: Vec<_> = skill_ids.iter()
            .filter_map(|id| {
                self.domains.values()
                    .flat_map(|d| &d.skills)
                    .find(|s| s.id == *id)
                    .cloned()
            })
            .collect();
        
        if skills.len() < 2 {
            return None;
        }
        
        // Calculate composed proficiency (geometric mean)
        let proficiency = skills.iter()
            .map(|s| s.proficiency)
            .product::<f32>()
            .powf(1.0 / skills.len() as f32);
        
        // Calculate domain specificity (average)
        let specificity = skills.iter()
            .map(|s| s.domain_specificity)
            .sum::<f32>() / skills.len() as f32;
        
        let composed = Skill {
            id: Uuid::new_v4(),
            name: new_name.to_string(),
            description: format!("Composed from: {}", 
                skills.iter().map(|s| s.name.as_str()).collect::<Vec<_>>().join(", ")),
            proficiency,
            prerequisites: skill_ids.to_vec(),
            composable_with: Vec::new(),
            domain_specificity: specificity,
        };
        
        // Add to domain
        if let Some(domain) = self.domains.get_mut(&domain_id) {
            domain.skills.push(composed.clone());
        }
        
        tracing::info!("Composed skill: {} (proficiency: {:.2})", new_name, proficiency);
        
        Some(composed)
    }
    
    /// Get transferable principles for a target domain
    pub fn get_applicable_principles(&self, target_id: Uuid) -> Vec<&Principle> {
        let target = match self.domains.get(&target_id) {
            Some(d) => d,
            None => return Vec::new(),
        };
        
        self.principles.values()
            .filter(|p| {
                // Check ELP compatibility
                let similarity = self.calculate_elp_similarity(&p.elp_signature, &target.elp_profile);
                similarity > 0.5 && p.applicability_score > 0.4
            })
            .collect()
    }
    
    /// Get statistics
    pub fn get_stats(&self) -> &TransferStats {
        &self.stats
    }
    
    // ========================================================================
    // Private helpers
    // ========================================================================
    
    fn calculate_elp_similarity(&self, elp1: &ELPTensor, elp2: &ELPTensor) -> f32 {
        // Cosine similarity in ELP space
        let dot = elp1.ethos * elp2.ethos + elp1.logos * elp2.logos + elp1.pathos * elp2.pathos;
        let mag1 = (elp1.ethos.powi(2) + elp1.logos.powi(2) + elp1.pathos.powi(2)).sqrt();
        let mag2 = (elp2.ethos.powi(2) + elp2.logos.powi(2) + elp2.pathos.powi(2)).sqrt();
        
        if mag1 > 0.0 && mag2 > 0.0 {
            (dot / (mag1 * mag2)) as f32
        } else {
            0.0
        }
    }
    
    fn find_concept_mappings(&self, source: &Domain, target: &Domain) -> Vec<ConceptMapping> {
        let mut mappings = Vec::new();
        
        for s_concept in &source.concepts {
            for t_concept in &target.concepts {
                // Check for potential mapping based on:
                // 1. Similar abstraction level
                // 2. Similar ELP weights
                // 3. Name/definition similarity (simplified)
                
                let abstraction_diff = (s_concept.abstraction_level - t_concept.abstraction_level).abs();
                let elp_sim = self.calculate_elp_similarity(&s_concept.elp_weights, &t_concept.elp_weights);
                
                // Simple name similarity check
                let name_overlap = s_concept.name.to_lowercase()
                    .split_whitespace()
                    .filter(|w| t_concept.name.to_lowercase().contains(w))
                    .count();
                
                let confidence = (1.0 - abstraction_diff) * 0.3 + elp_sim * 0.5 + (name_overlap as f32 * 0.1).min(0.2);
                
                if confidence > 0.4 {
                    let mapping_type = if abstraction_diff < 0.1 {
                        if elp_sim > 0.9 { MappingType::Equivalent } else { MappingType::Analogous }
                    } else if s_concept.abstraction_level > t_concept.abstraction_level {
                        MappingType::Generalization
                    } else {
                        MappingType::Specialization
                    };
                    
                    mappings.push(ConceptMapping {
                        source_concept: s_concept.id,
                        target_concept: t_concept.id,
                        mapping_type,
                        confidence,
                        explanation: format!(
                            "{} maps to {} via {:?}",
                            s_concept.name, t_concept.name, mapping_type
                        ),
                    });
                }
            }
        }
        
        mappings
    }
    
    fn calculate_transfer_effectiveness(
        &self,
        source: &Domain,
        target: &Domain,
        analogy: Option<&Analogy>,
        principle: Option<&Principle>,
    ) -> f32 {
        let mut effectiveness = 0.0;
        
        // Base effectiveness from ELP similarity
        let elp_sim = self.calculate_elp_similarity(&source.elp_profile, &target.elp_profile);
        effectiveness += elp_sim * 0.3;
        
        // Boost from analogy
        if let Some(a) = analogy {
            effectiveness += a.structural_similarity * 0.3;
            effectiveness += a.transfer_success_rate * 0.2;
        }
        
        // Boost from principle
        if let Some(p) = principle {
            effectiveness += p.applicability_score * 0.2;
        }
        
        effectiveness.min(1.0)
    }
    
    fn transfer_skills(&self, source: &Domain, target: &Domain) -> Vec<Skill> {
        source.skills.iter()
            .filter(|s| s.domain_specificity < 0.5) // Only transfer general skills
            .cloned()
            .collect()
    }
}

/// Result of a transfer attempt
#[derive(Debug, Clone)]
pub struct TransferResult {
    pub success: bool,
    pub effectiveness: f32,
    pub transferred_concepts: usize,
    pub transferred_skills: usize,
    pub new_principles: Vec<Uuid>,
    pub explanation: String,
}

impl TransferResult {
    fn failed(reason: &str) -> Self {
        Self {
            success: false,
            effectiveness: 0.0,
            transferred_concepts: 0,
            transferred_skills: 0,
            new_principles: Vec::new(),
            explanation: reason.to_string(),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_domain_registration() {
        let mut engine = TransferLearningEngine::new();
        let elp = ELPTensor { ethos: 5.0, logos: 8.0, pathos: 4.0 };
        
        let id = engine.register_domain("Mathematics", "Study of numbers and patterns", &elp);
        
        assert!(engine.domains.contains_key(&id));
        assert_eq!(engine.stats.domains_learned, 1);
    }
    
    #[test]
    fn test_concept_addition() {
        let mut engine = TransferLearningEngine::new();
        let elp = ELPTensor { ethos: 5.0, logos: 8.0, pathos: 4.0 };
        
        let domain_id = engine.register_domain("Physics", "Study of matter and energy", &elp);
        let concept_id = engine.add_concept(
            domain_id,
            "Force",
            "Push or pull on an object",
            0.3,
            &elp,
        );
        
        assert!(concept_id.is_some());
        assert_eq!(engine.domains.get(&domain_id).unwrap().concepts.len(), 1);
    }
    
    #[test]
    fn test_principle_extraction() {
        let mut engine = TransferLearningEngine::new();
        let elp = ELPTensor { ethos: 5.0, logos: 9.0, pathos: 3.0 };
        
        let domain_id = engine.register_domain("Economics", "Study of resource allocation", &elp);
        let principle_id = engine.extract_principle(
            domain_id,
            "Supply and Demand",
            "Price is determined by supply and demand",
            "Equilibrium emerges from opposing forces",
        );
        
        assert!(principle_id.is_some());
        assert_eq!(engine.stats.principles_extracted, 1);
    }
    
    #[test]
    fn test_analogy_discovery() {
        let mut engine = TransferLearningEngine::new();
        
        let elp1 = ELPTensor { ethos: 5.0, logos: 8.0, pathos: 4.0 };
        let elp2 = ELPTensor { ethos: 6.0, logos: 7.0, pathos: 5.0 };
        
        let physics = engine.register_domain("Physics", "Study of matter", &elp1);
        let economics = engine.register_domain("Economics", "Study of markets", &elp2);
        
        // Add similar concepts
        engine.add_concept(physics, "Equilibrium", "State of balance", 0.5, &elp1);
        engine.add_concept(economics, "Market Equilibrium", "Supply equals demand", 0.5, &elp2);
        
        let analogy = engine.discover_analogy(physics, economics);
        
        assert!(analogy.is_some());
        assert!(analogy.unwrap().structural_similarity > 0.5);
    }
    
    #[test]
    fn test_knowledge_transfer() {
        let mut engine = TransferLearningEngine::new();
        
        let elp1 = ELPTensor { ethos: 5.0, logos: 8.0, pathos: 4.0 };
        let elp2 = ELPTensor { ethos: 5.0, logos: 7.0, pathos: 5.0 };
        
        let source = engine.register_domain("Source", "Source domain", &elp1);
        let target = engine.register_domain("Target", "Target domain", &elp2);
        
        // Add transferable skill
        engine.add_skill(source, "Analysis", "Analytical thinking", 0.8, 0.3);
        
        let result = engine.transfer_knowledge(source, target, None);
        
        // Transfer should be attempted
        assert_eq!(engine.stats.transfers_attempted, 1);
    }
    
    #[test]
    fn test_skill_composition() {
        let mut engine = TransferLearningEngine::new();
        let elp = ELPTensor { ethos: 6.0, logos: 7.0, pathos: 5.0 };
        
        let domain = engine.register_domain("General", "General skills", &elp);
        
        let skill1 = engine.add_skill(domain, "Reading", "Reading comprehension", 0.9, 0.2).unwrap();
        let skill2 = engine.add_skill(domain, "Writing", "Written communication", 0.8, 0.2).unwrap();
        
        let composed = engine.compose_skills(&[skill1, skill2], "Communication", domain);
        
        assert!(composed.is_some());
        let skill = composed.unwrap();
        assert!(skill.proficiency > 0.7); // Geometric mean of 0.9 and 0.8
    }
}
