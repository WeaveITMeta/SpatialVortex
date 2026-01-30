//! World Knowledge Graph for Commonsense Reasoning
//!
//! Provides structured knowledge about:
//! - Physical properties (weight, size, material, state)
//! - Causal relations (causes, effects, preconditions)
//! - Functional knowledge (UsedFor, CapableOf, HasProperty)
//! - Social/behavioral norms
//!
//! This is NOT benchmark-specific - it's general world knowledge
//! that enables reasoning about physical and social situations.

use std::collections::{HashMap, HashSet};

// =============================================================================
// KNOWLEDGE RELATION TYPES
// =============================================================================

/// Types of knowledge relations (ConceptNet-inspired)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RelationType {
    // Taxonomic
    IsA,           // "dog IsA animal"
    InstanceOf,    // "Fido InstanceOf dog"
    
    // Properties
    HasProperty,   // "ice HasProperty cold"
    HasSize,       // "elephant HasSize large"
    HasWeight,     // "feather HasWeight light"
    MadeOf,        // "table MadeOf wood"
    
    // Functional
    UsedFor,       // "knife UsedFor cutting"
    CapableOf,     // "bird CapableOf flying"
    HasA,          // "car HasA wheels"
    PartOf,        // "wheel PartOf car"
    
    // Causal
    Causes,        // "fire Causes heat"
    CausedBy,      // "heat CausedBy fire"
    Requires,      // "fire Requires oxygen"
    
    // Spatial
    LocatedAt,     // "fish LocatedAt water"
    AtLocation,    // "water AtLocation ocean"
    
    // Temporal
    HasPrerequisite, // "eating HasPrerequisite cooking"
    HasSubevent,     // "cooking HasSubevent stirring"
    
    // Social
    MotivatedBy,   // "working MotivatedBy money"
    Desires,       // "human Desires happiness"
}

/// A knowledge triple with confidence
#[derive(Debug, Clone)]
pub struct KnowledgeTriple {
    pub subject: String,
    pub relation: RelationType,
    pub object: String,
    pub confidence: f32,
}

// =============================================================================
// WORLD KNOWLEDGE GRAPH
// =============================================================================

/// World knowledge graph for commonsense reasoning
#[derive(Debug)]
pub struct WorldKnowledgeGraph {
    /// All triples indexed by subject
    by_subject: HashMap<String, Vec<KnowledgeTriple>>,
    /// All triples indexed by object
    by_object: HashMap<String, Vec<KnowledgeTriple>>,
    /// All triples indexed by relation type
    by_relation: HashMap<RelationType, Vec<KnowledgeTriple>>,
    /// Physical properties cache
    physical_properties: HashMap<String, PhysicalProperties>,
    /// Concept embeddings for similarity
    concept_embeddings: HashMap<String, Vec<f32>>,
    /// Embedding dimension
    embed_dim: usize,
}

/// Physical properties of an object
#[derive(Debug, Clone, Default)]
pub struct PhysicalProperties {
    pub weight: Option<WeightClass>,
    pub size: Option<SizeClass>,
    pub material: Option<String>,
    pub state: Option<PhysicalState>,
    pub temperature: Option<TemperatureClass>,
    pub is_container: bool,
    pub is_living: bool,
    pub is_edible: bool,
    pub is_dangerous: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WeightClass { VeryLight, Light, Medium, Heavy, VeryHeavy }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SizeClass { Tiny, Small, Medium, Large, Huge }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PhysicalState { Solid, Liquid, Gas, Plasma }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TemperatureClass { Frozen, Cold, Cool, Warm, Hot, Burning }

impl WorldKnowledgeGraph {
    pub fn new(embed_dim: usize) -> Self {
        let mut graph = Self {
            by_subject: HashMap::new(),
            by_object: HashMap::new(),
            by_relation: HashMap::new(),
            physical_properties: HashMap::new(),
            concept_embeddings: HashMap::new(),
            embed_dim,
        };
        
        // Initialize with core commonsense knowledge
        graph.load_core_knowledge();
        
        graph
    }
    
    /// Add a knowledge triple
    pub fn add_triple(&mut self, subject: &str, relation: RelationType, object: &str, confidence: f32) {
        let triple = KnowledgeTriple {
            subject: subject.to_lowercase(),
            relation: relation.clone(),
            object: object.to_lowercase(),
            confidence,
        };
        
        self.by_subject
            .entry(subject.to_lowercase())
            .or_default()
            .push(triple.clone());
            
        self.by_object
            .entry(object.to_lowercase())
            .or_default()
            .push(triple.clone());
            
        self.by_relation
            .entry(relation)
            .or_default()
            .push(triple);
    }
    
    /// Query knowledge by subject
    pub fn query_subject(&self, subject: &str) -> Vec<&KnowledgeTriple> {
        self.by_subject
            .get(&subject.to_lowercase())
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }
    
    /// Query knowledge by subject and relation
    pub fn query(&self, subject: &str, relation: &RelationType) -> Vec<&KnowledgeTriple> {
        self.by_subject
            .get(&subject.to_lowercase())
            .map(|v| v.iter().filter(|t| &t.relation == relation).collect())
            .unwrap_or_default()
    }
    
    /// Check if a relation holds
    pub fn holds(&self, subject: &str, relation: &RelationType, object: &str) -> Option<f32> {
        let subject_lower = subject.to_lowercase();
        let object_lower = object.to_lowercase();
        
        // Direct lookup
        if let Some(triples) = self.by_subject.get(&subject_lower) {
            for triple in triples {
                if &triple.relation == relation && triple.object == object_lower {
                    return Some(triple.confidence);
                }
            }
        }
        
        // Try inheritance through IsA
        if let Some(parents) = self.by_subject.get(&subject_lower) {
            for parent_triple in parents.iter().filter(|t| t.relation == RelationType::IsA) {
                if let Some(conf) = self.holds(&parent_triple.object, relation, object) {
                    return Some(conf * 0.9); // Decay confidence through inheritance
                }
            }
        }
        
        None
    }
    
    /// Get physical properties of a concept
    pub fn get_physical_properties(&self, concept: &str) -> Option<&PhysicalProperties> {
        self.physical_properties.get(&concept.to_lowercase())
    }
    
    /// Compare two objects on a property
    pub fn compare(&self, obj1: &str, obj2: &str, property: &str) -> Option<std::cmp::Ordering> {
        let props1 = self.get_physical_properties(obj1)?;
        let props2 = self.get_physical_properties(obj2)?;
        
        match property {
            "weight" | "heavy" | "heavier" => {
                let w1 = props1.weight.as_ref()?;
                let w2 = props2.weight.as_ref()?;
                Some((*w1 as i32).cmp(&(*w2 as i32)))
            }
            "size" | "big" | "bigger" | "large" | "larger" => {
                let s1 = props1.size.as_ref()?;
                let s2 = props2.size.as_ref()?;
                Some((*s1 as i32).cmp(&(*s2 as i32)))
            }
            _ => None,
        }
    }
    
    /// Score how plausible a statement is
    pub fn score_plausibility(&self, subject: &str, action: &str, object: &str) -> f32 {
        let mut score: f32 = 0.5; // Neutral baseline
        
        // Check if subject is capable of action
        if self.holds(subject, &RelationType::CapableOf, action).is_some() {
            score += 0.3;
        }
        
        // Check if object is typically used for action
        if self.holds(object, &RelationType::UsedFor, action).is_some() {
            score += 0.2;
        }
        
        // Check physical compatibility
        if let (Some(subj_props), Some(obj_props)) = (
            self.get_physical_properties(subject),
            self.get_physical_properties(object)
        ) {
            // Living things can't be used as tools
            if obj_props.is_living && action.contains("use") {
                score -= 0.3;
            }
            
            // Can't eat non-edible things
            if action.contains("eat") && !obj_props.is_edible {
                score -= 0.4;
            }
            
            // Can't lift very heavy things easily
            if action.contains("lift") || action.contains("carry") {
                if let Some(WeightClass::VeryHeavy) = obj_props.weight {
                    score -= 0.3;
                }
            }
        }
        
        score.clamp(0.0f32, 1.0f32)
    }
    
    /// Answer a commonsense question
    pub fn answer_question(&self, question: &str, choices: &[String]) -> Option<(usize, f32)> {
        let question_lower = question.to_lowercase();
        
        // Physical comparison questions
        if question_lower.contains("heavier") || question_lower.contains("lighter") {
            return self.answer_weight_question(&question_lower, choices);
        }
        
        if question_lower.contains("bigger") || question_lower.contains("smaller") {
            return self.answer_size_question(&question_lower, choices);
        }
        
        // "What is X used for?" questions
        if question_lower.contains("used for") || question_lower.contains("use a") {
            return self.answer_function_question(&question_lower, choices);
        }
        
        // "What can X do?" questions
        if question_lower.contains("can a") || question_lower.contains("capable of") {
            return self.answer_capability_question(&question_lower, choices);
        }
        
        // PIQA-style: "How to accomplish X?" - score physical plausibility
        if question_lower.contains("how to") || question_lower.contains("how do") || 
           question_lower.contains("how can") || question_lower.contains("what is the best way") {
            return self.answer_how_to_question(&question_lower, choices);
        }
        
        // WinoGrande-style: Coreference with blank
        if question_lower.contains("_") || question_lower.contains("[blank]") {
            return self.answer_coreference_question(&question_lower, choices);
        }
        
        // General plausibility scoring
        let mut best_idx = 0;
        let mut best_score = 0.0f32;
        
        for (idx, choice) in choices.iter().enumerate() {
            let score = self.score_choice_plausibility(&question_lower, &choice.to_lowercase());
            if score > best_score {
                best_score = score;
                best_idx = idx;
            }
        }
        
        if best_score > 0.55 {
            Some((best_idx, best_score))
        } else {
            None
        }
    }
    
    /// Answer PIQA-style "How to" questions using physical reasoning
    fn answer_how_to_question(&self, question: &str, choices: &[String]) -> Option<(usize, f32)> {
        let mut best_idx = 0;
        let mut best_score = 0.0f32;
        
        for (idx, choice) in choices.iter().enumerate() {
            let choice_lower = choice.to_lowercase();
            let mut score: f32 = 0.5;
            
            // Extract action words from choice
            let choice_words: Vec<&str> = choice_lower.split_whitespace().collect();
            
            // Check for physically plausible actions
            for word in &choice_words {
                // Positive indicators - common sensible actions
                if ["use", "put", "place", "hold", "take", "move", "turn", "push", "pull",
                    "open", "close", "cut", "pour", "mix", "heat", "cool", "wash", "dry",
                    "wrap", "cover", "remove", "add", "apply", "press", "lift", "lower"].contains(word) {
                    score += 0.05;
                }
                
                // Check if word relates to known objects
                if self.by_subject.contains_key(*word) {
                    score += 0.1;
                }
            }
            
            // Penalize nonsensical combinations
            if choice_lower.contains("eat") && choice_lower.contains("metal") {
                score -= 0.3;
            }
            if choice_lower.contains("drink") && choice_lower.contains("solid") {
                score -= 0.3;
            }
            
            // Prefer shorter, more direct answers (Occam's razor)
            if choice_words.len() < 15 {
                score += 0.05;
            }
            
            if score > best_score {
                best_score = score;
                best_idx = idx;
            }
        }
        
        if best_score > 0.55 {
            Some((best_idx, best_score))
        } else {
            None
        }
    }
    
    /// Answer WinoGrande-style coreference questions
    fn answer_coreference_question(&self, question: &str, choices: &[String]) -> Option<(usize, f32)> {
        // WinoGrande format: "The X couldn't fit in the Y because _ was too big"
        // Need to determine which entity the pronoun refers to
        
        let mut best_idx = 0;
        let mut best_score = 0.0f32;
        
        for (idx, choice) in choices.iter().enumerate() {
            let choice_lower = choice.to_lowercase();
            let mut score: f32 = 0.5;
            
            // Fill in the blank and check semantic coherence
            let filled = question.replace("_", &choice_lower)
                                 .replace("[blank]", &choice_lower);
            
            // Check for size/fit relationships
            if filled.contains("too big") || filled.contains("too large") {
                // The subject that is "too big" should be the larger one
                if let Some(props) = self.get_physical_properties(&choice_lower) {
                    if let Some(SizeClass::Large | SizeClass::Huge) = props.size {
                        score += 0.2;
                    }
                }
            }
            
            if filled.contains("too small") || filled.contains("too little") {
                if let Some(props) = self.get_physical_properties(&choice_lower) {
                    if let Some(SizeClass::Small | SizeClass::Tiny) = props.size {
                        score += 0.2;
                    }
                }
            }
            
            // Check for weight relationships
            if filled.contains("too heavy") {
                if let Some(props) = self.get_physical_properties(&choice_lower) {
                    if let Some(WeightClass::Heavy | WeightClass::VeryHeavy) = props.weight {
                        score += 0.2;
                    }
                }
            }
            
            if score > best_score {
                best_score = score;
                best_idx = idx;
            }
        }
        
        if best_score > 0.55 {
            Some((best_idx, best_score))
        } else {
            None
        }
    }
    
    fn answer_weight_question(&self, question: &str, choices: &[String]) -> Option<(usize, f32)> {
        for (idx, choice) in choices.iter().enumerate() {
            let choice_lower = choice.to_lowercase();
            if let Some(props) = self.get_physical_properties(&choice_lower) {
                if question.contains("heavier") {
                    if let Some(WeightClass::Heavy | WeightClass::VeryHeavy) = props.weight {
                        return Some((idx, 0.8));
                    }
                }
                if question.contains("lighter") {
                    if let Some(WeightClass::Light | WeightClass::VeryLight) = props.weight {
                        return Some((idx, 0.8));
                    }
                }
            }
        }
        None
    }
    
    fn answer_size_question(&self, question: &str, choices: &[String]) -> Option<(usize, f32)> {
        for (idx, choice) in choices.iter().enumerate() {
            let choice_lower = choice.to_lowercase();
            if let Some(props) = self.get_physical_properties(&choice_lower) {
                if question.contains("bigger") || question.contains("larger") {
                    if let Some(SizeClass::Large | SizeClass::Huge) = props.size {
                        return Some((idx, 0.8));
                    }
                }
                if question.contains("smaller") {
                    if let Some(SizeClass::Small | SizeClass::Tiny) = props.size {
                        return Some((idx, 0.8));
                    }
                }
            }
        }
        None
    }
    
    fn answer_function_question(&self, question: &str, choices: &[String]) -> Option<(usize, f32)> {
        // Extract the object being asked about
        let words: Vec<&str> = question.split_whitespace().collect();
        
        for word in &words {
            let triples = self.query(word, &RelationType::UsedFor);
            if !triples.is_empty() {
                for (idx, choice) in choices.iter().enumerate() {
                    let choice_lower = choice.to_lowercase();
                    for triple in &triples {
                        if choice_lower.contains(&triple.object) {
                            return Some((idx, triple.confidence));
                        }
                    }
                }
            }
        }
        None
    }
    
    fn answer_capability_question(&self, question: &str, choices: &[String]) -> Option<(usize, f32)> {
        let words: Vec<&str> = question.split_whitespace().collect();
        
        for word in &words {
            let triples = self.query(word, &RelationType::CapableOf);
            if !triples.is_empty() {
                for (idx, choice) in choices.iter().enumerate() {
                    let choice_lower = choice.to_lowercase();
                    for triple in &triples {
                        if choice_lower.contains(&triple.object) {
                            return Some((idx, triple.confidence));
                        }
                    }
                }
            }
        }
        None
    }
    
    fn score_choice_plausibility(&self, question: &str, choice: &str) -> f32 {
        let mut score: f32 = 0.5;
        
        // Check for known relations between question words and choice
        let q_words: HashSet<&str> = question.split_whitespace()
            .filter(|w| w.len() > 2)
            .collect();
        let c_words: HashSet<&str> = choice.split_whitespace()
            .filter(|w| w.len() > 2)
            .collect();
        
        for q_word in &q_words {
            for c_word in &c_words {
                // Check any relation
                if let Some(triples) = self.by_subject.get(*q_word) {
                    for triple in triples {
                        if triple.object.contains(c_word) {
                            score += 0.1 * triple.confidence;
                        }
                    }
                }
            }
        }
        
        score.clamp(0.0f32, 1.0f32)
    }
    
    /// Load core commonsense knowledge
    fn load_core_knowledge(&mut self) {
        // =================================================================
        // PHYSICAL OBJECTS AND PROPERTIES
        // =================================================================
        
        // Animals
        self.add_triple("dog", RelationType::IsA, "animal", 1.0);
        self.add_triple("cat", RelationType::IsA, "animal", 1.0);
        self.add_triple("bird", RelationType::IsA, "animal", 1.0);
        self.add_triple("fish", RelationType::IsA, "animal", 1.0);
        self.add_triple("elephant", RelationType::IsA, "animal", 1.0);
        self.add_triple("mouse", RelationType::IsA, "animal", 1.0);
        self.add_triple("horse", RelationType::IsA, "animal", 1.0);
        
        self.add_triple("animal", RelationType::CapableOf, "move", 1.0);
        self.add_triple("animal", RelationType::CapableOf, "eat", 1.0);
        self.add_triple("animal", RelationType::CapableOf, "sleep", 1.0);
        self.add_triple("bird", RelationType::CapableOf, "fly", 0.9);
        self.add_triple("fish", RelationType::CapableOf, "swim", 1.0);
        self.add_triple("dog", RelationType::CapableOf, "bark", 1.0);
        self.add_triple("cat", RelationType::CapableOf, "meow", 1.0);
        
        // Physical properties
        self.physical_properties.insert("elephant".to_string(), PhysicalProperties {
            weight: Some(WeightClass::VeryHeavy),
            size: Some(SizeClass::Huge),
            is_living: true,
            ..Default::default()
        });
        
        self.physical_properties.insert("mouse".to_string(), PhysicalProperties {
            weight: Some(WeightClass::VeryLight),
            size: Some(SizeClass::Tiny),
            is_living: true,
            ..Default::default()
        });
        
        self.physical_properties.insert("feather".to_string(), PhysicalProperties {
            weight: Some(WeightClass::VeryLight),
            size: Some(SizeClass::Small),
            ..Default::default()
        });
        
        self.physical_properties.insert("rock".to_string(), PhysicalProperties {
            weight: Some(WeightClass::Heavy),
            size: Some(SizeClass::Medium),
            material: Some("stone".to_string()),
            state: Some(PhysicalState::Solid),
            ..Default::default()
        });
        
        self.physical_properties.insert("water".to_string(), PhysicalProperties {
            state: Some(PhysicalState::Liquid),
            temperature: Some(TemperatureClass::Cool),
            ..Default::default()
        });
        
        self.physical_properties.insert("ice".to_string(), PhysicalProperties {
            state: Some(PhysicalState::Solid),
            temperature: Some(TemperatureClass::Frozen),
            material: Some("water".to_string()),
            ..Default::default()
        });
        
        self.physical_properties.insert("fire".to_string(), PhysicalProperties {
            state: Some(PhysicalState::Plasma),
            temperature: Some(TemperatureClass::Burning),
            is_dangerous: true,
            ..Default::default()
        });
        
        // =================================================================
        // TOOLS AND FUNCTIONS
        // =================================================================
        
        self.add_triple("knife", RelationType::UsedFor, "cutting", 1.0);
        self.add_triple("hammer", RelationType::UsedFor, "hitting", 1.0);
        self.add_triple("hammer", RelationType::UsedFor, "building", 0.8);
        self.add_triple("pen", RelationType::UsedFor, "writing", 1.0);
        self.add_triple("cup", RelationType::UsedFor, "drinking", 1.0);
        self.add_triple("cup", RelationType::IsA, "container", 1.0);
        self.add_triple("bowl", RelationType::UsedFor, "eating", 0.9);
        self.add_triple("bowl", RelationType::IsA, "container", 1.0);
        self.add_triple("chair", RelationType::UsedFor, "sitting", 1.0);
        self.add_triple("bed", RelationType::UsedFor, "sleeping", 1.0);
        self.add_triple("stove", RelationType::UsedFor, "cooking", 1.0);
        self.add_triple("refrigerator", RelationType::UsedFor, "storing food", 1.0);
        self.add_triple("car", RelationType::UsedFor, "transportation", 1.0);
        self.add_triple("phone", RelationType::UsedFor, "communication", 1.0);
        self.add_triple("computer", RelationType::UsedFor, "computing", 1.0);
        
        // =================================================================
        // CAUSAL KNOWLEDGE
        // =================================================================
        
        self.add_triple("fire", RelationType::Causes, "heat", 1.0);
        self.add_triple("fire", RelationType::Causes, "light", 1.0);
        self.add_triple("fire", RelationType::Requires, "oxygen", 1.0);
        self.add_triple("rain", RelationType::Causes, "wet", 1.0);
        self.add_triple("cold", RelationType::Causes, "freezing", 0.8);
        self.add_triple("heat", RelationType::Causes, "melting", 0.8);
        self.add_triple("eating", RelationType::HasPrerequisite, "food", 1.0);
        self.add_triple("cooking", RelationType::HasPrerequisite, "ingredients", 1.0);
        self.add_triple("driving", RelationType::HasPrerequisite, "car", 1.0);
        
        // =================================================================
        // FOOD AND EDIBILITY
        // =================================================================
        
        let edible_items = ["apple", "bread", "meat", "fish", "rice", "vegetable", "fruit", "cheese", "egg"];
        for item in edible_items {
            self.add_triple(item, RelationType::IsA, "food", 1.0);
            self.physical_properties.insert(item.to_string(), PhysicalProperties {
                is_edible: true,
                ..Default::default()
            });
        }
        
        // =================================================================
        // LOCATIONS
        // =================================================================
        
        self.add_triple("kitchen", RelationType::UsedFor, "cooking", 1.0);
        self.add_triple("bedroom", RelationType::UsedFor, "sleeping", 1.0);
        self.add_triple("bathroom", RelationType::UsedFor, "washing", 1.0);
        self.add_triple("office", RelationType::UsedFor, "working", 1.0);
        self.add_triple("school", RelationType::UsedFor, "learning", 1.0);
        self.add_triple("hospital", RelationType::UsedFor, "healing", 1.0);
        self.add_triple("restaurant", RelationType::UsedFor, "eating", 1.0);
        
        // =================================================================
        // HUMAN CAPABILITIES AND NEEDS
        // =================================================================
        
        self.add_triple("human", RelationType::CapableOf, "think", 1.0);
        self.add_triple("human", RelationType::CapableOf, "walk", 1.0);
        self.add_triple("human", RelationType::CapableOf, "talk", 1.0);
        self.add_triple("human", RelationType::CapableOf, "read", 0.9);
        self.add_triple("human", RelationType::CapableOf, "write", 0.9);
        self.add_triple("human", RelationType::Desires, "food", 1.0);
        self.add_triple("human", RelationType::Desires, "water", 1.0);
        self.add_triple("human", RelationType::Desires, "shelter", 1.0);
        self.add_triple("human", RelationType::Desires, "happiness", 1.0);
        
        // =================================================================
        // SOCIAL KNOWLEDGE
        // =================================================================
        
        self.add_triple("working", RelationType::MotivatedBy, "money", 0.9);
        self.add_triple("studying", RelationType::MotivatedBy, "knowledge", 0.8);
        self.add_triple("exercising", RelationType::MotivatedBy, "health", 0.9);
        self.add_triple("helping", RelationType::MotivatedBy, "kindness", 0.8);
    }
    
    /// Get embedding for a concept (generates if not cached)
    pub fn get_concept_embedding(&mut self, concept: &str) -> Vec<f32> {
        let concept_lower = concept.to_lowercase();
        
        if let Some(embed) = self.concept_embeddings.get(&concept_lower) {
            return embed.clone();
        }
        
        // Generate embedding based on concept's relations
        let mut embedding = vec![0.0f32; self.embed_dim];
        
        // Use hash of concept name as seed
        let hash = concept_lower.bytes().fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
        
        for i in 0..self.embed_dim {
            let seed = hash.wrapping_add(i as u64);
            embedding[i] = ((seed as f32 * 0.0001).sin() + 1.0) / 2.0;
        }
        
        // Modify based on known properties
        if let Some(triples) = self.by_subject.get(&concept_lower) {
            for (i, triple) in triples.iter().enumerate().take(10) {
                let idx = (i * 7) % self.embed_dim;
                embedding[idx] += triple.confidence * 0.1;
            }
        }
        
        // Normalize
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in &mut embedding {
                *x /= norm;
            }
        }
        
        self.concept_embeddings.insert(concept_lower, embedding.clone());
        embedding
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_queries() {
        let graph = WorldKnowledgeGraph::new(64);
        
        // Test IsA inheritance
        assert!(graph.holds("dog", &RelationType::IsA, "animal").is_some());
        
        // Test capability through inheritance
        assert!(graph.holds("dog", &RelationType::CapableOf, "eat").is_some());
    }
    
    #[test]
    fn test_physical_properties() {
        let graph = WorldKnowledgeGraph::new(64);
        
        let elephant = graph.get_physical_properties("elephant").unwrap();
        assert_eq!(elephant.weight, Some(WeightClass::VeryHeavy));
        assert_eq!(elephant.size, Some(SizeClass::Huge));
        
        let mouse = graph.get_physical_properties("mouse").unwrap();
        assert_eq!(mouse.weight, Some(WeightClass::VeryLight));
    }
    
    #[test]
    fn test_comparison() {
        let graph = WorldKnowledgeGraph::new(64);
        
        let cmp = graph.compare("elephant", "mouse", "weight");
        assert_eq!(cmp, Some(std::cmp::Ordering::Greater));
        
        let cmp = graph.compare("mouse", "elephant", "size");
        assert_eq!(cmp, Some(std::cmp::Ordering::Less));
    }
    
    #[test]
    fn test_function_query() {
        let graph = WorldKnowledgeGraph::new(64);
        
        let triples = graph.query("knife", &RelationType::UsedFor);
        assert!(!triples.is_empty());
        assert!(triples.iter().any(|t| t.object == "cutting"));
    }
}
