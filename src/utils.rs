//! Utility functions for file processing and input/output

use std::io::{self, Read};

/// Add line numbers to content with proper padding
pub fn add_line_numbers(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let padding = lines.len().to_string().len();
    
    lines.iter()
        .enumerate()
        .map(|(i, line)| format!("{:width$}  {}", i + 1, line, width = padding))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Read paths from stdin, respecting the null separator option
pub fn read_paths_from_stdin(use_null_separator: bool) -> io::Result<Vec<String>> {
    use atty::Stream;
    
    // Check if stdin has data available (not a TTY)
    if atty::is(Stream::Stdin) {
        return Ok(vec![]);
    }
    
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    
    let paths: Vec<String> = if use_null_separator {
        buffer.split('\0')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()
    } else {
        buffer.split_whitespace()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()
    };
    
    Ok(paths)
}

/// Determine the appropriate number of backticks needed for markdown code blocks
pub fn determine_backtick_count(content: &str) -> String {
    let mut backticks = "```".to_string();
    while content.contains(&backticks) {
        backticks.push('`');
    }
    backticks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_line_numbers() {
        let content = "line 1\nline 2\nline 3";
        let result = add_line_numbers(content);
        assert_eq!(result, "1  line 1\n2  line 2\n3  line 3");
    }

    #[test]
    fn test_add_line_numbers_padding() {
        let content = "1\n2\n3\n4\n5\n6\n7\n8\n9\n10";
        let result = add_line_numbers(content);
        assert!(result.contains(" 1  1\n"));
        assert!(result.contains("10  10"));
    }

    #[test]
    fn test_determine_backtick_count_simple() {
        let content = "hello world";
        assert_eq!(determine_backtick_count(content), "```");
    }

    #[test]
    fn test_determine_backtick_count_with_backticks() {
        let content = "code with ``` in it";
        assert_eq!(determine_backtick_count(content), "````");
    }

    #[test]
    fn test_determine_backtick_count_multiple() {
        let content = "code with ``` and ```` in it";
        assert_eq!(determine_backtick_count(content), "`````");
    }

    #[test]
    fn test_empty_content() {
        assert_eq!(add_line_numbers(""), "");
        assert_eq!(determine_backtick_count(""), "```");
    }
}