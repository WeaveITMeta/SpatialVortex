//! Multi-Source Web Search Engine
//!
//! Aggregates results from multiple search engines (Brave, Google, Bing, DuckDuckGo)
//! with credibility scoring and source deduplication

use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Configuration for multi-source search
#[derive(Debug, Clone)]
pub struct SearchConfig {
    /// Maximum sources to return
    pub max_sources: usize,
    /// Engines to use
    pub engines: Vec<SearchEngine>,
    /// Request timeout in seconds
    pub timeout_secs: u64,
    /// Minimum credibility threshold (0.0-1.0)
    pub min_credibility: f32,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            max_sources: 15,  // Aggregate top 15 sources
            engines: vec![
                SearchEngine::Brave,
                SearchEngine::DuckDuckGo,
                SearchEngine::Bing,
                SearchEngine::Google,
            ],
            timeout_secs: 10,
            min_credibility: 0.4,
        }
    }
}

/// Available search engines
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SearchEngine {
    Brave,
    Google,
    Bing,
    DuckDuckGo,
}

impl SearchEngine {
    pub fn name(&self) -> &str {
        match self {
            SearchEngine::Brave => "brave",
            SearchEngine::Google => "google",
            SearchEngine::Bing => "bing",
            SearchEngine::DuckDuckGo => "duckduckgo",
        }
    }
    
    /// Weight for ranking (higher = more trusted)
    pub fn weight(&self) -> f32 {
        match self {
            SearchEngine::Brave => 1.0,      // Privacy-focused, high quality
            SearchEngine::Google => 0.95,    // Most comprehensive
            SearchEngine::Bing => 0.85,      // Good coverage
            SearchEngine::DuckDuckGo => 0.80, // Privacy-focused
        }
    }
}

/// Individual web source with credibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSource {
    pub url: String,
    pub title: String,
    pub snippet: String,
    pub credibility_score: f32,
    pub source_type: SourceType,
    pub timestamp: String,
    pub search_engine: String,
    pub relevance_score: f32,
    pub domain: String,
    /// Published or updated date (ISO 8601 format)
    pub published_date: Option<String>,
    /// Freshness score (0.0-1.0) - higher for more recent content
    pub freshness_score: f32,
    /// User rating (1-5 stars), None if not rated
    pub user_rating: Option<f32>,
    /// Is this source bookmarked by the user?
    pub is_bookmarked: bool,
}

/// Source type classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SourceType {
    Academic,      // .edu, arxiv.org, scholar.google.com, etc.
    Government,    // .gov
    Wikipedia,     // wikipedia.org
    Technical,     // stackoverflow, github, docs sites
    News,          // reuters, ap, nyt, wsj, etc.
    Reference,     // britannica, dictionary, etc.
    Commercial,    // .com, .net
    Unknown,
}

impl WebSource {
    /// Calculate credibility based on domain and source type
    pub fn calculate_credibility(&self, engine_weight: f32) -> f32 {
        let base_score = match self.source_type {
            SourceType::Academic => 0.95,     // Highest trust
            SourceType::Government => 0.90,   // Very high trust
            SourceType::Reference => 0.85,    // High trust
            SourceType::News => 0.75,         // Good trust
            SourceType::Wikipedia => 0.70,    // Moderate trust
            SourceType::Technical => 0.75,    // Good for tech queries
            SourceType::Commercial => 0.50,   // Moderate trust
            SourceType::Unknown => 0.35,      // Low trust
        };
        
        // Boost for HTTPS
        let https_boost = if self.url.starts_with("https://") { 0.05 } else { 0.0 };
        
        // Boost for common high-quality domains
        let domain_boost = self.get_domain_boost();
        
        // Calculate final score: base × relevance × engine_weight + boosts
        let score = (base_score * self.relevance_score * engine_weight) + https_boost + domain_boost;
        
        score.min(1.0)
    }
    
    fn get_domain_boost(&self) -> f32 {
        let domain = self.domain.to_lowercase();
        
        // High-quality domain list
        if domain.contains("arxiv.org") || domain.contains("ieee.org") {
            return 0.10;
        }
        if domain.contains("nature.com") || domain.contains("science.org") {
            return 0.08;
        }
        if domain.contains("nytimes.com") || domain.contains("reuters.com") {
            return 0.05;
        }
        
        0.0
    }
    
    /// Determine source type from URL
    pub fn classify_source(url: &str) -> SourceType {
        let url_lower = url.to_lowercase();
        
        // Academic
        if url_lower.contains(".edu") 
            || url_lower.contains("arxiv.org") 
            || url_lower.contains("scholar.google")
            || url_lower.contains("ieee.org")
            || url_lower.contains("acm.org")
            || url_lower.contains("nature.com")
            || url_lower.contains("science.org") {
            return SourceType::Academic;
        }
        
        // Government
        if url_lower.contains(".gov") || url_lower.contains(".mil") {
            return SourceType::Government;
        }
        
        // Wikipedia
        if url_lower.contains("wikipedia.org") {
            return SourceType::Wikipedia;
        }
        
        // Reference
        if url_lower.contains("britannica.com") 
            || url_lower.contains("dictionary.com")
            || url_lower.contains("merriam-webster") {
            return SourceType::Reference;
        }
        
        // Technical
        if url_lower.contains("stackoverflow.com") 
            || url_lower.contains("github.com")
            || url_lower.contains("docs.") 
            || url_lower.contains("developer.") {
            return SourceType::Technical;
        }
        
        // News
        if url_lower.contains("reuters.com") 
            || url_lower.contains("apnews.com") 
            || url_lower.contains("nytimes.com")
            || url_lower.contains("wsj.com")
            || url_lower.contains("bbc.com")
            || url_lower.contains("cnn.com")
            || url_lower.contains("npr.org") {
            return SourceType::News;
        }
        
        // Commercial
        if url_lower.contains(".com") || url_lower.contains(".net") || url_lower.contains(".org") {
            return SourceType::Commercial;
        }
        
        SourceType::Unknown
    }
    
    /// Extract domain from URL
    pub fn extract_domain(url: &str) -> String {
        url.split("://")
            .nth(1)
            .and_then(|s| s.split('/').next())
            .unwrap_or(url)
            .to_string()
    }
    
    /// Calculate freshness score based on publication date
    /// Returns 1.0 for very recent (< 1 month), decays over time
    pub fn calculate_freshness(published_date: Option<&str>) -> f32 {
        let Some(date_str) = published_date else {
            return 0.5; // Default if no date available
        };
        
        use chrono::{DateTime, Utc};
        
        // Try to parse the date
        let published = match DateTime::parse_from_rfc3339(date_str)
            .or_else(|_| DateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S %z"))
            .or_else(|_| DateTime::parse_from_str(&format!("{} 00:00:00 +0000", date_str), "%Y-%m-%d %H:%M:%S %z"))
        {
            Ok(dt) => dt.with_timezone(&Utc),
            Err(_) => return 0.5, // Can't parse, use default
        };
        
        let now = Utc::now();
        let age_days = (now - published).num_days() as f32;
        
        // Freshness decay function
        // < 7 days: 1.0 (perfect freshness)
        // < 30 days: 0.9-1.0 (very fresh)
        // < 90 days: 0.7-0.9 (fresh)
        // < 180 days: 0.5-0.7 (moderate)
        // < 365 days: 0.3-0.5 (aging)
        // > 365 days: 0.1-0.3 (old)
        if age_days < 0.0 {
            1.0 // Future date, treat as fresh
        } else if age_days < 7.0 {
            1.0
        } else if age_days < 30.0 {
            1.0 - (age_days - 7.0) / 230.0 // 1.0 -> 0.9
        } else if age_days < 90.0 {
            0.9 - (age_days - 30.0) / 300.0 // 0.9 -> 0.7
        } else if age_days < 180.0 {
            0.7 - (age_days - 90.0) / 450.0 // 0.7 -> 0.5
        } else if age_days < 365.0 {
            0.5 - (age_days - 180.0) / 925.0 // 0.5 -> 0.3
        } else {
            let years = age_days / 365.0;
            (0.3 - (years - 1.0) * 0.05).max(0.1) // Decay to min 0.1
        }
    }
    
    /// Boost credibility based on freshness for time-sensitive queries
    pub fn apply_freshness_boost(&self, is_time_sensitive: bool) -> f32 {
        if !is_time_sensitive {
            return self.credibility_score;
        }
        
        // For time-sensitive queries, boost recent sources
        let freshness_weight = 0.2; // 20% weight for freshness
        let base_weight = 0.8; // 80% weight for credibility
        
        self.credibility_score * base_weight + self.freshness_score * freshness_weight
    }
}

/// Multi-source search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiSourceResult {
    pub query: String,
    pub sources: Vec<WebSource>,
    pub aggregated_answer: String,
    pub confidence: f32,
    pub search_engines_used: Vec<String>,
    pub total_results: usize,
    pub search_time_ms: u64,
}

/// Main multi-source search aggregator
pub struct MultiSourceSearcher {
    config: SearchConfig,
    client: Client,
    brave_api_key: Option<String>,
    google_api_key: Option<String>,
    bing_api_key: Option<String>,
}

impl MultiSourceSearcher {
    pub fn new(config: SearchConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()?;
        
        // Load API keys from environment
        let brave_api_key = std::env::var("BRAVE_API_KEY").ok();
        let google_api_key = std::env::var("GOOGLE_SEARCH_API_KEY").ok();
        let bing_api_key = std::env::var("BING_SEARCH_API_KEY").ok();
        
        Ok(Self {
            config,
            client,
            brave_api_key,
            google_api_key,
            bing_api_key,
        })
    }
    
    /// Search across all configured engines
    pub async fn search(&self, query: &str) -> Result<MultiSourceResult> {
        let start_time = std::time::Instant::now();
        let mut all_sources = Vec::new();
        let mut engines_used = Vec::new();
        
        // Search each engine in parallel
        let mut tasks = Vec::new();
        
        for engine in &self.config.engines {
            let query = query.to_string();
            let engine_clone = engine.clone();
            
            let task = match engine {
                SearchEngine::Brave if self.brave_api_key.is_some() => {
                    let searcher = self.clone_for_brave();
                    Some(tokio::spawn(async move {
                        searcher.search_brave(&query).await
                    }))
                },
                SearchEngine::Google if self.google_api_key.is_some() => {
                    let searcher = self.clone_for_google();
                    Some(tokio::spawn(async move {
                        searcher.search_google(&query).await
                    }))
                },
                SearchEngine::Bing if self.bing_api_key.is_some() => {
                    let searcher = self.clone_for_bing();
                    Some(tokio::spawn(async move {
                        searcher.search_bing(&query).await
                    }))
                },
                SearchEngine::DuckDuckGo => {
                    let searcher = self.clone_for_ddg();
                    Some(tokio::spawn(async move {
                        searcher.search_duckduckgo(&query).await
                    }))
                },
                _ => None,
            };
            
            if let Some(task) = task {
                tasks.push((engine_clone, task));
            }
        }
        
        // Collect results
        for (engine, task) in tasks {
            match task.await {
                Ok(Ok(sources)) => {
                    engines_used.push(engine.name().to_string());
                    all_sources.extend(sources);
                },
                Ok(Err(e)) => {
                    eprintln!("Error searching {}: {}", engine.name(), e);
                },
                Err(e) => {
                    eprintln!("Task error for {}: {}", engine.name(), e);
                }
            }
        }
        
        // Aggregate and rank sources
        let ranked_sources = self.aggregate_sources(all_sources)?;
        
        // Generate aggregated answer
        let (answer, confidence) = self.generate_answer(&ranked_sources, query);
        
        let search_time_ms = start_time.elapsed().as_millis() as u64;
        
        Ok(MultiSourceResult {
            query: query.to_string(),
            sources: ranked_sources.clone(),
            aggregated_answer: answer,
            confidence,
            search_engines_used: engines_used,
            total_results: ranked_sources.len(),
            search_time_ms,
        })
    }
    
    /// Search Brave Search API
    async fn search_brave(&self, query: &str) -> Result<Vec<WebSource>> {
        let api_key = self.brave_api_key.as_ref()
            .ok_or_else(|| anyhow!("Brave API key not set"))?;
        
        let url = format!(
            "https://api.search.brave.com/res/v1/web/search?q={}&count=10",
            urlencoding::encode(query)
        );
        
        let response = self.client
            .get(&url)
            .header("Accept", "application/json")
            .header("Accept-Encoding", "gzip")
            .header("X-Subscription-Token", api_key)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Brave API error: {}", response.status()));
        }
        
        let json: serde_json::Value = response.json().await?;
        let mut sources = Vec::new();
        
        if let Some(results) = json["web"]["results"].as_array() {
            for result in results {
                let url = result["url"].as_str().unwrap_or("").to_string();
                let domain = WebSource::extract_domain(&url);
                
                let published_date = result["published"].as_str().map(|s| s.to_string());
                let freshness_score = WebSource::calculate_freshness(published_date.as_deref());
                
                sources.push(WebSource {
                    url: url.clone(),
                    title: result["title"].as_str().unwrap_or("").to_string(),
                    snippet: result["description"].as_str().unwrap_or("").to_string(),
                    credibility_score: 0.0,  // Will be calculated
                    source_type: WebSource::classify_source(&url),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    search_engine: "brave".to_string(),
                    relevance_score: 0.9,  // Brave has good relevance
                    domain,
                    published_date,
                    freshness_score,
                    user_rating: None,
                    is_bookmarked: false,
                });
            }
        }
        
        Ok(sources)
    }
    
    /// Search DuckDuckGo (Instant Answer API - Free, no key needed)
    async fn search_duckduckgo(&self, query: &str) -> Result<Vec<WebSource>> {
        // DuckDuckGo Instant Answer API (free, no authentication)
        // Documentation: https://duckduckgo.com/api
        
        let url = format!(
            "https://api.duckduckgo.com/?q={}&format=json&no_html=1&skip_disambig=1",
            urlencoding::encode(query)
        );
        
        let response = self.client
            .get(&url)
            .header("User-Agent", "SpatialVortex/1.0")
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("DuckDuckGo API error: {}", response.status()));
        }
        
        let json: serde_json::Value = response.json().await?;
        let mut sources = Vec::new();
        
        // Parse Abstract (main answer)
        if let Some(abstract_text) = json["Abstract"].as_str() {
            if !abstract_text.is_empty() {
                if let Some(abstract_url) = json["AbstractURL"].as_str() {
                    if !abstract_url.is_empty() {
                        let domain = WebSource::extract_domain(abstract_url);
                        let freshness_score = WebSource::calculate_freshness(None);
                        sources.push(WebSource {
                            url: abstract_url.to_string(),
                            title: json["Heading"].as_str().unwrap_or("").to_string(),
                            snippet: abstract_text.to_string(),
                            credibility_score: 0.0,
                            source_type: WebSource::classify_source(abstract_url),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                            search_engine: "duckduckgo".to_string(),
                            relevance_score: 0.95,  // Abstract is highly relevant
                            domain,
                            published_date: None,
                            freshness_score,
                            user_rating: None,
                            is_bookmarked: false,
                        });
                    }
                }
            }
        }
        
        // Parse RelatedTopics (additional results)
        if let Some(related) = json["RelatedTopics"].as_array() {
            for item in related.iter().take(10) {  // Limit to 10 results
                // Handle direct results
                if let (Some(text), Some(url)) = (
                    item["Text"].as_str(),
                    item["FirstURL"].as_str()
                ) {
                    if !text.is_empty() && !url.is_empty() {
                        let domain = WebSource::extract_domain(url);
                        let freshness_score = WebSource::calculate_freshness(None);
                        sources.push(WebSource {
                            url: url.to_string(),
                            title: text.split(" - ").next().unwrap_or(text).to_string(),
                            snippet: text.to_string(),
                            credibility_score: 0.0,
                            source_type: WebSource::classify_source(url),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                            search_engine: "duckduckgo".to_string(),
                            relevance_score: 0.85,
                            domain,
                            published_date: None,
                            freshness_score,
                            user_rating: None,
                            is_bookmarked: false,
                        });
                    }
                }
                
                // Handle nested topics
                if let Some(topics) = item["Topics"].as_array() {
                    for topic in topics.iter().take(5) {
                        if let (Some(text), Some(url)) = (
                            topic["Text"].as_str(),
                            topic["FirstURL"].as_str()
                        ) {
                            if !text.is_empty() && !url.is_empty() {
                                let domain = WebSource::extract_domain(url);
                                let freshness_score = WebSource::calculate_freshness(None);
                                sources.push(WebSource {
                                    url: url.to_string(),
                                    title: text.split(" - ").next().unwrap_or(text).to_string(),
                                    snippet: text.to_string(),
                                    credibility_score: 0.0,
                                    source_type: WebSource::classify_source(url),
                                    timestamp: chrono::Utc::now().to_rfc3339(),
                                    search_engine: "duckduckgo".to_string(),
                                    relevance_score: 0.80,
                                    domain,
                                    published_date: None,
                                    freshness_score,
                                    user_rating: None,
                                    is_bookmarked: false,
                                });
                            }
                        }
                    }
                }
            }
        }
        
        // Parse Results (additional links)
        if let Some(results) = json["Results"].as_array() {
            for result in results.iter().take(5) {
                if let (Some(text), Some(url)) = (
                    result["Text"].as_str(),
                    result["FirstURL"].as_str()
                ) {
                    if !text.is_empty() && !url.is_empty() {
                        let domain = WebSource::extract_domain(url);
                        let freshness_score = WebSource::calculate_freshness(None);
                        sources.push(WebSource {
                            url: url.to_string(),
                            title: text.split(" - ").next().unwrap_or(text).to_string(),
                            snippet: text.to_string(),
                            credibility_score: 0.0,
                            source_type: WebSource::classify_source(url),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                            search_engine: "duckduckgo".to_string(),
                            relevance_score: 0.85,
                            domain,
                            published_date: None,
                            freshness_score,
                            user_rating: None,
                            is_bookmarked: false,
                        });
                    }
                }
            }
        }
        
        Ok(sources)
    }
    
    /// Search Google Custom Search API
    async fn search_google(&self, _query: &str) -> Result<Vec<WebSource>> {
        // TODO: Implement Google Custom Search API
        // Requires: GOOGLE_SEARCH_API_KEY and GOOGLE_SEARCH_ENGINE_ID
        Ok(vec![])
    }
    
    /// Search Bing Search API
    async fn search_bing(&self, _query: &str) -> Result<Vec<WebSource>> {
        // TODO: Implement Bing Search API
        Ok(vec![])
    }
    
    /// Aggregate and rank sources from all engines
    fn aggregate_sources(&self, sources: Vec<WebSource>) -> Result<Vec<WebSource>> {
        let mut scored_sources = sources;
        
        // Calculate credibility scores
        for source in &mut scored_sources {
            let engine_weight = self.get_engine_weight(&source.search_engine);
            source.credibility_score = source.calculate_credibility(engine_weight);
        }
        
        // Filter by minimum credibility
        scored_sources.retain(|s| s.credibility_score >= self.config.min_credibility);
        
        // Sort by credibility (highest first)
        scored_sources.sort_by(|a, b| {
            b.credibility_score.partial_cmp(&a.credibility_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Deduplicate by URL
        let mut seen_urls = HashSet::new();
        scored_sources.retain(|s| seen_urls.insert(s.url.clone()));
        
        // Deduplicate by domain (keep highest scoring)
        let mut seen_domains = HashMap::new();
        let mut final_sources = Vec::new();
        
        for source in scored_sources {
            let domain = &source.domain;
            
            if let Some(existing_score) = seen_domains.get(domain) {
                // Only keep if this source is significantly better
                if source.credibility_score > existing_score + 0.1 {
                    seen_domains.insert(domain.clone(), source.credibility_score);
                    final_sources.push(source);
                }
            } else {
                seen_domains.insert(domain.clone(), source.credibility_score);
                final_sources.push(source);
            }
        }
        
        // Limit to max_sources
        final_sources.truncate(self.config.max_sources);
        
        Ok(final_sources)
    }
    
    /// Generate aggregated answer from sources
    fn generate_answer(&self, sources: &[WebSource], _query: &str) -> (String, f32) {
        if sources.is_empty() {
            return ("No reliable sources found.".to_string(), 0.0);
        }
        
        // Take top 5 most credible sources
        let top_sources: Vec<_> = sources.iter().take(5).collect();
        
        // Calculate confidence based on source quality
        let avg_credibility: f32 = top_sources.iter()
            .map(|s| s.credibility_score)
            .sum::<f32>() / top_sources.len() as f32;
        
        // Combine snippets with source attribution
        let combined_answer = top_sources.iter()
            .enumerate()
            .map(|(i, s)| {
                let source_icon = match s.source_type {
                    SourceType::Academic => "🎓",
                    SourceType::Government => "🏛️",
                    SourceType::Wikipedia => "📖",
                    SourceType::Technical => "💻",
                    SourceType::News => "📰",
                    SourceType::Reference => "📚",
                    _ => "🌐",
                };
                
                format!(
                    "{}. {} **{}** (Credibility: {:.0}%)\n   {}\n   Source: {}",
                    i + 1,
                    source_icon,
                    s.title,
                    s.credibility_score * 100.0,
                    s.snippet,
                    s.domain
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n");
        
        (combined_answer, avg_credibility)
    }
    
    fn get_engine_weight(&self, engine_name: &str) -> f32 {
        match engine_name {
            "brave" => SearchEngine::Brave.weight(),
            "google" => SearchEngine::Google.weight(),
            "bing" => SearchEngine::Bing.weight(),
            "duckduckgo" => SearchEngine::DuckDuckGo.weight(),
            _ => 0.5,
        }
    }
    
    // Clone methods for async tasks
    fn clone_for_brave(&self) -> Self {
        Self {
            config: self.config.clone(),
            client: self.client.clone(),
            brave_api_key: self.brave_api_key.clone(),
            google_api_key: None,
            bing_api_key: None,
        }
    }
    
    fn clone_for_google(&self) -> Self {
        Self {
            config: self.config.clone(),
            client: self.client.clone(),
            brave_api_key: None,
            google_api_key: self.google_api_key.clone(),
            bing_api_key: None,
        }
    }
    
    fn clone_for_bing(&self) -> Self {
        Self {
            config: self.config.clone(),
            client: self.client.clone(),
            brave_api_key: None,
            google_api_key: None,
            bing_api_key: self.bing_api_key.clone(),
        }
    }
    
    fn clone_for_ddg(&self) -> Self {
        Self {
            config: self.config.clone(),
            client: self.client.clone(),
            brave_api_key: None,
            google_api_key: None,
            bing_api_key: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_source_classification() {
        assert_eq!(
            WebSource::classify_source("https://arxiv.org/abs/12345"),
            SourceType::Academic
        );
        
        assert_eq!(
            WebSource::classify_source("https://www.whitehouse.gov/"),
            SourceType::Government
        );
        
        assert_eq!(
            WebSource::classify_source("https://en.wikipedia.org/wiki/Test"),
            SourceType::Wikipedia
        );
    }
    
    #[test]
    fn test_domain_extraction() {
        assert_eq!(
            WebSource::extract_domain("https://www.example.com/path/to/page"),
            "www.example.com"
        );
    }
}
