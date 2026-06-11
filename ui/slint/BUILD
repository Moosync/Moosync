load("@rules_rust//cargo:defs.bzl", "cargo_build_script")
load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_library", "rust_shared_library")

package(default_visibility = ["//visibility:public"])

alias(
    name = "slint_app",
    actual = select({
        "@platforms//os:android": ":slint_app_android",
        "//conditions:default": ":slint_app_desktop",
    }),
)

cargo_build_script(
    name = "build_script",
    srcs = ["build.rs"],
    build_script_env = {
        "CARGO_PKG_VERSION": "1.17.0",
        "CARGO_PKG_VERSION_MAJOR": "1",
        "CARGO_PKG_VERSION_MINOR": "17",
        "CARGO_PKG_VERSION_PATCH": "0",
        "SLINT_ENABLE_EXPERIMENTAL_FEATURES": "1",
    },
    data = glob([
        "src/**/*.slint",
        "src/**/*.svg",
    ]),
    deps = ["@crates//:slint-build"],
)

rust_library(
    name = "slint_app_lib",
    srcs = [
        "src/lib.rs",
        "src/main_content/album_content.rs",
        "src/main_content/albums.rs",
        "src/main_content/all_songs.rs",
        "src/main_content/artist_content.rs",
        "src/main_content/artists.rs",
        "src/main_content/explore.rs",
        "src/main_content/genre_content.rs",
        "src/main_content/genres.rs",
        "src/main_content/mod.rs",
        "src/main_content/playlist_content.rs",
        "src/main_content/playlists.rs",
        "src/main_content/queue.rs",
        "src/main_content/search.rs",
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
        "@crates//:image",
        "@crates//:platform-dirs",
        "@crates//:reqwest",
        "@crates//:slint",
        "@crates//:tokio",
        "@crates//:tracing",
        "@crates//:tracing-appender",
        "@crates//:tracing-subscriber",
        "@crates//:uuid",
    ] + select({
        "@platforms//os:android": [],
        "//conditions:default": [
            "@crates//:rfd",
        ],
    }),
)

rust_binary(
    name = "slint_app_desktop",
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
    ] + select({
        "@platforms//os:windows": [],
        "@platforms//os:android": [],
        "//conditions:default": [
            "@crates//:tikv-jemallocator",
        ],
    }),
)

rust_shared_library(
    name = "slint_app_android",
    srcs = [
        "src/lib.rs",
        "src/main_content/album_content.rs",
        "src/main_content/albums.rs",
        "src/main_content/all_songs.rs",
        "src/main_content/artist_content.rs",
        "src/main_content/artists.rs",
        "src/main_content/explore.rs",
        "src/main_content/genre_content.rs",
        "src/main_content/genres.rs",
        "src/main_content/mod.rs",
        "src/main_content/playlist_content.rs",
        "src/main_content/playlists.rs",
        "src/main_content/queue.rs",
        "src/main_content/search.rs",
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
    target_compatible_with = ["@platforms//os:android"],
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
        "@crates//:image",
        "@crates//:jni",
        "@crates//:platform-dirs",
        "@crates//:reqwest",
        "@crates//:slint",
        "@crates//:tokio",
        "@crates//:tracing",
        "@crates//:tracing-appender",
        "@crates//:tracing-subscriber",
        "@crates//:uuid",
    ],
)
