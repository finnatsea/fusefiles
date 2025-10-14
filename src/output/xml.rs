//! XML output formatter for Claude's preferred format

use crate::output::OutputFormatter;
use crate::utils::add_line_numbers;
use std::path::Path;

/// XML formatter that outputs files in Claude's preferred XML format:
/// <documents>
/// <document index="1">
/// <source>path</source>
/// <document_content>
/// content
/// </document_content>
/// </document>
/// </documents>
pub struct XmlFormatter {
    index: usize,
}

impl Default for XmlFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl XmlFormatter {
    pub fn new() -> Self {
        Self { index: 1 }
    }
}

impl OutputFormatter for XmlFormatter {
    fn format_file(&mut self, path: &Path, content: &str, line_numbers: bool) -> String {
        let content = if line_numbers {
            add_line_numbers(content)
        } else {
            content.to_string()
        };

        let output = format!(
            r#"<document index="{}">
<source>{}</source>
<document_content>
{}
</document_content>
</document>"#,
            self.index,
            path.display(),
            content
        );

        self.index += 1;
        output
    }

    fn format_table_of_contents(&mut self, toc: &str) -> String {
        format!(
            r#"<table_of_contents>
{}
</table_of_contents>"#,
            toc
        )
    }

    fn start_output(&mut self) -> String {
        "<documents>".to_string()
    }

    fn end_output(&mut self) -> String {
        "</documents>".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_xml_format() {
        let mut formatter = XmlFormatter::new();
        let path = PathBuf::from("test.txt");
        let content = "Hello, world!";

        let result = formatter.format_file(&path, content, false);
        let expected = r#"<document index="1">
<source>test.txt</source>
<document_content>
Hello, world!
</document_content>
</document>"#;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_xml_format_multiple_files() {
        let mut formatter = XmlFormatter::new();
        let path1 = PathBuf::from("test1.txt");
        let path2 = PathBuf::from("test2.txt");

        let result1 = formatter.format_file(&path1, "content1", false);
        let result2 = formatter.format_file(&path2, "content2", false);

        assert!(result1.contains(r#"index="1""#));
        assert!(result2.contains(r#"index="2""#));
    }

    #[test]
    fn test_xml_format_with_line_numbers() {
        let mut formatter = XmlFormatter::new();
        let path = PathBuf::from("test.txt");
        let content = "line 1\nline 2";

        let result = formatter.format_file(&path, content, true);
        assert!(result.contains("1  line 1\n2  line 2"));
    }

    #[test]
    fn test_start_end_output() {
        let mut formatter = XmlFormatter::new();
        assert_eq!(formatter.start_output(), "<documents>");
        assert_eq!(formatter.end_output(), "</documents>");
    }
}
