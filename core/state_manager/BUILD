load("@rules_rust//rust:defs.bzl", "rust_library")

rust_library(
    name = "state_manager",
    srcs = glob(["src/**/*.rs"]),
    crate_root = "src/lib.rs",
    proc_macro_deps = [
        "//core/plugin_macro",
    ],
    visibility = ["//visibility:public"],
    deps = [
        "//core/types",
        "//core/database",
        "//core/extensions",
        "//core/file_scanner",
        "//core/lyrics",
        "//core/preferences",
        "//core/types/protos:extensions_rs",
        "//core/types/protos:songs_rs",
        "//core/player",
        # "//core/spotify_player",
        "//core/themes",
        "@crates//:tokio",
        "@crates//:thiserror",
        "@crates//:platform-dirs",
        "@crates//:tempdir",
        "@crates//:tracing",
        "@crates//:lru",
        "//core/mpris",
    ] + select({
        "@platforms//os:android": [
            "@crates//:jni",
        ],
        "//conditions:default": [],
    }),
)
