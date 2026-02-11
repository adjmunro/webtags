# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1] - 2026-02-12

### Fixed
- Release workflow now uses fine-grained PAT for Homebrew formula updates
- Homebrew formula SHA256 now correctly calculated from source archive
- Release workflow properly scoped to prevent cross-repository permission issues

### Changed
- Automated Homebrew formula verification in CI pipeline
- Improved security with fine-grained tokens (single-repo access only)

### Added
- Comprehensive Homebrew token setup documentation
- Alternative workflow examples (deploy key, PR-based)

## [0.1.0] - 2026-02-11

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
