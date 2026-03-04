#!/usr/bin/env bash
# Local CI check script - runs the same checks as GitHub Actions CI
# Run this before pushing to catch warnings/errors locally

set -e

echo "🔍 Running local CI checks..."
echo ""

echo "📝 Checking formatting..."
cargo fmt --all -- --check
echo "✓ Formatting OK"
echo ""

echo "📎 Running Clippy (with warnings as errors)..."
cargo clippy --all-targets -- -D warnings
echo "✓ Clippy OK"
echo ""

echo "🧪 Running tests..."
cargo test
echo "✓ Tests OK"
echo ""

echo "✅ All checks passed! Safe to push."
