//! Object Context Utilities
//!
//! Helper functions for creating ObjectContext from queries and text inputs.

use chrono::Utc;
use crate::data::models::ObjectContext;
use crate::data::attributes::{Attributes, AttributeValue};

/// Create ObjectContext from a query string and subject
pub fn create_object_context(query: &str, subject: &str, attributes: Attributes) -> ObjectContext {
    let keywords = extract_keywords(query);
    let semantic_matches = count_semantic_matches(query, subject);
    
    ObjectContext {
        query: query.to_string(),
        subject: subject.to_string(),
        attributes,
        keywords,
        semantic_matches,
        timestamp: Utc::now(),
    }
}

/// Extract meaningful keywords from query
fn extract_keywords(query: &str) -> Vec<String> {
    // Common stop words to filter out
    let stop_words = [
        "the", "is", "at", "which", "on", "a", "an", "and", "or", "but",
        "in", "with", "to", "for", "of", "as", "by", "from", "be", "have",
        "it", "that", "this", "was", "are", "were", "been", "being",
        "what", "when", "where", "who", "why", "how",
    ];
    
    query
        .to_lowercase()
        .split_whitespace()
        .filter(|word| {
            // Keep words that are:
            // 1. Not stop words
            // 2. Longer than 2 characters
            // 3. Alphanumeric
            !stop_words.contains(&word.trim_matches(|c: char| !c.is_alphanumeric()))
                && word.len() > 2
        })
        .map(|word| word.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
        .filter(|word| !word.is_empty())
        .collect()
}

/// Count semantic matches between query and subject
fn count_semantic_matches(query: &str, subject: &str) -> u32 {
    let query_lower = query.to_lowercase();
    let subject_lower = subject.to_lowercase();
    
    let mut matches = 0u32;
    
    // Direct subject mention
    if query_lower.contains(&subject_lower) {
        matches += 3;
    }
    
    // Sacred keywords (fundamental concepts)
    let sacred_keywords = [
        "fundamental", "essence", "nature", "ultimate", "absolute",
        "divine", "sacred", "pure", "eternal", "infinite",
    ];
    for keyword in &sacred_keywords {
        if query_lower.contains(keyword) {
            matches += 2;
        }
    }
    
    // Subject-specific strong indicators
    let subject_indicators = match subject_lower.as_str() {
        "consciousness" => vec!["awareness", "mind", "thought", "sentience", "cognition"],
        "mathematics" => vec!["number", "equation", "formula", "proof", "theorem"],
        "physics" => vec!["energy", "matter", "force", "particle", "quantum"],
        "philosophy" => vec!["meaning", "truth", "reality", "existence", "knowledge"],
        "geometry" => vec!["shape", "pattern", "symmetry", "dimension", "space"],
        "biology" => vec!["life", "organism", "cell", "evolution", "genetic"],
        "chemistry" => vec!["molecule", "atom", "reaction", "element", "compound"],
        "psychology" => vec!["behavior", "emotion", "mental", "cognitive", "personality"],
        "art" => vec!["creative", "beauty", "aesthetic", "expression", "design"],
        "music" => vec!["sound", "rhythm", "harmony", "melody", "tone"],
        "language" => vec!["word", "meaning", "syntax", "semantic", "communication"],
        _ => vec![],
    };
    
    for indicator in &subject_indicators {
        if query_lower.contains(indicator) {
            matches += 1;
        }
    }
    
    matches.min(10) // Cap at 10
}

/// Estimate attributes from query content
pub fn estimate_attributes_from_query(query: &str) -> Attributes {
    let query_lower = query.to_lowercase();
    
    let mut ethos = 0.0f32;
    let mut logos = 0.0f32;
    let mut pathos = 0.0f32;
    
    // Ethos indicators (character, values, ethics)
    let ethos_words = [
        "should", "ought", "must", "right", "wrong", "good", "bad",
        "moral", "ethical", "virtue", "duty", "responsibility",
        "honest", "just", "fair", "integrity",
    ];
    for word in &ethos_words {
        if query_lower.contains(word) {
            ethos += 1.0;
        }
    }
    
    // Logos indicators (logic, reason, analysis)
    let logos_words = [
        "why", "how", "because", "therefore", "thus", "prove", "explain",
        "reason", "logic", "analyze", "think", "understand", "calculate",
        "define", "determine", "conclude", "deduce", "infer",
    ];
    for word in &logos_words {
        if query_lower.contains(word) {
            logos += 1.0;
        }
    }
    
    // Pathos indicators (emotion, feeling, experience)
    let pathos_words = [
        "feel", "emotion", "love", "hate", "fear", "joy", "sad",
        "pain", "pleasure", "desire", "hope", "wish", "dream",
        "beautiful", "terrible", "amazing", "awful", "wonderful",
    ];
    for word in &pathos_words {
        if query_lower.contains(word) {
            pathos += 1.0;
        }
    }
    
    // Normalize to -13 to +13 range (sacred scale)
    let total = (ethos + logos + pathos).max(1.0);
    let scale = 13.0;
    
    let ethos_val = (ethos / total * scale * 2.0) - scale;
    let logos_val = (logos / total * scale * 2.0) - scale;
    let pathos_val = (pathos / total * scale * 2.0) - scale;
    
    Attributes::with_elp(ethos_val, logos_val, pathos_val)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_keywords() {
        let query = "What is the fundamental nature of consciousness?";
        let keywords = extract_keywords(query);
        
        assert!(keywords.contains(&"fundamental".to_string()));
        assert!(keywords.contains(&"nature".to_string()));
        assert!(keywords.contains(&"consciousness".to_string()));
        assert!(!keywords.contains(&"the".to_string()));
        assert!(!keywords.contains(&"is".to_string()));
    }
    
    #[test]
    fn test_semantic_matches() {
        let matches = count_semantic_matches(
            "What is consciousness and awareness?",
            "consciousness"
        );
        
        assert!(matches >= 4); // Subject mention + awareness indicator
    }
    
    #[test]
    fn test_estimate_attributes() {
        let logical_query = "Why does this happen? How can we prove it?";
        let attrs = estimate_attributes_from_query(logical_query);
        
        // Should have high logos
        assert!(attrs.logos() > attrs.ethos());
        assert!(attrs.logos() > attrs.pathos());
    }
}
