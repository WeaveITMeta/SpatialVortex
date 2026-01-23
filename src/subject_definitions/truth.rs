//! Truth Subject Definition with Rich Semantic Associations
//!
//! Defines the FluxMatrix structure for the subject of "truth"
//! following sacred geometry order of operations

use crate::subject_definitions::consciousness::{
    SubjectDefinitionWithSemantics, NodeWithSemantics, SacredWithSemantics
};

/// Get truth subject definition with populated semantics
pub fn definition() -> SubjectDefinitionWithSemantics {
    SubjectDefinitionWithSemantics {
        name: "truth".to_string(),
        nodes: vec![
            // Position 0 - Center (Neutral/Balance)
            NodeWithSemantics {
                position: 0,
                name: "Truth Awareness".to_string(),
                positive: vec![
                    ("recognition", 1, 0.85),
                    ("acknowledgment", 1, 0.8),
                    ("awareness", 1, 0.85),
                    ("clarity", 2, 0.8),
                ],
                negative: vec![
                    ("ignorance", -1, 0.7),
                    ("blindness", -2, 0.65),
                    ("oblivion", -2, 0.6),
                ],
            },
            // Position 1 - Beginning (Self/Identity - ETHOS)
            NodeWithSemantics {
                position: 1,
                name: "Personal Truth".to_string(),
                positive: vec![
                    ("authenticity", 2, 0.95),
                    ("sincerity", 2, 0.9),
                    ("genuineness", 2, 0.85),
                    ("self-honesty", 3, 0.9),
                    ("truthfulness", 2, 0.85),
                ],
                negative: vec![
                    ("deception", -2, 0.8),
                    ("self-delusion", -2, 0.75),
                    ("pretense", -2, 0.7),
                ],
            },
            // Position 2 - Expansion (Growth/Perception)
            NodeWithSemantics {
                position: 2,
                name: "Empirical Truth".to_string(),
                positive: vec![
                    ("observation", 2, 0.9),
                    ("evidence", 2, 0.85),
                    ("facts", 1, 0.8),
                    ("data", 1, 0.75),
                    ("verification", 3, 0.8),
                ],
                negative: vec![
                    ("falsehood", -2, 0.7),
                    ("error", -1, 0.65),
                    ("misperception", -2, 0.6),
                ],
            },
            // Position 4 - Power (Cognition/Reason - LOGOS)
            NodeWithSemantics {
                position: 4,
                name: "Logical Truth".to_string(),
                positive: vec![
                    ("validity", 2, 0.9),
                    ("consistency", 2, 0.85),
                    ("coherence", 2, 0.8),
                    ("deduction", 3, 0.85),
                    ("inference", 2, 0.75),
                ],
                negative: vec![
                    ("contradiction", -2, 0.75),
                    ("inconsistency", -2, 0.7),
                    ("fallacy", -2, 0.8),
                ],
            },
            // Position 5 - Change (Emotion/Dynamics - PATHOS)
            NodeWithSemantics {
                position: 5,
                name: "Felt Truth".to_string(),
                positive: vec![
                    ("intuition", 3, 0.9),
                    ("resonance", 3, 0.85),
                    ("conviction", 2, 0.8),
                    ("certainty", 2, 0.75),
                    ("insight", 4, 0.85),
                ],
                negative: vec![
                    ("doubt", -1, 0.6),
                    ("uncertainty", -1, 0.55),
                    ("confusion", -2, 0.65),
                ],
            },
            // Position 7 - Wisdom (Knowledge/Understanding)
            NodeWithSemantics {
                position: 7,
                name: "Epistemic Truth".to_string(),
                positive: vec![
                    ("knowledge", 3, 0.9),
                    ("justified-belief", 4, 0.85),
                    ("epistemology", 4, 0.8),
                    ("theory-of-knowledge", 5, 0.85),
                    ("warranted-assertion", 4, 0.75),
                ],
                negative: vec![
                    ("false-belief", -2, 0.7),
                    ("unjustified-claim", -3, 0.65),
                ],
            },
            // Position 8 - Mastery (Peak/Excellence)
            NodeWithSemantics {
                position: 8,
                name: "Higher Truth".to_string(),
                positive: vec![
                    ("wisdom", 5, 0.95),
                    ("enlightenment", 6, 0.9),
                    ("revelation", 5, 0.85),
                    ("profound-understanding", 6, 0.9),
                    ("deep-knowledge", 5, 0.85),
                ],
                negative: vec![
                    ("superficiality", -2, 0.7),
                    ("shallowness", -2, 0.65),
                    ("misunderstanding", -2, 0.7),
                ],
            },
        ],
        sacred_guides: vec![
            // Position 3 - Sacred Ethos (Unity/Integration)
            SacredWithSemantics {
                position: 3,
                name: "Truth Unity".to_string(),
                divine_properties: vec![
                    ("coherent-truth", 0.95),
                    ("integrated-reality", 0.93),
                    ("unified-understanding", 0.9),
                    ("whole-truth", 0.92),
                    ("complete-picture", 0.88),
                    ("unifies", 0.92),
                    ("integrates-truth", 0.93),
                    ("brings-together", 0.89),
                    ("unified-reality", 0.91),
                    ("complete-truth", 0.9),
                ],
            },
            // Position 6 - Sacred Pathos (Emotional Core)
            SacredWithSemantics {
                position: 6,
                name: "Truth Heart".to_string(),
                divine_properties: vec![
                    ("truth-felt-deeply", 0.95),
                    ("authentic-knowing", 0.93),
                    ("heartfelt-truth", 0.92),
                    ("emotional-honesty", 0.9),
                    ("sincere-understanding", 0.91),
                    ("felt-truth", 0.96),      // Direct match for query
                    ("heart-of", 0.94),
                    ("core-of", 0.93),
                    ("feeling-truth", 0.92),
                    ("truth-in-heart", 0.91),
                ],
            },
            // Position 9 - Sacred Logos (Divine/Ultimate)
            SacredWithSemantics {
                position: 9,
                name: "Ultimate Truth".to_string(),
                divine_properties: vec![
                    ("absolute-truth", 0.99),
                    ("universal-reality", 0.97),
                    ("fundamental-truth", 0.96),
                    ("eternal-verity", 0.95),
                    ("divine-truth", 0.98),
                    ("ultimate-reality", 0.97),
                    ("essence-of-truth", 0.96),
                    ("nature-of-reality", 0.97),
                    ("ontological-truth", 0.94),
                ],
            },
        ],
    }
}
