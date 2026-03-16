# Contributing to LumenX

Thank you for your interest in contributing to LumenX!

## How to Contribute

1. Check existing issues for open tasks or feature requests
2. Fork the repository
3. Create a feature branch (`git checkout -b feature/amazing-feature`)
4. Make your changes
5. Write tests for new features
6. Ensure all tests pass (`cargo test`)
7. Run linters (`cargo clippy -- -D warnings`)
8. Submit a pull request

## Development Setup

```bash
# Clone your fork
git clone https://github.com/your-username/lumen.git
cd lumen

# Install development tools
cargo install cargo-watch cargo-hack

# Build the project
cargo build --release

# Run tests
cargo test

# Watch for changes during development
cargo watch -x check -x test -x run --bin lumenx
```

## Coding Standards

- Follow Rust best practices and idiomatic patterns
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting before committing
- Write tests for new features and bug fixes
- Update documentation as needed

## Project Structure

```
lumen/
├── crates/
│   ├── lumenx-cli/     # Command-line interface
│   ├── lumenx-core/    # Core types and utilities
│   ├── lumenx-detect/   # Framework detection
│   ├── lumenx-analyze/  # Code analysis engines
│   ├── lumenx-score/    # Quality scoring system
│   ├── lumenx-fix/      # Automatic fixes
│   ├── lumenx-report/   # Report generation
│   ├── lumenx-testgen/  # Test generation
│   ├── lumenx-history/  # Score history tracking
│   └── lumenx-diff/     # Git diff analysis
└── examples/            # Example projects
```

## Pull Request Process

- Describe your changes clearly in the PR description
- Reference related issues (e.g., "Fixes #123")
- Ensure all CI checks pass
- Add tests for new functionality
- Update documentation if needed

## Getting Help

- Open an issue for bugs or feature requests
- Join discussions in existing issues
- Check the documentation in `README.md`

Thank you for contributing to LumenX! 🚀
