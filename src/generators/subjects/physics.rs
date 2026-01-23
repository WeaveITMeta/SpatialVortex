/// Physics subject matter definition
///
/// Defines the semantic structure for Physics concepts mapped to
/// the 9-position flux matrix (1-9). Sacred guides at positions 3, 6, 9
/// provide geometric anchoring, while regular nodes occupy 1, 2, 4, 5, 7, 8.
///
/// Semantic associations (synonyms/antonyms) are fetched dynamically via AI/API.
use super::{SubjectDefinition, SubjectNodeDef, SubjectSacredDef};

/// Get the complete Physics subject definition
pub fn get_physics_definition() -> SubjectDefinition {
    SubjectDefinition {
        name: "Physics".to_string(),
        nodes: vec![
            SubjectNodeDef {
                position: 1,
                name: "Object".to_string(),
            },
            SubjectNodeDef {
                position: 2,
                name: "Forces".to_string(),
            },
            SubjectNodeDef {
                position: 4,
                name: "Value".to_string(),
            },
            SubjectNodeDef {
                position: 5,
                name: "Unit".to_string(),
            },
            SubjectNodeDef {
                position: 7,
                name: "Assembly".to_string(),
            },
            SubjectNodeDef {
                position: 8,
                name: "Constraints".to_string(),
            },
        ],
        sacred_guides: vec![
            SubjectSacredDef {
                position: 3,
                name: "Law".to_string(),
            },
            SubjectSacredDef {
                position: 6,
                name: "Anti-Matter".to_string(),
            },
            SubjectSacredDef {
                position: 9,
                name: "Material".to_string(),
            },
        ],
    }
}
