load("@rules_rust//cargo:defs.bzl", "cargo_build_script")
load("@rules_rust//rust:defs.bzl", "rust_library")
load("//:version.bzl", "VERSION")
load("//tools:defs.bzl", "expand_template")
load("//tools/rust:defs.bzl", "cargo_build", "rust_binary_with_export")

filegroup(
    name = "resources",
    srcs = glob([
        "capabilities/**/*",
    ]),
)

expand_template(
    name = "tauri_config",
    data = [
        "//package:icons",
        "//ui:moosync_ui",
    ],
    out_name = "tauri.conf.json",
    substitutions = {
        # Relative from tauri build script but we need to escape to exec root
        "@DIST@": "../../../../$(execpath //ui:moosync_ui)",
        "@DESKTOP@": "",
        "@ICONS@": "../../../../$(execpath //package:icons)",
    },
    template = "tauri.conf.json",
)

rust_library(
    name = "app_lib",
    srcs = [
        "src/db/mod.rs",
        "src/extensions/mod.rs",
        "src/extensions/request_handler.rs",
        "src/lib.rs",
        "src/librespot/mod.rs",
        "src/logger/mod.rs",
        "src/lyrics/mod.rs",
        "src/macros.rs",
        "src/mobile_player/mod.rs",
        "src/mpris/mod.rs",
        "src/oauth/handler.rs",
        "src/oauth/mod.rs",
        "src/preference_holder/mod.rs",
        "src/providers/extension.rs",
        "src/providers/handler.rs",
        "src/providers/mod.rs",
        "src/rodio/mod.rs",
        "src/scanner/mod.rs",
        "src/themes/mod.rs",
        "src/updater/desktop.rs",
        "src/updater/mobile.rs",
        "src/updater/mod.rs",
        "src/window/handler.rs",
        "src/window/mod.rs",
    ],
    compile_data = [
        ":resources",
        "//package:icons",
        "//ui:moosync_ui",
    ],
    data = [
        ":tauri_config",
    ],
    proc_macro_deps = [
        "@crates//:async-trait",
        "//ui/tauri-invoke-proc:tauri_invoke_proc",
    ],
    rustc_env = {
        "APP_VERSION": VERSION,
        "TAURI_CONF_JSON": "$(execpath :tauri_config)",
    },
    visibility = ["//visibility:public"],
    deps = [
        ":build_script",
        "//core/database",
        "//core/extensions",
        "//core/file_scanner",
        "//core/librespot",
        "//core/lyrics",
        "//core/mpris",
        "//core/preferences",
        "//core/rodio_player",
        "//core/themes",
        "//core/types",
        "//lib/tauri-plugin-audioplayer",
        "//lib/tauri-plugin-file-scanner",
        "//lib/tauri-plugin-self-update",
        "@crates//:chrono",
        "@crates//:clap",
        "@crates//:futures",
        "@crates//:oauth2",
        "@crates//:regex",
        "@crates//:reqwest",
        "@crates//:rspotify",
        "@crates//:rustls",
        "@crates//:serde",
        "@crates//:serde_json",
        "@crates//:tauri",
        "@crates//:tauri-plugin-autostart",
        "@crates//:tauri-plugin-deep-link",
        "@crates//:tauri-plugin-dialog",
        "@crates//:tauri-plugin-opener",
        "@crates//:tauri-plugin-single-instance",
        "@crates//:tauri-plugin-updater",
        "@crates//:tauri-utils",
        "@crates//:tokio",
        "@crates//:tracing",
        "@crates//:tracing-appender",
        "@crates//:tracing-subscriber",
        "@crates//:url",
        "@crates//:uuid",
    ] + select({
        "@platforms//os:linux": [
            "@crates//:webkit2gtk",
        ],
        "//conditions:default": [],
    }),
)

rust_binary_with_export(
    name = "moosync_internal",
    srcs = ["src/main.rs"],
    binary_name = "moosync",
    rustc_flags = select({
        "//:release": [
            "-C",
            "codegen-units=1",
            "-C",
            "opt-level=3",
            "-C",
            "strip=symbols",
        ],
        "//conditions:default": [],
    }),
    visibility = ["//visibility:private"],
    deps = [
        ":app_lib",
        "@crates//:tracing",
    ],
)

cargo_build_script(
    name = "build_script",
    srcs = ["build.rs"],
    build_script_env = {
        "DEP_TAURI_DEV": "false",
    },
    crate_root = "build.rs",
    data = [
        "Cargo.toml",
        ":tauri_config",
    ] + glob(["src/**/*"]),
    visibility = ["//visibility:public"],
    deps = [
        "@crates//:tauri-build",
    ],
)

cargo_build(
    name = "moosync_cargo",
    srcs = [":moosync_internal_export"],
    # app-lib.exe builds with an icon
    binary_name = "app-lib.exe",
    data = [
        "//external/ffmpeg:gen_dir",
        "//external/openssl",
        "//external/openssl:gen_dir",
    ],
    env = {
        "FFMPEG_INCLUDE_DIR": "$ROOT/$(execpath //external/ffmpeg:gen_dir)/include",
        "FFMPEG_LIBS_DIR": "$ROOT/$(execpath //external/ffmpeg:gen_dir)/lib",
        "FFMPEG_LINK_MODE": "static",
        "DEP_TAURI_DEV": "false",
        "OPENSSL_NO_VENDOR": "1",
        "OPENSSL_LIB_DIR": "$ROOT/$(execpath //external/openssl:gen_dir)/lib",
        "OPENSSL_INCLUDE_DIR": "$ROOT/$(execpath //external/openssl:gen_dir)/include",
        "OPENSSL_STATIC": "1",
    },
    out_name = "moosync_cargo.exe",
    rustflags = [
        "-C",
        "link-arg=User32.lib",
        "-C",
        "link-arg=Gdi32.lib",
        "-C",
        "link-arg=Crypt32.lib",
        "-C",
        "link-arg=Advapi32.lib",
        "-C",
        "link-arg=Ws2_32.lib",
        "-C",
        "link-arg=Bcrypt.lib",
        "-C",
        "link-arg=libssl.lib",
        "-C",
        "link-arg=libcrypto.lib",
        "-C",
        "link-arg=/LIBPATH:$ROOT/$(execpath //external/openssl:gen_dir)/lib",
    ],
    target_compatible_with = ["@platforms//os:windows"],
    visibility = ["//visibility:private"],
)

alias(
    name = "moosync",
    actual = select({
        "@platforms//os:windows": ":moosync_cargo",
        "//conditions:default": ":moosync_internal",
    }),
    visibility = ["//visibility:public"],
)
