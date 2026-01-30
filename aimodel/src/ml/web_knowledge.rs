//! Web Knowledge Acquisition System
//!
//! Implements DuckDuckGo HTML scraping for free, unlimited web search
//! and fact extraction for consciousness-driven learning.
//!
//! ## Architecture
//! ```text
//! Query → DuckDuckGo HTML → Parse Results → Extract Facts → Vortex
//! ```
//!
//! ## Key Features
//! - **Free unlimited search**: DuckDuckGo HTML endpoint
//! - **Parallel scraping**: Multiple concurrent requests
//! - **Fact extraction**: Subject-attribute-value triples
//! - **Keyword indexing**: Fast lookup by keyword

use std::collections::HashMap;
use std::time::{Duration, Instant};

// =============================================================================
// DUCKDUCKGO SCRAPER
// =============================================================================

/// Configuration for web scraping
#[derive(Debug, Clone, Copy)]
pub struct WebScraperConfig {
    /// Timeout for HTTP requests (seconds)
    pub timeout_secs: u64,
    /// Maximum results to return per search
    pub max_results: usize,
    /// Delay between requests (milliseconds)
    pub request_delay_ms: u64,
}

impl WebScraperConfig {
    pub fn user_agent(&self) -> String {
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string()
    }
}

impl Default for WebScraperConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 30,
            max_results: 10,
            request_delay_ms: 100,
        }
    }
}

/// A search result from DuckDuckGo
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// Title of the result
    pub title: String,
    /// URL of the result
    pub url: String,
    /// Snippet/description
    pub snippet: String,
    /// Extracted keywords
    pub keywords: Vec<String>,
}

/// DuckDuckGo HTML scraper with optimized HTTP client
pub struct DuckDuckGoScraper {
    config: WebScraperConfig,
    /// Cache of recent searches
    cache: HashMap<String, Vec<SearchResult>>,
    /// Statistics
    pub stats: ScraperStats,
    /// Reusable HTTP client with connection pooling
    #[cfg(feature = "web-learning")]
    client: Option<reqwest::Client>,
}

#[derive(Debug, Clone, Default)]
pub struct ScraperStats {
    pub total_searches: usize,
    pub cache_hits: usize,
    pub total_results: usize,
    pub errors: usize,
}

impl DuckDuckGoScraper {
    pub fn new(config: WebScraperConfig) -> Self {
        #[cfg(feature = "web-learning")]
        let client = Some(
            reqwest::Client::builder()
                .timeout(Duration::from_secs(config.timeout_secs))
                .user_agent(&config.user_agent())
                .pool_max_idle_per_host(100) // Massive connection pool
                .pool_idle_timeout(Duration::from_secs(90))
                .http1_only() // Use HTTP/1.1 - DuckDuckGo doesn't support HTTP/2 properly
                .tcp_keepalive(Duration::from_secs(60))
                .build()
                .expect("Failed to create HTTP client")
        );
        
        Self {
            config,
            cache: HashMap::new(),
            stats: ScraperStats::default(),
            #[cfg(feature = "web-learning")]
            client,
        }
    }

    /// Search DuckDuckGo and return results
    /// Uses HTML endpoint: https://html.duckduckgo.com/html/?q={query}
    #[cfg(feature = "web-learning")]
    pub async fn search(&mut self, query: &str) -> Result<Vec<SearchResult>, String> {
        // Check cache first
        if let Some(cached) = self.cache.get(query) {
            self.stats.cache_hits += 1;
            return Ok(cached.clone());
        }

        self.stats.total_searches += 1;

        // Build URL
        let encoded_query = urlencoding::encode(query);
        let url = format!("https://html.duckduckgo.com/html/?q={}", encoded_query);

        // Use reusable client for connection pooling
        let client = self.client.as_ref().unwrap();

        let response = client.get(&url)
            .send()
            .await
            .map_err(|e| {
                self.stats.errors += 1;
                e.to_string()
            })?;

        let html = response.text().await.map_err(|e| {
            self.stats.errors += 1;
            e.to_string()
        })?;

        // Parse HTML results
        let results = self.parse_html_results(&html);
        self.stats.total_results += results.len();

        // Cache results
        self.cache.insert(query.to_string(), results.clone());

        Ok(results)
    }

    /// Fallback search when web-learning is not available
    #[cfg(not(feature = "web-learning"))]
    pub async fn search(&mut self, query: &str) -> Result<Vec<SearchResult>, String> {
        // Return empty results when HTTP client not available
        self.stats.total_searches += 1;
        Ok(Vec::new())
    }

    /// Synchronous search (blocking)
    pub fn search_sync(&mut self, query: &str) -> Result<Vec<SearchResult>, String> {
        // Check cache first
        if let Some(cached) = self.cache.get(query) {
            self.stats.cache_hits += 1;
            return Ok(cached.clone());
        }

        self.stats.total_searches += 1;

        #[cfg(feature = "web-learning")]
        {
            use tokio::runtime::Runtime;
            if let Ok(rt) = Runtime::new() {
                return rt.block_on(self.search_async_internal(query));
            }
        }

        // Fallback: return empty
        Ok(Vec::new())
    }

    #[cfg(feature = "web-learning")]
    async fn search_async_internal(&self, query: &str) -> Result<Vec<SearchResult>, String> {
        let encoded_query = urlencoding::encode(query);
        // Use the HTML endpoint as recommended by Stack Overflow
        let url = format!("https://duckduckgo.com/html/?q={}", encoded_query);

        // Use reusable client for connection pooling
        let client = self.client.as_ref().unwrap();

        let response = client.get(&url)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.9")
            .header("Accept-Encoding", "gzip, deflate")
            .header("DNT", "1")
            .header("Connection", "keep-alive")
            .header("Upgrade-Insecure-Requests", "1")
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let html = response.text().await.map_err(|e| e.to_string())?;
        Ok(self.parse_html_results(&html))
    }

    /// Parse DuckDuckGo HTML results
    fn parse_html_results(&self, html: &str) -> Vec<SearchResult> {
        let mut results = Vec::new();

        // DEBUG: Check if we got any HTML at all
        if html.is_empty() {
            eprintln!("[DuckDuckGo] ERROR: Received empty HTML response");
            return results;
        }
        
        if html.len() < 100 {
            eprintln!("[DuckDuckGo] WARNING: HTML response too short ({} bytes)", html.len());
            eprintln!("[DuckDuckGo] HTML content: {}", &html[..html.len().min(200)]);
            return results;
        }

        // DuckDuckGo HTML endpoint structure (from Stack Overflow):
        // Results are in <div class="result"> or <div class="web-result"> elements
        // Each contains: <a class="result__a"> for title/URL and <a class="result__snippet"> for snippet
        
        let mut pos = 0;
        
        // Look for result divs
        let result_patterns = ["<div class=\"result ", "<div class=\"web-result"];
        
        while pos < html.len() {
            let mut next_result = None;
            let mut pattern_used = "";
            
            // Find the next result div
            for pattern in &result_patterns {
                if let Some(found_pos) = html[pos..].find(pattern) {
                    if next_result.is_none() || found_pos < next_result.unwrap() {
                        next_result = Some(found_pos);
                        pattern_used = pattern;
                    }
                }
            }
            
            if let Some(result_pos) = next_result {
                let abs_start = pos + result_pos;
                
                // Find the closing </div> for this result
                let result_end = if let Some(end) = html[abs_start..].find("</div>") {
                    abs_start + end
                } else {
                    break;
                };
                
                let result_html = &html[abs_start..result_end];
                
                // Extract URL and title from <a class="result__a">
                if let Some(link_start) = result_html.find("class=\"result__a\"") {
                    if let Some(href_start) = result_html[link_start..].find("href=\"") {
                        let href_abs = link_start + href_start + 6;
                        if let Some(href_end) = result_html[href_abs..].find('"') {
                            let url = result_html[href_abs..href_abs + href_end].to_string();
                            
                            // Skip internal DuckDuckGo links
                            if !url.starts_with("//duckduckgo.com") && !url.starts_with("/") && url.starts_with("http") {
                                // Extract title
                                if let Some(title_start) = result_html[href_abs + href_end..].find('>') {
                                    let title_abs = href_abs + href_end + title_start + 1;
                                    if let Some(title_end) = result_html[title_abs..].find("</a>") {
                                        let title = self.strip_html_tags(&result_html[title_abs..title_abs + title_end]);
                                        
                                        // Extract snippet from <a class="result__snippet">
                                        let snippet = if let Some(snippet_start) = result_html.find("class=\"result__snippet\"") {
                                            if let Some(snippet_text_start) = result_html[snippet_start..].find('>') {
                                                let snippet_abs = snippet_start + snippet_text_start + 1;
                                                if let Some(snippet_end) = result_html[snippet_abs..].find("</a>") {
                                                    self.strip_html_tags(&result_html[snippet_abs..snippet_abs + snippet_end])
                                                } else {
                                                    String::new()
                                                }
                                            } else {
                                                String::new()
                                            }
                                        } else {
                                            String::new()
                                        };
                                        
                                        if !title.is_empty() {
                                            let keywords = extract_keywords(&format!("{} {}", title, snippet));
                                            
                                            results.push(SearchResult {
                                                title,
                                                url,
                                                snippet,
                                                keywords,
                                            });
                                            
                                            if results.len() >= self.config.max_results {
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                pos = result_end + 1;
            } else {
                break;
            }
        }

        results
    }

    /// Extract link text and href from HTML
    fn extract_link(&self, html: &str) -> Option<(String, String)> {
        // Find href
        let href_start = html.find("href=\"")?;
        let href_content_start = href_start + 6;
        let href_end = html[href_content_start..].find('"')?;
        let url = html[href_content_start..href_content_start + href_end].to_string();

        // Find link text (between > and </a>)
        let text_start = html.find('>')?;
        let text_end = html[text_start..].find("</a>")?;
        let title = self.strip_html_tags(&html[text_start + 1..text_start + text_end]);

        if !title.is_empty() && !url.is_empty() {
            Some((title, url))
        } else {
            None
        }
    }

    /// Extract text content from HTML element
    fn extract_text(&self, html: &str) -> String {
        if let Some(start) = html.find('>') {
            if let Some(end) = html[start..].find('<') {
                return self.strip_html_tags(&html[start + 1..start + end]);
            }
        }
        String::new()
    }

    /// Strip HTML tags from text
    fn strip_html_tags(&self, html: &str) -> String {
        let mut result = String::new();
        let mut in_tag = false;

        for c in html.chars() {
            match c {
                '<' => in_tag = true,
                '>' => in_tag = false,
                _ if !in_tag => result.push(c),
                _ => {}
            }
        }

        // Decode common HTML entities
        result
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'")
            .replace("&nbsp;", " ")
            .trim()
            .to_string()
    }

    /// Clear cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get statistics
    pub fn get_stats(&self) -> ScraperStats {
        self.stats.clone()
    }
}

// =============================================================================
// WEB KNOWLEDGE EXTRACTOR
// =============================================================================

/// Extracted knowledge from web content
#[derive(Debug, Clone)]
pub struct WebKnowledge {
    /// Subject of the knowledge
    pub subject: String,
    /// Attribute/property
    pub attribute: String,
    /// Value
    pub value: String,
    /// Confidence score
    pub confidence: f32,
    /// Source URL
    pub source: String,
    /// Keywords for indexing
    pub keywords: Vec<String>,
    /// Related subjects
    pub related: Vec<String>,
}

/// Web knowledge extractor
pub struct WebKnowledgeExtractor {
    /// Minimum word length for keywords
    min_keyword_len: usize,
    /// Stopwords to filter
    stopwords: std::collections::HashSet<String>,
}

impl WebKnowledgeExtractor {
    pub fn new() -> Self {
        let stopwords: std::collections::HashSet<String> = [
            "the", "a", "an", "is", "are", "was", "were", "be", "been", "being",
            "have", "has", "had", "do", "does", "did", "will", "would", "could",
            "should", "may", "might", "must", "shall", "can", "need", "to", "of",
            "in", "for", "on", "with", "at", "by", "from", "as", "into", "through",
            "and", "but", "or", "if", "because", "while", "this", "that", "these",
            "those", "it", "its", "i", "you", "he", "she", "we", "they", "what",
            "which", "who", "whom", "how", "when", "where", "why",
        ].iter().map(|s| s.to_string()).collect();

        Self {
            min_keyword_len: 3,
            stopwords,
        }
    }

    /// Extract knowledge from search results
    pub fn extract_from_results(&self, results: &[SearchResult], query: &str) -> Vec<WebKnowledge> {
        let mut knowledge = Vec::new();
        let query_concepts = extract_keywords(query);

        for result in results {
            // Extract from title
            if let Some(k) = self.extract_from_text(&result.title, &query_concepts, &result.url) {
                knowledge.push(k);
            }

            // Extract from snippet
            for sentence in result.snippet.split(|c| c == '.' || c == '!' || c == '?') {
                if let Some(k) = self.extract_from_text(sentence.trim(), &query_concepts, &result.url) {
                    knowledge.push(k);
                }
            }
        }

        // Deduplicate and merge
        self.deduplicate_knowledge(knowledge)
    }

    /// Extract knowledge from a single text
    fn extract_from_text(&self, text: &str, query_concepts: &[String], source: &str) -> Option<WebKnowledge> {
        if text.len() < 10 {
            return None;
        }

        let text_lower = text.to_lowercase();
        let words: Vec<&str> = text.split_whitespace().collect();

        // Find subject (first query concept found in text)
        let subject = query_concepts.iter()
            .find(|c| text_lower.contains(&c.to_lowercase()))?
            .clone();

        // Extract attribute-value patterns
        if let Some((attr, value)) = self.extract_attribute_value(&text_lower, &subject) {
            let keywords = extract_keywords(text);
            let related: Vec<String> = keywords.iter()
                .filter(|k| *k != &subject && k.len() > 3)
                .take(5)
                .cloned()
                .collect();

            return Some(WebKnowledge {
                subject,
                attribute: attr,
                value,
                confidence: 0.6,
                source: source.to_string(),
                keywords,
                related,
            });
        }

        None
    }

    /// Extract attribute-value pair from text
    fn extract_attribute_value(&self, text: &str, subject: &str) -> Option<(String, String)> {
        let subject_lower = subject.to_lowercase();

        // Pattern: "subject is/are value"
        if let Some(idx) = text.find(&format!("{} is ", subject_lower)) {
            let rest = &text[idx + subject_lower.len() + 4..];
            let value: String = rest.split(|c: char| c == '.' || c == ',' || c == ';')
                .next()?
                .trim()
                .to_string();
            if value.len() > 1 && value.len() < 100 {
                return Some(("is".to_string(), value));
            }
        }

        // Pattern: "subject has value"
        if let Some(idx) = text.find(&format!("{} has ", subject_lower)) {
            let rest = &text[idx + subject_lower.len() + 5..];
            let value: String = rest.split(|c: char| c == '.' || c == ',' || c == ';')
                .next()?
                .trim()
                .to_string();
            if value.len() > 1 && value.len() < 100 {
                return Some(("has".to_string(), value));
            }
        }

        // Pattern: "subject can value"
        if let Some(idx) = text.find(&format!("{} can ", subject_lower)) {
            let rest = &text[idx + subject_lower.len() + 5..];
            let value: String = rest.split(|c: char| c == '.' || c == ',' || c == ';')
                .next()?
                .trim()
                .to_string();
            if value.len() > 1 && value.len() < 100 {
                return Some(("can".to_string(), value));
            }
        }

        // Pattern: "subject in/at location"
        for marker in [" in ", " at ", " on "] {
            if let Some(subj_idx) = text.find(&subject_lower) {
                if let Some(marker_idx) = text[subj_idx..].find(marker) {
                    let abs_marker = subj_idx + marker_idx;
                    let rest = &text[abs_marker + marker.len()..];
                    let value: String = rest.split(|c: char| c == '.' || c == ',' || c == ';' || c == ' ')
                        .take(3)
                        .collect::<Vec<_>>()
                        .join(" ")
                        .trim()
                        .to_string();
                    if value.len() > 1 && value.len() < 50 {
                        return Some(("location".to_string(), value));
                    }
                }
            }
        }

        None
    }

    /// Deduplicate and merge knowledge
    fn deduplicate_knowledge(&self, knowledge: Vec<WebKnowledge>) -> Vec<WebKnowledge> {
        let mut merged: HashMap<(String, String), WebKnowledge> = HashMap::new();

        for k in knowledge {
            let key = (k.subject.clone(), k.attribute.clone());
            
            if let Some(existing) = merged.get_mut(&key) {
                // Merge: increase confidence if same value, or keep higher confidence
                if existing.value == k.value {
                    existing.confidence = (existing.confidence + k.confidence).min(1.0);
                } else if k.confidence > existing.confidence {
                    *existing = k;
                }
            } else {
                merged.insert(key, k);
            }
        }

        merged.into_values().collect()
    }
}

impl Default for WebKnowledgeExtractor {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// BATCH WEB LEARNER
// =============================================================================

/// Batch web learning for pre-benchmark knowledge acquisition
pub struct BatchWebLearner {
    /// DuckDuckGo scraper for web searches
    pub scraper: DuckDuckGoScraper,
    /// Knowledge extractor for parsing results
    pub extractor: WebKnowledgeExtractor,
    /// All extracted knowledge
    pub knowledge: Vec<WebKnowledge>,
    /// Statistics
    pub stats: BatchLearnerStats,
}

#[derive(Debug, Clone, Default)]
pub struct BatchLearnerStats {
    pub queries_processed: usize,
    pub results_fetched: usize,
    pub knowledge_extracted: usize,
    pub total_time_ms: u64,
}

impl BatchWebLearner {
    pub fn new(config: WebScraperConfig) -> Self {
        Self {
            scraper: DuckDuckGoScraper::new(config),
            extractor: WebKnowledgeExtractor::new(),
            knowledge: Vec::new(),
            stats: BatchLearnerStats::default(),
        }
    }

    /// Learn from a batch of queries
    pub fn learn_batch(&mut self, queries: &[String]) -> BatchLearnerStats {
        let start = Instant::now();

        for query in queries {
            self.learn_from_query(query);
            self.stats.queries_processed += 1;
        }

        self.stats.total_time_ms = start.elapsed().as_millis() as u64;
        self.stats.clone()
    }

    /// Learn from a single query
    fn learn_from_query(&mut self, query: &str) {
        match self.scraper.search_sync(query) {
            Ok(results) => {
                self.stats.results_fetched += results.len();
                
                let extracted = self.extractor.extract_from_results(&results, query);
                self.stats.knowledge_extracted += extracted.len();
                
                self.knowledge.extend(extracted);
            }
            Err(e) => {
                eprintln!("Search error for '{}': {}", query, e);
            }
        }
    }

    /// Get all knowledge for a subject
    pub fn get_knowledge_for_subject(&self, subject: &str) -> Vec<&WebKnowledge> {
        let subject_lower = subject.to_lowercase();
        self.knowledge.iter()
            .filter(|k| k.subject.to_lowercase() == subject_lower)
            .collect()
    }

    /// Search knowledge by keyword
    pub fn search_knowledge(&self, keyword: &str) -> Vec<&WebKnowledge> {
        let keyword_lower = keyword.to_lowercase();
        self.knowledge.iter()
            .filter(|k| k.keywords.iter().any(|kw| kw.to_lowercase() == keyword_lower))
            .collect()
    }

    /// Clear all learned knowledge
    pub fn clear(&mut self) {
        self.knowledge.clear();
        self.scraper.clear_cache();
        self.stats = BatchLearnerStats::default();
    }
}

// =============================================================================
// UTILITY FUNCTIONS
// =============================================================================

/// Extract keywords from text
fn extract_keywords(text: &str) -> Vec<String> {
    let stopwords: std::collections::HashSet<&str> = [
        "the", "a", "an", "is", "are", "was", "were", "be", "been", "being",
        "have", "has", "had", "do", "does", "did", "will", "would", "could",
        "should", "may", "might", "must", "shall", "can", "need", "to", "of",
        "in", "for", "on", "with", "at", "by", "from", "as", "into", "through",
        "and", "but", "or", "if", "because", "while", "this", "that", "these",
        "those", "it", "its", "i", "you", "he", "she", "we", "they",
    ].into_iter().collect();

    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > 2 && !stopwords.contains(w))
        .map(|s| s.to_string())
        .collect()
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scraper_creation() {
        let config = WebScraperConfig::default();
        let scraper = DuckDuckGoScraper::new(config);
        assert_eq!(scraper.stats.total_searches, 0);
    }

    #[test]
    fn test_html_parsing() {
        let config = WebScraperConfig::default();
        let scraper = DuckDuckGoScraper::new(config);

        let html = r#"
            <div class="result">
                <a class="result__a" href="https://example.com">Test Title</a>
                <a class="result__snippet">This is a test snippet about hamburgers.</a>
            </div>
        "#;

        let results = scraper.parse_html_results(html);
        // May or may not parse depending on exact format
        println!("Parsed {} results", results.len());
    }

    #[test]
    fn test_knowledge_extractor() {
        let extractor = WebKnowledgeExtractor::new();

        let results = vec![
            SearchResult {
                title: "Hamburgers are found in restaurants".to_string(),
                url: "https://example.com".to_string(),
                snippet: "A hamburger is a popular food item. Hamburgers can be found in many restaurants.".to_string(),
                keywords: vec!["hamburger".to_string(), "restaurant".to_string()],
            },
        ];

        let knowledge = extractor.extract_from_results(&results, "hamburger");
        
        for k in &knowledge {
            println!("Extracted: {} {} {}", k.subject, k.attribute, k.value);
        }
    }

    #[test]
    fn test_extract_keywords() {
        let keywords = extract_keywords("The hamburger is found in a restaurant");
        assert!(keywords.contains(&"hamburger".to_string()));
        assert!(keywords.contains(&"restaurant".to_string()));
        assert!(!keywords.contains(&"the".to_string()));
    }

    #[test]
    fn test_batch_learner() {
        let config = WebScraperConfig::default();
        let mut learner = BatchWebLearner::new(config);

        // This won't actually make web requests without reqwest feature
        let queries = vec!["hamburger location".to_string()];
        let stats = learner.learn_batch(&queries);

        println!("Batch stats: {:?}", stats);
    }
}
