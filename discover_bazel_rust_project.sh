#!/usr/bin/bash

bazel \
    run \
    @rules_rust//tools/rust_analyzer:discover_bazel_rust_project -- \
    --bazel_arg=--watchfs \
    ${1:+"$1"} 2>/dev/null