//! Recursive Chain System with SNOAT Macros
//!
//! Subject-Node-Object-Attribute-Trait (SNOAT) macro system for infinite
//! multi-hop reasoning chains. Integrates with exhaustive pathway search
//! to find optimal reasoning paths up to depth 9 (sacred number).
//!
//! ## Architecture
//! ```text
//! SNOAT Chain: S₁ --[rel₁]--> O₁/S₂ --[rel₂]--> O₂/S₃ --...--> Oₙ
//!              │              │                │
//!              N₁ (attrs)     N₂ (attrs)       Nₙ (attrs)
//!              │              │                │
//!              T₁ (traits)    T₂ (traits)      Tₙ (traits)
//!
//! Pathway Search: Find optimal permutation through SNOAT graph
//! Chain Score = Π(confidence_i * 0.9^depth) * pathway_score
//! ```
//!
//! ## Key Features
//! - **SNOAT Macros**: Define Subject-Node-Object-Attribute-Trait chains
//! - **Reference Chains**: Objects become Subjects in next hop
//! - **Pathway Integration**: n! search over chain graph
//! - **Depth 9 Maximum**: Sacred number limit with confidence decay
//! - **Cycle Detection**: Prevents infinite reference loops

use crate::ml::pathway::{ExhaustivePathwayOptimizer, PathwayConfig, ScoredPathway};
use crate::ml::flux_compression_sota::{FluxCompression24, RelationType, EntityType};
use std::collections::{HashMap, HashSet, VecDeque};

// =============================================================================
// SNOAT CORE TYPES
// =============================================================================

/// Subject-Node-Object-Attribute-Trait unit
#[derive(Debug, Clone)]
pub struct SNOATNode {
    /// Unique node ID
    pub id: usize,
    /// Subject (WHO)
    pub subject: String,
    /// Relation type
    pub relation: RelationType,
    /// Object (WHAT/WHO target)
    pub object: String,
    /// Attributes (properties of this relation)
    pub attributes: HashMap<String, f32>,
    /// Traits (behavioral patterns)
    pub traits: Vec<String>,
    /// Confidence in this node
    pub confidence: f32,
    /// Depth in chain (0 = root)
    pub depth: usize,
    /// Reference to next node (Object becomes Subject)
    pub next_ref: Option<usize>,
    /// References from previous nodes
    pub prev_refs: Vec<usize>,
    /// Embedding for pathway search
    pub embedding: Vec<f32>,
    /// Compressed 24-byte representation
    pub compressed: FluxCompression24,
}

impl SNOATNode {
    pub fn new(id: usize, subject: &str, relation: RelationType, object: &str) -> Self {
        let mut node = Self {
            id,
            subject: subject.to_lowercase(),
            relation,
            object: object.to_lowercase(),
            attributes: HashMap::new(),
            traits: Vec::new(),
            confidence: 1.0,
            depth: 0,
            next_ref: None,
            prev_refs: Vec::new(),
            embedding: Vec::new(),
            compressed: FluxCompression24::default(),
        };
        node.compute_embedding(256);
        node
    }
    
    /// Add attribute to node
    pub fn with_attr(mut self, key: &str, value: f32) -> Self {
        self.attributes.insert(key.to_string(), value);
        self
    }
    
    /// Add trait to node
    pub fn with_trait(mut self, trait_name: &str) -> Self {
        self.traits.push(trait_name.to_string());
        self
    }
    
    /// Compute embedding from SNOAT content
    pub fn compute_embedding(&mut self, dim: usize) {
        let content = format!("{}_{}_{}", self.subject, self.relation as u8, self.object);
        let hash = content.bytes().fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
        
        self.embedding = (0..dim)
            .map(|i| {
                let seed = hash.wrapping_add(i as u64);
                ((seed as f32 * 0.0001).sin() + (seed as f32 * 0.00003).cos()) * 0.5
            })
            .collect();
    }
    
    /// Pack into 24-byte compressed format
    pub fn pack_compressed(&mut self) {
        self.compressed.pack_who(&self.subject, EntityType::Thing);
        // pack_what expects ActionType, use pack_who for object as entity
        self.compressed.inference[0] = self.depth as u8;
        self.compressed.confidence[0] = (self.confidence * 255.0) as u8;
        self.compressed.relation[0] = self.relation as u8;
    }
}

// =============================================================================
// SNOAT CHAIN (Linked nodes forming reasoning path)
// =============================================================================

/// A chain of SNOAT nodes representing a multi-hop reasoning path
#[derive(Debug, Clone)]
pub struct SNOATChain {
    /// All nodes in the chain
    pub nodes: Vec<SNOATNode>,
    /// Root node IDs (entry points)
    pub roots: Vec<usize>,
    /// Total chain confidence (product of node confidences with decay)
    pub confidence: f32,
    /// Maximum depth reached
    pub max_depth: usize,
    /// Subject -> Node ID index
    subject_index: HashMap<String, Vec<usize>>,
    /// Object -> Node ID index  
    object_index: HashMap<String, Vec<usize>>,
}

impl SNOATChain {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            roots: Vec::new(),
            confidence: 1.0,
            max_depth: 0,
            subject_index: HashMap::new(),
            object_index: HashMap::new(),
        }
    }
    
    /// Add a node to the chain
    pub fn add_node(&mut self, mut node: SNOATNode) -> usize {
        let id = self.nodes.len();
        node.id = id;
        
        // Index by subject and object
        self.subject_index
            .entry(node.subject.clone())
            .or_default()
            .push(id);
        self.object_index
            .entry(node.object.clone())
            .or_default()
            .push(id);
        
        // Auto-link: if object matches another node's subject, create reference
        if let Some(next_ids) = self.subject_index.get(&node.object) {
            if let Some(&next_id) = next_ids.first() {
                if next_id != id {
                    node.next_ref = Some(next_id);
                    node.depth = 0; // Will be computed later
                }
            }
        }
        
        // Check if this node's subject is another node's object (back-link)
        if let Some(prev_ids) = self.object_index.get(&node.subject) {
            for &prev_id in prev_ids {
                if prev_id != id && prev_id < self.nodes.len() {
                    node.prev_refs.push(prev_id);
                    // Update previous node's next_ref
                    self.nodes[prev_id].next_ref = Some(id);
                }
            }
        }
        
        // If no previous refs, this is a root
        if node.prev_refs.is_empty() {
            self.roots.push(id);
        }
        
        self.nodes.push(node);
        self.recompute_depths();
        id
    }
    
    /// Recompute depths from roots using BFS
    fn recompute_depths(&mut self) {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        
        // Start from roots
        for &root_id in &self.roots {
            queue.push_back((root_id, 0usize));
        }
        
        while let Some((node_id, depth)) = queue.pop_front() {
            if visited.contains(&node_id) || node_id >= self.nodes.len() {
                continue;
            }
            visited.insert(node_id);
            
            self.nodes[node_id].depth = depth;
            self.nodes[node_id].confidence = 0.9_f32.powi(depth as i32);
            self.max_depth = self.max_depth.max(depth);
            
            // Follow next_ref
            if let Some(next_id) = self.nodes[node_id].next_ref {
                if depth < 9 { // Sacred limit
                    queue.push_back((next_id, depth + 1));
                }
            }
        }
        
        // Compute total chain confidence
        self.confidence = self.nodes.iter()
            .map(|n| n.confidence)
            .product();
    }
    
    /// Find all paths from a subject to answer a question
    pub fn find_paths(&self, start_subject: &str, max_depth: usize) -> Vec<Vec<usize>> {
        let mut paths = Vec::new();
        let start_subject = start_subject.to_lowercase();
        
        if let Some(start_ids) = self.subject_index.get(&start_subject) {
            for &start_id in start_ids {
                let mut current_path = vec![start_id];
                self.dfs_paths(start_id, &mut current_path, &mut paths, max_depth, &mut HashSet::new());
            }
        }
        
        paths
    }
    
    fn dfs_paths(
        &self,
        node_id: usize,
        current_path: &mut Vec<usize>,
        all_paths: &mut Vec<Vec<usize>>,
        max_depth: usize,
        visited: &mut HashSet<usize>,
    ) {
        if current_path.len() > max_depth || visited.contains(&node_id) {
            return;
        }
        visited.insert(node_id);
        
        // Record current path
        all_paths.push(current_path.clone());
        
        // Follow next_ref
        if let Some(next_id) = self.nodes[node_id].next_ref {
            current_path.push(next_id);
            self.dfs_paths(next_id, current_path, all_paths, max_depth, visited);
            current_path.pop();
        }
        
        visited.remove(&node_id);
    }
    
    /// Answer a question using multi-hop chain traversal
    pub fn answer_question(&self, question: &str) -> Option<(String, f32, Vec<usize>)> {
        let question_lower = question.to_lowercase();
        
        // Try give/transfer questions first (bAbI 5)
        if let Some((answer, conf)) = self.answer_give_question(&question_lower) {
            return Some((answer, conf, vec![]));
        }
        
        // Extract subject from question
        let subject = self.extract_subject(&question_lower)?;
        
        // Find all paths from subject
        let paths = self.find_paths(&subject, 9);
        
        if paths.is_empty() {
            return None;
        }
        
        // Score each path and find best answer
        let mut best_answer: Option<(String, f32, Vec<usize>)> = None;
        
        for path in paths {
            if path.is_empty() {
                continue;
            }
            
            // Get the final node's object as potential answer
            let final_node = &self.nodes[*path.last().unwrap()];
            let answer = final_node.object.clone();
            
            // Compute path confidence (product with decay)
            let confidence: f32 = path.iter()
                .map(|&id| self.nodes[id].confidence)
                .product();
            
            // Check if this is better than current best
            if best_answer.is_none() || confidence > best_answer.as_ref().unwrap().1 {
                best_answer = Some((answer, confidence, path));
            }
        }
        
        best_answer
    }
    
    fn extract_subject(&self, question: &str) -> Option<String> {
        // Pattern: "Where is X?" or "What does X have?"
        let patterns = [
            "where is the ", "where is ", "where's the ", "where's ",
            "what is the ", "what is ", "what does ", "who has the ", "who has "
        ];
        
        for pattern in &patterns {
            if let Some(pos) = question.find(pattern) {
                let after = &question[pos + pattern.len()..];
                // Get all words until ? and take the first meaningful one
                let words: Vec<&str> = after.split(|c: char| c == '?' || c == '.')
                    .next()
                    .unwrap_or("")
                    .split_whitespace()
                    .collect();
                
                for word in words {
                    let cleaned = word.trim_matches(|c: char| !c.is_alphanumeric()).to_string();
                    if !cleaned.is_empty() && !["the", "a", "an"].contains(&cleaned.as_str()) {
                        return Some(cleaned);
                    }
                }
            }
        }
        None
    }
    
    /// Answer "Who did X give Y to?" questions (bAbI 5)
    pub fn answer_give_question(&self, question: &str) -> Option<(String, f32)> {
        let question_lower = question.to_lowercase();
        
        // Pattern: "Who did X give the Y to?"
        if question_lower.contains("who did") && question_lower.contains("give") && question_lower.contains("to") {
            // Extract giver and object
            if let Some(did_pos) = question_lower.find("who did ") {
                let after_did = &question_lower[did_pos + 8..];
                let giver = after_did.split_whitespace().next().unwrap_or("").to_string();
                
                // Extract object (between "give" and "to")
                if let Some(give_pos) = after_did.find("give") {
                    let after_give = &after_did[give_pos + 4..];
                    let after_give = after_give.trim_start_matches(" the ").trim_start();
                    
                    if let Some(to_pos) = after_give.find(" to") {
                        let object = after_give[..to_pos].trim().to_string();
                        
                        // Find who received this object from this giver
                        for node in &self.nodes {
                            if node.attributes.contains_key("received") && node.object == object {
                                return Some((node.subject.clone(), node.confidence));
                            }
                        }
                    }
                }
            }
        }
        
        // Pattern: "What did X give to Y?"
        if question_lower.contains("what did") && question_lower.contains("give to") {
            if let Some(did_pos) = question_lower.find("what did ") {
                let after_did = &question_lower[did_pos + 9..];
                let giver = after_did.split_whitespace().next().unwrap_or("").to_string();
                
                if let Some(to_pos) = after_did.find("give to ") {
                    let recipient = after_did[to_pos + 8..]
                        .split(|c: char| c == '?' || c == '.')
                        .next()
                        .unwrap_or("")
                        .trim()
                        .to_string();
                    
                    // Find what the recipient received
                    for node in &self.nodes {
                        if node.subject == recipient && node.attributes.contains_key("received") {
                            return Some((node.object.clone(), node.confidence));
                        }
                    }
                }
            }
        }
        
        None
    }
}

impl Default for SNOATChain {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// PATHWAY-INTEGRATED CHAIN REASONER
// =============================================================================

/// Reasoner that uses exhaustive pathway search over SNOAT chains
pub struct ChainPathwayReasoner {
    /// SNOAT chain
    pub chain: SNOATChain,
    /// Pathway optimizer
    pub pathway: ExhaustivePathwayOptimizer,
    /// Embedding dimension
    pub embed_dim: usize,
    /// Maximum chain depth (sacred 9)
    pub max_depth: usize,
}

impl ChainPathwayReasoner {
    pub fn new(embed_dim: usize) -> Self {
        let config = PathwayConfig {
            n_nodes: 9, // Sacred number
            dimension: embed_dim,
            num_stacks: 3,
            top_k_per_stack: 10,
            parallel: true,
            initial_beta: 1.0,
            kl_bound: 0.1,
        };
        
        Self {
            chain: SNOATChain::new(),
            pathway: ExhaustivePathwayOptimizer::new(config),
            embed_dim,
            max_depth: 9,
        }
    }
    
    /// Extract facts from context and build SNOAT chain
    pub fn process_context(&mut self, context: &str) {
        self.chain = SNOATChain::new();
        let context_lower = context.to_lowercase();
        
        // Location patterns: "X went to Y" / "X is in Y"
        let location_patterns = [
            ("went to the ", RelationType::Moves),
            ("went to ", RelationType::Moves),
            ("moved to the ", RelationType::Moves),
            ("moved to ", RelationType::Moves),
            ("is in the ", RelationType::Inside),
            ("is in ", RelationType::Inside),
            ("is at the ", RelationType::Inside),
            ("is at ", RelationType::Inside),
        ];
        
        for sentence in context_lower.split('.') {
            let sentence = sentence.trim();
            if sentence.is_empty() {
                continue;
            }
            
            // Try location patterns
            for (pattern, rel_type) in &location_patterns {
                if let Some(pos) = sentence.find(pattern) {
                    let subject = sentence[..pos].split_whitespace().last().unwrap_or("").to_string();
                    let object = sentence[pos + pattern.len()..]
                        .split(|c: char| c == '.' || c == ',' || c == '?')
                        .next()
                        .unwrap_or("")
                        .trim()
                        .to_string();
                    
                    if !subject.is_empty() && !object.is_empty() {
                        let node = SNOATNode::new(0, &subject, *rel_type, &object)
                            .with_attr("location", 1.0);
                        self.chain.add_node(node);
                    }
                }
            }
            
            // Possession patterns: "X picked up Y" / "X has Y"
            let possession_patterns = [
                ("picked up the ", RelationType::Owns),
                ("picked up ", RelationType::Owns),
                ("got the ", RelationType::Owns),
                ("got ", RelationType::Owns),
                ("took the ", RelationType::Owns),
                ("took ", RelationType::Owns),
                ("has the ", RelationType::HasA),
                ("has ", RelationType::HasA),
            ];
            
            for (pattern, rel_type) in &possession_patterns {
                if let Some(pos) = sentence.find(pattern) {
                    let subject = sentence[..pos].split_whitespace().last().unwrap_or("").to_string();
                    let object = sentence[pos + pattern.len()..]
                        .split(|c: char| c == '.' || c == ',' || c == '?')
                        .next()
                        .unwrap_or("")
                        .trim()
                        .to_string();
                    
                    if !subject.is_empty() && !object.is_empty() {
                        let node = SNOATNode::new(0, &subject, *rel_type, &object)
                            .with_attr("possession", 1.0);
                        self.chain.add_node(node);
                        
                        // Create reverse reference: object is_at subject's location
                        // This enables multi-hop: "Where is the apple?" -> who has apple -> where is that person
                        // Clone the location first to avoid borrow conflict
                        let derived_location: Option<String> = self.chain.subject_index
                            .get(&subject)
                            .and_then(|loc_ids| {
                                loc_ids.iter().find_map(|&loc_id| {
                                    if self.chain.nodes.get(loc_id)
                                        .map(|n| n.attributes.contains_key("location"))
                                        .unwrap_or(false) 
                                    {
                                        self.chain.nodes.get(loc_id).map(|n| n.object.clone())
                                    } else {
                                        None
                                    }
                                })
                            });
                        
                        if let Some(location) = derived_location {
                            let obj_loc_node = SNOATNode::new(0, &object, RelationType::Inside, &location)
                                .with_attr("derived", 1.0)
                                .with_attr("via_possession", 1.0);
                            self.chain.add_node(obj_loc_node);
                        }
                    }
                }
            }
            
            // Three-argument relations: "X gave Y to Z" / "X passed Y to Z"
            let give_patterns = [
                ("gave the ", " to "),
                ("gave ", " to "),
                ("passed the ", " to "),
                ("passed ", " to "),
                ("handed the ", " to "),
                ("handed ", " to "),
            ];
            
            for (verb_pattern, to_pattern) in &give_patterns {
                if let Some(verb_pos) = sentence.find(verb_pattern) {
                    let subject = sentence[..verb_pos].split_whitespace().last().unwrap_or("").to_string();
                    let after_verb = &sentence[verb_pos + verb_pattern.len()..];
                    
                    if let Some(to_pos) = after_verb.find(to_pattern) {
                        let object = after_verb[..to_pos].trim().to_string();
                        let recipient = after_verb[to_pos + to_pattern.len()..]
                            .split(|c: char| c == '.' || c == ',' || c == '?')
                            .next()
                            .unwrap_or("")
                            .trim()
                            .to_string();
                        
                        if !subject.is_empty() && !object.is_empty() && !recipient.is_empty() {
                            // Subject no longer has object
                            let give_node = SNOATNode::new(0, &subject, RelationType::None, &object)
                                .with_attr("gave_away", 1.0);
                            self.chain.add_node(give_node);
                            
                            // Recipient now has object
                            let receive_node = SNOATNode::new(0, &recipient, RelationType::Owns, &object)
                                .with_attr("received", 1.0);
                            self.chain.add_node(receive_node);
                            
                            // If we know recipient's location, object is there too
                            let recipient_loc: Option<String> = self.chain.subject_index
                                .get(&recipient)
                                .and_then(|ids| {
                                    ids.iter().find_map(|&id| {
                                        self.chain.nodes.get(id)
                                            .filter(|n| n.attributes.contains_key("location"))
                                            .map(|n| n.object.clone())
                                    })
                                });
                            
                            if let Some(loc) = recipient_loc {
                                let obj_loc = SNOATNode::new(0, &object, RelationType::Inside, &loc)
                                    .with_attr("derived", 1.0)
                                    .with_attr("via_transfer", 1.0);
                                self.chain.add_node(obj_loc);
                            }
                        }
                    }
                }
            }
            
            // Drop patterns: "X dropped Y"
            if sentence.contains("dropped") {
                if let Some(pos) = sentence.find("dropped") {
                    let subject = sentence[..pos].split_whitespace().last().unwrap_or("").to_string();
                    let after = &sentence[pos + 7..];
                    let object = after.trim_start_matches(" the ")
                        .trim_start()
                        .split(|c: char| c == '.' || c == ',' || c == '?')
                        .next()
                        .unwrap_or("")
                        .trim()
                        .to_string();
                    
                    if !subject.is_empty() && !object.is_empty() {
                        // Mark possession as ended (negative relation)
                        let node = SNOATNode::new(0, &subject, RelationType::None, &object)
                            .with_attr("dropped", 1.0)
                            .with_attr("possession_ended", 1.0);
                        self.chain.add_node(node);
                    }
                }
            }
        }
    }
    
    /// Use pathway search to find optimal reasoning chain
    pub fn find_optimal_path(&mut self, question: &str) -> Option<(String, f32, Vec<usize>)> {
        let question_lower = question.to_lowercase();
        
        // First try direct chain answer
        if let Some(result) = self.chain.answer_question(&question_lower) {
            return Some(result);
        }
        
        // If no direct answer, use pathway search over all nodes
        if self.chain.nodes.is_empty() {
            return None;
        }
        
        // Build embeddings for pathway search
        let embeddings: Vec<Vec<f32>> = self.chain.nodes.iter()
            .take(9) // Sacred limit
            .map(|n| n.embedding.clone())
            .collect();
        
        if embeddings.is_empty() {
            return None;
        }
        
        // Create target embedding from question
        let question_hash = question_lower.bytes()
            .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
        let target: Vec<f32> = (0..self.embed_dim)
            .map(|i| {
                let seed = question_hash.wrapping_add(i as u64);
                ((seed as f32 * 0.0001).sin() + (seed as f32 * 0.00003).cos()) * 0.5
            })
            .collect();
        
        // Run pathway search
        self.pathway.set_embeddings(&embeddings);
        self.pathway.set_target(&target);
        let top_paths = self.pathway.fast_search(5);
        
        // Use best path to construct answer
        if let Some(best_path) = top_paths.first() {
            if let Some(&first_idx) = best_path.perm.first() {
                if first_idx < self.chain.nodes.len() {
                    let node = &self.chain.nodes[first_idx];
                    return Some((node.object.clone(), best_path.score as f32, best_path.perm.clone()));
                }
            }
        }
        
        None
    }
    
    /// Score an answer candidate using chain reasoning
    pub fn score_answer(&self, question: &str, candidate: &str) -> f32 {
        let question_lower = question.to_lowercase();
        let candidate_lower = candidate.to_lowercase();
        
        // Check if candidate matches any chain answer
        if let Some((answer, confidence, _path)) = self.chain.answer_question(&question_lower) {
            if answer == candidate_lower || answer.contains(&candidate_lower) || candidate_lower.contains(&answer) {
                return confidence * 50.0;
            }
        }
        
        // Check if candidate appears as object in any relevant node
        for node in &self.chain.nodes {
            if node.object == candidate_lower {
                return node.confidence * 20.0;
            }
        }
        
        0.0
    }
}

// =============================================================================
// SNOAT MACROS
// =============================================================================

/// Macro to define a SNOAT node inline
#[macro_export]
macro_rules! snoat {
    ($subject:expr => $relation:ident => $object:expr) => {
        SNOATNode::new(0, $subject, RelationType::$relation, $object)
    };
    ($subject:expr => $relation:ident => $object:expr, $($attr:ident: $val:expr),*) => {
        {
            let mut node = SNOATNode::new(0, $subject, RelationType::$relation, $object);
            $(
                node.attributes.insert(stringify!($attr).to_string(), $val);
            )*
            node
        }
    };
}

/// Macro to build a chain of SNOAT nodes
#[macro_export]
macro_rules! snoat_chain {
    ($($subject:expr => $relation:ident => $object:expr);+ $(;)?) => {
        {
            let mut chain = SNOATChain::new();
            $(
                let node = SNOATNode::new(0, $subject, RelationType::$relation, $object);
                chain.add_node(node);
            )+
            chain
        }
    };
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_snoat_node() {
        let node = SNOATNode::new(0, "john", RelationType::Moves, "kitchen");
        assert_eq!(node.subject, "john");
        assert_eq!(node.object, "kitchen");
        assert!(!node.embedding.is_empty());
    }
    
    #[test]
    fn test_snoat_chain_linking() {
        let mut chain = SNOATChain::new();
        
        // John went to kitchen
        chain.add_node(SNOATNode::new(0, "john", RelationType::Moves, "kitchen"));
        // John picked up apple
        chain.add_node(SNOATNode::new(0, "john", RelationType::Owns, "apple"));
        
        assert_eq!(chain.nodes.len(), 2);
        // Both should have john as subject, so they share a root
    }
    
    #[test]
    fn test_multi_hop_reasoning() {
        let mut reasoner = ChainPathwayReasoner::new(256);
        
        let context = "John went to the kitchen. John picked up the apple.";
        reasoner.process_context(context);
        
        // Direct question about John
        let result = reasoner.chain.answer_question("Where is John?");
        assert!(result.is_some());
        let (answer, _, _) = result.unwrap();
        assert_eq!(answer, "kitchen");
    }
    
    #[test]
    fn test_object_location_inference() {
        let mut reasoner = ChainPathwayReasoner::new(256);
        
        let context = "Mary went to the garden. Mary picked up the ball.";
        reasoner.process_context(context);
        
        // Multi-hop: Where is the ball? -> Mary has ball -> Mary is in garden
        let result = reasoner.chain.answer_question("Where is the ball?");
        assert!(result.is_some());
        let (answer, _, _) = result.unwrap();
        assert_eq!(answer, "garden");
    }
}
