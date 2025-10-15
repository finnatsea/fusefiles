# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.6.0] - 2025-10-14

### Added
- Initial open source release
- Concatenate files from directories into a single output for LLM prompts
- Support for multiple output formats (default, markdown, XML)
- File extension filtering
- Custom ignore patterns with glob support
- `.gitignore` integration for automatic file exclusion
- Hidden file handling
- Line numbering option
- Table of contents generation (auto, dirs-only, files-and-dirs modes)
- Stdin input support with newline or null-separated paths
- Cross-platform support (Linux, macOS, Windows)
- Comprehensive integration test suite

### Features
- **Output formats**: Plain text, Markdown with syntax highlighting, Claude XML
- **Filtering**: Extension-based filtering, custom ignore patterns, gitignore support
- **Tree generation**: Auto-detect mode, directory-only mode, full file tree mode
- **Input modes**: File paths, directory recursion, stdin piping
- **Configuration**: Hidden file inclusion, line numbers, ignore files vs directories

[0.6.0]: https://github.com/finnatsea/fusefiles/releases/tag/v0.6.0
