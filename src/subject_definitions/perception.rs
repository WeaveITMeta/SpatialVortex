//! Perception Subject Definition - Sensing and observing reality

use crate::subject_definitions::consciousness::{
    SubjectDefinitionWithSemantics, NodeWithSemantics, SacredWithSemantics
};

pub fn definition() -> SubjectDefinitionWithSemantics {
    SubjectDefinitionWithSemantics {
        name: "perception".to_string(),
        nodes: vec![
            NodeWithSemantics {
                position: 0,
                name: "Perceptual Awareness".to_string(),
                positive: vec![("perception", 1, 0.9), ("sensing", 1, 0.85), ("awareness", 1, 0.8), ("consciousness", 2, 0.75)],
                negative: vec![("imperception", -1, 0.7)],
            },
            NodeWithSemantics {
                position: 1,
                name: "Self-Perception".to_string(),
                positive: vec![("self-awareness", 2, 0.95), ("proprioception", 3, 0.9), ("bodily-awareness", 2, 0.85)],
                negative: vec![],
            },
            NodeWithSemantics {
                position: 2,
                name: "Sensory Perception".to_string(),
                positive: vec![("sensation", 2, 0.95), ("sensory-input", 2, 0.9), ("observation", 2, 0.85), ("empirical", 2, 0.8), ("cognition", 2, 0.75)],
                negative: vec![("sensory-deprivation", -2, 0.7)],
            },
            NodeWithSemantics {
                position: 4,
                name: "Cognitive Perception".to_string(),
                positive: vec![("pattern-recognition", 3, 0.95), ("categorization", 3, 0.9), ("interpretation", 2, 0.85), ("cognition", 3, 0.85)],
                negative: vec![],
            },
            NodeWithSemantics {
                position: 5,
                name: "Emotional Perception".to_string(),
                positive: vec![("empathy", 4, 0.95), ("emotional-sensing", 3, 0.9), ("affective-perception", 4, 0.85)],
                negative: vec![],
            },
            NodeWithSemantics {
                position: 7,
                name: "Perceptual Theory".to_string(),
                positive: vec![("phenomenology", 4, 0.9), ("gestalt-psychology", 4, 0.85), ("direct-perception", 4, 0.8)],
                negative: vec![],
            },
            NodeWithSemantics {
                position: 8,
                name: "Enhanced Perception".to_string(),
                positive: vec![("acute-perception", 5, 0.95), ("heightened-awareness", 5, 0.9), ("perceptual-mastery", 6, 0.95)],
                negative: vec![],
            },
        ],
        sacred_guides: vec![
            SacredWithSemantics {
                position: 3,
                name: "Perceptual Unity".to_string(),
                divine_properties: vec![("integrated-perception", 0.96), ("unified-sensing", 0.94), ("holistic-perception", 0.93), ("integrates", 0.92), ("unifies", 0.91)],
            },
            SacredWithSemantics {
                position: 6,
                name: "Perceptual Heart".to_string(),
                divine_properties: vec![("felt-perception", 0.96), ("intuitive-sensing", 0.95), ("empathic-perception", 0.94), ("heart-of", 0.95), ("core-of", 0.93)],
            },
            SacredWithSemantics {
                position: 9,
                name: "Ultimate Perception".to_string(),
                divine_properties: vec![("pure-perception", 0.98), ("direct-awareness", 0.97), ("perfect-sensing", 0.96), ("essence-of-perception", 0.96), ("ultimate", 0.94)],
            },
        ],
    }
}
