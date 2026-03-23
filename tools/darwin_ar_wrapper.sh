#!/bin/bash
# Wrapper script to translate GNU ar/libtool syntax to llvm tools
# For -static (create): uses llvm-libtool-darwin
# For x (extract): uses llvm-ar

TOOLCHAIN="${EXT_BUILD_ROOT}/external/rules_applecross++apple_cross_toolchain+apple_cross_toolchain/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin"
LIBTOOL_DARWIN="${TOOLCHAIN}/llvm-libtool-darwin"
LLVM_AR="${TOOLCHAIN}/llvm-ar"

# Scan all args to determine mode
USE_LIBTOOL=false
for arg in "$@"; do
    case "$arg" in
        -static)
            USE_LIBTOOL=true
            break
            ;;
        x)
            # Extract mode - use llvm-ar directly
            exec "$LLVM_AR" "$@"
            ;;
    esac
done

if $USE_LIBTOOL; then
    # Static library creation mode - use llvm-libtool-darwin
    # Parse arguments to convert syntax
    OUTPUT=""
    OBJECTS=()
    FLAGS=()

    while [[ $# -gt 0 ]]; do
        case "$1" in
            -static|-no_warning_for_no_symbols)
                FLAGS+=("$1")
                shift
                ;;
            -o)
                OUTPUT="$2"
                shift 2
                ;;
            *.a)
                # If we haven't seen -o yet, this is the output archive
                if [[ -z "$OUTPUT" ]]; then
                    OUTPUT="$1"
                else
                    OBJECTS+=("$1")
                fi
                shift
                ;;
            *)
                OBJECTS+=("$1")
                shift
                ;;
        esac
    done

    if [[ -z "$OUTPUT" ]]; then
        echo "Error: No output archive specified" >&2
        exit 1
    fi

    exec "$LIBTOOL_DARWIN" "${FLAGS[@]}" -o "$OUTPUT" "${OBJECTS[@]}"
else
    # Unknown mode - pass through to llvm-ar
    exec "$LLVM_AR" "$@"
fi
