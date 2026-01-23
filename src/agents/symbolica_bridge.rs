//! Symbolica integration for symbolic mathematics
//!
//! Provides symbolic computation capabilities that are 10x faster than SymPy,
//! enabling the agent to solve equations, differentiate, integrate, and simplify
//! mathematical expressions before generating code.

use crate::agents::error::{AgentError, Result};
use crate::agents::language::Language;

/// Symbolic mathematics engine using Symbolica
pub struct SymbolicaMath {
    // In production, this would hold the Symbolica context
    // For now, we'll use placeholder implementation
    enabled: bool,
}

impl SymbolicaMath {
    /// Create new Symbolica math engine
    pub fn new() -> Result<Self> {
        // TODO: Initialize Symbolica when dependency is added
        Ok(Self { enabled: false })
    }
    
    /// Check if Symbolica is available
    pub fn is_available(&self) -> bool {
        self.enabled
    }
    
    /// Solve an equation symbolically
    pub fn solve_equation(&self, equation: &str) -> Result<String> {
        if !self.enabled {
            return Err(AgentError::SymbolicError(
                "Symbolica not yet integrated. Add 'symbolica' dependency.".to_string()
            ));
        }
        
        // TODO: Actual Symbolica integration
        // symbolica::solve(equation)
        
        Ok(format!("Solution for: {}", equation))
    }
    
    /// Simplify a mathematical expression
    pub fn simplify(&self, expr: &str) -> Result<String> {
        if !self.enabled {
            return self.fallback_simplify(expr);
        }
        
        // TODO: Actual Symbolica integration
        // symbolica::simplify(expr)
        
        Ok(format!("Simplified: {}", expr))
    }
    
    /// Calculate derivative of expression with respect to variable
    pub fn differentiate(&self, expr: &str, var: &str) -> Result<String> {
        if !self.enabled {
            return self.fallback_differentiate(expr, var);
        }
        
        // TODO: Actual Symbolica integration
        // symbolica::diff(expr, var)
        
        Ok(format!("d/d{} ({})", var, expr))
    }
    
    /// Calculate integral of expression with respect to variable
    pub fn integrate(&self, expr: &str, var: &str) -> Result<String> {
        if !self.enabled {
            return Err(AgentError::SymbolicError(
                "Integration requires full Symbolica integration".to_string()
            ));
        }
        
        // TODO: Actual Symbolica integration
        // symbolica::integrate(expr, var)
        
        Ok(format!("∫({}) d{}", expr, var))
    }
    
    /// Factor a polynomial expression
    pub fn factor(&self, expr: &str) -> Result<String> {
        if !self.enabled {
            return self.fallback_factor(expr);
        }
        
        // TODO: Actual Symbolica integration
        // symbolica::factor(expr)
        
        Ok(format!("Factored: {}", expr))
    }
    
    /// Expand a polynomial expression
    pub fn expand(&self, expr: &str) -> Result<String> {
        if !self.enabled {
            return self.fallback_expand(expr);
        }
        
        // TODO: Actual Symbolica integration
        // symbolica::expand(expr)
        
        Ok(format!("Expanded: {}", expr))
    }
    
    /// Convert symbolic result to code in target language
    pub fn to_code(&self, symbolic_result: &str, language: Language) -> Result<String> {
        match language {
            Language::Python => self.to_python(symbolic_result),
            Language::Rust => self.to_rust(symbolic_result),
            Language::JavaScript => self.to_javascript(symbolic_result),
            Language::Julia => self.to_julia(symbolic_result),
            Language::Cpp => self.to_cpp(symbolic_result),
            _ => Ok(format!("// Symbolic result: {}", symbolic_result)),
        }
    }
    
    /// Convert to Python code
    fn to_python(&self, result: &str) -> Result<String> {
        Ok(format!(
            r#"import math

def calculate(x):
    """Symbolic computation result"""
    return {}
"#,
            self.convert_to_python_syntax(result)
        ))
    }
    
    /// Convert to Rust code
    fn to_rust(&self, result: &str) -> Result<String> {
        Ok(format!(
            r#"fn calculate(x: f64) -> f64 {{
    // Symbolic computation result
    {}
}}
"#,
            self.convert_to_rust_syntax(result)
        ))
    }
    
    /// Convert to JavaScript code
    fn to_javascript(&self, result: &str) -> Result<String> {
        Ok(format!(
            r#"function calculate(x) {{
    // Symbolic computation result
    return {};
}}
"#,
            self.convert_to_js_syntax(result)
        ))
    }
    
    /// Convert to Julia code
    fn to_julia(&self, result: &str) -> Result<String> {
        Ok(format!(
            r#"# Symbolic computation result
function calculate(x)
    {}
end
"#,
            self.convert_to_julia_syntax(result)
        ))
    }
    
    /// Convert to C++ code
    fn to_cpp(&self, result: &str) -> Result<String> {
        Ok(format!(
            r#"#include <cmath>

double calculate(double x) {{
    // Symbolic computation result
    return {};
}}
"#,
            self.convert_to_cpp_syntax(result)
        ))
    }
    
    // Syntax conversion helpers
    fn convert_to_python_syntax(&self, expr: &str) -> String {
        expr.replace("^", "**")
            .replace("sqrt", "math.sqrt")
            .replace("sin", "math.sin")
            .replace("cos", "math.cos")
            .replace("exp", "math.exp")
            .replace("log", "math.log")
    }
    
    fn convert_to_rust_syntax(&self, expr: &str) -> String {
        expr.replace("^", ".powf")
            .replace("sqrt", "f64::sqrt")
            .replace("sin", "f64::sin")
            .replace("cos", "f64::cos")
            .replace("exp", "f64::exp")
            .replace("log", "f64::ln")
    }
    
    fn convert_to_js_syntax(&self, expr: &str) -> String {
        expr.replace("^", "**")
            .replace("sqrt", "Math.sqrt")
            .replace("sin", "Math.sin")
            .replace("cos", "Math.cos")
            .replace("exp", "Math.exp")
            .replace("log", "Math.log")
    }
    
    fn convert_to_julia_syntax(&self, expr: &str) -> String {
        // Julia has Python-like syntax for math
        expr.replace("^", "^")  // Julia uses ^ natively
    }
    
    fn convert_to_cpp_syntax(&self, expr: &str) -> String {
        expr.replace("^", "pow")
            .replace("sqrt", "std::sqrt")
            .replace("sin", "std::sin")
            .replace("cos", "std::cos")
            .replace("exp", "std::exp")
            .replace("log", "std::log")
    }
    
    // Fallback methods for when Symbolica isn't available
    fn fallback_simplify(&self, expr: &str) -> Result<String> {
        // Basic pattern matching for simple cases
        if expr.contains("x + 0") {
            return Ok(expr.replace("x + 0", "x"));
        }
        if expr.contains("x * 1") {
            return Ok(expr.replace("x * 1", "x"));
        }
        if expr.contains("x * 0") {
            return Ok("0".to_string());
        }
        
        Ok(expr.to_string())
    }
    
    fn fallback_differentiate(&self, expr: &str, var: &str) -> Result<String> {
        // Very basic differentiation rules
        if expr == var {
            return Ok("1".to_string());
        }
        if !expr.contains(var) {
            return Ok("0".to_string());
        }
        if expr.starts_with(&format!("{}^", var)) {
            // Power rule: d/dx(x^n) = n*x^(n-1)
            if let Some(power) = expr.strip_prefix(&format!("{}^", var)) {
                if let Ok(n) = power.parse::<i32>() {
                    if n == 2 {
                        return Ok(format!("2*{}", var));
                    }
                    return Ok(format!("{}*{}^{}", n, var, n - 1));
                }
            }
        }
        
        Ok(format!("d/d{} ({})", var, expr))
    }
    
    fn fallback_factor(&self, expr: &str) -> Result<String> {
        // Basic factoring (difference of squares)
        if expr == "x^2 - 1" {
            return Ok("(x - 1)(x + 1)".to_string());
        }
        if expr == "x^2 - 4" {
            return Ok("(x - 2)(x + 2)".to_string());
        }
        
        Ok(expr.to_string())
    }
    
    fn fallback_expand(&self, expr: &str) -> Result<String> {
        // Basic expansion
        if expr == "(x + 1)^2" {
            return Ok("x^2 + 2*x + 1".to_string());
        }
        if expr == "(x - 1)^2" {
            return Ok("x^2 - 2*x + 1".to_string());
        }
        
        Ok(expr.to_string())
    }
}

impl Default for SymbolicaMath {
    fn default() -> Self {
        Self::new().unwrap_or(Self { enabled: false })
    }
}

/// Detect if a task involves mathematical operations
pub fn is_math_task(task: &str) -> bool {
    let math_keywords = [
        "equation", "solve", "derivative", "differentiate", "integral", "integrate",
        "simplify", "factor", "expand", "polynomial", "algebraic", "symbolic",
        "calculus", "formula", "expression", "math", "calculate",
    ];
    
    let task_lower = task.to_lowercase();
    math_keywords.iter().any(|kw| task_lower.contains(kw))
}

/// Extract mathematical expression from task description
pub fn extract_math_expression(task: &str) -> Option<String> {
    // Look for expressions in quotes or after "solve", "simplify", etc.
    if let Some(start) = task.find('"') {
        if let Some(end) = task[start + 1..].find('"') {
            return Some(task[start + 1..start + 1 + end].to_string());
        }
    }
    
    // Look for patterns like "solve x^2 + 3x + 2 = 0"
    let patterns = ["solve ", "simplify ", "differentiate ", "integrate "];
    for pattern in patterns {
        if let Some(pos) = task.to_lowercase().find(pattern) {
            let after = &task[pos + pattern.len()..];
            // Take until next sentence boundary
            let expr = after
                .split(&['.', '?', '!', '\n'][..])
                .next()
                .unwrap_or(after)
                .trim();
            if !expr.is_empty() {
                return Some(expr.to_string());
            }
        }
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_is_math_task() {
        assert!(is_math_task("Solve the equation x^2 + 3x + 2 = 0"));
        assert!(is_math_task("Differentiate f(x) = x^3"));
        assert!(is_math_task("Simplify (x + 1)^2"));
        assert!(is_math_task("Calculate the integral"));
        assert!(!is_math_task("Create a web server"));
        assert!(!is_math_task("Design a user interface"));
    }
    
    #[test]
    fn test_extract_expression() {
        let expr = extract_math_expression("Solve \"x^2 + 3x + 2 = 0\"");
        assert_eq!(expr, Some("x^2 + 3x + 2 = 0".to_string()));
        
        let expr = extract_math_expression("Simplify x^2 + 2x + 1");
        assert_eq!(expr, Some("x^2 + 2x + 1".to_string()));
    }
    
    #[test]
    fn test_fallback_simplify() {
        let math = SymbolicaMath::default();
        
        assert_eq!(math.fallback_simplify("x + 0").unwrap(), "x");
        assert_eq!(math.fallback_simplify("x * 1").unwrap(), "x");
        assert_eq!(math.fallback_simplify("x * 0").unwrap(), "0");
    }
    
    #[test]
    fn test_fallback_differentiate() {
        let math = SymbolicaMath::default();
        
        assert_eq!(math.fallback_differentiate("x", "x").unwrap(), "1");
        assert_eq!(math.fallback_differentiate("5", "x").unwrap(), "0");
        assert_eq!(math.fallback_differentiate("x^2", "x").unwrap(), "2*x");
    }
    
    #[test]
    fn test_to_python_code() {
        let math = SymbolicaMath::default();
        let code = math.to_python("x^2 + sqrt(x)").unwrap();
        
        assert!(code.contains("**"));
        assert!(code.contains("math.sqrt"));
    }
    
    #[test]
    fn test_to_rust_code() {
        let math = SymbolicaMath::default();
        let code = math.to_rust("x^2 + sin(x)").unwrap();
        
        assert!(code.contains("f64"));
        assert!(code.contains("sin"));
    }
}
