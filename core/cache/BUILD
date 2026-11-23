load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")

package(default_visibility = ["//visibility:public"])

rust_library(
    name = "cache",
    srcs = [
        "src/backend.rs",
        "src/cache_test.rs",
        "src/error.rs",
        "src/lib.rs",
        "src/manager.rs",
        "src/models.rs",
    ],
    crate_root = "src/lib.rs",
    proc_macro_deps = [
        "@crates//:async-trait",
    ],
    deps = [
        "@crates//:thiserror",
        "@crates//:tokio",
        "@crates//:tracing",
    ],
)

rust_test(
    name = "cache_test",
    crate = ":cache",
    deps = [
        "@crates//:assertables",
        "@crates//:rstest",
        "@crates//:tempdir",
        "@crates//:tracing-test",
    ],
)
