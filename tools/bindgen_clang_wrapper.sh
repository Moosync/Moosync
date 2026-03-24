#!/bin/bash
# Wrapper for clang that adds macOS SDK sysroot for bindgen cross-compilation
# DEVELOPER_DIR should be set by the build script env

set -eu

# Construct SDKROOT from DEVELOPER_DIR
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

    # Last resort: on native macOS, use xcrun to find SDK; otherwise run without sysroot
    if command -v xcrun &>/dev/null; then
        NATIVE_SDK="$(xcrun --show-sdk-path 2>/dev/null)" || true
        if [[ -n "${NATIVE_SDK}" && -d "${NATIVE_SDK}" ]]; then
            exec "${CLANG}" -isysroot "${NATIVE_SDK}" "$@"
        fi
    fi
    exec "${CLANG}" "$@"
fi
