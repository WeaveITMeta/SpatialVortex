//! Wisdom Subject Definition - Practical judgment and deep understanding

use crate::subject_definitions::consciousness::{
    SubjectDefinitionWithSemantics, NodeWithSemantics, SacredWithSemantics
};

pub fn definition() -> SubjectDefinitionWithSemantics {
    SubjectDefinitionWithSemantics {
        name: "wisdom".to_string(),
        nodes: vec![
            NodeWithSemantics {
                position: 0,
                name: "Wisdom Awareness".to_string(),
                positive: vec![("wise", 1, 0.9), ("sagacity", 2, 0.85), ("discernment", 2, 0.8), ("knowledge", 2, 0.75)],
                negative: vec![("foolishness", -2, 0.7)],
            },
            NodeWithSemantics {
                position: 1,
                name: "Personal Wisdom".to_string(),
                positive: vec![("self-knowledge", 3, 0.95), ("maturity", 2, 0.9), ("character-wisdom", 3, 0.85)],
                negative: vec![],
            },
            NodeWithSemantics {
                position: 2,
                name: "Practical Wisdom".to_string(),
                positive: vec![("phronesis", 3, 0.95), ("practical-judgment", 3, 0.9), ("applied-knowledge", 3, 0.85), ("reasoning", 2, 0.75)],
                negative: vec![],
            },
            NodeWithSemantics {
                position: 4,
                name: "Theoretical Wisdom".to_string(),
                positive: vec![("sophia", 4, 0.95), ("philosophical-wisdom", 4, 0.9), ("contemplation", 3, 0.85)],
                negative: vec![],
            },
            NodeWithSemantics {
                position: 5,
                name: "Emotional Wisdom".to_string(),
                positive: vec![("emotional-intelligence", 4, 0.95), ("empathic-understanding", 4, 0.9), ("compassionate-wisdom", 4, 0.85)],
                negative: vec![],
            },
            NodeWithSemantics {
                position: 7,
                name: "Philosophy of Wisdom".to_string(),
                positive: vec![("epistemology", 4, 0.85), ("virtue-theory", 4, 0.8), ("ethics", 3, 0.8)],
                negative: vec![],
            },
            NodeWithSemantics {
                position: 8,
                name: "Sage Wisdom".to_string(),
                positive: vec![("enlightened-understanding", 6, 0.95), ("profound-wisdom", 6, 0.9), ("mastery-of-life", 5, 0.85)],
                negative: vec![],
            },
        ],
        sacred_guides: vec![
            SacredWithSemantics {
                position: 3,
                name: "Wisdom Unity".to_string(),
                divine_properties: vec![("integrated-wisdom", 0.96), ("unified-understanding", 0.94), ("holistic-wisdom", 0.93), ("integrates", 0.92), ("unifies", 0.91)],
            },
            SacredWithSemantics {
                position: 6,
                name: "Wisdom Heart".to_string(),
                divine_properties: vec![("compassionate-wisdom", 0.96), ("heart-wisdom", 0.95), ("wise-compassion", 0.94), ("heart-of", 0.95), ("core-of", 0.93)],
            },
            SacredWithSemantics {
                position: 9,
                name: "Ultimate Wisdom".to_string(),
                divine_properties: vec![("divine-wisdom", 0.98), ("absolute-understanding", 0.97), ("perfect-wisdom", 0.96), ("essence-of-wisdom", 0.96), ("ultimate", 0.94)],
            },
        ],
    }
}
