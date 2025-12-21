load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")

rust_library(
    name = "themes",
    srcs = [
        "src/lib.rs",
        "src/test.rs",
        "src/themes.rs",
    ],
    compile_data = ["Cargo.toml"],
    visibility = ["//visibility:public"],
    deps = [
        "//core/types",
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
    compile_data = ["Cargo.toml"],
    crate = ":themes",
)
