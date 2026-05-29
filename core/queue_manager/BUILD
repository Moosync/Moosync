load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")

rust_library(
    name = "queue_manager",
    srcs = [
        "src/lib.rs",
        "src/test.rs",
    ],
    crate_root = "src/lib.rs",
    visibility = ["//visibility:public"],
    deps = [
        "//core/types",
        "//core/types/protos:songs_rs",
    ],
)

rust_test(
    name = "queue_manager_test",
    crate = ":queue_manager",
)
