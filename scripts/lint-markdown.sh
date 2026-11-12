#!/bin/bash
# lint-markdown.sh - Run rumdl markdown linter on the repository
#
# Usage: ./scripts/lint-markdown.sh [--fix]
#   --fix: Automatically fix issues where possible
#
# Exit codes:
#   0 - All markdown files are valid
#   1 - Issues found

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"

FIX=false
while [[ $# -gt 0 ]]; do
    case "$1" in
        --fix)
            FIX=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

cd "$REPO_ROOT"

echo "Running rumdl markdown linter..."

# Check if rumdl is installed
if ! command -v rumdl &> /dev/null; then
    echo "ERROR: rumdl is not installed."
    echo "Install it with: cargo install rumdl"
    exit 1
fi

# Run rumdl
if [ "$FIX" = true ]; then
    echo "Fixing issues..."
    rumdl fmt --config .rumdl.toml .
else
    echo "Checking markdown files..."
    rumdl check --config .rumdl.toml .
fi

echo "Markdown linting complete."
exit 0
