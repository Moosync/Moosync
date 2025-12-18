load("@rules_rust//rust:defs.bzl", "rust_library", "rust_proc_macro")

rust_proc_macro(
    name = "tauri_invoke_proc",
    srcs = [
        "src/common.rs",
        "src/core.rs",
        "src/lib.rs",
    ],
    compile_data = ["Cargo.toml"],
    crate_root = "src/lib.rs",
    visibility = ["//visibility:public"],
    deps = [
        "@crates//:lazy_static",
        "@crates//:once_cell",
        "@crates//:quote",
        "@crates//:regex",
        "@crates//:serde",
        "@crates//:serde_json",
        "@crates//:syn",
    ],
)

rust_library(
    name = "tauri_invoke_proc_ui",
    srcs = [
        "src/common.rs",
        "src/ui.rs",
    ],
    compile_data = ["Cargo.toml"],
    crate_root = "src/ui.rs",
    visibility = ["//visibility:public"],
    deps = [
        "@crates//:lazy_static",
        "@crates//:once_cell",
        "@crates//:proc-macro2",
        "@crates//:quote",
        "@crates//:regex",
        "@crates//:serde",
        "@crates//:serde_json",
        "@crates//:syn",
    ],
)
