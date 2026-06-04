load("@rules_rust//cargo:defs.bzl", "cargo_build_script")
load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_library", "rust_shared_library")
load("//:defs.bzl", "platform_transition_rule")

package(default_visibility = ["//visibility:public"])

cargo_build_script(
    name = "build_script",
    srcs = ["build.rs"],
    data = glob([
        "src/**/*.slint",
        "src/**/*.svg",
    ]),
    deps = ["@crates//:slint-build"],  # Make sure slint-build is in your Cargo.toml/crates list
    build_script_env = {
        "CARGO_PKG_VERSION": "1.17.0",
        "CARGO_PKG_VERSION_MAJOR": "1",
        "CARGO_PKG_VERSION_MINOR": "17",
        "CARGO_PKG_VERSION_PATCH": "0",
    },
)

rust_library(
    name = "slint_app_lib",
    srcs = [
        "src/lib.rs",
        "src/main_content/albums.rs",
        "src/main_content/all_songs.rs",
        "src/main_content/artists.rs",
        "src/main_content/explore.rs",
        "src/main_content/genres.rs",
        "src/main_content/mod.rs",
        "src/main_content/playlists.rs",
        "src/pages.rs",
        "src/settings/extensions.rs",
        "src/settings/mod.rs",
        "src/settings/paths.rs",
        "src/settings/system.rs",
        "src/settings/themes.rs",
        "src/utils.rs",
        "src/window_info.rs",
    ],
    compile_data = glob([
        "src/**/*.slint",
        "src/**/*.svg",
        "src/**/*.yaml",
    ]),
    crate_name = "slint_app",
    edition = "2021",
    proc_macro_deps = [
        "//ui/pref_macro",
        "//core/theme_macro",
    ],
    deps = [
        ":build_script",  # Exposes the generated code
        "//core/player",
        "//core/preferences",
        "//core/state_manager",
        "//core/themes",
        "//core/types",
        "//core/types/protos:extensions_rs",
        "//core/types/protos:songs_rs",
        "//core/types/protos:themes_rs",
        "@crates//:platform-dirs",
        "@crates//:reqwest",
        "@crates//:rfd",
        "@crates//:slint",
        "@crates//:tokio",
        "@crates//:tracing",
        "@crates//:tracing-appender",
        "@crates//:tracing-subscriber",
        "@crates//:uuid",
    ],
)

rust_binary(
    name = "slint_app",
    srcs = ["src/main.rs"],
    edition = "2021",
    rustc_flags = select({
        "@platforms//os:windows": [
            "-Clink-args=-Wl,--start-group -lmsvcrt -lmingw32 -lmingwex -luser32 -lkernel32 -lws2_32 -lcrypt32 -ladvapi32 -Wl,--end-group",
        ],
        "//conditions:default": [],
    }),
    deps = [
        ":slint_app_lib",
        "@crates//:tokio",
    ],
)

# Android Shared Library target — compiled as cdylib for NativeActivity.
# Uses src/lib.rs (same as the desktop lib) with the full dep tree so that
# state_manager and tracing are present on Android.
rust_shared_library(
    name = "slint_app_android",
    srcs = [
        "src/lib.rs",
        "src/main_content/albums.rs",
        "src/main_content/all_songs.rs",
        "src/main_content/artists.rs",
        "src/main_content/explore.rs",
        "src/main_content/genres.rs",
        "src/main_content/mod.rs",
        "src/main_content/playlists.rs",
        "src/pages.rs",
        "src/settings/extensions.rs",
        "src/settings/mod.rs",
        "src/settings/paths.rs",
        "src/settings/system.rs",
        "src/settings/themes.rs",
        "src/utils.rs",
        "src/window_info.rs",
    ],
    compile_data = glob([
        "src/**/*.slint",
        "src/**/*.svg",
        "src/**/*.yaml",
    ]),
    crate_name = "slint_app",
    edition = "2021",
    proc_macro_deps = [
        "//ui/pref_macro",
        "//core/theme_macro",
    ],
    deps = [
        ":build_script",
        "//core/player",
        "//core/preferences",
        "//core/state_manager",
        "//core/themes",
        "//core/types",
        "//core/types/protos:extensions_rs",
        "//core/types/protos:songs_rs",
        "//core/types/protos:themes_rs",
        "@crates//:jni",
        "@crates//:platform-dirs",
        "@crates//:reqwest",
        "@crates//:rfd",
        "@crates//:slint",
        "@crates//:tokio",
        "@crates//:tracing",
        "@crates//:tracing-appender",
        "@crates//:tracing-subscriber",
        "@crates//:uuid",
    ],
)

# Transition rules for Android platforms
platform_transition_rule(
    name = "slint_app_android_x86_64",
    src = ":slint_app_android",
    platform = "//:x86_64",
)

platform_transition_rule(
    name = "slint_app_android_aarch64",
    src = ":slint_app_android",
    platform = "//:arm64-v8a",
)
