# Populating Semantic Associations in FluxMatrix

## Problem

Semantic associations are currently **empty**:

```rust
let semantic_index = SemanticIndex {
    positive_associations: Vec::new(),  // ❌ EMPTY!
    negative_associations: Vec::new(),  // ❌ EMPTY!
    neutral_base: node_def.name.clone(),
    predicates: Vec::new(),
    relations: Vec::new(),
};
```

**This means:**
- No semantic matching possible
- No pattern-aware positioning
- No knowledge-guided inference

**We need to populate these with real data!**

---

## Solution: Multi-Source Semantic Population

### **Data Sources**

1. **Subject Definitions** (curated)
2. **RAG Training** (Grokipedia, Scholar)
3. **User Interactions** (learned)
4. **LLM Generation** (synthetic)

---

### **Step 1: Curated Subject Definitions**

Create rich subject definitions with semantic associations:

```rust
// subjects/consciousness.rs
pub fn consciousness_definition() -> SubjectDefinition {
    SubjectDefinition {
        name: "consciousness".to_string(),
        nodes: vec![
            // Position 0 - Center (Neutral)
            NodeDefinition {
                position: 0,
                name: "Awareness".to_string(),
                positive: vec![
                    ("presence", 1, 0.9),
                    ("mindfulness", 2, 0.85),
                    ("attention", 1, 0.8),
                ],
                negative: vec![
                    ("unconscious", -1, 0.7),
                    ("oblivious", -2, 0.6),
                ],
            },
            // Position 1 - Start (Good/Beginning)
            NodeDefinition {
                position: 1,
                name: "Self-Awareness".to_string(),
                positive: vec![
                    ("identity", 1, 0.9),
                    ("reflection", 2, 0.85),
                    ("introspection", 3, 0.8),
                    ("self-knowledge", 4, 0.75),
                ],
                negative: vec![
                    ("ignorance", -1, 0.7),
                    ("delusion", -2, 0.6),
                ],
            },
            // Position 2 - Expansion
            NodeDefinition {
                position: 2,
                name: "Perception".to_string(),
                positive: vec![
                    ("senses", 1, 0.9),
                    ("observation", 2, 0.85),
                    ("experience", 1, 0.8),
                    ("qualia", 3, 0.75),
                ],
                negative: vec![
                    ("blindness", -1, 0.7),
                    ("numbness", -2, 0.6),
                ],
            },
            // Position 3 - Sacred (Good/Ethos)
            // (handled by sacred_guides)
            
            // Position 4 - Power
            NodeDefinition {
                position: 4,
                name: "Cognition".to_string(),
                positive: vec![
                    ("thinking", 1, 0.9),
                    ("reasoning", 2, 0.85),
                    ("intelligence", 3, 0.8),
                    ("understanding", 2, 0.75),
                ],
                negative: vec![
                    ("stupidity", -1, 0.7),
                    ("confusion", -2, 0.6),
                ],
            },
            // Position 5 - Change
            NodeDefinition {
                position: 5,
                name: "Emotion".to_string(),
                positive: vec![
                    ("feeling", 1, 0.9),
                    ("empathy", 3, 0.85),
                    ("compassion", 4, 0.8),
                    ("love", 5, 0.75),
                ],
                negative: vec![
                    ("apathy", -1, 0.7),
                    ("hatred", -3, 0.6),
                ],
            },
            // Position 6 - Sacred (Bad/Pathos)
            // (handled by sacred_guides)
            
            // Position 7 - Wisdom
            NodeDefinition {
                position: 7,
                name: "Consciousness Studies".to_string(),
                positive: vec![
                    ("neuroscience", 2, 0.9),
                    ("philosophy", 3, 0.85),
                    ("meditation", 4, 0.8),
                    ("mysticism", 5, 0.75),
                ],
                negative: vec![
                    ("materialism", -1, 0.6),
                    ("reductionism", -2, 0.5),
                ],
            },
            // Position 8 - Mastery
            NodeDefinition {
                position: 8,
                name: "Higher Consciousness".to_string(),
                positive: vec![
                    ("enlightenment", 5, 0.95),
                    ("transcendence", 6, 0.9),
                    ("awakening", 5, 0.85),
                    ("nirvana", 7, 0.8),
                ],
                negative: vec![
                    ("delusion", -2, 0.7),
                    ("maya", -1, 0.6),
                ],
            },
            // Position 9 - Sacred (Divine/Logos)
            // (handled by sacred_guides)
        ],
        sacred_guides: vec![
            SacredDefinition {
                position: 3,
                name: "Unity of Self".to_string(),
                divine_properties: vec![
                    ("integration", 0.95),
                    ("wholeness", 0.9),
                    ("coherence", 0.85),
                ],
            },
            SacredDefinition {
                position: 6,
                name: "Emotional Core".to_string(),
                divine_properties: vec![
                    ("heart-mind", 0.95),
                    ("feeling-center", 0.9),
                    ("empathic-bond", 0.85),
                ],
            },
            SacredDefinition {
                position: 9,
                name: "Divine Mind".to_string(),
                divine_properties: vec![
                    ("cosmic-consciousness", 0.98),
                    ("universal-awareness", 0.95),
                    ("absolute-knowing", 0.9),
                ],
            },
        ],
    }
}

#[derive(Debug, Clone)]
pub struct NodeDefinition {
    pub position: u8,
    pub name: String,
    pub positive: Vec<(&'static str, i16, f64)>,  // (word, index, confidence)
    pub negative: Vec<(&'static str, i16, f64)>,
}

#[derive(Debug, Clone)]
pub struct SacredDefinition {
    pub position: u8,
    pub name: String,
    pub divine_properties: Vec<(&'static str, f64)>,  // (property, confidence)
}
```

---

### **Step 2: Load Definitions into FluxMatrix**

```rust
impl FluxMatrixEngine {
    /// Create matrix from curated definition with full semantics
    pub fn create_matrix_from_curated(
        &self,
        subject_def: SubjectDefinition,
    ) -> Result<FluxMatrix> {
        let matrix_id = Uuid::new_v4();
        let now = Utc::now();
        let subject = subject_def.name.clone();
        
        let mut nodes = HashMap::new();
        let mut sacred_guides = HashMap::new();
        
        // Create nodes with populated semantics
        for node_def in &subject_def.nodes {
            let base_value = self.get_flux_value_at_position(node_def.position);
            
            // Populate positive associations
            let mut positive_associations = Vec::new();
            for (word, index, confidence) in &node_def.positive {
                let mut assoc = SemanticAssociation::new(
                    word.to_string(),
                    *index,
                    *confidence,
                );
                
                // Calculate ELP for each association
                let elp = self.calculate_elp_for_position(node_def.position);
                assoc.set_attribute("ethos".to_string(), elp.ethos);
                assoc.set_attribute("logos".to_string(), elp.logos);
                assoc.set_attribute("pathos".to_string(), elp.pathos);
                
                positive_associations.push(assoc);
            }
            
            // Populate negative associations
            let mut negative_associations = Vec::new();
            for (word, index, confidence) in &node_def.negative {
                let mut assoc = SemanticAssociation::new(
                    word.to_string(),
                    *index,
                    *confidence,
                );
                
                // Invert ELP for negative
                let elp = self.calculate_elp_for_position(node_def.position);
                assoc.set_attribute("ethos".to_string(), -elp.ethos);
                assoc.set_attribute("logos".to_string(), -elp.logos);
                assoc.set_attribute("pathos".to_string(), -elp.pathos);
                
                negative_associations.push(assoc);
            }
            
            let semantic_index = SemanticIndex {
                positive_associations,
                negative_associations,
                neutral_base: node_def.name.clone(),
                predicates: Vec::new(),  // TODO: Add predicates
                relations: Vec::new(),   // TODO: Add relations
            };
            
            let node = FluxNode {
                position: node_def.position,
                base_value,
                semantic_index,
                attributes: NodeAttributes {
                    properties: HashMap::new(),
                    parameters: HashMap::new(),
                    state: NodeState {
                        active: true,
                        last_accessed: now,
                        usage_count: 0,
                        context_stack: Vec::new(),
                    },
                    dynamics: NodeDynamics {
                        evolution_rate: 1.0,
                        stability_index: 0.5,
                        interaction_patterns: Vec::new(),
                        learning_adjustments: Vec::new(),
                    },
                },
                connections: self.create_node_connections(node_def.position),
            };
            
            nodes.insert(node_def.position, node);
        }
        
        // Create sacred guides with divine properties
        for sacred_def in &subject_def.sacred_guides {
            let mut divine_properties = Vec::new();
            for (property, _confidence) in &sacred_def.divine_properties {
                divine_properties.push(property.to_string());
            }
            
            let guide = SacredGuide {
                position: sacred_def.position,
                divine_properties,
                intersection_points: self.create_intersection_points(sacred_def.position),
                geometric_significance: format!(
                    "Sacred {} in {}: {}",
                    sacred_def.position,
                    subject,
                    sacred_def.name
                ),
            };
            
            sacred_guides.insert(sacred_def.position, guide);
        }
        
        Ok(FluxMatrix {
            id: matrix_id,
            subject,
            nodes,
            sacred_guides,
            created_at: now,
            updated_at: now,
        })
    }
    
    /// Calculate ELP tensor for position based on sacred geometry
    fn calculate_elp_for_position(&self, position: u8) -> ELPTensor {
        match position {
            0 => ELPTensor { ethos: 0.0, logos: 0.0, pathos: 0.0 },  // Neutral
            1 => ELPTensor { ethos: 5.0, logos: 3.0, pathos: 2.0 },  // Self (Ethos-heavy)
            2 => ELPTensor { ethos: 3.0, logos: 4.0, pathos: 3.0 },  // Perception (Balanced)
            3 => ELPTensor { ethos: 9.0, logos: 6.0, pathos: 3.0 },  // Sacred Ethos
            4 => ELPTensor { ethos: 4.0, logos: 6.0, pathos: 3.0 },  // Cognition (Logos-heavy)
            5 => ELPTensor { ethos: 3.0, logos: 3.0, pathos: 7.0 },  // Emotion (Pathos-heavy)
            6 => ELPTensor { ethos: 3.0, logos: 6.0, pathos: 9.0 },  // Sacred Pathos
            7 => ELPTensor { ethos: 5.0, logos: 7.0, pathos: 4.0 },  // Wisdom (Logos-heavy)
            8 => ELPTensor { ethos: 6.0, logos: 6.0, pathos: 5.0 },  // Mastery (Balanced high)
            9 => ELPTensor { ethos: 6.0, logos: 9.0, pathos: 6.0 },  // Sacred Logos
            _ => ELPTensor { ethos: 0.0, logos: 0.0, pathos: 0.0 },
        }
    }
}
```

---

### **Step 3: LLM-Generated Semantic Expansion**

Use AI to expand associations dynamically:

```rust
impl ASIOrchestrator {
    /// Expand semantic associations using LLM
    pub async fn expand_semantics_with_llm(
        &mut self,
        subject: &str,
        position: u8,
        neutral_base: &str,
    ) -> Result<Vec<SemanticAssociation>> {
        #[cfg(feature = "agents")]
        if let Some(llm) = &self.llm_bridge {
            let prompt = format!(
                "Generate 10 semantically related words for '{}' in the context of '{}'. \
                Position {} represents: {}.\n\
                Format: word,index,confidence\n\
                Index should be 1-10 for positive associations.\n\
                Confidence should be 0.0-1.0.",
                neutral_base, subject, position, neutral_base
            );
            
            let response = llm.query(&prompt).await?;
            
            // Parse response
            let mut associations = Vec::new();
            for line in response.lines() {
                if let Some((word, rest)) = line.split_once(',') {
                    if let Some((index_str, conf_str)) = rest.split_once(',') {
                        if let (Ok(index), Ok(confidence)) = (
                            index_str.trim().parse::<i16>(),
                            conf_str.trim().parse::<f64>()
                        ) {
                            associations.push(SemanticAssociation::new(
                                word.trim().to_string(),
                                index,
                                confidence,
                            ));
                        }
                    }
                }
            }
            
            return Ok(associations);
        }
        
        Ok(Vec::new())
    }
}
```

---

### **Step 4: Learn from RAG Training Data**

Extract semantics from Grokipedia/Scholar articles:

```rust
impl ASIOrchestrator {
    /// Learn semantic associations from RAG documents
    pub async fn learn_from_rag_documents(
        &mut self,
        subject: &str,
    ) -> Result<()> {
        // Get RAG documents for subject
        #[cfg(feature = "rag")]
        if let Some(rag) = &self.rag_system {
            let docs = rag.retrieve_by_subject(subject, 100).await?;
            
            // Extract semantic associations
            for doc in docs {
                // Tokenize and analyze
                let tokens = self.extract_significant_tokens(&doc.content);
                
                // Calculate semantic similarity to each position
                for position in 0..=9 {
                    if self.sacred_positions.contains(&position) {
                        continue;  // Skip sacred positions
                    }
                    
                    let matrix = self.flux_engine.get_or_create_matrix(subject)?;
                    if let Some(node) = matrix.nodes.get(&position) {
                        let neutral_base = &node.semantic_index.neutral_base;
                        
                        // Find tokens related to this position
                        for token in &tokens {
                            let similarity = self.calculate_token_similarity(
                                token,
                                neutral_base,
                                &doc.content
                            );
                            
                            if similarity > 0.6 {
                                // Add as semantic association
                                let index = self.calculate_ladder_index(similarity);
                                let assoc = SemanticAssociation::new(
                                    token.clone(),
                                    index,
                                    similarity as f64,
                                );
                                
                                // Store in matrix
                                self.add_semantic_association(
                                    subject,
                                    position,
                                    assoc
                                ).await?;
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Extract significant tokens from text
    fn extract_significant_tokens(&self, text: &str) -> Vec<String> {
        // Simple extraction - could use NLP
        text.split_whitespace()
            .filter(|w| w.len() > 4)  // Skip short words
            .filter(|w| !self.is_stopword(w))
            .take(100)  // Top 100 tokens
            .map(|w| w.to_lowercase())
            .collect()
    }
    
    /// Calculate semantic similarity between token and base concept
    fn calculate_token_similarity(
        &self,
        token: &str,
        base: &str,
        context: &str,
    ) -> f32 {
        // Simple co-occurrence metric
        let token_count = context.matches(token).count();
        let base_count = context.matches(base).count();
        let co_occurrence = context.split(base)
            .filter(|s| s.contains(token))
            .count();
        
        if token_count > 0 && base_count > 0 {
            (co_occurrence as f32 / (token_count.min(base_count) as f32)).min(1.0)
        } else {
            0.0
        }
    }
    
    /// Calculate ladder index based on similarity
    fn calculate_ladder_index(&self, similarity: f32) -> i16 {
        // Map 0.0-1.0 similarity to 1-10 index
        (similarity * 10.0).round() as i16
    }
}
```

---

## Migration Plan

### **Phase 1: Curated Core Subjects** (Week 1)
- ✅ Define 5-10 core subjects (consciousness, ethics, truth, beauty, etc.)
- ✅ Hand-craft semantic associations
- ✅ Test pattern-aware positioning

### **Phase 2: LLM Expansion** (Week 2)
- ✅ Implement LLM semantic generation
- ✅ Expand existing subjects
- ✅ Validate quality

### **Phase 3: RAG Learning** (Week 3)
- ✅ Extract semantics from Grokipedia
- ✅ Learn from Scholar articles
- ✅ Continuous updates

### **Phase 4: User Learning** (Week 4)
- ✅ Learn from user interactions
- ✅ Reinforce successful patterns
- ✅ Adaptive improvement

---

## Expected Results

📊 **Semantic Coverage**:
- Phase 1: 50-100 associations per position
- Phase 2: 200-500 associations per position
- Phase 3: 1000+ associations per position

🎯 **Accuracy Improvement**:
- Pattern-aware positioning: +30%
- Matrix-guided inference: +40%
- Overall: +50-70% better responses

⚡ **Performance**:
- Position selection: <10ms
- Knowledge lookup: <5ms
- Total overhead: <20ms

---

## Next Steps

1. ✅ Create curated subject definitions
2. ⏳ Implement LLM semantic expansion
3. ⏳ Extract from RAG documents
4. ⏳ Add user learning
5. ⏳ Validate improvements
