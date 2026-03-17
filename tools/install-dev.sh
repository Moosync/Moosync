#!/bin/bash
# Install development build symlinks for distrobox export
# In a distrobox container, system paths are writable without sudo

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

BINARY_PATH="$PROJECT_ROOT/bazel-bin/tauri/moosync"
DESKTOP_PATH="$PROJECT_ROOT/package/linux/moosync.desktop"

# Check if binary exists
if [ ! -f "$BINARY_PATH" ]; then
    echo "Error: Binary not found at $BINARY_PATH"
    echo "Run 'bazel build //tauri:moosync' first"
    exit 1
fi

echo "Installing development symlinks..."

# Create binary symlink
ln -sf "$BINARY_PATH" /usr/local/bin/moosync
echo "  /usr/local/bin/moosync -> $BINARY_PATH"

# Create desktop file symlink
ln -sf "$DESKTOP_PATH" /usr/share/applications/moosync.desktop
echo "  /usr/share/applications/moosync.desktop -> $DESKTOP_PATH"

echo ""
echo "Done. You can now run:"
echo "  distrobox-export-gpg moosync"
