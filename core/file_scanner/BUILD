load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")

rust_library(
    name = "file_scanner",
    srcs = [
        "src/context/android.rs",
        "src/context/desktop/mod.rs",
        "src/context/desktop/image_processor.rs",
        "src/context/desktop/lyrics_scanner.rs",
        "src/context/desktop/playlist_scanner.rs",
        "src/context/desktop/song_scanner.rs",
        "src/context/mod.rs",
        "src/lib.rs",
        "src/tests.rs",
    ],
    crate_root = "src/lib.rs",
    proc_macro_deps = [
        "//core/plugin_macro",
    ],
    visibility = ["//visibility:public"],
    deps = [
        "//core/types",
        "//core/types/protos:songs_rs",
        "@crates//:blake3",
        "@crates//:dunce",
        "@crates//:fast_image_resize",
        "@crates//:futures",
        "@crates//:image",
        "@crates//:lazy_static",
        "@crates//:lofty",
        "@crates//:num_cpus",
        "@crates//:regex",
        "@crates//:substring",
        "@crates//:tokio",
        "@crates//:tracing",
        "@crates//:uuid",
    ],
)

rust_test(
    name = "file_scanner_test",
    crate = ":file_scanner",
)
