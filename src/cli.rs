//! Command-line interface implementation using clap

use clap::Parser;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use crate::output::{DefaultFormatter, MarkdownFormatter, XmlFormatter};
use crate::utils::read_paths_from_stdin;
use crate::{FileProcessor, Result};

// ============================================================================
// Shared documentation pieces (single source of truth)
// ============================================================================

const DESCRIPTION: &str = "Turn many files -> single file, useful for LLM prompting.";

const USAGE: &str = "\
Usage:
  fuse [path/to/file_or_directory] [options]
  fuse [file1] [file2] [folder1] [folder2] [options]";

const EXAMPLES: &str = r#"Here's a few samples to get started:
  fuse src/                                      # All files in src/
  fuse src/ test/ -e ts                          # Only .ts files in src/ and test
  fuse src/ --toc-files --ignore "__tests__"     # Files in src/ except __tests__, with toc tree
  fuse . --ignore "*.log" --ignore "test_*"      # Skip logs and files that start with "test_"
  fuse . -o output.txt                           # Save to file instead of printing or use >"#;

const OPTIONS_HELP: &str = "\
OPTIONS
Input Control:
  -e, --extension <EXT>     Only include these extensions (e.g. -e py -e js)
      --include-hidden      Include hidden files (starting with .)
      --ignore-files-only   Make --ignore patterns skip files only, not directories
      --ignore-gitignore    Don't use .gitignore rules
      --ignore <PATTERN>    Skip files matching pattern (*.log, test_*, *foo*, __pycache__)

Output Format:
  -c, --cxml               Output in Claude XML format
  -m, --markdown           Output as Markdown code blocks
  -n, --line-numbers       Add line numbers
  -o, --output <FILE>      Save to file instead of printing
      --toc                Include table of contents tree (auto: files+dirs if <100 lines, dirs only if ≥100)
      --toc-dirs-only      Table of contents shows directories only
      --toc-files          Table of contents shows files and directories

Other:
  -0, --null               Read null-separated paths from stdin
  -h, --help               Print help
  -V, --version            Print version";

const PATTERN_USAGE: &str = r#"Pattern Usage:
  --ignore "test_*"        → Matches: test_utils.py, test_data.json
  --ignore "*.log"         → Matches: debug.log, error.log
  --ignore "*foo*"         → Matches: foo.txt, config_foo_bar.xml
  --ignore "__init__.py"   → Matches: any file/folder named exactly "__init__.py""#;

// ============================================================================
// CLI definition
// ============================================================================

#[derive(Parser)]
#[command(name = "fuse")]
#[command(about = DESCRIPTION)]
#[command(version)]
#[command(disable_help_flag = true)]
#[command(disable_version_flag = true)]
pub struct Cli {
    /// Files or directories to include
    #[arg(value_name = "PATHS")]
    pub paths: Vec<PathBuf>,

    /// Show full help message
    #[arg(long = "help", short = 'h')]
    pub help: bool,

    // Input Control
    /// Only include these extensions (e.g. -e py -e js)
    #[arg(short = 'e', long = "extension", action = clap::ArgAction::Append, value_name = "EXT", help_heading = "Input Control")]
    pub extensions: Vec<String>,

    /// Include hidden files (starting with .)
    #[arg(long = "include-hidden", help_heading = "Input Control")]
    pub include_hidden: bool,

    /// Make --ignore patterns skip files only, not directories
    #[arg(long = "ignore-files-only", help_heading = "Input Control")]
    pub ignore_files_only: bool,

    /// Don't use .gitignore rules
    #[arg(long = "ignore-gitignore", help_heading = "Input Control")]
    pub ignore_gitignore: bool,

    /// Skip files matching pattern (*.log, test_*, *foo*, __pycache__)
    #[arg(long = "ignore", action = clap::ArgAction::Append, value_name = "PATTERN", help_heading = "Input Control")]
    pub ignore_patterns: Vec<String>,

    // Output Format
    /// Output in Claude XML format
    #[arg(short = 'c', long = "cxml", help_heading = "Output Format")]
    pub claude_xml: bool,

    /// Output as Markdown code blocks
    #[arg(short = 'm', long = "markdown", help_heading = "Output Format")]
    pub markdown: bool,

    /// Add line numbers
    #[arg(short = 'n', long = "line-numbers", help_heading = "Output Format")]
    pub line_numbers: bool,

    /// Save to file instead of printing
    #[arg(
        short = 'o',
        long = "output",
        value_name = "FILE",
        help_heading = "Output Format"
    )]
    pub output_file: Option<PathBuf>,

    /// Include table of contents tree (auto: files+dirs if <100 lines, dirs only if ≥100)
    #[arg(long = "toc", help_heading = "Output Format")]
    pub table_of_contents: bool,

    /// Table of contents shows directories only
    #[arg(long = "toc-dirs-only", help_heading = "Output Format")]
    pub toc_dirs_only: bool,

    /// Table of contents shows files and directories
    #[arg(long = "toc-files", help_heading = "Output Format")]
    pub toc_files: bool,

    // Other
    /// Read null-separated paths from stdin
    #[arg(short = '0', long = "null", help_heading = "Other")]
    pub null_separator: bool,

    /// Print version
    #[arg(short = 'V', long = "version", action = clap::ArgAction::Version, help_heading = "Other")]
    pub version: Option<bool>,
}

fn print_short_help() {
    println!("{DESCRIPTION}\n\n{USAGE}\n\n{EXAMPLES}\n\nFor a full list of options, run `fuse --help`.");
}

fn print_full_help() {
    println!("{DESCRIPTION}\n\n{USAGE}\n\n{EXAMPLES}\n\n{OPTIONS_HELP}\n\n{PATTERN_USAGE}");
}

/// Main entry point for the CLI application
pub fn run() -> Result<()> {
    let raw_args: Vec<String> = std::env::args().collect();

    // Handle special cases before parsing
    if raw_args.len() == 1 {
        // No arguments provided, show short help
        print_short_help();
        return Ok(());
    }

    // Check for help argument
    if raw_args
        .iter()
        .any(|arg| arg == "help" || arg == "--help" || arg == "-h")
    {
        print_full_help();
        return Ok(());
    }

    // Check for version argument
    if raw_args.iter().any(|arg| arg == "--version" || arg == "-V") {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let args = Cli::parse();

    // Combine paths from arguments and stdin
    let mut all_paths = args.paths.clone();

    // Only read from stdin if no paths were provided via command line
    // This prevents stdin from being read when paths are already specified
    if all_paths.is_empty() {
        let stdin_paths = read_paths_from_stdin(args.null_separator)?;
        for path in stdin_paths {
            all_paths.push(PathBuf::from(path));
        }
    }

    // Validate that we have at least one path
    if all_paths.is_empty() {
        print_short_help();
        std::process::exit(1);
    }

    // Validate that all paths exist
    for path in &all_paths {
        if !path.exists() {
            eprintln!("Path does not exist: {}", path.display());
            std::process::exit(1);
        }
    }

    // Validate table of contents flags
    if args.toc_dirs_only && args.toc_files {
        eprintln!("Error: Cannot specify both --toc-dirs-only and --toc-files");
        std::process::exit(1);
    }

    // Determine table of contents mode
    let toc_mode = if args.table_of_contents || args.toc_dirs_only || args.toc_files {
        if args.toc_files {
            Some(crate::TocMode::FilesAndDirs)
        } else if args.toc_dirs_only {
            Some(crate::TocMode::DirsOnly)
        } else {
            Some(crate::TocMode::Auto)
        }
    } else {
        None
    };

    // Create file processor
    let processor = FileProcessor::new(
        args.extensions,
        args.include_hidden,
        args.ignore_files_only,
        args.ignore_gitignore,
        args.ignore_patterns,
        args.line_numbers,
        toc_mode,
    )?;

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
