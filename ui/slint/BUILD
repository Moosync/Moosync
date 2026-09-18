load("@rules_rust//cargo:defs.bzl", "cargo_build_script")
load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_library", "rust_shared_library", "rust_test")

SRC_LIST = [
    "src/accounts.rs",
    "src/accounts_test.rs",
    "src/error.rs",
    "src/lib.rs",
    "src/lib_test.rs",
    "src/main_content/album_content.rs",
    "src/main_content/album_content_test.rs",
    "src/main_content/albums.rs",
    "src/main_content/albums_test.rs",
    "src/main_content/all_songs.rs",
    "src/main_content/all_songs_test.rs",
    "src/main_content/artist_content.rs",
    "src/main_content/artist_content_test.rs",
    "src/main_content/artists.rs",
    "src/main_content/artists_test.rs",
    "src/main_content/explore.rs",
    "src/main_content/explore_test.rs",
    "src/main_content/genre_content.rs",
    "src/main_content/genre_content_test.rs",
    "src/main_content/genres.rs",
    "src/main_content/genres_test.rs",
    "src/main_content/mod.rs",
    "src/main_content/playlist_content.rs",
    "src/main_content/playlist_content_test.rs",
    "src/main_content/playlists.rs",
    "src/main_content/playlists_test.rs",
    "src/main_content/queue.rs",
    "src/main_content/queue_test.rs",
    "src/main_content/search.rs",
    "src/main_content/search_test.rs",
    "src/pages.rs",
    "src/pages_test.rs",
    "src/settings/extensions.rs",
    "src/settings/extensions_test.rs",
    "src/settings/mod.rs",
    "src/settings/paths.rs",
    "src/settings/paths_test.rs",
    "src/settings/system.rs",
    "src/settings/system_test.rs",
    "src/settings/themes.rs",
    "src/settings/themes_test.rs",
    "src/settings/utils.rs",
    "src/settings/utils_test.rs",
    "src/test_utils.rs",
    "src/utils/context_menu.rs",
    "src/utils/context_menu_test.rs",
    "src/utils/entity_content.rs",
    "src/utils/entity_content_test.rs",
    "src/utils/entity_list.rs",
    "src/utils/entity_list_test.rs",
    "src/utils/filter_sort.rs",
    "src/utils/filter_sort_test.rs",
    "src/utils/image.rs",
    "src/utils/image_test.rs",
    "src/utils/lazy_model.rs",
    "src/utils/lazy_model_test.rs",
    "src/utils/mod.rs",
    "src/utils/models.rs",
    "src/utils/models_test.rs",
    "src/utils/navigation.rs",
    "src/utils/navigation_test.rs",
    "src/utils/parsers.rs",
    "src/utils/parsers_test.rs",
    "src/utils/queue.rs",
    "src/utils/queue_test.rs",
    "src/utils/selection.rs",
    "src/utils/selection_test.rs",
    "src/utils/validation.rs",
    "src/utils/validation_test.rs",
    "src/window_info.rs",
    "src/window_info_test.rs",
]

rust_test(
    name = "slint_unit_test",
    size = "medium",
    crate = ":slint_app_lib",
    deps = [
        "@crates//:assertables",
        "@crates//:tracing-test",
    ],
)

rust_test(
    name = "slint_navigation_test",
    size = "large",
    srcs = ["src/integration_tests/navigation_test.rs"],
    deps = [
        ":slint_app_lib",
        "//core/state_manager",
        "//core/types/protos:songs_rs",
        "@crates//:i-slint-backend-testing",
        "@crates//:slint",
        "@crates//:tokio",
        "@crates//:tracing",
        "@crates//:tracing-test",
    ],
)

rust_test(
    name = "slint_views_test",
    size = "large",
    srcs = ["src/integration_tests/views_test.rs"],
    deps = [
        ":slint_app_lib",
        "//core/state_manager",
        "//core/types/protos:songs_rs",
        "@crates//:i-slint-backend-testing",
        "@crates//:slint",
        "@crates//:tokio",
        "@crates//:tracing",
        "@crates//:tracing-test",
    ],
)

rust_test(
    name = "slint_playback_test",
    size = "large",
    srcs = ["src/integration_tests/playback_test.rs"],
    deps = [
        ":slint_app_lib",
        "//core/state_manager",
        "@crates//:i-slint-backend-testing",
        "@crates//:slint",
        "@crates//:tokio",
        "@crates//:tracing",
        "@crates//:tracing-test",
    ],
)

rust_test(
    name = "slint_search_test",
    size = "large",
    srcs = ["src/integration_tests/search_test.rs"],
    data = [
        "@moosync_ext//sample_extensions/rs:sample_extension",
    ],
    deps = [
        ":slint_app_lib",
        "//core/state_manager",
        "//core/types/protos:songs_rs",
        "@crates//:i-slint-backend-testing",
        "@crates//:slint",
        "@crates//:tokio",
        "@crates//:tracing",
        "@crates//:tracing-test",
    ],
)

rust_test(
    name = "slint_queue_test",
    size = "large",
    srcs = ["src/integration_tests/queue_test.rs"],
    deps = [
        ":slint_app_lib",
        "//core/state_manager",
        "@crates//:i-slint-backend-testing",
        "@crates//:slint",
        "@crates//:tokio",
        "@crates//:tracing",
        "@crates//:tracing-test",
    ],
)

rust_test(
    name = "slint_settings_test",
    size = "large",
    srcs = ["src/integration_tests/settings_test.rs"],
    deps = [
        ":slint_app_lib",
        "//core/preferences",
        "//core/state_manager",
        "//core/types/protos:preferences_rs",
        "@crates//:i-slint-backend-testing",
        "@crates//:slint",
        "@crates//:tokio",
        "@crates//:tracing",
        "@crates//:tracing-test",
    ],
)

rust_test(
    name = "slint_accounts_test",
    size = "large",
    srcs = ["src/integration_tests/accounts_test.rs"],
    data = [
        "@moosync_ext//sample_extensions/rs:sample_extension",
    ],
    deps = [
        ":slint_app_lib",
        "//core/state_manager",
        "@crates//:i-slint-backend-testing",
        "@crates//:slint",
        "@crates//:tokio",
        "@crates//:tracing",
        "@crates//:tracing-test",
    ],
)

rust_test(
    name = "slint_extensions_test",
    size = "large",
    srcs = ["src/integration_tests/extensions_test.rs"],
    data = [
        "@moosync_ext//sample_extensions/rs:sample_extension",
    ],
    deps = [
        ":slint_app_lib",
        "//core/preferences",
        "//core/state_manager",
        "//core/types/protos:songs_rs",
        "@crates//:i-slint-backend-testing",
        "@crates//:slint",
        "@crates//:tokio",
        "@crates//:tracing",
        "@crates//:tracing-test",
    ],
)

test_suite(
    name = "slint_tests",
    tests = [
        ":slint_accounts_test",
        ":slint_extensions_test",
        ":slint_navigation_test",
        ":slint_playback_test",
        ":slint_queue_test",
        ":slint_search_test",
        ":slint_settings_test",
        ":slint_unit_test",
        ":slint_views_test",
    ],
)

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
        "CARGO_PKG_NAME": "slint_app",
        "CARGO_PKG_VERSION": "1.18.0",
        "CARGO_PKG_VERSION_MAJOR": "1",
        "CARGO_PKG_VERSION_MINOR": "18",
        "CARGO_PKG_VERSION_PATCH": "0",
        "SLINT_ENABLE_EXPERIMENTAL_FEATURES": "1",
        "SLINT_EMIT_DEBUG_INFO": "1",
    },
    data = glob([
        "locales/**/*.po",
        "src/**/*.slint",
        "src/**/*.svg",
    ]),
    deps = ["@crates//:slint-build"],
)

rust_library(
    name = "slint_app_lib",
    srcs = SRC_LIST,
    compile_data = glob([
        "locales/**/*.po",
        "src/**/*.slint",
        "src/**/*.svg",
    ]),
    crate_name = "slint_app",
    edition = "2021",
    proc_macro_deps = [
        "//core/theme_macro",
        "@crates//:async-trait",
    ],
    deps = [
        ":build_script",
        "//core/database",
        "//core/extensions",
        "//core/player",
        "//core/preferences",
        "//core/state_manager",
        "//core/themes",
        "//core/types",
        "//core/types/protos:extensions_rs",
        "//core/types/protos:preferences_rs",
        "//core/types/protos:songs_rs",
        "//core/types/protos:themes_rs",
        "//core/types/protos:ui_rs",
        "@crates//:csscolorparser",
        "@crates//:futures",
        "@crates//:i-slint-backend-testing",
        "@crates//:image",
        "@crates//:open",
        "@crates//:platform-dirs",
        "@crates//:prost-types",
        "@crates//:regex",
        "@crates//:reqwest",
        "@crates//:rstest",
        "@crates//:serde_json",
        "@crates//:slint",
        "@crates//:tempdir",
        "@crates//:thiserror",
        "@crates//:tokio",
        "@crates//:tracing",
        "@crates//:tracing-appender",
        "@crates//:tracing-subscriber",
        "@crates//:uuid",
    ] + select({
        "@platforms//os:android": [],
        "//conditions:default": [
            "@crates//:rfd",
            "@crates//:sysuri",
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
    srcs = SRC_LIST,
    compile_data = glob([
        "locales/**/*.po",
        "src/**/*.slint",
        "src/**/*.svg",
    ]),
    crate_name = "slint_app",
    edition = "2021",
    proc_macro_deps = [
        "//core/theme_macro",
    ],
    target_compatible_with = ["@platforms//os:android"],
    deps = [
        ":build_script",
        "//core/database",
        "//core/extensions",
        "//core/player",
        "//core/preferences",
        "//core/state_manager",
        "//core/themes",
        "//core/types",
        "//core/types/protos:extensions_rs",
        "//core/types/protos:songs_rs",
        "//core/types/protos:themes_rs",
        "//core/types/protos:ui_rs",
        "@crates//:futures",
        "@crates//:image",
        "@crates//:jni",
        "@crates//:platform-dirs",
        "@crates//:prost-types",
        "@crates//:regex",
        "@crates//:reqwest",
        "@crates//:serde_json",
        "@crates//:slint",
        "@crates//:thiserror",
        "@crates//:tokio",
        "@crates//:tracing",
        "@crates//:tracing-appender",
        "@crates//:tracing-subscriber",
        "@crates//:uuid",
    ],
)
