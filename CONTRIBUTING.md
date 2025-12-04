# Contributing to FastHooks

Thank you for your interest in contributing to FastHooks! This document provides guidelines and information for contributors.

## Code of Conduct

By participating in this project, you agree to maintain a respectful and inclusive environment for everyone.

## How to Contribute

### Reporting Bugs

Before creating a bug report, please check existing issues to avoid duplicates.

**When reporting a bug, include:**

1. FastHooks version (`fasthooks --version`)
2. Operating system and version
3. Rust version if building from source
4. Your `fasthooks.toml` configuration (sanitized)
5. Complete error message
6. Steps to reproduce

### Suggesting Features

Feature requests are welcome! Please open an issue with:

1. Clear description of the feature
2. Use case and motivation
3. Example of how it would work
4. Any alternatives you've considered

### Pull Requests

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests (`cargo test`)
5. Run lints (`cargo clippy`)
6. Format code (`cargo fmt`)
7. Commit your changes
8. Push to your fork
9. Open a Pull Request

## Development Setup

### Prerequisites

- Rust 1.70 or later
- Git

### Building

```bash
# Clone your fork
git clone https://github.com/alfredo-baratta/fasthooks.git
cd fasthooks

# Build
cargo build

# Run tests
cargo test

# Run with debug output
RUST_LOG=debug cargo run -- run pre-commit
```

### Project Structure

```
fasthooks/
├── src/
│   ├── main.rs           # Entry point
│   ├── cli.rs            # CLI argument parsing
│   ├── commands/         # Command implementations
│   │   ├── init.rs
│   │   ├── install.rs
│   │   ├── run.rs
│   │   └── ...
│   ├── config/           # Configuration parsing
│   │   ├── mod.rs
│   │   ├── schema.rs
│   │   └── parser.rs
│   ├── hooks/            # Git hook management
│   │   ├── mod.rs
│   │   ├── git.rs
│   │   ├── installer.rs
│   │   └── template.rs
│   ├── runner/           # Task execution
│   │   ├── mod.rs
│   │   ├── executor.rs
│   │   └── stats.rs
│   └── utils/            # Utility functions
├── docs/                 # Documentation
├── tests/                # Integration tests
├── benches/              # Benchmarks
└── .github/              # GitHub Actions workflows
```

## Coding Standards

### Rust Style

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Write documentation for public APIs

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add parallel execution support
fix: handle empty glob patterns
docs: update configuration reference
refactor: simplify task executor
test: add integration tests for migration
chore: update dependencies
```

### Testing

- Write tests for new functionality
- Ensure all tests pass before submitting PR
- Include both unit tests and integration tests where appropriate

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

### Documentation

- Update documentation for user-facing changes
- Include doc comments for public APIs
- Update README.md if adding new features

## Areas for Contribution

### Good First Issues

Look for issues labeled `good first issue` for beginner-friendly tasks:

- Documentation improvements
- Error message enhancements
- Small bug fixes
- Test coverage improvements

### Help Wanted

Issues labeled `help wanted` are areas where we especially need community input:

- Platform-specific fixes (Windows, macOS, Linux)
- Performance optimizations
- New features

### Current Priorities

1. **Windows support** - Ensure full compatibility
2. **Performance** - Benchmark and optimize
3. **Documentation** - Improve getting started guides
4. **Testing** - Increase test coverage

## Release Process

Releases are automated via GitHub Actions when a version tag is pushed:

1. Update version in `Cargo.toml`
2. Update CHANGELOG.md
3. Create and push a tag: `git tag v0.1.0 && git push --tags`
4. GitHub Actions builds and publishes to crates.io

## Getting Help

- Open an issue for questions
- Join discussions in existing issues
- Check documentation in `/docs`

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
