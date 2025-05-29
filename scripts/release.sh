#!/bin/bash
# Release Helper Script for py2pyd
# This script helps automate the release process

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Function to print colored output
print_color() {
    printf "${1}${2}${NC}\n"
}

# Check if version is provided
if [ $# -eq 0 ]; then
    print_color $RED "Error: Version is required"
    echo "Usage: $0 <version> [message]"
    echo "Example: $0 1.0.0 'Initial release'"
    exit 1
fi

VERSION=$1
MESSAGE=${2:-"Release version $VERSION"}

# Validate version format (semantic versioning)
if ! [[ $VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    print_color $RED "Error: Version must be in format x.y.z (e.g., 1.0.0)"
    exit 1
fi

print_color $GREEN "🚀 Preparing release for version $VERSION"

# Check if we're on main branch
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
if [ "$CURRENT_BRANCH" != "main" ]; then
    print_color $YELLOW "Warning: You are not on the main branch. Current branch: $CURRENT_BRANCH"
    read -p "Do you want to continue? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_color $YELLOW "Release cancelled."
        exit 0
    fi
fi

# Check if working directory is clean
if [ -n "$(git status --porcelain)" ]; then
    print_color $RED "Error: Working directory is not clean. Please commit or stash your changes."
    git status
    exit 1
fi

# Update version in Cargo.toml
print_color $BLUE "📝 Updating version in Cargo.toml..."
sed -i.bak "s/version = \"[0-9]*\.[0-9]*\.[0-9]*\"/version = \"$VERSION\"/" Cargo.toml
rm -f Cargo.toml.bak

# Verify the change
NEW_VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
if [ "$NEW_VERSION" != "$VERSION" ]; then
    print_color $RED "Error: Failed to update version in Cargo.toml"
    exit 1
fi

print_color $GREEN "✅ Version updated to $NEW_VERSION"

# Build and test locally
print_color $BLUE "🔨 Building and testing locally..."
if ! cargo build --release; then
    print_color $RED "Error: Build failed. Please fix the issues before releasing."
    exit 1
fi

if ! cargo test; then
    print_color $RED "Error: Tests failed. Please fix the issues before releasing."
    exit 1
fi

print_color $GREEN "✅ Build and tests passed"

# Commit the version change
print_color $BLUE "📦 Committing version change..."
git add Cargo.toml
git commit -m "bump: version $VERSION

$MESSAGE

Signed-off-by: loonghao <hal.long@outlook.com>"

# Push to main branch
print_color $BLUE "🚀 Pushing to main branch..."
git push origin main

print_color $GREEN "🎉 Release process initiated!"
echo ""
print_color $CYAN "The automated release workflow will now:"
print_color $CYAN "  1. Detect the version change"
print_color $CYAN "  2. Build binaries for all platforms"
print_color $CYAN "  3. Create a Git tag (v$VERSION)"
print_color $CYAN "  4. Create a GitHub release with artifacts"
echo ""
print_color $YELLOW "You can monitor the progress at:"
print_color $YELLOW "https://github.com/loonghao/py2pyd/actions"
