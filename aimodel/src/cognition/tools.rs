//! Tool Use System for External Learning
//!
//! Enables the VortexRunner to learn from external sources:
//! - Web search
//! - API calls
//! - File reading
//! - Database queries
//!
//! Each tool result flows through the vortex for integration.

use crate::data::models::BeamTensor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ToolParameter>,
    pub tool_type: ToolType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    pub name: String,
    pub param_type: ParamType,
    pub required: bool,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParamType {
    String,
    Number,
    Boolean,
    Array,
    Object,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolType {
    WebSearch,
    HttpRequest,
    FileRead,
    FileWrite,
    Database,
    Shell,
    Custom(String),
}

/// Result from tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_name: String,
    pub success: bool,
    pub content: String,
    pub beams: Vec<BeamTensor>,
    pub metadata: HashMap<String, String>,
    pub timestamp: i64,
}

impl ToolResult {
    pub fn success(tool_name: &str, content: String) -> Self {
        Self {
            tool_name: tool_name.to_string(),
            success: true,
            content,
            beams: Vec::new(),
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    pub fn error(tool_name: &str, error: String) -> Self {
        Self {
            tool_name: tool_name.to_string(),
            success: false,
            content: error,
            beams: Vec::new(),
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    pub fn with_beams(mut self, beams: Vec<BeamTensor>) -> Self {
        self.beams = beams;
        self
    }

    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

/// Tool call request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool_name: String,
    pub arguments: HashMap<String, serde_json::Value>,
}

/// Tool registry - manages available tools
pub struct ToolRegistry {
    tools: HashMap<String, Tool>,
    /// HTTP client for web requests
    #[cfg(feature = "reqwest")]
    http_client: Option<reqwest::Client>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
            #[cfg(feature = "reqwest")]
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .ok(),
        };
        registry.register_default_tools();
        registry
    }

    /// Register default tools
    fn register_default_tools(&mut self) {
        // Web search tool
        self.register(Tool {
            name: "web_search".to_string(),
            description: "Search the web for information".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "query".to_string(),
                    param_type: ParamType::String,
                    required: true,
                    description: "Search query".to_string(),
                },
            ],
            tool_type: ToolType::WebSearch,
        });

        // HTTP GET tool
        self.register(Tool {
            name: "http_get".to_string(),
            description: "Make an HTTP GET request".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "url".to_string(),
                    param_type: ParamType::String,
                    required: true,
                    description: "URL to fetch".to_string(),
                },
            ],
            tool_type: ToolType::HttpRequest,
        });

        // File read tool
        self.register(Tool {
            name: "read_file".to_string(),
            description: "Read contents of a file".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "path".to_string(),
                    param_type: ParamType::String,
                    required: true,
                    description: "File path".to_string(),
                },
            ],
            tool_type: ToolType::FileRead,
        });

        // Calculate tool
        self.register(Tool {
            name: "calculate".to_string(),
            description: "Perform mathematical calculation".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "expression".to_string(),
                    param_type: ParamType::String,
                    required: true,
                    description: "Math expression to evaluate".to_string(),
                },
            ],
            tool_type: ToolType::Custom("math".to_string()),
        });
    }

    /// Register a tool
    pub fn register(&mut self, tool: Tool) {
        self.tools.insert(tool.name.clone(), tool);
    }

    /// Get a tool by name
    pub fn get(&self, name: &str) -> Option<&Tool> {
        self.tools.get(name)
    }

    /// List all tools
    pub fn list(&self) -> Vec<&Tool> {
        self.tools.values().collect()
    }

    /// Execute a tool call
    pub async fn execute(&self, call: &ToolCall) -> ToolResult {
        let Some(tool) = self.get(&call.tool_name) else {
            return ToolResult::error(&call.tool_name, format!("Tool '{}' not found", call.tool_name));
        };

        match &tool.tool_type {
            ToolType::WebSearch => self.execute_web_search(call).await,
            ToolType::HttpRequest => self.execute_http_request(call).await,
            ToolType::FileRead => self.execute_file_read(call).await,
            ToolType::Custom(name) if name == "math" => self.execute_calculate(call),
            _ => ToolResult::error(&call.tool_name, "Tool type not implemented".to_string()),
        }
    }

    /// Execute web search
    async fn execute_web_search(&self, call: &ToolCall) -> ToolResult {
        let query = call.arguments.get("query")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if query.is_empty() {
            return ToolResult::error(&call.tool_name, "Query is required".to_string());
        }

        // In production, would use a search API (DuckDuckGo, Brave, etc.)
        // For now, return a placeholder that indicates the search was requested
        ToolResult::success(&call.tool_name, format!("Search results for: {}", query))
            .with_metadata("query", query)
            .with_metadata("source", "web_search")
    }

    /// Execute HTTP request
    async fn execute_http_request(&self, call: &ToolCall) -> ToolResult {
        let url = call.arguments.get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if url.is_empty() {
            return ToolResult::error(&call.tool_name, "URL is required".to_string());
        }

        #[cfg(feature = "reqwest")]
        if let Some(client) = &self.http_client {
            match client.get(url).send().await {
                Ok(response) => {
                    match response.text().await {
                        Ok(text) => {
                            // Truncate if too long
                            let content = if text.len() > 10000 {
                                format!("{}...[truncated]", &text[..10000])
                            } else {
                                text
                            };
                            return ToolResult::success(&call.tool_name, content)
                                .with_metadata("url", url)
                                .with_metadata("source", "http");
                        }
                        Err(e) => return ToolResult::error(&call.tool_name, e.to_string()),
                    }
                }
                Err(e) => return ToolResult::error(&call.tool_name, e.to_string()),
            }
        }

        // Fallback when reqwest not available
        ToolResult::error(&call.tool_name, "HTTP client not available".to_string())
    }

    /// Execute file read
    async fn execute_file_read(&self, call: &ToolCall) -> ToolResult {
        let path = call.arguments.get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if path.is_empty() {
            return ToolResult::error(&call.tool_name, "Path is required".to_string());
        }

        match std::fs::read_to_string(path) {
            Ok(content) => {
                // Truncate if too long
                let content = if content.len() > 50000 {
                    format!("{}...[truncated]", &content[..50000])
                } else {
                    content
                };
                ToolResult::success(&call.tool_name, content)
                    .with_metadata("path", path)
                    .with_metadata("source", "file")
            }
            Err(e) => ToolResult::error(&call.tool_name, e.to_string()),
        }
    }

    /// Execute calculation
    fn execute_calculate(&self, call: &ToolCall) -> ToolResult {
        let expr = call.arguments.get("expression")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if expr.is_empty() {
            return ToolResult::error(&call.tool_name, "Expression is required".to_string());
        }

        // Simple expression evaluator (basic arithmetic)
        match self.eval_simple_expr(expr) {
            Ok(result) => ToolResult::success(&call.tool_name, result.to_string())
                .with_metadata("expression", expr)
                .with_metadata("source", "calculate"),
            Err(e) => ToolResult::error(&call.tool_name, e),
        }
    }

    /// Simple expression evaluator
    fn eval_simple_expr(&self, expr: &str) -> Result<f64, String> {
        // Very basic: just handle simple operations
        let expr = expr.trim();
        
        // Try to parse as number first
        if let Ok(n) = expr.parse::<f64>() {
            return Ok(n);
        }

        // Handle basic operations
        let ops: [(char, fn(f64, f64) -> f64); 4] = [
            ('+', |a, b| a + b),
            ('-', |a, b| a - b),
            ('*', |a, b| a * b),
            ('/', |a, b| if b != 0.0 { a / b } else { f64::NAN }),
        ];
        
        for (op, func) in ops {
            if let Some(pos) = expr.rfind(op) {
                if pos > 0 {
                    let left = expr[..pos].trim();
                    let right = expr[pos+1..].trim();
                    if let (Ok(a), Ok(b)) = (left.parse::<f64>(), right.parse::<f64>()) {
                        return Ok(func(a, b));
                    }
                }
            }
        }

        Err(format!("Cannot evaluate: {}", expr))
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_registry() {
        let registry = ToolRegistry::new();
        assert!(registry.get("web_search").is_some());
        assert!(registry.get("http_get").is_some());
        assert!(registry.get("read_file").is_some());
        assert!(registry.get("calculate").is_some());
    }

    #[test]
    fn test_calculate() {
        let registry = ToolRegistry::new();
        
        let call = ToolCall {
            tool_name: "calculate".to_string(),
            arguments: [("expression".to_string(), serde_json::json!("2 + 3"))].into_iter().collect(),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(registry.execute(&call));
        
        assert!(result.success);
        assert_eq!(result.content, "5");
    }

    #[test]
    fn test_simple_eval() {
        let registry = ToolRegistry::new();
        
        assert_eq!(registry.eval_simple_expr("5").unwrap(), 5.0);
        assert_eq!(registry.eval_simple_expr("2 + 3").unwrap(), 5.0);
        assert_eq!(registry.eval_simple_expr("10 - 4").unwrap(), 6.0);
        assert_eq!(registry.eval_simple_expr("3 * 4").unwrap(), 12.0);
        assert_eq!(registry.eval_simple_expr("15 / 3").unwrap(), 5.0);
    }
}
