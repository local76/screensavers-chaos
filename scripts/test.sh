#!/usr/bin/env bash
set -euo pipefail

# Ensure we are in the project root
cd "$(dirname "$0")/.."

echo "=== Running Cargo Check ==="
cargo check

echo "=== Running Cargo Clippy ==="
cargo clippy

echo "=== Running Cargo Test ==="
cargo test

echo "=== All checks and tests passed successfully! ==="
