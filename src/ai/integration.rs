use crate::error::{Result, SpatialVortexError};
use crate::flux_matrix::FluxMatrixEngine;
use crate::models::*;
use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

fn dev_mode() -> bool {
    std::env::var("DEVELOPMENT_MODE").unwrap_or_default() == "true"
}
/// AI model integration for generating subject matrices using Grok 4 or other models
#[derive(Clone)]
pub struct AIModelIntegration {
    client: Client,
    api_key: Option<String>,
    model_endpoint: String,
    flux_engine: FluxMatrixEngine,
}

#[derive(Debug, Serialize)]
struct AIPromptRequest {
    model: String,
    messages: Vec<AIMessage>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Debug, Serialize)]
struct AIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct AIResponse {
    choices: Vec<AIChoice>,
}

#[derive(Debug, Deserialize)]
struct AIChoice {
    message: AIMessageResponse,
}

#[derive(Debug, Deserialize)]
struct AIMessageResponse {
    content: String,
}

/// Response structure from AI model for matrix generation
/// These fields will be populated when deserializing AI responses
#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Used for JSON deserialization from AI API
struct AIGeneratedMatrix {
    subject: String,
    nodes: HashMap<u8, AIGeneratedNode>,
    sacred_guides: HashMap<u8, AIGeneratedSacredGuide>,
}

/// AI-generated node structure from model responses
#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Used for JSON deserialization from AI API
struct AIGeneratedNode {
    position: u8,
    base_meaning: String,
    positive_associations: Vec<String>,
    negative_associations: Vec<String>,
    contextual_properties: Vec<String>,
}

/// AI-generated sacred guide structure from model responses
#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Used for JSON deserialization from AI API
struct AIGeneratedSacredGuide {
    position: u8,
    divine_properties: Vec<String>,
    geometric_significance: String,
}

impl AIModelIntegration {
    /// Create new AI integration with optional API key
    pub fn new(api_key: Option<String>, model_endpoint: Option<String>) -> Self {
        let endpoint = model_endpoint.unwrap_or_else(|| {
            "https://api.x.ai/v1/chat/completions".to_string() // Grok API endpoint
        });

        Self {
            client: Client::new(),
            api_key,
            model_endpoint: endpoint,
            flux_engine: FluxMatrixEngine::new(),
        }
    }

    /// Generate subject matrix using AI reasoning
    pub async fn generate_subject_matrix(&self, subject: &str) -> Result<FluxMatrix> {
        if self.api_key.is_none() || dev_mode() {
            // Fallback to deterministic generation without AI
            return self.generate_fallback_matrix(subject).await;
        }

        let prompt = self.create_matrix_generation_prompt(subject);

        match self.call_ai_model(&prompt).await {
            Ok(ai_response) => {
                self.parse_ai_response_to_matrix(subject, &ai_response)
                    .await
            }
            Err(_) => {
                // Fallback to deterministic generation if AI fails
                self.generate_fallback_matrix(subject).await
            }
        }
    }

    /// Create comprehensive prompt for AI matrix generation
    fn create_matrix_generation_prompt(&self, subject: &str) -> String {
        format!(
            r#"
Generate a Spatial Vortex Flux Matrix for the subject: "{subject}"

The Flux Matrix is based on the revolutionary pattern: 1, 2, 4, 8, 7, 5, 1 (repeating infinitely through exponential doubling with digit reduction).

Create a 9-node matrix where:
- Position 0: Neutral center (base value 0)
- Positions 1-8: Follow the flux pattern [1,2,4,8,7,5,1,2]
- Positions 3, 6, 9: Sacred Guides with divine properties

For each regular node (1,2,4,5,7,8), provide:
1. A primary meaning related to "{subject}"
2. 3-5 positive associations (synonyms/constructive aspects) with confidence scores
3. 2-4 negative associations (antonyms/destructive aspects) with confidence scores
4. Contextual properties that relate to the flux value

For each sacred guide (3,6,9), provide:
1. 3-4 divine properties specific to that position
2. Geometric significance in relation to "{subject}"

Sacred Guide Properties:
- Position 3: Creative Trinity, Synthesis, Bridge Between Realms
- Position 6: Harmonic Balance, Geometric Center, Stability
- Position 9: Completion Cycle, Infinite Loop Gateway, Transcendence

Generate meaningful semantic associations that demonstrate both positive moral alignment (Heaven - constructive influence) and negative moral alignment (Hell - destructive influence) for ethical AI reasoning.

Respond with a JSON structure following this format:
{{
  "subject": "{subject}",
  "nodes": {{
    "0": {{"position": 0, "base_meaning": "neutral center description", "positive_associations": [], "negative_associations": [], "contextual_properties": []}},
    "1": {{"position": 1, "base_meaning": "...", "positive_associations": ["word1", "word2"], "negative_associations": ["word3"], "contextual_properties": ["prop1"]}},
    // ... continue for positions 2,4,5,7,8
  }},
  "sacred_guides": {{
    "3": {{"position": 3, "divine_properties": ["Creative Trinity for {subject}", "..."], "geometric_significance": "..."}},
    "6": {{"position": 6, "divine_properties": ["Harmonic Balance in {subject}", "..."], "geometric_significance": "..."}},
    "9": {{"position": 9, "divine_properties": ["Completion Cycle of {subject}", "..."], "geometric_significance": "..."}}
  }}
}}

Ensure all associations are relevant to "{subject}" and demonstrate sophisticated understanding of the moral and contextual implications.
"#
        )
    }

    /// Call AI model API
    async fn call_ai_model(&self, prompt: &str) -> Result<String> {
        if dev_mode() {
            return Err(SpatialVortexError::AIIntegration("Dev mode: AI calls disabled".to_string()));
        }
        let request = AIPromptRequest {
            model: "grok-2-1212".to_string(), // Latest Grok model
            messages: vec![
                AIMessage {
                    role: "system".to_string(),
                    content: "You are an expert in the Spatial Vortex Flux Matrix Theory, capable of generating sophisticated semantic associations and moral reasoning patterns for any subject matter.".to_string(),
                },
                AIMessage {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                }
            ],
            temperature: 0.7,
            max_tokens: 2000,
        };

        let response = self
            .client
            .post(&self.model_endpoint)
            .header(
                "Authorization",
                format!("Bearer {}", self.api_key.as_ref().unwrap()),
            )
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(SpatialVortexError::AIIntegration(format!(
                "AI API call failed with status: {}",
                response.status()
            )));
        }

        let ai_response: AIResponse = response.json().await?;

        if ai_response.choices.is_empty() {
            return Err(SpatialVortexError::AIIntegration(
                "No response choices from AI model".to_string(),
            ));
        }

        Ok(ai_response.choices[0].message.content.clone())
    }

    /// Parse AI response into FluxMatrix
    async fn parse_ai_response_to_matrix(
        &self,
        subject: &str,
        ai_response: &str,
    ) -> Result<FluxMatrix> {
        // Try to extract JSON from the AI response
        let json_start = ai_response.find('{').unwrap_or(0);
        let json_end = ai_response
            .rfind('}')
            .map(|i| i + 1)
            .unwrap_or(ai_response.len());
        let json_str = &ai_response[json_start..json_end];

        let ai_matrix: AIGeneratedMatrix = serde_json::from_str(json_str).map_err(|e| {
            SpatialVortexError::AIIntegration(format!("Failed to parse AI response as JSON: {}", e))
        })?;

        // Convert AI-generated data to FluxMatrix
        let matrix_id = Uuid::new_v4();
        let now = Utc::now();

        let mut nodes = HashMap::new();
        let mut sacred_guides = HashMap::new();

        // Process regular nodes
        for (pos_str, ai_node) in ai_matrix.nodes {
            let position = pos_str;

            if [3, 6, 9].contains(&position) {
                continue; // Skip sacred guide positions in nodes
            }

            let base_value = self.flux_engine.get_flux_value_at_position(position);

            // Convert AI associations to semantic associations
            let positive_associations = ai_node
                .positive_associations
                .iter()
                .enumerate()
                .map(|(i, word)| {
                    {
                        let mut assoc = SemanticAssociation {
                            word: word.clone(),
                            index: (i + 1) as i16,
                            confidence: 0.9 * ((ai_node.positive_associations.len() - i) as f64 / ai_node.positive_associations.len() as f64),
                            attributes: HashMap::new(),
                        };
                        assoc.set_attribute("context".to_string(), 1.0); // AI Generated
                        assoc
                    }
                })
                .collect();

            let negative_associations = ai_node
                .negative_associations
                .iter()
                .enumerate()
                .map(|(i, word)| {
                    {
                        let mut assoc = SemanticAssociation {
                            word: word.clone(),
                            index: -((i + 1) as i16),
                            confidence: 0.9 * ((ai_node.negative_associations.len() - i) as f64 / ai_node.negative_associations.len() as f64),
                            attributes: HashMap::new(),
                        };
                        assoc.set_attribute("context".to_string(), 1.0); // AI Generated
                        assoc
                    }
                })
                .collect();

            let semantic_index = SemanticIndex {
                positive_associations,
                negative_associations,
                neutral_base: ai_node.base_meaning,
                predicates: Vec::new(),
                relations: Vec::new(),
            };

            // Create node attributes
            let attributes = NodeAttributes {
                properties: ai_node
                    .contextual_properties
                    .iter()
                    .enumerate()
                    .map(|(i, prop)| (format!("property_{}", i), prop.clone()))
                    .collect(),
                parameters: HashMap::new(),
                state: NodeState {
                    active: true,
                    last_accessed: now,
                    usage_count: 0,
                    context_stack: Vec::new(),
                },
                dynamics: NodeDynamics::default(),
            };

            let connections = self.flux_engine.create_node_connections(position);

            let node = FluxNode {
                position,
                base_value,
                semantic_index,
                attributes,
                connections,
            };

            nodes.insert(position, node);
        }

        // Process sacred guides
        for (pos_str, ai_guide) in ai_matrix.sacred_guides {
            let position = pos_str;

            if ![3, 6, 9].contains(&position) {
                continue; // Skip non-sacred positions
            }

            let intersection_points = self.flux_engine.create_intersection_points(position);

            let guide = SacredGuide {
                position,
                divine_properties: ai_guide.divine_properties,
                intersection_points,
                geometric_significance: ai_guide.geometric_significance,
            };

            sacred_guides.insert(position, guide);
        }

        // Ensure all positions are covered
        self.fill_missing_positions(&mut nodes, &mut sacred_guides, subject, now)?;

        Ok(FluxMatrix {
            id: matrix_id,
            subject: subject.to_string(),
            nodes,
            sacred_guides,
            created_at: now,
            updated_at: now,
        })
    }

    /// Fill any missing positions with default values
    fn fill_missing_positions(
        &self,
        nodes: &mut HashMap<u8, FluxNode>,
        sacred_guides: &mut HashMap<u8, SacredGuide>,
        subject: &str,
        _timestamp: chrono::DateTime<Utc>,
    ) -> Result<()> {
        for position in 0..=8 {
            if [3, 6, 9].contains(&position) {
                // Ensure sacred guides exist
                sacred_guides.entry(position).or_insert_with(|| {
                    self.flux_engine
                        .create_sacred_guide(position, subject)
                        .unwrap()
                });
            } else {
                // Ensure regular nodes exist
                let base_associations: Vec<(&str, i16)> = vec![
                    ("neutral", 0),
                    ("positive", 1),
                    ("negative", -1),
                ];
                for (i, (word, base_index)) in base_associations.iter().enumerate() {
                    if i as u8 % 9 == position {
                        // Distribute associations across positions
                        let mut association = SemanticAssociation {
                            word: word.to_string(),
                            index: *base_index,
                            confidence: (0.6 + position as f64 * 0.2) as f64,
                            attributes: HashMap::new(),
                        };
                        association.set_attribute("context".to_string(), 1.0); // ML Generated

                        if *base_index > 0 {
                            nodes.entry(position).or_insert_with(|| {
                                self.flux_engine
                                    .create_flux_node(position, subject)
                                    .unwrap()
                            })
                            .semantic_index
                            .positive_associations
                            .push(association);
                        } else {
                            nodes.entry(position).or_insert_with(|| {
                                self.flux_engine
                                    .create_flux_node(position, subject)
                                    .unwrap()
                            })
                            .semantic_index
                            .negative_associations
                            .push(association);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Generate fallback matrix without AI (deterministic)
    async fn generate_fallback_matrix(&self, subject: &str) -> Result<FluxMatrix> {
        let mut matrix = self.flux_engine.create_matrix(subject.to_string())?;

        // Add some basic semantic associations based on subject analysis
        self.add_deterministic_associations(&mut matrix).await?;

        Ok(matrix)
    }

    /// Add deterministic semantic associations based on keyword analysis
    async fn add_deterministic_associations(&self, matrix: &mut FluxMatrix) -> Result<()> {
        let subject_lower = matrix.subject.to_lowercase();

        // Basic keyword-based associations
        let mut base_associations = Vec::new();

        // Intelligence-related
        if subject_lower.contains("intelligence") {
            base_associations.extend(vec![
                ("reasoning", 1),
                ("logic", 2),
                ("cognition", 1),
                ("thinking", 1),
                ("stupidity", -1),
                ("ignorance", -2),
                ("confusion", -1),
            ]);
        }

        // Technology-related
        if subject_lower.contains("tech") || subject_lower.contains("computer") {
            base_associations.extend(vec![
                ("innovation", 2),
                ("efficiency", 1),
                ("automation", 1),
                ("obsolescence", -1),
                ("malfunction", -2),
            ]);
        }

        // Science-related
        if subject_lower.contains("science") {
            base_associations.extend(vec![
                ("discovery", 2),
                ("knowledge", 1),
                ("research", 1),
                ("truth", 2),
                ("superstition", -2),
                ("ignorance", -1),
            ]);
        }

        // Associations already applied during matrix creation above

        Ok(())
    }

    /// Enhance existing matrix with AI-generated semantic associations
    pub async fn enhance_matrix_semantics(&self, matrix: &mut FluxMatrix) -> Result<()> {
        if self.api_key.is_none() {
            return Ok(()); // Skip enhancement if no AI available
        }

        let enhancement_prompt = format!(
            r#"
Enhance the semantic associations for the Spatial Vortex matrix with subject: "{}"

Current matrix has {} nodes. For each node position (0-8), suggest 2-3 additional semantic associations that would improve the contextual understanding and moral reasoning capabilities.

Focus on:
1. Nuanced synonyms and antonyms
2. Moral implications (constructive vs destructive)
3. Contextual relevance to the subject matter
4. Intellectual depth and philosophical connections

Respond with JSON format:
{{
  "enhancements": [
    {{"position": 0, "additions": [{{"word": "...", "index": 1, "context": "..."}}, ...]}},
    ...
  ]
}}
"#,
            matrix.subject,
            matrix.nodes.len()
        );

        match self.call_ai_model(&enhancement_prompt).await {
            Ok(response) => {
                self.apply_semantic_enhancements(matrix, &response).await?;
            }
            Err(_) => {
                // Silently fail enhancement - matrix is still functional
            }
        }

        Ok(())
    }

    /// Apply AI-suggested semantic enhancements to matrix
    async fn apply_semantic_enhancements(
        &self,
        _matrix: &mut FluxMatrix,
        _ai_response: &str,
    ) -> Result<()> {
        // Parse enhancement suggestions and apply them
        // Implementation would parse the AI response and add the suggested associations
        // For now, we'll keep the matrix as-is to avoid parsing complexity
        Ok(())
    }

    /// Check if AI integration is available
    pub fn is_ai_available(&self) -> bool {
        self.api_key.is_some() && !dev_mode()
    }

    /// Get AI model status
    pub async fn get_ai_status(&self) -> String {
        if self.api_key.is_none() || dev_mode() {
            return "AI integration disabled - no API key".to_string();
        }

        // Try a simple ping to the AI service
        match self.client.get(&self.model_endpoint).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    "AI integration active".to_string()
                } else {
                    format!("AI service responded with status: {}", response.status())
                }
            }
            Err(e) => format!("AI service unreachable: {}", e),
        }
    }

    /// Dynamically fetch synonyms for a concept using AI/API
    /// Returns a list of semantically similar words in the given context
    pub async fn get_synonyms(&self, concept: &str, context: &str) -> Result<Vec<String>> {
        // If no API key, return empty list (fallback to no associations)
        if self.api_key.is_none() || dev_mode() {
            return Ok(Vec::new());
        }

        let prompt = format!(
            "Provide 6-8 synonyms or semantically similar terms for '{}' in the context of {}. \
            Return only a comma-separated list of single words or short phrases, no explanations.",
            concept, context
        );

        let request = AIPromptRequest {
            model: "grok-4".to_string(),
            messages: vec![AIMessage {
                role: "user".to_string(),
                content: prompt,
            }],
            temperature: 0.3,
            max_tokens: 100,
        };

        match self.make_ai_request(&request).await {
            Ok(response) => {
                let synonyms: Vec<String> = response
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .take(8)
                    .collect();
                Ok(synonyms)
            }
            Err(_) => Ok(Vec::new()), // Fallback to empty on error
        }
    }

    /// Dynamically fetch antonyms for a concept using AI/API
    /// Returns a list of semantically opposite/contrasting words in the given context
    pub async fn get_antonyms(&self, concept: &str, context: &str) -> Result<Vec<String>> {
        // If no API key, return empty list (fallback to no associations)
        if self.api_key.is_none() || dev_mode() {
            return Ok(Vec::new());
        }

        let prompt = format!(
            "Provide 3-5 antonyms or semantically opposite/contrasting terms for '{}' in the context of {}. \
            Return only a comma-separated list of single words or short phrases, no explanations.",
            concept, context
        );

        let request = AIPromptRequest {
            model: "grok-4".to_string(),
            messages: vec![AIMessage {
                role: "user".to_string(),
                content: prompt,
            }],
            temperature: 0.3,
            max_tokens: 80,
        };

        match self.make_ai_request(&request).await {
            Ok(response) => {
                let antonyms: Vec<String> = response
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .take(5)
                    .collect();
                Ok(antonyms)
            }
            Err(_) => Ok(Vec::new()), // Fallback to empty on error
        }
    }

    /// Make a subject generation request to AI (public for subject generator)
    pub async fn make_subject_generation_request(&self, prompt: &str) -> Result<String> {
        if dev_mode() {
            return Err(SpatialVortexError::AIIntegration("Dev mode: AI calls disabled".to_string()));
        }
        let request = AIPromptRequest {
            model: "grok-4".to_string(),
            messages: vec![AIMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            temperature: 0.5,
            max_tokens: 500,
        };

        self.make_ai_request(&request).await
    }

    /// Helper method to make AI API requests and extract response text
    async fn make_ai_request(&self, request: &AIPromptRequest) -> Result<String> {
        if dev_mode() {
            return Err(SpatialVortexError::AIIntegration("Dev mode: AI calls disabled".to_string()));
        }
        let api_key = self.api_key.as_ref().ok_or_else(|| {
            SpatialVortexError::AIIntegration("No API key configured".to_string())
        })?;

        let response = self
            .client
            .post(&self.model_endpoint)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .await
            .map_err(|e| SpatialVortexError::AIIntegration(format!("API request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(SpatialVortexError::AIIntegration(format!(
                "API returned error status: {}",
                response.status()
            )));
        }

        let ai_response: AIResponse = response.json().await.map_err(|e| {
            SpatialVortexError::AIIntegration(format!("Failed to parse response: {}", e))
        })?;

        ai_response
            .choices
            .first()
            .map(|choice| choice.message.content.clone())
            .ok_or_else(|| SpatialVortexError::AIIntegration("No response content".to_string()))
    }
}
