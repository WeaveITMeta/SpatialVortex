//! Atman Pattern - Thoughts on Top of Thoughts
//!
//! Before implementing new functionality, search for existing implementations
//! in the codebase and external sources. This enables true recursive self-improvement
//! by building on existing knowledge rather than reinventing.
//!
//! ## Key Concepts
//! - **Term Search**: Search for RL, ML, and domain-specific terms
//! - **Implementation Discovery**: Find existing code that solves similar problems
//! - **Knowledge Layering**: Build new thoughts on top of existing thoughts
//! - **Codebase Awareness**: Understand what already exists before creating new

use crate::data::models::BeamTensor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Atman configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtmanConfig {
    /// Search paths for codebase exploration
    pub search_paths: Vec<PathBuf>,
    /// Key terms to always search for
    pub key_terms: Vec<String>,
    /// Minimum similarity threshold for matches
    pub similarity_threshold: f32,
    /// Maximum results per search
    pub max_results: usize,
    /// Enable external search (web, papers)
    pub external_search: bool,
}

impl Default for AtmanConfig {
    fn default() -> Self {
        Self {
            search_paths: vec![PathBuf::from("./src")],
            key_terms: vec![
                // Reinforcement Learning
                "reinforcement".to_string(),
                "reward".to_string(),
                "policy".to_string(),
                "value_function".to_string(),
                "q_learning".to_string(),
                "actor_critic".to_string(),
                "ppo".to_string(),
                "dqn".to_string(),
                // Machine Learning
                "gradient".to_string(),
                "optimizer".to_string(),
                "loss".to_string(),
                "backprop".to_string(),
                "attention".to_string(),
                "transformer".to_string(),
                "embedding".to_string(),
                "encoder".to_string(),
                "decoder".to_string(),
                // Sacred Geometry / Vortex
                "vortex".to_string(),
                "flux".to_string(),
                "sacred".to_string(),
                "beam".to_string(),
                "latent".to_string(),
            ],
            similarity_threshold: 0.5,
            max_results: 20,
            external_search: false,
        }
    }
}

/// A discovered implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredImpl {
    /// File path where found
    pub path: PathBuf,
    /// Line number
    pub line: usize,
    /// The matched content
    pub content: String,
    /// Matched terms
    pub matched_terms: Vec<String>,
    /// Relevance score
    pub relevance: f32,
    /// Type of implementation
    pub impl_type: ImplType,
}

/// Type of discovered implementation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ImplType {
    Function,
    Struct,
    Trait,
    Module,
    Algorithm,
    Pattern,
    Unknown,
}

/// Search result from Atman
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtmanSearchResult {
    /// Query that was searched
    pub query: String,
    /// Discovered implementations
    pub discoveries: Vec<DiscoveredImpl>,
    /// Related terms found
    pub related_terms: Vec<String>,
    /// Suggested next steps
    pub suggestions: Vec<String>,
    /// Search duration in ms
    pub duration_ms: f64,
}

/// Atman - The Self-Aware Codebase Explorer
pub struct Atman {
    config: AtmanConfig,
    /// Cache of discovered implementations
    cache: HashMap<String, Vec<DiscoveredImpl>>,
    /// Term frequency index
    term_index: HashMap<String, Vec<(PathBuf, usize)>>,
    /// Knowledge graph of related concepts
    knowledge_graph: HashMap<String, Vec<String>>,
}

impl Atman {
    pub fn new(config: AtmanConfig) -> Self {
        let mut atman = Self {
            config,
            cache: HashMap::new(),
            term_index: HashMap::new(),
            knowledge_graph: HashMap::new(),
        };
        atman.build_knowledge_graph();
        atman
    }

    /// Build the knowledge graph of related concepts
    fn build_knowledge_graph(&mut self) {
        // RL concepts
        self.add_relations("reinforcement_learning", &[
            "reward", "policy", "value_function", "q_learning", "actor_critic",
            "ppo", "dqn", "sarsa", "monte_carlo", "temporal_difference"
        ]);
        
        // ML concepts
        self.add_relations("machine_learning", &[
            "gradient", "optimizer", "loss", "backprop", "neural_network",
            "attention", "transformer", "embedding", "encoder", "decoder"
        ]);
        
        // Optimization
        self.add_relations("optimization", &[
            "gradient_descent", "adam", "sgd", "learning_rate", "momentum",
            "spectral", "sphere", "sso"
        ]);
        
        // Sacred geometry
        self.add_relations("sacred_geometry", &[
            "vortex", "flux", "sacred", "beam", "position", "3_6_9",
            "fibonacci", "golden_ratio"
        ]);
        
        // Vortex specific
        self.add_relations("vortex", &[
            "flux_matrix", "beam_tensor", "latent_state", "calm", "ebrm",
            "pathway", "entropic", "puct"
        ]);
        
        // Entropic objective
        self.add_relations("entropic_objective", &[
            "log_sum_exp", "adaptive_beta", "kl_divergence", "temperature",
            "exploration", "exploitation"
        ]);
    }

    fn add_relations(&mut self, concept: &str, related: &[&str]) {
        let related_vec: Vec<String> = related.iter().map(|s| s.to_string()).collect();
        self.knowledge_graph.insert(concept.to_string(), related_vec.clone());
        
        // Bidirectional relations
        for term in related {
            self.knowledge_graph
                .entry(term.to_string())
                .or_default()
                .push(concept.to_string());
        }
    }

    /// Search for implementations before coding
    pub fn search_before_coding(&mut self, task: &str) -> AtmanSearchResult {
        let start = std::time::Instant::now();
        
        // Extract key terms from task
        let terms = self.extract_terms(task);
        
        // Search codebase for each term
        let mut all_discoveries = Vec::new();
        for term in &terms {
            if let Some(cached) = self.cache.get(term) {
                all_discoveries.extend(cached.clone());
            } else {
                let discoveries = self.search_codebase(term);
                self.cache.insert(term.clone(), discoveries.clone());
                all_discoveries.extend(discoveries);
            }
        }

        // Deduplicate and sort by relevance
        all_discoveries.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap_or(std::cmp::Ordering::Equal));
        all_discoveries.dedup_by(|a, b| a.path == b.path && a.line == b.line);
        all_discoveries.truncate(self.config.max_results);

        // Find related terms
        let related_terms = self.find_related_terms(&terms);

        // Generate suggestions
        let suggestions = self.generate_suggestions(&all_discoveries, &terms);

        AtmanSearchResult {
            query: task.to_string(),
            discoveries: all_discoveries,
            related_terms,
            suggestions,
            duration_ms: start.elapsed().as_secs_f64() * 1000.0,
        }
    }

    /// Extract key terms from a task description
    fn extract_terms(&self, task: &str) -> Vec<String> {
        let task_lower = task.to_lowercase();
        let words: Vec<&str> = task_lower
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|s| s.len() > 2)
            .collect();

        let mut terms = Vec::new();
        
        // Check against key terms
        for word in &words {
            if self.config.key_terms.iter().any(|t| t.contains(word) || word.contains(t.as_str())) {
                terms.push(word.to_string());
            }
        }

        // Check knowledge graph
        for word in &words {
            if self.knowledge_graph.contains_key(*word) {
                terms.push(word.to_string());
            }
        }

        terms.sort();
        terms.dedup();
        terms
    }

    /// Search codebase for a term
    fn search_codebase(&self, term: &str) -> Vec<DiscoveredImpl> {
        let mut discoveries = Vec::new();

        for search_path in &self.config.search_paths {
            if let Ok(entries) = std::fs::read_dir(search_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map(|e| e == "rs").unwrap_or(false) {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            discoveries.extend(self.search_file(&path, &content, term));
                        }
                    }
                }
            }
        }

        discoveries
    }

    /// Search a single file for a term
    fn search_file(&self, path: &PathBuf, content: &str, term: &str) -> Vec<DiscoveredImpl> {
        let mut discoveries = Vec::new();
        let term_lower = term.to_lowercase();

        for (line_num, line) in content.lines().enumerate() {
            let line_lower = line.to_lowercase();
            
            if line_lower.contains(&term_lower) {
                let impl_type = self.detect_impl_type(line);
                let relevance = self.calculate_relevance(line, term);

                if relevance >= self.config.similarity_threshold {
                    discoveries.push(DiscoveredImpl {
                        path: path.clone(),
                        line: line_num + 1,
                        content: line.trim().to_string(),
                        matched_terms: vec![term.to_string()],
                        relevance,
                        impl_type,
                    });
                }
            }
        }

        discoveries
    }

    /// Detect the type of implementation from a line
    fn detect_impl_type(&self, line: &str) -> ImplType {
        let line_trimmed = line.trim();
        
        if line_trimmed.starts_with("fn ") || line_trimmed.starts_with("pub fn ") {
            ImplType::Function
        } else if line_trimmed.starts_with("struct ") || line_trimmed.starts_with("pub struct ") {
            ImplType::Struct
        } else if line_trimmed.starts_with("trait ") || line_trimmed.starts_with("pub trait ") {
            ImplType::Trait
        } else if line_trimmed.starts_with("mod ") || line_trimmed.starts_with("pub mod ") {
            ImplType::Module
        } else if line_trimmed.contains("algorithm") || line_trimmed.contains("impl ") {
            ImplType::Algorithm
        } else {
            ImplType::Unknown
        }
    }

    /// Calculate relevance score
    fn calculate_relevance(&self, line: &str, term: &str) -> f32 {
        let line_lower = line.to_lowercase();
        let term_lower = term.to_lowercase();

        let mut score = 0.0f32;

        // Exact match bonus
        if line_lower.contains(&term_lower) {
            score += 0.5;
        }

        // Function/struct definition bonus
        if line.contains("fn ") || line.contains("struct ") || line.contains("impl ") {
            score += 0.3;
        }

        // Public API bonus
        if line.starts_with("pub ") {
            score += 0.2;
        }

        // Documentation bonus
        if line.contains("///") || line.contains("//!") {
            score += 0.1;
        }

        score.min(1.0)
    }

    /// Find related terms from knowledge graph
    fn find_related_terms(&self, terms: &[String]) -> Vec<String> {
        let mut related = Vec::new();

        for term in terms {
            if let Some(relations) = self.knowledge_graph.get(term) {
                related.extend(relations.clone());
            }
        }

        related.sort();
        related.dedup();
        related
    }

    /// Generate suggestions based on discoveries
    fn generate_suggestions(&self, discoveries: &[DiscoveredImpl], terms: &[String]) -> Vec<String> {
        let mut suggestions = Vec::new();

        if discoveries.is_empty() {
            suggestions.push(format!(
                "No existing implementations found for: {}. Consider creating new module.",
                terms.join(", ")
            ));
        } else {
            // Group by impl type
            let functions: Vec<_> = discoveries.iter()
                .filter(|d| d.impl_type == ImplType::Function)
                .collect();
            let structs: Vec<_> = discoveries.iter()
                .filter(|d| d.impl_type == ImplType::Struct)
                .collect();

            if !functions.is_empty() {
                suggestions.push(format!(
                    "Found {} existing functions. Consider extending rather than reimplementing.",
                    functions.len()
                ));
            }

            if !structs.is_empty() {
                suggestions.push(format!(
                    "Found {} existing structs. Consider composition or trait implementation.",
                    structs.len()
                ));
            }

            // Suggest reviewing top matches
            if let Some(top) = discoveries.first() {
                suggestions.push(format!(
                    "Review {} at line {} (relevance: {:.2})",
                    top.path.display(),
                    top.line,
                    top.relevance
                ));
            }
        }

        suggestions
    }

    /// Convert search result to BeamTensors for vortex processing
    pub fn result_to_beams(&self, result: &AtmanSearchResult) -> Vec<BeamTensor> {
        result.discoveries.iter().map(|d| {
            let mut beam = BeamTensor::default();
            
            // Encode relevance
            beam.digits[0] = d.relevance;
            
            // Encode impl type
            beam.digits[1] = match d.impl_type {
                ImplType::Function => 0.1,
                ImplType::Struct => 0.2,
                ImplType::Trait => 0.3,
                ImplType::Module => 0.4,
                ImplType::Algorithm => 0.5,
                ImplType::Pattern => 0.6,
                ImplType::Unknown => 0.0,
            };
            
            // Encode line number (normalized)
            beam.digits[2] = (d.line as f32 / 1000.0).min(1.0);
            
            // Encode content hash
            let hash = d.content.bytes().fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
            for i in 3..9 {
                beam.digits[i] = ((hash >> ((i - 3) * 8)) & 0xFF) as f32 / 255.0;
            }
            
            beam.confidence = d.relevance;
            beam
        }).collect()
    }

    /// Get config
    pub fn config(&self) -> &AtmanConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atman_creation() {
        let config = AtmanConfig::default();
        let atman = Atman::new(config);
        assert!(!atman.knowledge_graph.is_empty());
    }

    #[test]
    fn test_extract_terms() {
        let config = AtmanConfig::default();
        let atman = Atman::new(config);
        
        let terms = atman.extract_terms("implement reinforcement learning with gradient descent");
        assert!(terms.contains(&"reinforcement".to_string()) || terms.contains(&"gradient".to_string()));
    }

    #[test]
    fn test_knowledge_graph() {
        let config = AtmanConfig::default();
        let atman = Atman::new(config);
        
        assert!(atman.knowledge_graph.contains_key("vortex"));
        assert!(atman.knowledge_graph.get("vortex").unwrap().contains(&"flux_matrix".to_string()));
    }

    #[test]
    fn test_detect_impl_type() {
        let config = AtmanConfig::default();
        let atman = Atman::new(config);
        
        assert_eq!(atman.detect_impl_type("pub fn test()"), ImplType::Function);
        assert_eq!(atman.detect_impl_type("pub struct Test"), ImplType::Struct);
        assert_eq!(atman.detect_impl_type("pub trait Test"), ImplType::Trait);
    }

    #[test]
    fn test_result_to_beams() {
        let config = AtmanConfig::default();
        let atman = Atman::new(config);
        
        let result = AtmanSearchResult {
            query: "test".to_string(),
            discoveries: vec![DiscoveredImpl {
                path: PathBuf::from("test.rs"),
                line: 10,
                content: "fn test()".to_string(),
                matched_terms: vec!["test".to_string()],
                relevance: 0.8,
                impl_type: ImplType::Function,
            }],
            related_terms: vec![],
            suggestions: vec![],
            duration_ms: 1.0,
        };
        
        let beams = atman.result_to_beams(&result);
        assert_eq!(beams.len(), 1);
        assert_eq!(beams[0].confidence, 0.8);
    }
}
