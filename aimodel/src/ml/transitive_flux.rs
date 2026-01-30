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
//! - **Graph Traversal**: BFS/DFS path finding for bAbI Task 19
//! - **Sequential Counting**: Linear O(n) counting mode for bAbI Task 7
//! - **Calibration**: Confidence adjustment based on evidence strength
//! - **Context Extraction**: Federated pathway context for generalization

use std::collections::{HashMap, HashSet, VecDeque};

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

/// Counting mode for flux matrix operations
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CountingMode {
    /// Exponential mode (default): 2^n scaling for vortex positions
    Exponential,
    /// Sequential/Linear mode: O(n) for counting tasks (bAbI Task 7)
    Sequential,
}

/// Path in the graph for traversal results
#[derive(Debug, Clone)]
pub struct GraphPath {
    /// Sequence of nodes in the path
    pub nodes: Vec<String>,
    /// Total confidence (product of edge confidences)
    pub confidence: f32,
    /// Path length (number of hops)
    pub length: usize,
    /// Relation types used in path
    pub relations: Vec<String>,
}

/// Context extraction result for federated learning
#[derive(Debug, Clone)]
pub struct ExtractedContext {
    /// Entities found in context
    pub entities: Vec<String>,
    /// Entity counts (for counting tasks)
    pub entity_counts: HashMap<String, i64>,
    /// Relations extracted
    pub relations: Vec<(String, String, String)>,
    /// Generalized patterns
    pub patterns: Vec<String>,
    /// Specifics (concrete instances)
    pub specifics: Vec<String>,
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
    /// Counting mode (exponential vs sequential)
    counting_mode: CountingMode,
    /// Adjacency list for graph traversal
    pub adjacency: HashMap<String, Vec<(String, String, f32)>>, // node -> [(target, relation, weight)]
    /// Calibration factor (adjusts confidence based on evidence)
    calibration_factor: f32,
    /// Entity counts for counting tasks
    entity_counts: HashMap<String, i64>,
    /// Location tracking for path finding
    locations: HashMap<String, String>, // entity -> current_location
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
            counting_mode: CountingMode::Exponential,
            adjacency: HashMap::new(),
            calibration_factor: 1.0,
            entity_counts: HashMap::new(),
            locations: HashMap::new(),
        }
    }
    
    /// Set counting mode (exponential vs sequential)
    pub fn set_counting_mode(&mut self, mode: CountingMode) {
        self.counting_mode = mode;
    }
    
    /// Clear all state
    pub fn clear(&mut self) {
        self.relations.clear();
        self.entity_embeddings.clear();
        self.relation_embeddings.clear();
        self.adjacency.clear();
        self.entity_counts.clear();
        self.locations.clear();
        self.calibration_factor = 1.0;
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
    
    // =========================================================================
    // GRAPH TRAVERSAL FOR PATH FINDING (bAbI Task 19)
    // =========================================================================
    
    /// Extract location/movement relations for path finding
    /// Parses patterns like "John went to the kitchen", "Mary is in the garden"
    pub fn extract_locations(&mut self, context: &str) {
        self.locations.clear();
        self.adjacency.clear();
        let context_lower = context.to_lowercase();
        
        // Movement patterns (bAbI Task 19)
        let movement_patterns = [
            ("went to the", "moved_to"),
            ("went to", "moved_to"),
            ("moved to the", "moved_to"),
            ("moved to", "moved_to"),
            ("travelled to the", "moved_to"),
            ("travelled to", "moved_to"),
            ("journeyed to the", "moved_to"),
            ("journeyed to", "moved_to"),
            ("is in the", "is_at"),
            ("is in", "is_at"),
            ("is at the", "is_at"),
            ("is at", "is_at"),
        ];
        
        // Connection patterns (for graph edges)
        let connection_patterns = [
            ("is connected to the", "connected_to"),
            ("is connected to", "connected_to"),
            ("leads to the", "connected_to"),
            ("leads to", "connected_to"),
            ("is north of the", "north_of"),
            ("is north of", "north_of"),
            ("is south of the", "south_of"),
            ("is south of", "south_of"),
            ("is east of the", "east_of"),
            ("is east of", "east_of"),
            ("is west of the", "west_of"),
            ("is west of", "west_of"),
        ];
        
        for sentence in context_lower.split(|c| c == '.' || c == '\n') {
            let sentence = sentence.trim();
            if sentence.is_empty() { continue; }
            
            // Parse movement patterns
            for (pattern, rel_type) in &movement_patterns {
                if let Some((entity, location)) = self.parse_relation_pattern(sentence, pattern) {
                    // Update entity's current location
                    self.locations.insert(entity.clone(), location.clone());
                    
                    // Add to adjacency if it's a movement (creates implicit path)
                    if *rel_type == "moved_to" {
                        self.adjacency.entry(entity.clone())
                            .or_insert_with(Vec::new)
                            .push((location.clone(), rel_type.to_string(), 1.0));
                    }
                }
            }
            
            // Parse connection patterns (bidirectional graph edges)
            // "A is west of B" means: A is located west of B
            // So to go FROM B TO A, you go west
            // Edge: B -> A with direction "west_of"
            for (pattern, rel_type) in &connection_patterns {
                if let Some((source, target)) = self.parse_relation_pattern(sentence, pattern) {
                    // "source is west_of target" means: from target, go west to reach source
                    // So edge is: target -> source with rel_type
                    self.adjacency.entry(target.clone())
                        .or_insert_with(Vec::new)
                        .push((source.clone(), rel_type.to_string(), 1.0));
                    
                    // Add reverse edge for bidirectional connections
                    if *rel_type == "connected_to" {
                        self.adjacency.entry(source.clone())
                            .or_insert_with(Vec::new)
                            .push((target.clone(), rel_type.to_string(), 1.0));
                    }
                    
                    // Add inverse relations for directional connections
                    // From source, go opposite direction to reach target
                    let inverse = match *rel_type {
                        "north_of" => Some("south_of"),
                        "south_of" => Some("north_of"),
                        "east_of" => Some("west_of"),
                        "west_of" => Some("east_of"),
                        _ => None,
                    };
                    if let Some(inv) = inverse {
                        self.adjacency.entry(source)
                            .or_insert_with(Vec::new)
                            .push((target, inv.to_string(), 1.0));
                    }
                }
            }
        }
    }
    
    /// Find path between two locations using BFS (shortest path)
    /// Returns the path and confidence
    pub fn find_path(&self, start: &str, end: &str) -> Option<GraphPath> {
        let start_lower = start.to_lowercase();
        let end_lower = end.to_lowercase();
        
        if start_lower == end_lower {
            return Some(GraphPath {
                nodes: vec![start_lower],
                confidence: 1.0,
                length: 0,
                relations: vec![],
            });
        }
        
        // BFS for shortest path
        let mut visited: HashSet<String> = HashSet::new();
        let mut queue: VecDeque<(String, Vec<String>, Vec<String>, f32)> = VecDeque::new();
        
        queue.push_back((start_lower.clone(), vec![start_lower.clone()], vec![], 1.0));
        visited.insert(start_lower);
        
        while let Some((current, path, relations, confidence)) = queue.pop_front() {
            if let Some(neighbors) = self.adjacency.get(&current) {
                for (neighbor, rel_type, weight) in neighbors {
                    if neighbor == &end_lower {
                        // Found path!
                        let mut final_path = path.clone();
                        final_path.push(neighbor.clone());
                        let mut final_rels = relations.clone();
                        final_rels.push(rel_type.clone());
                        
                        return Some(GraphPath {
                            nodes: final_path,
                            confidence: confidence * weight * self.confidence_decay,
                            length: path.len(),
                            relations: final_rels,
                        });
                    }
                    
                    if !visited.contains(neighbor) && path.len() < self.max_chain_depth {
                        visited.insert(neighbor.clone());
                        let mut new_path = path.clone();
                        new_path.push(neighbor.clone());
                        let mut new_rels = relations.clone();
                        new_rels.push(rel_type.clone());
                        queue.push_back((
                            neighbor.clone(),
                            new_path,
                            new_rels,
                            confidence * weight * self.confidence_decay,
                        ));
                    }
                }
            }
        }
        
        None // No path found
    }
    
    /// Find all paths between two locations (DFS)
    pub fn find_all_paths(&self, start: &str, end: &str, max_paths: usize) -> Vec<GraphPath> {
        let start_lower = start.to_lowercase();
        let end_lower = end.to_lowercase();
        
        let mut paths = Vec::new();
        let mut visited = HashSet::new();
        
        self.dfs_paths(
            &start_lower,
            &end_lower,
            &mut visited,
            vec![start_lower.clone()],
            vec![],
            1.0,
            &mut paths,
            max_paths,
        );
        
        // Sort by confidence (descending)
        paths.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
        paths
    }
    
    fn dfs_paths(
        &self,
        current: &str,
        end: &str,
        visited: &mut HashSet<String>,
        path: Vec<String>,
        relations: Vec<String>,
        confidence: f32,
        paths: &mut Vec<GraphPath>,
        max_paths: usize,
    ) {
        if paths.len() >= max_paths || path.len() > self.max_chain_depth {
            return;
        }
        
        if current == end {
            paths.push(GraphPath {
                nodes: path,
                confidence,
                length: relations.len(),
                relations,
            });
            return;
        }
        
        visited.insert(current.to_string());
        
        if let Some(neighbors) = self.adjacency.get(current) {
            for (neighbor, rel_type, weight) in neighbors {
                if !visited.contains(neighbor) {
                    let mut new_path = path.clone();
                    new_path.push(neighbor.clone());
                    let mut new_rels = relations.clone();
                    new_rels.push(rel_type.clone());
                    
                    self.dfs_paths(
                        neighbor,
                        end,
                        visited,
                        new_path,
                        new_rels,
                        confidence * weight * self.confidence_decay,
                        paths,
                        max_paths,
                    );
                }
            }
        }
        
        visited.remove(current);
    }
    
    /// Get entity's current location
    pub fn get_location(&self, entity: &str) -> Option<&String> {
        self.locations.get(&entity.to_lowercase())
    }
    
    /// Answer path-finding questions (bAbI Task 19)
    pub fn answer_path_question(&self, question: &str) -> Option<(String, f32)> {
        let question_lower = question.to_lowercase();
        
        // Pattern: "How do you go from X to Y?"
        if let Some(from_pos) = question_lower.find("from ") {
            if let Some(to_pos) = question_lower.find(" to ") {
                let start = question_lower[from_pos + 5..to_pos]
                    .trim()
                    .trim_start_matches("the ")
                    .to_string();
                let end = question_lower[to_pos + 4..]
                    .trim()
                    .trim_end_matches('?')
                    .trim_start_matches("the ")
                    .to_string();
                
                if let Some(path) = self.find_path(&start, &end) {
                    // Format path as bAbI 19 answer (e.g., "s,s" for south,south)
                    let directions: Vec<&str> = path.relations.iter()
                        .map(|r| match r.as_str() {
                            "north_of" => "n",
                            "south_of" => "s",
                            "east_of" => "e",
                            "west_of" => "w",
                            "connected_to" => "c",
                            _ => "?",
                        })
                        .collect();
                    let answer = directions.join(",");
                    return Some((answer, path.confidence));
                }
            }
        }
        
        // Pattern: "Where is X?"
        if question_lower.contains("where is ") {
            if let Some(pos) = question_lower.find("where is ") {
                let entity = question_lower[pos + 9..]
                    .trim()
                    .trim_end_matches('?')
                    .trim_start_matches("the ")
                    .to_string();
                
                if let Some(location) = self.get_location(&entity) {
                    return Some((location.clone(), 1.0));
                }
            }
        }
        
        None
    }
    
    // =========================================================================
    // SEQUENTIAL COUNTING MODE (bAbI Task 7)
    // =========================================================================
    
    /// Extract entity counts from context (linear O(n) counting)
    /// Parses patterns like "Daniel picked up the apple", "Daniel dropped the apple"
    pub fn extract_counts(&mut self, context: &str) {
        self.entity_counts.clear();
        let context_lower = context.to_lowercase();
        
        // Acquisition patterns (increment count)
        let acquire_patterns = [
            "picked up",
            "got",
            "grabbed",
            "took",
            "received",
        ];
        
        // Release patterns (decrement count)
        let release_patterns = [
            "dropped",
            "put down",
            "discarded",
            "gave",
            "left",
        ];
        
        // Track items per entity
        let mut entity_items: HashMap<String, HashSet<String>> = HashMap::new();
        
        for sentence in context_lower.split(|c| c == '.' || c == '\n') {
            let sentence = sentence.trim();
            if sentence.is_empty() { continue; }
            
            // Find entity (usually first word or after "the")
            let words: Vec<&str> = sentence.split_whitespace().collect();
            if words.is_empty() { continue; }
            
            let entity = words[0].trim_start_matches("the ").to_string();
            
            // Check for acquisition
            for pattern in &acquire_patterns {
                if sentence.contains(pattern) {
                    // Extract item (usually after the pattern)
                    if let Some(pos) = sentence.find(pattern) {
                        let after = &sentence[pos + pattern.len()..];
                        let item = after.trim()
                            .trim_start_matches("the ")
                            .split_whitespace()
                            .next()
                            .unwrap_or("")
                            .trim_end_matches(|c: char| !c.is_alphanumeric())
                            .to_string();
                        
                        if !item.is_empty() {
                            entity_items.entry(entity.clone())
                                .or_insert_with(HashSet::new)
                                .insert(item);
                        }
                    }
                }
            }
            
            // Check for release
            for pattern in &release_patterns {
                if sentence.contains(pattern) {
                    if let Some(pos) = sentence.find(pattern) {
                        let after = &sentence[pos + pattern.len()..];
                        let item = after.trim()
                            .trim_start_matches("the ")
                            .split_whitespace()
                            .next()
                            .unwrap_or("")
                            .trim_end_matches(|c: char| !c.is_alphanumeric())
                            .to_string();
                        
                        if !item.is_empty() {
                            if let Some(items) = entity_items.get_mut(&entity) {
                                items.remove(&item);
                            }
                        }
                    }
                }
            }
        }
        
        // Convert to counts
        for (entity, items) in entity_items {
            self.entity_counts.insert(entity, items.len() as i64);
        }
    }
    
    /// Count items for an entity (sequential linear mode)
    /// Returns count using O(n) sequential traversal, not exponential
    pub fn count_items(&self, entity: &str) -> i64 {
        match self.counting_mode {
            CountingMode::Sequential => {
                // Linear O(n) counting
                self.entity_counts.get(&entity.to_lowercase()).copied().unwrap_or(0)
            }
            CountingMode::Exponential => {
                // Exponential mode - use vortex position scaling (legacy)
                let base_count = self.entity_counts.get(&entity.to_lowercase()).copied().unwrap_or(0);
                // Apply vortex scaling (but cap to prevent overflow)
                base_count.min(9)
            }
        }
    }
    
    /// Answer counting questions (bAbI Task 7)
    pub fn answer_counting_question(&self, question: &str) -> Option<(i64, f32)> {
        let question_lower = question.to_lowercase();
        
        // Pattern: "How many objects is X carrying?"
        if question_lower.contains("how many") {
            // Extract entity
            let patterns = ["is ", "does ", "did "];
            for pattern in &patterns {
                if let Some(pos) = question_lower.find(pattern) {
                    let after = &question_lower[pos + pattern.len()..];
                    let entity = after.split_whitespace()
                        .next()
                        .unwrap_or("")
                        .to_string();
                    
                    if !entity.is_empty() {
                        let count = self.count_items(&entity);
                        // Calibrated confidence based on evidence
                        let confidence = self.calibrate_confidence(count > 0);
                        return Some((count, confidence));
                    }
                }
            }
        }
        
        None
    }
    
    // =========================================================================
    // CALIBRATION (Confidence vs Accuracy)
    // =========================================================================
    
    /// Calibrate confidence based on evidence strength
    /// Addresses the issue of high confidence on wrong answers
    pub fn calibrate_confidence(&self, has_evidence: bool) -> f32 {
        if has_evidence {
            // We have evidence - confidence based on relation count
            let evidence_count = self.relations.len() + self.entity_counts.len();
            let base_conf = match evidence_count {
                0 => 0.3,
                1..=2 => 0.5,
                3..=5 => 0.7,
                6..=10 => 0.85,
                _ => 0.95,
            };
            base_conf * self.calibration_factor
        } else {
            // No evidence - low confidence
            0.25 * self.calibration_factor
        }
    }
    
    /// Update calibration factor based on feedback
    /// Call this after getting correct/incorrect results
    pub fn update_calibration(&mut self, was_correct: bool, predicted_confidence: f32) {
        // Platt scaling-inspired calibration update
        if was_correct {
            // If correct with low confidence, increase calibration
            if predicted_confidence < 0.5 {
                self.calibration_factor *= 1.1;
            }
        } else {
            // If wrong with high confidence, decrease calibration
            if predicted_confidence > 0.7 {
                self.calibration_factor *= 0.9;
            }
        }
        // Clamp to reasonable range
        self.calibration_factor = self.calibration_factor.clamp(0.5, 2.0);
    }
    
    /// Get calibrated confidence for a query
    pub fn get_calibrated_confidence(&self, raw_confidence: f32) -> f32 {
        (raw_confidence * self.calibration_factor).clamp(0.0, 1.0)
    }
    
    // =========================================================================
    // CONTEXT EXTRACTION FOR FEDERATED LEARNING
    // =========================================================================
    
    /// Extract comprehensive context for federated pathway learning
    /// Returns entities, relations, patterns, and specifics for generalization
    pub fn extract_context(&mut self, text: &str) -> ExtractedContext {
        // Extract relations first
        self.extract_relations(text);
        self.extract_locations(text);
        self.extract_counts(text);
        
        let text_lower = text.to_lowercase();
        
        // Extract all entities
        let mut entities: HashSet<String> = HashSet::new();
        for rel in &self.relations {
            entities.insert(rel.source.clone());
            entities.insert(rel.target.clone());
        }
        for (entity, _) in &self.locations {
            entities.insert(entity.clone());
        }
        for (entity, _) in &self.entity_counts {
            entities.insert(entity.clone());
        }
        
        // Extract patterns (generalizations)
        let mut patterns: Vec<String> = Vec::new();
        
        // Relation type patterns
        let mut rel_types: HashSet<String> = HashSet::new();
        for rel in &self.relations {
            rel_types.insert(rel.relation.clone());
        }
        for rel_type in rel_types {
            patterns.push(format!("ENTITY {} ENTITY", rel_type));
        }
        
        // Movement patterns
        if !self.locations.is_empty() {
            patterns.push("ENTITY moved_to LOCATION".to_string());
        }
        
        // Counting patterns
        if !self.entity_counts.is_empty() {
            patterns.push("ENTITY has COUNT items".to_string());
        }
        
        // Extract specifics (concrete instances)
        let mut specifics: Vec<String> = Vec::new();
        for rel in &self.relations {
            specifics.push(format!("{} {} {}", rel.source, rel.relation, rel.target));
        }
        for (entity, location) in &self.locations {
            specifics.push(format!("{} is_at {}", entity, location));
        }
        for (entity, count) in &self.entity_counts {
            specifics.push(format!("{} has {} items", entity, count));
        }
        
        // Build relations list
        let relations: Vec<(String, String, String)> = self.relations.iter()
            .map(|r| (r.source.clone(), r.relation.clone(), r.target.clone()))
            .collect();
        
        ExtractedContext {
            entities: entities.into_iter().collect(),
            entity_counts: self.entity_counts.clone(),
            relations,
            patterns,
            specifics,
        }
    }
    
    /// Merge context from multiple sources (federated learning)
    pub fn merge_contexts(&self, contexts: &[ExtractedContext]) -> ExtractedContext {
        let mut merged_entities: HashSet<String> = HashSet::new();
        let mut merged_counts: HashMap<String, i64> = HashMap::new();
        let mut merged_relations: Vec<(String, String, String)> = Vec::new();
        let mut merged_patterns: HashSet<String> = HashSet::new();
        let mut merged_specifics: Vec<String> = Vec::new();
        
        for ctx in contexts {
            for entity in &ctx.entities {
                merged_entities.insert(entity.clone());
            }
            for (entity, count) in &ctx.entity_counts {
                *merged_counts.entry(entity.clone()).or_insert(0) += count;
            }
            merged_relations.extend(ctx.relations.clone());
            for pattern in &ctx.patterns {
                merged_patterns.insert(pattern.clone());
            }
            merged_specifics.extend(ctx.specifics.clone());
        }
        
        ExtractedContext {
            entities: merged_entities.into_iter().collect(),
            entity_counts: merged_counts,
            relations: merged_relations,
            patterns: merged_patterns.into_iter().collect(),
            specifics: merged_specifics,
        }
    }
    
    /// Score answer using all reasoning modes
    pub fn score_answer_comprehensive(
        &mut self,
        context: &str,
        question: &str,
        answer: &str,
    ) -> f32 {
        let mut total_score = 0.0f32;
        
        // 1. Transitive reasoning (spatial/size)
        let transitive_score = self.score_yes_no(context, question, answer);
        total_score += transitive_score;
        
        // 2. Path finding
        if let Some((path_answer, confidence)) = self.answer_path_question(question) {
            let answer_lower = answer.to_lowercase();
            if answer_lower.contains(&path_answer) || path_answer.contains(&answer_lower) {
                total_score += confidence * 30.0;
            }
        }
        
        // 3. Counting
        if let Some((count, confidence)) = self.answer_counting_question(question) {
            let answer_lower = answer.to_lowercase();
            let count_str = count.to_string();
            let count_words = ["zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine"];
            
            let matches = answer_lower == count_str 
                || (count < 10 && answer_lower == count_words[count as usize])
                || answer_lower.contains(&count_str);
            
            if matches {
                total_score += confidence * 35.0;
            }
        }
        
        // Apply calibration
        self.get_calibrated_confidence(total_score / 100.0) * 100.0
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
    
    #[test]
    fn test_path_finding() {
        let mut reasoner = TransitiveFluxReasoner::new(64);
        
        let context = "The kitchen is connected to the hallway. The hallway is connected to the garden. The garden is connected to the bedroom.";
        reasoner.extract_locations(context);
        
        // Find path from kitchen to bedroom
        let path = reasoner.find_path("kitchen", "bedroom");
        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path.length, 3); // kitchen -> hallway -> garden -> bedroom
        assert!(path.confidence > 0.5);
    }
    
    #[test]
    fn test_babi19_directional_path() {
        let mut reasoner = TransitiveFluxReasoner::new(64);
        
        // bAbI 19 format: "The garden is west of the bathroom"
        // means: garden is located west of bathroom
        // so from bathroom, go west to reach garden
        let context = "The garden is west of the bathroom.\nThe bedroom is north of the hallway.\nThe office is south of the hallway.\nThe bathroom is north of the bedroom.\nThe kitchen is east of the bedroom.";
        reasoner.extract_locations(context);
        
        // Question: How do you go from the bathroom to the hallway?
        // bathroom -> bedroom (south) -> hallway (south) = s,s
        let path = reasoner.find_path("bathroom", "hallway");
        assert!(path.is_some(), "Should find path from bathroom to hallway");
        let path = path.unwrap();
        
        // Check the answer format matches bAbI 19
        let answer = reasoner.answer_path_question("How do you go from the bathroom to the hallway?");
        assert!(answer.is_some(), "Should answer path question");
        let (directions, _conf) = answer.unwrap();
        
        // Should be direction abbreviations like "s,s"
        assert!(directions.contains(',') || directions.len() == 1, 
            "Answer should be comma-separated directions, got: {}", directions);
        assert!(directions.chars().all(|c| c == 'n' || c == 's' || c == 'e' || c == 'w' || c == ','),
            "Answer should only contain n,s,e,w and commas, got: {}", directions);
    }
    
    #[test]
    fn test_babi19_full_format() {
        // Test with exact bAbI format: context + question in one string
        let mut reasoner = TransitiveFluxReasoner::new(64);
        
        // This is how bAbI questions are formatted in the evaluator
        let full_question = "The garden is west of the bathroom.
The bedroom is north of the hallway.
The office is south of the hallway.
The bathroom is north of the bedroom.
The kitchen is east of the bedroom.
How do you go from the bathroom to the hallway?";
        
        // Extract locations from the full text
        reasoner.extract_locations(full_question);
        
        // Check adjacency was built
        assert!(!reasoner.adjacency.is_empty(), "Adjacency should not be empty after extraction");
        
        // Answer the path question
        let answer = reasoner.answer_path_question(full_question);
        assert!(answer.is_some(), "Should answer path question from full text");
        let (directions, _conf) = answer.unwrap();
        
        // Should be s,s (bathroom -> bedroom (south) -> hallway (south))
        assert_eq!(directions, "s,s", "Expected s,s but got {}", directions);
    }
    
    #[test]
    fn test_counting_sequential() {
        let mut reasoner = TransitiveFluxReasoner::new(64);
        reasoner.set_counting_mode(CountingMode::Sequential);
        
        let context = "Daniel picked up the apple. Daniel picked up the football. Daniel dropped the apple.";
        reasoner.extract_counts(context);
        
        // Daniel should have 1 item (football)
        let count = reasoner.count_items("daniel");
        assert_eq!(count, 1);
    }
    
    #[test]
    fn test_calibration() {
        let mut reasoner = TransitiveFluxReasoner::new(64);
        
        // Initial calibration factor should be 1.0
        assert!((reasoner.calibration_factor - 1.0).abs() < 0.01);
        
        // Wrong answer with high confidence should decrease calibration
        reasoner.update_calibration(false, 0.9);
        assert!(reasoner.calibration_factor < 1.0);
        
        // Correct answer with low confidence should increase calibration
        reasoner.update_calibration(true, 0.3);
        assert!(reasoner.calibration_factor > 0.9 * 0.9); // Should have increased
    }
    
    #[test]
    fn test_context_extraction() {
        let mut reasoner = TransitiveFluxReasoner::new(64);
        
        let context = "The box is bigger than the ball. John went to the kitchen. Mary picked up the apple.";
        let extracted = reasoner.extract_context(context);
        
        // Should have entities
        assert!(!extracted.entities.is_empty());
        
        // Should have patterns
        assert!(!extracted.patterns.is_empty());
        
        // Should have specifics
        assert!(!extracted.specifics.is_empty());
    }
}
