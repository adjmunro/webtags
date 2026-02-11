# WebTags - AI Agent Instructions

**WebTags** is a Git-synced browser bookmark tagging extension with a Rust native messaging host and TypeScript browser extension.

## Project Essentials

- **Package Manager**: `npm` (extension), `cargo` (native host)
- **Build Commands**:
  - Rust: `cd native-host && cargo build --release`
  - Extension: `cd extension && npm run build`
  - CI checks: `just ci` (requires [just](https://github.com/casey/just))
- **Test Commands**:
  - Rust: `cd native-host && cargo test`
  - Extension: `cd extension && npm test`

## Key Conventions

- **Language**: Rust for native host, TypeScript for extension
- **Formatting**: `cargo fmt` for Rust, Prettier/ESLint for TypeScript
- **Commits**: Follow conventional commits format
- **Tests**: TDD approach - write tests first

## Documentation Structure

Detailed instructions organized by topic:

- **[Development Workflow](docs/workflows/DEVELOPMENT.md)** - TDD process, building, debugging
- **[Git Workflow](docs/workflows/GIT.md)** - Branching, commits, CI/CD
- **[Testing Strategy](docs/workflows/TESTING.md)** - Unit tests, integration tests, manual testing
- **[Architecture](docs/architecture/OVERVIEW.md)** - System design, module structure
- **[Security](docs/security/)** - Security audits, encryption design

## Quick Reference

### File Organization
```
extension/          # TypeScript browser extension
native-host/        # Rust native messaging host
schemas/            # JSON API v1.1 schemas
docs/               # Documentation
```

### Running the Project
```bash
# Build everything
just build

# Run all tests
just test

# Run CI checks locally
just ci
```

## For AI Agents

When working on this project:
1. **Always read relevant docs** before making changes
2. **Run tests** after code modifications
3. **Follow existing patterns** in the codebase
4. **Ask for clarification** on architectural decisions before major changes
