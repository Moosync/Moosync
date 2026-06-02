load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_proc_macro")

rust_proc_macro(
    name = "pref_macro",
    srcs = ["src/lib.rs"],
    edition = "2021",
    visibility = ["//visibility:public"],
    deps = [
        "@crates//:syn",
        "@crates//:quote",
        "@crates//:proc-macro2",
    ],
)

rust_binary(
    name = "print_expanded",
    srcs = ["src/main.rs"],
    edition = "2021",
    visibility = ["//visibility:public"],
    deps = [
        "@crates//:quote",
        "@crates//:proc-macro2",
    ],
)
