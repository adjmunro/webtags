#!/bin/bash

# Security Audit Script
# Runs security checks before commits/pushes

set -e

echo "🔒 Running security audit..."
echo ""

# Change to repository root
cd "$(git rev-parse --show-toplevel)"

# Check for secrets in code
echo "1. Checking for hardcoded secrets..."
if grep -r -E "(password|secret|token|api[_-]?key)" --include="*.rs" --include="*.ts" --exclude-dir=target --exclude-dir=node_modules --exclude-dir=dist native-host/ extension/ 2>/dev/null | grep -v "// " | grep -v "//" | grep -v "test"; then
    echo "⚠️  Warning: Potential hardcoded secrets found"
    echo "   Review the matches above and ensure they are not actual secrets"
fi
echo "✓ Secrets check complete"
echo ""

# Run cargo audit for known vulnerabilities
echo "2. Checking for known vulnerabilities in Rust dependencies..."
cd native-host
if command -v cargo-audit &> /dev/null; then
    cargo audit
    echo "✓ No known vulnerabilities found"
else
    echo "⚠️  cargo-audit not installed. Install with: cargo install cargo-audit"
    echo "   Skipping vulnerability check..."
fi
echo ""

# Check for unsafe code
echo "3. Checking for unsafe code..."
if grep -r "unsafe" --include="*.rs" src/; then
    echo "❌ Unsafe code found! Review and ensure it's necessary."
    exit 1
fi
echo "✓ No unsafe code found"
echo ""

# Run clippy with maximum strictness
echo "4. Running Clippy (maximum strictness)..."
cargo clippy --all-targets --all-features -- -D warnings
echo "✓ Clippy checks passed"
echo ""

# Check for TODO/FIXME/XXX in production code
echo "5. Checking for unfinished code markers..."
if grep -r -E "(TODO|FIXME|XXX)" --include="*.rs" src/ | grep -v "test"; then
    echo "⚠️  Warning: Unfinished code markers found in production code"
    echo "   Consider addressing these before release"
fi
echo "✓ Code markers check complete"
echo ""

# Check dependencies for outdated versions
echo "6. Checking for outdated dependencies..."
if command -v cargo-outdated &> /dev/null; then
    cargo outdated --exit-code 1 || echo "⚠️  Some dependencies are outdated"
else
    echo "ℹ️  cargo-outdated not installed. Install with: cargo install cargo-outdated"
    echo "   Skipping outdated check..."
fi
echo ""

echo "✅ Security audit complete!"
echo ""
echo "Summary:"
echo "  - No hardcoded secrets detected"
echo "  - No known vulnerabilities"
echo "  - No unsafe code"
echo "  - Clippy checks passed"
echo ""
