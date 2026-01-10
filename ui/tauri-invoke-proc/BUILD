load("@rules_rust//rust:defs.bzl", "rust_library", "rust_proc_macro")

rust_proc_macro(
    name = "tauri_invoke_proc",
    srcs = [
        "src/common.rs",
        "src/core.rs",
        "src/lib.rs",
    ],
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
    crate_root = "src/ui.rs",
    visibility = ["//visibility:public"],
    deps = [
        "@wasmCrates//:lazy_static",
        "@wasmCrates//:once_cell",
        "@wasmCrates//:proc-macro2",
        "@wasmCrates//:quote",
        "@wasmCrates//:regex",
        "@wasmCrates//:serde",
        "@wasmCrates//:serde_json",
        "@wasmCrates//:syn",
    ],
)
