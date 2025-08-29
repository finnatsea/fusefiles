//! Default output formatter - simple format with path, separator, and content

use std::path::Path;
use crate::output::OutputFormatter;
use crate::utils::add_line_numbers;

/// Default formatter that outputs files in simple format:
/// path
/// ---
/// content
/// 
/// ---
pub struct DefaultFormatter;

impl DefaultFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl OutputFormatter for DefaultFormatter {
    fn format_file(&mut self, path: &Path, content: &str, line_numbers: bool) -> String {
        let content = if line_numbers {
            add_line_numbers(content)
        } else {
            content.to_string()
        };
        
        format!("{}\n---\n{}\n\n---", path.display(), content)
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
    fn test_default_format() {
        let mut formatter = DefaultFormatter::new();
        let path = PathBuf::from("test.txt");
        let content = "Hello, world!";
        
        let result = formatter.format_file(&path, content, false);
        assert_eq!(result, "test.txt\n---\nHello, world!\n\n---");
    }

    #[test]
    fn test_default_format_with_line_numbers() {
        let mut formatter = DefaultFormatter::new();
        let path = PathBuf::from("test.txt");
        let content = "line 1\nline 2";
        
        let result = formatter.format_file(&path, content, true);
        assert_eq!(result, "test.txt\n---\n1  line 1\n2  line 2\n\n---");
    }

    #[test]
    fn test_start_end_output() {
        let mut formatter = DefaultFormatter::new();
        assert_eq!(formatter.start_output(), "");
        assert_eq!(formatter.end_output(), "");
    }
}