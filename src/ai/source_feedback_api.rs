//! Source Feedback API Endpoints
//!
//! Handles user ratings and bookmarks for web sources

use actix_web::{post, get, web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// In-memory storage for user feedback (in production, use database)
pub struct FeedbackStore {
    ratings: Arc<RwLock<HashMap<String, f32>>>,      // url -> rating
    bookmarks: Arc<RwLock<HashMap<String, bool>>>,   // url -> is_bookmarked
}

impl FeedbackStore {
    pub fn new() -> Self {
        Self {
            ratings: Arc::new(RwLock::new(HashMap::new())),
            bookmarks: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for FeedbackStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Rate source request
#[derive(Debug, Deserialize)]
pub struct RateSourceRequest {
    pub url: String,
    pub rating: f32,  // 1-5 stars
}

/// Bookmark source request
#[derive(Debug, Deserialize)]
pub struct BookmarkSourceRequest {
    pub url: String,
    pub bookmarked: bool,
}

/// Get user ratings response
#[derive(Debug, Serialize)]
pub struct UserRatingsResponse {
    pub ratings: HashMap<String, f32>,
    pub total_ratings: usize,
}

/// Get bookmarks response
#[derive(Debug, Serialize)]
pub struct BookmarksResponse {
    pub bookmarks: Vec<String>,
    pub total_bookmarks: usize,
}

/// Rate a source (1-5 stars)
/// 
/// POST /api/v1/sources/rate
#[post("/sources/rate")]
pub async fn rate_source(
    req: web::Json<RateSourceRequest>,
    store: web::Data<Arc<FeedbackStore>>,
) -> Result<HttpResponse> {
    // Validate rating
    if req.rating < 1.0 || req.rating > 5.0 {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid rating",
            "message": "Rating must be between 1.0 and 5.0"
        })));
    }
    
    // Store rating
    let mut ratings = store.ratings.write().await;
    ratings.insert(req.url.clone(), req.rating);
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "url": req.url,
        "rating": req.rating,
        "message": "Rating saved successfully"
    })))
}

/// Toggle bookmark for a source
/// 
/// POST /api/v1/sources/bookmark
#[post("/sources/bookmark")]
pub async fn bookmark_source(
    req: web::Json<BookmarkSourceRequest>,
    store: web::Data<Arc<FeedbackStore>>,
) -> Result<HttpResponse> {
    let mut bookmarks = store.bookmarks.write().await;
    
    if req.bookmarked {
        bookmarks.insert(req.url.clone(), true);
    } else {
        bookmarks.remove(&req.url);
    }
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "url": req.url,
        "bookmarked": req.bookmarked,
        "message": if req.bookmarked { "Source bookmarked" } else { "Bookmark removed" }
    })))
}

/// Get all user ratings
/// 
/// GET /api/v1/sources/ratings
#[get("/sources/ratings")]
pub async fn get_user_ratings(
    store: web::Data<Arc<FeedbackStore>>,
) -> Result<HttpResponse> {
    let ratings = store.ratings.read().await;
    let ratings_map = ratings.clone();
    
    Ok(HttpResponse::Ok().json(UserRatingsResponse {
        total_ratings: ratings_map.len(),
        ratings: ratings_map,
    }))
}

/// Get all bookmarked sources
/// 
/// GET /api/v1/sources/bookmarks
#[get("/sources/bookmarks")]
pub async fn get_bookmarks(
    store: web::Data<Arc<FeedbackStore>>,
) -> Result<HttpResponse> {
    let bookmarks = store.bookmarks.read().await;
    let bookmarked_urls: Vec<String> = bookmarks
        .iter()
        .filter(|(_, &bookmarked)| bookmarked)
        .map(|(url, _)| url.clone())
        .collect();
    
    Ok(HttpResponse::Ok().json(BookmarksResponse {
        total_bookmarks: bookmarked_urls.len(),
        bookmarks: bookmarked_urls,
    }))
}

/// Configure source feedback routes
pub fn configure_feedback_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(rate_source)
        .service(bookmark_source)
        .service(get_user_ratings)
        .service(get_bookmarks);
}
