#!/usr/bin/env bash
# Bump HyperHost version across all project files.
#
# Usage:
#   ./scripts/bump-version.sh 0.2.0
#   ./scripts/bump-version.sh 0.2.0 --push

set -euo pipefail

VERSION="${1:-}"
PUSH=false

if [[ -z "$VERSION" ]]; then
    echo "Usage: $0 <version> [--push]"
    echo "  version: semver string, e.g. 0.2.0"
    exit 1
fi

if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Error: version must match X.Y.Z (got '$VERSION')"
    exit 1
fi

if [[ "${2:-}" == "--push" ]]; then
    PUSH=true
fi

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

echo ""
echo "Bumping HyperHost to v$VERSION"
echo ""

# --- 1. tauri.conf.json ---
TAURI_CONF="$ROOT/src-tauri/tauri.conf.json"
OLD_VERSION=$(grep '"version"' "$TAURI_CONF" | head -1 | sed 's/.*"version": "\([^"]*\)".*/\1/')
sed -i "s/\"version\": \"$OLD_VERSION\"/\"version\": \"$VERSION\"/" "$TAURI_CONF"
echo "  tauri.conf.json    $OLD_VERSION → $VERSION"

# --- 2. Cargo.toml ---
CARGO_TOML="$ROOT/src-tauri/Cargo.toml"
sed -i "0,/^version = \"[^\"]*\"/s//version = \"$VERSION\"/" "$CARGO_TOML"
echo "  Cargo.toml         → $VERSION"

# --- 3. package.json ---
PKG_JSON="$ROOT/package.json"
sed -i "s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/" "$PKG_JSON"
echo "  package.json       → $VERSION"

# --- 4. App.tsx ---
APP_TSX="$ROOT/src/App.tsx"
if [[ -f "$APP_TSX" ]]; then
    sed -i "s/v[0-9]\+\.[0-9]\+\.[0-9]\+<\/span>/v$VERSION<\/span>/" "$APP_TSX"
    echo "  App.tsx            → v$VERSION"
fi

echo ""
echo "All files updated to v$VERSION"

# --- 5. Git commit + tag + push (optional) ---
if [[ "$PUSH" == true ]]; then
    echo ""
    echo "Creating git tag v$VERSION..."
    cd "$ROOT"
    git add src-tauri/tauri.conf.json src-tauri/Cargo.toml package.json src/App.tsx
    git commit -m "release: v$VERSION"
    git tag "v$VERSION"
    git push origin HEAD --tags
    echo "  Pushed v$VERSION to origin"
else
    echo ""
    echo "Next steps:"
    echo "  1. git add src-tauri/tauri.conf.json src-tauri/Cargo.toml package.json src/App.tsx"
    echo "  2. git commit -m 'release: v$VERSION'"
    echo "  3. git tag v$VERSION"
    echo "  4. git push origin HEAD --tags"
    echo ""
    echo "  Or run: $0 $VERSION --push"
fi
echo ""
