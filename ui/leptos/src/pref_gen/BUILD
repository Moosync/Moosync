load("@rules_rust//rust:defs.bzl", "rust_library")

rust_library(
    name = "pref_gen",
    srcs = ["src/lib.rs"],
    visibility = ["//visibility:public"],
    deps = [
        "//core/types/protos:extensions_rs",
        "//core/types/protos:ui_rs",
        "@crates//:proc-macro2",
        "@crates//:quote",
        "@crates//:serde",
        "@crates//:serde_json",
        "@crates//:serde_yaml",
        "@crates//:syn",
        "@crates//:tracing",
    ],
)
