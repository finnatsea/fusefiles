# fusefiles

[![Crates.io](https://img.shields.io/crates/v/fusefiles.svg)](https://crates.io/crates/fusefiles)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![CI](https://github.com/finnatsea/fusefiles/workflows/CI/badge.svg)](https://github.com/finnatsea/fusefiles/actions)

A CLI tool that concatenates files into a single prompt for use with LLMs.

## What does it do?

Both `fuse` and `fusefiles` work (use whichever you prefer!)
You run `fuse src/ --toc` and get:

```
Table of Contents
---
└── src/
    ├── cli.rs
    ├── main.rs
    ├── output/
    │   ├── default.rs
    │   ├── markdown.rs
    │   └── xml.rs
    ├── tree.rs
    └── utils.rs

---
src/cli.rs
---
use clap::Parser;
use std::fs::File;
... (contents of all files concatenated)
```

The tool generates a **table of contents tree** showing your project structure, followed by the concatenated contents of all files. Perfect for feeding to LLMs like Claude or ChatGPT!

## How to Use

### Installation

#### Install from crates.io

```bash
cargo install fusefiles
```

This installs both the `fuse` and `fusefiles` commands (they're aliases for the same tool).

#### Build and Install from Source

Clone the repository and install:

```bash
git clone https://github.com/finnatsea/fusefiles.git
cd fusefiles
cargo install --path .
```

This installs both `fuse` and `fusefiles` to `~/.cargo/bin/`, making them available system-wide.

#### Build Only (for development)

```bash
cargo build --release
```

The executable will be in `target/release/fuse`.


### How do I use it?

```bash
# Fuse all files in a directory
fuse src/

# Fuse multiple paths
fuse src/ tests/ Cargo.toml

# Only include Python and Rust files
fuse src/ -e py -e rs

# Output in markdown format with code blocks
fuse src/ --markdown

# Output in Claude XML format
fuse src/ --cxml

# Save output to a file
fuse src/ -o output.txt

# Exclude test files
fuse src/ --ignore "*test*"

# Include hidden files (git ignored files are ignored by default)
fuse . --include-hidden

# Add line numbers
fuse src/main.rs -n

# Add a table of contents tree
fuse src/ --toc

# Table of contents with directories only
fuse src/ --toc-dirs-only

# Table of contents with files and directories
fuse src/ --toc-files

# Ignore files only (not directories) with pattern
fuse src/ --ignore "*test*" --ignore-files-only

# Ignore .gitignore rules
fuse . --ignore-gitignore

# Pipe file paths from another command
find . -name "*.rs" | fuse

# Use with null-separated paths
find . -name "*.rs" -print0 | fuse --null
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
### Shoutout
Inspired by Simon Willison's [files-to-prompt](https://github.com/simonw/files-to-prompt) but faster and with more features, like including a file tree at the top of the single file.