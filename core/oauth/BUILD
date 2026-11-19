load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")

rust_library(
    name = "oauth",
    srcs = [
        "src/lib.rs",
        "src/lib_test.rs",
    ],
    crate_root = "src/lib.rs",
    proc_macro_deps = [
        "//core/plugin_macro",
    ],
    visibility = ["//visibility:public"],
    deps = [
        "//core/types",
        "@crates//:tracing",
        "@crates//:url",
    ],
)

rust_test(
    name = "oauth_test",
    crate = ":oauth",
    deps = [
        "@crates//:assertables",
        "@crates//:tracing-test",
    ],
)
