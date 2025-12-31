#!/bin/bash
set -o errexit

PREFIX='{"reason":"compiler-message","package_id":"","target":{"kind":[""],"crate_types":[""],"name":"","src_path":"","edition":"2021","doc":true,"doctest":true,"test":true},"message":'
SUFFIX='}'
OUTPUT_BASE="/tmp/bazel-rust-analyzer"
SAVED_FILE="$1"
PATH_PREFIX="$2"
run_analyzer() {
  FILE_PATH="${SAVED_FILE/#"${PATH_PREFIX}/"}"
  FILE_TARGET=$(bazel query "${FILE_PATH}")
  BAZEL_TARGET=$(bazel query "attr('srcs', ${FILE_TARGET}, ${FILE_TARGET//:*/}:*)")
  set +o errexit
  rustfmt "${FILE_PATH}"
  # can substitute bazel -> builders/tools/bazel-debian (slight performance hit)
  bazel --output_base="${OUTPUT_BASE}" build --@rules_rust//rust/settings:error_format=json "${BAZEL_TARGET}"
  RC=$?
  set -o errexit
  if [[ "$RC" -ne 0 ]]; then
    STD_ERR_DIR="${OUTPUT_BASE}/execroot/_main/bazel-out/_tmp/actions"
    while read -r line; do
      echo "${PREFIX}$line${SUFFIX}"
    done <<< "$(cat ${STD_ERR_DIR}/stderr-*)"
  fi
}

run_analyzer