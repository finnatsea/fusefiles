//! Core file processing and directory traversal logic

use crate::ignore::CustomIgnore;
use crate::output::OutputFormatter;
use crate::tree::TreeGenerator;
use crate::{FilesToPromptError, Result, TocMode};
use ignore::WalkBuilder;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Handles file processing with filtering and directory traversal
pub struct FileProcessor {
    extensions: Vec<String>,
    include_hidden: bool,
    ignore_gitignore: bool,
    line_numbers: bool,
    toc_mode: Option<TocMode>,
    custom_ignore: CustomIgnore,
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
    ) -> Result<Self> {
        let custom_ignore = CustomIgnore::new(ignore_patterns, ignore_files_only)?;

        Ok(Self {
            extensions,
            include_hidden,
            ignore_gitignore,
            line_numbers,
            toc_mode,
            custom_ignore,
        })
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
                self.ignore_gitignore,
                self.custom_ignore.clone(),
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
        path: &Path,
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
                eprintln!("Warning: Skipping binary file {}", path.display());
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
        let walker = self.build_walker(dir_path)?;

        for result in walker {
            let entry = match result {
                Ok(entry) => entry,
                Err(err) => return Err(map_walk_error(err)),
            };

            let path = entry.path();
            if entry.depth() == 0 {
                continue;
            }

            let is_file = entry
                .file_type()
                .map(|ft| ft.is_file())
                .unwrap_or_else(|| path.is_file());
            if !is_file {
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

            // Check custom ignore patterns for files
            if self.custom_ignore.should_ignore_file(path) {
                continue;
            }

            // Process the file
            match self.read_file_content(path) {
                Ok(content) => {
                    let formatted = formatter.format_file(path, &content, self.line_numbers);
                    output.push(formatted);
                }
                Err(FilesToPromptError::BinaryFile { path }) => {
                    eprintln!("Warning: Skipping binary file {}", path.display());
                }
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }

    fn build_walker(&self, dir_path: &Path) -> Result<ignore::Walk> {
        let mut builder = WalkBuilder::new(dir_path);
        builder.sort_by_file_name(|a, b| a.cmp(b));
        builder.follow_links(false);
        if self.include_hidden {
            builder.hidden(false);
        }

        if self.ignore_gitignore {
            builder.git_ignore(false);
            builder.git_global(false);
            builder.git_exclude(false);
            builder.ignore(false);
            builder.parents(false);
        } else {
            builder.git_ignore(true);
            builder.git_global(true);
            builder.git_exclude(true);
            builder.ignore(true);
            builder.parents(true);
            builder.require_git(false);
        }

        let root = dir_path.to_path_buf();
        let custom_for_dirs = self.custom_ignore.clone();
        let include_hidden = self.include_hidden;
        builder.filter_entry(move |entry| {
            if entry.path() == root {
                return true;
            }

            let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);

            if !include_hidden
                && is_dir
                && entry
                    .path()
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| name.starts_with('.'))
                    .unwrap_or(false)
            {
                return false;
            }

            if is_dir && custom_for_dirs.should_ignore_dir(entry.path()) {
                return false;
            }

            true
        });

        Ok(builder.build())
    }

    /// Read file content and handle binary files
    fn read_file_content(&self, path: &Path) -> Result<String> {
        let bytes = fs::read(path)?;

        if Self::is_binary(&bytes) {
            return Err(FilesToPromptError::BinaryFile {
                path: path.to_path_buf(),
            });
        }

        match String::from_utf8(bytes) {
            Ok(content) => Ok(content),
            Err(_) => Err(FilesToPromptError::BinaryFile {
                path: path.to_path_buf(),
            }),
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

    fn is_binary(bytes: &[u8]) -> bool {
        const SAMPLE_SIZE: usize = 1024;
        let sample_len = bytes.len().min(SAMPLE_SIZE);

        if sample_len == 0 {
            return false;
        }

        let mut suspicious = 0;
        for &byte in &bytes[..sample_len] {
            if byte == 0 {
                return true;
            }

            if matches!(byte, 0x01..=0x08 | 0x0B | 0x0E..=0x1F) {
                suspicious += 1;
            }
        }

        suspicious * 100 / sample_len > 10
    }
}

fn map_walk_error(err: ignore::Error) -> FilesToPromptError {
    if let Some(io_err) = err.io_error() {
        FilesToPromptError::Io(io::Error::new(io_err.kind(), io_err.to_string()))
    } else {
        FilesToPromptError::Io(io::Error::other(err.to_string()))
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
        )
        .unwrap();

        assert!(processor.should_include_file_by_extension(&PathBuf::from("test.txt")));
        assert!(processor.should_include_file_by_extension(&PathBuf::from("test.py")));
        assert!(!processor.should_include_file_by_extension(&PathBuf::from("test.rs")));
    }

    #[test]
    fn test_should_include_file_no_extensions() {
        let processor =
            FileProcessor::new(vec![], false, false, false, vec![], false, None).unwrap();

        assert!(processor.should_include_file_by_extension(&PathBuf::from("test.txt")));
        assert!(processor.should_include_file_by_extension(&PathBuf::from("test.py")));
        assert!(processor.should_include_file_by_extension(&PathBuf::from("test.rs")));
    }

    #[test]
    fn test_is_hidden_file() {
        let processor =
            FileProcessor::new(vec![], false, false, false, vec![], false, None).unwrap();

        assert!(processor.is_hidden_file(&PathBuf::from(".hidden")));
        assert!(processor.is_hidden_file(&PathBuf::from(".gitignore")));
        assert!(!processor.is_hidden_file(&PathBuf::from("visible.txt")));
    }

    #[test]
    fn test_process_single_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "Hello, world!").unwrap();

        let processor =
            FileProcessor::new(vec![], false, false, false, vec![], false, None).unwrap();
        let mut formatter = DefaultFormatter::new();
        let mut output = Vec::new();

        processor
            .process_file(&file_path, &mut formatter, &mut output)
            .unwrap();

        assert_eq!(output.len(), 1);
        assert!(output[0].contains("test.txt"));
        assert!(output[0].contains("Hello, world!"));
    }
}
