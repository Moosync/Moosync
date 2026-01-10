load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")

rust_library(
    name = "lyrics",
    srcs = [
        "src/lib.rs",
        "src/test.rs",
    ],
    crate_root = "src/lib.rs",
    visibility = ["//visibility:public"],
    deps = [
        "//core/librespot",
        "//core/types",
        "//core/types/protos:songs_rs",
        "@crates//:regex",
        "@crates//:reqwest",
        "@crates//:serde_json",
        "@crates//:tracing",
    ],
)

rust_test(
    name = "lyrics_test",
    crate = ":lyrics",
)
