load("@rules_rust//rust:defs.bzl", "rust_proc_macro", "rust_test")

rust_proc_macro(
    name = "theme_macro",
    srcs = [
        "src/lib.rs",
        "src/lib_test.rs",
    ],
    crate_root = "src/lib.rs",
    visibility = ["//visibility:public"],
    deps = [
        "@crates//:proc-macro2",
        "@crates//:quote",
        "@crates//:syn",
    ],
)

rust_test(
    name = "theme_macro_test",
    crate = ":theme_macro",
    deps = [
        "@crates//:assertables",
        "@crates//:tracing",
        "@crates//:tracing-test",
    ],
)
