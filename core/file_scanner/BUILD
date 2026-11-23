load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")

rust_library(
    name = "file_scanner",
    srcs = [
        "src/context/android.rs",
        "src/context/desktop/image_processor.rs",
        "src/context/desktop/image_processor_test.rs",
        "src/context/desktop/lyrics_scanner.rs",
        "src/context/desktop/lyrics_scanner_test.rs",
        "src/context/desktop/mod.rs",
        "src/context/desktop/mod_test.rs",
        "src/context/desktop/playlist_scanner.rs",
        "src/context/desktop/playlist_scanner_test.rs",
        "src/context/desktop/song_scanner.rs",
        "src/context/desktop/song_scanner_test.rs",
        "src/context/mod.rs",
        "src/error.rs",
        "src/lib.rs",
        "src/lib_test.rs",
        "src/lib_test_smoke.rs",
    ],
    crate_root = "src/lib.rs",
    proc_macro_deps = [
        "//core/plugin_macro",
        "@crates//:async-trait",
    ],
    visibility = ["//visibility:public"],
    deps = [
        "//core/cache",
        "//core/types",
        "//core/types/protos:android_scanner_rs",
        "//core/types/protos:songs_rs",
        "@crates//:blake3",
        "@crates//:dunce",
        "@crates//:fast_image_resize",
        "@crates//:futures",
        "@crates//:image",
        "@crates//:lazy_static",
        "@crates//:lofty",
        "@crates//:num_cpus",
        "@crates//:prost",
        "@crates//:regex",
        "@crates//:substring",
        "@crates//:thiserror",
        "@crates//:tokio",
        "@crates//:tracing",
        "@crates//:uuid",
    ] + select({
        "@platforms//os:android": [
            "@crates//:jni",
        ],
        "//conditions:default": [],
    }),
)

rust_test(
    name = "file_scanner_test",
    crate = ":file_scanner",
    deps = [
        "@crates//:assertables",
        "@crates//:mockall",
        "@crates//:rstest",
        "@crates//:tempdir",
        "@crates//:tracing-test",
    ],
)
