load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")

rust_library(
    name = "themes",
    srcs = [
        "src/error.rs",
        "src/error_test.rs",
        "src/lib.rs",
        "src/lib_test_smoke.rs",
        "src/themes.rs",
        "src/themes_test.rs",
    ],
    crate_root = "src/lib.rs",
    proc_macro_deps = [
        "//core/plugin_macro",
    ],
    visibility = ["//visibility:public"],
    deps = [
        "//core/types",
        "//core/types/protos:themes_rs",
        "@crates//:thiserror",
        "@crates//:fs_extra",
        "@crates//:futures",
        "@crates//:notify",
        "@crates//:regex",
        "@crates//:reqwest",
        "@crates//:serde",
        "@crates//:serde_json",
        "@crates//:tempdir",
        "@crates//:tracing",
        "@crates//:uuid",
        "@crates//:zip-extensions",
    ],
)

rust_test(
    name = "themes_test",
    crate = ":themes",
)
