//! Cognition Subject Definition
//!
//! Thinking, reasoning, and mental processing with cross-references

use crate::subject_definitions::consciousness::{
    SubjectDefinitionWithSemantics, NodeWithSemantics, SacredWithSemantics
};

pub fn definition() -> SubjectDefinitionWithSemantics {
    SubjectDefinitionWithSemantics {
        name: "cognition".to_string(),
        nodes: vec![
            // Position 0 - CENTER
            NodeWithSemantics {
                position: 0,
                name: "Cognitive Awareness".to_string(),
                positive: vec![
                    ("mental-activity", 1, 0.9),
                    ("cognition", 1, 0.85),
                    ("thinking-process", 2, 0.8),
                    // Cross-references
                    ("consciousness", 2, 0.75),
                    ("psychology", 2, 0.7),
                    ("inference", 2, 0.75),
                ],
                negative: vec![
                    ("cognitive-blindness", -2, 0.7),
                ],
            },
            
            // Position 1 - BEGINNING (Ethos)
            NodeWithSemantics {
                position: 1,
                name: "Personal Cognition".to_string(),
                positive: vec![
                    ("self-reflection", 2, 0.95),
                    ("metacognition", 3, 0.9),
                    ("thinking-about-thinking", 3, 0.85),
                    ("introspection", 2, 0.8),
                ],
                negative: vec![
                    ("unreflective", -2, 0.7),
                ],
            },
            
            // Position 2 - EXPANSION
            NodeWithSemantics {
                position: 2,
                name: "Perceptual Cognition".to_string(),
                positive: vec![
                    ("perception", 2, 0.9),
                    ("sensation", 2, 0.85),
                    ("pattern-recognition", 3, 0.8),
                    ("observation", 2, 0.75),
                ],
                negative: vec![
                    ("misperception", -2, 0.7),
                ],
            },
            
            // Position 4 - POWER (Logos)
            NodeWithSemantics {
                position: 4,
                name: "Logical Cognition".to_string(),
                positive: vec![
                    ("reasoning", 2, 0.95),
                    ("logic", 2, 0.9),
                    ("analysis", 2, 0.85),
                    ("deduction", 3, 0.8),
                    ("critical-thinking", 3, 0.85),
                    // Cross-reference
                    ("inference", 3, 0.85),
                ],
                negative: vec![
                    ("illogical", -2, 0.75),
                    ("fallacious", -2, 0.7),
                ],
            },
            
            // Position 5 - CHANGE (Pathos)
            NodeWithSemantics {
                position: 5,
                name: "Intuitive Cognition".to_string(),
                positive: vec![
                    ("intuition", 3, 0.95),
                    ("insight", 3, 0.9),
                    ("heuristic", 3, 0.85),
                    ("gut-feeling", 2, 0.8),
                    ("tacit-knowledge", 4, 0.85),
                ],
                negative: vec![
                    ("cognitive-rigidity", -2, 0.7),
                ],
            },
            
            // Position 7 - WISDOM
            NodeWithSemantics {
                position: 7,
                name: "Cognitive Science".to_string(),
                positive: vec![
                    ("cognitive-theory", 3, 0.9),
                    ("information-processing", 4, 0.85),
                    ("mental-models", 3, 0.8),
                    ("schema", 3, 0.75),
                    ("computational-mind", 4, 0.8),
                ],
                negative: vec![],
            },
            
            // Position 8 - MASTERY
            NodeWithSemantics {
                position: 8,
                name: "Cognitive Excellence".to_string(),
                positive: vec![
                    ("wisdom", 5, 0.95),
                    ("expert-cognition", 5, 0.9),
                    ("cognitive-mastery", 6, 0.95),
                    ("brilliant-thinking", 5, 0.85),
                    ("intellectual-peak", 6, 0.9),
                ],
                negative: vec![
                    ("cognitive-decline", -3, 0.75),
                ],
            },
        ],
        sacred_guides: vec![
            // Position 3 - SACRED ETHOS
            SacredWithSemantics {
                position: 3,
                name: "Cognitive Unity".to_string(),
                divine_properties: vec![
                    ("integrated-cognition", 0.96),
                    ("unified-thinking", 0.94),
                    ("coherent-thought", 0.92),
                    ("cognitive-integration", 0.95),
                    ("integrates", 0.92),
                    ("unifies", 0.91),
                ],
            },
            
            // Position 6 - SACRED PATHOS
            SacredWithSemantics {
                position: 6,
                name: "Cognitive Heart".to_string(),
                divine_properties: vec![
                    ("thinking-with-feeling", 0.95),
                    ("emotional-intelligence", 0.94),
                    ("heart-mind-unity", 0.93),
                    ("heart-of", 0.95),
                    ("core-of", 0.93),
                    ("felt-cognition", 0.92),
                ],
            },
            
            // Position 9 - SACRED LOGOS
            SacredWithSemantics {
                position: 9,
                name: "Ultimate Cognition".to_string(),
                divine_properties: vec![
                    ("universal-mind", 0.98),
                    ("absolute-cognition", 0.97),
                    ("fundamental-thinking", 0.96),
                    ("essence-of-cognition", 0.96),
                    ("nature-of-thought", 0.97),
                    ("pure-reason", 0.95),
                    ("ultimate", 0.94),
                    ("fundamental", 0.93),
                ],
            },
        ],
    }
}
