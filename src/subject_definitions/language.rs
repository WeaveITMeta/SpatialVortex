//! Language Subject Definition - Communication and meaning

use crate::subject_definitions::consciousness::{
    SubjectDefinitionWithSemantics, NodeWithSemantics, SacredWithSemantics
};

pub fn definition() -> SubjectDefinitionWithSemantics {
    SubjectDefinitionWithSemantics {
        name: "language".to_string(),
        nodes: vec![
            NodeWithSemantics {
                position: 0,
                name: "Linguistic Awareness".to_string(),
                positive: vec![("language", 1, 0.9), ("communication", 1, 0.85), ("meaning", 2, 0.8), ("cognition", 2, 0.75)],
                negative: vec![("aphasia", -2, 0.7)],
            },
            NodeWithSemantics {
                position: 1,
                name: "Personal Language".to_string(),
                positive: vec![("self-expression", 2, 0.95), ("inner-speech", 3, 0.9), ("personal-voice", 2, 0.85)],
                negative: vec![],
            },
            NodeWithSemantics {
                position: 2,
                name: "Social Language".to_string(),
                positive: vec![("conversation", 2, 0.95), ("dialogue", 2, 0.9), ("interpersonal-communication", 3, 0.85)],
                negative: vec![],
            },
            NodeWithSemantics {
                position: 4,
                name: "Logical Language".to_string(),
                positive: vec![("syntax", 3, 0.95), ("grammar", 2, 0.9), ("formal-language", 3, 0.85), ("semantics", 3, 0.8), ("reasoning", 3, 0.8)],
                negative: vec![],
            },
            NodeWithSemantics {
                position: 5,
                name: "Expressive Language".to_string(),
                positive: vec![("metaphor", 3, 0.95), ("poetry", 3, 0.9), ("emotional-expression", 4, 0.85), ("rhetoric", 3, 0.8)],
                negative: vec![],
            },
            NodeWithSemantics {
                position: 7,
                name: "Linguistic Theory".to_string(),
                positive: vec![("linguistics", 4, 0.9), ("semiotics", 4, 0.85), ("pragmatics", 4, 0.8), ("philosophy-of-language", 5, 0.85)],
                negative: vec![],
            },
            NodeWithSemantics {
                position: 8,
                name: "Linguistic Mastery".to_string(),
                positive: vec![("eloquence", 5, 0.95), ("linguistic-excellence", 6, 0.9), ("perfect-expression", 5, 0.85)],
                negative: vec![],
            },
        ],
        sacred_guides: vec![
            SacredWithSemantics {
                position: 3,
                name: "Linguistic Unity".to_string(),
                divine_properties: vec![("unified-language", 0.96), ("integrated-meaning", 0.94), ("coherent-communication", 0.93), ("integrates", 0.92), ("unifies", 0.91)],
            },
            SacredWithSemantics {
                position: 6,
                name: "Linguistic Heart".to_string(),
                divine_properties: vec![("heartfelt-speech", 0.96), ("authentic-expression", 0.95), ("emotional-language", 0.94), ("heart-of", 0.95), ("core-of", 0.93)],
            },
            SacredWithSemantics {
                position: 9,
                name: "Ultimate Language".to_string(),
                divine_properties: vec![("perfect-language", 0.98), ("universal-communication", 0.97), ("absolute-meaning", 0.96), ("essence-of-language", 0.96), ("ultimate", 0.94)],
            },
        ],
    }
}
