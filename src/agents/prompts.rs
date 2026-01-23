//! Prompt engineering for code generation

use crate::agents::language::Language;

/// Example for few-shot prompting
#[derive(Debug, Clone)]
pub struct Example {
    pub task: String,
    pub code: String,
}

/// Prompt builder for LLM code generation
pub struct PromptBuilder {
    system_prompt: String,
    few_shot_examples: Vec<Example>,
}

impl PromptBuilder {
    /// Create new prompt builder
    pub fn new() -> Self {
        Self {
            system_prompt: Self::default_system_prompt(),
            few_shot_examples: Vec::new(),
        }
    }
    
    /// Build code generation prompt
    pub fn build_generation_prompt(
        &self,
        task: &str,
        language: Language,
        flux_position: Option<u8>,
    ) -> String {
        let mut prompt = String::new();
        
        // System instructions
        prompt.push_str(&self.system_prompt);
        prompt.push_str("\n\n");
        
        // Sacred geometry context if flux position provided
        if let Some(pos) = flux_position {
            prompt.push_str(&self.add_sacred_geometry_context(pos));
            prompt.push_str("\n\n");
        }
        
        // Language-specific instructions
        prompt.push_str(&self.language_instructions(language));
        prompt.push_str("\n\n");
        
        // Few-shot examples for this language (if available)
        let examples = self.get_language_examples(language);
        if !examples.is_empty() {
            prompt.push_str("Examples:\n\n");
            for example in examples.iter().take(3) {
                prompt.push_str(&format!("Task: {}\n", example.task));
                prompt.push_str(&format!("```{}\n{}\n```\n\n", language.extension(), example.code));
            }
        }
        
        // The actual task
        prompt.push_str(&format!("Now generate code for this task:\n\n"));
        prompt.push_str(&format!("Task: {}\n", task));
        prompt.push_str(&format!("Language: {}\n\n", language.name()));
        prompt.push_str(&format!("Provide ONLY the code in a ```{} code block, without explanation.\n", language.extension()));
        
        prompt
    }
    
    /// Build correction prompt
    pub fn build_correction_prompt(
        &self,
        code: &str,
        error: &str,
        task: &str,
        language: Language,
    ) -> String {
        format!(
            r#"{}

The following code was generated but produced an error:

```{}
{}
```

Error:
```
{}
```

Original Task: {}
Language: {}

Please provide a CORRECTED version of the code that fixes the error.
Provide ONLY the corrected code in a ```{} code block, without explanation.
"#,
            self.system_prompt,
            language.extension(),
            code,
            error,
            task,
            language.name(),
            language.extension()
        )
    }
    
    /// Add sacred geometry context based on flux position
    pub fn add_sacred_geometry_context(&self, position: u8) -> String {
        match position {
            3 => {
                r#"SACRED POSITION 3 (Ethos - Character/Architecture):
Focus on clean architecture, design patterns, and code structure.
Emphasize maintainability, extensibility, and best practices.
Create code that demonstrates good engineering character."#.to_string()
            }
            6 => {
                r#"SACRED POSITION 6 (Pathos - Emotion/UX):
Focus on user experience, readability, and intuitive interfaces.
Emphasize clear variable names, helpful comments, and elegant solutions.
Create code that feels natural and pleasant to use."#.to_string()
            }
            9 => {
                r#"SACRED POSITION 9 (Logos - Logic/Reasoning):
Focus on algorithmic efficiency, mathematical correctness, and pure logic.
Emphasize optimal complexity, precise calculations, and logical clarity.
Create code that demonstrates rigorous reasoning and correctness."#.to_string()
            }
            _ => String::new(),
        }
    }
    
    /// Add few-shot example
    pub fn add_example(&mut self, task: String, code: String) {
        self.few_shot_examples.push(Example { task, code });
    }
    
    /// Get language-specific examples
    fn get_language_examples(&self, language: Language) -> Vec<Example> {
        // In production, these would be loaded from Confidence Lake or RAG
        // For now, return built-in examples
        match language {
            Language::Python => vec![
                Example {
                    task: "Sort a list of numbers".to_string(),
                    code: "def sort_numbers(numbers):\n    return sorted(numbers)".to_string(),
                },
                Example {
                    task: "Calculate factorial".to_string(),
                    code: "def factorial(n):\n    if n <= 1:\n        return 1\n    return n * factorial(n - 1)".to_string(),
                },
            ],
            Language::Rust => vec![
                Example {
                    task: "Sort a vector of numbers".to_string(),
                    code: "fn sort_numbers(mut numbers: Vec<i32>) -> Vec<i32> {\n    numbers.sort();\n    numbers\n}".to_string(),
                },
                Example {
                    task: "Calculate factorial".to_string(),
                    code: "fn factorial(n: u64) -> u64 {\n    match n {\n        0 | 1 => 1,\n        _ => n * factorial(n - 1)\n    }\n}".to_string(),
                },
            ],
            Language::JavaScript => vec![
                Example {
                    task: "Sort an array of numbers".to_string(),
                    code: "function sortNumbers(numbers) {\n    return numbers.sort((a, b) => a - b);\n}".to_string(),
                },
            ],
            Language::Go => vec![
                Example {
                    task: "Sort a slice of numbers".to_string(),
                    code: "import \"sort\"\n\nfunc sortNumbers(numbers []int) []int {\n    sort.Ints(numbers)\n    return numbers\n}".to_string(),
                },
            ],
            _ => Vec::new(),
        }
    }
    
    /// Language-specific instructions
    fn language_instructions(&self, language: Language) -> String {
        match language {
            Language::Rust => {
                r#"Generate idiomatic Rust code following these guidelines:
- Use proper error handling (Result, Option)
- Follow ownership and borrowing rules
- Use iterators where appropriate
- Include proper type annotations
- Follow Rust naming conventions (snake_case for functions/variables)"#.to_string()
            }
            Language::Python => {
                r#"Generate idiomatic Python code following these guidelines:
- Use type hints where appropriate
- Follow PEP 8 style guide
- Use list comprehensions and generators when suitable
- Include docstrings for functions
- Follow Python naming conventions (snake_case)"#.to_string()
            }
            Language::JavaScript => {
                r#"Generate modern JavaScript code following these guidelines:
- Use ES6+ features (const/let, arrow functions, async/await)
- Use proper error handling (try/catch)
- Follow camelCase naming convention
- Use descriptive variable names
- Include JSDoc comments for functions"#.to_string()
            }
            Language::TypeScript => {
                r#"Generate idiomatic TypeScript code following these guidelines:
- Use proper type annotations
- Use interfaces for complex types
- Follow strict type checking
- Use async/await for asynchronous code
- Follow camelCase naming convention"#.to_string()
            }
            Language::Go => {
                r#"Generate idiomatic Go code following these guidelines:
- Use proper error handling (error return values)
- Follow Go naming conventions (camelCase, PascalCase for exported)
- Use defer for cleanup
- Keep it simple and readable
- Use goroutines and channels where appropriate"#.to_string()
            }
            Language::Elixir => {
                r#"Generate idiomatic Elixir code following these guidelines:
- Use pattern matching
- Follow functional programming principles
- Use pipe operator |> for chaining
- Use guards in function clauses
- Follow snake_case naming convention"#.to_string()
            }
            _ => format!("Generate clean, idiomatic {} code with proper error handling.", language.name()),
        }
    }
    
    /// Default system prompt
    fn default_system_prompt() -> String {
        r#"You are Vortex, an advanced AI coding assistant.

Your role is to generate high-quality, production-ready code that solves the given task.

Guidelines:
1. Write clean, idiomatic code following language best practices
2. Include proper error handling
3. Use descriptive variable and function names
4. Keep code concise but readable
5. Follow the language's style guide
6. Ensure code compiles and runs without errors
7. Focus on correctness and efficiency

IMPORTANT: Provide ONLY the code without explanations, comments, or markdown text.
Wrap the code in a markdown code block with the appropriate language tag."#.to_string()
    }
}

impl Default for PromptBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generation_prompt() {
        let builder = PromptBuilder::new();
        let prompt = builder.build_generation_prompt(
            "Sort a list",
            Language::Python,
            Some(9), // Logos position
        );
        
        assert!(prompt.contains("Sort a list"));
        assert!(prompt.contains("Python"));
        assert!(prompt.contains("SACRED POSITION 9"));
        assert!(prompt.contains("Logos"));
    }
    
    #[test]
    fn test_correction_prompt() {
        let builder = PromptBuilder::new();
        let prompt = builder.build_correction_prompt(
            "def sort(x): return x.sort()",
            "AttributeError: 'list' object attribute 'sort' is None",
            "Sort a list",
            Language::Python,
        );
        
        assert!(prompt.contains("CORRECTED"));
        assert!(prompt.contains("AttributeError"));
        assert!(prompt.contains("Sort a list"));
    }
    
    #[test]
    fn test_sacred_geometry_context() {
        let builder = PromptBuilder::new();
        
        let pos3 = builder.add_sacred_geometry_context(3);
        assert!(pos3.contains("Ethos"));
        assert!(pos3.contains("Architecture"));
        
        let pos6 = builder.add_sacred_geometry_context(6);
        assert!(pos6.contains("Pathos"));
        assert!(pos6.contains("UX"));
        
        let pos9 = builder.add_sacred_geometry_context(9);
        assert!(pos9.contains("Logos"));
        assert!(pos9.contains("Logic"));
    }
}
