load("@rules_rust//rust:defs.bzl", "rust_library")

rust_library(
    name = "pref_gen",
    srcs = ["src/lib.rs"],
    compile_data = ["Cargo.toml"],
    visibility = ["//visibility:public"],
    deps = [
        "//core/types:types_ui",
        "@wasmCrates//:proc-macro2",
        "@wasmCrates//:quote",
        "@wasmCrates//:serde_yaml",
        "@wasmCrates//:syn",
        "@wasmCrates//:tracing",
    ],
)
