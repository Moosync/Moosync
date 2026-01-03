load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")

rust_library(
    name = "mpris",
    srcs = [
        "src/context/mod.rs",
        "src/lib.rs",
        "src/mpris.rs",
        "src/tests.rs",
        "src/win32.rs",
    ],
    crate_root = "src/lib.rs",
    visibility = ["//visibility:public"],
    deps = [
        "//core/types",
        "//core/types/protos:songs_rs",
        "@crates//:mockall",
        "@crates//:serde",
        "@crates//:souvlaki",
        "@crates//:tracing",
        "@crates//:windows",
    ],
)

rust_test(
    name = "mpris_test",
    crate = ":mpris",
)
