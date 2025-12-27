load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")

rust_library(
    name = "preferences",
    srcs = [
        "src/lib.rs",
        "src/preferences.rs",
        "src/tests.rs",
        "src/context/mod.rs",
    ],
    crate_root = "src/lib.rs",
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
        "@crates//:mockall",
    ],
)

rust_test(
    name = "preferences_test",
    crate = ":preferences",
    deps = [
        "@crates//:uuid",
        "@crates//:mockall",
    ],
)
