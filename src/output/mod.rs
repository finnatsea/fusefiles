//! Output formatting modules for different output formats

use std::path::Path;

/// Trait for different output formatters
pub trait OutputFormatter {
    /// Format a single file's content
    fn format_file(&mut self, path: &Path, content: &str, line_numbers: bool) -> String;
    
    /// Get the string to output at the beginning
    fn start_output(&mut self) -> String;
    
    /// Get the string to output at the end
    fn end_output(&mut self) -> String;
}

pub mod default;
pub mod xml;
pub mod markdown;

pub use default::DefaultFormatter;
pub use xml::XmlFormatter;
pub use markdown::MarkdownFormatter;