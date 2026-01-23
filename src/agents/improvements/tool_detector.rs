//! Tool Usage Detector - Identifies when tools should be used
//!
//! Prevents the anti-pattern of EXPLAINING tools instead of USING them.


/// Detects required tools from user query
pub struct ToolDetector {
    /// Available tools
    #[allow(dead_code)]
    available_tools: Vec<ToolCapability>,
}

#[derive(Debug, Clone)]
pub enum ToolCapability {
    WebSearch,
    RAGIngest,
    DocumentAnalysis,
    WeatherAPI,
    CodeExecution,
    ImageGeneration,
}

#[derive(Debug, Clone)]
pub struct ToolRequirement {
    pub tool: ToolCapability,
    pub confidence: f64,
    pub reason: String,
    pub priority: Priority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Required,   // Must use tool or response is wrong
    Recommended, // Should use tool for best results
    Optional,    // Could enhance response
}

impl ToolDetector {
    pub fn new() -> Self {
        Self {
            available_tools: vec![
                ToolCapability::WebSearch,
                ToolCapability::RAGIngest,
                ToolCapability::DocumentAnalysis,
                ToolCapability::WeatherAPI,
                ToolCapability::CodeExecution,
            ],
        }
    }
    
    /// Analyze query and detect required tools
    pub fn detect_tools(&self, query: &str) -> Vec<ToolRequirement> {
        let mut requirements = Vec::new();
        let query_lower = query.to_lowercase();
        
        // Weather queries - REQUIRED
        if self.is_weather_query(&query_lower) {
            requirements.push(ToolRequirement {
                tool: ToolCapability::WeatherAPI,
                confidence: 0.95,
                reason: "User asking for current/realtime weather data".to_string(),
                priority: Priority::Required,
            });
        }
        
        // URL summarization - REQUIRED
        if self.contains_url(&query) && 
           (query_lower.contains("summarize") || query_lower.contains("what") || query_lower.contains("about")) {
            requirements.push(ToolRequirement {
                tool: ToolCapability::RAGIngest,
                confidence: 0.9,
                reason: "User requesting summary of specific URL".to_string(),
                priority: Priority::Required,
            });
        }
        
        // Web search - REQUIRED for current info
        if self.needs_current_info(&query_lower) {
            requirements.push(ToolRequirement {
                tool: ToolCapability::WebSearch,
                confidence: 0.85,
                reason: "Query requires current/recent information".to_string(),
                priority: Priority::Required,
            });
        }
        
        // Document analysis - REQUIRED
        if query_lower.contains("document") || query_lower.contains("pdf") || 
           query_lower.contains("file") {
            requirements.push(ToolRequirement {
                tool: ToolCapability::DocumentAnalysis,
                confidence: 0.8,
                reason: "User mentioning documents/files".to_string(),
                priority: Priority::Recommended,
            });
        }
        
        // Code execution - RECOMMENDED for demos
        if query_lower.contains("practical example") || query_lower.contains("demonstrate") ||
           query_lower.contains("show me") {
            requirements.push(ToolRequirement {
                tool: ToolCapability::CodeExecution,
                confidence: 0.7,
                reason: "User requesting demonstration".to_string(),
                priority: Priority::Recommended,
            });
        }
        
        // Sort by priority
        requirements.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        requirements
    }
    
    /// Check if query is about weather
    fn is_weather_query(&self, query: &str) -> bool {
        let weather_keywords = ["weather", "temperature", "forecast", "rain", "snow", "sunny"];
        let location_indicators = ["in", "at", "for"];
        
        let has_weather = weather_keywords.iter().any(|k| query.contains(k));
        let has_location = location_indicators.iter().any(|k| query.contains(k));
        
        has_weather && has_location
    }
    
    /// Check if query needs current/recent information
    fn needs_current_info(&self, query: &str) -> bool {
        let current_keywords = [
            "current", "latest", "recent", "today", "now", "right now",
            "this week", "this month", "breaking", "news"
        ];
        
        current_keywords.iter().any(|k| query.contains(k))
    }
    
    /// Check if query contains URL
    fn contains_url(&self, query: &str) -> bool {
        query.contains("http://") || query.contains("https://") || query.contains(".com") || query.contains(".org")
    }
    
    /// Generate error message if tools not used
    pub fn generate_warning(&self, requirements: &[ToolRequirement]) -> Option<String> {
        if requirements.is_empty() {
            return None;
        }
        
        let required: Vec<_> = requirements.iter()
            .filter(|r| r.priority == Priority::Required)
            .collect();
        
        if required.is_empty() {
            return None;
        }
        
        let tool_names: Vec<String> = required.iter()
            .map(|r| format!("{:?}", r.tool))
            .collect();
        
        Some(format!(
            "⚠️ WARNING: This query requires using {}. Do NOT just explain what they are - USE THEM!",
            tool_names.join(", ")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_weather_detection() {
        let detector = ToolDetector::new();
        
        let requirements = detector.detect_tools("What is the weather in Tucson, Arizona?");
        assert!(!requirements.is_empty());
        assert!(matches!(requirements[0].tool, ToolCapability::WeatherAPI));
        assert_eq!(requirements[0].priority, Priority::Required);
    }
    
    #[test]
    fn test_url_summarization() {
        let detector = ToolDetector::new();
        
        let requirements = detector.detect_tools("Can you summarize McKaleOlson.com");
        assert!(!requirements.is_empty());
        assert!(matches!(requirements[0].tool, ToolCapability::RAGIngest));
        assert_eq!(requirements[0].priority, Priority::Required);
    }
    
    #[test]
    fn test_current_info_detection() {
        let detector = ToolDetector::new();
        
        let requirements = detector.detect_tools("What are the latest AI developments?");
        assert!(!requirements.is_empty());
        assert!(matches!(requirements[0].tool, ToolCapability::WebSearch));
    }
    
    #[test]
    fn test_demonstration_request() {
        let detector = ToolDetector::new();
        
        let requirements = detector.detect_tools("Show me a practical example");
        assert!(!requirements.is_empty());
        assert!(matches!(requirements[0].tool, ToolCapability::CodeExecution));
    }
}
