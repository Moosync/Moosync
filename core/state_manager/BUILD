load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")

rust_test(
    name = "state_manager_test",
    crate = ":state_manager",
)

rust_library(
    name = "state_manager",
    srcs = [
        "src/error.rs",
        "src/hooks/extensions.rs",
        "src/hooks/extensions_test.rs",
        "src/hooks/mod.rs",
        "src/hooks/mod_test.rs",
        "src/hooks/player.rs",
        "src/hooks/player_test.rs",
        "src/hooks/scanner.rs",
        "src/hooks/scanner_test.rs",
        "src/interceptors/mod.rs",
        "src/lib.rs",
        "src/lib_test.rs",
        "src/reply_handler.rs",
        "src/reply_handler_test.rs",
    ],
    crate_root = "src/lib.rs",
    proc_macro_deps = [
        "//core/plugin_macro",
        "@crates//:async-trait",
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
        "//core/types/protos:ui_rs",
        "//core/player",
        # "//core/spotify_player",
        "//core/themes",
        "@crates//:tokio",
        "@crates//:thiserror",
        "@crates//:platform-dirs",
        "@crates//:tempdir",
        "@crates//:tracing",
        "@crates//:lru",
        "@crates//:serde",
        "@crates//:serde_json",
        "@crates//:prost-types",
        "@crates//:uuid",
        "//core/mpris",
    ] + select({
        "@platforms//os:android": [
            "@crates//:jni",
        ],
        "//conditions:default": [],
    }),
)
