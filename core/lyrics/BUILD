load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")

rust_library(
    name = "lyrics",
    srcs = [
        "src/error.rs",
        "src/lib.rs",
        "src/lib_test.rs",
        "src/lib_test_smoke.rs",
    ],
    crate_root = "src/lib.rs",
    proc_macro_deps = [
        "//core/plugin_macro",
    ],
    visibility = ["//visibility:public"],
    deps = [
        "//core/types",
        "//core/types/protos:songs_rs",
        "@crates//:regex",
        "@crates//:reqwest",
        "@crates//:serde_json",
        "@crates//:tempdir",
        "@crates//:thiserror",
        "@crates//:tokio",
        "@crates//:tracing",
    ],
)

rust_test(
    name = "lyrics_test",
    crate = ":lyrics",
    deps = [
        "@crates//:wiremock",
    ],
)
