load("@rules_rust//rust:defs.bzl", "rust_library")

rust_library(
    name = "mpris",
    srcs = [
        "src/lib.rs",
        "src/mpris.rs",
    ],
    visibility = ["//visibility:public"],
    deps = [
        "//core/types",
        "@crates//:souvlaki",
        "@crates//:tracing",
        "@crates//:windows",
    ],
)
