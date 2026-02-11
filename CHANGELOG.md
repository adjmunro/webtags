# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- CI/CD pipeline with GitHub Actions
- Homebrew tap for easy installation on macOS
- Development tooling with justfile
- Worktree management scripts
- CLAUDE.md/AGENTS.md symlinking support
- Code coverage reporting
- Multi-platform release automation

## [0.1.0] - 2025-02-11

### Added
- Native messaging host in Rust
- Browser extension in TypeScript
- Git/GitHub integration
- OAuth Device Flow authentication
- JSON API v1.1 compliant bookmark storage
- Hierarchical tag support
- Encryption with macOS Touch ID
- Cross-browser support (Chrome, Firefox, Safari)
- Comprehensive test suite (45 unit tests)

### Security
- AES-256-GCM encryption for bookmarks
- macOS Keychain integration for secure key storage
- Touch ID biometric authentication
- Input validation and sanitization
- Path traversal prevention

[Unreleased]: https://github.com/adjmunro/webtags/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/adjmunro/webtags/releases/tag/v0.1.0
