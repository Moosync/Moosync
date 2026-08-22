load("@rules_rust//cargo:defs.bzl", "cargo_build_script")
load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")

cargo_build_script(
    name = "build_script",
    srcs = ["build.rs"],
    crate_root = "build.rs",
    data = ["//external/ffmpeg:gen_dir"],
    visibility = ["//visibility:public"],
)

rust_library(
    name = "player",
    srcs = [
        "src/audio_source.rs",
        "src/audio_source_test.rs",
        "src/error.rs",
        "src/generic.rs",
        "src/lib.rs",
        "src/lib_test.rs",
        "src/lib_test_smoke.rs",
        "src/mux_player.rs",
        "src/mux_player_test.rs",
        "src/rodio/decoder.rs",
        "src/rodio/decoder_test.rs",
        "src/rodio/mod.rs",
        "src/rodio/mod_test.rs",
        "src/rodio/pulse_monitor.rs",
        "src/rodio/pulse_monitor_test.rs",
        "src/source.rs",
        "src/source_test.rs",
    ],
    crate_root = "src/lib.rs",
    data = ["//external/ffmpeg:gen_dir"],
    proc_macro_deps = [
        "//core/plugin_macro",
    ],
    rustc_env = {
        "FFMPEG_INCLUDE_DIR": "$(execpath //external/ffmpeg:gen_dir)/include",
        "FFMPEG_LIB_DIR": "$(execpath //external/ffmpeg:gen_dir)/lib",
    },
    rustc_flags = select({
        "@platforms//os:windows": [],
        "//conditions:default": ["-lz"],
    }),
    visibility = ["//visibility:public"],
    deps = [
        ":build_script",
        "//core/types",
        "//core/types/protos:extensions_rs",
        "//core/types/protos:songs_rs",
        "//external/ffmpeg",
        "@crates//:rand",
        "@crates//:rodio",
        "@crates//:rsmpeg",
        "@crates//:rusty_ffmpeg",
        "@crates//:thiserror",
        "@crates//:tempdir",
        "@crates//:tokio",
        "@crates//:tracing",
    ] + select({
        "@platforms//os:linux": ["@crates//:libpulse-binding"],
        "//conditions:default": [],
    }),
)

rust_test(
    name = "player_test",
    crate = ":player",
    data = [
        "src/rodio/test_data/LRMonoPhase4.mp3",
        "src/rodio/test_data/ff-16b-2c-44100hz.mp3",
    ],
    rustc_flags = select({
        "@platforms//os:windows": [],
        "//conditions:default": ["-lz"],
    }),
)
