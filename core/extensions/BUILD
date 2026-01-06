load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")

rust_test(
    name = "extensions_test",
    crate = ":extensions",
    data = glob(["tests/fixtures/**"]),
)

rust_library(
    name = "extensions",
    srcs = [
        "src/context/extism_context.rs",
        "src/context/mod.rs",
        "src/errors.rs",
        "src/ext_runner.rs",
        "src/lib.rs",
        "src/models.rs",
        "src/sample_tests.rs",
        "src/tests.rs",
    ],
    crate_root = "src/lib.rs",
    proc_macro_deps = [
        "@crates//:async-trait",
    ],
    visibility = ["//visibility:public"],
    deps = [
        "//core/types",
        "//core/types/protos:extensions_rs",
        "//core/types/protos:songs_rs",
        "//core/types/protos:ui_rs",
        "@crates//:extism",
        "@crates//:extism-convert",
        "@crates//:fs_extra",
        "@crates//:futures",
        "@crates//:interprocess",
        "@crates//:mockall",
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
