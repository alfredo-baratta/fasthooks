# Changelog

All notable changes to FastHooks will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release
- Core hook management (install, uninstall, run)
- TOML configuration support
- Parallel task execution
- Glob pattern matching for staged files
- Husky + lint-staged migration tool
- Performance benchmarking command
- Carbon savings estimation
- Support for all standard Git hooks

### Commands
- `fasthooks init` - Initialize in repository
- `fasthooks install` - Install Git hooks
- `fasthooks uninstall` - Remove Git hooks
- `fasthooks run <hook>` - Run hook manually
- `fasthooks add <hook> <cmd>` - Add command to hook
- `fasthooks list` - List configured hooks
- `fasthooks migrate` - Migrate from Husky
- `fasthooks benchmark` - Performance comparison

## [0.1.0] - 2025-12-04

### Added
- First public release
- 27x faster startup than Husky (12ms vs 1400ms)
- Parallel task execution with ~4.5s savings per commit
- Carbon savings tracking (~0.11g CO₂ per commit)

[Unreleased]: https://github.com/alfredo-baratta/fasthooks/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/alfredo-baratta/fasthooks/releases/tag/v0.1.0
