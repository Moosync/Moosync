load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")

package(default_visibility = ["//visibility:public"])

rust_library(
    name = "types",
    srcs = [
        "src/canvaz.rs",
        "src/common.rs",
        "src/entities.rs",
        "src/errors/mod.rs",
        "src/extensions.rs",
        "src/lib.rs",
        "src/mpris/mod.rs",
        "src/preferences.rs",
        "src/providers/generic.rs",
        "src/providers/mod.rs",
        "src/songs.rs",
        "src/tests.rs",
        "src/ui/extensions.rs",
        "src/ui/mod.rs",
        "src/ui/player_details.rs",
        "src/ui/song_details.rs",
        "src/ui/themes.rs",
        "src/ui/updater.rs",
        "src/window.rs",
    ],
    proc_macro_deps = [
        "@crates//:async-trait",
    ],
    visibility = ["//visibility:public"],
    deps =
        [
            "//core/types/protos:songs_rs",
            "//core/types/protos:themes_rs",
            "@crates//:bitcode",
            "@crates//:serde",
            "@crates//:serde_json",
            "@crates//:thiserror",
            "@crates//:tracing",
        ],
)

rust_test(
    name = "types_test",
    crate = ":types",
)

rust_library(
    name = "types_ui",
    srcs = [
        "src/canvaz.rs",
        "src/common.rs",
        "src/entities.rs",
        "src/errors/mod.rs",
        "src/extensions.rs",
        "src/lib.rs",
        "src/mpris/mod.rs",
        "src/preferences.rs",
        "src/providers/generic.rs",
        "src/providers/mod.rs",
        "src/songs.rs",
        "src/ui/extensions.rs",
        "src/ui/mod.rs",
        "src/ui/player_details.rs",
        "src/ui/song_details.rs",
        "src/ui/themes.rs",
        "src/ui/updater.rs",
        "src/window.rs",
    ],
    crate_name = "types",
    proc_macro_deps = ["@wasmCrates//:async-trait"],
    target_compatible_with = ["@platforms//cpu:wasm32"],
    visibility = ["//visibility:public"],
    deps = [
        "//core/types/protos:songs_rs",
        "//core/types/protos:themes_rs",
        "@wasmCrates//:bitcode",
        "@wasmCrates//:serde",
        "@wasmCrates//:serde_json",
        "@wasmCrates//:thiserror",
        "@wasmCrates//:tracing",
    ],
)
