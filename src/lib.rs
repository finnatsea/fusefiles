//! files-to-prompt: Concatenate a directory full of files into a single prompt for use with LLMs
//!
//! This crate provides functionality to recursively process files and directories,
//! concatenating their contents with various output formats suitable for LLM prompts.

use std::path::PathBuf;
use thiserror::Error;

/// Main error type for the files-to-prompt application
#[derive(Debug, Error)]
pub enum FilesToPromptError {
    #[error("File not found: {path}")]
    FileNotFound { path: PathBuf },

    #[error("Permission denied: {path}")]
    PermissionDenied { path: PathBuf },

    #[error("Binary file detected: {path}")]
    BinaryFile { path: PathBuf },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Walk directory error: {0}")]
    WalkDir(#[from] walkdir::Error),

    #[error("UTF-8 decode error in file: {path}")]
    Utf8Error { path: PathBuf },

    #[error("Pattern matching error: {0}")]
    PatternError(String),
}

/// Result type alias for the files-to-prompt application
pub type Result<T> = std::result::Result<T, FilesToPromptError>;

/// Table of contents mode for tree generation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TocMode {
    /// Auto-detect based on estimated output size (files+dirs if <100 lines, dirs only if ≥100)
    Auto,
    /// Show directories only
    DirsOnly,
    /// Show files and directories
    FilesAndDirs,
}

// Public modules
pub mod cli;
pub mod extensions;
pub mod file_processor;
pub mod ignore;
pub mod output;
pub mod tree;
pub mod utils;

// Re-exports for convenience
pub use file_processor::FileProcessor;
pub use output::{DefaultFormatter, MarkdownFormatter, OutputFormatter, XmlFormatter};
pub use tree::{TreeGenerator, TreeNode};
