load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")

rust_test(
    name = "extensions_test",
    crate = ":extensions",
    data = [
        "tests/fixtures/package.json",
        "@moosync_ext//sample_extensions/go:sample_extension",
        "@moosync_ext//sample_extensions/py:sample_extension",
        "@moosync_ext//sample_extensions/rs:sample_extension",
    ],
    deps = [
        "@crates//:wiremock",
    ],
)

rust_test(
    name = "extensions_integration_test",
    timeout = "long",
    crate = ":extensions",
    crate_features = ["wasm_integration_tests"],
    data = [
        "tests/fixtures/package.json",
        "@moosync_ext//sample_extensions/go:sample_extension",
        "@moosync_ext//sample_extensions/py:sample_extension",
        "@moosync_ext//sample_extensions/rs:sample_extension",
    ],
    deps = [
        "@crates//:wiremock",
    ],
)

rust_library(
    name = "extensions",
    srcs = [
        "src/context/extism_context.rs",
        "src/context/extism_context_test.rs",
        "src/context/mod.rs",
        "src/context/mod_test.rs",
        "src/errors.rs",
        "src/ext_runner.rs",
        "src/ext_runner_test.rs",
        "src/extension.rs",
        "src/extension_test.rs",
        "src/lib.rs",
        "src/lib_test.rs",
        "src/lib_test_smoke.rs",
        "src/models.rs",
        "src/models_test.rs",
        "src/remote.rs",
        "src/remote_test.rs",
        "src/remote_test_smoke.rs",
        "src/sample_tests.rs",
        "src/tests.rs",
    ],
    crate_root = "src/lib.rs",
    proc_macro_deps = [
        "@crates//:async-trait",
        "//core/plugin_macro",
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
        "@crates//:regex",
        "@crates//:reqwest",
        "@crates//:rust-crypto",
        "@crates//:serde",
        "@crates//:serde_json",
        "@crates//:tempdir",
        "@crates//:thiserror",
        "@crates//:tokio",
        "@crates//:tracing",
        "@crates//:tracing-subscriber",
        "@crates//:uuid",
        "@crates//:zip-extensions",
    ],
)
