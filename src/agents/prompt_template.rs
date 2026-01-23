/// Enhanced Prompt Template System
/// Addresses inconsistent prompt quality identified in testing
/// 
/// Key Finding: Medium problems (100% success) had better prompts than Easy problems (80%)
/// Solution: Standardize all prompts with algorithm hints, examples, and constraints

use crate::agents::language::Language;
use serde::{Deserialize, Serialize};

/// Difficulty levels for adaptive prompting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Difficulty {
    Easy = 1,
    Medium = 2,
    Hard = 3,
}

/// Code complexity requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityRequirement {
    pub time: Option<String>,   // e.g., "O(n)"
    pub space: Option<String>,  // e.g., "O(1)"
}

/// Problem example with input/output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Example {
    pub input: String,
    pub output: String,
    pub explanation: Option<String>,
}

/// Enhanced prompt template with structured components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    /// Core task description
    pub task_description: String,
    
    /// Difficulty level (affects what guidance is included)
    pub difficulty: Difficulty,
    
    /// Target programming language
    pub language: Language,
    
    /// Algorithm hint (e.g., "Use two pointers approach")
    /// Automatically included for Medium+ difficulty
    pub algorithm_hint: Option<String>,
    
    /// Concrete examples with expected output
    pub examples: Vec<Example>,
    
    /// Time/space complexity requirements
    pub complexity: Option<ComplexityRequirement>,
    
    /// Constraints and edge cases
    pub constraints: Vec<String>,
    
    /// Edge cases to handle
    pub edge_cases: Vec<String>,
    
    /// Function signature if specific format required
    pub function_signature: Option<String>,
    
    /// Sacred geometry position guidance (3, 6, or 9)
    pub sacred_position: Option<u8>,
}

impl PromptTemplate {
    /// Create a new template with minimal required fields
    pub fn new(task: String, language: Language) -> Self {
        Self {
            task_description: task,
            difficulty: Difficulty::Easy,
            language,
            algorithm_hint: None,
            examples: Vec::new(),
            complexity: None,
            constraints: Vec::new(),
            edge_cases: Vec::new(),
            function_signature: None,
            sacred_position: None,
        }
    }
    
    /// Builder pattern: Set difficulty
    pub fn with_difficulty(mut self, difficulty: Difficulty) -> Self {
        self.difficulty = difficulty;
        self
    }
    
    /// Builder pattern: Add algorithm hint
    pub fn with_algorithm_hint(mut self, hint: String) -> Self {
        self.algorithm_hint = Some(hint);
        self
    }
    
    /// Builder pattern: Add example
    pub fn with_example(mut self, input: String, output: String) -> Self {
        self.examples.push(Example {
            input,
            output,
            explanation: None,
        });
        self
    }
    
    /// Builder pattern: Add example with explanation
    pub fn with_example_explained(
        mut self,
        input: String,
        output: String,
        explanation: String,
    ) -> Self {
        self.examples.push(Example {
            input,
            output,
            explanation: Some(explanation),
        });
        self
    }
    
    /// Builder pattern: Set complexity requirements
    pub fn with_complexity(mut self, time: Option<String>, space: Option<String>) -> Self {
        self.complexity = Some(ComplexityRequirement { time, space });
        self
    }
    
    /// Builder pattern: Add constraint
    pub fn with_constraint(mut self, constraint: String) -> Self {
        self.constraints.push(constraint);
        self
    }
    
    /// Builder pattern: Add edge case
    pub fn with_edge_case(mut self, edge_case: String) -> Self {
        self.edge_cases.push(edge_case);
        self
    }
    
    /// Builder pattern: Set function signature
    pub fn with_signature(mut self, signature: String) -> Self {
        self.function_signature = Some(signature);
        self
    }
    
    /// Builder pattern: Set sacred position
    pub fn with_sacred_position(mut self, position: u8) -> Self {
        if [3, 6, 9].contains(&position) {
            self.sacred_position = Some(position);
        }
        self
    }
    
    /// Build the complete prompt
    pub fn build(&self) -> String {
        let mut prompt = String::new();
        
        // 1. Task Header
        prompt.push_str("# Task\n");
        prompt.push_str(&self.task_description);
        prompt.push_str("\n\n");
        
        // 2. Language Specification
        prompt.push_str(&format!("# Language\n{}\n\n", self.language.name()));
        
        // 3. Function Signature (if specified)
        if let Some(sig) = &self.function_signature {
            prompt.push_str(&format!("# Function Signature\n```{}\n{}\n```\n\n", 
                self.language.extension(), sig));
        }
        
        // 4. Algorithm Hint (for Medium+ or if explicitly provided)
        if self.difficulty as u8 >= 2 || self.algorithm_hint.is_some() {
            if let Some(hint) = &self.algorithm_hint {
                prompt.push_str("# Recommended Approach\n");
                prompt.push_str(hint);
                prompt.push_str("\n\n");
            }
        }
        
        // 5. Examples
        if !self.examples.is_empty() {
            prompt.push_str("# Examples\n\n");
            for (i, example) in self.examples.iter().enumerate() {
                prompt.push_str(&format!("## Example {}\n", i + 1));
                prompt.push_str(&format!("Input: {}\n", example.input));
                prompt.push_str(&format!("Output: {}\n", example.output));
                if let Some(exp) = &example.explanation {
                    prompt.push_str(&format!("Explanation: {}\n", exp));
                }
                prompt.push_str("\n");
            }
        }
        
        // 6. Complexity Requirements
        if let Some(complexity) = &self.complexity {
            prompt.push_str("# Complexity Requirements\n");
            if let Some(time) = &complexity.time {
                prompt.push_str(&format!("- Time Complexity: {}\n", time));
            }
            if let Some(space) = &complexity.space {
                prompt.push_str(&format!("- Space Complexity: {}\n", space));
            }
            prompt.push_str("\n");
        }
        
        // 7. Constraints
        if !self.constraints.is_empty() {
            prompt.push_str("# Constraints\n");
            for constraint in &self.constraints {
                prompt.push_str(&format!("- {}\n", constraint));
            }
            prompt.push_str("\n");
        }
        
        // 8. Edge Cases
        if !self.edge_cases.is_empty() {
            prompt.push_str("# Edge Cases to Handle\n");
            for edge_case in &self.edge_cases {
                prompt.push_str(&format!("- {}\n", edge_case));
            }
            prompt.push_str("\n");
        }
        
        // 9. Sacred Position Guidance
        if let Some(position) = self.sacred_position {
            let guidance = match position {
                3 => "Focus on CODE CLARITY and BEST PRACTICES (Ethos - Character)",
                6 => "Emphasize EDGE CASE HANDLING and ROBUSTNESS (Pathos - Emotion/Feeling)",
                9 => "Optimize for ALGORITHMIC EFFICIENCY (Logos - Logic/Reason)",
                _ => "",
            };
            prompt.push_str("# Sacred Geometry Guidance\n");
            prompt.push_str(&format!("Position {}: {}\n\n", position, guidance));
        }
        
        // 10. Language-Specific Guidelines
        prompt.push_str(&self.get_language_specific_guidelines());
        
        // 11. Output Format
        prompt.push_str("# Output Format\n");
        prompt.push_str(&format!(
            "Please provide ONLY the {} code implementation. No explanations before or after.\n",
            self.language.name()
        ));
        prompt.push_str("The code should be production-ready and fully functional.\n");
        
        prompt
    }
    
    /// Get language-specific guidelines
    fn get_language_specific_guidelines(&self) -> String {
        let mut guidelines = String::from("# Language-Specific Guidelines\n");
        
        match self.language {
            Language::Python => {
                guidelines.push_str("- Use type hints where appropriate\n");
                guidelines.push_str("- Follow PEP 8 style guidelines\n");
                guidelines.push_str("- Use list comprehensions for concise operations\n");
                guidelines.push_str("- Handle exceptions appropriately\n");
            },
            Language::Rust => {
                guidelines.push_str("- Respect ownership and borrowing rules\n");
                guidelines.push_str("- Use &mut for mutable references\n");
                guidelines.push_str("- Handle Result/Option types properly\n");
                guidelines.push_str("- Be careful with usize underflow (use checked operations)\n");
                guidelines.push_str("- Prefer iterators over index-based loops\n");
            },
            Language::JavaScript => {
                guidelines.push_str("- Use const/let instead of var\n");
                guidelines.push_str("- Use arrow functions where appropriate\n");
                guidelines.push_str("- Handle promises with async/await\n");
                guidelines.push_str("- Use modern ES6+ features\n");
            },
            Language::TypeScript => {
                guidelines.push_str("- Define proper interfaces/types\n");
                guidelines.push_str("- Use strict type checking\n");
                guidelines.push_str("- Avoid 'any' type when possible\n");
                guidelines.push_str("- Use generics for reusable code\n");
            },
            Language::Go => {
                guidelines.push_str("- Follow Go idioms and conventions\n");
                guidelines.push_str("- Handle errors explicitly\n");
                guidelines.push_str("- Use defer for cleanup\n");
                guidelines.push_str("- Keep functions focused and simple\n");
            },
            Language::Java => {
                guidelines.push_str("- Follow Java naming conventions\n");
                guidelines.push_str("- Use appropriate access modifiers\n");
                guidelines.push_str("- Handle exceptions properly\n");
                guidelines.push_str("- Use generics for type safety\n");
            },
            _ => {
                guidelines.push_str(&format!("- Follow {} best practices\n", self.language.name()));
                guidelines.push_str("- Write clean, readable code\n");
                guidelines.push_str("- Handle edge cases\n");
            }
        }
        
        guidelines.push_str("\n");
        guidelines
    }
}

/// Pre-built templates for common problems
pub struct PromptTemplateLibrary;

impl PromptTemplateLibrary {
    /// Template for Two Sum problem
    pub fn two_sum(language: Language) -> PromptTemplate {
        PromptTemplate::new(
            "Write a function that takes an array of integers and a target integer. \
             Return the indices of two numbers that add up to the target.".to_string(),
            language,
        )
        .with_difficulty(Difficulty::Easy)
        .with_algorithm_hint(
            "Use a hash table (dictionary/map) to store seen numbers with their indices. \
             For each number, check if (target - number) exists in the hash table. \
             This achieves O(n) time complexity.".to_string()
        )
        .with_example(
            "nums = [2, 7, 11, 15], target = 9".to_string(),
            "[0, 1]".to_string()
        )
        .with_example_explained(
            "nums = [3, 2, 4], target = 6".to_string(),
            "[1, 2]".to_string(),
            "nums[1] + nums[2] = 2 + 4 = 6".to_string(),
        )
        .with_complexity(Some("O(n)".to_string()), Some("O(n)".to_string()))
        .with_constraint("Each input has exactly one solution".to_string())
        .with_constraint("You may not use the same element twice".to_string())
        .with_edge_case("Array with only 2 elements".to_string())
        .with_edge_case("Large numbers (within integer range)".to_string())
    }
    
    /// Template for Edit Distance problem
    pub fn edit_distance(language: Language) -> PromptTemplate {
        PromptTemplate::new(
            "Write a function to find the minimum number of operations required to \
             convert one string to another. Operations: insert, delete, replace.".to_string(),
            language,
        )
        .with_difficulty(Difficulty::Hard)
        .with_algorithm_hint(
            "Use Dynamic Programming with a 2D table where dp[i][j] represents the \
             minimum edit distance between s1[0..i] and s2[0..j].\n\n\
             Base cases:\n\
             - dp[0][j] = j (insert j characters)\n\
             - dp[i][0] = i (delete i characters)\n\n\
             Recurrence:\n\
             - If s1[i] == s2[j]: dp[i][j] = dp[i-1][j-1]\n\
             - Else: dp[i][j] = 1 + min(dp[i-1][j], dp[i][j-1], dp[i-1][j-1])\n\n\
             Return dp[m][n] where m, n are string lengths.".to_string()
        )
        .with_example(
            "s1 = 'horse', s2 = 'ros'".to_string(),
            "3".to_string()
        )
        .with_example_explained(
            "s1 = 'intention', s2 = 'execution'".to_string(),
            "5".to_string(),
            "intention -> inention (delete t) -> enention (replace i with e) -> \
             exention (replace n with x) -> exection (replace n with c) -> \
             execution (insert u)".to_string(),
        )
        .with_complexity(Some("O(m*n)".to_string()), Some("O(m*n)".to_string()))
        .with_edge_case("Empty string as input".to_string())
        .with_edge_case("Identical strings".to_string())
        .with_sacred_position(9)  // Hard problem → Position 9 (Logos)
    }
    
    /// Template for 3Sum problem
    pub fn three_sum(language: Language) -> PromptTemplate {
        PromptTemplate::new(
            "Write a function that finds all unique triplets in an array that sum to zero.".to_string(),
            language,
        )
        .with_difficulty(Difficulty::Medium)
        .with_algorithm_hint(
            "1. Sort the array first\n\
             2. Use three pointers: i (outer loop), left, right\n\
             3. For each i, use two pointers to find pairs that sum to -nums[i]\n\
             4. Skip duplicates to avoid duplicate triplets\n\
             This achieves O(n²) time complexity.".to_string()
        )
        .with_example(
            "nums = [-1, 0, 1, 2, -1, -4]".to_string(),
            "[[-1, -1, 2], [-1, 0, 1]]".to_string()
        )
        .with_complexity(Some("O(n²)".to_string()), Some("O(1)".to_string()))
        .with_constraint("Result should not contain duplicate triplets".to_string())
        .with_edge_case("Array with less than 3 elements".to_string())
        .with_edge_case("Array with all zeros".to_string())
        .with_sacred_position(6)  // Medium problem → Position 6 (Pathos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_template() {
        let template = PromptTemplate::new(
            "Write a function to reverse a string".to_string(),
            Language::Python,
        )
        .with_example("'hello'".to_string(), "'olleh'".to_string());
        
        let prompt = template.build();
        
        assert!(prompt.contains("# Task"));
        assert!(prompt.contains("reverse a string"));
        assert!(prompt.contains("# Language"));
        assert!(prompt.contains("Python"));
        assert!(prompt.contains("# Examples"));
        assert!(prompt.contains("'hello'"));
    }
    
    #[test]
    fn test_difficulty_based_hints() {
        let easy = PromptTemplate::new("Task".to_string(), Language::Python)
            .with_difficulty(Difficulty::Easy);
        
        let medium = PromptTemplate::new("Task".to_string(), Language::Python)
            .with_difficulty(Difficulty::Medium)
            .with_algorithm_hint("Use hash table".to_string());
        
        let easy_prompt = easy.build();
        let medium_prompt = medium.build();
        
        // Medium should include algorithm hint
        assert!(medium_prompt.contains("# Recommended Approach"));
        assert!(medium_prompt.contains("hash table"));
    }
    
    #[test]
    fn test_sacred_position_guidance() {
        let template = PromptTemplate::new("Task".to_string(), Language::Python)
            .with_sacred_position(9);
        
        let prompt = template.build();
        
        assert!(prompt.contains("Sacred Geometry Guidance"));
        assert!(prompt.contains("Position 9"));
        assert!(prompt.contains("ALGORITHMIC EFFICIENCY"));
    }
    
    #[test]
    fn test_library_templates() {
        let two_sum = PromptTemplateLibrary::two_sum(Language::Python);
        let prompt = two_sum.build();
        
        assert!(prompt.contains("hash table"));
        assert!(prompt.contains("O(n)"));
        assert!(prompt.contains("[2, 7, 11, 15]"));
    }
}
