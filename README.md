# files-to-prompt

A Rust CLI tool that concatenates files into a single prompt for use with LLMs.

## How to Use

### Installation

#### Install from crates.io

```bash
cargo install files-to-prompt
```

#### Build and Install from Source

Clone the repository and install:

```bash
git clone https://github.com/finnatsea/files-to-prompt-rs.git
cd files-to-prompt-rs
cargo install --path .
```

This installs the binary to `~/.cargo/bin/files-to-prompt`, making it available system-wide.

#### Build Only (for development)

```bash
cargo build --release
```

The executable will be in `target/release/files-to-prompt`.


### How do I use it?

```bash
# Process a directory
files-to-prompt src/

# Process multiple paths
files-to-prompt src/ tests/ Cargo.toml

# Only include Python and Rust files
files-to-prompt src/ -e py -e rs

# Output in markdown format with code blocks
files-to-prompt src/ --markdown

# Output in Claude XML format
files-to-prompt src/ --cxml

# Save output to a file
files-to-prompt src/ -o output.txt

# Exclude test files
files-to-prompt src/ --ignore "*test*"

# Include hidden files
files-to-prompt . --include-hidden

# Add line numbers
files-to-prompt src/main.rs -n

# Add a table of contents tree
files-to-prompt src/ --toc

# Table of contents with directories only
files-to-prompt src/ --toc-dirs-only

# Table of contents with files and directories
files-to-prompt src/ --toc-files

# Ignore files only (not directories) with pattern
files-to-prompt src/ --ignore "*test*" --ignore-files-only

# Ignore .gitignore rules
files-to-prompt . --ignore-gitignore

# Pipe file paths from another command
find . -name "*.rs" | files-to-prompt

# Use with null-separated paths
find . -name "*.rs" -print0 | files-to-prompt --null
```

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test by name
cargo test test_basic_functionality

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test integration_tests
```

### Linting and Formatting

```bash
# Format code
cargo fmt

# Lint with clippy
cargo clippy
```