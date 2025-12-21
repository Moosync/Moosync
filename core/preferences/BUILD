load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")

rust_library(
    name = "preferences",
    srcs = [
        "src/lib.rs",
        "src/preferences.rs",
        "src/test.rs",
    ],
    visibility = ["//visibility:public"],
    deps = [
        "//core/types",
        "@crates//:chacha20poly1305",
        "@crates//:crossbeam-channel",
        "@crates//:hex",
        "@crates//:json_dotpath",
        "@crates//:keyring",
        "@crates//:serde",
        "@crates//:serde_json",
        "@crates//:tracing",
        "@crates//:whoami",
    ],
)

rust_test(
    name = "preferences_test",
    crate = ":preferences",
)
