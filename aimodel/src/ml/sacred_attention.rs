//! Sacred Attention Headers - 3-6-9 Verification Pipeline
//!
//! Implements the sacred attention verification system for web data processing.
//! Each sacred position (3, 6, 9) serves a specific verification function:
//!
//! - **Position 3**: Keyword Extraction - What is this about?
//! - **Position 6**: Relation Verification - How does this connect?
//! - **Position 9**: Knowledge Integration - Verified, store it
//!
//! ## Architecture
//! ```text
//! Raw Web Data
//!      ↓
//! [Position 3] Keyword Extraction
//!   - Extract subjects, objects, actions
//!   - Build keyword index
//!   - Identify entity types
//!      ↓
//! [Position 6] Relation Verification
//!   - Verify against existing knowledge
//!   - Check geometric consistency
//!   - Filter contradictions
//!      ↓
//! [Position 9] Knowledge Integration
//!   - Store verified facts
//!   - Update embeddings
//!   - Increment confidence
//! ```

use std::collections::HashMap;
use crate::ml::geometric_world_model::{GeometricWorldModel, WorldEncoder, GeometricRelationPredictor};

// =============================================================================
// SACRED POSITIONS
// =============================================================================

/// Sacred position constants
pub const SACRED_POSITION_3: u8 = 3; // Keyword Extraction
pub const SACRED_POSITION_6: u8 = 6; // Relation Verification
pub const SACRED_POSITION_9: u8 = 9; // Knowledge Integration

/// Vortex cycle for non-sacred positions
pub const VORTEX_CYCLE: [u8; 6] = [1, 2, 4, 8, 7, 5];

// =============================================================================
// EXTRACTED ENTITY
// =============================================================================

/// An entity extracted from text
#[derive(Debug, Clone)]
pub struct ExtractedEntity {
    /// Entity text
    pub text: String,
    /// Entity type (person, place, thing, action, etc.)
    pub entity_type: EntityType,
    /// Position in original text
    pub position: usize,
    /// Confidence score
    pub confidence: f32,
    /// Associated keywords
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EntityType {
    Person,
    Place,
    Thing,
    Action,
    Property,
    Quantity,
    Time,
    Unknown,
}

impl EntityType {
    /// Infer entity type from context
    pub fn infer(word: &str, context: &str) -> Self {
        let word_lower = word.to_lowercase();
        let context_lower = context.to_lowercase();

        // Place indicators
        let place_words = ["restaurant", "kitchen", "bathroom", "office", "store", "shop",
            "library", "hospital", "school", "park", "beach", "forest", "city", "country",
            "room", "house", "building", "street", "road", "airport", "station"];
        if place_words.iter().any(|p| word_lower.contains(p)) {
            return EntityType::Place;
        }

        // Person indicators (names often capitalized, or role words)
        let person_words = ["person", "man", "woman", "child", "doctor", "teacher",
            "worker", "employee", "customer", "friend", "family"];
        if person_words.iter().any(|p| word_lower.contains(p)) {
            return EntityType::Person;
        }

        // Action indicators (verbs)
        let action_words = ["run", "walk", "eat", "drink", "sleep", "work", "play",
            "make", "create", "build", "destroy", "move", "go", "come", "take", "give"];
        if action_words.iter().any(|a| word_lower.contains(a)) {
            return EntityType::Action;
        }

        // Property indicators
        let property_words = ["big", "small", "large", "tiny", "hot", "cold", "warm",
            "fast", "slow", "hard", "soft", "heavy", "light", "old", "new", "young"];
        if property_words.iter().any(|p| word_lower.contains(p)) {
            return EntityType::Property;
        }

        // Quantity indicators
        if word_lower.chars().any(|c| c.is_numeric()) {
            return EntityType::Quantity;
        }

        // Time indicators
        let time_words = ["today", "tomorrow", "yesterday", "morning", "evening",
            "night", "day", "week", "month", "year", "hour", "minute", "second"];
        if time_words.iter().any(|t| word_lower.contains(t)) {
            return EntityType::Time;
        }

        // Default to Thing
        EntityType::Thing
    }
}

// =============================================================================
// EXTRACTED RELATION
// =============================================================================

/// A relation extracted from text
#[derive(Debug, Clone)]
pub struct ExtractedRelation {
    /// Source entity
    pub source: String,
    /// Relation type
    pub relation_type: String,
    /// Target entity
    pub target: String,
    /// Confidence score
    pub confidence: f32,
    /// Whether this has been verified
    pub verified: bool,
}

// =============================================================================
// VERIFIED FACT
// =============================================================================

/// A verified fact ready for integration
#[derive(Debug, Clone)]
pub struct VerifiedFact {
    /// Subject of the fact
    pub subject: String,
    /// Attribute/property
    pub attribute: String,
    /// Value
    pub value: String,
    /// Confidence after verification
    pub confidence: f32,
    /// Source of the fact
    pub source: String,
    /// Keywords for indexing
    pub keywords: Vec<String>,
    /// Related entities
    pub related_entities: Vec<String>,
    /// Verification details
    pub verification: VerificationResult,
}

/// Result of verification at position 6
#[derive(Debug, Clone)]
pub struct VerificationResult {
    /// Whether the fact passed verification
    pub passed: bool,
    /// Geometric consistency score
    pub geometric_score: f32,
    /// Contradiction score (lower is better)
    pub contradiction_score: f32,
    /// Confirmation score (higher is better)
    pub confirmation_score: f32,
    /// Reason for pass/fail
    pub reason: String,
}

// =============================================================================
// POSITION 3: KEYWORD EXTRACTION HEADER
// =============================================================================

/// Sacred Attention Header at Position 3: Keyword Extraction
pub struct KeywordExtractionHeader {
    /// Embedding dimension
    embed_dim: usize,
    /// World encoder for embeddings
    encoder: WorldEncoder,
    /// Stopwords to filter
    stopwords: std::collections::HashSet<String>,
    /// Entity type patterns
    entity_patterns: HashMap<String, EntityType>,
}

impl KeywordExtractionHeader {
    pub fn new(embed_dim: usize) -> Self {
        let stopwords: std::collections::HashSet<String> = [
            "the", "a", "an", "is", "are", "was", "were", "be", "been", "being",
            "have", "has", "had", "do", "does", "did", "will", "would", "could",
            "should", "may", "might", "must", "shall", "can", "need", "to", "of",
            "in", "for", "on", "with", "at", "by", "from", "as", "into", "through",
            "and", "but", "or", "if", "because", "while", "this", "that", "these",
            "those", "it", "its", "i", "you", "he", "she", "we", "they", "what",
            "which", "who", "whom", "how", "when", "where", "why", "there", "here",
        ].iter().map(|s| s.to_string()).collect();

        Self {
            embed_dim,
            encoder: WorldEncoder::new(embed_dim),
            stopwords,
            entity_patterns: HashMap::new(),
        }
    }

    /// Process text through Position 3 (Keyword Extraction)
    pub fn process(&self, text: &str) -> Position3Output {
        let mut entities = Vec::new();
        let mut keywords = Vec::new();
        let mut relations = Vec::new();

        // Split into sentences
        for sentence in text.split(|c| c == '.' || c == '!' || c == '?') {
            let sentence = sentence.trim();
            if sentence.len() < 5 {
                continue;
            }

            // Extract entities from sentence
            let sentence_entities = self.extract_entities(sentence);
            entities.extend(sentence_entities.clone());

            // Extract keywords
            let sentence_keywords = self.extract_keywords(sentence);
            keywords.extend(sentence_keywords);

            // Extract relations between entities
            if sentence_entities.len() >= 2 {
                let sentence_relations = self.extract_relations(sentence, &sentence_entities);
                relations.extend(sentence_relations);
            }
        }

        // Deduplicate keywords
        keywords.sort();
        keywords.dedup();

        Position3Output {
            entities,
            keywords,
            relations,
            raw_text: text.to_string(),
        }
    }

    /// Extract entities from a sentence
    fn extract_entities(&self, sentence: &str) -> Vec<ExtractedEntity> {
        let mut entities = Vec::new();
        let words: Vec<&str> = sentence.split_whitespace().collect();

        for (i, word) in words.iter().enumerate() {
            let clean = word.trim_matches(|c: char| !c.is_alphanumeric());
            if clean.len() < 2 || self.stopwords.contains(&clean.to_lowercase()) {
                continue;
            }

            // Check if this looks like an entity
            let entity_type = EntityType::infer(clean, sentence);
            
            // Higher confidence for capitalized words (likely proper nouns)
            let confidence = if clean.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                0.8
            } else if entity_type != EntityType::Unknown {
                0.6
            } else {
                0.4
            };

            entities.push(ExtractedEntity {
                text: clean.to_string(),
                entity_type,
                position: i,
                confidence,
                keywords: vec![clean.to_lowercase()],
            });
        }

        entities
    }

    /// Extract keywords from a sentence
    fn extract_keywords(&self, sentence: &str) -> Vec<String> {
        sentence.to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() > 2 && !self.stopwords.contains(*w))
            .map(|s| s.to_string())
            .collect()
    }

    /// Extract relations between entities
    fn extract_relations(&self, sentence: &str, entities: &[ExtractedEntity]) -> Vec<ExtractedRelation> {
        let mut relations = Vec::new();
        let sentence_lower = sentence.to_lowercase();

        // Relation patterns to look for
        let relation_patterns = [
            (" is ", "is_a"),
            (" are ", "is_a"),
            (" has ", "has"),
            (" have ", "has"),
            (" in ", "located_in"),
            (" at ", "located_at"),
            (" on ", "located_on"),
            (" to ", "goes_to"),
            (" from ", "comes_from"),
            (" with ", "associated_with"),
            (" for ", "used_for"),
            (" can ", "capable_of"),
            (" will ", "will_do"),
        ];

        for (pattern, rel_type) in relation_patterns {
            if let Some(idx) = sentence_lower.find(pattern) {
                // Find entities before and after the pattern
                let before_entities: Vec<_> = entities.iter()
                    .filter(|e| e.position < idx / 5) // Approximate word position
                    .collect();
                let after_entities: Vec<_> = entities.iter()
                    .filter(|e| e.position > idx / 5)
                    .collect();

                if let (Some(source), Some(target)) = (before_entities.last(), after_entities.first()) {
                    relations.push(ExtractedRelation {
                        source: source.text.clone(),
                        relation_type: rel_type.to_string(),
                        target: target.text.clone(),
                        confidence: (source.confidence + target.confidence) / 2.0,
                        verified: false,
                    });
                }
            }
        }

        relations
    }
}

/// Output from Position 3 processing
#[derive(Debug, Clone)]
pub struct Position3Output {
    /// Extracted entities
    pub entities: Vec<ExtractedEntity>,
    /// Extracted keywords
    pub keywords: Vec<String>,
    /// Extracted relations
    pub relations: Vec<ExtractedRelation>,
    /// Original text
    pub raw_text: String,
}

// =============================================================================
// POSITION 6: RELATION VERIFICATION HEADER
// =============================================================================

/// Sacred Attention Header at Position 6: Relation Verification
pub struct RelationVerificationHeader {
    /// Embedding dimension
    embed_dim: usize,
    /// World encoder for embeddings
    encoder: WorldEncoder,
    /// Relation predictor for geometric verification
    predictor: GeometricRelationPredictor,
    /// Known facts for contradiction checking
    known_facts: HashMap<String, Vec<(String, String)>>, // subject -> [(attribute, value)]
}

impl RelationVerificationHeader {
    pub fn new(embed_dim: usize) -> Self {
        Self {
            embed_dim,
            encoder: WorldEncoder::new(embed_dim),
            predictor: GeometricRelationPredictor::new(embed_dim),
            known_facts: HashMap::new(),
        }
    }

    /// Process Position 3 output through Position 6 (Relation Verification)
    pub fn process(&self, input: &Position3Output, world_model: &GeometricWorldModel) -> Position6Output {
        let mut verified_facts = Vec::new();
        let mut rejected_facts = Vec::new();

        // Verify each relation
        for relation in &input.relations {
            let verification = self.verify_relation(relation, world_model);
            
            if verification.passed {
                verified_facts.push(VerifiedFact {
                    subject: relation.source.clone(),
                    attribute: relation.relation_type.clone(),
                    value: relation.target.clone(),
                    confidence: relation.confidence * verification.geometric_score,
                    source: "web".to_string(),
                    keywords: input.keywords.clone(),
                    related_entities: input.entities.iter()
                        .map(|e| e.text.clone())
                        .collect(),
                    verification: verification.clone(),
                });
            } else {
                rejected_facts.push((relation.clone(), verification));
            }
        }

        // Also extract facts from entities with properties
        for entity in &input.entities {
            if entity.entity_type == EntityType::Place {
                // Places are often locations for other things
                verified_facts.push(VerifiedFact {
                    subject: entity.text.clone(),
                    attribute: "is_a".to_string(),
                    value: "place".to_string(),
                    confidence: entity.confidence,
                    source: "inference".to_string(),
                    keywords: entity.keywords.clone(),
                    related_entities: vec![],
                    verification: VerificationResult {
                        passed: true,
                        geometric_score: 0.8,
                        contradiction_score: 0.0,
                        confirmation_score: 0.5,
                        reason: "Entity type inference".to_string(),
                    },
                });
            }
        }

        Position6Output {
            verified_facts,
            rejected_facts,
            entities: input.entities.clone(),
            keywords: input.keywords.clone(),
        }
    }

    /// Verify a single relation
    fn verify_relation(&self, relation: &ExtractedRelation, world_model: &GeometricWorldModel) -> VerificationResult {
        // 1. Geometric consistency check
        let source_embed = self.encoder.encode_entity(&relation.source);
        let target_embed = self.encoder.encode_entity(&relation.target);
        
        let (geo_holds, geo_conf) = self.predictor.check_relation(
            &source_embed,
            &relation.relation_type,
            &target_embed,
        );

        let geometric_score = if geo_holds { geo_conf } else { geo_conf * 0.5 };

        // 2. Contradiction check
        let contradiction_score = self.check_contradiction(relation);

        // 3. Confirmation check (does this align with known patterns?)
        let confirmation_score = self.check_confirmation(relation);

        // Determine if passed
        let passed = geometric_score > 0.2 && contradiction_score < 0.5;

        let reason = if !passed {
            if contradiction_score >= 0.5 {
                "Contradicts existing knowledge".to_string()
            } else {
                "Low geometric consistency".to_string()
            }
        } else {
            "Verified".to_string()
        };

        VerificationResult {
            passed,
            geometric_score,
            contradiction_score,
            confirmation_score,
            reason,
        }
    }

    /// Check if relation contradicts known facts
    fn check_contradiction(&self, relation: &ExtractedRelation) -> f32 {
        if let Some(facts) = self.known_facts.get(&relation.source.to_lowercase()) {
            for (attr, value) in facts {
                // Check for direct contradiction
                if attr == &relation.relation_type && value != &relation.target {
                    // Same attribute, different value - potential contradiction
                    // But some attributes can have multiple values
                    if attr == "is_a" || attr == "has" || attr == "located_in" {
                        return 0.3; // Mild contradiction (could be multiple)
                    } else {
                        return 0.8; // Strong contradiction
                    }
                }
            }
        }
        0.0 // No contradiction
    }

    /// Check if relation is confirmed by patterns
    fn check_confirmation(&self, relation: &ExtractedRelation) -> f32 {
        // Common sense patterns that boost confidence
        let confirmed_patterns: Vec<(&str, &str, &str)> = vec![
            ("hamburger", "located_in", "restaurant"),
            ("book", "located_in", "library"),
            ("car", "located_in", "garage"),
            ("food", "located_in", "kitchen"),
            ("water", "is_a", "liquid"),
            ("ice", "is_a", "solid"),
        ];

        for (subj, rel, targ) in confirmed_patterns {
            if relation.source.to_lowercase().contains(subj)
                && relation.relation_type == rel
                && relation.target.to_lowercase().contains(targ)
            {
                return 0.9; // High confirmation
            }
        }

        0.5 // Neutral
    }

    /// Add known fact for future contradiction checking
    pub fn add_known_fact(&mut self, subject: &str, attribute: &str, value: &str) {
        self.known_facts
            .entry(subject.to_lowercase())
            .or_default()
            .push((attribute.to_string(), value.to_string()));
    }
}

/// Output from Position 6 processing
#[derive(Debug, Clone)]
pub struct Position6Output {
    /// Facts that passed verification
    pub verified_facts: Vec<VerifiedFact>,
    /// Facts that failed verification with reasons
    pub rejected_facts: Vec<(ExtractedRelation, VerificationResult)>,
    /// Entities passed through
    pub entities: Vec<ExtractedEntity>,
    /// Keywords passed through
    pub keywords: Vec<String>,
}

// =============================================================================
// POSITION 9: KNOWLEDGE INTEGRATION HEADER
// =============================================================================

/// Sacred Attention Header at Position 9: Knowledge Integration
pub struct KnowledgeIntegrationHeader {
    /// Embedding dimension
    embed_dim: usize,
    /// World encoder for embeddings
    encoder: WorldEncoder,
    /// Minimum confidence to integrate
    min_confidence: f32,
    /// Integration statistics
    pub stats: IntegrationStats,
}

#[derive(Debug, Clone, Default)]
pub struct IntegrationStats {
    pub facts_received: usize,
    pub facts_integrated: usize,
    pub facts_rejected: usize,
    pub embeddings_created: usize,
}

impl KnowledgeIntegrationHeader {
    pub fn new(embed_dim: usize, min_confidence: f32) -> Self {
        Self {
            embed_dim,
            encoder: WorldEncoder::new(embed_dim),
            min_confidence,
            stats: IntegrationStats::default(),
        }
    }

    /// Process Position 6 output through Position 9 (Knowledge Integration)
    pub fn process(&mut self, input: &Position6Output) -> Position9Output {
        let mut integrated_facts = Vec::new();
        let mut embeddings = Vec::new();

        self.stats.facts_received += input.verified_facts.len();

        for fact in &input.verified_facts {
            if fact.confidence >= self.min_confidence {
                // Create embedding for the fact
                let fact_text = format!("{} {} {}", fact.subject, fact.attribute, fact.value);
                let embedding = self.encoder.encode_entity(&fact_text);
                
                embeddings.push(IntegratedEmbedding {
                    text: fact_text,
                    embedding,
                    subject: fact.subject.clone(),
                    confidence: fact.confidence,
                });

                integrated_facts.push(fact.clone());
                self.stats.facts_integrated += 1;
                self.stats.embeddings_created += 1;
            } else {
                self.stats.facts_rejected += 1;
            }
        }

        Position9Output {
            integrated_facts,
            embeddings,
            keywords: input.keywords.clone(),
            stats: self.stats.clone(),
        }
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = IntegrationStats::default();
    }
}

/// An embedding ready for storage
#[derive(Debug, Clone)]
pub struct IntegratedEmbedding {
    /// Text that was embedded
    pub text: String,
    /// Embedding vector
    pub embedding: Vec<f32>,
    /// Subject this relates to
    pub subject: String,
    /// Confidence score
    pub confidence: f32,
}

/// Output from Position 9 processing
#[derive(Debug, Clone)]
pub struct Position9Output {
    /// Facts that were integrated
    pub integrated_facts: Vec<VerifiedFact>,
    /// Embeddings created
    pub embeddings: Vec<IntegratedEmbedding>,
    /// Keywords for indexing
    pub keywords: Vec<String>,
    /// Integration statistics
    pub stats: IntegrationStats,
}

// =============================================================================
// SACRED ATTENTION PIPELINE - Complete 3-6-9 Flow
// =============================================================================

/// Complete Sacred Attention Pipeline (3-6-9)
pub struct SacredAttentionPipeline {
    /// Position 3: Keyword Extraction
    pub position_3: KeywordExtractionHeader,
    /// Position 6: Relation Verification
    pub position_6: RelationVerificationHeader,
    /// Position 9: Knowledge Integration
    pub position_9: KnowledgeIntegrationHeader,
    /// Pipeline statistics
    pub stats: PipelineStats,
}

#[derive(Debug, Clone, Default)]
pub struct PipelineStats {
    pub texts_processed: usize,
    pub entities_extracted: usize,
    pub relations_extracted: usize,
    pub facts_verified: usize,
    pub facts_integrated: usize,
}

impl SacredAttentionPipeline {
    pub fn new(embed_dim: usize) -> Self {
        Self {
            position_3: KeywordExtractionHeader::new(embed_dim),
            position_6: RelationVerificationHeader::new(embed_dim),
            position_9: KnowledgeIntegrationHeader::new(embed_dim, 0.3),
            stats: PipelineStats::default(),
        }
    }

    /// Process text through the complete 3-6-9 pipeline
    pub fn process(&mut self, text: &str, world_model: &GeometricWorldModel) -> Position9Output {
        self.stats.texts_processed += 1;

        // Position 3: Keyword Extraction
        let pos3_output = self.position_3.process(text);
        self.stats.entities_extracted += pos3_output.entities.len();
        self.stats.relations_extracted += pos3_output.relations.len();

        // Position 6: Relation Verification
        let pos6_output = self.position_6.process(&pos3_output, world_model);
        self.stats.facts_verified += pos6_output.verified_facts.len();

        // Position 9: Knowledge Integration
        let pos9_output = self.position_9.process(&pos6_output);
        self.stats.facts_integrated += pos9_output.integrated_facts.len();

        pos9_output
    }

    /// Process multiple texts in batch
    pub fn process_batch(&mut self, texts: &[&str], world_model: &GeometricWorldModel) -> Vec<Position9Output> {
        texts.iter()
            .map(|text| self.process(text, world_model))
            .collect()
    }

    /// Get pipeline statistics
    pub fn get_stats(&self) -> PipelineStats {
        self.stats.clone()
    }

    /// Reset all statistics
    pub fn reset_stats(&mut self) {
        self.stats = PipelineStats::default();
        self.position_9.reset_stats();
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_type_inference() {
        assert_eq!(EntityType::infer("restaurant", ""), EntityType::Place);
        assert_eq!(EntityType::infer("doctor", ""), EntityType::Person);
        assert_eq!(EntityType::infer("running", ""), EntityType::Action);
        assert_eq!(EntityType::infer("big", ""), EntityType::Property);
    }

    #[test]
    fn test_keyword_extraction_header() {
        let header = KeywordExtractionHeader::new(64);
        
        let output = header.process("The hamburger is found in a restaurant. Water is a liquid.");
        
        assert!(!output.entities.is_empty());
        assert!(!output.keywords.is_empty());
        println!("Entities: {:?}", output.entities.iter().map(|e| &e.text).collect::<Vec<_>>());
        println!("Keywords: {:?}", output.keywords);
    }

    #[test]
    fn test_relation_verification_header() {
        let header = RelationVerificationHeader::new(64);
        let world_model = GeometricWorldModel::new(64);

        let relation = ExtractedRelation {
            source: "hamburger".to_string(),
            relation_type: "located_in".to_string(),
            target: "restaurant".to_string(),
            confidence: 0.8,
            verified: false,
        };

        let result = header.verify_relation(&relation, &world_model);
        println!("Verification: {:?}", result);
        assert!(result.confirmation_score > 0.5); // Should be confirmed by pattern
    }

    #[test]
    fn test_complete_pipeline() {
        let mut pipeline = SacredAttentionPipeline::new(64);
        let world_model = GeometricWorldModel::new(64);

        let text = "A hamburger is a popular food. Hamburgers are found in restaurants. Water is a liquid that we drink.";
        
        let output = pipeline.process(text, &world_model);
        
        println!("Integrated facts: {}", output.integrated_facts.len());
        for fact in &output.integrated_facts {
            println!("  {} {} {} (conf: {:.2})", fact.subject, fact.attribute, fact.value, fact.confidence);
        }
        
        println!("Pipeline stats: {:?}", pipeline.get_stats());
    }

    #[test]
    fn test_batch_processing() {
        let mut pipeline = SacredAttentionPipeline::new(64);
        let world_model = GeometricWorldModel::new(64);

        let texts = vec![
            "Hamburgers are found in restaurants.",
            "Books are found in libraries.",
            "Cars are parked in garages.",
        ];

        let outputs = pipeline.process_batch(&texts, &world_model);
        
        assert_eq!(outputs.len(), 3);
        println!("Total facts integrated: {}", pipeline.stats.facts_integrated);
    }
}
