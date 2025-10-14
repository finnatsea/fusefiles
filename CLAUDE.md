# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### Build
```bash
cargo build            # Debug build
cargo build --release  # Optimized release build
```

### Run
```bash
cargo run -- [args]                    # Run with cargo
./target/debug/files-to-prompt [args]  # Run debug binary directly
```

### Test
```bash
cargo test              # Run all tests
cargo test [test_name]  # Run specific test by name
cargo test --lib        # Run library tests only
cargo test --test integration_tests  # Run integration tests only
```

### Lint and Format
```bash
cargo fmt        # Format code
cargo clippy     # Lint with clippy
```

## Architecture Overview

This is a Rust implementation of the `files-to-prompt` CLI tool that concatenates files into a single prompt for use with LLMs. The codebase is organized around a modular architecture with clear separation of concerns.

### Core Components

1. **CLI Interface (`src/cli.rs`)**: Uses clap for argument parsing. Coordinates the entire application flow from parsing command-line arguments to outputting results.

2. **File Processing (`src/file_processor.rs`)**: Core logic for traversing directories and processing files. Handles filtering by extensions, hidden files, and ignore patterns.

3. **Output Formatters (`src/output/`)**: Trait-based system for different output formats:
   - `DefaultFormatter`: Plain text with `---` separators
   - `XmlFormatter`: Claude-specific XML format
   - `MarkdownFormatter`: Fenced code blocks with language detection

4. **Ignore System (`src/ignore.rs`)**: Handles both `.gitignore` files and custom ignore patterns using glob-style matching.

5. **Extensions (`src/extensions.rs`)**: Maps file extensions to programming languages for markdown syntax highlighting.

### Key Design Patterns

- **Trait-based Output**: The `OutputFormatter` trait allows easy extension of output formats
- **Error Handling**: Custom error types using `thiserror` for comprehensive error context
- **Iterator Usage**: Leverages Rust's iterator patterns for efficient file processing
- **Path Handling**: Consistent use of `PathBuf` and `Path` for cross-platform compatibility

### Testing Strategy

Integration tests in `tests/integration_tests.rs` cover all major features including:
- Basic file processing
- Hidden file handling
- Gitignore integration
- Multiple output formats
- Extension filtering
- Stdin input processing

When you change files, in general you should run: 
`cargo build` - to check that it builds
`cargo fmt` - to keep file formatting clean
`cargo clippy` - to lint
`cargo test` - to ensure everything works