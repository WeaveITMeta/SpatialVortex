/// Dynamic subject generation module
///
/// Generates new subject definition modules as separate Rust files
/// that can be compiled into the subjects directory.
use crate::ai_integration::AIModelIntegration;
use crate::error::{Result, SpatialVortexError};
use std::fs;
use std::path::Path;

/// Subject generator for creating new subject modules
pub struct SubjectGenerator {
    ai_integration: AIModelIntegration,
    subjects_dir: String,
}

/// Generated subject structure from AI
#[derive(Debug)]
pub struct GeneratedSubject {
    pub name: String,
    pub nodes: Vec<GeneratedNode>,
    pub sacred_guides: Vec<GeneratedSacredGuide>,
}

#[derive(Debug)]
pub struct GeneratedNode {
    pub position: u8,
    pub name: String,
}

#[derive(Debug)]
pub struct GeneratedSacredGuide {
    pub position: u8,
    pub name: String,
}

impl SubjectGenerator {
    /// Create a new subject generator
    pub fn new(ai_integration: AIModelIntegration, subjects_dir: Option<String>) -> Self {
        Self {
            ai_integration,
            subjects_dir: subjects_dir.unwrap_or_else(|| "src/subjects".to_string()),
        }
    }

    /// Generate a new subject definition using AI
    pub async fn generate_subject(&self, subject_name: &str) -> Result<GeneratedSubject> {
        println!("[Subject Generator] Generating subject: {}", subject_name);

        // Use AI to determine appropriate node positions and names
        let prompt = format!(
            "For the subject '{}', define a flux matrix with 9 positions (1-9). \
            Positions 3, 6, and 9 are sacred guides representing fundamental principles. \
            Regular nodes occupy positions 1, 2, 4, 5, 7, 8.\n\
            \n\
            Provide:
            1. For each regular node position (1,2,4,5,7,8): A single-word or short phrase concept name
            2. For each sacred guide position (3,6,9): A single-word fundamental principle name
            
            Format your response as JSON:
            {{
              \"nodes\": [
                {{\"position\": 1, \"name\": \"ConceptName\"}},
                ...
              ],
              \"sacred_guides\": [
                {{\"position\": 3, \"name\": \"PrincipleName\"}},
                ...
              ]
            }}
            
            Make the names specific to {} and avoid generic terms.",
            subject_name, subject_name
        );

        let response = self
            .ai_integration
            .make_subject_generation_request(&prompt)
            .await?;

        // Parse the JSON response
        let parsed: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            SpatialVortexError::AIIntegration(format!("Failed to parse AI response as JSON: {}", e))
        })?;

        let mut nodes = Vec::new();
        if let Some(nodes_array) = parsed["nodes"].as_array() {
            for node_obj in nodes_array {
                if let (Some(pos), Some(name)) =
                    (node_obj["position"].as_u64(), node_obj["name"].as_str())
                {
                    nodes.push(GeneratedNode {
                        position: pos as u8,
                        name: name.to_string(),
                    });
                }
            }
        }

        let mut sacred_guides = Vec::new();
        if let Some(guides_array) = parsed["sacred_guides"].as_array() {
            for guide_obj in guides_array {
                if let (Some(pos), Some(name)) =
                    (guide_obj["position"].as_u64(), guide_obj["name"].as_str())
                {
                    sacred_guides.push(GeneratedSacredGuide {
                        position: pos as u8,
                        name: name.to_string(),
                    });
                }
            }
        }

        Ok(GeneratedSubject {
            name: subject_name.to_string(),
            nodes,
            sacred_guides,
        })
    }

    /// Generate Rust code for a subject module
    pub fn generate_rust_code(&self, subject: &GeneratedSubject) -> String {
        let module_name = subject.name.to_lowercase().replace(" ", "_");
        let mut code = String::new();

        // File header
        code.push_str(&format!("/// {} subject matter definition\n", subject.name));
        code.push_str("/// \n");
        code.push_str(&format!(
            "/// Defines the semantic structure for {} concepts mapped to\n",
            subject.name
        ));
        code.push_str("/// the 9-position flux matrix (1-9). Sacred guides at positions 3, 6, 9\n");
        code.push_str(
            "/// provide geometric anchoring, while regular nodes occupy 1, 2, 4, 5, 7, 8.\n",
        );
        code.push_str("/// \n");
        code.push_str(
            "/// Semantic associations (synonyms/antonyms) are fetched dynamically via AI/API.\n",
        );
        code.push_str("\n");
        code.push_str("use super::{SubjectDefinition, SubjectNodeDef, SubjectSacredDef};\n");
        code.push_str("\n");

        // Function definition
        code.push_str(&format!(
            "/// Get the complete {} subject definition\n",
            subject.name
        ));
        code.push_str(&format!(
            "pub fn get_{}_definition() -> SubjectDefinition {{\n",
            module_name
        ));
        code.push_str("    SubjectDefinition {\n");
        code.push_str(&format!(
            "        name: \"{}\".to_string(),\n",
            subject.name
        ));
        code.push_str("        nodes: vec![\n");

        // Generate nodes
        for node in &subject.nodes {
            code.push_str("            SubjectNodeDef {\n");
            code.push_str(&format!("                position: {},\n", node.position));
            code.push_str(&format!(
                "                name: \"{}\".to_string(),\n",
                node.name
            ));
            code.push_str("            },\n");
        }

        code.push_str("        ],\n");
        code.push_str("        sacred_guides: vec![\n");

        // Generate sacred guides
        for guide in &subject.sacred_guides {
            code.push_str("            SubjectSacredDef {\n");
            code.push_str(&format!("                position: {},\n", guide.position));
            code.push_str(&format!(
                "                name: \"{}\".to_string(),\n",
                guide.name
            ));
            code.push_str("            },\n");
        }

        code.push_str("        ],\n");
        code.push_str("    }\n");
        code.push_str("}\n");

        code
    }

    /// Write the generated subject to a file
    pub fn write_subject_file(&self, subject: &GeneratedSubject) -> Result<String> {
        let module_name = subject.name.to_lowercase().replace(" ", "_");
        let filename = format!("{}.rs", module_name);
        let filepath = Path::new(&self.subjects_dir).join(&filename);

        // Check if file already exists
        if filepath.exists() {
            return Err(SpatialVortexError::AIIntegration(format!(
                "Subject file already exists: {}",
                filepath.display()
            )));
        }

        // Generate the code
        let code = self.generate_rust_code(subject);

        // Write to file
        fs::write(&filepath, code).map_err(|e| SpatialVortexError::Io(e))?;

        println!("[Subject Generator] Created file: {}", filepath.display());
        Ok(filename)
    }

    /// Update subjects/mod.rs to include the new module
    pub fn update_mod_file(&self, subject: &GeneratedSubject) -> Result<()> {
        let module_name = subject.name.to_lowercase().replace(" ", "_");
        let mod_path = Path::new(&self.subjects_dir).join("mod.rs");

        // Read current mod.rs content
        let current_content =
            fs::read_to_string(&mod_path).map_err(|e| SpatialVortexError::Io(e))?;

        // Check if module is already declared
        let module_declaration = format!("pub mod {};", module_name);
        if current_content.contains(&module_declaration) {
            println!(
                "[Subject Generator] Module '{}' already declared in mod.rs",
                module_name
            );
            return Ok(());
        }

        // Find the position to insert (after other pub mod declarations)
        let mut lines: Vec<String> = current_content.lines().map(|s| s.to_string()).collect();
        let mut insert_pos = 0;

        for (i, line) in lines.iter().enumerate() {
            if line.starts_with("pub mod ") {
                insert_pos = i + 1;
            }
        }

        // Insert the new module declaration
        lines.insert(insert_pos, module_declaration);

        // Write back to file
        let new_content = lines.join("\n") + "\n";
        fs::write(&mod_path, new_content).map_err(|e| SpatialVortexError::Io(e))?;

        println!(
            "[Subject Generator] Updated mod.rs with module '{}'",
            module_name
        );
        Ok(())
    }

    /// Update the get_subject_definition match statement
    pub fn update_subject_getter(&self, subject: &GeneratedSubject) -> Result<()> {
        let module_name = subject.name.to_lowercase().replace(" ", "_");
        let subject_lower = subject.name.to_lowercase();
        let mod_path = Path::new(&self.subjects_dir).join("mod.rs");

        let current_content =
            fs::read_to_string(&mod_path).map_err(|e| SpatialVortexError::Io(e))?;

        // Find the get_subject_definition function and update it
        let match_pattern = format!(
            "\"{}\" => Some({}::get_{}_definition())",
            subject_lower, module_name, module_name
        );

        if current_content.contains(&match_pattern) {
            println!(
                "[Subject Generator] Subject '{}' already in match statement",
                subject.name
            );
            return Ok(());
        }

        // Insert new match arm before the default case
        let new_content = current_content.replace(
            "        _ => None,",
            &format!(
                "        \"{}\" => Some({}::get_{}_definition()),\n        _ => None,",
                subject_lower, module_name, module_name
            ),
        );

        fs::write(&mod_path, new_content).map_err(|e| SpatialVortexError::Io(e))?;

        println!(
            "[Subject Generator] Added '{}' to subject getter",
            subject.name
        );
        Ok(())
    }

    /// Complete workflow: generate, write, and register a new subject
    pub async fn create_subject(&self, subject_name: &str) -> Result<()> {
        println!(
            "\n[Subject Generator] === Creating Subject: {} ===\n",
            subject_name
        );

        // Step 1: Generate subject definition using AI
        let generated = self.generate_subject(subject_name).await?;

        println!("\n[Subject Generator] Generated structure:");
        println!("  Nodes: {} regular nodes", generated.nodes.len());
        for node in &generated.nodes {
            println!("    Position {}: {}", node.position, node.name);
        }
        println!("  Sacred Guides: {} guides", generated.sacred_guides.len());
        for guide in &generated.sacred_guides {
            println!("    Position {}: {}", guide.position, guide.name);
        }

        // Step 2: Write the subject file
        let filename = self.write_subject_file(&generated)?;

        // Step 3: Update mod.rs
        self.update_mod_file(&generated)?;

        // Step 4: Update subject getter
        self.update_subject_getter(&generated)?;

        println!(
            "\n[Subject Generator] === Subject '{}' Created Successfully ===",
            subject_name
        );
        println!("[Subject Generator] File: {}", filename);
        println!("[Subject Generator] Next steps:");
        println!("  1. Run 'cargo fmt' to format the new code");
        println!("  2. Run 'cargo check' to verify compilation");
        println!("  3. Rebuild your application to use the new subject\n");

        Ok(())
    }
}
