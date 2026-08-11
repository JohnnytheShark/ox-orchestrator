#!/usr/bin/env bash
set -e

if [ -z "$1" ]; then
  echo "Usage: $0 <new-version> (e.g. v0.3.0 or 0.3.0)"
  exit 1
fi

NEW_VERSION=$1
# Strip leading 'v' if present
NEW_VERSION=${NEW_VERSION#v}

# Extract current version from Cargo.toml
CURRENT_VERSION=$(grep '^version = ' Cargo.toml | head -n 1 | sed -E 's/version = "(.*)"/\1/')

if [ -z "$CURRENT_VERSION" ]; then
  echo "Error: Could not extract current version from Cargo.toml"
  exit 1
fi

echo "Bumping version from ${CURRENT_VERSION} to ${NEW_VERSION}..."

replace() {
  # Use GNU sed word boundaries to prevent replacing "0.2.1" inside "0.2.10"
  sed -i "s/\b${CURRENT_VERSION}\b/${NEW_VERSION}/g" "$1"
}

FILES=(
  "Cargo.toml"
  ".github/workflows/release.yml"
  "README.md"
  "install.sh"
  "install.ps1"
  "site/app.js"
  "site/index.html"
)

for file in "${FILES[@]}"; do
  if [ -f "$file" ]; then
    echo "Updating $file..."
    replace "$file"
  else
    echo "Warning: File $file not found!"
  fi
done

echo "Updating Cargo.lock..."
if command -v cargo >/dev/null 2>&1; then
  cargo check --quiet
else
  echo "Cargo not found, skipping lockfile update"
fi

echo "Version bump complete!"
