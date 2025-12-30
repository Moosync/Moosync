load("@rules_rust//rust:defs.bzl", "rust_binary")
load(":cargo_build.bzl", "cargo_build_")
load(":cargo_gen.bzl", "export_cargo_package")

def rust_binary_with_export(name, srcs, deps = [], edition = "2024", visibility = None, **kwargs):
    internal_name = name + "_internal"
    export_name = name + "_export"

    export_cargo_package(
        name = export_name,
        target = ":" + internal_name,
        visibility = ["//visibility:private"],
    )
    rust_binary(
        name = internal_name,
        srcs = srcs,
        deps = deps,
        edition = edition,
        **kwargs
    )

    native.alias(
        visibility = visibility,
        name = name,
        actual = select({
            "@platforms//os:windows": ":" + export_name,
            "//conditions:default": ":" + internal_name,
        }),
    )

cargo_build = cargo_build_
