#!/bin/bash
# Wrapper for wrapped_clang that ensures DEVELOPER_DIR is absolute.
# This fixes cc-rs cargo build scripts which run from a different working directory.

set -eu

# Find our own location (this script should be next to where wrapped_clang lives)
MY_LOCATION="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# The actual wrapped_clang is in the apple_cross_toolchain
# This wrapper is called from the execroot, so we can compute the absolute path
if [[ -z "${DEVELOPER_DIR:-}" ]]; then
    # Compute DEVELOPER_DIR relative to EXT_BUILD_ROOT if available, otherwise from execroot
    if [[ -n "${EXT_BUILD_ROOT:-}" ]]; then
        export DEVELOPER_DIR="${EXT_BUILD_ROOT}/external/rules_applecross++apple_cross_toolchain+apple_cross_toolchain/Xcode.app/Contents/Developer"
    elif [[ -n "${BUILD_WORKING_DIRECTORY:-}" ]]; then
        # Bazel sets this for run commands
        export DEVELOPER_DIR="${BUILD_WORKING_DIRECTORY}/external/rules_applecross++apple_cross_toolchain+apple_cross_toolchain/Xcode.app/Contents/Developer"
    else
        # Try to find execroot from /proc/self/cwd or PWD going up
        EXECROOT=""
        SEARCH_DIR="${PWD}"
        while [[ "${SEARCH_DIR}" != "/" ]]; do
            if [[ -d "${SEARCH_DIR}/external/rules_applecross++apple_cross_toolchain+apple_cross_toolchain" ]]; then
                EXECROOT="${SEARCH_DIR}"
                break
            fi
            SEARCH_DIR="$(dirname "${SEARCH_DIR}")"
        done
        if [[ -n "${EXECROOT}" ]]; then
            export DEVELOPER_DIR="${EXECROOT}/external/rules_applecross++apple_cross_toolchain+apple_cross_toolchain/Xcode.app/Contents/Developer"
        fi
    fi
fi

# Find the actual wrapped_clang binary
# It should be in the toolchain directory
WRAPPED_CLANG="${DEVELOPER_DIR%/Xcode.app/Contents/Developer}/wrapped_clang"

if [[ ! -x "${WRAPPED_CLANG}" ]]; then
    echo "Error: Cannot find wrapped_clang at ${WRAPPED_CLANG}" >&2
    exit 1
fi

exec "${WRAPPED_CLANG}" "$@"
