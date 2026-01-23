//! Document Ingestion Pipeline for RAG
//!
//! Automatically ingests various document formats, chunks them intelligently,
//! and prepares them for embedding with sacred geometry awareness.

use crate::models::ELPTensor;
use crate::core::sacred_geometry::flux_matrix::FluxMatrixEngine;
use crate::ml::inference::OnnxSessionPool;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;
// async_trait not needed for current implementation

/// Supported document types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentType {
    PlainText,
    Markdown,
    PDF,
    HTML,
    JSON,
    Code(String), // programming language
    Research,     // academic papers
    Documentation,
}

/// A document ready for ingestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub source: String,
    pub doc_type: DocumentType,
    pub content: String,
    pub metadata: DocumentMetadata,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Document metadata for context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub tags: Vec<String>,
    pub category: Option<String>,
    pub language: String,
    pub sacred_relevance: f32, // 0.0-1.0, how relevant to sacred geometry
}

/// A chunk of a document for embedding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentChunk {
    pub id: String,
    pub doc_id: String,
    pub chunk_index: usize,
    pub content: String,
    pub tokens: usize,
    pub elp_tensor: ELPTensor,
    pub flux_position: u8,
    pub overlap_prev: Option<String>,
    pub overlap_next: Option<String>,
}

/// Configuration for document ingestion
#[derive(Debug, Clone)]
pub struct IngestionConfig {
    pub chunk_size: usize,        // tokens per chunk
    pub chunk_overlap: usize,     // overlapping tokens
    pub max_chunks: usize,        // max chunks per document
    pub min_chunk_size: usize,    // minimum viable chunk
    pub sacred_boost: bool,       // boost sacred geometry mentions
    pub auto_categorize: bool,    // automatically categorize documents
}

impl Default for IngestionConfig {
    fn default() -> Self {
        Self {
            chunk_size: 512,
            chunk_overlap: 50,
            max_chunks: 1000,
            min_chunk_size: 100,
            sacred_boost: true,
            auto_categorize: true,
        }
    }
}

/// Main document ingester
pub struct DocumentIngester {
    config: IngestionConfig,
    flux_engine: FluxMatrixEngine,
    onnx_pool: Option<OnnxSessionPool>,
}

impl DocumentIngester {
    pub fn new(config: IngestionConfig) -> Self {
        Self {
            config,
            flux_engine: FluxMatrixEngine::new(),
            onnx_pool: None,
        }
    }
    
    pub fn with_onnx_pool(mut self, pool: OnnxSessionPool) -> Self {
        self.onnx_pool = Some(pool);
        self
    }
    
    /// Ingest a document from file
    pub async fn ingest_file(&self, path: &Path) -> Result<Document> {
        let content = fs::read_to_string(path).await?;
        let doc_type = self.detect_document_type(path);
        let metadata = self.extract_metadata(&content, &doc_type).await?;
        
        Ok(Document {
            id: uuid::Uuid::new_v4().to_string(),
            source: path.display().to_string(),
            doc_type,
            content,
            metadata,
            timestamp: chrono::Utc::now(),
        })
    }
    
    /// Ingest documents from directory recursively
    pub async fn ingest_directory(&self, dir: &Path) -> Result<Vec<Document>> {
        let mut documents = Vec::new();
        let mut entries = tokio::fs::read_dir(dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            if path.is_dir() {
                // Recursive ingestion
                let sub_docs = Box::pin(self.ingest_directory(&path)).await?;
                documents.extend(sub_docs);
            } else if self.is_supported_file(&path) {
                match self.ingest_file(&path).await {
                    Ok(doc) => documents.push(doc),
                    Err(e) => eprintln!("Failed to ingest {}: {}", path.display(), e),
                }
            }
        }
        
        Ok(documents)
    }
    
    /// Chunk a document into overlapping segments
    pub async fn chunk_document(&self, doc: &Document) -> Result<Vec<DocumentChunk>> {
        let mut chunks = Vec::new();
        let words: Vec<&str> = doc.content.split_whitespace().collect();
        
        // Estimate tokens (rough approximation)
        let total_tokens = words.len() * 2; // Average 2 tokens per word
        
        // Adjust chunk size if document is very large
        let effective_chunk_size = if total_tokens > 10000 {
            self.config.chunk_size * 2 // Larger chunks for large documents
        } else {
            self.config.chunk_size
        };
        
        let mut start = 0;
        let mut chunk_index = 0;
        
        while start < words.len() && chunk_index < self.config.max_chunks {
            let end = (start + effective_chunk_size / 2).min(words.len());
            let chunk_content = words[start..end].join(" ");
            
            // Skip if too small
            if chunk_content.len() < self.config.min_chunk_size {
                break;
            }
            
            // Calculate overlap
            let overlap_prev = if chunk_index > 0 && start > 0 {
                let overlap_start = (start as i32 - self.config.chunk_overlap as i32 / 2).max(0) as usize;
                Some(words[overlap_start..start].join(" "))
            } else {
                None
            };
            
            let overlap_next = if end < words.len() {
                let overlap_end = (end + self.config.chunk_overlap / 2).min(words.len());
                Some(words[end..overlap_end].join(" "))
            } else {
                None
            };
            
            // Calculate ELP tensor from content
            let elp_tensor = self.calculate_elp_tensor(&chunk_content).await?;
            
            // Determine flux position
            let flux_position = self.flux_engine.calculate_position_from_elp(
                elp_tensor.ethos as f32,
                elp_tensor.logos as f32,
                elp_tensor.pathos as f32,
            );
            
            chunks.push(DocumentChunk {
                id: format!("{}-{}", doc.id, chunk_index),
                doc_id: doc.id.clone(),
                chunk_index,
                content: chunk_content,
                tokens: (end - start) * 2,
                elp_tensor,
                flux_position,
                overlap_prev,
                overlap_next,
            });
            
            // Move forward with overlap
            start = end - self.config.chunk_overlap / 2;
            chunk_index += 1;
        }
        
        Ok(chunks)
    }
    
    /// Calculate ELP tensor from text content
    async fn calculate_elp_tensor(&self, content: &str) -> Result<ELPTensor> {
        // Sacred geometry keywords boost
        let sacred_keywords = [
            "sacred", "geometry", "vortex", "triangle", "3-6-9",
            "ethos", "logos", "pathos", "flux", "matrix",
        ];
        
        let mut sacred_score: f32 = 0.0;
        for keyword in &sacred_keywords {
            if content.to_lowercase().contains(keyword) {
                sacred_score += 0.2;
            }
        }
        sacred_score = sacred_score.min(1.0);
        
        // Analyze content characteristics
        let word_count = content.split_whitespace().count();
        let question_marks = content.matches('?').count() as f32;
        let exclamations = content.matches('!').count() as f32;
        
        // Heuristic ELP calculation (would use ML model in production)
        let ethos = (sacred_score * 3.0 + 5.0).min(9.0); // Character/ethics
        let logos = ((word_count as f32 / 50.0) * 3.0 + question_marks + 4.0).min(9.0); // Logic/reason
        let pathos = (exclamations * 2.0 + (1.0 - sacred_score) * 3.0 + 4.0).min(9.0); // Emotion
        
        Ok(ELPTensor {
            ethos: ethos as f64,
            logos: logos as f64,
            pathos: pathos as f64,
        })
    }
    
    /// Extract metadata from content
    async fn extract_metadata(&self, content: &str, doc_type: &DocumentType) -> Result<DocumentMetadata> {
        // Extract title (first line or heading)
        let title = content.lines()
            .find(|line| !line.trim().is_empty())
            .map(|s| s.to_string());
        
        // Detect sacred geometry relevance
        let sacred_relevance = self.calculate_sacred_relevance(content);
        
        // Auto-categorize if enabled
        let category = if self.config.auto_categorize {
            self.auto_categorize(content, doc_type)
        } else {
            None
        };
        
        Ok(DocumentMetadata {
            title,
            author: None, // Could extract from metadata
            tags: self.extract_tags(content),
            category,
            language: "en".to_string(),
            sacred_relevance,
        })
    }
    
    /// Calculate how relevant content is to sacred geometry
    fn calculate_sacred_relevance(&self, content: &str) -> f32 {
        let sacred_patterns = [
            (r"\b3[-\s]?6[-\s]?9\b", 0.3),
            (r"\bsacred\s+geometry\b", 0.25),
            (r"\bvortex\s+(mathematics|math|flow)\b", 0.2),
            (r"\bethos.*logos.*pathos\b", 0.15),
            (r"\b(flux|matrix|tensor)\b", 0.1),
        ];
        
        let mut relevance: f32 = 0.0;
        for (pattern, weight) in &sacred_patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if re.is_match(&content.to_lowercase()) {
                    relevance += weight;
                }
            }
        }
        
        relevance.min(1.0)
    }
    
    /// Extract tags from content
    fn extract_tags(&self, content: &str) -> Vec<String> {
        let mut tags = Vec::new();
        
        // Look for hashtags
        if let Ok(re) = regex::Regex::new(r"#(\w+)") {
            for cap in re.captures_iter(content) {
                if let Some(tag) = cap.get(1) {
                    tags.push(tag.as_str().to_string());
                }
            }
        }
        
        // Add content-based tags
        if content.contains("AI") || content.contains("artificial intelligence") {
            tags.push("AI".to_string());
        }
        if content.contains("machine learning") || content.contains("ML") {
            tags.push("ML".to_string());
        }
        if content.contains("sacred") || content.contains("geometry") {
            tags.push("sacred-geometry".to_string());
        }
        
        tags
    }
    
    /// Auto-categorize document
    fn auto_categorize(&self, content: &str, doc_type: &DocumentType) -> Option<String> {
        match doc_type {
            DocumentType::Research => Some("research".to_string()),
            DocumentType::Documentation => Some("docs".to_string()),
            DocumentType::Code(_) => Some("code".to_string()),
            _ => {
                // Content-based categorization
                if content.contains("theorem") || content.contains("proof") {
                    Some("mathematics".to_string())
                } else if content.contains("experiment") || content.contains("hypothesis") {
                    Some("science".to_string())
                } else if content.contains("function") || content.contains("class") {
                    Some("programming".to_string())
                } else {
                    Some("general".to_string())
                }
            }
        }
    }
    
    /// Detect document type from file extension
    fn detect_document_type(&self, path: &Path) -> DocumentType {
        match path.extension().and_then(|e| e.to_str()) {
            Some("txt") => DocumentType::PlainText,
            Some("md") => DocumentType::Markdown,
            Some("pdf") => DocumentType::PDF,
            Some("html") | Some("htm") => DocumentType::HTML,
            Some("json") => DocumentType::JSON,
            Some("rs") => DocumentType::Code("rust".to_string()),
            Some("py") => DocumentType::Code("python".to_string()),
            Some("js") | Some("ts") => DocumentType::Code("javascript".to_string()),
            _ => DocumentType::PlainText,
        }
    }
    
    /// Check if file is supported
    fn is_supported_file(&self, path: &Path) -> bool {
        let supported_extensions = [
            "txt", "md", "json", "rs", "py", "js", "ts", 
            "html", "htm", "yaml", "yml", "toml"
        ];
        
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| supported_extensions.contains(&e))
            .unwrap_or(false)
    }
}

/// Batch ingestion for performance
pub struct BatchIngester {
    ingester: DocumentIngester,
    batch_size: usize,
}

impl BatchIngester {
    pub fn new(config: IngestionConfig, batch_size: usize) -> Self {
        Self {
            ingester: DocumentIngester::new(config),
            batch_size,
        }
    }
    
    /// Ingest multiple directories in parallel
    pub async fn ingest_parallel(&self, directories: Vec<PathBuf>) -> Result<Vec<Document>> {
        use futures::stream::{self, StreamExt};
        
        let results = stream::iter(directories)
            .map(|dir| async move {
                self.ingester.ingest_directory(&dir).await
            })
            .buffer_unordered(self.batch_size)
            .collect::<Vec<_>>()
            .await;
        
        let mut all_documents = Vec::new();
        for result in results {
            match result {
                Ok(docs) => all_documents.extend(docs),
                Err(e) => eprintln!("Batch ingestion error: {}", e),
            }
        }
        
        Ok(all_documents)
    }
}
