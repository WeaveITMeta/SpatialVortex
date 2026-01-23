//! Subject Definition Template
//!
//! Copy this template to create new subjects.
//! Follow the sacred geometry order of operations!

use crate::subject_definitions::consciousness::{
    SubjectDefinitionWithSemantics, NodeWithSemantics, SacredWithSemantics
};

/// Template for creating new subject definitions
///
/// ## Order of Operations (MUST FOLLOW):
/// - Position 0: CENTER (Neutral/Balance) - Recognition/Awareness
/// - Position 1: BEGINNING (Ethos - Self/Identity) - Personal aspect
/// - Position 2: EXPANSION (Growth/Perception) - External/Social aspect
/// - Position 3: SACRED ETHOS (Unity/Integration) ✨ - Bringing together
/// - Position 4: POWER (Logos - Cognition/Reason) - Rational aspect
/// - Position 5: CHANGE (Pathos - Emotion/Dynamics) - Emotional aspect
/// - Position 6: SACRED PATHOS (Emotional Core) ✨ - Heart/Feeling center
/// - Position 7: WISDOM (Knowledge/Understanding) - Theoretical aspect
/// - Position 8: MASTERY (Peak/Excellence) - Highest achievement
/// - Position 9: SACRED LOGOS (Divine/Ultimate) ✨ - Absolute/Fundamental
///
/// ## Cross-References:
/// Add related subjects as semantic associations to enable inference enrichment

#[allow(dead_code)]
pub fn template_definition() -> SubjectDefinitionWithSemantics {
    SubjectDefinitionWithSemantics {
        name: "SUBJECT_NAME".to_string(),
        nodes: vec![
            // Position 0 - CENTER
            NodeWithSemantics {
                position: 0,
                name: "SUBJECT Awareness".to_string(),
                positive: vec![
                    ("keyword1", 1, 0.85),
                    ("keyword2", 1, 0.8),
                    // Add related subjects for cross-reference
                    ("related-subject-1", 2, 0.7),
                ],
                negative: vec![
                    ("opposite1", -1, 0.7),
                ],
            },
            
            // Position 1 - BEGINNING (Ethos)
            NodeWithSemantics {
                position: 1,
                name: "Personal SUBJECT".to_string(),
                positive: vec![
                    ("self-keyword", 2, 0.9),
                    ("identity-keyword", 2, 0.85),
                ],
                negative: vec![],
            },
            
            // Position 2 - EXPANSION
            NodeWithSemantics {
                position: 2,
                name: "Social/External SUBJECT".to_string(),
                positive: vec![
                    ("external-keyword", 2, 0.9),
                ],
                negative: vec![],
            },
            
            // Position 4 - POWER (Logos)
            NodeWithSemantics {
                position: 4,
                name: "SUBJECT Reasoning".to_string(),
                positive: vec![
                    ("logical-keyword", 2, 0.9),
                    ("rational-keyword", 2, 0.85),
                ],
                negative: vec![],
            },
            
            // Position 5 - CHANGE (Pathos)
            NodeWithSemantics {
                position: 5,
                name: "SUBJECT Feeling".to_string(),
                positive: vec![
                    ("emotional-keyword", 3, 0.9),
                ],
                negative: vec![],
            },
            
            // Position 7 - WISDOM
            NodeWithSemantics {
                position: 7,
                name: "SUBJECT Theory".to_string(),
                positive: vec![
                    ("theory-keyword", 3, 0.85),
                    ("study-keyword", 4, 0.8),
                ],
                negative: vec![],
            },
            
            // Position 8 - MASTERY
            NodeWithSemantics {
                position: 8,
                name: "SUBJECT Mastery".to_string(),
                positive: vec![
                    ("excellence-keyword", 5, 0.95),
                    ("mastery-keyword", 6, 0.9),
                ],
                negative: vec![],
            },
        ],
        sacred_guides: vec![
            // Position 3 - SACRED ETHOS (Unity)
            SacredWithSemantics {
                position: 3,
                name: "SUBJECT Unity".to_string(),
                divine_properties: vec![
                    ("integrated-subject", 0.95),
                    ("unified-subject", 0.93),
                    ("coherent-subject", 0.9),
                    ("integrates", 0.92),
                    ("unifies", 0.91),
                ],
            },
            
            // Position 6 - SACRED PATHOS (Heart)
            SacredWithSemantics {
                position: 6,
                name: "SUBJECT Heart".to_string(),
                divine_properties: vec![
                    ("heart-of-subject", 0.95),
                    ("core-of-subject", 0.93),
                    ("heart-of", 0.94),
                    ("core-of", 0.92),
                    ("emotional-center", 0.9),
                ],
            },
            
            // Position 9 - SACRED LOGOS (Ultimate)
            SacredWithSemantics {
                position: 9,
                name: "Ultimate SUBJECT".to_string(),
                divine_properties: vec![
                    ("ultimate-subject", 0.98),
                    ("absolute-subject", 0.97),
                    ("fundamental-subject", 0.96),
                    ("essence-of-subject", 0.95),
                    ("nature-of-subject", 0.96),
                    // Always add these for Position 9
                    ("fundamental", 0.94),
                    ("ultimate", 0.93),
                    ("essence", 0.92),
                    ("nature", 0.91),
                ],
            },
        ],
    }
}
