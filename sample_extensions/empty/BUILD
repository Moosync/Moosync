load("@extensions_sdk//:defs.bzl", "rust_extension")

package(default_visibility = ["//visibility:public"])

rust_extension(
    name = "sample_extension",
    package_name = "rs.sample.empty",
    srcs = ["src/lib.rs"],
    display_name = "Empty Sample Extension (Rust)",
    version = "1.0.0",
)

alias(
    name = "empty",
    actual = ":sample_extension",
)
