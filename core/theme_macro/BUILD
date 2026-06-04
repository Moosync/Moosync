load("@rules_rust//rust:defs.bzl", "rust_proc_macro")

rust_proc_macro(
    name = "theme_macro",
    srcs = ["src/lib.rs"],
    crate_root = "src/lib.rs",
    visibility = ["//visibility:public"],
    deps = [
        "@crates//:syn",
        "@crates//:quote",
        "@crates//:proc-macro2",
    ],
)
