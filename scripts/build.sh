#!/usr/bin/env bash
set -euo pipefail

# Ensure we are in the project root
cd "$(dirname "$0")/.."

echo "==> Building release binary..."
cargo build --release

echo "==> Copying binary to dist/binaries..."
mkdir -p dist/binaries
cp target/release/chaos dist/binaries/
echo "==> Release binary copied successfully."
