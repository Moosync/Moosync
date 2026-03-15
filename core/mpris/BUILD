load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")

rust_library(
    name = "mpris",
    srcs = [
        "src/context/mod.rs",
        "src/lib.rs",
        "src/mpris.rs",
        "src/mpris_android.rs",
        "src/tests.rs",
        "src/win32.rs",
    ],
    crate_root = "src/lib.rs",
    visibility = ["//visibility:public"],
    deps = [
        "//core/types",
        "//core/types/protos:extensions_rs",
        "//core/types/protos:songs_rs",
        "@crates//:mockall",
        "@crates//:serde",
        "@crates//:tracing",
    ] + select({
        "@platforms//os:android": [
            "//lib/tauri-plugin-audioplayer",
            "@crates//:serde_json",
            "@crates//:tauri",
        ],
        "@platforms//os:windows": [
            "@crates//:souvlaki",
            "@crates//:windows",
        ],
        "//conditions:default": [
            "@crates//:souvlaki",
        ],
    }),
)

rust_test(
    name = "mpris_test",
    crate = ":mpris",
)
