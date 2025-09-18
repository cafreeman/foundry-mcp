#!/bin/bash
set -euo pipefail

# Extract changelog for a specific version
# Usage: extract-changelog.sh <version>

if [ $# -ne 1 ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 v0.4.0"
    exit 1
fi

VERSION="$1"
# Remove 'v' prefix if present for changelog lookup
VERSION_NO_V="${VERSION#v}"

# Check if CHANGELOG.md exists
if [ ! -f "CHANGELOG.md" ]; then
    echo "Error: CHANGELOG.md not found"
    exit 1
fi

# Extract changelog section for this version
awk -v version="$VERSION_NO_V" '
/^## \[/ {
    if (found) exit
    if ($0 ~ "\\[" version "\\]") {
        found = 1
        next
    }
}
found && /^## \[/ { exit }
found && !/^## \[/ { print }
' CHANGELOG.md > release_notes.txt

# If no release notes found, create a default message
if [ ! -s release_notes.txt ]; then
    echo "Release $VERSION" > release_notes.txt
fi

# Output the release notes
cat release_notes.txt

# Clean up
rm -f release_notes.txt