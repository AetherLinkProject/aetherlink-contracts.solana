#!/bin/bash
# coverage.sh - Code coverage automation for Anchor+Rust project
# Usage: bash coverage.sh
#
# 1. Clean old data
# 2. Run tests
# 3. Run tarpaulin for coverage
# 4. Extract and update coverage in tracker

set -e

# Check cargo-tarpaulin
if ! command -v cargo-tarpaulin &> /dev/null; then
  echo "cargo-tarpaulin not found. Install with: cargo install cargo-tarpaulin"
  exit 1
fi

# Clean old data
echo "Cleaning old build and coverage data..."
rm -rf target/
rm -f tarpaulin-report.html

# Run tests and coverage for oracle program
cd programs/oracle

echo "Running unit tests..."
cargo test --all-features

echo "Running tarpaulin for coverage..."
cargo tarpaulin --out Html --output-dir ../../

cd ../..

echo "Extracting and updating coverage..."
node scripts/extract_coverage.ts

echo "Coverage workflow complete. See docs/project_tracker.md for updated coverage." 