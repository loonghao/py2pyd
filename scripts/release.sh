#!/bin/bash
# Semantic Release Helper Script for py2pyd
# This script helps create conventional commits for automatic releases

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

# Check if commit type is provided
if [ $# -eq 0 ]; then
    print_color $RED "Error: Commit type and message are required"
    echo "Usage: $0 <type> <message> [scope]"
    echo ""
    echo "Types:"
    echo "  feat     - New feature (minor version bump)"
    echo "  fix      - Bug fix (patch version bump)"
    echo "  docs     - Documentation changes (patch version bump)"
    echo "  style    - Code style changes (patch version bump)"
    echo "  refactor - Code refactoring (patch version bump)"
    echo "  perf     - Performance improvements (patch version bump)"
    echo "  test     - Test changes (patch version bump)"
    echo "  chore    - Build/tool changes (patch version bump)"
    echo "  feat!    - Breaking change (major version bump)"
    echo ""
    echo "Examples:"
    echo "  $0 feat 'add Python 3.12 support'"
    echo "  $0 fix 'resolve memory leak in parser'"
    echo "  $0 feat 'redesign API' --breaking"
    echo "  $0 docs 'update installation guide'"
    exit 1
fi

TYPE=$1
MESSAGE=$2
SCOPE=$3
BREAKING=false

# Check for breaking change flag
if [ "$3" = "--breaking" ] || [ "$4" = "--breaking" ]; then
    BREAKING=true
    if [ "$3" = "--breaking" ]; then
        SCOPE=""
    fi
fi

# Validate commit type
VALID_TYPES="feat fix docs style refactor perf test chore"
if [[ ! " $VALID_TYPES " =~ " $TYPE " ]]; then
    print_color $RED "Error: Invalid commit type '$TYPE'"
    print_color $YELLOW "Valid types: $VALID_TYPES"
    exit 1
fi

print_color $GREEN "🚀 Creating semantic release commit"

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

# Build commit message
COMMIT_MSG=""
if [ -n "$SCOPE" ]; then
    COMMIT_MSG="${TYPE}(${SCOPE}): ${MESSAGE}"
else
    COMMIT_MSG="${TYPE}: ${MESSAGE}"
fi

# Add breaking change indicator
if [ "$BREAKING" = true ]; then
    if [[ "$COMMIT_MSG" != *"!"* ]]; then
        COMMIT_MSG="${TYPE}!: ${MESSAGE}"
        if [ -n "$SCOPE" ]; then
            COMMIT_MSG="${TYPE}(${SCOPE})!: ${MESSAGE}"
        fi
    fi
fi

print_color $BLUE "📝 Commit message: $COMMIT_MSG"

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

# Commit the changes
print_color $BLUE "📦 Committing changes..."
git add .
if [ "$BREAKING" = true ]; then
    git commit -m "$COMMIT_MSG

BREAKING CHANGE: $MESSAGE

Signed-off-by: loonghao <hal.long@outlook.com>"
else
    git commit -m "$COMMIT_MSG

Signed-off-by: loonghao <hal.long@outlook.com>"
fi

# Push to main branch
print_color $BLUE "🚀 Pushing to main branch..."
git push origin main

print_color $GREEN "🎉 Semantic release commit created!"
echo ""
print_color $CYAN "The automated semantic release workflow will now:"
print_color $CYAN "  1. Analyze the commit message"
print_color $CYAN "  2. Calculate the new version automatically"
print_color $CYAN "  3. Update Cargo.toml with the new version"
print_color $CYAN "  4. Build binaries for all platforms"
print_color $CYAN "  5. Create a Git tag and GitHub release"
echo ""
print_color $YELLOW "Expected version bump based on commit type:"
case $TYPE in
    "feat")
        if [ "$BREAKING" = true ]; then
            print_color $YELLOW "  Major version bump (e.g., 0.1.0 → 1.0.0)"
        else
            print_color $YELLOW "  Minor version bump (e.g., 0.1.0 → 0.2.0)"
        fi
        ;;
    "fix"|"docs"|"style"|"refactor"|"perf"|"test"|"chore")
        print_color $YELLOW "  Patch version bump (e.g., 0.1.0 → 0.1.1)"
        ;;
esac
echo ""
print_color $YELLOW "You can monitor the progress at:"
print_color $YELLOW "https://github.com/loonghao/py2pyd/actions"
