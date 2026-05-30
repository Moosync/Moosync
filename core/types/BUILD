load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")

package(default_visibility = ["//visibility:public"])

rust_library(
    name = "types",
    srcs = [
        "src/android/mod.rs",
        "src/canvaz.rs",
        "src/errors/mod.rs",
        "src/lib.rs",
        "src/mpris/mod.rs",
        "src/plugin.rs",
        "src/preferences.rs",
        "src/providers/generic.rs",
        "src/providers/mod.rs",
        "src/tests.rs",
        "src/ui/mod.rs",
        "src/ui/player_details.rs",
        "src/ui/updater.rs",
        "src/window.rs",
    ],
    proc_macro_deps = [
        "@crates//:async-trait",
    ],
    visibility = ["//visibility:public"],
    deps =
        [
            "//core/types/protos:extensions_rs",
            "//core/types/protos:songs_rs",
            "//core/types/protos:themes_rs",
            "@crates//:bitcode",
            "@crates//:prost",
            "@crates//:serde",
            "@crates//:serde_json",
            "@crates//:thiserror",
            "@crates//:tokio",
            "@crates//:tracing",
        ] + select({
            "//:android_x86_64_config": ["@crates//:jni"],
            "//:android_arm64_config": ["@crates//:jni"],
            "//conditions:default": [],
        }),
)

rust_test(
    name = "types_test",
    crate = ":types",
)
