//! Psychology Subject Definition
//!
//! Mind, behavior, and mental processes with cross-references to cognition and consciousness

use crate::subject_definitions::consciousness::{
    SubjectDefinitionWithSemantics, NodeWithSemantics, SacredWithSemantics
};

pub fn definition() -> SubjectDefinitionWithSemantics {
    SubjectDefinitionWithSemantics {
        name: "psychology".to_string(),
        nodes: vec![
            // Position 0 - CENTER
            NodeWithSemantics {
                position: 0,
                name: "Psychological Awareness".to_string(),
                positive: vec![
                    ("mental-state", 1, 0.9),
                    ("psychological", 1, 0.85),
                    ("mind", 1, 0.8),
                    // Cross-references
                    ("consciousness", 2, 0.75),
                    ("cognition", 2, 0.7),
                ],
                negative: vec![
                    ("mindless", -1, 0.7),
                    ("unaware", -1, 0.65),
                ],
            },
            
            // Position 1 - BEGINNING (Ethos)
            NodeWithSemantics {
                position: 1,
                name: "Self-Psychology".to_string(),
                positive: vec![
                    ("self-image", 2, 0.95),
                    ("identity", 2, 0.9),
                    ("personality", 3, 0.85),
                    ("ego", 2, 0.8),
                    ("self-concept", 3, 0.85),
                ],
                negative: vec![
                    ("identity-crisis", -2, 0.75),
                    ("fragmented-self", -2, 0.7),
                ],
            },
            
            // Position 2 - EXPANSION
            NodeWithSemantics {
                position: 2,
                name: "Social Psychology".to_string(),
                positive: vec![
                    ("behavior", 2, 0.9),
                    ("social-interaction", 3, 0.85),
                    ("group-dynamics", 3, 0.8),
                    ("interpersonal", 2, 0.75),
                ],
                negative: vec![
                    ("isolation", -2, 0.7),
                    ("social-dysfunction", -3, 0.65),
                ],
            },
            
            // Position 4 - POWER (Logos)
            NodeWithSemantics {
                position: 4,
                name: "Cognitive Psychology".to_string(),
                positive: vec![
                    ("mental-processes", 2, 0.95),
                    ("thinking", 2, 0.9),
                    ("reasoning", 2, 0.85),
                    ("problem-solving", 3, 0.8),
                    // Cross-reference
                    ("cognition", 3, 0.85),
                    ("inference", 3, 0.8),
                ],
                negative: vec![
                    ("cognitive-bias", -2, 0.7),
                    ("irrational", -2, 0.65),
                ],
            },
            
            // Position 5 - CHANGE (Pathos)
            NodeWithSemantics {
                position: 5,
                name: "Emotional Psychology".to_string(),
                positive: vec![
                    ("emotion", 3, 0.95),
                    ("feelings", 2, 0.9),
                    ("affect", 3, 0.85),
                    ("mood", 2, 0.8),
                    ("emotional-response", 3, 0.85),
                ],
                negative: vec![
                    ("emotional-suppression", -2, 0.7),
                    ("alexithymia", -3, 0.75),
                ],
            },
            
            // Position 7 - WISDOM
            NodeWithSemantics {
                position: 7,
                name: "Psychological Theory".to_string(),
                positive: vec![
                    ("behaviorism", 3, 0.85),
                    ("psychoanalysis", 4, 0.8),
                    ("humanistic-psychology", 4, 0.85),
                    ("cognitive-behavioral", 4, 0.8),
                    ("psychotherapy", 3, 0.75),
                ],
                negative: vec![
                    ("pseudopsychology", -3, 0.8),
                ],
            },
            
            // Position 8 - MASTERY
            NodeWithSemantics {
                position: 8,
                name: "Psychological Integration".to_string(),
                positive: vec![
                    ("self-actualization", 5, 0.95),
                    ("psychological-health", 5, 0.9),
                    ("mental-wellbeing", 4, 0.85),
                    ("flourishing", 5, 0.9),
                    ("optimal-functioning", 5, 0.85),
                ],
                negative: vec![
                    ("psychopathology", -3, 0.8),
                    ("mental-disorder", -3, 0.75),
                ],
            },
        ],
        sacred_guides: vec![
            // Position 3 - SACRED ETHOS
            SacredWithSemantics {
                position: 3,
                name: "Psychological Unity".to_string(),
                divine_properties: vec![
                    ("integrated-psyche", 0.96),
                    ("unified-mind", 0.94),
                    ("psychological-coherence", 0.92),
                    ("whole-person", 0.93),
                    ("integrates", 0.92),
                    ("unifies", 0.91),
                    ("mental-integration", 0.94),
                ],
            },
            
            // Position 6 - SACRED PATHOS
            SacredWithSemantics {
                position: 6,
                name: "Psychological Heart".to_string(),
                divine_properties: vec![
                    ("emotional-core", 0.96),
                    ("psychological-center", 0.94),
                    ("heart-mind-connection", 0.93),
                    ("heart-of", 0.95),
                    ("core-of", 0.93),
                    ("inner-life", 0.92),
                ],
            },
            
            // Position 9 - SACRED LOGOS
            SacredWithSemantics {
                position: 9,
                name: "Ultimate Psychology".to_string(),
                divine_properties: vec![
                    ("universal-psyche", 0.98),
                    ("archetypal-mind", 0.96),
                    ("collective-unconscious", 0.95),
                    ("fundamental-psychology", 0.97),
                    ("essence-of-mind", 0.96),
                    ("nature-of-psychology", 0.97),
                    ("ultimate", 0.94),
                    ("fundamental", 0.93),
                ],
            },
        ],
    }
}
