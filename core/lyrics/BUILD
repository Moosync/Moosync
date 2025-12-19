load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")

rust_library(
    name = "lyrics",
    srcs = [
        "src/lib.rs",
        "src/test.rs",
    ],
    visibility = ["//visibility:public"],
    deps = [
        "//core/librespot",
        "//core/types",
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
