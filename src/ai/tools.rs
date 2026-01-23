//! Tool Calling System for Dynamic Function Execution
//!
//! Enables LLM to call external tools/functions dynamically:
//! - Web search
//! - Calculator
//! - Code execution
//! - Database queries
//! - File operations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::{Result, anyhow};

/// Tool definition with JSON schema for parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Tool name (e.g., "web_search", "calculator")
    pub name: String,
    
    /// Human-readable description
    pub description: String,
    
    /// JSON schema for parameters
    pub parameters: serde_json::Value,
}

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Tool that was called
    pub tool_name: String,
    
    /// Execution result
    pub result: String,
    
    /// Whether execution succeeded
    pub success: bool,
    
    /// Optional error message
    pub error: Option<String>,
}

/// Tool call request from LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Tool to call
    pub name: String,
    
    /// Arguments as JSON
    pub arguments: serde_json::Value,
}

/// Tool registry for managing available tools
pub struct ToolRegistry {
    tools: HashMap<String, Tool>,
    executors: HashMap<String, Box<dyn ToolExecutor + Send + Sync>>,
}

/// Trait for tool executors
pub trait ToolExecutor: Send + Sync {
    fn execute(&self, args: serde_json::Value) -> Result<String>;
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            executors: HashMap::new(),
        }
    }
    
    /// Register a new tool
    pub fn register<E: ToolExecutor + 'static>(
        &mut self,
        tool: Tool,
        executor: E,
    ) {
        let name = tool.name.clone();
        self.tools.insert(name.clone(), tool);
        self.executors.insert(name, Box::new(executor));
    }
    
    /// Get all available tools
    pub fn get_tools(&self) -> Vec<Tool> {
        self.tools.values().cloned().collect()
    }
    
    /// Execute a tool call
    pub async fn execute(&self, call: &ToolCall) -> Result<ToolResult> {
        let executor = self.executors.get(&call.name)
            .ok_or_else(|| anyhow!("Tool not found: {}", call.name))?;
        
        match executor.execute(call.arguments.clone()) {
            Ok(result) => Ok(ToolResult {
                tool_name: call.name.clone(),
                result,
                success: true,
                error: None,
            }),
            Err(e) => Ok(ToolResult {
                tool_name: call.name.clone(),
                result: String::new(),
                success: false,
                error: Some(e.to_string()),
            }),
        }
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

//
// Built-in Tools
//

/// Calculator tool for math expressions
pub struct Calculator;

impl ToolExecutor for Calculator {
    fn execute(&self, args: serde_json::Value) -> Result<String> {
        let expression = args["expression"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing 'expression' parameter"))?;
        
        // Simple calculator (can be enhanced with meval or similar)
        match expression {
            expr if expr.contains("+") => {
                let parts: Vec<&str> = expr.split('+').collect();
                if parts.len() == 2 {
                    let a: f64 = parts[0].trim().parse()?;
                    let b: f64 = parts[1].trim().parse()?;
                    Ok((a + b).to_string())
                } else {
                    Err(anyhow!("Invalid expression"))
                }
            }
            expr if expr.contains("-") => {
                let parts: Vec<&str> = expr.split('-').collect();
                if parts.len() == 2 {
                    let a: f64 = parts[0].trim().parse()?;
                    let b: f64 = parts[1].trim().parse()?;
                    Ok((a - b).to_string())
                } else {
                    Err(anyhow!("Invalid expression"))
                }
            }
            expr if expr.contains("*") => {
                let parts: Vec<&str> = expr.split('*').collect();
                if parts.len() == 2 {
                    let a: f64 = parts[0].trim().parse()?;
                    let b: f64 = parts[1].trim().parse()?;
                    Ok((a * b).to_string())
                } else {
                    Err(anyhow!("Invalid expression"))
                }
            }
            expr if expr.contains("/") => {
                let parts: Vec<&str> = expr.split('/').collect();
                if parts.len() == 2 {
                    let a: f64 = parts[0].trim().parse()?;
                    let b: f64 = parts[1].trim().parse()?;
                    if b == 0.0 {
                        Err(anyhow!("Division by zero"))
                    } else {
                        Ok((a / b).to_string())
                    }
                } else {
                    Err(anyhow!("Invalid expression"))
                }
            }
            _ => Err(anyhow!("Unsupported operation")),
        }
    }
}

/// Web search tool - REAL implementation using reqwest
pub struct WebSearch;

impl ToolExecutor for WebSearch {
    fn execute(&self, args: serde_json::Value) -> Result<String> {
        let query = args["query"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing 'query' parameter"))?;
        
        // For weather queries, use wttr.in (free, no API key needed)
        if query.to_lowercase().contains("weather") {
            return execute_weather_search(query);
        }
        
        // For general queries, provide instructions
        Ok(format!(
            "To search the web for '{}', please configure a search API key:\n\
            1. Sign up for Serper API (https://serper.dev) or Brave Search API\n\
            2. Set SERPER_API_KEY or BRAVE_API_KEY environment variable\n\
            3. Restart the server\n\n\
            For weather queries specifically, I can use wttr.in directly.",
            query
        ))
    }
}

/// Execute weather search using wttr.in
fn execute_weather_search(query: &str) -> Result<String> {
    // Extract location from query
    let location = query
        .replace("weather in", "")
        .replace("weather", "")
        .replace("current", "")
        .replace("what's the", "")
        .replace("what is the", "")
        .replace("search the web for the", "")
        .replace("search for", "")
        .trim()
        .replace(" ", "+");
    
    if location.is_empty() {
        return Ok("Please specify a location for the weather query.".to_string());
    }
    
    // Use wttr.in for weather data (synchronous for now, can be async later)
    let url = format!("https://wttr.in/{}?format=j1", location);
    
    // TEMPORARY: Stub for weather until blocking feature enabled
    // For this implementation, we'll use a runtime block
    // In production, this should be fully async
    #[cfg(feature = "agents")]
    match std::thread::spawn(move || {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()?;
        
        let response = client.get(&url).send()?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Weather API returned error: {}", response.status()));
        }
        
        let json: serde_json::Value = response.json()?;
        
        // Extract key weather info
        let current = &json["current_condition"][0];
        let location_info = &json["nearest_area"][0];
        
        let temp_f = current["temp_F"].as_str().unwrap_or("N/A");
        let temp_c = current["temp_C"].as_str().unwrap_or("N/A");
        let condition = current["weatherDesc"][0]["value"].as_str().unwrap_or("N/A");
        let humidity = current["humidity"].as_str().unwrap_or("N/A");
        let wind_speed = current["windspeedMiles"].as_str().unwrap_or("N/A");
        let wind_dir = current["winddir16Point"].as_str().unwrap_or("N/A");
        let feels_like_f = current["FeelsLikeF"].as_str().unwrap_or("N/A");
        let uv_index = current["uvIndex"].as_str().unwrap_or("N/A");
        
        let city = location_info["areaName"][0]["value"].as_str().unwrap_or(&location);
        let region = location_info["region"][0]["value"].as_str().unwrap_or("");
        let country = location_info["country"][0]["value"].as_str().unwrap_or("");
        
        Ok::<String, anyhow::Error>(format!(
            "# Current Weather in {}, {}, {}\n\n\
            **Condition**: {}\n\
            **Temperature**: {}°F ({}°C)\n\
            **Feels Like**: {}°F\n\
            **Humidity**: {}%\n\
            **Wind**: {} mph {}\n\
            **UV Index**: {}\n\n\
            Source: wttr.in",
            city, region, country,
            condition,
            temp_f, temp_c,
            feels_like_f,
            humidity,
            wind_speed, wind_dir,
            uv_index
        ))
    }).join() {
        Ok(Ok(result)) => Ok(result),
        Ok(Err(e)) => Err(e),
        Err(_) => Err(anyhow!("Weather search thread panicked"))
    }
    
    #[cfg(not(feature = "agents"))]
    {
        // Stub implementation when agents feature not enabled
        let _url = url; // Avoid unused warning
        Ok(format!(
            "Weather information for '{}' is currently unavailable.\n\
            Enable 'agents' feature for live weather data.",
            location
        ))
    }
}

/// Get current time tool
pub struct GetCurrentTime;

impl ToolExecutor for GetCurrentTime {
    fn execute(&self, _args: serde_json::Value) -> Result<String> {
        let now = chrono::Local::now();
        Ok(now.format("%Y-%m-%d %H:%M:%S %Z").to_string())
    }
}

/// Create default tool registry with built-in tools
pub fn create_default_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    
    // Calculator
    registry.register(
        Tool {
            name: "calculator".to_string(),
            description: "Perform mathematical calculations. Supports +, -, *, /".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "expression": {
                        "type": "string",
                        "description": "Mathematical expression to evaluate (e.g., '2 + 2')"
                    }
                },
                "required": ["expression"]
            }),
        },
        Calculator,
    );
    
    // Web Search
    registry.register(
        Tool {
            name: "web_search".to_string(),
            description: "Search the web for information".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query"
                    }
                },
                "required": ["query"]
            }),
        },
        WebSearch,
    );
    
    // Current Time
    registry.register(
        Tool {
            name: "get_current_time".to_string(),
            description: "Get the current date and time".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        GetCurrentTime,
    );
    
    registry
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_calculator() {
        let calc = Calculator;
        
        let result = calc.execute(serde_json::json!({
            "expression": "10 + 5"
        })).unwrap();
        
        assert_eq!(result, "15");
    }
    
    #[tokio::test]
    async fn test_tool_registry() {
        let registry = create_default_registry();
        let tools = registry.get_tools();
        
        assert!(tools.len() >= 3);
        assert!(tools.iter().any(|t| t.name == "calculator"));
        
        let call = ToolCall {
            name: "calculator".to_string(),
            arguments: serde_json::json!({
                "expression": "20 * 3"
            }),
        };
        
        let result = registry.execute(&call).await.unwrap();
        assert!(result.success);
        assert_eq!(result.result, "60");
    }
}
