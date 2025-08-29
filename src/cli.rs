//! Command-line interface implementation using clap

use clap::Parser;
use std::path::PathBuf;
use std::fs::File;
use std::io::Write;

use crate::{Result, FileProcessor};
use crate::output::{DefaultFormatter, XmlFormatter, MarkdownFormatter};
use crate::utils::read_paths_from_stdin;

/// files-to-prompt: Concatenate a directory full of files into a single prompt for use with LLMs
#[derive(Parser)]
#[command(name = "files-to-prompt")]
#[command(about = "Concatenate a directory full of files into a single prompt for use with LLMs")]
#[command(version)]
pub struct Cli {
    /// Paths to files or directories
    #[arg(value_name = "PATHS")]
    pub paths: Vec<PathBuf>,
    
    /// Only include files with specified extensions
    #[arg(short = 'e', long = "extension", action = clap::ArgAction::Append)]
    pub extensions: Vec<String>,
    
    /// Include files and folders starting with .
    #[arg(long = "include-hidden")]
    pub include_hidden: bool,
    
    /// --ignore option only ignores files
    #[arg(long = "ignore-files-only")]
    pub ignore_files_only: bool,
    
    /// Ignore .gitignore files and include all files
    #[arg(long = "ignore-gitignore")]
    pub ignore_gitignore: bool,
    
    /// List of patterns to ignore
    #[arg(long = "ignore", action = clap::ArgAction::Append)]
    pub ignore_patterns: Vec<String>,
    
    /// Output to a file instead of stdout
    #[arg(short = 'o', long = "output")]
    pub output_file: Option<PathBuf>,
    
    /// Output in XML-ish format suitable for Claude
    #[arg(short = 'c', long = "cxml")]
    pub claude_xml: bool,
    
    /// Output Markdown with fenced code blocks
    #[arg(short = 'm', long = "markdown")]
    pub markdown: bool,
    
    /// Add line numbers to the output
    #[arg(short = 'n', long = "line-numbers")]
    pub line_numbers: bool,
    
    /// Use NUL character as separator when reading from stdin
    #[arg(short = '0', long = "null")]
    pub null_separator: bool,
}

/// Main entry point for the CLI application
pub fn run() -> Result<()> {
    let args = Cli::parse();
    
    // Read paths from stdin if available
    let stdin_paths = read_paths_from_stdin(args.null_separator)?;
    
    // Combine paths from arguments and stdin
    let mut all_paths = args.paths.clone();
    for path in stdin_paths {
        all_paths.push(PathBuf::from(path));
    }
    
    // Validate that we have at least one path
    if all_paths.is_empty() {
        eprintln!("No paths provided. Please specify files or directories to process.");
        std::process::exit(1);
    }
    
    // Validate that all paths exist
    for path in &all_paths {
        if !path.exists() {
            eprintln!("Path does not exist: {}", path.display());
            std::process::exit(1);
        }
    }
    
    // Create file processor
    let processor = FileProcessor::new(
        args.extensions,
        args.include_hidden,
        args.ignore_files_only,
        args.ignore_gitignore,
        args.ignore_patterns,
        args.line_numbers,
    );
    
    // Determine output format and process files
    let output = if args.claude_xml {
        let mut formatter = XmlFormatter::new();
        processor.process_paths(&all_paths, &mut formatter)?
    } else if args.markdown {
        let mut formatter = MarkdownFormatter::new();
        processor.process_paths(&all_paths, &mut formatter)?
    } else {
        let mut formatter = DefaultFormatter::new();
        processor.process_paths(&all_paths, &mut formatter)?
    };
    
    // Write output
    if let Some(output_path) = args.output_file {
        let mut file = File::create(output_path)?;
        file.write_all(output.as_bytes())?;
    } else {
        print!("{}", output);
    }
    
    Ok(())
}