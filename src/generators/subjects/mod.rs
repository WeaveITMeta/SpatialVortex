/// Subject matter definitions module
///
/// This module contains predefined semantic mappings for various subjects,
/// providing structured knowledge that can be loaded into flux matrices.
pub mod physics;

use crate::models::SemanticAssociation;
use crate::error::Result;
use std::collections::HashMap;

#[cfg(not(target_arch = "wasm32"))]
use crate::ai_integration::AIModelIntegration;

/// Definition of a node in a subject matrix
/// Semantic associations (synonyms/antonyms) are fetched dynamically via API
pub struct SubjectNodeDef {
    pub position: u8,
    pub name: String,
}

/// Definition of a sacred guide in a subject matrix
/// Properties represent the geometric/divine significance
pub struct SubjectSacredDef {
    pub position: u8,
    pub name: String,
}

/// Complete subject definition with all nodes and guides
pub struct SubjectDefinition {
    pub name: String,
    pub nodes: Vec<SubjectNodeDef>,
    pub sacred_guides: Vec<SubjectSacredDef>,
}

impl SubjectDefinition {
    /// Dynamically fetch semantic associations for a node using AI/API
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn fetch_node_associations(
        &self,
        node: &SubjectNodeDef,
        ai_integration: &AIModelIntegration,
    ) -> Result<(Vec<SemanticAssociation>, Vec<SemanticAssociation>)> {
        // Fetch synonyms (positive associations)
        let synonyms = ai_integration.get_synonyms(&node.name, &self.name).await?;

        let positive_associations: Vec<SemanticAssociation> = synonyms
            .iter()
            .enumerate()
            .map(|(i, word)| {
                let mut assoc = SemanticAssociation {
                    word: word.clone(),
                    index: (i + 1) as i16,
                    confidence: 0.85,
                    attributes: HashMap::new(),
                };
                assoc.set_attribute("context".to_string(), 1.0); // AI Generated
                assoc
            })
            .collect();

        // Fetch antonyms (negative associations)
        let antonyms = ai_integration.get_antonyms(&node.name, &self.name).await?;

        let negative_associations: Vec<SemanticAssociation> = antonyms
            .iter()
            .enumerate()
            .map(|(i, word)| {
                let mut assoc = SemanticAssociation {
                    word: word.clone(),
                    index: -((i + 1) as i16),
                    confidence: 0.85,
                    attributes: HashMap::new(),
                };
                assoc.set_attribute("context".to_string(), 1.0); // AI Generated
                assoc
            })
            .collect();

        Ok((positive_associations, negative_associations))
    }
}

/// Get subject definition by name
pub fn get_subject_definition(subject_name: &str) -> Option<SubjectDefinition> {
    match subject_name.to_lowercase().as_str() {
        "physics" => Some(physics::get_physics_definition()),
        _ => None,
    }
}
