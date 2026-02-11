# Pre-Push Workflow

## Overview

**CRITICAL**: Always run pre-push checks before `git push` to ensure code quality and security.

## Quick Start

```bash
just pre-push
```

This runs:
1. **CI checks** (format, lint, test, build)
2. **Security audit** (secrets, vulnerabilities, unsafe code)

## What Gets Checked

### 1. Code Quality (CI Checks)

- **Format Check**: `cargo fmt --check`
  - Ensures consistent code formatting
  - Auto-fix with: `just format`

- **Clippy Lints**: Maximum strictness enabled
  - Pedantic lints for best practices
  - Custom restrictions (no `dbg!`, `todo!`, `unimplemented!`)
  - Uniform async signatures allowed
  - Auto-fix some issues with: `cargo clippy --fix`

- **Tests**: All unit and integration tests
  - 45 unit tests
  - 10 integration tests
  - 100% must pass

- **Build**: Release build verification
  - Ensures production build works
  - Optimized with LTO and single codegen unit

### 2. Security Audit

- **Secrets Detection**
  - Scans for hardcoded passwords, tokens, API keys
  - Warns if potential secrets found
  - Review and ensure they're test data only

- **Vulnerability Scan** (requires `cargo-audit`)
  - Checks for known CVEs in dependencies
  - Install: `cargo install cargo-audit`
  - Fails on known vulnerabilities

- **Unsafe Code Detection**
  - Scans for `unsafe` blocks
  - **Forbidden** in production code (compile error)
  - Unsafe code not allowed per Cargo.toml lints

- **Unfinished Code Markers**
  - Checks for TODO, FIXME, XXX in production code
  - Warns but doesn't fail
  - Address before releases

- **Outdated Dependencies** (requires `cargo-outdated`)
  - Checks for outdated crate versions
  - Install: `cargo install cargo-outdated`
  - Warns but doesn't fail

## Lint Configuration

Maximum strictness is configured in `native-host/Cargo.toml`:

```toml
[lints.rust]
unsafe_code = "forbid"           # No unsafe code allowed
unused_must_use = "deny"         # Must use Result/Option returns

[lints.clippy]
pedantic = "warn"                # Best practices
dbg_macro = "warn"               # No debug macros in production
todo = "warn"                    # No unfinished code
unimplemented = "warn"           # No placeholder code
```

## CI Pipeline

GitHub Actions runs the same checks on every push:

```yaml
- Format Check (cargo fmt --check)
- Clippy Lints (--all-targets --all-features)
- Tests (Ubuntu + macOS)
- Build (Release mode)
- Code Coverage
```

**All checks must pass** before PR can be merged.

## Release Workflow

**NEVER** create releases/tags without:

1. ✅ All CI checks passing
2. ✅ Security audit passing
3. ✅ Manual testing complete
4. ✅ CHANGELOG updated

### Creating a Release

```bash
# 1. Run pre-push checks
just pre-push

# 2. Update version and create tag
just release 0.2.0

# 3. Push (triggers CI + release workflow)
git push origin master && git push origin v0.2.0
```

The release workflow:
1. Waits for CI to pass
2. Builds binaries (Linux, macOS x64/ARM)
3. Creates GitHub release
4. Uploads artifacts
5. Updates Homebrew formula

## Fixing Issues

### Format Errors

```bash
just format
```

### Clippy Warnings

```bash
# See all warnings
cargo clippy --all-targets --all-features

# Auto-fix some issues
cargo clippy --fix --allow-dirty

# Review remaining warnings manually
```

### Test Failures

```bash
# Run tests with output
just test-verbose

# Run specific test
cargo test test_name -- --nocapture

# Run integration tests only
just test-integration
```

### Security Issues

#### Hardcoded Secrets
- **Action**: Remove and use environment variables or keychain
- **Never commit**: tokens, passwords, API keys

#### Known Vulnerabilities
- **Action**: Update dependencies
- **Command**: `cargo update`
- **Verify**: `cargo audit`

#### Unsafe Code
- **Action**: Rewrite without unsafe
- **If truly needed**: Document why and get review

## Best Practices

1. **Run pre-push locally** before every push
2. **Fix all warnings** - don't ignore them
3. **Review security audit** carefully
4. **Update dependencies** regularly
5. **Write tests** for new code
6. **Document** non-obvious code

## Automation

### Git Hook (Optional)

Create `.git/hooks/pre-push`:

```bash
#!/bin/bash
just pre-push
```

Make executable:
```bash
chmod +x .git/hooks/pre-push
```

Now checks run automatically before every push.

## Troubleshooting

### "npm: command not found"
- Extension build requires Node.js
- Install: `brew install node` (macOS)
- Or skip extension build during Rust-only work

### "cargo-audit not installed"
- Security check skipped
- Install: `cargo install cargo-audit`
- Recommended for comprehensive security checks

### "cargo-outdated not installed"
- Dependency check skipped
- Install: `cargo install cargo-outdated`
- Optional but useful

### CI Passes Locally But Fails in GitHub
- Check for platform-specific issues
- macOS vs Linux differences
- Review CI logs on GitHub

## Summary

**Before every push:**

```bash
just pre-push
```

**Before every release:**

```bash
just pre-push
just release <version>
git push origin master && git push origin v<version>
```

This ensures:
- High code quality
- No security vulnerabilities
- Consistent formatting
- All tests passing
- Production build works
