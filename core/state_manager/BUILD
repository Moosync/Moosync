load("@rules_rust//rust:defs.bzl", "rust_library")

rust_library(
    name = "state_manager",
    srcs = [
        "src/lib.rs",
    ],
    crate_root = "src/lib.rs",
    visibility = ["//visibility:public"],
    deps = [
        "//core/types",
        "//core/database",
        "//core/extensions",
        "//core/file_scanner",
        "//core/lyrics",
        "//core/preferences",
        "//core/types/protos:extensions_rs",
        "//core/rodio_player",
        "//core/queue_manager",
        # "//core/spotify_player",
        "//core/themes",
        "@crates//:tokio",
        "@crates//:thiserror",
        "@crates//:platform-dirs",
        "@crates//:tempdir",
        "@crates//:tracing",
    ] + select({
        "@platforms//os:android": [
            "//core/mpris",
            "@crates//:jni",
        ],
        "//conditions:default": ["//core/mpris"],
    }),
)
