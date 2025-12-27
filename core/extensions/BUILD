load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")

rust_test(
    name = "extensions_test",
    crate = ":extensions",
)

rust_library(
    name = "extensions",
    srcs = [
        "src/ext_runner.rs",
        "src/lib.rs",
        "src/models.rs",
        "src/tests.rs",
    ],
    crate_root = "src/lib.rs",
    visibility = ["//visibility:public"],
    deps = [
        "//core/types",
        "@crates//:extism",
        "@crates//:extism-convert",
        "@crates//:fs_extra",
        "@crates//:futures",
        "@crates//:interprocess",
        "@crates//:regex",
        "@crates//:reqwest",
        "@crates//:rust_crypto",
        "@crates//:serde",
        "@crates//:serde_json",
        "@crates//:tokio",
        "@crates//:tracing",
        "@crates//:uuid",
        "@crates//:zip-extensions",
    ],
)
