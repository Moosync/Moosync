load("@rules_rust//rust:defs.bzl", "rust_library")

package(default_visibility = ["//visibility:public"])

config_setting(
    name = "is_wasm",
    constraint_values = [
        "@platforms//cpu:wasm32",
    ],
)

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
        "src/themes.rs",
        "src/ui/extensions.rs",
        "src/ui/mod.rs",
        "src/ui/player_details.rs",
        "src/ui/song_details.rs",
        "src/ui/themes.rs",
        "src/ui/updater.rs",
        "src/window.rs",
    ],
    compile_data = ["Cargo.toml"],
    proc_macro_deps = [
        "@crates//:async-trait",
    ],
    visibility = ["//visibility:public"],
    deps =
        [
            "@crates//:bitcode",
            "@crates//:serde",
            "@crates//:serde_json",
            "@crates//:thiserror",
            "@crates//:tracing",
        ],
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
        "src/themes.rs",
        "src/ui/extensions.rs",
        "src/ui/mod.rs",
        "src/ui/player_details.rs",
        "src/ui/song_details.rs",
        "src/ui/themes.rs",
        "src/ui/updater.rs",
        "src/window.rs",
    ],
    compile_data = ["Cargo.toml"],
    crate_name = "types",
    proc_macro_deps = ["@wasmCrates//:async-trait"],
    visibility = ["//visibility:public"],
    deps = [
        "@wasmCrates//:bitcode",
        "@wasmCrates//:serde",
        "@wasmCrates//:serde_json",
        "@wasmCrates//:thiserror",
        "@wasmCrates//:tracing",
    ],
)

# rust_library(
#     name = "types_gen",
#     srcs = [
#         "src/canvaz.rs",
#         "src/common.rs",
#         "src/entities.rs",
#         "src/errors/mod.rs",
#         "src/extensions.rs",
#         "src/lib.rs",
#         "src/mpris/mod.rs",
#         "src/preferences.rs",
#         "src/providers/generic.rs",
#         "src/providers/mod.rs",
#         "src/songs.rs",
#         "src/themes.rs",
#         "src/ui/extensions.rs",
#         "src/ui/mod.rs",
#         "src/ui/player_details.rs",
#         "src/ui/song_details.rs",
#         "src/ui/themes.rs",
#         "src/ui/updater.rs",
#         "src/window.rs",
#     ],
#     compile_data = ["Cargo.toml"],
#     crate_name = "types",
#     proc_macro_deps = ["@macroCrates//:async-trait"],
#     visibility = ["//visibility:public"],
#     deps = [
#         "@macroCrates//:bitcode",
#         "@macroCrates//:serde",
#         "@macroCrates//:serde_json",
#         "@macroCrates//:thiserror",
#         "@macroCrates//:tracing",
#     ],
# )
