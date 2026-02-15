# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Smart stdin mode: prompts are automatically piped via stdin when they exceed 800 characters or contain shell-special characters (cross-platform: covers both Unix and Windows cmd.exe metacharacters)
- `force_stdin` tool parameter to force stdin mode regardless of prompt content
- `timeout_secs` tool parameter for configuring codex execution timeout
- Regression test for stdin early-exit handling (`test_force_stdin_early_exit_returns_structured_result`)
- Comprehensive test suite (25 tests total)
  - 10 unit tests for prompt escaping and Options validation
  - 10 integration tests for end-to-end scenarios
  - 5 server tests for MCP protocol implementation
- Test utilities in tests/common/mod.rs
- TESTING.md documentation with testing guide
- Codecov integration for coverage reporting (.codecov.yml)
- Enhanced CI workflow with:
  - Code coverage reporting via cargo-tarpaulin
  - Security audits via cargo-audit
  - Benchmarking support
  - Multi-platform testing (Ubuntu, macOS, Windows)
  - Multiple Rust versions (stable, beta)
- Makefile commands for testing (test, test-lib, test-integration, coverage)

### Fixed
- Clippy warnings for needless borrows removed
- Test assertions updated to match actual Implementation::from_build_env() behavior

## [0.1.0] - 2025-01-28

### Added
- Initial release of codex-mcp-rs
- MCP server implementation using official Rust SDK (rmcp)
- Codex CLI wrapper with JSON output parsing
- Session management for multi-turn conversations
- Configurable sandbox policies (read-only, workspace-write, danger-full-access)
- Image attachment support
- Async I/O with Tokio runtime
- NPM package with automatic binary downloads
- Cross-platform support (Linux, macOS, Windows × x86_64, arm64)
- GitHub Actions CI/CD workflows
- Comprehensive documentation (README, CLAUDE.md, CONTRIBUTING.md, QUICKSTART.md)
- MIT License

### Features
- **Tool**: `codex` - Execute Codex CLI for AI-assisted coding tasks
  - Required parameters: `PROMPT`, `cd`
  - Optional parameters: `sandbox`, `SESSION_ID`, `skip_git_repo_check`, `return_all_messages`, `image`, `model`, `yolo`, `profile`
- **Transport**: stdio (standard input/output)
- **Error handling**: Comprehensive validation and error messages
- **Performance**: High-performance Rust implementation with low memory footprint

### Documentation
- Installation guides (npm, binary, source)
- Usage examples and common use cases
- Architecture documentation for developers
- Contribution guidelines
- Quick start guide

### Infrastructure
- Automated multi-platform builds
- NPM package publishing
- MCP registry integration
- Continuous Integration testing
- Makefile for development convenience

[Unreleased]: https://github.com/missdeer/codex-mcp-rs/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/missdeer/codex-mcp-rs/releases/tag/v0.1.0
