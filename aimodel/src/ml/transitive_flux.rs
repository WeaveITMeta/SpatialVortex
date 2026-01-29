//! Transitive Flux Reasoner
//!
//! Uses the Vortex Flux Matrix with infinite ladder index for transitive reasoning.
//! Integrates with JEPA + Exhaustive Pathway Search for optimal reasoning paths.
//!
//! Key concepts:
//! - **Ladder Index**: Relations stack transitively (A→B, B→C implies A→C)
//! - **Flux Matrix**: 9-position vortex with sacred guides at 3, 6, 9
//! - **Vector Similarity**: Cosine similarity in latent space for nearby concepts
//! - **Statistical Probability**: Confidence propagation through transitive chains

use std::collections::HashMap;

// =============================================================================
// Transitive Relation Graph
// =============================================================================

/// A relation in the transitive graph with ladder index
#[derive(Debug, Clone)]
pub struct TransitiveRelation {
    /// Source entity
    pub source: String,
    /// Relation type (e.g., "left_of", "bigger_than", "is_a")
    pub relation: String,
    /// Target entity
    pub target: String,
    /// Ladder rank (lower = higher priority in chain)
    pub ladder_rank: u8,
    /// Confidence/weight of this relation
    pub confidence: f32,
    /// Embedding of this relation triple
    pub embedding: Vec<f32>,
}

/// Transitive reasoning engine using flux matrix ladder indices
#[derive(Debug)]
pub struct TransitiveFluxReasoner {
    /// Direct relations extracted from context
    relations: Vec<TransitiveRelation>,
    /// Entity embeddings for similarity search
    entity_embeddings: HashMap<String, Vec<f32>>,
    /// Relation type embeddings
    relation_embeddings: HashMap<String, Vec<f32>>,
    /// Embedding dimension
    embed_dim: usize,
    /// Maximum transitive chain depth
    max_chain_depth: usize,
    /// Confidence decay per hop
    confidence_decay: f32,
}

impl TransitiveFluxReasoner {
    pub fn new(embed_dim: usize) -> Self {
        Self {
            relations: Vec::new(),
            entity_embeddings: HashMap::new(),
            relation_embeddings: HashMap::new(),
            embed_dim,
            max_chain_depth: 9, // Sacred 9 - maximum ladder depth
            confidence_decay: 0.9, // 10% decay per hop
        }
    }
    
    /// Extract relations from context text
    /// Parses patterns like "A is left of B", "A is bigger than B", etc.
    pub fn extract_relations(&mut self, context: &str) {
        self.relations.clear();
        let context_lower = context.to_lowercase();
        
        // Spatial relations (bAbI Task 17)
        let spatial_patterns = [
            ("is to the left of", "left_of"),
            ("is left of", "left_of"),
            ("is to the right of", "right_of"),
            ("is right of", "right_of"),
            ("is above", "above"),
            ("is below", "below"),
        ];
        
        // Size relations (bAbI Task 18)
        let size_patterns = [
            ("is bigger than", "bigger_than"),
            ("is larger than", "bigger_than"),
            ("is smaller than", "smaller_than"),
            ("fits inside", "fits_inside"),
            ("fits in", "fits_inside"),
        ];
        
        let mut ladder_rank = 1u8;
        
        for sentence in context_lower.split(|c| c == '.' || c == '\n') {
            let sentence = sentence.trim();
            if sentence.is_empty() { continue; }
            
            // Try spatial patterns
            for (pattern, rel_type) in &spatial_patterns {
                if let Some((source, target)) = self.parse_relation_pattern(sentence, pattern) {
                    self.add_relation(source, rel_type.to_string(), target, ladder_rank, 1.0);
                    ladder_rank = ladder_rank.saturating_add(1);
                }
            }
            
            // Try size patterns
            for (pattern, rel_type) in &size_patterns {
                if let Some((source, target)) = self.parse_relation_pattern(sentence, pattern) {
                    self.add_relation(source, rel_type.to_string(), target, ladder_rank, 1.0);
                    ladder_rank = ladder_rank.saturating_add(1);
                }
            }
        }
        
        // Compute transitive closure
        self.compute_transitive_closure();
    }
    
    /// Parse "A <pattern> B" and return (A, B)
    /// Handles multi-word entities like "pink rectangle", "red square"
    fn parse_relation_pattern(&self, sentence: &str, pattern: &str) -> Option<(String, String)> {
        if let Some(pos) = sentence.find(pattern) {
            let before = sentence[..pos].trim();
            let after = sentence[pos + pattern.len()..].trim();
            
            // Extract source - could be multi-word like "the pink rectangle"
            // Take everything before pattern, remove leading "the"
            let source = before
                .trim_start_matches("the ")
                .trim()
                .to_string();
            
            // Extract target - could be multi-word like "the red square"
            // Take everything after pattern until end or punctuation
            let target = after
                .split(|c: char| c == '.' || c == '?' || c == '!')
                .next()
                .unwrap_or("")
                .trim_start_matches("the ")
                .trim()
                .to_string();
            
            if !source.is_empty() && !target.is_empty() && source.len() > 1 && target.len() > 1 {
                return Some((source, target));
            }
        }
        None
    }
    
    /// Add a relation to the graph
    fn add_relation(&mut self, source: String, relation: String, target: String, ladder_rank: u8, confidence: f32) {
        // Create embedding for this triple
        let embedding = self.create_triple_embedding(&source, &relation, &target);
        
        self.relations.push(TransitiveRelation {
            source,
            relation,
            target,
            ladder_rank,
            confidence,
            embedding,
        });
    }
    
    /// Create embedding for a relation triple
    fn create_triple_embedding(&self, source: &str, relation: &str, target: &str) -> Vec<f32> {
        let mut embed = vec![0.0; self.embed_dim];
        
        // Hash-based embedding (simple but deterministic)
        let source_hash = self.hash_string(source);
        let rel_hash = self.hash_string(relation);
        let target_hash = self.hash_string(target);
        
        for i in 0..self.embed_dim {
            let s = ((source_hash.wrapping_add(i as u64)) as f32 / u64::MAX as f32) * 2.0 - 1.0;
            let r = ((rel_hash.wrapping_add(i as u64)) as f32 / u64::MAX as f32) * 2.0 - 1.0;
            let t = ((target_hash.wrapping_add(i as u64)) as f32 / u64::MAX as f32) * 2.0 - 1.0;
            embed[i] = (s + r + t) / 3.0;
        }
        
        // Normalize
        let norm: f32 = embed.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 1e-8 {
            for x in &mut embed {
                *x /= norm;
            }
        }
        
        embed
    }
    
    fn hash_string(&self, s: &str) -> u64 {
        let mut hash = 5381u64;
        for c in s.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(c as u64);
        }
        hash
    }
    
    /// Compute transitive closure using ladder index
    /// A→B (rank 1), B→C (rank 2) implies A→C (rank 3)
    fn compute_transitive_closure(&mut self) {
        let mut new_relations = Vec::new();
        let mut next_rank = self.relations.iter()
            .map(|r| r.ladder_rank)
            .max()
            .unwrap_or(0)
            .saturating_add(1);
        
        // Iterate until no new relations found or max depth reached
        for _depth in 0..self.max_chain_depth {
            let mut found_new = false;
            
            // For each pair of relations, check if they can be chained
            for r1 in &self.relations {
                for r2 in &self.relations {
                    // Chain: r1.target == r2.source AND same relation type
                    if r1.target == r2.source && r1.relation == r2.relation {
                        // Check if this transitive relation already exists
                        let exists = self.relations.iter().chain(new_relations.iter())
                            .any(|r| r.source == r1.source && r.target == r2.target && r.relation == r1.relation);
                        
                        if !exists {
                            // Confidence decays through chain
                            let new_conf = r1.confidence * r2.confidence * self.confidence_decay;
                            
                            let embedding = self.create_triple_embedding(&r1.source, &r1.relation, &r2.target);
                            
                            new_relations.push(TransitiveRelation {
                                source: r1.source.clone(),
                                relation: r1.relation.clone(),
                                target: r2.target.clone(),
                                ladder_rank: next_rank,
                                confidence: new_conf,
                                embedding,
                            });
                            
                            next_rank = next_rank.saturating_add(1);
                            found_new = true;
                        }
                    }
                    
                    // Handle inverse relations
                    // left_of inverse is right_of
                    // bigger_than inverse is smaller_than
                    if r1.target == r2.source {
                        let inverse = self.get_inverse_relation(&r1.relation);
                        if let Some(inv) = inverse {
                            if r2.relation == inv {
                                // A left_of B, B right_of C => A left_of C (if B == B)
                                // This is actually just chaining with inverse awareness
                            }
                        }
                    }
                }
            }
            
            if !found_new {
                break;
            }
        }
        
        self.relations.extend(new_relations);
    }
    
    /// Get inverse relation type
    fn get_inverse_relation(&self, relation: &str) -> Option<&'static str> {
        match relation {
            "left_of" => Some("right_of"),
            "right_of" => Some("left_of"),
            "above" => Some("below"),
            "below" => Some("above"),
            "bigger_than" => Some("smaller_than"),
            "smaller_than" => Some("bigger_than"),
            "fits_inside" => None, // Not symmetric
            _ => None,
        }
    }
    
    /// Query if a relation holds between two entities
    /// Returns (holds: bool, confidence: f32)
    pub fn query_relation(&self, source: &str, relation: &str, target: &str) -> (bool, f32) {
        let source_lower = source.to_lowercase();
        let target_lower = target.to_lowercase();
        let relation_lower = relation.to_lowercase();
        
        // Direct lookup
        for r in &self.relations {
            if r.source == source_lower && r.target == target_lower {
                if r.relation == relation_lower {
                    return (true, r.confidence);
                }
                // Check if asking about inverse
                if let Some(inv) = self.get_inverse_relation(&r.relation) {
                    if inv == relation_lower {
                        return (false, r.confidence); // Inverse holds, so original doesn't
                    }
                }
            }
        }
        
        // Check inverse direction
        for r in &self.relations {
            if r.source == target_lower && r.target == source_lower {
                if let Some(inv) = self.get_inverse_relation(&r.relation) {
                    if inv == relation_lower {
                        return (true, r.confidence);
                    }
                }
                if r.relation == relation_lower {
                    return (false, r.confidence); // Reverse direction exists
                }
            }
        }
        
        // No relation found - check if we have ANY info about these entities
        let has_source = self.relations.iter().any(|r| r.source == source_lower || r.target == source_lower);
        let has_target = self.relations.iter().any(|r| r.source == target_lower || r.target == target_lower);
        
        if has_source && has_target {
            // We know about both entities but no relation - likely "no"
            (false, 0.5)
        } else {
            // Unknown entities - uncertain
            (false, 0.0)
        }
    }
    
    /// Score a yes/no answer based on transitive reasoning
    /// Returns score boost for the correct answer
    pub fn score_yes_no(&self, context: &str, question: &str, answer: &str) -> f32 {
        // Extract the relation being asked about
        let question_lower = question.to_lowercase();
        let answer_lower = answer.to_lowercase();
        
        // Parse question to extract source, relation, target
        // Pattern: "Is X <relation> Y?"
        let mut source = String::new();
        let mut target = String::new();
        let mut relation = String::new();
        
        // Try to parse spatial question
        let spatial_patterns = [
            ("to the left of", "left_of"),
            ("left of", "left_of"),
            ("to the right of", "right_of"),
            ("right of", "right_of"),
            ("above", "above"),
            ("below", "below"),
        ];
        
        let size_patterns = [
            ("bigger than", "bigger_than"),
            ("larger than", "bigger_than"),
            ("smaller than", "smaller_than"),
            ("fit in", "fits_inside"),
            ("fits in", "fits_inside"),
        ];
        
        for (pattern, rel_type) in spatial_patterns.iter().chain(size_patterns.iter()) {
            if question_lower.contains(pattern) {
                relation = rel_type.to_string();
                
                // Extract source (after "is the" or "is")
                if let Some(is_pos) = question_lower.find("is the ") {
                    let after_is = &question_lower[is_pos + 7..];
                    if let Some(pat_pos) = after_is.find(pattern) {
                        source = after_is[..pat_pos].trim().to_string();
                    }
                } else if let Some(is_pos) = question_lower.find("is ") {
                    let after_is = &question_lower[is_pos + 3..];
                    if let Some(pat_pos) = after_is.find(pattern) {
                        source = after_is[..pat_pos].trim().to_string();
                    }
                }
                
                // Extract target (after pattern, before ?)
                if let Some(pat_pos) = question_lower.find(pattern) {
                    let after_pat = &question_lower[pat_pos + pattern.len()..];
                    target = after_pat.trim()
                        .trim_end_matches('?')
                        .trim()
                        .trim_start_matches("the ")
                        .to_string();
                }
                
                break;
            }
        }
        
        // Only apply transitive reasoning if we found a valid spatial/size question
        if source.is_empty() || target.is_empty() || relation.is_empty() {
            return 0.0;
        }
        
        // Query the transitive graph
        let (holds, confidence) = self.query_relation(&source, &relation, &target);
        
        // Only apply scoring if we have actual relations in our graph
        if self.relations.is_empty() {
            return 0.0;
        }
        
        // Score based on whether answer matches
        let answer_is_yes = answer_lower == "yes";
        let answer_is_no = answer_lower == "no";
        let answer_is_uncertain = answer_lower == "maybe" || answer_lower == "unknown";
        
        // If we have transitive evidence, penalize uncertain answers (but less aggressively)
        if answer_is_uncertain && confidence > 0.5 {
            return -confidence * 25.0; // Moderate penalty for "maybe"/"unknown" when we have strong evidence
        }
        
        if holds == answer_is_yes {
            // Correct answer - boost score
            confidence * 40.0
        } else if !holds && answer_is_no {
            // Correct "no" answer - boost score
            confidence * 40.0
        } else if confidence > 0.5 {
            // Wrong answer with high confidence - penalize
            -confidence * 20.0
        } else {
            // Uncertain - no boost
            0.0
        }
    }
    
    /// Get all relations as embeddings for vector similarity search
    pub fn get_relation_embeddings(&self) -> Vec<(String, Vec<f32>)> {
        self.relations.iter()
            .map(|r| {
                let key = format!("{}:{}:{}", r.source, r.relation, r.target);
                (key, r.embedding.clone())
            })
            .collect()
    }
    
    /// Find similar relations using cosine similarity
    pub fn find_similar_relations(&self, query_embed: &[f32], top_k: usize) -> Vec<(&TransitiveRelation, f32)> {
        let mut scored: Vec<_> = self.relations.iter()
            .map(|r| {
                let sim = cosine_similarity(query_embed, &r.embedding);
                (r, sim)
            })
            .collect();
        
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);
        scored
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let n = a.len().min(b.len());
    if n == 0 { return 0.0; }
    
    let dot: f32 = a.iter().zip(b.iter()).take(n).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().take(n).map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().take(n).map(|x| x * x).sum::<f32>().sqrt();
    
    if norm_a < 1e-8 || norm_b < 1e-8 { return 0.0; }
    dot / (norm_a * norm_b)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_spatial_transitivity() {
        let mut reasoner = TransitiveFluxReasoner::new(64);
        
        let context = "The pink rectangle is to the left of the triangle. The triangle is to the left of the red square.";
        reasoner.extract_relations(context);
        
        // Direct relations
        let (holds, conf) = reasoner.query_relation("pink rectangle", "left_of", "triangle");
        assert!(holds);
        assert!(conf > 0.9);
        
        // Transitive relation: pink rectangle left_of red square
        let (holds, conf) = reasoner.query_relation("pink rectangle", "left_of", "red square");
        assert!(holds);
        assert!(conf > 0.5); // Lower confidence due to transitivity
        
        // Inverse: pink rectangle is NOT right_of red square
        let (holds, _) = reasoner.query_relation("pink rectangle", "right_of", "red square");
        assert!(!holds);
    }
    
    #[test]
    fn test_size_transitivity() {
        let mut reasoner = TransitiveFluxReasoner::new(64);
        
        let context = "The container is bigger than the box. The box is bigger than the chocolate.";
        reasoner.extract_relations(context);
        
        // Transitive: container bigger_than chocolate
        let (holds, conf) = reasoner.query_relation("container", "bigger_than", "chocolate");
        assert!(holds);
        assert!(conf > 0.5);
    }
    
    #[test]
    fn test_yes_no_scoring() {
        let mut reasoner = TransitiveFluxReasoner::new(64);
        
        let context = "The pink rectangle is to the left of the triangle. The triangle is to the left of the red square.";
        reasoner.extract_relations(context);
        
        let question = "Is the pink rectangle to the left of the red square?";
        
        let yes_score = reasoner.score_yes_no(context, question, "yes");
        let no_score = reasoner.score_yes_no(context, question, "no");
        
        assert!(yes_score > no_score);
    }
}
