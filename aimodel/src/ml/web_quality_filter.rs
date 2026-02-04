//! Web Content Quality Filtering (Blu-WERP Inspired)
//!
//! Implements quality filtering inspired by the Blu-WERP paper:
//! - Gopher-style quality heuristics
//! - Repetition detection (structural redundancy)
//! - Stopword-based content scoring
//! - Semantic quality classification

use std::collections::{HashMap, HashSet};

/// Quality filter configuration
#[derive(Debug, Clone)]
pub struct QualityFilterConfig {
    /// Minimum document length in characters
    pub min_doc_length: usize,
    /// Maximum document length in characters
    pub max_doc_length: usize,
    /// Minimum words per sentence
    pub min_words_per_sentence: usize,
    /// Maximum ratio of non-alpha characters
    pub max_symbol_to_word_ratio: f32,
    /// Maximum line-level repetition ratio
    pub max_line_repetition: f32,
    /// Minimum stopword ratio (jusText-style)
    pub min_stopword_ratio: f32,
    /// Maximum ellipsis ratio (indicates low quality)
    pub max_ellipsis_ratio: f32,
}

impl Default for QualityFilterConfig {
    fn default() -> Self {
        Self {
            min_doc_length: 50,
            max_doc_length: 100_000,
            min_words_per_sentence: 3,
            max_symbol_to_word_ratio: 0.3,
            max_line_repetition: 0.3,
            min_stopword_ratio: 0.15,  // jusText-style: needs sufficient stopwords
            max_ellipsis_ratio: 0.1,
        }
    }
}

/// Gopher-style quality filter
#[derive(Debug, Clone)]
pub struct GopherQualityFilter {
    config: QualityFilterConfig,
    stopwords: HashSet<String>,
    // Common spam/boilerplate patterns
    spam_patterns: Vec<String>,
}

impl GopherQualityFilter {
    pub fn new(config: QualityFilterConfig) -> Self {
        let stopwords: HashSet<String> = [
            "the", "a", "an", "is", "are", "was", "were", "be", "been", "being",
            "have", "has", "had", "do", "does", "did", "will", "would", "could",
            "should", "may", "might", "must", "shall", "can", "need", "dare", "ought",
            "used", "to", "of", "in", "for", "on", "with", "at", "by", "from",
            "as", "into", "through", "during", "before", "after", "above", "below",
            "between", "under", "and", "but", "or", "yet", "so", "if", "because",
            "although", "though", "while", "where", "when", "that", "which", "who",
            "whom", "whose", "what", "this", "these", "those", "i", "me", "my",
            "myself", "we", "our", "ours", "ourselves", "you", "your", "yours",
            "yourself", "yourselves", "he", "him", "his", "himself", "she", "her",
            "hers", "herself", "it", "its", "itself", "they", "them", "their",
            "theirs", "themselves",
        ].iter().map(|s| s.to_string()).collect();

        let spam_patterns = vec![
            "cookie policy".to_string(),
            "privacy policy".to_string(),
            "terms of service".to_string(),
            "© 20".to_string(),
            "all rights reserved".to_string(),
            "click here to".to_string(),
            "subscribe now".to_string(),
            "sign up for".to_string(),
            "advertisement".to_string(),
            "powered by".to_string(),
            "loading...".to_string(),
        ];

        Self {
            config,
            stopwords,
            spam_patterns,
        }
    }

    /// Apply all quality filters to text
    pub fn filter(&self, text: &str) -> FilterResult {
        let mut result = FilterResult::default();
        
        // Document-level filters
        if !self.check_length(text) {
            result.failures.push(FilterFailure::TooShort);
        }
        
        if !self.check_symbol_ratio(text) {
            result.failures.push(FilterFailure::HighSymbolRatio);
        }
        
        if !self.check_boilerplate(text) {
            result.failures.push(FilterFailure::Boilerplate);
        }
        
        // Line-level repetition (GopherRepetition style)
        if !self.check_line_repetition(text) {
            result.failures.push(FilterFailure::Repetitive);
        }
        
        // Stopword ratio (jusText style quality signal)
        let stopword_ratio = self.compute_stopword_ratio(text);
        result.quality_score = stopword_ratio;
        
        if stopword_ratio < self.config.min_stopword_ratio {
            result.failures.push(FilterFailure::LowStopwordRatio);
        }
        
        // Check ellipsis ratio
        if !self.check_ellipsis_ratio(text) {
            result.failures.push(FilterFailure::HighEllipsisRatio);
        }
        
        result.passed = result.failures.is_empty();
        result
    }

    /// Check document length
    fn check_length(&self, text: &str) -> bool {
        let len = text.len();
        len >= self.config.min_doc_length && len <= self.config.max_doc_length
    }

    /// Check symbol-to-word ratio (Gopher-style)
    fn check_symbol_ratio(&self, text: &str) -> bool {
        let non_alpha_count = text.chars().filter(|c| !c.is_alphabetic() && !c.is_whitespace()).count();
        let word_count = text.split_whitespace().count();
        
        if word_count == 0 {
            return false;
        }
        
        let ratio = non_alpha_count as f32 / word_count as f32;
        ratio <= self.config.max_symbol_to_word_ratio
    }

    /// Check for common boilerplate/spam patterns
    fn check_boilerplate(&self, text: &str) -> bool {
        let text_lower = text.to_lowercase();
        let spam_count = self.spam_patterns
            .iter()
            .filter(|p| text_lower.contains(&p.to_lowercase()))
            .count();
        
        // Allow up to 2 spam patterns
        spam_count <= 2
    }

    /// Check line-level repetition (GopherRepetition)
    fn check_line_repetition(&self, text: &str) -> bool {
        let lines: Vec<&str> = text.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();
        if lines.len() < 3 {
            return true;
        }
        
        let mut duplicates = 0usize;
        let mut seen = HashSet::new();
        
        for line in &lines {
            let normalized = line.to_lowercase().replace(|c: char| !c.is_alphanumeric(), "");
            if normalized.len() > 10 && seen.contains(&normalized) {
                duplicates += 1;
            }
            seen.insert(normalized);
        }
        
        let ratio = duplicates as f32 / lines.len() as f32;
        ratio <= self.config.max_line_repetition
    }

    /// Compute stopword ratio (jusText-style quality signal)
    fn compute_stopword_ratio(&self, text: &str) -> f32 {
        let words: Vec<&str> = text
            .split(|c: char| !c.is_alphabetic())
            .filter(|w| !w.is_empty() && w.len() > 1)
            .collect();
        
        if words.is_empty() {
            return 0.0;
        }
        
        let stopword_count = words
            .iter()
            .filter(|w| self.stopwords.contains(&w.to_lowercase()))
            .count();
        
        stopword_count as f32 / words.len() as f32
    }

    /// Check ellipsis ratio (indicates truncated/placeholder content)
    fn check_ellipsis_ratio(&self, text: &str) -> bool {
        let ellipsis_count = text.matches("...").count() + text.matches("…").count();
        let words = text.split_whitespace().count();
        
        if words == 0 {
            return true;
        }
        
        let ratio = ellipsis_count as f32 / words as f32;
        ratio <= self.config.max_ellipsis_ratio
    }
}

/// Repetition filter (structural redundancy detection)
#[derive(Debug, Clone)]
pub struct RepetitionFilter {
    /// Minimum n-gram size for duplicate detection
    min_ngram_size: usize,
    /// Maximum allowed duplicate ratio
    max_duplicate_ratio: f32,
}

impl Default for RepetitionFilter {
    fn default() -> Self {
        Self {
            min_ngram_size: 13,  // Blu-WERP uses 13 tokens
            max_duplicate_ratio: 0.15,
        }
    }
}

impl RepetitionFilter {
    /// Check for repetitive content using paragraph-level deduplication
    pub fn check_repetition(&self, text: &str) -> bool {
        let paragraphs: Vec<&str> = text.split("\n\n")
            .map(|p| p.trim())
            .filter(|p| !p.is_empty() && p.len() > 20)
            .collect();
        
        if paragraphs.len() < 2 {
            return true;
        }
        
        // Check for paragraph-level duplicates
        let mut seen_hashes = HashSet::new();
        let mut duplicates = 0;
        
        for para in &paragraphs {
            let normalized = self.normalize_for_hash(para);
            let hash = self.simple_hash(&normalized);
            
            if seen_hashes.contains(&hash) {
                duplicates += 1;
            }
            seen_hashes.insert(hash);
        }
        
        let ratio = duplicates as f32 / paragraphs.len() as f32;
        ratio <= self.max_duplicate_ratio
    }

    /// Check for repeated sentence structures
    pub fn check_sentence_repetition(&self, text: &str) -> bool {
        let sentences: Vec<&str> = text
            .split(|c| c == '.' || c == '!' || c == '?')
            .map(|s| s.trim())
            .filter(|s| s.len() > 10)
            .collect();
        
        if sentences.len() < 3 {
            return true;
        }
        
        // Check for sentence starts (structural repetition)
        let mut start_counts: HashMap<String, usize> = HashMap::new();
        
        for sent in &sentences {
            let start: String = sent.split_whitespace()
                .take(3)
                .collect::<Vec<_>>()
                .join(" ")
                .to_lowercase();
            
            if start.len() > 5 {
                *start_counts.entry(start).or_insert(0) += 1;
            }
        }
        
        // Check if any start appears too frequently
        let max_repeats = (sentences.len() as f32 * 0.3) as usize;
        start_counts.values().all(|&count| count <= max_repeats)
    }

    /// Normalize text for hashing (deduplication)
    fn normalize_for_hash(&self, text: &str) -> String {
        text.to_lowercase()
            .replace(|c: char| !c.is_alphanumeric(), "")
            .replace(|c: char| c.is_whitespace(), "")
    }

    /// Simple hash function for deduplication
    fn simple_hash(&self, text: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        hasher.finish()
    }
}

/// Semantic quality scorer (FastText-inspired)
#[derive(Debug, Clone)]
pub struct SemanticQualityScorer {
    /// Educational/informative keywords (positive signals)
    positive_keywords: HashSet<String>,
    /// Spam/marketing keywords (negative signals)
    negative_keywords: HashSet<String>,
}

impl Default for SemanticQualityScorer {
    fn default() -> Self {
        let positive_keywords: HashSet<String> = [
            "definition", "explain", "example", "therefore", "however",
            "consequently", "specifically", "according to", "research",
            "study", "analysis", "evidence", "demonstrates", "indicates",
            "refers to", "consists of", "characterized by", "defined as",
            "theory", "concept", "principle", "framework", "methodology",
            "results", "conclusion", "significant", "importantly",
            "furthermore", "additionally", "moreover", "nevertheless",
        ].iter().map(|s| s.to_string()).collect();

        let negative_keywords: HashSet<String> = [
            "click here", "buy now", "limited time", "act now",
            "amazing", "incredible", "revolutionary", "breakthrough",
            "miracle", "guaranteed", "risk-free", "special offer",
            "order now", "call now", "don't miss", "hurry",
            "!!!", "click below", "subscribe", "sign up today",
        ].iter().map(|s| s.to_string()).collect();

        Self {
            positive_keywords,
            negative_keywords,
        }
    }
}

impl SemanticQualityScorer {
    /// Score content quality based on semantic signals
    pub fn score(&self, text: &str) -> f32 {
        let text_lower = text.to_lowercase();
        
        // Count positive signals
        let positive_count = self.positive_keywords
            .iter()
            .filter(|kw| text_lower.contains(*kw))
            .count();
        
        // Count negative signals
        let negative_count = self.negative_keywords
            .iter()
            .filter(|kw| text_lower.contains(*kw))
            .count();
        
        // Compute quality score
        let word_count = text.split_whitespace().count();
        if word_count == 0 {
            return 0.0;
        }
        
        let positive_score = (positive_count as f32 * 0.1).min(1.0);
        let negative_penalty = (negative_count as f32 * 0.2).min(0.5);
        
        (positive_score - negative_penalty + 0.5).clamp(0.0, 1.0)
    }

    /// Check if text is likely educational/informative
    pub fn is_educational(&self, text: &str) -> bool {
        self.score(text) > 0.5
    }
}

/// Filter result with detailed failure information
#[derive(Debug, Clone, Default)]
pub struct FilterResult {
    /// Whether the content passed all filters
    pub passed: bool,
    /// Quality score (stopword ratio or semantic score)
    pub quality_score: f32,
    /// List of filter failures
    pub failures: Vec<FilterFailure>,
}

/// Types of filter failures
#[derive(Debug, Clone)]
pub enum FilterFailure {
    TooShort,
    TooLong,
    HighSymbolRatio,
    Boilerplate,
    Repetitive,
    LowStopwordRatio,
    HighEllipsisRatio,
    NotEducational,
}

/// Combined web content filter (Blu-WERP pipeline)
#[derive(Debug, Clone)]
pub struct BluWerpFilter {
    gopher: GopherQualityFilter,
    repetition: RepetitionFilter,
    semantic: SemanticQualityScorer,
}

impl Default for BluWerpFilter {
    fn default() -> Self {
        Self {
            gopher: GopherQualityFilter::new(QualityFilterConfig::default()),
            repetition: RepetitionFilter::default(),
            semantic: SemanticQualityScorer::default(),
        }
    }
}

impl BluWerpFilter {
    /// Apply full Blu-WERP filtering pipeline
    /// 
    /// Pipeline stages:
    /// 1. Gopher quality filters (fast heuristics)
    /// 2. Repetition detection (structural redundancy)
    /// 3. Semantic quality scoring (educational value)
    pub fn filter(&self, text: &str) -> FilterResult {
        // Stage 1: Gopher quality filters
        let mut result = self.gopher.filter(text);
        
        if !result.passed {
            return result;
        }
        
        // Stage 2: Repetition detection
        if !self.repetition.check_repetition(text) {
            result.failures.push(FilterFailure::Repetitive);
        }
        
        if !self.repetition.check_sentence_repetition(text) {
            result.failures.push(FilterFailure::Repetitive);
        }
        
        // Stage 3: Semantic quality scoring
        let semantic_score = self.semantic.score(text);
        if semantic_score < 0.3 {
            result.failures.push(FilterFailure::NotEducational);
        }
        
        // Combine quality scores
        result.quality_score = (result.quality_score + semantic_score) / 2.0;
        result.passed = result.failures.is_empty();
        
        result
    }

    /// Quick filter for high-throughput scenarios
    pub fn filter_fast(&self, text: &str) -> bool {
        // Only apply fast Gopher filters
        let result = self.gopher.filter(text);
        result.passed
    }

    /// Score content quality without filtering
    pub fn score_quality(&self, text: &str) -> f32 {
        let gopher_score = self.gopher.compute_stopword_ratio(text);
        let semantic_score = self.semantic.score(text);
        
        // Weighted combination
        (gopher_score * 0.4 + semantic_score * 0.6).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gopher_length_filter() {
        let filter = GopherQualityFilter::new(QualityFilterConfig::default());
        
        assert!(!filter.check_length("short"));  // Too short
        assert!(filter.check_length("This is a proper length document with enough content.".repeat(5).as_str()));
    }

    #[test]
    fn test_stopword_ratio() {
        let filter = GopherQualityFilter::new(QualityFilterConfig::default());
        
        let high_quality = "The quick brown fox jumps over the lazy dog. This is a test of the emergency broadcast system.";
        let ratio = filter.compute_stopword_ratio(high_quality);
        assert!(ratio > 0.15, "Stopword ratio should be high for English text");
    }

    #[test]
    fn test_repetition_detection() {
        let filter = RepetitionFilter::default();
        
        let repetitive = "This is a paragraph.\n\nThis is a paragraph.\n\nThis is a paragraph.";
        assert!(!filter.check_repetition(repetitive));
        
        let unique = "First paragraph with unique content.\n\nSecond paragraph with different content.\n\nThird paragraph is also unique.";
        assert!(filter.check_repetition(unique));
    }

    #[test]
    fn test_semantic_scorer() {
        let scorer = SemanticQualityScorer::default();
        
        let educational = "According to recent research, this theory demonstrates significant results in analysis.";
        assert!(scorer.score(educational) > 0.5);
        
        let spam = "Click here now!!! Buy this amazing revolutionary breakthrough product!!!";
        assert!(scorer.score(spam) < 0.5);
    }
}
