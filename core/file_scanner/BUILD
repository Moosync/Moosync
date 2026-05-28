load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")

rust_library(
    name = "file_scanner",
    srcs = [
        "src/context/android.rs",
        "src/context/desktop.rs",
        "src/context/mod.rs",
        "src/lib.rs",
        "src/scanner.rs",
        "src/tests.rs",
        "src/types.rs",
    ],
    crate_root = "src/lib.rs",
    visibility = ["//visibility:public"],
    deps = [
        "//core/types",
        "//core/types/protos:songs_rs",
        "@crates//:blake3",
        "@crates//:dunce",
        "@crates//:futures",
        "@crates//:fast_image_resize",
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
