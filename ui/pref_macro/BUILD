load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_proc_macro", "rust_test")

rust_proc_macro(
    name = "pref_macro",
    srcs = [
        "src/lib.rs",
        "src/shared.rs",
        "src/shared_test.rs",
    ],
    edition = "2021",
    visibility = ["//visibility:public"],
    deps = [
        "@crates//:proc-macro2",
        "@crates//:quote",
        "@crates//:syn",
        "@crates//:tracing",
    ],
)

rust_test(
    name = "pref_macro_test",
    crate = ":pref_macro",
    deps = [
        "@crates//:tracing",
    ],
)

rust_binary(
    name = "print_expanded",
    srcs = [
        "src/main.rs",
        "src/shared.rs",
    ],
    edition = "2021",
    visibility = ["//visibility:public"],
    deps = [
        "@crates//:proc-macro2",
        "@crates//:quote",
        "@crates//:syn",
    ],
)
