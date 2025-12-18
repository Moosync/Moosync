load("@rules_rust//rust:defs.bzl", "rust_library")

package(default_visibility = ["//visibility:public"])

# gazelle:rust_cargo_lockfile Cargo.lock
# gazelle:rust_crates_prefix @crates//:
# gazelle:rust_mode generate_from_cargo

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
    proc_macro_deps = ["@crates//:async-trait"],
    visibility = ["//visibility:public"],
    deps =
        select({
            ":is_wasm": [
                "@wasmCrates//:bitcode",
                "@wasmCrates//:serde",
                "@wasmCrates//:serde_json",
                "@wasmCrates//:thiserror",
                "@wasmCrates//:tracing",
            ],
            "//conditions:default": [
                "@crates//:bitcode",
                "@crates//:serde",
                "@crates//:serde_json",
                "@crates//:thiserror",
                "@crates//:tracing",
            ],
        }),
)
