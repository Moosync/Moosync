load("@rules_rust//cargo:defs.bzl", "cargo_build_script")
load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")

rust_library(
    name = "rodio_player",
    srcs = [
        "src/decoder.rs",
        "src/lib.rs",
        "src/test.rs",
    ],
    data = ["//external/ffmpeg:gen_dir"],
    rustc_env = {
        "FFMPEG_INCLUDE_DIR": "$(execpath //external/ffmpeg:gen_dir)/include",
        "FFMPEG_LIB_DIR": "$(execpath //external/ffmpeg:gen_dir)/lib",
    },
    visibility = ["//visibility:public"],
    deps = [
        ":build_script",
        "//core/types",
        "//external/ffmpeg",
        "@crates//:rodio",
        "@crates//:rsmpeg",
        "@crates//:rusty_ffmpeg",
        "@crates//:thiserror",
        "@crates//:tokio",
        "@crates//:tracing",
    ],
)

cargo_build_script(
    name = "build_script",
    srcs = ["build.rs"],
    build_script_env = {
        "FFMPEG_INCLUDE_DIR": "$(execpath //external/ffmpeg:gen_dir)/include",
        "FFMPEG_LIB_DIR": "$(execpath //external/ffmpeg:gen_dir)/lib",
    },
    crate_root = "build.rs",
    data = ["//external/ffmpeg:gen_dir"],
    visibility = ["//visibility:public"],
)

rust_test(
    name = "rodio_player_test",
    crate = ":rodio_player",
    deps = ["@crates//:rodio"],
)
