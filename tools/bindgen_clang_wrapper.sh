#!/bin/bash
# Wrapper for clang that adds macOS SDK sysroot for bindgen cross-compilation
# DEVELOPER_DIR should be set by the build script env

set -eu

# On native macOS, just use system clang directly - no wrapper needed
if [[ "$(uname -s)" == "Darwin" ]]; then
    exec clang "$@"
fi

# Cross-compilation from Linux: construct SDKROOT from DEVELOPER_DIR
if [[ -n "${DEVELOPER_DIR:-}" ]]; then
    # Normalize the path (remove /.. components)
    DEVELOPER_DIR_REAL="$(cd "${DEVELOPER_DIR}" 2>/dev/null && pwd)" || DEVELOPER_DIR_REAL="${DEVELOPER_DIR}"
    SDKROOT="${DEVELOPER_DIR_REAL}/Platforms/MacOSX.platform/Developer/SDKs/MacOSX15.5.sdk"
fi

# Use system clang (bindgen's libclang needs flags, not a custom binary)
CLANG="clang"

# Add sysroot and target if SDKROOT is valid
if [[ -n "${SDKROOT:-}" && -d "${SDKROOT}" ]]; then
    exec "${CLANG}" --target=x86_64-apple-darwin -isysroot "${SDKROOT}" "$@"
else
    # Fallback: try to find SDK from script location
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    EXECROOT="${SCRIPT_DIR}"
    while [[ "${EXECROOT}" != "/" ]]; do
        if [[ -d "${EXECROOT}/external/rules_applecross++apple_cross_toolchain+apple_cross_toolchain" ]]; then
            break
        fi
        EXECROOT="$(dirname "${EXECROOT}")"
    done

    if [[ -d "${EXECROOT}/external/rules_applecross++apple_cross_toolchain+apple_cross_toolchain" ]]; then
        SDKROOT="${EXECROOT}/external/rules_applecross++apple_cross_toolchain+apple_cross_toolchain/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX15.5.sdk"
        if [[ -d "${SDKROOT}" ]]; then
            exec "${CLANG}" --target=x86_64-apple-darwin -isysroot "${SDKROOT}" "$@"
        fi
    fi

    # Last resort: run without sysroot (will likely fail on cross-compile)
    exec "${CLANG}" "$@"
fi
