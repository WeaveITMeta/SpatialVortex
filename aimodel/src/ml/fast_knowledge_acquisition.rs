//! Fast Knowledge Acquisition System
//! 
//! High-throughput knowledge extraction using in-house web crawler.
//! Replaces slow Wikipedia API with parallel crawling for maximum speed.

use crate::ml::web_crawler::{WebCrawler, CrawlerConfig, CrawledPage};
use crate::ml::web_knowledge::{WebKnowledgeExtractor, WebKnowledge};
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Fast knowledge acquisition configuration
#[derive(Debug, Clone)]
pub struct FastKnowledgeConfig {
    /// Crawler configuration
    pub crawler_config: CrawlerConfig,
    /// Maximum knowledge entries per query
    pub max_knowledge_per_query: usize,
    /// Enable parallel knowledge extraction
    pub parallel_extraction: bool,
}

impl Default for FastKnowledgeConfig {
    fn default() -> Self {
        Self {
            // Use fast_eval preset to avoid runaway crawl at test time
            crawler_config: CrawlerConfig::fast_eval(),
            max_knowledge_per_query: 50,
            parallel_extraction: true,
        }
    }
}

/// Fast knowledge acquisition system
pub struct FastKnowledgeAcquisition {
    config: FastKnowledgeConfig,
    crawler: WebCrawler,
    extractor: WebKnowledgeExtractor,
    cache: Arc<RwLock<HashMap<String, Vec<WebKnowledge>>>>,
}

impl FastKnowledgeAcquisition {
    /// Create a new fast knowledge acquisition system
    pub fn new(config: FastKnowledgeConfig) -> anyhow::Result<Self> {
        let crawler = WebCrawler::new(config.crawler_config.clone())?;
        let extractor = WebKnowledgeExtractor::new();
        let cache = Arc::new(RwLock::new(HashMap::new()));

        Ok(Self {
            config,
            crawler,
            extractor,
            cache,
        })
    }

    /// Learn from a query using fast parallel crawling
    pub async fn learn_from_query(&self, query: &str) -> Vec<WebKnowledge> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(query) {
                return cached.clone();
            }
        }

        // Generate search URLs for the query
        let search_urls = self.generate_search_urls(query);

        // Crawl pages in parallel
        let pages = self.crawler.crawl_batch(search_urls).await;

        // Extract knowledge from crawled pages using Rayon for parallelism
        let mut knowledge: Vec<WebKnowledge> = if self.config.parallel_extraction {
            pages.par_iter()
                .flat_map(|page| self.extract_knowledge_from_page(page))
                .collect()
        } else {
            pages.iter()
                .flat_map(|page| self.extract_knowledge_from_page(page))
                .collect()
        };
        
        // Limit to max knowledge per query
        knowledge.truncate(self.config.max_knowledge_per_query);

        // Cache results
        {
            let mut cache = self.cache.write().await;
            cache.insert(query.to_string(), knowledge.clone());
        }

        knowledge
    }

    /// Learn from multiple queries in parallel
    pub async fn learn_from_queries(&self, queries: &[String]) -> HashMap<String, Vec<WebKnowledge>> {
        let mut results = HashMap::new();

        // Process queries in parallel using Tokio
        let futures: Vec<_> = queries.iter()
            .map(|query| {
                let query = query.clone();
                async move {
                    let knowledge = self.learn_from_query(&query).await;
                    (query, knowledge)
                }
            })
            .collect();

        let results_vec = futures::future::join_all(futures).await;

        for (query, knowledge) in results_vec {
            results.insert(query, knowledge);
        }

        results
    }

    /// Generate search URLs for a query
    fn generate_search_urls(&self, query: &str) -> Vec<String> {
        let encoded_query = urlencoding::encode(query);
        
        // Use multiple search engines and knowledge sources for diversity
        vec![
            // Wikipedia (still useful but now crawled, not API)
            format!("https://en.wikipedia.org/wiki/{}", encoded_query),
            format!("https://en.wikipedia.org/w/index.php?search={}", encoded_query),
            
            // Academic sources
            format!("https://scholar.google.com/scholar?q={}", encoded_query),
            
            // General knowledge
            format!("https://www.britannica.com/search?query={}", encoded_query),
            
            // News/current events
            format!("https://news.google.com/search?q={}", encoded_query),
        ]
    }

    /// Extract knowledge from a crawled page
    fn extract_knowledge_from_page(&self, page: &CrawledPage) -> Vec<WebKnowledge> {
        // Split markdown into sentences for extraction
        let sentences: Vec<&str> = page.markdown
            .split(&['.', '!', '?'][..])
            .filter(|s| !s.trim().is_empty())
            .collect();

        // Extract knowledge from each sentence using extract_from_text
        let query_concepts: Vec<String> = page.title.split_whitespace()
            .map(|s| s.to_lowercase())
            .collect();
        
        sentences.iter()
            .filter_map(|sentence| {
                self.extractor.extract_from_text(sentence.trim(), &query_concepts, &page.url)
            })
            .collect()
    }

    /// Get acquisition statistics
    pub async fn stats(&self) -> AcquisitionStats {
        let cache = self.cache.read().await;
        let crawler_stats = self.crawler.stats();

        AcquisitionStats {
            queries_cached: cache.len(),
            pages_crawled: crawler_stats.pages_visited,
            total_knowledge_entries: cache.values().map(|v| v.len()).sum(),
        }
    }

    /// Clear cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }
}

/// Acquisition statistics
#[derive(Debug, Clone)]
pub struct AcquisitionStats {
    pub queries_cached: usize,
    pub pages_crawled: usize,
    pub total_knowledge_entries: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fast_knowledge_acquisition() {
        let config = FastKnowledgeConfig::default();
        let system = FastKnowledgeAcquisition::new(config).unwrap();

        let knowledge = system.learn_from_query("artificial intelligence").await;
        assert!(!knowledge.is_empty());

        let stats = system.stats().await;
        assert!(stats.queries_cached > 0);
    }

    #[tokio::test]
    async fn test_parallel_queries() {
        let config = FastKnowledgeConfig::default();
        let system = FastKnowledgeAcquisition::new(config).unwrap();

        let queries = vec![
            "machine learning".to_string(),
            "neural networks".to_string(),
            "deep learning".to_string(),
        ];

        let results = system.learn_from_queries(&queries).await;
        assert_eq!(results.len(), 3);
    }
}
