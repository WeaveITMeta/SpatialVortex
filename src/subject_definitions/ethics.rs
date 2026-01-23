//! Ethics Subject Definition with Rich Semantic Associations
//!
//! Defines the FluxMatrix structure for the subject of "ethics"
//! following sacred geometry order of operations

use crate::subject_definitions::consciousness::{
    SubjectDefinitionWithSemantics, NodeWithSemantics, SacredWithSemantics
};

/// Get ethics subject definition with populated semantics
pub fn definition() -> SubjectDefinitionWithSemantics {
    SubjectDefinitionWithSemantics {
        name: "ethics".to_string(),
        nodes: vec![
            // Position 0 - Center (Neutral/Balance)
            NodeWithSemantics {
                position: 0,
                name: "Moral Awareness".to_string(),
                positive: vec![
                    ("conscience", 1, 0.9),
                    ("awareness", 1, 0.85),
                    ("moral-sense", 2, 0.8),
                    ("ethical-consciousness", 2, 0.75),
                ],
                negative: vec![
                    ("amoral", -1, 0.7),
                    ("unconscious", -1, 0.65),
                    ("indifferent", -2, 0.6),
                ],
            },
            // Position 1 - Beginning (Self/Identity - ETHOS)
            NodeWithSemantics {
                position: 1,
                name: "Personal Ethics".to_string(),
                positive: vec![
                    ("integrity", 2, 0.95),
                    ("character", 2, 0.9),
                    ("virtue", 3, 0.85),
                    ("self-discipline", 2, 0.8),
                    ("honesty", 2, 0.85),
                ],
                negative: vec![
                    ("corruption", -2, 0.8),
                    ("dishonesty", -2, 0.75),
                    ("hypocrisy", -2, 0.7),
                ],
            },
            // Position 2 - Expansion (Growth/Perception)
            NodeWithSemantics {
                position: 2,
                name: "Social Ethics".to_string(),
                positive: vec![
                    ("responsibility", 2, 0.9),
                    ("fairness", 2, 0.85),
                    ("respect", 1, 0.8),
                    ("cooperation", 2, 0.75),
                    ("reciprocity", 3, 0.7),
                ],
                negative: vec![
                    ("irresponsibility", -2, 0.7),
                    ("unfairness", -2, 0.65),
                    ("disrespect", -1, 0.6),
                ],
            },
            // Position 4 - Power (Cognition/Reason - LOGOS)
            NodeWithSemantics {
                position: 4,
                name: "Moral Reasoning".to_string(),
                positive: vec![
                    ("rationality", 2, 0.9),
                    ("deliberation", 3, 0.85),
                    ("judgment", 2, 0.8),
                    ("principle", 3, 0.85),
                    ("logic", 2, 0.75),
                ],
                negative: vec![
                    ("irrationality", -2, 0.7),
                    ("impulsiveness", -1, 0.65),
                    ("thoughtlessness", -2, 0.6),
                ],
            },
            // Position 5 - Change (Emotion/Dynamics - PATHOS)
            NodeWithSemantics {
                position: 5,
                name: "Moral Sentiment".to_string(),
                positive: vec![
                    ("empathy", 3, 0.95),
                    ("compassion", 4, 0.9),
                    ("care", 2, 0.85),
                    ("sympathy", 3, 0.8),
                    ("kindness", 2, 0.85),
                ],
                negative: vec![
                    ("cruelty", -3, 0.8),
                    ("callousness", -2, 0.75),
                    ("indifference", -2, 0.7),
                ],
            },
            // Position 7 - Wisdom (Knowledge/Understanding)
            NodeWithSemantics {
                position: 7,
                name: "Ethical Theory".to_string(),
                positive: vec![
                    ("deontology", 3, 0.85),
                    ("consequentialism", 4, 0.8),
                    ("virtue-ethics", 4, 0.85),
                    ("utilitarianism", 3, 0.75),
                    ("kantian-ethics", 4, 0.8),
                ],
                negative: vec![
                    ("moral-relativism", -2, 0.6),
                    ("nihilism", -3, 0.7),
                ],
            },
            // Position 8 - Mastery (Peak/Excellence)
            NodeWithSemantics {
                position: 8,
                name: "Moral Excellence".to_string(),
                positive: vec![
                    ("wisdom", 5, 0.95),
                    ("righteousness", 5, 0.9),
                    ("nobility", 4, 0.85),
                    ("moral-perfection", 6, 0.9),
                    ("exemplary-virtue", 5, 0.85),
                ],
                negative: vec![
                    ("moral-failure", -3, 0.75),
                    ("vice", -2, 0.7),
                    ("depravity", -3, 0.8),
                ],
            },
        ],
        sacred_guides: vec![
            // Position 3 - Sacred Ethos (Unity/Integration)
            SacredWithSemantics {
                position: 3,
                name: "Moral Unity".to_string(),
                divine_properties: vec![
                    ("integration-of-self", 0.95),
                    ("moral-coherence", 0.93),
                    ("ethical-wholeness", 0.9),
                    ("unified-character", 0.92),
                    ("harmony-of-values", 0.88),
                    ("unifies", 0.93),
                    ("unify", 0.93),
                    ("integrates-morality", 0.92),
                    ("moral-unity", 0.94),
                    ("ethical-integration", 0.91),
                    ("character-unity", 0.9),
                ],
            },
            // Position 6 - Sacred Pathos (Emotional Core)
            SacredWithSemantics {
                position: 6,
                name: "Moral Heart".to_string(),
                divine_properties: vec![
                    ("pure-compassion", 0.96),
                    ("unconditional-love", 0.95),
                    ("moral-emotion", 0.92),
                    ("empathic-core", 0.93),
                    ("ethical-feeling", 0.9),
                    ("heart-of", 0.95),
                    ("core-of", 0.94),
                    ("center-of-compassion", 0.93),
                    ("soul-of-morality", 0.92),
                    ("moral-heart", 0.94),
                    ("compassionate-core", 0.91),
                ],
            },
            // Position 9 - Sacred Logos (Divine/Ultimate)
            SacredWithSemantics {
                position: 9,
                name: "Moral Law".to_string(),
                divine_properties: vec![
                    ("universal-ethics", 0.98),
                    ("absolute-morality", 0.96),
                    ("divine-justice", 0.95),
                    ("categorical-imperative", 0.94),
                    ("moral-truth", 0.97),
                    ("fundamental-good", 0.95),
                    ("ultimate-right", 0.93),
                    ("essence-of-ethics", 0.94),
                    ("nature-of-morality", 0.96),
                ],
            },
        ],
    }
}
