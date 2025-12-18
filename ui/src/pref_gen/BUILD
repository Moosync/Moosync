load("@rules_rust//rust:defs.bzl", "rust_library")

rust_library(
    name = "pref_gen",
    srcs = ["src/lib.rs"],
    compile_data = ["Cargo.toml"],
    visibility = ["//visibility:public"],
    deps = [
        "//core/types",
        "@crates//:proc-macro2",
        "@crates//:quote",
        "@crates//:serde_yaml",
        "@crates//:syn",
        "@crates//:tracing",
    ],
)
