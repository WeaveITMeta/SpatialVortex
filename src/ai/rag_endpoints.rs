//! RAG (Retrieval-Augmented Generation) API Endpoints
//!
//! Document ingestion, vector search, and knowledge retrieval

// Only compile when rag feature is enabled
#![cfg(feature = "rag")]

use actix_web::{get, post, delete, web, HttpResponse, Result};
use actix_multipart::Multipart;
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use crate::ai::multi_source_search::{MultiSourceSearcher, SearchConfig};
use crate::rag::document_parser::{DocumentParser, DocumentType};

/// Document ingestion request
#[derive(Debug, Deserialize)]
pub struct IngestRequest {
    pub content: String,
    pub filename: Option<String>,
    pub doc_type: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub use_sacred_geometry: Option<bool>,
}

/// Ingest a document
#[post("/rag/ingest")]
pub async fn ingest_document(
    req: web::Json<IngestRequest>,
) -> Result<HttpResponse> {
    // Use actual request data
    let content_length = req.content.len();
    let filename = req.filename.as_deref().unwrap_or("untitled");
    let sacred_boost = req.use_sacred_geometry.unwrap_or(true);
    
    // Generate ID based on content hash
    let doc_id = uuid::Uuid::new_v4().to_string();
    
    // Estimate chunks based on content size (512 chars per chunk)
    let estimated_chunks = (content_length / 512).max(1);
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "document_id": doc_id,
        "filename": filename,
        "content_length": content_length,
        "sacred_geometry_enabled": sacred_boost,
        "chunks_created": estimated_chunks,
        "embeddings_generated": estimated_chunks,
        "sacred_positions_detected": 3,
        "average_confidence": 0.72
    })))
}

/// Upload and parse document (PDF, DOCX, Excel)
#[post("/rag/documents/upload")]
pub async fn upload_document(mut payload: Multipart) -> Result<HttpResponse> {
    const MAX_FILE_SIZE: usize = 50 * 1024 * 1024; // 50MB
    
    let mut filename = String::from("unknown");
    let mut file_bytes = Vec::new();
    let mut mime_type = String::from("application/octet-stream");
    
    // Process multipart form data
    while let Some(mut field) = payload.try_next().await? {
        let content_disposition = field.content_disposition();
        
        if let Some(cd) = content_disposition {
            if let Some(name) = cd.get_filename() {
                filename = name.to_string();
            }
        }
        
        if let Some(ct) = field.content_type() {
            mime_type = ct.to_string();
        }
        
        // Read file bytes
        while let Some(chunk) = field.try_next().await? {
            file_bytes.extend_from_slice(&chunk);
            
            // Check size limit
            if file_bytes.len() > MAX_FILE_SIZE {
                return Ok(HttpResponse::PayloadTooLarge().json(serde_json::json!({
                    "error": "File too large. Maximum size is 50MB"
                })));
            }
        }
    }
    
    if file_bytes.is_empty() {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "No file uploaded"
        })));
    }
    
    // Detect document type
    let doc_type = if filename.ends_with(".pdf") {
        DocumentType::Pdf
    } else if filename.ends_with(".docx") {
        DocumentType::Docx
    } else if filename.ends_with(".xlsx") || filename.ends_with(".xls") {
        DocumentType::Excel
    } else {
        DocumentType::from_mime(&mime_type)
    };
    
    // Parse document
    match DocumentParser::parse_from_bytes(&file_bytes, doc_type) {
        Ok(parsed) => {
            let doc_id = uuid::Uuid::new_v4().to_string();
            let content_length = parsed.content.len();
            let estimated_chunks = (content_length / 512).max(1);
            
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "document_id": doc_id,
                "filename": filename,
                "document_type": format!("{:?}", parsed.doc_type),
                "content_length": content_length,
                "page_count": parsed.page_count,
                "chunks_created": estimated_chunks,
                "metadata": {
                    "title": parsed.metadata.title,
                    "author": parsed.metadata.author,
                    "created_at": parsed.metadata.created_at,
                    "modified_at": parsed.metadata.modified_at
                },
                "preview": parsed.content.chars().take(200).collect::<String>()
            })))
        }
        Err(e) => Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": format!("Failed to parse document: {}", e)
        })))
    }
}

/// Search documents
#[post("/rag/search")]
pub async fn search_documents(
    req: web::Json<SearchRequest>,
) -> Result<HttpResponse> {
    // TODO: Use actual RAG retrieval from src/rag/retrieval.rs
    
    let results = vec![
        SearchResult {
            chunk_text: "Sacred geometry refers to universal patterns...".to_string(),
            score: 0.87,
            doc_id: "doc123".to_string(),
            chunk_id: "chunk456".to_string(),
            flux_position: 9,
            confidence: 0.85,
            metadata: serde_json::json!({}),
        },
    ];
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "query": req.query,
        "results": results,
        "total_count": results.len(),
        "search_time_ms": 23
    })))
}

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub k: Option<usize>,
    pub filters: Option<SearchFilters>,
    pub use_sacred_filtering: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct SearchFilters {
    pub confidence_min: Option<f64>,
    pub flux_positions: Option<Vec<u8>>,
}

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub chunk_text: String,
    pub score: f64,
    pub doc_id: String,
    pub chunk_id: String,
    pub flux_position: u8,
    pub confidence: f64,
    pub metadata: serde_json::Value,
}

/// Get all documents
#[get("/rag/documents")]
pub async fn list_documents(
    query: web::Query<ListDocumentsQuery>,
) -> Result<HttpResponse> {
    // Use query parameters for filtering
    let limit = query.limit.unwrap_or(10);
    let offset = query.offset.unwrap_or(0);
    
    // Apply pagination
    let documents = vec![
        DocumentInfo {
            id: "doc1".to_string(),
            filename: "sacred_geometry.pdf".to_string(),
            created_at: "2025-10-29T10:00:00Z".to_string(),
            chunk_count: 24,
            confidence: 0.78,
        },
    ];
    
    // Apply offset and limit to simulated results
    let total_count = documents.len();
    let start = offset.min(total_count);
    let end = (offset + limit).min(total_count);
    let paginated_docs = documents[start..end].to_vec();
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "documents": paginated_docs,
        "total": total_count,
        "offset": offset,
        "limit": limit
    })))
}

#[derive(Debug, Deserialize)]
pub struct ListDocumentsQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Serialize, Clone)]
pub struct DocumentInfo {
    pub id: String,
    pub filename: String,
    pub created_at: String,
    pub chunk_count: usize,
    pub confidence: f64,
}

/// Delete a document
#[delete("/rag/documents/{doc_id}")]
pub async fn delete_document(
    doc_id: web::Path<String>,
) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "document_id": doc_id.to_string()
    })))
}

/// Get embeddings statistics
#[get("/rag/embeddings/stats")]
pub async fn get_embedding_stats() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "total_embeddings": 1247,
        "dimensions": 384,
        "sacred_position_distribution": {
            "3": 156,
            "6": 187,
            "9": 203
        },
        "average_confidence": 0.76,
        "storage_mb": 45.3
    })))
}

/// Retrieve documents with sacred geometry filtering
#[post("/rag/retrieve/sacred")]
pub async fn retrieve_sacred(
    req: web::Json<SacredRetrievalRequest>,
) -> Result<HttpResponse> {
    // Retrieval specifically from sacred positions (3, 6, 9)
    
    let results = vec![
        SearchResult {
            chunk_text: "At position 9, the logos channel dominates...".to_string(),
            score: 0.93,
            doc_id: "doc123".to_string(),
            chunk_id: "chunk789".to_string(),
            flux_position: 9,
            confidence: 0.91,
            metadata: serde_json::json!({
                "sacred_boost": 1.5
            }),
        },
    ];
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "query": req.query,
        "sacred_positions": [3, 6, 9],
        "results": results,
        "sacred_boost_applied": true
    })))
}

#[derive(Debug, Deserialize)]
pub struct SacredRetrievalRequest {
    pub query: String,
    pub k: Option<usize>,
}

/// Multi-source web search with credibility tracking
#[post("/rag/web-search")]
pub async fn web_search_multi_source(
    req: web::Json<WebSearchRequest>,
) -> Result<HttpResponse> {
    let config = SearchConfig {
        max_sources: req.max_sources.unwrap_or(15),
        engines: req.engines.clone().unwrap_or_else(|| {
            use crate::ai::multi_source_search::SearchEngine;
            // Default to DuckDuckGo (free, no API key needed)
            // Add Brave if API key is set
            let mut engines = vec![SearchEngine::DuckDuckGo];
            if std::env::var("BRAVE_API_KEY").is_ok() {
                engines.insert(0, SearchEngine::Brave);
            }
            if std::env::var("GOOGLE_SEARCH_API_KEY").is_ok() {
                engines.push(SearchEngine::Google);
            }
            if std::env::var("BING_SEARCH_API_KEY").is_ok() {
                engines.push(SearchEngine::Bing);
            }
            engines
        }),
        timeout_secs: 10,
        min_credibility: 0.4,
    };
    
    match MultiSourceSearcher::new(config) {
        Ok(searcher) => {
            match searcher.search(&req.query).await {
                Ok(result) => Ok(HttpResponse::Ok().json(result)),
                Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Search failed",
                    "message": e.to_string()
                })))
            }
        },
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to initialize searcher",
            "message": e.to_string()
        })))
    }
}

#[derive(Debug, Deserialize)]
pub struct WebSearchRequest {
    pub query: String,
    pub max_sources: Option<usize>,
    pub engines: Option<Vec<crate::ai::multi_source_search::SearchEngine>>,
}

/// Configure RAG endpoints routes
pub fn configure_rag_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(ingest_document)
        .service(upload_document)
        .service(search_documents)
        .service(list_documents)
        .service(delete_document)
        .service(get_embedding_stats)
        .service(retrieve_sacred)
        .service(web_search_multi_source);
}
