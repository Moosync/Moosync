#!/usr/bin/env bash
# Formatter script using hermetic tools from Bazel runfiles.

# --- begin runfiles.bash initialization v2 ---
# Copy-pasted from the Bazel Bash runfiles library v2.
set -uo pipefail; f=bazel_tools/tools/bash/runfiles/runfiles.bash
source "${RUNFILES_DIR:-/dev/null}/$f" 2>/dev/null || \
  source "$(grep -sm1 "^$f " "${RUNFILES_MANIFEST_FILE:-/dev/null}" | cut -f2- -d' ')" 2>/dev/null || \
  source "$0.runfiles/$f" 2>/dev/null || \
  source "$(grep -sm1 "^$f " "$0.runfiles_manifest" | cut -f2- -d' ')" 2>/dev/null || \
  source "$(dirname "$0")/$f" 2>/dev/null || \
  source "$(dirname "$0")/$0.runfiles/$f" 2>/dev/null || \
  { echo >&2 "ERROR: cannot find $f"; exit 1; }
# --- end runfiles.bash initialization v2 ---

RUSTFMT_PATH=$(rlocation "$1")
SLINT_LSP_PATH=$(rlocation "$2")
BUILDIFIER_PATH=$(rlocation "$3")
BUF_PATH=$(rlocation "$4")
shift 4

if [ -z "$RUSTFMT_PATH" ] || [ ! -f "$RUSTFMT_PATH" ]; then
  echo "ERROR: Could not resolve hermetic rustfmt path" >&2
  exit 1
fi

if [ -z "$SLINT_LSP_PATH" ] || [ ! -f "$SLINT_LSP_PATH" ]; then
  echo "ERROR: Could not resolve hermetic slint-lsp path" >&2
  exit 1
fi

if [ -z "$BUILDIFIER_PATH" ] || [ ! -f "$BUILDIFIER_PATH" ]; then
  echo "ERROR: Could not resolve hermetic buildifier path" >&2
  exit 1
fi

if [ -z "$BUF_PATH" ] || [ ! -f "$BUF_PATH" ]; then
  echo "ERROR: Could not resolve hermetic buf path" >&2
  exit 1
fi

# Ensure buf is executable
chmod +x "$BUF_PATH" 2>/dev/null || true

# Get the root of the workspace
WORKSPACE_DIR="${BUILD_WORKSPACE_DIRECTORY:-$(pwd)}"
cd "$WORKSPACE_DIR"

if [ $# -gt 0 ]; then
  # Format only the specified files (e.g. staged files)
  for file in "$@"; do
    if [ -f "$file" ]; then
      case "$file" in
        *.rs)
          echo "Formatting $file..."
          "$RUSTFMT_PATH" --edition 2024 "$file"
          ;;
        *.slint)
          echo "Formatting $file..."
          "$SLINT_LSP_PATH" format -i "$file"
          ;;
        *BUILD|*BUILD.bazel|*WORKSPACE|*WORKSPACE.bazel|*MODULE.bazel|*.bzl|*.bazel)
          echo "Formatting $file..."
          "$BUILDIFIER_PATH" "$file"
          ;;
        *.proto)
          echo "Formatting $file..."
          "$BUF_PATH" format -w "$file"
          ;;
      esac
    fi
  done
else
  # Format all files in the workspace
  echo "Formatting all Rust files..."
  find . -type f -name "*.rs" \
    -not -path "*/bazel-*" \
    -not -path "*/target/*" \
    -not -path "*/.git/*" \
    -print0 | xargs -0 "$RUSTFMT_PATH" --edition 2024

  echo "Formatting all Slint files..."
  find . -type f -name "*.slint" \
    -not -path "*/bazel-*" \
    -not -path "*/.git/*" \
    -print0 | xargs -0 -n 1 "$SLINT_LSP_PATH" format -i

  echo "Formatting all Bazel files..."
  find . -type f \( -name "BUILD" -o -name "BUILD.bazel" -o -name "WORKSPACE" -o -name "WORKSPACE.bazel" -o -name "MODULE.bazel" -o -name "*.MODULE.bazel" -o -name "*.bzl" -o -name "*.bazel" \) \
    -not -path "*/bazel-*" \
    -not -path "*/target/*" \
    -not -path "*/.git/*" \
    -print0 | xargs -0 "$BUILDIFIER_PATH"

  echo "Formatting all Proto files..."
  find . -type f -name "*.proto" \
    -not -path "*/bazel-*" \
    -not -path "*/target/*" \
    -not -path "*/.git/*" \
    -print0 | xargs -0 -n 1 "$BUF_PATH" format -w
fi

echo "Formatting completed."
