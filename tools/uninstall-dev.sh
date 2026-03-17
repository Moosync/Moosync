#!/bin/bash
# Remove development build symlinks
# In a distrobox container, system paths are writable without sudo

set -e

echo "Removing development symlinks..."

# Remove binary symlink
if [ -L /usr/local/bin/moosync ]; then
    rm /usr/local/bin/moosync
    echo "  Removed /usr/local/bin/moosync"
else
    echo "  /usr/local/bin/moosync not found (skipped)"
fi

# Remove desktop file symlink
if [ -L /usr/share/applications/moosync.desktop ]; then
    rm /usr/share/applications/moosync.desktop
    echo "  Removed /usr/share/applications/moosync.desktop"
else
    echo "  /usr/share/applications/moosync.desktop not found (skipped)"
fi

echo ""
echo "Done. You may also want to run:"
echo "  distrobox-export --delete --app moosync"
