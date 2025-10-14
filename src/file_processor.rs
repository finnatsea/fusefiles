//! Core file processing and directory traversal logic

use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;
use crate::{Result, FilesToPromptError, TocMode};
use crate::ignore::IgnoreChecker;
use crate::output::OutputFormatter;
use crate::tree::TreeGenerator;

/// Handles file processing with filtering and directory traversal
pub struct FileProcessor {
    extensions: Vec<String>,
    include_hidden: bool,
    ignore_files_only: bool,
    ignore_gitignore: bool,
    ignore_patterns: Vec<String>,
    line_numbers: bool,
    toc_mode: Option<TocMode>,
}

impl FileProcessor {
    /// Create a new FileProcessor with the specified options
    pub fn new(
        extensions: Vec<String>,
        include_hidden: bool,
        ignore_files_only: bool,
        ignore_gitignore: bool,
        ignore_patterns: Vec<String>,
        line_numbers: bool,
        toc_mode: Option<TocMode>,
    ) -> Self {
        Self {
            extensions,
            include_hidden,
            ignore_files_only,
            ignore_gitignore,
            ignore_patterns,
            line_numbers,
            toc_mode,
        }
    }

    /// Process multiple paths and generate output using the specified formatter
    pub fn process_paths<F: OutputFormatter>(
        &self,
        paths: &[PathBuf],
        formatter: &mut F,
    ) -> Result<String> {
        let mut output = Vec::new();
        
        // Add start output
        let start = formatter.start_output();
        if !start.is_empty() {
            output.push(start);
        }

        // Generate and add table of contents if requested
        if let Some(toc_mode) = self.toc_mode {
            let tree_generator = TreeGenerator::new(
                self.extensions.clone(),
                self.include_hidden,
                self.ignore_files_only,
                self.ignore_gitignore,
                self.ignore_patterns.clone(),
            );
            
            let trees = tree_generator.generate_tree(paths)?;
            let toc = tree_generator.render_tree(&trees, toc_mode);
            
            if !toc.is_empty() {
                let formatted_toc = formatter.format_table_of_contents(&toc);
                output.push(formatted_toc);
                output.push(String::new()); // Add blank line after TOC
            }
        }

        // Process each path
        for path in paths {
            self.process_single_path(path, formatter, &mut output)?;
        }

        // Add end output
        let end = formatter.end_output();
        if !end.is_empty() {
            output.push(end);
        }

        Ok(output.join("\n"))
    }

    /// Process a single path (file or directory)
    fn process_single_path<F: OutputFormatter>(
        &self,
        path: &PathBuf,
        formatter: &mut F,
        output: &mut Vec<String>,
    ) -> Result<()> {
        if path.is_file() {
            self.process_file(path, formatter, output)?;
        } else if path.is_dir() {
            self.process_directory(path, formatter, output)?;
        }
        Ok(())
    }

    /// Process a single file
    fn process_file<F: OutputFormatter>(
        &self,
        file_path: &Path,
        formatter: &mut F,
        output: &mut Vec<String>,
    ) -> Result<()> {
        // Check if file should be included based on extension
        if !self.should_include_file_by_extension(file_path) {
            return Ok(());
        }

        // Check if file is hidden and should be excluded
        if !self.include_hidden && self.is_hidden_file(file_path) {
            return Ok(());
        }

        match self.read_file_content(file_path) {
            Ok(content) => {
                let formatted = formatter.format_file(file_path, &content, self.line_numbers);
                output.push(formatted);
            }
            Err(FilesToPromptError::BinaryFile { path }) => {
                eprintln!("Warning: Skipping file {} due to UnicodeDecodeError", path.display());
            }
            Err(e) => return Err(e),
        }

        Ok(())
    }

    /// Process a directory recursively
    fn process_directory<F: OutputFormatter>(
        &self,
        dir_path: &Path,
        formatter: &mut F,
        output: &mut Vec<String>,
    ) -> Result<()> {
        let mut ignore_checker = IgnoreChecker::new(self.ignore_files_only);
        
        // Add custom ignore patterns
        ignore_checker.add_custom_patterns(&self.ignore_patterns)?;

        // Add gitignore patterns from the base directory if not ignoring them
        if !self.ignore_gitignore {
            ignore_checker.add_gitignore_file(&dir_path.join(".gitignore"))?;
        }

        // Walk the directory tree
        let walker = WalkDir::new(dir_path)
            .sort_by_file_name()
            .into_iter()
            .filter_entry(|e| {
                // Filter out hidden directories if not including hidden files
                if !self.include_hidden && self.is_hidden_path(e.path()) {
                    return false;
                }

                true
            });

        for entry in walker {
            let entry = entry?;
            let path = entry.path();

            // Skip the root directory itself
            if path == dir_path {
                continue;
            }

            // Only process files
            if !path.is_file() {
                continue;
            }

            // Check if file should be included based on extension
            if !self.should_include_file_by_extension(path) {
                continue;
            }

            // Check if file is hidden and should be excluded
            if !self.include_hidden && self.is_hidden_file(path) {
                continue;
            }

            // Check gitignore rules if not ignoring them
            if !self.ignore_gitignore && ignore_checker.should_ignore_gitignore(path) {
                continue;
            }

            // Check custom ignore patterns for files
            if ignore_checker.should_ignore_custom(path, true) {
                continue;
            }

            // Process the file
            match self.read_file_content(path) {
                Ok(content) => {
                    let formatted = formatter.format_file(path, &content, self.line_numbers);
                    output.push(formatted);
                }
                Err(FilesToPromptError::BinaryFile { path }) => {
                    eprintln!("Warning: Skipping file {} due to UnicodeDecodeError", path.display());
                }
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }

    /// Read file content and handle binary files
    fn read_file_content(&self, path: &Path) -> Result<String> {
        match fs::read_to_string(path) {
            Ok(content) => Ok(content),
            Err(e) => {
                // Check if it's a UTF-8 decode error (likely binary file)
                if e.kind() == std::io::ErrorKind::InvalidData {
                    Err(FilesToPromptError::BinaryFile {
                        path: path.to_path_buf(),
                    })
                } else {
                    Err(FilesToPromptError::Io(e))
                }
            }
        }
    }

    /// Check if a file should be included based on its extension
    fn should_include_file_by_extension(&self, path: &Path) -> bool {
        if self.extensions.is_empty() {
            return true;
        }

        if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
            // Check if file ends with any of the specified extensions
            self.extensions.iter().any(|ext| {
                // Handle extensions with or without leading dot
                let ext = ext.strip_prefix('.').unwrap_or(ext);
                extension == ext
            })
        } else {
            false
        }
    }

    /// Check if a file is hidden (starts with '.')
    fn is_hidden_file(&self, path: &Path) -> bool {
        path.file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.starts_with('.'))
            .unwrap_or(false)
    }

    /// Check if any component of a path is hidden
    fn is_hidden_path(&self, path: &Path) -> bool {
        path.components().any(|component| {
            if let Some(name) = component.as_os_str().to_str() {
                name.starts_with('.') && name != "." && name != ".."
            } else {
                false
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::DefaultFormatter;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_should_include_file_by_extension() {
        let processor = FileProcessor::new(
            vec!["txt".to_string(), "py".to_string()],
            false,
            false,
            false,
            vec![],
            false,
            None,
        );

        assert!(processor.should_include_file_by_extension(&PathBuf::from("test.txt")));
        assert!(processor.should_include_file_by_extension(&PathBuf::from("test.py")));
        assert!(!processor.should_include_file_by_extension(&PathBuf::from("test.rs")));
    }

    #[test]
    fn test_should_include_file_no_extensions() {
        let processor = FileProcessor::new(
            vec![],
            false,
            false,
            false,
            vec![],
            false,
            None,
        );

        assert!(processor.should_include_file_by_extension(&PathBuf::from("test.txt")));
        assert!(processor.should_include_file_by_extension(&PathBuf::from("test.py")));
        assert!(processor.should_include_file_by_extension(&PathBuf::from("test.rs")));
    }

    #[test]
    fn test_is_hidden_file() {
        let processor = FileProcessor::new(vec![], false, false, false, vec![], false, None);

        assert!(processor.is_hidden_file(&PathBuf::from(".hidden")));
        assert!(processor.is_hidden_file(&PathBuf::from(".gitignore")));
        assert!(!processor.is_hidden_file(&PathBuf::from("visible.txt")));
    }

    #[test]
    fn test_process_single_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "Hello, world!").unwrap();

        let processor = FileProcessor::new(vec![], false, false, false, vec![], false, None);
        let mut formatter = DefaultFormatter::new();
        let mut output = Vec::new();

        processor.process_file(&file_path, &mut formatter, &mut output).unwrap();

        assert_eq!(output.len(), 1);
        assert!(output[0].contains("test.txt"));
        assert!(output[0].contains("Hello, world!"));
    }
}