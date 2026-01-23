//! Response Processor - Extract and Handle Task Lists, Format Markdown
//!
//! This module processes agent responses to:
//! 1. Extract task lists from markdown
//! 2. Remove task lists from chat responses
//! 3. Send tasks to task management system
//! 4. Format markdown for proper rendering

use regex::Regex;
use serde::{Deserialize, Serialize};
use pulldown_cmark::{Parser, Options, html};

/// Extracted task from markdown response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedTask {
    pub description: String,
    pub completed: bool,
    pub order: usize,
}

/// Processed response with tasks extracted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedResponse {
    /// Cleaned response without task lists
    pub content: String,
    /// Extracted tasks
    pub tasks: Vec<ExtractedTask>,
    /// Whether any tasks were found
    pub has_tasks: bool,
}

/// Response processor for handling agent outputs
pub struct ResponseProcessor {
    task_list_regex: Regex,
}

impl ResponseProcessor {
    /// Create new response processor
    pub fn new() -> Self {
        Self {
            // Match markdown task lists: - [ ] or - [x]
            task_list_regex: Regex::new(r"(?m)^[\s]*[-*]\s*\[([ xX])\]\s*(.+)$").unwrap(),
        }
    }
    
    /// Process response: extract tasks, clean content, convert markdown to HTML
    pub fn process(&self, raw_response: &str) -> ProcessedResponse {
        let mut tasks = Vec::new();
        let mut order = 0;
        
        // Extract tasks
        for cap in self.task_list_regex.captures_iter(raw_response) {
            let completed = cap[1].trim().to_lowercase() == "x";
            let description = cap[2].trim().to_string();
            
            if !description.is_empty() {
                tasks.push(ExtractedTask {
                    description,
                    completed,
                    order,
                });
                order += 1;
            }
        }
        
        // Remove task lists from content
        let cleaned_content = self.remove_task_lists(raw_response);
        
        // Convert markdown to HTML
        let html_content = self.markdown_to_html(&cleaned_content);
        
        let has_tasks = !tasks.is_empty();
        
        ProcessedResponse {
            content: html_content,  // HTML instead of markdown
            tasks,
            has_tasks,
        }
    }
    
    /// Convert markdown to HTML using pulldown-cmark
    fn markdown_to_html(&self, markdown: &str) -> String {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_TASKLISTS);
        
        let parser = Parser::new_ext(markdown, options);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);
        html_output
    }
    
    /// Remove task lists from markdown
    fn remove_task_lists(&self, content: &str) -> String {
        let mut result = String::new();
        let mut in_task_section = false;
        
        for line in content.lines() {
            // Check if line is a task list item
            if self.task_list_regex.is_match(line) {
                in_task_section = true;
                continue; // Skip task list lines
            }
            
            // Check if we're exiting task section
            if in_task_section && !line.trim().is_empty() && !line.trim().starts_with('-') && !line.trim().starts_with('*') {
                in_task_section = false;
            }
            
            // Skip task list headers
            if line.to_lowercase().contains("task list") || 
               (line.to_lowercase().contains("tasks") && line.trim().starts_with('#')) {
                continue;
            }
            
            // Add non-task lines
            if !in_task_section {
                result.push_str(line);
                result.push('\n');
            }
        }
        
        // Clean up extra newlines
        self.clean_extra_newlines(&result)
    }
    
    /// Clean up excess newlines (max 2 consecutive)
    fn clean_extra_newlines(&self, content: &str) -> String {
        let newline_regex = Regex::new(r"\n{3,}").unwrap();
        newline_regex.replace_all(content, "\n\n").to_string()
    }
    
    /// Extract only tasks from markdown (for API use)
    pub fn extract_tasks_only(&self, content: &str) -> Vec<ExtractedTask> {
        let mut tasks = Vec::new();
        let mut order = 0;
        
        for cap in self.task_list_regex.captures_iter(content) {
            let completed = cap[1].trim().to_lowercase() == "x";
            let description = cap[2].trim().to_string();
            
            if !description.is_empty() {
                tasks.push(ExtractedTask {
                    description,
                    completed,
                    order,
                });
                order += 1;
            }
        }
        
        tasks
    }
    
    /// Check if content contains task lists
    pub fn has_task_lists(&self, content: &str) -> bool {
        self.task_list_regex.is_match(content)
    }

    /// Format markdown text with proper spacing and structure
    pub fn format_markdown(&self, input: &str) -> String {
        let mut result = String::new();
        let lines: Vec<&str> = input.lines().collect();
        
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            // Add spacing around headers
            if trimmed.starts_with('#') {
                if i > 0 && !lines[i - 1].trim().is_empty() {
                    result.push('\n');
                }
                result.push_str(line);
                result.push('\n');
                if i < lines.len() - 1 && !lines[i + 1].trim().is_empty() {
                    result.push('\n');
                }
            } else {
                result.push_str(line);
                result.push('\n');
            }
        }
        
        self.clean_extra_newlines(&result)
    }
}

impl Default for ResponseProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_tasks() {
        let processor = ResponseProcessor::new();
        
        let input = r#"Here's what we need to do:

- [ ] First task
- [ ] Second task
- [x] Completed task

Some other content here."#;
        
        let result = processor.process(input);
        
        assert_eq!(result.tasks.len(), 3);
        assert_eq!(result.tasks[0].description, "First task");
        assert!(!result.tasks[0].completed);
        assert_eq!(result.tasks[2].description, "Completed task");
        assert!(result.tasks[2].completed);
    }
    
    #[test]
    fn test_remove_task_lists() {
        let processor = ResponseProcessor::new();
        
        let input = r#"# Understanding Consciousness

Here's my answer.

### Task List
- [ ] Explore quantum mechanics
- [ ] Investigate sacred geometry

This is more content."#;
        
        let result = processor.process(input);
        
        assert!(!result.content.contains("[ ]"));
        assert!(!result.content.contains("Task List"));
        assert!(result.content.contains("Understanding Consciousness"));
        assert!(result.content.contains("This is more content"));
    }
    
    #[test]
    fn test_format_headers() {
        let processor = ResponseProcessor::new();
        
        let input = "Some text\n# Header\nMore text";
        let result = processor.format_markdown(input);
        
        // Should have spacing around header
        assert!(result.contains("\n# Header\n"));
    }
    
    #[test]
    fn test_clean_newlines() {
        let processor = ResponseProcessor::new();
        
        let input = "Text\n\n\n\n\nMore text";
        let result = processor.clean_extra_newlines(input);
        
        // Should have max 2 newlines
        assert_eq!(result, "Text\n\nMore text");
    }
}
