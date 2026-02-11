# WebTags - Project Commands
# Install `just` via Homebrew with `brew install just`
# Run `just` or `just --list` to see all available commands

# Default recipe - show help
default:
    @just --list

# Build everything (native host + extension)
build:
    @echo "Building native host..."
    cd native-host && cargo build --release
    @echo ""
    @echo "Building extension..."
    cd extension && npm run build

# Build native host only
build-host:
    cd native-host && cargo build --release

# Build extension only
build-extension:
    cd extension && npm run build

# Build for development (unoptimized)
build-dev:
    cd native-host && cargo build
    cd extension && npm run build

alias unit-test := test
# Run all tests (Rust unit + integration tests)
test:
    cd native-host && cargo test

# Run only unit tests (library tests)
test-unit:
    cd native-host && cargo test --lib

# Run only integration tests
test-integration:
    cd native-host && cargo test --test integration_tests

# Run tests with output
test-verbose:
    cd native-host && cargo test -- --nocapture

# Check code without building (fast)
check:
    cd native-host && cargo check

# Format code
format:
    cd native-host && cargo fmt

# Check formatting without changing files
format-check:
    cd native-host && cargo fmt -- --check

# Lint the code
lint:
    cd native-host && cargo clippy -- -D warnings

# Clean build artifacts
clean:
    cd native-host && cargo clean
    cd extension && rm -rf dist node_modules

# Install extension dependencies
install:
    cd extension && npm install

# Copy worktree secrets to a target worktree
cp-secrets TARGET:
    @./scripts/copy-worktree-secrets.sh {{TARGET}}

alias wtc := wt-create
# Create a new worktree and copy secrets to it
# Usage: just wt-create <branch-name> [path]
wt-create BRANCH PATH="":
    #!/usr/bin/env bash
    set -euo pipefail

    # Default path if not provided
    if [ -z "{{PATH}}" ]; then
        WORKTREE_PATH="../wt-$(basename {{BRANCH}})"
    else
        WORKTREE_PATH="{{PATH}}"
    fi

    echo "Creating worktree for branch '{{BRANCH}}' at '$WORKTREE_PATH'..."
    git worktree add -b "{{BRANCH}}" "$WORKTREE_PATH"

    echo ""
    echo "Copying secrets to new worktree..."
    ./scripts/copy-worktree-secrets.sh "$WORKTREE_PATH"

    echo ""
    echo "✓ Worktree setup complete!"
    echo "  cd $WORKTREE_PATH"

alias wtls := wt-list
# List all worktrees
wt-list:
    @git worktree list

alias wtrm := wt-remove
# Remove a worktree
wt-remove PATH:
    git worktree remove {{PATH}}

alias reagent := symlink-claude
# Setup CLAUDE.md symlinks to AGENTS.md files
symlink-claude:
    @./scripts/symlink-claude.sh

# Run the native host (for testing)
run-host:
    cd native-host && cargo run

# Create a release build with version bump
release VERSION:
    #!/usr/bin/env bash
    set -euo pipefail

    echo "Creating release {{VERSION}}..."

    # Update version in Cargo.toml
    cd native-host
    sed -i.bak "s/^version = .*/version = \"{{VERSION}}\"/" Cargo.toml
    rm Cargo.toml.bak

    # Update version in manifest.json
    cd ../extension
    sed -i.bak "s/\"version\": \".*\"/\"version\": \"{{VERSION}}\"/" manifest.json
    rm manifest.json.bak

    cd ..

    # Build release
    just build

    # Commit version bump
    git add native-host/Cargo.toml extension/manifest.json
    git commit -m "Bump version to {{VERSION}}"

    # Create tag
    git tag -a "v{{VERSION}}" -m "Release v{{VERSION}}"

    echo ""
    echo "✓ Release v{{VERSION}} created!"
    echo "  To publish: git push origin master && git push origin v{{VERSION}}"

# Package the native host for distribution
package:
    #!/usr/bin/env bash
    set -euo pipefail

    cd native-host
    cargo build --release

    VERSION=$(cargo metadata --format-version 1 --no-deps | jq -r '.packages[0].version')
    TARGET_DIR="target/release"
    PACKAGE_DIR="target/package/webtags-$VERSION"

    echo "Packaging webtags-host v$VERSION..."

    # Create package directory
    rm -rf target/package
    mkdir -p "$PACKAGE_DIR"

    # Copy binary
    cp "$TARGET_DIR/webtags-host" "$PACKAGE_DIR/"

    # Create tarball
    cd target/package
    tar -czf "webtags-$VERSION-$(uname -m)-apple-darwin.tar.gz" "webtags-$VERSION"

    echo ""
    echo "✓ Package created: native-host/target/package/webtags-$VERSION-$(uname -m)-apple-darwin.tar.gz"

# Watch for changes and rebuild (requires cargo-watch)
watch:
    cd native-host && cargo watch -x check -x test

# Generate code coverage (requires cargo-tarpaulin)
coverage:
    cd native-host && cargo tarpaulin --out Html --output-dir coverage

# Update dependencies
update:
    cd native-host && cargo update
    cd extension && npm update

# Run CI checks locally (same as CI pipeline)
ci:
    @echo "Running CI checks..."
    just format-check
    just lint
    just test
    just build
    @echo ""
    @echo "✓ All CI checks passed!"
