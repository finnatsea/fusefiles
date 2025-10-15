# Contributing to fusefiles

Thanks for your interest in contributing to fusefiles! This document provides guidelines for contributing to the project.

## Getting Started

### Prerequisites

- Rust (latest stable version)
- Git

### Development Setup

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/fusefiles.git
   cd fusefiles
   ```
3. Build the project:
   ```bash
   cargo build
   ```
4. Run tests to make sure everything works:
   ```bash
   cargo test
   ```

## Development Workflow

### Running the Project

```bash
# Run the CLI tool
cargo run -- [args]

# Example: process a directory
cargo run -- src/

# Run with release optimizations
cargo build --release
./target/release/fuse src/
```

### Testing

Before submitting a pull request, make sure all tests pass:

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test integration_tests
```

### Code Quality

We use standard Rust tooling to maintain code quality:

```bash
# Format code (required before submitting PR)
cargo fmt

# Run clippy linter (should have no warnings)
cargo clippy

# Run both checks
cargo fmt && cargo clippy
```

## Pull Request Process

1. **Create a feature branch** from `master`:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes** with clear, focused commits:
   - Write descriptive commit messages
   - Keep commits atomic (one logical change per commit)
   - Follow existing code style

3. **Test your changes**:
   ```bash
   cargo test
   cargo fmt
   cargo clippy
   ```

4. **Push to your fork**:
   ```bash
   git push origin feature/your-feature-name
   ```

5. **Submit a pull request**:
   - Provide a clear description of what your PR does
   - Reference any related issues
   - Ensure CI checks pass

## Code Style

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for consistent formatting
- Address all `cargo clippy` warnings
- Write doc comments for public APIs
- Add tests for new functionality

## Reporting Bugs

If you find a bug, please open an issue with:

- A clear, descriptive title
- Steps to reproduce the issue
- Expected behavior vs. actual behavior
- Your environment (OS, Rust version, etc.)
- Minimal code example if applicable

## Suggesting Enhancements

Enhancement suggestions are welcome! Please open an issue with:

- A clear description of the enhancement
- Use cases and benefits
- Any potential implementation details you've considered

## Questions?

Feel free to open an issue for any questions about contributing.

## License

By contributing to fusefiles, you agree that your contributions will be licensed under the Apache 2.0 License.
