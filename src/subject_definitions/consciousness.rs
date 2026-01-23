//! Consciousness Subject Definition with Rich Semantic Associations
//!
//! Defines the FluxMatrix structure for the subject of "consciousness"
//! with detailed semantic associations for pattern-aware positioning

/// Get consciousness subject definition with populated semantics
pub fn definition() -> SubjectDefinitionWithSemantics {
    SubjectDefinitionWithSemantics {
        name: "consciousness".to_string(),
        nodes: vec![
            // Position 0 - Center (Neutral/Balance)
            NodeWithSemantics {
                position: 0,
                name: "Awareness".to_string(),
                positive: vec![
                    ("presence", 1, 0.9),
                    ("mindfulness", 2, 0.85),
                    ("attention", 1, 0.8),
                    ("alertness", 1, 0.75),
                ],
                negative: vec![
                    ("unconscious", -1, 0.7),
                    ("oblivious", -2, 0.6),
                    ("unaware", -1, 0.65),
                ],
            },
            // Position 1 - Start (Beginning/Self)
            NodeWithSemantics {
                position: 1,
                name: "Self-Awareness".to_string(),
                positive: vec![
                    ("identity", 1, 0.9),
                    ("reflection", 2, 0.85),
                    ("introspection", 3, 0.8),
                    ("self-knowledge", 4, 0.75),
                    ("mirror", 2, 0.7),
                ],
                negative: vec![
                    ("ignorance", -1, 0.7),
                    ("delusion", -2, 0.6),
                    ("denial", -1, 0.65),
                ],
            },
            // Position 2 - Expansion (Growth)
            NodeWithSemantics {
                position: 2,
                name: "Perception".to_string(),
                positive: vec![
                    ("senses", 1, 0.9),
                    ("observation", 2, 0.85),
                    ("experience", 1, 0.8),
                    ("qualia", 3, 0.75),
                    ("sensation", 1, 0.7),
                ],
                negative: vec![
                    ("blindness", -1, 0.7),
                    ("numbness", -2, 0.6),
                    ("insensitivity", -1, 0.65),
                ],
            },
            // Position 4 - Power (Mental Capability)
            NodeWithSemantics {
                position: 4,
                name: "Cognition".to_string(),
                positive: vec![
                    ("thinking", 1, 0.9),
                    ("reasoning", 2, 0.85),
                    ("intelligence", 3, 0.8),
                    ("understanding", 2, 0.75),
                    ("analysis", 2, 0.7),
                ],
                negative: vec![
                    ("stupidity", -1, 0.7),
                    ("confusion", -2, 0.6),
                    ("ignorance", -1, 0.65),
                ],
            },
            // Position 5 - Change (Emotional Dynamics)
            NodeWithSemantics {
                position: 5,
                name: "Emotion".to_string(),
                positive: vec![
                    ("feeling", 1, 0.9),
                    ("empathy", 3, 0.85),
                    ("compassion", 4, 0.8),
                    ("love", 5, 0.75),
                    ("joy", 3, 0.7),
                ],
                negative: vec![
                    ("apathy", -1, 0.7),
                    ("hatred", -3, 0.6),
                    ("coldness", -2, 0.65),
                ],
            },
            // Position 7 - Wisdom (Knowledge)
            NodeWithSemantics {
                position: 7,
                name: "Consciousness Studies".to_string(),
                positive: vec![
                    ("neuroscience", 2, 0.9),
                    ("philosophy", 3, 0.85),
                    ("meditation", 4, 0.8),
                    ("mysticism", 5, 0.75),
                    ("psychology", 2, 0.7),
                ],
                negative: vec![
                    ("materialism", -1, 0.6),
                    ("reductionism", -2, 0.5),
                ],
            },
            // Position 8 - Mastery (Peak States)
            NodeWithSemantics {
                position: 8,
                name: "Higher Consciousness".to_string(),
                positive: vec![
                    ("enlightenment", 5, 0.95),
                    ("transcendence", 6, 0.9),
                    ("awakening", 5, 0.85),
                    ("nirvana", 7, 0.8),
                    ("samadhi", 6, 0.75),
                ],
                negative: vec![
                    ("delusion", -2, 0.7),
                    ("maya", -1, 0.6),
                    ("illusion", -2, 0.65),
                ],
            },
        ],
        sacred_guides: vec![
            // Position 3 - Sacred Ethos (Unity of Self)
            SacredWithSemantics {
                position: 3,
                name: "Unity of Self".to_string(),
                divine_properties: vec![
                    ("integration", 0.95),
                    ("wholeness", 0.9),
                    ("coherence", 0.85),
                    ("unity", 0.9),
                    ("self-realization", 0.88),
                    ("integrates", 0.93),  // Verb form
                    ("integrate", 0.93),   // Base verb
                    ("unifies", 0.92),
                    ("unify", 0.92),
                    ("brings-together", 0.88),
                    ("unified-self", 0.91),
                ],
            },
            // Position 6 - Sacred Pathos (Emotional Core)
            SacredWithSemantics {
                position: 6,
                name: "Emotional Core".to_string(),
                divine_properties: vec![
                    ("heart-mind", 0.95),
                    ("feeling-center", 0.9),
                    ("empathic-bond", 0.85),
                    ("emotional-intelligence", 0.88),
                    ("compassion-core", 0.87),
                    ("heart-of", 0.94),      // Phrase match
                    ("core-of", 0.93),       // Phrase match
                    ("center-of", 0.92),
                    ("soul-of", 0.91),
                    ("essence-of-feeling", 0.89),
                    ("emotional-heart", 0.9),
                ],
            },
            // Position 9 - Sacred Logos (Divine Mind)
            SacredWithSemantics {
                position: 9,
                name: "Divine Mind".to_string(),
                divine_properties: vec![
                    ("cosmic-consciousness", 0.98),
                    ("universal-awareness", 0.95),
                    ("absolute-knowing", 0.9),
                    ("infinite-intelligence", 0.93),
                    ("supreme-wisdom", 0.91),
                    ("nature-of-consciousness", 0.96),  // Match "nature of" queries
                    ("fundamental-principle", 0.94),
                    ("essence-of-being", 0.92),
                    ("ultimate-reality", 0.93),
                ],
            },
        ],
    }
}

/// Subject definition with semantic associations
#[derive(Debug, Clone)]
pub struct SubjectDefinitionWithSemantics {
    pub name: String,
    pub nodes: Vec<NodeWithSemantics>,
    pub sacred_guides: Vec<SacredWithSemantics>,
}

/// Node definition with semantic data
#[derive(Debug, Clone)]
pub struct NodeWithSemantics {
    pub position: u8,
    pub name: String,
    pub positive: Vec<(&'static str, i16, f64)>,  // (word, index, confidence)
    pub negative: Vec<(&'static str, i16, f64)>,
}

/// Sacred guide definition with semantic data
#[derive(Debug, Clone)]
pub struct SacredWithSemantics {
    pub position: u8,
    pub name: String,
    pub divine_properties: Vec<(&'static str, f64)>,  // (property, confidence)
}
