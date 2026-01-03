load("@rules_rust//rust:defs.bzl", "rust_library")

rust_library(
    name = "pref_gen",
    srcs = ["src/lib.rs"],
    visibility = ["//visibility:public"],
    deps = [
        "@wasmCrates//:proc-macro2",
        "@wasmCrates//:quote",
        "@wasmCrates//:serde",
        "@wasmCrates//:serde_json",
        "@wasmCrates//:serde_yaml",
        "@wasmCrates//:syn",
        "@wasmCrates//:tracing",
    ],
)
