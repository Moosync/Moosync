package(default_visibility = ["//visibility:public"])

sh_binary(
    name = "format",
    srcs = ["format.sh"],
    args = [
        "$(rlocationpath @rules_rust//tools/upstream_wrapper:rustfmt)",
        "$(rlocationpath @crates//:slint-lsp__slint-lsp)",
    ],
    data = [
        "@rules_rust//tools/upstream_wrapper:rustfmt",
        "@crates//:slint-lsp__slint-lsp",
    ],
    deps = ["@bazel_tools//tools/bash/runfiles"],
)
