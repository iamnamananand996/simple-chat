#!/bin/bash

# Script to install git hooks for the simple-chat project
# Run this script after cloning the repository to set up development environment

set -e

echo "🔧 Installing git hooks for simple-chat project..."

# Check if we're in a git repository
if [ ! -d ".git" ]; then
    echo "Error: Not in a git repository. Please run this script from the project root."
    exit 1
fi

# Check if hooks directory exists
if [ ! -d ".git/hooks" ]; then
    echo "Error: .git/hooks directory not found."
    exit 1
fi

# Install pre-commit hook
echo "Installing pre-commit hook..."
if [ -f "scripts/pre-commit" ]; then
    cp scripts/pre-commit .git/hooks/pre-commit
    chmod +x .git/hooks/pre-commit
    echo "Pre-commit hook installed successfully!"
else
    echo "Error: scripts/pre-commit not found."
    exit 1
fi

# Verify installation
if [ -x ".git/hooks/pre-commit" ]; then
    echo "🎉 Git hooks installation complete!"
    echo ""
    echo "The pre-commit hook will now run automatically before each commit to:"
    echo "  • Check code formatting (cargo fmt)"
    echo "  • Verify compilation (cargo check)"
    echo "  • Run clippy linting (cargo clippy)"
    echo "  • Execute tests (cargo test)"
    echo ""
    echo "To test the hook manually, run: .git/hooks/pre-commit"
else
    echo "Error: Hook installation failed."
    exit 1
fi