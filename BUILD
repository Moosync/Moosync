load("@rules_python//python:defs.bzl", "py_library")
load("@rules_rust_wasm_bindgen//:defs.bzl", "rust_wasm_bindgen_toolchain")

rust_wasm_bindgen_toolchain(
    name = "wasm_bindgen_toolchain_impl",
    wasm_bindgen_cli = "@bindeps//:wasm-bindgen-cli__wasm-bindgen",
)

py_library(
    name = "moosync_python_root",
    imports = ["."],
    visibility = ["//visibility:public"],
)

toolchain(
    name = "wasm_bindgen_toolchain",
    toolchain = "wasm_bindgen_toolchain_impl",
    toolchain_type = "@rules_rust_wasm_bindgen//:toolchain_type",
)

config_setting(
    name = "release",
    values = {"compilation_mode": "opt"},
    visibility = ["//visibility:public"],
)

platform(
    name = "android_x86_64",
    constraint_values = [
        "@platforms//cpu:x86_64",
        "@platforms//os:android",
    ],
    visibility = ["//visibility:public"],
)

platform(
    name = "android_arm64",
    constraint_values = [
        "@platforms//cpu:aarch64",
        "@platforms//os:android",
    ],
    visibility = ["//visibility:public"],
)

