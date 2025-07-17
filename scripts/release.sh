#!/bin/bash
set -e

# Release script for Apollo Air-1 Exporter

if [ -z "$1" ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 0.1.1"
    exit 1
fi

VERSION="$1"
TAG="v$VERSION"

echo "Preparing release $TAG..."

# Check if on main branch
BRANCH=$(git branch --show-current)
if [ "$BRANCH" != "main" ]; then
    echo "Error: Must be on main branch to release (currently on $BRANCH)"
    exit 1
fi

# Check for uncommitted changes
if ! git diff-index --quiet HEAD --; then
    echo "Error: There are uncommitted changes"
    exit 1
fi

# Update version in Cargo.toml
sed -i.bak "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
rm Cargo.toml.bak

# Update Cargo.lock
cargo update -p apollo-air1-exporter

# Run tests
echo "Running tests..."
cargo test

# Run clippy
echo "Running clippy..."
cargo clippy -- -D warnings

# Update CHANGELOG
echo "Please update CHANGELOG.md for version $VERSION"
echo "Press Enter when done..."
read

# Commit version bump
git add Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "chore: bump version to $VERSION"

# Create and push tag
git tag -a "$TAG" -m "Release $TAG"

echo ""
echo "Release $TAG prepared!"
echo ""
echo "To publish the release:"
echo "  git push origin main"
echo "  git push origin $TAG"
echo ""
echo "This will trigger:"
echo "- GitHub Release creation"
echo "- Docker multi-arch builds"
echo "- Binary artifacts upload"