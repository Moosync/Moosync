load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")

rust_library(
    name = "mpris",
    srcs = [
        "src/lib.rs",
        "src/mpris.rs",
        "src/tests.rs",
        "src/context/mod.rs",
    ],
    crate_root = "src/lib.rs",
    visibility = ["//visibility:public"],
    deps = [
        "//core/types",
        "@crates//:souvlaki",
        "@crates//:tracing",
        "@crates//:windows",
        "@crates//:mockall",
    ],
)

rust_test(
    name = "mpris_test",
    crate = ":mpris",
)
