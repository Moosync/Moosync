load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")

package(default_visibility = ["//visibility:public"])

rust_library(
    name = "logging",
    srcs = [
        "src/lib.rs",
        "src/lib_test.rs",
    ],
    crate_root = "src/lib.rs",
    proc_macro_deps = [
        "//core/plugin_macro",
    ],
    deps = [
        "//core/cache",
        "//core/types",
        "@crates//:platform-dirs",
        "@crates//:tracing",
        "@crates//:tracing-appender",
        "@crates//:tracing-subscriber",
    ] + select({
        "@platforms//os:android": [],
        "//conditions:default": [
            "@crates//:tracing-tracy",
        ],
    }),
)

rust_test(
    name = "logging_test",
    crate = ":logging",
    deps = [
        "@crates//:rstest",
        "@crates//:tempdir",
        "@crates//:tracing-test",
    ],
)
