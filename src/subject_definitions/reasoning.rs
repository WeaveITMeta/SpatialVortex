//! Reasoning Subject Definition - Problem-solving and logical thinking

use crate::subject_definitions::consciousness::{
    SubjectDefinitionWithSemantics, NodeWithSemantics, SacredWithSemantics
};

pub fn definition() -> SubjectDefinitionWithSemantics {
    SubjectDefinitionWithSemantics {
        name: "reasoning".to_string(),
        nodes: vec![
            NodeWithSemantics {
                position: 0,
                name: "Reasoning Awareness".to_string(),
                positive: vec![("reason", 1, 0.9), ("logical-thinking", 2, 0.85), ("rationality", 2, 0.8), ("cognition", 2, 0.75), ("inference", 2, 0.75)],
                negative: vec![("irrationality", -2, 0.7)],
            },
            NodeWithSemantics {
                position: 1,
                name: "Personal Reasoning".to_string(),
                positive: vec![("self-reasoning", 2, 0.95), ("personal-logic", 2, 0.9), ("individual-judgment", 3, 0.85)],
                negative: vec![],
            },
            NodeWithSemantics {
                position: 2,
                name: "Practical Reasoning".to_string(),
                positive: vec![("problem-solving", 3, 0.95), ("decision-making", 3, 0.9), ("applied-reasoning", 3, 0.85)],
                negative: vec![],
            },
            NodeWithSemantics {
                position: 4,
                name: "Logical Reasoning".to_string(),
                positive: vec![("deduction", 3, 0.95), ("induction", 3, 0.9), ("formal-logic", 3, 0.85), ("inference", 3, 0.85), ("cognition", 3, 0.8)],
                negative: vec![("fallacy", -2, 0.8)],
            },
            NodeWithSemantics {
                position: 5,
                name: "Creative Reasoning".to_string(),
                positive: vec![("abduction", 4, 0.95), ("creative-problem-solving", 4, 0.9), ("innovative-thinking", 4, 0.85), ("insight", 3, 0.8)],
                negative: vec![],
            },
            NodeWithSemantics {
                position: 7,
                name: "Theory of Reasoning".to_string(),
                positive: vec![("logic-theory", 4, 0.9), ("argumentation-theory", 4, 0.85), ("critical-thinking", 4, 0.8), ("wisdom", 3, 0.75)],
                negative: vec![],
            },
            NodeWithSemantics {
                position: 8,
                name: "Reasoning Mastery".to_string(),
                positive: vec![("perfect-logic", 6, 0.95), ("flawless-reasoning", 6, 0.9), ("reasoning-excellence", 5, 0.85)],
                negative: vec![],
            },
        ],
        sacred_guides: vec![
            SacredWithSemantics {
                position: 3,
                name: "Reasoning Unity".to_string(),
                divine_properties: vec![("integrated-reasoning", 0.96), ("unified-logic", 0.94), ("coherent-thinking", 0.93), ("integrates", 0.92), ("unifies", 0.91)],
            },
            SacredWithSemantics {
                position: 6,
                name: "Reasoning Heart".to_string(),
                divine_properties: vec![("compassionate-reasoning", 0.96), ("ethical-logic", 0.95), ("heart-logic", 0.94), ("heart-of", 0.95), ("core-of", 0.93)],
            },
            SacredWithSemantics {
                position: 9,
                name: "Ultimate Reasoning".to_string(),
                divine_properties: vec![("perfect-rationality", 0.98), ("absolute-logic", 0.97), ("divine-reason", 0.96), ("essence-of-reasoning", 0.96), ("ultimate", 0.94)],
            },
        ],
    }
}
