//! Consciousness-Driven Learning System
//!
//! Implements self-improving AI through autoregressive self-prompting,
//! web knowledge acquisition, and sacred attention verification.
//!
//! ## Architecture
//! ```text
//! Autoregressive Self-Prompting (model generates its own queries)
//!              ↓
//! Web Knowledge Acquisition (DuckDuckGo HTML scraping)
//!              ↓
//! Sacred Attention Pipeline (3-6-9 verification)
//!   Position 3: Keyword Extraction
//!   Position 6: Relation Verification
//!   Position 9: Knowledge Integration
//!              ↓
//! Dynamic Vortex (subject-indexed, sequential ordering)
//!              ↓
//! GeometricWorldModel (foundation for reasoning)
//! ```
//!
//! ## Key Innovations
//! - **Autoregressive Self-Improvement**: Model generates learning queries
//! - **Sacred Verification Pipeline**: 3-stage fact verification
//! - **Dynamic Vortex**: Subject-indexed with temporal ordering
//! - **Geometric Consistency**: Facts must be geometrically coherent

use std::collections::HashMap;
use std::time::{Duration, Instant};
use crate::ml::geometric_world_model::{GeometricWorldModel, WorldEncoder, CoordinateSpace};
use crate::ml::world_knowledge::WorldKnowledgeGraph;

// =============================================================================
// SUBJECT NODE - Dynamic Vortex Entry
// =============================================================================

/// A node in the dynamic vortex representing a subject
#[derive(Debug, Clone)]
pub struct SubjectNode {
    /// Subject identifier (e.g., "hamburger", "restaurant")
    pub subject: String,
    /// Keywords associated with this subject for fast lookup
    pub keywords: Vec<String>,
    /// Attributes: property → value with confidence
    pub attributes: HashMap<String, AttributeValue>,
    /// Relations to other subjects: (relation_type, target_subject, confidence)
    pub relations: Vec<(String, String, f32)>,
    /// Sequential order of operations (temporal log)
    pub sequence_order: Vec<OperationLog>,
    /// Current flux position in vortex cycle (1-9)
    pub flux_position: u8,
    /// Confidence score for this subject (0.0-1.0)
    pub confidence: f32,
    /// Last verification timestamp
    pub last_verified: Instant,
    /// Number of times this subject has been verified
    pub verification_count: u32,
    /// Embedding vector for similarity search
    pub embedding: Vec<f32>,
}

/// Attribute value with confidence and source tracking
#[derive(Debug, Clone)]
pub struct AttributeValue {
    pub value: String,
    pub confidence: f32,
    pub sources: Vec<String>,
    pub verified: bool,
}

/// Operation log entry for temporal ordering
#[derive(Debug, Clone)]
pub struct OperationLog {
    pub operation: OperationType,
    pub timestamp: Instant,
    pub details: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OperationType {
    Created,
    AttributeAdded,
    RelationAdded,
    Verified,
    Updated,
    Merged,
}

impl SubjectNode {
    pub fn new(subject: &str, embedding: Vec<f32>) -> Self {
        Self {
            subject: subject.to_string(),
            keywords: vec![subject.to_lowercase()],
            attributes: HashMap::new(),
            relations: Vec::new(),
            sequence_order: vec![OperationLog {
                operation: OperationType::Created,
                timestamp: Instant::now(),
                details: format!("Created subject: {}", subject),
            }],
            flux_position: 1,
            confidence: 0.5,
            last_verified: Instant::now(),
            verification_count: 0,
            embedding,
        }
    }

    /// Add a keyword for this subject
    pub fn add_keyword(&mut self, keyword: &str) {
        let kw = keyword.to_lowercase();
        if !self.keywords.contains(&kw) {
            self.keywords.push(kw);
        }
    }

    /// Add an attribute with confidence
    pub fn add_attribute(&mut self, key: &str, value: &str, confidence: f32, source: &str) {
        let entry = self.attributes.entry(key.to_string()).or_insert_with(|| AttributeValue {
            value: value.to_string(),
            confidence: 0.0,
            sources: Vec::new(),
            verified: false,
        });
        
        // Update if higher confidence or same value (reinforcement)
        if confidence > entry.confidence || entry.value == value {
            entry.value = value.to_string();
            entry.confidence = (entry.confidence + confidence) / 2.0; // Average
            if !entry.sources.contains(&source.to_string()) {
                entry.sources.push(source.to_string());
            }
        }
        
        self.sequence_order.push(OperationLog {
            operation: OperationType::AttributeAdded,
            timestamp: Instant::now(),
            details: format!("{}={} (conf: {:.2})", key, value, confidence),
        });
    }

    /// Add a relation to another subject
    pub fn add_relation(&mut self, relation_type: &str, target: &str, confidence: f32) {
        // Check if relation already exists
        let existing = self.relations.iter_mut()
            .find(|(r, t, _)| r == relation_type && t == target);
        
        if let Some((_, _, conf)) = existing {
            // Reinforce existing relation
            *conf = (*conf + confidence) / 2.0;
        } else {
            self.relations.push((relation_type.to_string(), target.to_string(), confidence));
        }
        
        self.sequence_order.push(OperationLog {
            operation: OperationType::RelationAdded,
            timestamp: Instant::now(),
            details: format!("{} -> {} ({})", relation_type, target, confidence),
        });
    }

    /// Advance flux position through vortex cycle
    pub fn advance_flux(&mut self) {
        const VORTEX_CYCLE: [u8; 6] = [1, 2, 4, 8, 7, 5];
        let idx = VORTEX_CYCLE.iter().position(|&x| x == self.flux_position).unwrap_or(0);
        self.flux_position = VORTEX_CYCLE[(idx + 1) % 6];
    }

    /// Check if at sacred position (3, 6, or 9)
    pub fn is_at_sacred_position(&self) -> bool {
        matches!(self.flux_position, 3 | 6 | 9)
    }

    /// Mark as verified
    pub fn mark_verified(&mut self) {
        self.last_verified = Instant::now();
        self.verification_count += 1;
        self.confidence = (self.confidence + 0.1).min(1.0);
        
        self.sequence_order.push(OperationLog {
            operation: OperationType::Verified,
            timestamp: Instant::now(),
            details: format!("Verified (count: {})", self.verification_count),
        });
    }
}

// =============================================================================
// DYNAMIC VORTEX - Subject-Indexed Knowledge Store
// =============================================================================

/// Dynamic vortex for subject-indexed knowledge storage
#[derive(Debug)]
pub struct DynamicVortex {
    /// Subject index: subject_name → SubjectNode
    pub subjects: HashMap<String, SubjectNode>,
    /// Keyword index: keyword → list of subjects
    pub keyword_index: HashMap<String, Vec<String>>,
    /// World encoder for embeddings
    encoder: WorldEncoder,
    /// Embedding dimension
    embed_dim: usize,
    /// Global vortex position
    pub global_position: u8,
    /// Total knowledge items
    pub knowledge_count: usize,
}

impl DynamicVortex {
    pub fn new(embed_dim: usize) -> Self {
        Self {
            subjects: HashMap::new(),
            keyword_index: HashMap::new(),
            encoder: WorldEncoder::new(embed_dim),
            embed_dim,
            global_position: 1,
            knowledge_count: 0,
        }
    }

    /// Get or create a subject node
    pub fn get_or_create_subject(&mut self, subject: &str) -> &mut SubjectNode {
        let subject_lower = subject.to_lowercase();
        
        if !self.subjects.contains_key(&subject_lower) {
            let embedding = self.encoder.encode_entity(subject);
            let node = SubjectNode::new(subject, embedding);
            self.subjects.insert(subject_lower.clone(), node);
            
            // Index the subject keyword
            self.keyword_index
                .entry(subject_lower.clone())
                .or_default()
                .push(subject_lower.clone());
            
            self.knowledge_count += 1;
        }
        
        self.subjects.get_mut(&subject_lower).unwrap()
    }

    /// Add knowledge to a subject
    pub fn add_knowledge(
        &mut self,
        subject: &str,
        attribute: &str,
        value: &str,
        confidence: f32,
        source: &str,
    ) {
        // First, ensure subject exists and add attribute
        let subject_lower = subject.to_lowercase();
        {
            let node = self.get_or_create_subject(subject);
            node.add_attribute(attribute, value, confidence, source);
        }
        
        // Extract keywords from value
        let keywords_to_add: Vec<String> = value.split_whitespace()
            .map(|word| word.to_lowercase()
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_string())
            .filter(|w| w.len() > 2)
            .collect();
        
        // Now add keywords separately to avoid borrow issues
        for word_lower in keywords_to_add {
            // Add to subject node
            if let Some(node) = self.subjects.get_mut(&subject_lower) {
                node.add_keyword(&word_lower);
            }
            // Add to keyword index
            self.keyword_index
                .entry(word_lower)
                .or_default()
                .push(subject_lower.clone());
        }
        
        self.knowledge_count += 1;
    }

    /// Add a relation between subjects
    pub fn add_relation(
        &mut self,
        source_subject: &str,
        relation_type: &str,
        target_subject: &str,
        confidence: f32,
    ) {
        // Ensure both subjects exist
        self.get_or_create_subject(target_subject);
        let source_node = self.get_or_create_subject(source_subject);
        source_node.add_relation(relation_type, target_subject, confidence);
        
        self.knowledge_count += 1;
    }

    /// Search subjects by keyword
    pub fn search_by_keyword(&self, keyword: &str) -> Vec<&SubjectNode> {
        let keyword_lower = keyword.to_lowercase();
        
        if let Some(subjects) = self.keyword_index.get(&keyword_lower) {
            subjects.iter()
                .filter_map(|s| self.subjects.get(s))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Search subjects by embedding similarity
    pub fn search_by_similarity(&self, query: &str, top_k: usize) -> Vec<(&SubjectNode, f32)> {
        let query_embedding = self.encoder.encode_entity(query);
        
        let mut scores: Vec<_> = self.subjects.values()
            .map(|node| {
                let sim = cosine_similarity(&query_embedding, &node.embedding);
                (node, sim)
            })
            .collect();
        
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores.truncate(top_k);
        scores
    }

    /// Get all subjects related to a query
    pub fn query_knowledge(&self, query: &str) -> Vec<(&SubjectNode, f32)> {
        let mut results = Vec::new();
        
        // 1. Direct keyword match
        for word in query.split_whitespace() {
            for node in self.search_by_keyword(word) {
                if !results.iter().any(|(n, _): &(&SubjectNode, f32)| n.subject == node.subject) {
                    results.push((node, 0.8)); // High confidence for keyword match
                }
            }
        }
        
        // 2. Similarity search
        for (node, sim) in self.search_by_similarity(query, 5) {
            if !results.iter().any(|(n, _): &(&SubjectNode, f32)| n.subject == node.subject) {
                results.push((node, sim));
            }
        }
        
        // Sort by relevance
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    /// Advance global vortex position
    pub fn advance_global_position(&mut self) {
        const VORTEX_CYCLE: [u8; 6] = [1, 2, 4, 8, 7, 5];
        let idx = VORTEX_CYCLE.iter().position(|&x| x == self.global_position).unwrap_or(0);
        self.global_position = VORTEX_CYCLE[(idx + 1) % 6];
    }

    /// Get statistics
    pub fn stats(&self) -> VortexStats {
        VortexStats {
            subject_count: self.subjects.len(),
            keyword_count: self.keyword_index.len(),
            knowledge_count: self.knowledge_count,
            global_position: self.global_position,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VortexStats {
    pub subject_count: usize,
    pub keyword_count: usize,
    pub knowledge_count: usize,
    pub global_position: u8,
}

// =============================================================================
// CONSCIOUSNESS LEARNER - Main Learning Engine
// =============================================================================

/// Configuration for consciousness learning
#[derive(Debug, Clone)]
pub struct ConsciousnessConfig {
    /// Embedding dimension
    pub embed_dim: usize,
    /// Maximum concurrent web requests
    pub max_concurrent_requests: usize,
    /// Timeout for web requests (seconds)
    pub request_timeout_secs: u64,
    /// Minimum confidence to store knowledge
    pub min_confidence: f32,
    /// Enable web search
    pub enable_web_search: bool,
    /// Maximum learning time before benchmark (seconds)
    pub max_learning_time_secs: u64,
    /// Maximum queries per learning session
    pub max_queries_per_session: usize,
    /// Request timeout in seconds
    pub request_timeout_secs_web: usize,
    /// Number of parallel web requests
    pub parallel_requests: usize,
    /// Batch size for query processing
    pub query_batch_size: usize,
}

impl Default for ConsciousnessConfig {
    fn default() -> Self {
        Self {
            embed_dim: 256,
            max_concurrent_requests: 200, // Extreme parallelism
            request_timeout_secs: 30,
            min_confidence: 0.3,
            enable_web_search: true,
            max_learning_time_secs: 180,
            max_queries_per_session: 10000, // Learn entire documentation sets
            request_timeout_secs_web: 10, // Fast timeout
            parallel_requests: 100, // 100 concurrent workers
            query_batch_size: 200, // Massive batches
        }
    }
}
/// Main consciousness learning engine
pub struct ConsciousnessLearner {
    /// Configuration
    pub config: ConsciousnessConfig,
    /// Dynamic vortex for knowledge storage
    pub vortex: DynamicVortex,
    /// Geometric world model (foundation)
    pub world_model: GeometricWorldModel,
    /// World knowledge graph (commonsense)
    pub knowledge_graph: WorldKnowledgeGraph,
    /// Failed questions for analysis
    pub failure_log: Vec<FailedQuestion>,
    /// Learning statistics
    pub stats: LearningStats,
    /// Sacred attention processors (positions 3, 6, 9)
    sacred_processors: SacredAttentionPipeline,
    /// Web learner for DuckDuckGo scraping
    web_learner: crate::ml::web_knowledge::BatchWebLearner,
    /// Unique domains seen during learning
    seen_domains: std::collections::HashSet<String>,
}

/// A failed question for analysis
#[derive(Debug, Clone)]
pub struct FailedQuestion {
    pub question: String,
    pub expected: String,
    pub predicted: String,
    pub category: String,
    pub timestamp: Instant,
}

/// Learning statistics
#[derive(Debug, Clone, Default)]
pub struct LearningStats {
    pub queries_generated: usize,
    pub web_searches: usize,
    pub websites_referenced: usize,
    pub unique_domains: usize,
    pub facts_extracted: usize,
    pub facts_verified: usize,
    pub facts_integrated: usize,
    pub subjects_created: usize,
    pub learning_time_ms: u64,
    pub search_errors: usize,
}

/// Knowledge gap analysis report
#[derive(Debug, Clone)]
pub struct KnowledgeGapAnalysis {
    /// Total subjects in the vortex
    pub total_subjects: usize,
    /// Subjects with confidence below threshold
    pub low_confidence_count: usize,
    /// Subjects with few relations (isolated)
    pub isolated_count: usize,
    /// Subjects missing common attributes
    pub incomplete_count: usize,
    /// Average confidence across all subjects
    pub avg_confidence: f32,
    /// Average relations per subject
    pub avg_relations: f32,
    /// Knowledge density (attrs + relations per subject)
    pub knowledge_density: f32,
    /// Number of recommended queries to fill gaps
    pub recommended_queries: usize,
}

impl KnowledgeGapAnalysis {
    /// Get a health score (0.0-1.0) for the knowledge base
    pub fn health_score(&self) -> f32 {
        if self.total_subjects == 0 {
            return 0.0;
        }
        
        let confidence_score = self.avg_confidence;
        let connectivity_score = (self.avg_relations / 3.0).min(1.0); // 3+ relations is good
        let completeness_score = 1.0 - (self.incomplete_count as f32 / self.total_subjects as f32);
        let isolation_score = 1.0 - (self.isolated_count as f32 / self.total_subjects as f32);
        
        // Weighted average
        (confidence_score * 0.3 + connectivity_score * 0.25 + 
         completeness_score * 0.25 + isolation_score * 0.2).min(1.0)
    }
    
    /// Check if the knowledge base needs learning
    pub fn needs_learning(&self) -> bool {
        self.health_score() < 0.6 || self.recommended_queries > 10
    }
    
    /// Get priority areas for learning
    pub fn priority_areas(&self) -> Vec<String> {
        let mut areas = Vec::new();
        
        if self.low_confidence_count > self.total_subjects / 4 {
            areas.push("verification".to_string());
        }
        if self.isolated_count > self.total_subjects / 3 {
            areas.push("relations".to_string());
        }
        if self.incomplete_count > self.total_subjects / 3 {
            areas.push("attributes".to_string());
        }
        if self.total_subjects < 50 {
            areas.push("expansion".to_string());
        }
        
        areas
    }
}

impl ConsciousnessLearner {
    pub fn new(config: ConsciousnessConfig) -> Self {
        let embed_dim = config.embed_dim;
        let web_config = crate::ml::web_knowledge::WebScraperConfig {
            timeout_secs: config.request_timeout_secs,
            max_results: 30, // Maximum results per query
            request_delay_ms: 0, // ZERO delay - maximum throughput
        };
        Self {
            config,
            vortex: DynamicVortex::new(embed_dim),
            world_model: GeometricWorldModel::new(embed_dim),
            knowledge_graph: WorldKnowledgeGraph::new(embed_dim),
            failure_log: Vec::new(),
            stats: LearningStats::default(),
            sacred_processors: SacredAttentionPipeline::new(embed_dim),
            web_learner: crate::ml::web_knowledge::BatchWebLearner::new(web_config),
            seen_domains: std::collections::HashSet::new(),
        }
    }

    /// Log a failed question for later analysis
    pub fn log_failure(&mut self, question: &str, expected: &str, predicted: &str, category: &str) {
        self.failure_log.push(FailedQuestion {
            question: question.to_string(),
            expected: expected.to_string(),
            predicted: predicted.to_string(),
            category: category.to_string(),
            timestamp: Instant::now(),
        });
    }

    /// Generate improvement queries based on knowledge gap analysis
    /// DEPRECATED: No longer used - we rely on dynamic test-time learning only
    fn generate_improvement_queries(&self) -> Vec<String> {
        // Return empty - all learning happens dynamically at test time
        Vec::new()
    }
    
    /// Find subjects with confidence below threshold
    fn find_low_confidence_subjects(&self, threshold: f32) -> Vec<String> {
        self.vortex.subjects.iter()
            .filter(|(_, node)| node.confidence < threshold)
            .map(|(subject, _)| subject.clone())
            .take(20)
            .collect()
    }
    
    /// Find subjects with fewer than min_relations connections
    fn find_isolated_subjects(&self, min_relations: usize) -> Vec<String> {
        self.vortex.subjects.iter()
            .filter(|(_, node)| node.relations.len() < min_relations)
            .map(|(subject, _)| subject.clone())
            .take(20)
            .collect()
    }
    
    /// Find subjects missing common attributes (location, function, properties)
    fn find_incomplete_subjects(&self) -> Vec<(String, Vec<String>)> {
        let common_attrs = ["location", "function", "is", "has", "typical_location"];
        let mut incomplete = Vec::new();
        
        for (subject, node) in &self.vortex.subjects {
            let missing: Vec<String> = common_attrs.iter()
                .filter(|attr| !node.attributes.contains_key(&attr.to_string()))
                .map(|s| s.to_string())
                .collect();
            
            if !missing.is_empty() && missing.len() < common_attrs.len() {
                // Has some attrs but not all - worth completing
                incomplete.push((subject.clone(), missing));
            }
        }
        
        incomplete.into_iter().take(15).collect()
    }
    
    /// Find sparse regions in embedding space by analyzing embedding distribution
    fn find_sparse_embedding_regions(&self) -> Vec<String> {
        let mut queries = Vec::new();
        
        if self.vortex.subjects.is_empty() {
            // No knowledge yet - generate foundational queries
            return vec![
                "common household objects and their locations".to_string(),
                "physical properties of everyday materials".to_string(),
                "typical activities and where they happen".to_string(),
                "tools and their functions".to_string(),
                "food items and where to find them".to_string(),
            ];
        }
        
        // Compute centroid of all embeddings
        let embed_dim = self.config.embed_dim;
        let mut centroid = vec![0.0f32; embed_dim];
        let mut count = 0;
        
        for node in self.vortex.subjects.values() {
            if node.embedding.len() == embed_dim {
                for (i, &v) in node.embedding.iter().enumerate() {
                    centroid[i] += v;
                }
                count += 1;
            }
        }
        
        if count > 0 {
            for v in &mut centroid {
                *v /= count as f32;
            }
        }
        
        // Find subjects far from centroid (outliers that might need more context)
        let mut distances: Vec<(String, f32)> = self.vortex.subjects.iter()
            .filter(|(_, node)| node.embedding.len() == embed_dim)
            .map(|(subject, node)| {
                let dist = 1.0 - cosine_similarity(&node.embedding, &centroid);
                (subject.clone(), dist)
            })
            .collect();
        
        distances.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Generate queries for outlier subjects (they might be poorly connected)
        for (subject, _) in distances.into_iter().take(10) {
            queries.push(format!("{} common associations and context", subject));
        }
        
        queries
    }
    
    /// Find keywords that appear in few subjects (potential knowledge gaps)
    fn find_keyword_gaps(&self) -> Vec<String> {
        let mut keyword_counts: HashMap<String, usize> = HashMap::new();
        
        // Count keyword occurrences across subjects
        for subjects in self.vortex.keyword_index.values() {
            for subject in subjects {
                *keyword_counts.entry(subject.clone()).or_insert(0) += 1;
            }
        }
        
        // Find keywords that appear only once (isolated knowledge)
        self.vortex.keyword_index.keys()
            .filter(|kw| {
                self.vortex.keyword_index.get(*kw)
                    .map(|subjects| subjects.len() == 1)
                    .unwrap_or(false)
            })
            .take(15)
            .cloned()
            .collect()
    }
    
    /// Analyze the current knowledge state and return a gap analysis report
    pub fn analyze_knowledge_gaps(&self) -> KnowledgeGapAnalysis {
        let total_subjects = self.vortex.subjects.len();
        let low_confidence = self.find_low_confidence_subjects(0.5).len();
        let isolated = self.find_isolated_subjects(2).len();
        let incomplete = self.find_incomplete_subjects().len();
        
        // Calculate average confidence
        let avg_confidence = if total_subjects > 0 {
            self.vortex.subjects.values()
                .map(|n| n.confidence)
                .sum::<f32>() / total_subjects as f32
        } else {
            0.0
        };
        
        // Calculate average relations per subject
        let avg_relations = if total_subjects > 0 {
            self.vortex.subjects.values()
                .map(|n| n.relations.len())
                .sum::<usize>() as f32 / total_subjects as f32
        } else {
            0.0
        };
        
        // Calculate knowledge density (attributes + relations per subject)
        let knowledge_density = if total_subjects > 0 {
            self.vortex.subjects.values()
                .map(|n| n.attributes.len() + n.relations.len())
                .sum::<usize>() as f32 / total_subjects as f32
        } else {
            0.0
        };
        
        KnowledgeGapAnalysis {
            total_subjects,
            low_confidence_count: low_confidence,
            isolated_count: isolated,
            incomplete_count: incomplete,
            avg_confidence,
            avg_relations,
            knowledge_density,
            recommended_queries: self.generate_improvement_queries().len(),
        }
    }

    /// Process extracted web content through sacred attention pipeline
    pub fn process_web_content(&mut self, content: &str, source: &str) {
        // Position 3: Keyword Extraction
        let keywords_and_facts = self.sacred_processors.position_3_extract_keywords(content);
        self.stats.facts_extracted += keywords_and_facts.len();
        
        // Position 6: Relation Verification
        let verified_facts = self.sacred_processors.position_6_verify_relations(
            &keywords_and_facts,
            &self.world_model,
        );
        self.stats.facts_verified += verified_facts.len();
        
        // Position 9: Knowledge Integration
        for fact in verified_facts {
            if fact.confidence >= self.config.min_confidence {
                self.integrate_fact(&fact, source);
                self.stats.facts_integrated += 1;
            }
        }
    }

    /// Integrate a verified fact into the vortex and world model
    fn integrate_fact(&mut self, fact: &ExtractedFact, source: &str) {
        // Add to vortex
        self.vortex.add_knowledge(
            &fact.subject,
            &fact.attribute,
            &fact.value,
            fact.confidence,
            source,
        );
        
        // Add relation if present
        if let Some(ref target) = fact.related_subject {
            self.vortex.add_relation(
                &fact.subject,
                &fact.relation_type,
                target,
                fact.confidence,
            );
        }
        
        // Update world model
        self.world_model.process_context(&format!(
            "{} {} {}",
            fact.subject, fact.attribute, fact.value
        ));
        
        // Update knowledge graph using add_triple
        use crate::ml::world_knowledge::RelationType;
        let relation = match fact.attribute.as_str() {
            "location" | "found_at" | "located_in" => RelationType::LocatedAt,
            "is" | "is_a" => RelationType::IsA,
            "has" => RelationType::HasProperty,
            "can" | "capable_of" => RelationType::CapableOf,
            "function" | "used_for" => RelationType::UsedFor,
            _ => RelationType::HasProperty,
        };
        self.knowledge_graph.add_triple(&fact.subject, relation, &fact.value, fact.confidence);
    }

    /// Score a choice using consciousness-learned knowledge
    pub fn score_choice(&mut self, question: &str, choice: &str) -> f32 {
        let mut score = 0.0;
        
        // 1. Query vortex for relevant knowledge
        let relevant = self.vortex.query_knowledge(question);
        
        for (node, relevance) in relevant {
            // Check if choice matches any attribute
            for (attr, attr_val) in &node.attributes {
                if choice.to_lowercase().contains(&attr_val.value.to_lowercase()) {
                    score += relevance * attr_val.confidence * 10.0;
                }
            }
            
            // Check if choice matches any relation target
            for (_, target, conf) in &node.relations {
                if choice.to_lowercase().contains(&target.to_lowercase()) {
                    score += relevance * conf * 8.0;
                }
            }
            
            // Check keyword overlap
            let choice_words: std::collections::HashSet<_> = choice.to_lowercase()
                .split_whitespace()
                .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
                .collect();
            
            let keyword_overlap = node.keywords.iter()
                .filter(|k| choice_words.contains(*k))
                .count();
            
            score += relevance * keyword_overlap as f32 * 2.0;
        }
        
        // 2. Use geometric world model
        let (_, geo_conf) = self.world_model.answer_question(question, &[choice.to_string()]);
        score += geo_conf * 5.0;
        
        score
    }

    /// Pre-benchmark learning phase
    /// Analyzes failures and learns relevant knowledge before testing
    /// Now with REAL web learning from DuckDuckGo
    pub fn learn_before_benchmark(&mut self, categories: &[&str]) -> LearningStats {
        let start = Instant::now();
        
        // Reset stats for this session
        self.stats = LearningStats::default();
        self.seen_domains.clear();
        
        // Generate queries based on categories
        let mut queries = Vec::new();
        for category in categories {
            queries.extend(self.generate_category_queries(category));
        }
        
        // Also add queries from dynamic knowledge gap analysis
        queries.extend(self.generate_improvement_queries());
        
        // Deduplicate
        queries.sort();
        queries.dedup();
        queries.truncate(self.config.max_queries_per_session);
        
        self.stats.queries_generated = queries.len();
        
        println!("   [Web Learning] Starting with {} queries...", queries.len());
        
        // Perform web learning if enabled
        if self.config.enable_web_search {
            self.learn_from_web(&queries);
        }
        
        // Also use built-in commonsense knowledge as fallback
        for query in &queries {
            self.learn_from_query(query);
        }
        
        self.stats.learning_time_ms = start.elapsed().as_millis() as u64;
        self.stats.subjects_created = self.vortex.subjects.len();
        self.stats.unique_domains = self.seen_domains.len();
        
        self.stats.clone()
    }

    /// Generate queries for a specific benchmark category
    /// DEPRECATED: No longer used - we rely on dynamic test-time learning only
    fn generate_category_queries(&self, _category: &str) -> Vec<String> {
        // Return empty - all learning happens dynamically at test time
        Vec::new()
    }

    /// Learn from web searches - PARALLEL HIGH-THROUGHPUT version
    fn learn_from_web(&mut self, queries: &[String]) {
        use crate::ml::web_knowledge::WebKnowledge;
        use rayon::prelude::*;
        use std::sync::{Arc, Mutex};
        
        let batch_start = Instant::now();
        let batch_size = self.config.query_batch_size;
        
        println!("   [Web Learning] PARALLEL MODE: {} queries in batches of {}", 
                 queries.len(), batch_size);
        
        // Process queries in parallel batches
        let total_websites = Arc::new(Mutex::new(0usize));
        let total_facts = Arc::new(Mutex::new(0usize));
        let all_domains = Arc::new(Mutex::new(std::collections::HashSet::new()));
        let all_knowledge = Arc::new(Mutex::new(Vec::new()));
        let search_count = Arc::new(Mutex::new(0usize));
        let error_count = Arc::new(Mutex::new(0usize));
        
        // Process in parallel batches
        let config = self.config.clone();
        queries.par_chunks(batch_size).enumerate().for_each(|(batch_idx, batch)| {
            let web_config = crate::ml::web_knowledge::WebScraperConfig {
                timeout_secs: config.request_timeout_secs,
                max_results: 30,
                request_delay_ms: 0, // Zero delay for maximum throughput
            };
            
            let mut batch_websites = 0;
            let mut batch_facts = 0;
            let mut batch_domains = std::collections::HashSet::new();
            let mut batch_knowledge = Vec::new();
            
            // Process queries in this batch in parallel - each gets its own scraper
            let results: Vec<_> = batch.par_iter().map(|query| {
                let mut scraper = crate::ml::web_knowledge::DuckDuckGoScraper::new(web_config.clone());
                let extractor = crate::ml::web_knowledge::WebKnowledgeExtractor::new();
                
                println!("   [Web Learning] Searching: {}", query);
                match scraper.search_sync(query) {
                    Ok(results) => {
                        println!("   [Web Learning] ✓ Query '{}' returned {} results", query, results.len());
                        if results.is_empty() {
                            eprintln!("   [Web Learning] Warning: Query '{}' returned 0 results", query);
                        }
                        let knowledge = extractor.extract_from_results(&results, query);
                        println!("   [Web Learning] Extracted {} facts from query '{}'", knowledge.len(), query);
                        Some((results, knowledge))
                    }
                    Err(e) => {
                        eprintln!("   [Web Learning] ✗ Error for query '{}': {}", query, e);
                        None
                    }
                }
            }).collect();
            
            // Aggregate results
            for result in results {
                if let Some((search_results, knowledge)) = result {
                    *search_count.lock().unwrap() += 1;
                    
                    for result in &search_results {
                        batch_websites += 1;
                        if let Some(domain) = Self::extract_domain(&result.url) {
                            batch_domains.insert(domain);
                        }
                    }
                    
                    batch_facts += knowledge.len();
                    batch_knowledge.extend(knowledge);
                } else {
                    *error_count.lock().unwrap() += 1;
                }
            }
            
            // Update global stats
            *total_websites.lock().unwrap() += batch_websites;
            *total_facts.lock().unwrap() += batch_facts;
            all_domains.lock().unwrap().extend(batch_domains);
            all_knowledge.lock().unwrap().extend(batch_knowledge);
            
            // Progress update
            let elapsed = batch_start.elapsed().as_secs_f32();
            let rate = *total_websites.lock().unwrap() as f32 / elapsed.max(0.001);
            println!("   [Batch {}] {} websites, {} facts ({:.1} websites/sec)", 
                     batch_idx + 1, batch_websites, batch_facts, rate);
        });
        
        // Integrate all knowledge sequentially (vortex not thread-safe)
        let knowledge_vec = Arc::try_unwrap(all_knowledge).unwrap().into_inner().unwrap();
        for k in knowledge_vec {
            self.integrate_web_knowledge(&k);
        }
        
        // Update stats
        self.stats.web_searches = *search_count.lock().unwrap();
        self.stats.websites_referenced = *total_websites.lock().unwrap();
        self.stats.facts_extracted = *total_facts.lock().unwrap();
        self.stats.search_errors = *error_count.lock().unwrap();
        self.seen_domains = Arc::try_unwrap(all_domains).unwrap().into_inner().unwrap();
        
        let total_time = batch_start.elapsed();
        let throughput = self.stats.websites_referenced as f32 / total_time.as_secs_f32();
        println!("   [Web Learning] 🚀 COMPLETED: {} websites from {} searches in {:.2}s",
                 self.stats.websites_referenced, self.stats.web_searches, total_time.as_secs_f32());
        println!("   [Web Learning] 📊 THROUGHPUT: {:.1} websites/sec | {} facts from {} domains",
                 throughput, self.stats.facts_extracted, self.seen_domains.len());
    }
    
    /// Extract domain from URL
    fn extract_domain(url: &str) -> Option<String> {
        // Simple domain extraction
        let url = url.trim_start_matches("https://")
                     .trim_start_matches("http://")
                     .trim_start_matches("www.");
        
        url.split('/').next()
           .map(|s| s.to_lowercase())
    }
    
    /// Integrate web knowledge into the vortex with critical thinking
    fn integrate_web_knowledge(&mut self, knowledge: &crate::ml::web_knowledge::WebKnowledge) {
        // Critical thinking: verify confidence based on source reliability
        let mut confidence = knowledge.confidence;
        
        // Boost confidence for known reliable domains
        let domain = Self::extract_domain(&knowledge.source).unwrap_or_default();
        if domain.contains("wikipedia") || domain.contains("britannica") || 
           domain.contains("edu") || domain.contains("gov") {
            confidence = (confidence * 1.2).min(1.0);
        }
        
        // Reduce confidence for potentially unreliable sources
        if domain.contains("reddit") || domain.contains("quora") || domain.contains("yahoo") {
            confidence *= 0.8;
        }
        
        // Only integrate if confidence is above threshold
        if confidence >= self.config.min_confidence {
            // Add to vortex
            self.vortex.add_knowledge(
                &knowledge.subject,
                &knowledge.attribute,
                &knowledge.value,
                confidence,
                &knowledge.source,
            );
            
            // Add relations for related subjects
            for related in &knowledge.related {
                self.vortex.add_relation(
                    &knowledge.subject,
                    "related_to",
                    related,
                    confidence * 0.8,
                );
            }
            
            self.stats.facts_integrated += 1;
        } else {
            // Track verified but not integrated (low confidence)
            self.stats.facts_verified += 1;
        }
    }
    
    /// Learn from a single query using built-in knowledge
    fn learn_from_query(&mut self, query: &str) {
        // Extract concepts from query
        let concepts = extract_key_concepts(query);
        
        for concept in concepts {
            // Add basic commonsense knowledge
            self.add_commonsense_for_concept(&concept);
        }
    }

    /// Add commonsense knowledge for a concept
    fn add_commonsense_for_concept(&mut self, concept: &str) {
        // This would be replaced with actual web search results
        // For now, use built-in commonsense patterns
        
        let concept_lower = concept.to_lowercase();
        
        // Location knowledge
        let location_map: HashMap<&str, &str> = [
            ("hamburger", "restaurant"),
            ("book", "library"),
            ("medicine", "pharmacy"),
            ("bread", "bakery"),
            ("fish", "ocean"),
            ("tree", "forest"),
            ("car", "garage"),
            ("plane", "airport"),
            ("train", "station"),
            ("money", "bank"),
            ("clothes", "closet"),
            ("food", "kitchen"),
            ("bed", "bedroom"),
            ("shower", "bathroom"),
            ("desk", "office"),
        ].into_iter().collect();
        
        if let Some(&location) = location_map.get(concept_lower.as_str()) {
            self.vortex.add_knowledge(
                concept,
                "typical_location",
                location,
                0.9,
                "commonsense",
            );
            self.vortex.add_relation(concept, "found_at", location, 0.9);
        }
        
        // Physical properties
        let physical_map: HashMap<&str, (&str, &str)> = [
            ("water", ("state", "liquid")),
            ("ice", ("state", "solid")),
            ("steam", ("state", "gas")),
            ("rock", ("hardness", "hard")),
            ("cotton", ("texture", "soft")),
            ("glass", ("property", "transparent")),
            ("metal", ("property", "conductive")),
            ("wood", ("property", "flammable")),
            ("rubber", ("property", "elastic")),
        ].into_iter().collect();
        
        if let Some(&(attr, value)) = physical_map.get(concept_lower.as_str()) {
            self.vortex.add_knowledge(concept, attr, value, 0.9, "commonsense");
        }
        
        // Tool/function knowledge
        let function_map: HashMap<&str, &str> = [
            ("hammer", "hitting nails"),
            ("knife", "cutting"),
            ("scissors", "cutting paper"),
            ("pen", "writing"),
            ("phone", "communication"),
            ("computer", "computing"),
            ("car", "transportation"),
            ("oven", "cooking"),
            ("refrigerator", "cooling food"),
            ("washing machine", "cleaning clothes"),
        ].into_iter().collect();
        
        if let Some(&function) = function_map.get(concept_lower.as_str()) {
            self.vortex.add_knowledge(concept, "function", function, 0.9, "commonsense");
        }
    }

    /// Get vortex statistics
    pub fn get_stats(&self) -> (LearningStats, VortexStats) {
        (self.stats.clone(), self.vortex.stats())
    }
}

// =============================================================================
// SACRED ATTENTION PIPELINE - 3-6-9 Verification
// =============================================================================

/// Extracted fact from web content
#[derive(Debug, Clone)]
pub struct ExtractedFact {
    pub subject: String,
    pub attribute: String,
    pub value: String,
    pub confidence: f32,
    pub relation_type: String,
    pub related_subject: Option<String>,
    pub keywords: Vec<String>,
}

/// Sacred attention pipeline for 3-6-9 verification
struct SacredAttentionPipeline {
    embed_dim: usize,
    encoder: WorldEncoder,
}

impl SacredAttentionPipeline {
    fn new(embed_dim: usize) -> Self {
        Self {
            embed_dim,
            encoder: WorldEncoder::new(embed_dim),
        }
    }

    /// Position 3: Keyword Extraction
    /// Extracts subjects, objects, actions, and keywords from text
    fn position_3_extract_keywords(&self, content: &str) -> Vec<ExtractedFact> {
        let mut facts = Vec::new();
        
        // Split into sentences
        for sentence in content.split(|c| c == '.' || c == '!' || c == '?') {
            let sentence = sentence.trim();
            if sentence.len() < 10 {
                continue;
            }
            
            // Extract subject-verb-object patterns
            if let Some(fact) = self.extract_svo_pattern(sentence) {
                facts.push(fact);
            }
            
            // Extract "X is Y" patterns
            if let Some(fact) = self.extract_is_pattern(sentence) {
                facts.push(fact);
            }
            
            // Extract "X has Y" patterns
            if let Some(fact) = self.extract_has_pattern(sentence) {
                facts.push(fact);
            }
            
            // Extract location patterns
            if let Some(fact) = self.extract_location_pattern(sentence) {
                facts.push(fact);
            }
        }
        
        facts
    }

    /// Extract subject-verb-object pattern
    fn extract_svo_pattern(&self, sentence: &str) -> Option<ExtractedFact> {
        let words: Vec<&str> = sentence.split_whitespace().collect();
        if words.len() < 3 {
            return None;
        }
        
        // Simple heuristic: first noun-like word is subject
        let subject = words[0].trim_matches(|c: char| !c.is_alphanumeric());
        
        // Look for verb indicators
        let verbs = ["is", "are", "was", "were", "has", "have", "can", "will", "does", "do"];
        let verb_idx = words.iter().position(|w| verbs.contains(&w.to_lowercase().as_str()))?;
        
        // Object is after verb
        if verb_idx + 1 < words.len() {
            let object: String = words[verb_idx + 1..].join(" ")
                .trim_matches(|c: char| !c.is_alphanumeric() && c != ' ')
                .to_string();
            
            if !object.is_empty() && subject.len() > 1 {
                return Some(ExtractedFact {
                    subject: subject.to_string(),
                    attribute: words[verb_idx].to_lowercase(),
                    value: object,
                    confidence: 0.6,
                    relation_type: "has_property".to_string(),
                    related_subject: None,
                    keywords: words.iter()
                        .map(|w| w.to_lowercase().trim_matches(|c: char| !c.is_alphanumeric()).to_string())
                        .filter(|w| w.len() > 2)
                        .collect(),
                });
            }
        }
        
        None
    }

    /// Extract "X is Y" pattern
    fn extract_is_pattern(&self, sentence: &str) -> Option<ExtractedFact> {
        let lower = sentence.to_lowercase();
        
        // Pattern: "X is a/an Y" or "X is Y"
        if let Some(is_idx) = lower.find(" is ") {
            let subject = sentence[..is_idx].trim()
                .split_whitespace().last()?
                .trim_matches(|c: char| !c.is_alphanumeric());
            
            let rest = &sentence[is_idx + 4..];
            let value = rest.split_whitespace()
                .take(5)
                .collect::<Vec<_>>()
                .join(" ")
                .trim_matches(|c: char| !c.is_alphanumeric() && c != ' ')
                .to_string();
            
            if subject.len() > 1 && value.len() > 1 {
                return Some(ExtractedFact {
                    subject: subject.to_string(),
                    attribute: "is".to_string(),
                    value,
                    confidence: 0.7,
                    relation_type: "is_a".to_string(),
                    related_subject: None,
                    keywords: vec![subject.to_lowercase()],
                });
            }
        }
        
        None
    }

    /// Extract "X has Y" pattern
    fn extract_has_pattern(&self, sentence: &str) -> Option<ExtractedFact> {
        let lower = sentence.to_lowercase();
        
        if let Some(has_idx) = lower.find(" has ") {
            let subject = sentence[..has_idx].trim()
                .split_whitespace().last()?
                .trim_matches(|c: char| !c.is_alphanumeric());
            
            let rest = &sentence[has_idx + 5..];
            let value = rest.split_whitespace()
                .take(5)
                .collect::<Vec<_>>()
                .join(" ")
                .trim_matches(|c: char| !c.is_alphanumeric() && c != ' ')
                .to_string();
            
            if subject.len() > 1 && value.len() > 1 {
                return Some(ExtractedFact {
                    subject: subject.to_string(),
                    attribute: "has".to_string(),
                    value,
                    confidence: 0.7,
                    relation_type: "has".to_string(),
                    related_subject: None,
                    keywords: vec![subject.to_lowercase()],
                });
            }
        }
        
        None
    }

    /// Extract location patterns
    fn extract_location_pattern(&self, sentence: &str) -> Option<ExtractedFact> {
        let lower = sentence.to_lowercase();
        
        // Patterns: "X is found in Y", "X is located in Y", "X can be found at Y"
        let location_markers = [
            " is found in ", " is located in ", " can be found at ",
            " is found at ", " is in ", " are found in ",
        ];
        
        for marker in location_markers {
            if let Some(idx) = lower.find(marker) {
                let subject = sentence[..idx].trim()
                    .split_whitespace().last()?
                    .trim_matches(|c: char| !c.is_alphanumeric());
                
                let rest = &sentence[idx + marker.len()..];
                let location = rest.split_whitespace()
                    .take(3)
                    .collect::<Vec<_>>()
                    .join(" ")
                    .trim_matches(|c: char| !c.is_alphanumeric() && c != ' ')
                    .to_string();
                
                if subject.len() > 1 && location.len() > 1 {
                    return Some(ExtractedFact {
                        subject: subject.to_string(),
                        attribute: "location".to_string(),
                        value: location.clone(),
                        confidence: 0.8,
                        relation_type: "found_at".to_string(),
                        related_subject: Some(location),
                        keywords: vec![subject.to_lowercase()],
                    });
                }
            }
        }
        
        None
    }

    /// Position 6: Relation Verification
    /// Verifies extracted facts against existing knowledge
    fn position_6_verify_relations(
        &self,
        facts: &[ExtractedFact],
        world_model: &GeometricWorldModel,
    ) -> Vec<ExtractedFact> {
        let mut verified = Vec::new();
        
        for fact in facts {
            let mut adjusted_confidence = fact.confidence;
            
            // Check geometric consistency
            if !fact.subject.is_empty() && fact.related_subject.is_some() {
                let target = fact.related_subject.as_ref().unwrap();
                
                // Encode and check relation
                let source_embed = self.encoder.encode_entity(&fact.subject);
                let target_embed = self.encoder.encode_entity(target);
                
                // Check if relation is geometrically plausible
                let (_, geo_conf) = world_model.predictor.check_relation(
                    &source_embed,
                    &fact.relation_type,
                    &target_embed,
                );
                
                // Boost or reduce confidence based on geometric verification
                if geo_conf > 0.3 {
                    adjusted_confidence = (adjusted_confidence + geo_conf) / 2.0;
                } else {
                    adjusted_confidence *= 0.8; // Slight penalty for unverified
                }
            }
            
            // Only keep facts with sufficient confidence
            if adjusted_confidence >= 0.3 {
                let mut verified_fact = fact.clone();
                verified_fact.confidence = adjusted_confidence;
                verified.push(verified_fact);
            }
        }
        
        verified
    }
}

// =============================================================================
// UTILITY FUNCTIONS
// =============================================================================

/// Extract key concepts from text
fn extract_key_concepts(text: &str) -> Vec<String> {
    let stopwords: std::collections::HashSet<&str> = [
        "the", "a", "an", "is", "are", "was", "were", "be", "been", "being",
        "have", "has", "had", "do", "does", "did", "will", "would", "could",
        "should", "may", "might", "must", "shall", "can", "need", "dare",
        "ought", "used", "to", "of", "in", "for", "on", "with", "at", "by",
        "from", "as", "into", "through", "during", "before", "after", "above",
        "below", "between", "under", "again", "further", "then", "once", "here",
        "there", "when", "where", "why", "how", "all", "each", "few", "more",
        "most", "other", "some", "such", "no", "nor", "not", "only", "own",
        "same", "so", "than", "too", "very", "just", "and", "but", "if", "or",
        "because", "until", "while", "what", "which", "who", "whom", "this",
        "that", "these", "those", "am", "it", "its", "i", "you", "he", "she",
        "we", "they", "my", "your", "his", "her", "our", "their",
    ].into_iter().collect();
    
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > 2 && !stopwords.contains(w))
        .map(|s| s.to_string())
        .collect()
}

/// Cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let mut dot = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;
    
    for i in 0..a.len().min(b.len()) {
        dot += a[i] * b[i];
        norm_a += a[i] * a[i];
        norm_b += b[i] * b[i];
    }
    
    if norm_a > 1e-8 && norm_b > 1e-8 {
        dot / (norm_a.sqrt() * norm_b.sqrt())
    } else {
        0.0
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subject_node_creation() {
        let node = SubjectNode::new("hamburger", vec![0.0; 64]);
        assert_eq!(node.subject, "hamburger");
        assert!(node.keywords.contains(&"hamburger".to_string()));
        assert_eq!(node.flux_position, 1);
    }

    #[test]
    fn test_subject_node_attributes() {
        let mut node = SubjectNode::new("hamburger", vec![0.0; 64]);
        node.add_attribute("location", "restaurant", 0.9, "web");
        
        assert!(node.attributes.contains_key("location"));
        assert_eq!(node.attributes["location"].value, "restaurant");
    }

    #[test]
    fn test_dynamic_vortex() {
        let mut vortex = DynamicVortex::new(64);
        
        vortex.add_knowledge("hamburger", "location", "restaurant", 0.9, "test");
        vortex.add_relation("hamburger", "found_at", "restaurant", 0.9);
        
        assert!(vortex.subjects.contains_key("hamburger"));
        
        let results = vortex.search_by_keyword("hamburger");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_consciousness_learner() {
        let config = ConsciousnessConfig::default();
        let mut learner = ConsciousnessLearner::new(config);
        
        // Log a failure
        learner.log_failure(
            "Where would you find a hamburger?",
            "restaurant",
            "kitchen",
            "commonsenseqa",
        );
        
        // Generate improvement queries
        let queries = learner.generate_improvement_queries();
        assert!(!queries.is_empty());
    }

    #[test]
    fn test_pre_benchmark_learning() {
        let config = ConsciousnessConfig::default();
        let mut learner = ConsciousnessLearner::new(config);
        
        let stats = learner.learn_before_benchmark(&["commonsenseqa", "piqa"]);
        
        assert!(stats.queries_generated > 0);
        println!("Learning stats: {:?}", stats);
    }

    #[test]
    fn test_sacred_attention_extraction() {
        let pipeline = SacredAttentionPipeline::new(64);
        
        let content = "A hamburger is found in a restaurant. Water is a liquid. The hammer is used for hitting nails.";
        let facts = pipeline.position_3_extract_keywords(content);
        
        assert!(!facts.is_empty());
        for fact in &facts {
            println!("Extracted: {} {} {}", fact.subject, fact.attribute, fact.value);
        }
    }

    #[test]
    fn test_extract_key_concepts() {
        let concepts = extract_key_concepts("Where would you find a hamburger in a restaurant?");
        assert!(concepts.contains(&"hamburger".to_string()));
        assert!(concepts.contains(&"restaurant".to_string()));
        assert!(!concepts.contains(&"the".to_string()));
    }
    
    #[test]
    fn test_dynamic_knowledge_gap_detection() {
        let config = ConsciousnessConfig::default();
        let mut learner = ConsciousnessLearner::new(config);
        
        // Empty vortex should generate foundational queries
        let queries = learner.generate_improvement_queries();
        assert!(!queries.is_empty(), "Should generate queries even with empty vortex");
        
        // Check for foundational queries
        let has_foundational = queries.iter().any(|q| 
            q.contains("household") || q.contains("physical") || q.contains("tools")
        );
        assert!(has_foundational, "Empty vortex should generate foundational queries");
    }
    
    #[test]
    fn test_knowledge_gap_analysis() {
        let config = ConsciousnessConfig::default();
        let mut learner = ConsciousnessLearner::new(config);
        
        // Empty knowledge base
        let analysis = learner.analyze_knowledge_gaps();
        assert_eq!(analysis.total_subjects, 0);
        assert_eq!(analysis.health_score(), 0.0);
        assert!(analysis.needs_learning());
        
        // Add some knowledge
        learner.vortex.add_knowledge("hamburger", "location", "restaurant", 0.9, "test");
        learner.vortex.add_knowledge("book", "location", "library", 0.8, "test");
        learner.vortex.add_relation("hamburger", "found_at", "restaurant", 0.9);
        
        let analysis = learner.analyze_knowledge_gaps();
        // Note: add_relation creates "restaurant" as a target subject too
        assert!(analysis.total_subjects >= 2, "Should have at least 2 subjects, got {}", analysis.total_subjects);
        assert!(analysis.avg_confidence > 0.0);
        println!("Health score: {}", analysis.health_score());
        println!("Priority areas: {:?}", analysis.priority_areas());
    }
    
    #[test]
    fn test_find_low_confidence_subjects() {
        let config = ConsciousnessConfig::default();
        let mut learner = ConsciousnessLearner::new(config);
        
        // Add subjects - node confidence starts at 0.5 by default
        learner.vortex.add_knowledge("subject1", "attr", "value", 0.9, "test");
        learner.vortex.add_knowledge("subject2", "attr", "value", 0.3, "test");
        
        // Manually set confidence to test the filter
        if let Some(node) = learner.vortex.subjects.get_mut("subject1") {
            node.confidence = 0.9;
        }
        if let Some(node) = learner.vortex.subjects.get_mut("subject2") {
            node.confidence = 0.3;
        }
        
        let low_conf = learner.find_low_confidence_subjects(0.5);
        assert!(low_conf.contains(&"subject2".to_string()));
        assert!(!low_conf.contains(&"subject1".to_string()));
    }
    
    #[test]
    fn test_find_isolated_subjects() {
        let config = ConsciousnessConfig::default();
        let mut learner = ConsciousnessLearner::new(config);
        
        // Add subject with relations
        learner.vortex.add_knowledge("connected", "attr", "value", 0.9, "test");
        learner.vortex.add_relation("connected", "rel1", "target1", 0.9);
        learner.vortex.add_relation("connected", "rel2", "target2", 0.9);
        
        // Add isolated subject
        learner.vortex.add_knowledge("isolated", "attr", "value", 0.9, "test");
        
        let isolated = learner.find_isolated_subjects(2);
        assert!(isolated.contains(&"isolated".to_string()));
        assert!(!isolated.contains(&"connected".to_string()));
    }
    
    #[test]
    fn test_find_incomplete_subjects() {
        let config = ConsciousnessConfig::default();
        let mut learner = ConsciousnessLearner::new(config);
        
        // Add subject with partial attributes
        learner.vortex.add_knowledge("partial", "location", "somewhere", 0.9, "test");
        // Missing: function, is, has, typical_location
        
        let incomplete = learner.find_incomplete_subjects();
        assert!(!incomplete.is_empty());
        
        // Check that missing attrs are identified
        let (_, missing) = incomplete.iter().find(|(s, _)| s == "partial").unwrap();
        assert!(missing.contains(&"function".to_string()));
    }
    
    #[test]
    fn test_sparse_region_detection() {
        let config = ConsciousnessConfig::default();
        let mut learner = ConsciousnessLearner::new(config);
        
        // Add several subjects to create a distribution
        for i in 0..5 {
            learner.vortex.add_knowledge(
                &format!("subject_{}", i),
                "attr",
                "value",
                0.8,
                "test"
            );
        }
        
        let sparse_queries = learner.find_sparse_embedding_regions();
        // Should generate queries for outliers
        assert!(!sparse_queries.is_empty());
    }
    
    #[test]
    fn test_knowledge_gap_health_score() {
        let analysis = KnowledgeGapAnalysis {
            total_subjects: 100,
            low_confidence_count: 10,
            isolated_count: 20,
            incomplete_count: 15,
            avg_confidence: 0.8,
            avg_relations: 2.5,
            knowledge_density: 4.0,
            recommended_queries: 5,
        };
        
        let score = analysis.health_score();
        assert!(score > 0.5 && score < 1.0, "Health score should be reasonable: {}", score);
        assert!(!analysis.needs_learning(), "Healthy knowledge base shouldn't need learning");
    }
}
