# files-to-prompt

A Rust CLI tool that concatenates files into a single prompt for use with LLMs.

## How to Use

### How do I build it?

```bash
cargo build --release
```
(the executable will be in `target/release/files-to-prompt`)

/Users/finnianbrown/Developer/kilo-projects/files-to-prompt-rs/target/release/files-to-prompt relab/ > relab-code.txt

or install it via cargo:

```bash
cargo install files-to-prompt
```


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

# Pipe file paths from another command
find . -name "*.rs" | files-to-prompt

# Use with null-separated paths
find . -name "*.rs" -print0 | files-to-prompt --null
```