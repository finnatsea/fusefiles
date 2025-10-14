//! Output formatting modules for different output formats

use std::path::Path;

/// Trait for different output formatters
pub trait OutputFormatter {
    /// Format a single file's content
    fn format_file(&mut self, path: &Path, content: &str, line_numbers: bool) -> String;

    /// Format the table of contents tree
    fn format_table_of_contents(&mut self, toc: &str) -> String;

    /// Get the string to output at the beginning
    fn start_output(&mut self) -> String;

    /// Get the string to output at the end
    fn end_output(&mut self) -> String;
}

pub mod default;
pub mod markdown;
pub mod xml;

pub use default::DefaultFormatter;
pub use markdown::MarkdownFormatter;
pub use xml::XmlFormatter;
