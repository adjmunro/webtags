# Git Workflow

## Branching Strategy

- **main/master**: Production-ready code
- **feature/***: New features
- **fix/***: Bug fixes
- **docs/***: Documentation updates

## Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

### Types
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `style`: Formatting, missing semicolons
- `refactor`: Code restructuring
- `test`: Adding tests
- `chore`: Maintenance tasks

### Examples
```
feat(extension): add tag autocomplete to popup
fix(git): handle merge conflicts in pull operation
docs(readme): update installation instructions
test(storage): add test for circular tag references
```

## CI/CD Pipeline

### Continuous Integration (on every push)

1. **Format Check** - `cargo fmt -- --check`
2. **Clippy Lints** - `cargo clippy -- -D warnings`
3. **Tests** - `cargo test` on Ubuntu and macOS
4. **Build** - Compile native host and extension
5. **Coverage** - Generate coverage reports

All checks must pass before merging.

### Release Process

When pushing a version tag (e.g., `v0.2.0`):

1. **Create Release** - Automated GitHub release
2. **Build Binaries** - Multi-platform compilation
3. **Upload Assets** - Binaries + checksums
4. **Update Homebrew** - Auto-update formula

## Local CI Simulation

Before pushing:

```bash
# Run all CI checks
just ci

# Individual checks
just fmt-check
just clippy
just test
just build
```

## Pre-commit Hooks

Not currently configured, but recommended to add:
- Format checking
- Linting
- Test execution

## Merge Strategy

- **Squash and merge** for feature branches
- **Rebase** for keeping history clean
- **No force push** to master/main
