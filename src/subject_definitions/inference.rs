//! Inference Subject Definition
//!
//! Reasoning, deduction, and drawing conclusions with cross-references

use crate::subject_definitions::consciousness::{
    SubjectDefinitionWithSemantics, NodeWithSemantics, SacredWithSemantics
};

pub fn definition() -> SubjectDefinitionWithSemantics {
    SubjectDefinitionWithSemantics {
        name: "inference".to_string(),
        nodes: vec![
            // Position 0 - CENTER
            NodeWithSemantics {
                position: 0,
                name: "Inferential Awareness".to_string(),
                positive: vec![
                    ("inference", 1, 0.9),
                    ("reasoning-process", 2, 0.85),
                    ("drawing-conclusions", 2, 0.8),
                    // Cross-references
                    ("cognition", 2, 0.8),
                    ("logic", 2, 0.75),
                    ("truth", 2, 0.7),
                ],
                negative: vec![
                    ("non-sequitur", -2, 0.75),
                    ("invalid-inference", -2, 0.7),
                ],
            },
            
            // Position 1 - BEGINNING (Ethos)
            NodeWithSemantics {
                position: 1,
                name: "Personal Inference".to_string(),
                positive: vec![
                    ("judgment", 2, 0.95),
                    ("belief-formation", 3, 0.9),
                    ("personal-reasoning", 2, 0.85),
                    ("conclusion", 2, 0.8),
                ],
                negative: vec![
                    ("prejudgment", -2, 0.75),
                    ("hasty-conclusion", -2, 0.7),
                ],
            },
            
            // Position 2 - EXPANSION
            NodeWithSemantics {
                position: 2,
                name: "Empirical Inference".to_string(),
                positive: vec![
                    ("induction", 3, 0.9),
                    ("observation-based", 3, 0.85),
                    ("data-driven", 2, 0.8),
                    ("evidence", 2, 0.85),
                ],
                negative: vec![
                    ("baseless-inference", -2, 0.7),
                ],
            },
            
            // Position 4 - POWER (Logos)
            NodeWithSemantics {
                position: 4,
                name: "Logical Inference".to_string(),
                positive: vec![
                    ("deduction", 3, 0.95),
                    ("logical-reasoning", 3, 0.9),
                    ("entailment", 3, 0.85),
                    ("implication", 2, 0.8),
                    ("syllogism", 3, 0.85),
                    // Cross-reference
                    ("cognition", 3, 0.8),
                ],
                negative: vec![
                    ("fallacy", -2, 0.8),
                    ("invalid-logic", -3, 0.75),
                ],
            },
            
            // Position 5 - CHANGE (Pathos)
            NodeWithSemantics {
                position: 5,
                name: "Intuitive Inference".to_string(),
                positive: vec![
                    ("abduction", 3, 0.95),
                    ("best-explanation", 4, 0.9),
                    ("hypothesis", 3, 0.85),
                    ("intuitive-leap", 3, 0.8),
                    ("insight-based", 3, 0.85),
                ],
                negative: vec![
                    ("unfounded-guess", -2, 0.7),
                ],
            },
            
            // Position 7 - WISDOM
            NodeWithSemantics {
                position: 7,
                name: "Inferential Theory".to_string(),
                positive: vec![
                    ("bayesian-inference", 4, 0.9),
                    ("probabilistic-reasoning", 4, 0.85),
                    ("inference-theory", 4, 0.8),
                    ("epistemic-logic", 5, 0.85),
                ],
                negative: vec![],
            },
            
            // Position 8 - MASTERY
            NodeWithSemantics {
                position: 8,
                name: "Inferential Excellence".to_string(),
                positive: vec![
                    ("sound-reasoning", 5, 0.95),
                    ("valid-inference", 5, 0.9),
                    ("masterful-deduction", 6, 0.95),
                    ("perfect-reasoning", 6, 0.9),
                ],
                negative: vec![
                    ("flawed-reasoning", -3, 0.75),
                ],
            },
        ],
        sacred_guides: vec![
            // Position 3 - SACRED ETHOS
            SacredWithSemantics {
                position: 3,
                name: "Inferential Unity".to_string(),
                divine_properties: vec![
                    ("integrated-reasoning", 0.96),
                    ("unified-logic", 0.94),
                    ("coherent-inference", 0.95),
                    ("complete-reasoning", 0.93),
                    ("integrates", 0.92),
                    ("unifies", 0.91),
                ],
            },
            
            // Position 6 - SACRED PATHOS
            SacredWithSemantics {
                position: 6,
                name: "Inferential Heart".to_string(),
                divine_properties: vec![
                    ("felt-reasoning", 0.95),
                    ("intuitive-inference", 0.94),
                    ("heart-logic", 0.93),
                    ("heart-of", 0.95),
                    ("core-of", 0.93),
                ],
            },
            
            // Position 9 - SACRED LOGOS
            SacredWithSemantics {
                position: 9,
                name: "Ultimate Inference".to_string(),
                divine_properties: vec![
                    ("pure-logic", 0.98),
                    ("absolute-inference", 0.97),
                    ("fundamental-reasoning", 0.96),
                    ("essence-of-inference", 0.96),
                    ("nature-of-reasoning", 0.97),
                    ("perfect-logic", 0.95),
                    ("ultimate", 0.94),
                    ("fundamental", 0.93),
                ],
            },
        ],
    }
}
