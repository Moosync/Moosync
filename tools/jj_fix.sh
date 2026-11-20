#!/usr/bin/env bash
set -euo pipefail

WORKSPACE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$WORKSPACE_DIR"

find_manifest() {
  for candidate in \
    "$WORKSPACE_DIR/bazel-bin/tools/format.runfiles_manifest" \
    "$WORKSPACE_DIR/bazel-out/k8-fastbuild/bin/tools/format.runfiles_manifest" \
    "$WORKSPACE_DIR/bazel-Moosync/bazel-out/k8-fastbuild/bin/tools/format.runfiles_manifest"; do
    if [[ -f "$candidate" ]]; then
      printf '%s' "$candidate"
      return 0
    fi
  done
  find -L "$WORKSPACE_DIR/bazel-out" -path "*/bin/tools/format.runfiles_manifest" 2>/dev/null | head -n 1
}

MANIFEST=$(find_manifest)

if [[ -z "$MANIFEST" ]] || [[ ! -f "$MANIFEST" ]]; then
  LOCK_FILE="/tmp/moosync_bazel_format.lock"
  (
    flock 200
    M=$(find_manifest)
    if [[ -z "$M" ]] || [[ ! -f "$M" ]]; then
      if ! err=$(bazel build --ui_event_filters=-info,-stdout --noshow_progress //tools:format 2>&1); then
        echo "$err" >&2
        exit 1
      fi
    fi
  ) 200>"$LOCK_FILE"
  MANIFEST=$(find_manifest)
fi

if [[ -z "$MANIFEST" ]] || [[ ! -f "$MANIFEST" ]]; then
  echo "Error: format.runfiles_manifest could not be located" >&2
  exit 1
fi

export RUNFILES_MANIFEST_FILE="$MANIFEST"
export BUILD_WORKSPACE_DIRECTORY="$WORKSPACE_DIR"

find_rlocation() {
  grep -m1 "$1" "$MANIFEST" | cut -d' ' -f1
}

if [[ "${1:-}" == core/database/src/*.rs ]] || [[ "${1:-}" == core/database/migrations/*.sql ]]; then
  SQLX_LOCK="/tmp/moosync_sqlx_prepare.lock"
  (
    flock 201
    bazel run --ui_event_filters=-info,-stdout --noshow_progress //tools:sqlx_prepare >&2 || true
  ) 201>"$SQLX_LOCK"
fi

exec "$WORKSPACE_DIR/tools/format.sh" \
  "$(find_rlocation "upstream_wrapper/rustfmt")" \
  "$(find_rlocation "slint-lsp")" \
  "$(find_rlocation "buildifier/buildifier")" \
  "$(find_rlocation "rules_buf_toolchains/buf")" \
  --fix "$1"
