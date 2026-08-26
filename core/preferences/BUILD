load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")

rust_library(
    name = "preferences",
    srcs = [
        "src/context/keyring_context.rs",
        "src/context/keyring_context_test.rs",
        "src/context/mod.rs",
        "src/context/mod_test.rs",
        "src/error.rs",
        "src/keys.rs",
        "src/keys_test.rs",
        "src/lib.rs",
        "src/lib_test_smoke.rs",
        "src/preferences.rs",
        "src/preferences_test.rs",
    ],
    crate_root = "src/lib.rs",
    proc_macro_deps = [
        "//core/plugin_macro",
    ],
    visibility = ["//visibility:public"],
    deps = [
        "//core/types",
        "//core/types/protos:extensions_rs",
        "@crates//:chacha20poly1305",
        "@crates//:crossbeam-channel",
        "@crates//:hex",
        "@crates//:json_dotpath",
        "@crates//:keyring",
        "@crates//:mockall",
        "@crates//:serde",
        "@crates//:serde_json",
        "@crates//:tempdir",
        "@crates//:thiserror",
        "@crates//:tracing",
        "@crates//:whoami",
    ],
)

rust_test(
    name = "preferences_test",
    crate = ":preferences",
    deps = [
        "@crates//:mockall",
        "@crates//:uuid",
    ],
)
