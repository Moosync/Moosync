load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")

filegroup(
    name = "canvaz_proto",
    srcs = ["src/canvaz.proto"],
)

rust_library(
    name = "librespot",
    srcs = [
        "src/canvaz.rs",
        "src/lib.rs",
        "src/player.rs",
        "src/spirc.rs",
        "src/tests.rs",
        "src/utils.rs",
    ],
    crate_root = "src/lib.rs",
    visibility = ["//visibility:public"],
    deps = [
        "//core/types",
        "@crates//:env_logger",
        "@crates//:futures",
        "@crates//:futures-util",
        "@crates//:librespot",
        "@crates//:protobuf",
        "@crates//:regex",
        "@crates//:reqwest",
        "@crates//:serde",
        "@crates//:tokio",
        "@crates//:tracing",
        "@crates//:url",
        "@crates//:uuid",
    ],
)

rust_test(
    name = "librespot_test",
    crate = ":librespot",
)
