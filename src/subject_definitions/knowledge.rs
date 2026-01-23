//! Knowledge Subject Definition - Epistemology and understanding

use crate::subject_definitions::consciousness::{
    SubjectDefinitionWithSemantics, NodeWithSemantics, SacredWithSemantics
};

pub fn definition() -> SubjectDefinitionWithSemantics {
    SubjectDefinitionWithSemantics {
        name: "knowledge".to_string(),
        nodes: vec![
            NodeWithSemantics {
                position: 0,
                name: "Knowledge Awareness".to_string(),
                positive: vec![("knowing", 1, 0.9), ("knowledge", 1, 0.85), ("truth", 2, 0.75), ("cognition", 2, 0.7)],
                negative: vec![("ignorance", -1, 0.7)],
            },
            NodeWithSemantics {
                position: 1,
                name: "Personal Knowledge".to_string(),
                positive: vec![("belief", 2, 0.95), ("understanding", 2, 0.9), ("personal-truth", 3, 0.85)],
                negative: vec![("false-belief", -2, 0.75)],
            },
            NodeWithSemantics {
                position: 2,
                name: "Empirical Knowledge".to_string(),
                positive: vec![("observation", 2, 0.9), ("experience", 2, 0.85), ("evidence-based", 3, 0.8), ("perception", 2, 0.75)],
                negative: vec![],
            },
            NodeWithSemantics {
                position: 4,
                name: "Rational Knowledge".to_string(),
                positive: vec![("a-priori", 3, 0.9), ("deductive-knowledge", 3, 0.85), ("logical-truth", 2, 0.8), ("reasoning", 3, 0.8)],
                negative: vec![],
            },
            NodeWithSemantics {
                position: 5,
                name: "Intuitive Knowledge".to_string(),
                positive: vec![("tacit-knowledge", 4, 0.95), ("know-how", 3, 0.9), ("practical-wisdom", 4, 0.85)],
                negative: vec![],
            },
            NodeWithSemantics {
                position: 7,
                name: "Epistemology".to_string(),
                positive: vec![("theory-of-knowledge", 4, 0.9), ("justified-belief", 4, 0.85), ("warrant", 3, 0.8), ("cognition", 3, 0.75)],
                negative: vec![],
            },
            NodeWithSemantics {
                position: 8,
                name: "Wisdom".to_string(),
                positive: vec![("deep-knowledge", 5, 0.95), ("profound-understanding", 6, 0.9), ("mastery", 5, 0.85)],
                negative: vec![],
            },
        ],
        sacred_guides: vec![
            SacredWithSemantics {
                position: 3,
                name: "Knowledge Unity".to_string(),
                divine_properties: vec![("integrated-knowledge", 0.96), ("unified-understanding", 0.94), ("coherent-truth", 0.92), ("integrates", 0.92), ("unifies", 0.91)],
            },
            SacredWithSemantics {
                position: 6,
                name: "Knowledge Heart".to_string(),
                divine_properties: vec![("felt-knowledge", 0.95), ("intuitive-knowing", 0.94), ("heart-of", 0.95), ("core-of", 0.93)],
            },
            SacredWithSemantics {
                position: 9,
                name: "Ultimate Knowledge".to_string(),
                divine_properties: vec![("absolute-knowledge", 0.98), ("omniscience", 0.97), ("fundamental-knowing", 0.96), ("essence-of-knowledge", 0.96), ("ultimate", 0.94)],
            },
        ],
    }
}
