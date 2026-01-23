//! Text Formatting Utilities
//!
//! Improves paragraph line breaks, readability, and structure of generated text

/// Text formatting configuration
#[derive(Debug, Clone)]
pub struct FormattingConfig {
    /// Insert blank line between paragraphs
    pub paragraph_spacing: bool,
    
    /// Maximum line length before wrapping
    pub max_line_length: Option<usize>,
    
    /// Ensure proper sentence spacing
    pub fix_sentence_spacing: bool,
    
    /// Clean up excessive whitespace
    pub trim_whitespace: bool,
    
    /// Add line breaks after common paragraph markers
    pub detect_paragraph_breaks: bool,
}

fn protect_fenced_code(text: &str) -> (String, Vec<String>) {
    let mut out = String::new();
    let mut blocks: Vec<String> = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0;
    while let Some(start_rel) = text[i..].find("```") {
        let start = i + start_rel;
        let before = &text[i..start];
        out.push_str(before);
        let after_start = start + 3;
        if let Some(end_rel) = text[after_start..].find("```") {
            let end = after_start + end_rel + 3;
            let block = &text[start..end];
            let idx = blocks.len();
            let placeholder = format!("§CODE_BLOCK_{}§", idx);
            out.push_str(&placeholder);
            blocks.push(block.to_string());
            i = end;
        } else {
            out.push_str(&text[start..]);
            i = bytes.len();
            break;
        }
    }
    if i < bytes.len() {
        out.push_str(&text[i..]);
    }
    (out, blocks)
}

fn protect_inline_code(text: &str) -> (String, Vec<String>) {
    let mut out = String::new();
    let mut blocks: Vec<String> = Vec::new();
    let mut chars = text.chars().peekable();
    let mut idx = 0;
    while let Some(c) = chars.next() {
        if c == '`' {
            let mut block = String::from("`");
            let mut found = false;
            while let Some(nc) = chars.next() {
                block.push(nc);
                if nc == '`' {
                    found = true;
                    break;
                }
            }
            if found {
                let placeholder = format!("§INLINE_CODE_{}§", idx);
                idx += 1;
                blocks.push(block);
                out.push_str(&placeholder);
            } else {
                out.push_str(&block);
            }
        } else {
            out.push(c);
        }
    }
    (out, blocks)
}

fn restore_code_placeholders(mut text: String, fenced: &Vec<String>, inline: &Vec<String>) -> String {
    for (i, block) in fenced.iter().enumerate() {
        let ph = format!("§CODE_BLOCK_{}§", i);
        text = text.replace(&ph, block);
    }
    for (i, block) in inline.iter().enumerate() {
        let ph = format!("§INLINE_CODE_{}§", i);
        text = text.replace(&ph, block);
    }
    text
}

impl Default for FormattingConfig {
    fn default() -> Self {
        Self {
            paragraph_spacing: true,
            max_line_length: Some(80),
            fix_sentence_spacing: true,
            trim_whitespace: true,
            detect_paragraph_breaks: true,
        }
    }

    #[test]
    fn test_preserve_fenced_code_blocks() {
        let input = r#"Intro paragraph.

```rust
fn main() {
    println!("hello");
}
```

Conclusion paragraph.
"#;
        let out = format_quick(input);
        // Code fence and content must be intact
        assert!(out.contains("```rust"));
        assert!(out.contains("println!(\"hello\");"));
        assert!(out.contains("```"));
    }

    #[test]
    fn test_preserve_inline_code() {
        let input = "Use `foo()` inline and keep punctuation. Next sentence.";
        let out = format_quick(input);
        assert!(out.contains("`foo()`"));
    }
}

/// Format text with improved paragraph line breaks
pub fn format_paragraphs(text: &str, config: &FormattingConfig) -> String {
    let (result, fenced_blocks) = protect_fenced_code(text);
    let (mut result, inline_blocks) = protect_inline_code(&result);
    
    // Step 1: Clean up excessive whitespace
    if config.trim_whitespace {
        result = clean_whitespace(&result);
    }
    
    // Step 2: Detect and insert paragraph breaks
    if config.detect_paragraph_breaks {
        result = detect_paragraphs(&result);
    }
    
    // Step 3: Add paragraph spacing
    if config.paragraph_spacing {
        result = add_paragraph_spacing(&result);
    }
    
    // Step 4: Fix sentence spacing
    if config.fix_sentence_spacing {
        result = fix_sentence_spacing(&result);
    }
    
    // Step 5: Word wrap if needed
    if let Some(max_len) = config.max_line_length {
        result = word_wrap(&result, max_len);
    }
    
    let result = restore_code_placeholders(result, &fenced_blocks, &inline_blocks);
    result
}

/// Clean up excessive whitespace (AGGRESSIVE MODE)
fn clean_whitespace(text: &str) -> String {
    // Remove trailing whitespace from each line
    let lines: Vec<&str> = text.lines().collect();
    let cleaned: Vec<String> = lines.iter()
        .map(|line| line.trim_end().to_string())
        .collect();
    
    // Remove excessive blank lines (more than 1 consecutive now - more aggressive)
    let mut result = Vec::new();
    let mut blank_count = 0;
    
    for line in cleaned {
        if line.trim().is_empty() {
            blank_count += 1;
            if blank_count <= 1 {  // Changed from 2 to 1
                result.push(line);
            }
        } else {
            blank_count = 0;
            result.push(line);
        }
    }
    
    result.join("\n")
}

/// Detect paragraph breaks based on content patterns (AGGRESSIVE MODE)
fn detect_paragraphs(text: &str) -> String {
    let mut result = String::new();
    
    // Split by periods but preserve them
    let sentences: Vec<&str> = text.split('.').collect();
    
    for (i, sentence) in sentences.iter().enumerate() {
        let trimmed = sentence.trim();
        
        if trimmed.is_empty() {
            continue;
        }
        
        result.push_str(trimmed);
        result.push('.');
        
        // Add line break after sentence if:
        // 1. It's followed by a capital letter (new sentence/paragraph)
        // 2. It's a numbered/bulleted list
        // 3. It ends with certain keywords indicating topic change
        if i < sentences.len() - 1 {
            let next = sentences[i + 1].trim();
            
            if should_break_paragraph(trimmed, next) {
                result.push_str("\n\n");
            } else {
                result.push(' ');
            }
        }
    }
    
    result
}

/// Determine if a paragraph break should be inserted
fn should_break_paragraph(current: &str, next: &str) -> bool {
    if next.is_empty() {
        return false;
    }
    
    // Check if next sentence starts with capital (new thought)
    if let Some(first_char) = next.chars().next() {
        if first_char.is_uppercase() && is_new_paragraph_indicator(current, next) {
            return true;
        }
    }
    
    // Check for list markers
    if next.trim_start().starts_with('-') || 
       next.trim_start().starts_with('•') ||
       next.trim_start().chars().next().map_or(false, |c| c.is_numeric()) {
        return true;
    }
    
    // Check for topic transition words
    let transition_words = [
        "However", "Furthermore", "Moreover", "Additionally",
        "In contrast", "On the other hand", "Meanwhile",
        "First", "Second", "Third", "Finally",
        "Therefore", "Thus", "Consequently", "As a result",
    ];
    
    for word in &transition_words {
        if next.trim_start().starts_with(word) {
            return true;
        }
    }
    
    false
}

/// Check if sentence endings indicate new paragraph
fn is_new_paragraph_indicator(current: &str, next: &str) -> bool {
    // Long sentences often start new paragraphs
    if current.len() > 100 {
        return true;
    }
    
    // Questions often indicate topic shift
    if current.contains('?') {
        return true;
    }
    
    // Next sentence starts with common paragraph starters
    let paragraph_starters = [
        "The", "This", "These", "It", "In", "For", "To",
        "When", "Where", "Why", "How", "What", "Who",
    ];
    
    for starter in &paragraph_starters {
        if next.starts_with(starter) && current.len() > 50 {
            return true;
        }
    }
    
    false
}

/// Add spacing between paragraphs (AGGRESSIVE MODE)
fn add_paragraph_spacing(text: &str) -> String {
    let text = fix_bullet_spacing(text);  // Fix bullets first
    let text = fix_numbered_list_spacing(&text);  // Fix numbered lists
    
    // Final cleanup pass: ensure max 1 consecutive blank line
    let lines: Vec<&str> = text.lines().collect();
    let mut result = Vec::new();
    let mut blank_count = 0;
    
    for line in lines {
        let is_blank = line.trim().is_empty();
        
        if is_blank {
            blank_count += 1;
            if blank_count <= 1 {  // Only allow 1 consecutive blank line
                result.push(line.to_string());
            }
        } else {
            blank_count = 0;
            result.push(line.to_string());
        }
    }
    
    result.join("\n")
}

/// Fix bullet point spacing - ensure line breaks after each bullet
fn fix_bullet_spacing(text: &str) -> String {
    let mut result = String::new();
    let lines: Vec<&str> = text.lines().collect();
    let mut prev_was_blank = false;
    
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let is_blank = trimmed.is_empty();
        
        // Detect bullet points: *, -, •, or "* "
        if trimmed.starts_with("* ") || trimmed.starts_with("- ") || trimmed.starts_with("• ") {
            // Add ONE blank line before bullet only if previous line wasn't blank and it's not the first line
            if i > 0 && !prev_was_blank && !result.is_empty() {
                result.push('\n');
            }
            result.push_str(line);
            result.push('\n');
            prev_was_blank = false;
        } else {
            result.push_str(line);
            result.push('\n');
            prev_was_blank = is_blank;
        }
    }
    
    result
}

/// Fix numbered list spacing - ensure line breaks after each number
fn fix_numbered_list_spacing(text: &str) -> String {
    let mut result = String::new();
    let lines: Vec<&str> = text.lines().collect();
    let mut prev_was_blank = false;
    
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let is_blank = trimmed.is_empty();
        
        // Detect numbered lists: "1. ", "2. ", etc.
        let is_numbered = trimmed.chars().next().map(|c| c.is_numeric()).unwrap_or(false)
            && trimmed.contains(". ");
        
        if is_numbered {
            // Add ONE blank line before numbered item only if previous line wasn't blank and it's not the first line
            if i > 0 && !prev_was_blank && !result.is_empty() {
                result.push('\n');
            }
            result.push_str(line);
            result.push('\n');
            prev_was_blank = false;
        } else {
            result.push_str(line);
            result.push('\n');
            prev_was_blank = is_blank;
        }
    }
    
    result
}

/// Fix spacing after sentence-ending punctuation
fn fix_sentence_spacing(text: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = text.chars().collect();
    
    for i in 0..chars.len() {
        result.push(chars[i]);
        
        // After sentence-ending punctuation, ensure single space
        if i < chars.len() - 1 {
            let current = chars[i];
            let next = chars[i + 1];
            
            if (current == '.' || current == '!' || current == '?') && 
               next != ' ' && next != '\n' && next != '\t' {
                result.push(' ');
            }
        }
    }
    
    result
}

/// Wrap long lines to maximum length
fn word_wrap(text: &str, max_length: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let mut result = Vec::new();
    
    for line in lines {
        let trimmed = line.trim();
        
        // Skip wrapping for:
        // - Short lines
        // - Empty lines
        // - Markdown tables (lines starting with |)
        // - Markdown headers (lines starting with #)
        // - Code blocks (lines starting with ```)
        // - Horizontal rules (lines with ---)
        if line.len() <= max_length || 
           trimmed.is_empty() || 
           trimmed.starts_with('|') ||
           trimmed.starts_with('#') ||
           trimmed.starts_with("```") ||
           trimmed.starts_with("---") {
            result.push(line.to_string());
            continue;
        }
        
        // Wrap long lines
        let words: Vec<&str> = line.split_whitespace().collect();
        let mut current_line = String::new();
        
        for word in words {
            if current_line.len() + word.len() + 1 > max_length {
                if !current_line.is_empty() {
                    result.push(current_line.trim().to_string());
                    current_line = String::new();
                }
            }
            
            if !current_line.is_empty() {
                current_line.push(' ');
            }
            current_line.push_str(word);
        }
        
        if !current_line.is_empty() {
            result.push(current_line.trim().to_string());
        }
    }
    
    result.join("\n")
}

/// Quick format with default settings (AGGRESSIVE MODE)
pub fn format_quick(text: &str) -> String {
    let config = FormattingConfig {
        paragraph_spacing: true,
        max_line_length: Some(100),  // Increased from 80 for better readability
        fix_sentence_spacing: true,
        trim_whitespace: true,
        detect_paragraph_breaks: true,
    };
    format_paragraphs(text, &config)
}

/// Format for code output (no wrapping, preserve formatting)
pub fn format_code(text: &str) -> String {
    let config = FormattingConfig {
        paragraph_spacing: false,
        max_line_length: None,
        fix_sentence_spacing: false,
        trim_whitespace: true,
        detect_paragraph_breaks: false,
    };
    
    format_paragraphs(text, &config)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_clean_whitespace() {
        let input = "Line 1  \n\n\n\nLine 2   \n";
        let output = clean_whitespace(input);
        assert!(!output.contains("  \n"));
        assert!(!output.contains("\n\n\n\n"));
    }
    
    #[test]
    fn test_paragraph_detection() {
        let input = "This is sentence one.This is sentence two.However this starts a new paragraph.";
        let output = detect_paragraphs(input);
        assert!(output.contains("\n\n"));
    }
    
    #[test]
    fn test_sentence_spacing() {
        let input = "First sentence.Second sentence.Third sentence.";
        let output = fix_sentence_spacing(input);
        assert!(output.contains(". Second"));
        assert!(output.contains(". Third"));
    }
    
    #[test]
    fn test_word_wrap() {
        let long_line = "This is a very long line that should be wrapped at a certain character limit for better readability and formatting purposes.";
        let output = word_wrap(long_line, 40);
        let lines: Vec<&str> = output.lines().collect();
        for line in lines {
            assert!(line.len() <= 40 || line.split_whitespace().count() == 1);
        }
    }
    
    #[test]
    fn test_format_quick() {
        let input = "Sentence one.Sentence two.However, this is a new idea.";
        let output = format_quick(input);
        assert!(output.contains("\n"));
    }
    
    #[test]
    fn test_markdown_table_preservation() {
        let input = "Here is a table:\n\
                     | Column 1 | Column 2 | Column 3 |\n\
                     | --- | --- | --- |\n\
                     | Data 1 | Data 2 | Data 3 |\n\
                     | More data | Even more | Last column |";
        let output = format_quick(input);
        
        // Tables should remain on separate lines
        assert!(output.contains("| Column 1 |"));
        assert!(output.contains("| --- |"));
        assert!(output.contains("| Data 1 |"));
        
        // Count lines - should have at least 4 (table rows)
        let line_count = output.lines().count();
        assert!(line_count >= 4);
    }
}
