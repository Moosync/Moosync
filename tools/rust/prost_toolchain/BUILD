load("@rules_rust//rust:defs.bzl", "rust_library_group")
load("@rules_rust_prost//:defs.bzl", "rust_prost_toolchain")

PROST_OPTS = [
    "type_attribute=.=#[derive(serde::Serialize\\,serde::Deserialize)]",
    "type_attribute=.moosync.types.ThemeItem=#[serde(default)]",
    "type_attribute=.moosync.types.ThemeDetails=#[serde(default)]",
    "type_attribute=.=#[serde(rename_all=\"camelCase\")]",
]


rust_library_group(
    name = "prost_runtime_default",
    deps = [
        "@crates//:prost",
        "@crates//:serde",
    ],
)

rust_prost_toolchain(
    name = "prost_toolchain_impl_default",
    prost_opts = PROST_OPTS,
    prost_plugin = "@crates//:protoc-gen-prost__protoc-gen-prost",
    prost_runtime = ":prost_runtime_default",
    prost_types = "@crates//:prost-types",
    tonic_plugin_flag = "",
    visibility = ["//visibility:public"],
)

toolchain(
    name = "prost_toolchain_default",
    toolchain = ":prost_toolchain_impl_default",
    toolchain_type = "@rules_rust_prost//:toolchain_type",
    visibility = ["//visibility:public"],
)
