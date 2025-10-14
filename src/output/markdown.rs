//! Markdown output formatter with fenced code blocks

use std::path::Path;
use crate::output::OutputFormatter;
use crate::utils::{add_line_numbers, determine_backtick_count};
use crate::extensions::get_language_for_extension;

/// Markdown formatter that outputs files as fenced code blocks:
/// filename.ext
/// ```language
/// content
/// ```
pub struct MarkdownFormatter;

impl MarkdownFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl OutputFormatter for MarkdownFormatter {
    fn format_file(&mut self, path: &Path, content: &str, line_numbers: bool) -> String {
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let language = get_language_for_extension(extension);
        
        let content = if line_numbers {
            add_line_numbers(content)
        } else {
            content.to_string()
        };
        
        // Determine backtick count needed
        let backticks = determine_backtick_count(&content);
        
        format!("{}\n{}{}\n{}\n{}",
            path.display(), backticks, language, content, backticks)
    }
    
    fn format_table_of_contents(&mut self, toc: &str) -> String {
        format!("# Table of Contents\n\n```\n{}\n```", toc)
    }
    
    fn start_output(&mut self) -> String {
        String::new()
    }
    
    fn end_output(&mut self) -> String {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_markdown_format() {
        let mut formatter = MarkdownFormatter::new();
        let path = PathBuf::from("test.py");
        let content = "print('hello')";
        
        let result = formatter.format_file(&path, content, false);
        let expected = "test.py\n```python\nprint('hello')\n```";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_markdown_format_unknown_extension() {
        let mut formatter = MarkdownFormatter::new();
        let path = PathBuf::from("test.unknown");
        let content = "some content";
        
        let result = formatter.format_file(&path, content, false);
        let expected = "test.unknown\n```\nsome content\n```";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_markdown_format_with_backticks() {
        let mut formatter = MarkdownFormatter::new();
        let path = PathBuf::from("test.md");
        let content = "This has ``` in it";
        
        let result = formatter.format_file(&path, content, false);
        // Should use 4 backticks since content has 3
        assert!(result.contains("````"));
        assert!(result.contains("This has ``` in it"));
    }

    #[test]
    fn test_markdown_format_with_line_numbers() {
        let mut formatter = MarkdownFormatter::new();
        let path = PathBuf::from("test.py");
        let content = "line 1\nline 2";
        
        let result = formatter.format_file(&path, content, true);
        assert!(result.contains("```python"));
        assert!(result.contains("1  line 1\n2  line 2"));
    }

    #[test]
    fn test_start_end_output() {
        let mut formatter = MarkdownFormatter::new();
        assert_eq!(formatter.start_output(), "");
        assert_eq!(formatter.end_output(), "");
    }
}