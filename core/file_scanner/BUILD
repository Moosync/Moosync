load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")

rust_library(
    name = "file_scanner",
    srcs = [
        "src/lib.rs",
        "src/playlist_scanner.rs",
        "src/scanner.rs",
        "src/song_scanner.rs",
        "src/tests.rs",
        "src/types.rs",
        "src/utils.rs",
    ],
    visibility = ["//visibility:public"],
    deps = [
        "//core/types",
        "@crates//:blake3",
        "@crates//:dunce",
        "@crates//:fast_image_resize",
        "@crates//:image",
        "@crates//:lazy_static",
        "@crates//:lofty",
        "@crates//:num_cpus",
        "@crates//:regex",
        "@crates//:substring",
        "@crates//:threadpool",
        "@crates//:tracing",
        "@crates//:uuid",
    ],
)

rust_test(
    name = "file_scanner_test",
    crate = ":file_scanner",
)
