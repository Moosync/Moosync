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
        "src/error.rs",
        "src/generic.rs",
        "src/lib.rs",
        "src/mux_player.rs",
        "src/queue.rs",
        "src/rodio/decoder.rs",
        "src/rodio/mod.rs",
        "src/source.rs",
        "src/test.rs",
    ],
    crate_root = "src/lib.rs",
    data = ["//external/ffmpeg:gen_dir"],
    rustc_env = {
        "FFMPEG_INCLUDE_DIR": "$(execpath //external/ffmpeg:gen_dir)/include",
        "FFMPEG_LIB_DIR": "$(execpath //external/ffmpeg:gen_dir)/lib",
    },
    visibility = ["//visibility:public"],
    deps = [
        ":build_script",
        "//core/spotify_player",
        "//core/types",
        "//core/types/protos:extensions_rs",
        "//core/types/protos:songs_rs",
        "//external/ffmpeg",
        "@crates//:rodio",
        "@crates//:rsmpeg",
        "@crates//:rusty_ffmpeg",
        "@crates//:thiserror",
        "@crates//:tokio",
        "@crates//:tracing",
    ],
)

rust_test(
    name = "player_test",
    crate = ":player",
)
