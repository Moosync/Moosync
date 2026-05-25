#!/bin/bash
# Find runfiles directory
RUNFILES_DIR="${RUNFILES_DIR:-$0.runfiles}"

# Locate the moosync_flutter binary inside the runfiles directory
BIN_PATH=$(find "$RUNFILES_DIR" -type f -name "moosync_flutter" | grep "moosync_flutter/moosync_flutter$" | head -n 1)
if [ -z "$BIN_PATH" ]; then
    # Fallback to direct path
    BIN_PATH="$RUNFILES_DIR/_main/ui/flutter/moosync_flutter/moosync_flutter"
fi

exec "$BIN_PATH" "$@"
