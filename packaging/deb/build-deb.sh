#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$PROJECT_ROOT"

VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
STAGE="target/deb-staging"
ARCH="${1:-amd64}"

rm -rf "$STAGE"

# Create directory structure
mkdir -p "$STAGE/DEBIAN"
mkdir -p "$STAGE/usr/bin"
mkdir -p "$STAGE/usr/share/cogtome"

# Install binary
cp target/release/cogtome "$STAGE/usr/bin/"

# Install runtime assets
cp -r skills "$STAGE/usr/share/cogtome/"
cp -r units "$STAGE/usr/share/cogtome/"
cp -r assemblies "$STAGE/usr/share/cogtome/"
cp cogtome.toml "$STAGE/usr/share/cogtome/"

# Strip build artifacts from units (target/ dirs can be 38MB+)
find "$STAGE/usr/share/cogtome/units" -name target -type d -exec rm -rf {} + 2>/dev/null || true

# Generate control file
sed "s/@VERSION@/$VERSION/g" "$SCRIPT_DIR/control.in" > "$STAGE/DEBIAN/control"

# Set permissions
chmod 755 "$STAGE/usr/bin/cogtome"
find "$STAGE/usr/share/cogtome" -name "*.sh" -exec chmod 755 {} +
find "$STAGE/usr/share/cogtome" -path "*/bin/*" -type f -exec chmod 755 {} +

# Build .deb
dpkg-deb --root-owner-group --build "$STAGE" "target/cogtome_${VERSION}_${ARCH}.deb"

echo "Built: target/cogtome_${VERSION}_${ARCH}.deb"
