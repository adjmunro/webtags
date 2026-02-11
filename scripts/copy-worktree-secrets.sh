#!/bin/bash

# Script to copy worktree secrets to a target worktree
# Reads patterns from worktree-secrets and copies matching files
# Usage: ./scripts/copy-worktree-secrets.sh <target-worktree-path>

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Check if target path is provided
if [ $# -eq 0 ]; then
    echo -e "${RED}Error: No target worktree path provided${NC}"
    echo "Usage: $0 <target-worktree-path>"
    echo "Example: $0 ../wt-feature-branch"
    exit 1
fi

TARGET="$1"

# Change to repository root
REPO_ROOT=$(git rev-parse --show-toplevel)
cd "$REPO_ROOT"

# Check if worktree-secrets exists
if [ ! -f "worktree-secrets" ]; then
    echo -e "${RED}Error: worktree-secrets file not found${NC}"
    exit 1
fi

# Check if target exists
if [ ! -d "$TARGET" ]; then
    echo -e "${RED}Error: Target directory '$TARGET' does not exist${NC}"
    exit 1
fi

# Resolve target to absolute path
TARGET=$(cd "$TARGET" && pwd)

echo "Copying worktree secrets to: $TARGET"
echo ""

# Function to check if string contains glob characters
has_glob() {
    [[ "$1" =~ [\*\?\[] ]]
}

# Function to check if string contains a slash
has_slash() {
    [[ "$1" == */* ]]
}

# Counters
COPIED=0
NOT_FOUND=0

# Process each line in worktree-secrets
while IFS= read -r pattern; do
    # Skip empty lines and comments
    [[ -z "$pattern" ]] && continue
    [[ "$pattern" =~ ^[[:space:]]*# ]] && continue

    # Trim whitespace
    pattern=$(echo "$pattern" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')
    [[ -z "$pattern" ]] && continue

    echo -e "${YELLOW}Processing pattern:${NC} $pattern"

    # Determine pattern type and find matching files
    files=()

    if has_glob "$pattern" && has_slash "$pattern"; then
        # Type 1: Glob pattern with slash - matches against full paths
        while IFS= read -r file; do
            files+=("$file")
        done < <(find . -path "./$pattern" 2>/dev/null | sed 's|^\./||' || true)

    elif has_glob "$pattern" && ! has_slash "$pattern"; then
        # Type 2: Glob pattern without slash - matches basenames only
        while IFS= read -r file; do
            files+=("$file")
        done < <(find . -name "$pattern" -type f 2>/dev/null | sed 's|^\./||' || true)

    elif ! has_glob "$pattern" && has_slash "$pattern"; then
        # Type 3: Path fragment match
        if [[ "$pattern" == */ ]]; then
            # Directory pattern - copy entire directory tree
            while IFS= read -r file; do
                files+=("$file")
            done < <(find . -path "*$pattern*" 2>/dev/null | sed 's|^\./||' || true)
        else
            # File path fragment
            while IFS= read -r file; do
                files+=("$file")
            done < <(find . -path "*$pattern*" -type f 2>/dev/null | sed 's|^\./||' || true)
        fi

    else
        # Type 4: Exact basename match
        while IFS= read -r file; do
            files+=("$file")
        done < <(find . -name "$pattern" -type f 2>/dev/null | sed 's|^\./||' || true)
    fi

    # Filter out build artifacts (Rust target, node_modules, dist)
    filtered_files=()
    for file in "${files[@]}"; do
        # Skip build directories, dependencies, and generated files
        if [[ ! "$file" =~ /target/ ]] && \
           [[ ! "$file" =~ /node_modules/ ]] && \
           [[ ! "$file" =~ /dist/ ]] && \
           [[ ! "$file" =~ /.next/ ]]; then
            filtered_files+=("$file")
        fi
    done
    files=("${filtered_files[@]}")

    if [ ${#files[@]} -eq 0 ]; then
        echo -e "  ${RED}✗${NC} No files found"
        ((NOT_FOUND++))
    else
        for file in "${files[@]}"; do
            # Create target directory if needed
            target_dir="$TARGET/$(dirname "$file")"
            mkdir -p "$target_dir"

            # Copy file with permissions preserved
            if cp -p "$file" "$TARGET/$file" 2>/dev/null; then
                echo -e "  ${GREEN}✓${NC} Copied: $file"
                ((COPIED++))
            else
                echo -e "  ${RED}✗${NC} Failed to copy: $file"
            fi
        done
    fi
    echo ""

done < worktree-secrets

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${GREEN}✓ Worktree secrets copy complete!${NC}"
echo "  Files copied: $COPIED"
echo "  Patterns not found: $NOT_FOUND"
echo "  Target: $TARGET"
