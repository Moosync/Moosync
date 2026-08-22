load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")

rust_library(
    name = "mpris",
    srcs = [
        "src/context/mod.rs",
        "src/context/mod_test.rs",
        "src/error.rs",
        "src/lib.rs",
        "src/lib_test.rs",
        "src/mpris_android.rs",
        "src/win32.rs",
    ],
    crate_root = "src/lib.rs",
    proc_macro_deps = [
        "//core/plugin_macro",
    ],
    visibility = ["//visibility:public"],
    deps = [
        "//core/types",
        "//core/types/protos:extensions_rs",
        "//core/types/protos:songs_rs",
        "@crates//:thiserror",
        "@crates//:mockall",
        "@crates//:serde",
        "@crates//:tracing",
    ] + select({
        "@platforms//os:android": [
            "@crates//:jni",
        ],
        "@platforms//os:windows": [
            "@crates//:is_wine",
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
