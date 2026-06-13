#!/usr/bin/env bash
set -euo pipefail

# Ensure we are in the project root
cd "$(dirname "$0")/.."

echo "==> 1. Running cargo test..."
cargo test

echo "==> 2. Building release..."
./scripts/build.sh

# Extract version from Cargo.toml
VERSION=$(grep -m1 '^version = ' Cargo.toml | cut -d '"' -f2)
TAG_NAME="v$VERSION"

echo "==> 3. Tagging git release with $TAG_NAME..."
if git rev-parse "$TAG_NAME" >/dev/null 2>&1; then
    echo "Tag $TAG_NAME already exists, skipping tag creation."
else
    git tag -a "$TAG_NAME" -m "Release $TAG_NAME"
    echo "Created tag $TAG_NAME."
fi

echo "==> 4. Pushing tags..."
# Only push tags if there's a remote configured
if git remote | grep -q 'origin'; then
    git push origin "$TAG_NAME"
else
    echo "No remote 'origin' configured. Skipping push."
fi

echo "==> Release $TAG_NAME completed successfully."
