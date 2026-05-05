#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$PROJECT_ROOT"

VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')

# Prepare staging directory with runtime assets
rm -rf target/windows-staging
mkdir -p target/windows-staging
cp -r skills target/windows-staging/
cp -r units target/windows-staging/
cp -r assemblies target/windows-staging/
cp cogtome.toml target/windows-staging/

# Strip Unix build artifacts
find target/windows-staging/units -name target -type d -exec rm -rf {} + 2>/dev/null || true

# Run NSIS (must be installed: apt install nsis / brew install nsis / choco install nsis)
if ! command -v makensis &> /dev/null; then
    echo "Error: makensis not found."
    echo "Install NSIS:"
    echo "  Linux:  sudo apt install nsis"
    echo "  macOS:  brew install nsis"
    echo "  Windows: choco install nsis"
    exit 1
fi

export VERSION
makensis -DVERSION="$VERSION" "$SCRIPT_DIR/installer.nsi"

echo "Built: target/cogtome-${VERSION}-setup.exe"
