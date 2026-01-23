//! Retrieval-Augmented Generation
//!
//! Combines retrieved context with generation to produce
//! more accurate and grounded responses.

use crate::rag::retrieval::{RAGRetriever, RetrievalResult};
use crate::rag::vector_store::VectorDatabase;
use crate::ai::orchestrator::{ASIOrchestrator, ExecutionMode};
use tokio::sync::Mutex;
use crate::ai::multi_source_search::{MultiSourceSearcher, SearchConfig, SearchEngine, WebSource};
use crate::models::ELPTensor;
use crate::ml::hallucinations::VortexContextPreserver;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Configuration for augmented generation
#[derive(Debug, Clone)]
pub struct GenerationConfig {
    pub max_length: usize,
    pub temperature: f32,
    pub use_sacred_guidance: bool,
    pub hallucination_check: bool,
    pub context_integration: ContextIntegration,
    pub enable_web_search: bool,
    pub max_web_sources: usize,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            max_length: 512,
            temperature: 0.7,
            use_sacred_guidance: true,
            hallucination_check: true,
            context_integration: ContextIntegration::Hierarchical,
            enable_web_search: true,
            max_web_sources: 10,
        }
    }
}

/// How to integrate retrieved context
#[derive(Debug, Clone)]
pub enum ContextIntegration {
    Prepend,     // Simply prepend context
    Hierarchical, // Weight by relevance
    Fusion,      // Deep fusion with query
    Sacred,      // Sacred geometry guided
}

/// Augmented generation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationResult {
    pub response: String,
    pub context_used: Vec<String>,
    pub confidence: f32,
    pub hallucination_risk: f32,
    pub elp_tensor: ELPTensor,
    pub flux_position: u8,
    pub sources: Vec<SourceAttribution>,
}

/// Source attribution for generated content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceAttribution {
    pub doc_id: String,
    pub chunk_id: String,
    pub relevance: f32,
    pub content_snippet: String,
    /// Web source metadata (if from web search)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_source: Option<WebSourceMeta>,
}

/// Web source metadata for attribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSourceMeta {
    pub url: String,
    pub title: String,
    pub domain: String,
    pub credibility_score: f32,
    pub source_type: String,
    pub search_engine: String,
}

/// Main augmented generator
pub struct AugmentedGenerator {
    retriever: Arc<RAGRetriever>,
    asi_orchestrator: Arc<Mutex<ASIOrchestrator>>,
    #[allow(dead_code)]  // Reserved for hallucination detection during generation
    context_preserver: VortexContextPreserver,
    config: GenerationConfig,
    web_searcher: Option<MultiSourceSearcher>,
    vector_db: Option<Arc<VectorDatabase>>,
}

impl AugmentedGenerator {
    pub async fn new(
        retriever: Arc<RAGRetriever>,
        orchestrator: Arc<Mutex<ASIOrchestrator>>,
        config: GenerationConfig,
    ) -> Result<Self> {
        // Initialize web searcher if enabled
        let web_searcher = if config.enable_web_search {
            let search_config = SearchConfig {
                max_sources: config.max_web_sources,
                engines: vec![SearchEngine::DuckDuckGo], // Default to free engine
                timeout_secs: 10,
                min_credibility: 0.4,
            };
            Some(MultiSourceSearcher::new(search_config)?)
        } else {
            None
        };
        
        Ok(Self {
            retriever,
            asi_orchestrator: orchestrator,
            context_preserver: VortexContextPreserver::default(),
            config,
            web_searcher,
            vector_db: None, // Will be set via set_vector_db()
        })
    }
    
    /// Set vector database for storing web sources
    pub fn set_vector_db(&mut self, vector_db: Arc<VectorDatabase>) {
        self.vector_db = Some(vector_db);
    }
    
    /// Generate response with retrieved context
    pub async fn generate(&mut self, query: &str) -> Result<GenerationResult> {
        // Step 1: Perform web search if enabled
        let web_sources = if let Some(ref searcher) = self.web_searcher {
            let search_result = searcher.search(query).await?;
            
            // Store web sources in vector database if available
            if let Some(ref vector_db) = self.vector_db {
                self.store_web_sources(&search_result.sources, vector_db).await?;
            }
            
            search_result.sources
        } else {
            Vec::new()
        };
        
        // Step 2: Retrieve relevant context from local database
        let retrieved_context = self.retriever.hybrid_retrieve(query).await?;
        
        // Step 3: Integrate context based on strategy
        let integrated_context = self.integrate_context(query, &retrieved_context)?;
        
        // Step 4: Add web sources to context
        let web_context = self.format_web_sources(&web_sources);
        let full_context = if !web_context.is_empty() {
            format!("{}\n\n[WEB SOURCES]\n{}", integrated_context, web_context)
        } else {
            integrated_context
        };
        
        // Step 5: Generate with ASI orchestrator
        let prompt = self.build_prompt(query, &full_context);
        let asi_result = self.asi_orchestrator.lock().await.process(&prompt, ExecutionMode::Balanced).await?;
        
        // Step 6: Check for hallucinations
        let (confidence, hallucination_risk) = if self.config.hallucination_check {
            self.check_hallucination(&asi_result.result, &full_context)?
        } else {
            (0.8, 0.2) // Default values
        };
        
        // Step 7: Build source attributions (including web sources)
        let mut sources = self.build_attributions(&retrieved_context);
        sources.extend(self.build_web_attributions(&web_sources));
        
        Ok(GenerationResult {
            response: asi_result.result,
            context_used: {
                let mut context = retrieved_context.iter()
                    .map(|r| r.content.clone())
                    .collect::<Vec<_>>();
                context.extend(web_sources.iter().map(|ws| ws.snippet.clone()));
                context
            },
            confidence,
            hallucination_risk,
            elp_tensor: asi_result.elp,
            flux_position: asi_result.flux_position,
            sources,
        })
    }
    
    /// Integrate retrieved context based on strategy
    fn integrate_context(
        &self,
        query: &str,
        retrieved: &[RetrievalResult],
    ) -> Result<String> {
        match self.config.context_integration {
            ContextIntegration::Prepend => {
                // Simple concatenation
                Ok(retrieved
                    .iter()
                    .map(|r| r.content.clone())
                    .collect::<Vec<_>>()
                    .join("\n\n"))
            }
            
            ContextIntegration::Hierarchical => {
                // Weight by relevance score
                let mut weighted_context = Vec::new();
                
                for result in retrieved {
                    if result.relevance_score > 0.8 {
                        // High relevance - include full content
                        weighted_context.push(format!("[HIGH RELEVANCE]\n{}", result.content));
                    } else if result.relevance_score > 0.6 {
                        // Medium relevance - include summary
                        let summary = self.summarize(&result.content);
                        weighted_context.push(format!("[MEDIUM RELEVANCE]\n{}", summary));
                    } else {
                        // Low relevance - include key points only
                        let key_points = self.extract_key_points(&result.content);
                        weighted_context.push(format!("[LOW RELEVANCE]\n{}", key_points));
                    }
                }
                
                Ok(weighted_context.join("\n\n"))
            }
            
            ContextIntegration::Fusion => {
                // Deep fusion with query
                let mut fused = format!("Query: {}\n\nRelevant Context:\n", query);
                
                for result in retrieved {
                    // Interleave query terms with context
                    let query_terms: Vec<&str> = query.split_whitespace().collect();
                    let mut enhanced_content = result.content.clone();
                    
                    for term in &query_terms {
                        if result.content.to_lowercase().contains(&term.to_lowercase()) {
                            enhanced_content = enhanced_content.replace(
                                term,
                                &format!("**{}**", term)
                            );
                        }
                    }
                    
                    fused.push_str(&format!("\n- {}", enhanced_content));
                }
                
                Ok(fused)
            }
            
            ContextIntegration::Sacred => {
                // Sacred geometry guided integration
                let mut sacred_context = Vec::new();
                
                // Group by flux positions
                let mut position_groups: std::collections::HashMap<u8, Vec<&RetrievalResult>> = 
                    std::collections::HashMap::new();
                
                for result in retrieved {
                    position_groups
                        .entry(result.flux_position)
                        .or_insert(Vec::new())
                        .push(result);
                }
                
                // Process in vortex order: 1→2→4→8→7→5→1
                let vortex_order = [1, 2, 4, 8, 7, 5];
                
                for position in vortex_order {
                    if let Some(results) = position_groups.get(&position) {
                        for result in results {
                            sacred_context.push(format!(
                                "[Position {} - Signal {:.2}]\n{}",
                                position,
                                result.confidence,
                                result.content
                            ));
                        }
                    }
                }
                
                // Add sacred positions (3, 6, 9) with emphasis
                for sacred_pos in [3, 6, 9] {
                    if let Some(results) = position_groups.get(&sacred_pos) {
                        for result in results {
                            sacred_context.push(format!(
                                "[SACRED Position {} - Signal {:.2}]\n{}",
                                sacred_pos,
                                result.confidence,
                                result.content
                            ));
                        }
                    }
                }
                
                Ok(sacred_context.join("\n\n"))
            }
        }
    }
    
    /// Build prompt with context
    fn build_prompt(&self, query: &str, context: &str) -> String {
        format!(
            "Context:\n{}\n\nQuery: {}\n\nProvide a comprehensive response based on the context.",
            context,
            query
        )
    }
    
    /// Check for hallucinations by comparing response grounding in context
    fn check_hallucination(
        &self,
        response: &str,
        context: &str,
    ) -> Result<(f32, f32)> {
        // Compute signal strength based on word overlap (context grounding)
        let response_words: std::collections::HashSet<&str> = 
            response.split_whitespace().collect();
        let context_words: std::collections::HashSet<&str> = 
            context.split_whitespace().collect();
        
        let overlap = response_words.intersection(&context_words).count();
        let response_len = response_words.len().max(1);
        
        // Signal strength = how much response is grounded in context
        let word_overlap_signal = (overlap as f32 / response_len as f32).min(1.0);
        
        // Check for exact substring matches (stronger grounding)
        let substring_matches = response
            .split('.')
            .filter(|sent| !sent.trim().is_empty())
            .filter(|sent| context.contains(sent.trim()))
            .count();
        let total_sentences = response.split('.').filter(|s| !s.trim().is_empty()).count().max(1);
        let substring_signal = (substring_matches as f32 / total_sentences as f32).min(1.0);
        
        // Combine both signals
        let final_signal = (word_overlap_signal * 0.4 + substring_signal * 0.6).min(1.0);
        
        // Hallucination risk is inverse of signal strength
        let hallucination_risk = if final_signal < 0.5 {
            1.0 - final_signal  // High risk if poorly grounded
        } else {
            (1.0 - final_signal) * 0.5  // Lower risk if well grounded
        };
        
        Ok((final_signal, hallucination_risk))
    }
    
    /// Build source attributions
    fn build_attributions(&self, retrieved: &[RetrievalResult]) -> Vec<SourceAttribution> {
        retrieved
            .iter()
            .map(|result| {
                let snippet = if result.content.len() > 100 {
                    format!("{}...", &result.content[..100])
                } else {
                    result.content.clone()
                };
                
                SourceAttribution {
                    doc_id: result.doc_id.clone(),
                    chunk_id: result.chunk_id.clone(),
                    relevance: result.relevance_score,
                    content_snippet: snippet,
                    web_source: None,
                }
            })
            .collect()
    }
    
    /// Build attributions from web sources
    fn build_web_attributions(&self, web_sources: &[WebSource]) -> Vec<SourceAttribution> {
        web_sources
            .iter()
            .map(|ws| {
                SourceAttribution {
                    doc_id: format!("web_{}", ws.domain),
                    chunk_id: ws.url.clone(),
                    relevance: ws.credibility_score,
                    content_snippet: ws.snippet.clone(),
                    web_source: Some(WebSourceMeta {
                        url: ws.url.clone(),
                        title: ws.title.clone(),
                        domain: ws.domain.clone(),
                        credibility_score: ws.credibility_score,
                        source_type: format!("{:?}", ws.source_type),
                        search_engine: ws.search_engine.clone(),
                    }),
                }
            })
            .collect()
    }
    
    /// Format web sources for inclusion in context
    fn format_web_sources(&self, web_sources: &[WebSource]) -> String {
        if web_sources.is_empty() {
            return String::new();
        }
        
        web_sources
            .iter()
            .enumerate()
            .map(|(idx, ws)| {
                format!(
                    "[{}] {} ({:.0}% credible)\n   Source: {}\n   {}\n",
                    idx + 1,
                    ws.title,
                    ws.credibility_score * 100.0,
                    ws.domain,
                    ws.snippet
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
    
    /// Store web sources in vector database
    async fn store_web_sources(
        &self,
        web_sources: &[WebSource],
        vector_db: &Arc<VectorDatabase>,
    ) -> Result<()> {
        use crate::rag::vector_store::SacredEmbedding;
        use std::collections::HashMap;
        
        // Filter out duplicates and low-credibility sources
        let mut seen_urls = std::collections::HashSet::new();
        let filtered_sources: Vec<&WebSource> = web_sources
            .iter()
            .filter(|ws| {
                ws.credibility_score >= 0.75 &&
                seen_urls.insert(ws.url.clone())
            })
            .collect();
        
        // Convert web sources to embeddings
        // Note: In production, you'd generate actual embeddings from content
        // For now, we'll create placeholder embeddings
        let embeddings: Vec<SacredEmbedding> = filtered_sources
            .iter()
            .map(|ws| {
                let mut metadata = HashMap::new();
                metadata.insert("url".to_string(), ws.url.clone());
                metadata.insert("title".to_string(), ws.title.clone());
                metadata.insert("domain".to_string(), ws.domain.clone());
                metadata.insert("source_type".to_string(), format!("{:?}", ws.source_type));
                metadata.insert("search_engine".to_string(), ws.search_engine.clone());
                
                // Determine flux position based on credibility
                let flux_position = if ws.credibility_score >= 0.9 {
                    9 // High credibility -> sacred position
                } else if ws.credibility_score >= 0.75 {
                    6
                } else if ws.credibility_score >= 0.6 {
                    3
                } else {
                    1 // Default position
                };
                
                SacredEmbedding {
                    id: format!("web_{}", uuid::Uuid::new_v4()),
                    doc_id: format!("web_{}", ws.domain),
                    chunk_id: ws.url.clone(),
                    embedding: vec![0.0; 384], // Placeholder - should be actual embedding
                    elp_tensor: ELPTensor::default(), // TODO: Calculate from content
                    flux_position,
                    confidence: ws.credibility_score,
                    sacred_boost: if [3, 6, 9].contains(&flux_position) { 1.5 } else { 1.0 },
                    forward_chain_weight: 1.0,
                    back_prop_weight: 1.0,
                    metadata,
                }
            })
            .collect();
        
        // Store in database
        vector_db.add_embeddings_batch(embeddings).await?;
        
        Ok(())
    }
    
    /// Summarize content (simple version)
    fn summarize(&self, content: &str) -> String {
        let sentences: Vec<&str> = content.split('.').collect();
        if sentences.len() <= 2 {
            content.to_string()
        } else {
            // Take first and last sentence
            format!("{}. ... {}", sentences[0], sentences[sentences.len() - 2])
        }
    }
    
    /// Extract key points
    fn extract_key_points(&self, content: &str) -> String {
        // Look for patterns like "- ", "* ", numbers, etc.
        let lines: Vec<&str> = content.lines()
            .filter(|line| {
                line.starts_with('-') ||
                line.starts_with('*') ||
                line.starts_with(|c: char| c.is_numeric())
            })
            .collect();
        
        if lines.is_empty() {
            // Fallback to first line
            content.lines().next().unwrap_or("").to_string()
        } else {
            lines.join("\n")
        }
    }
    
    /// Generate with specific execution mode
    pub async fn generate_with_mode(
        &mut self,
        query: &str,
        mode: ExecutionMode,
    ) -> Result<GenerationResult> {
        // Retrieve context
        let retrieved_context = self.retriever.retrieve(query).await?;
        let integrated_context = self.integrate_context(query, &retrieved_context)?;
        
        // Generate with specified mode
        let prompt = self.build_prompt(query, &integrated_context);
        let asi_result = self.asi_orchestrator.lock().await.process(&prompt, mode).await?;
        
        // Check hallucinations
        let (confidence, hallucination_risk) = if self.config.hallucination_check {
            self.check_hallucination(&asi_result.result, &integrated_context)?
        } else {
            (0.8, 0.2)
        };
        
        Ok(GenerationResult {
            response: asi_result.result,
            context_used: retrieved_context.iter()
                .map(|r| r.content.clone())
                .collect(),
            confidence,
            hallucination_risk,
            elp_tensor: asi_result.elp,
            flux_position: asi_result.flux_position,
            sources: self.build_attributions(&retrieved_context),
        })
    }
    
    /// Stream generation (for real-time responses)
    pub async fn stream_generate(
        &mut self,
        query: &str,
        callback: impl Fn(&str),
    ) -> Result<GenerationResult> {
        // Get initial context
        let retrieved_context = self.retriever.retrieve(query).await?;
        let integrated_context = self.integrate_context(query, &retrieved_context)?;
        
        // Generate in chunks (simulated streaming)
        let prompt = self.build_prompt(query, &integrated_context);
        let asi_result = self.asi_orchestrator.lock().await.process(&prompt, ExecutionMode::Fast).await?;
        
        // Simulate streaming by sending words progressively
        let words: Vec<&str> = asi_result.result.split_whitespace().collect();
        let mut streamed = String::new();
        
        for word in &words {
            streamed.push_str(word);
            streamed.push(' ');
            callback(&streamed);
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
        
        // Final result
        let (confidence, hallucination_risk) = if self.config.hallucination_check {
            self.check_hallucination(&asi_result.result, &integrated_context)?
        } else {
            (0.8, 0.2)
        };
        
        Ok(GenerationResult {
            response: asi_result.result,
            context_used: retrieved_context.iter()
                .map(|r| r.content.clone())
                .collect(),
            confidence,
            hallucination_risk,
            elp_tensor: asi_result.elp,
            flux_position: asi_result.flux_position,
            sources: self.build_attributions(&retrieved_context),
        })
    }
}
