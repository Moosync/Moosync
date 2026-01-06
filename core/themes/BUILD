load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")

rust_library(
    name = "themes",
    srcs = [
        "src/lib.rs",
        "src/test.rs",
        "src/themes.rs",
    ],
    crate_root = "src/lib.rs",
    visibility = ["//visibility:public"],
    deps = [
        "//core/types",
        "//core/types/protos:themes_rs",
        "@crates//:fs_extra",
        "@crates//:futures",
        "@crates//:notify",
        "@crates//:regex",
        "@crates//:reqwest",
        "@crates//:serde",
        "@crates//:serde_json",
        "@crates//:tracing",
        "@crates//:uuid",
        "@crates//:zip-extensions",
    ],
)

rust_test(
    name = "themes_test",
    crate = ":themes",
)
