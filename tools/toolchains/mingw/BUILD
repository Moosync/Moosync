load("@rules_foreign_cc//toolchains/native_tools:native_tools_toolchain.bzl", "native_tool_toolchain")

package(default_visibility = ["//visibility:public"])

config_setting(
    name = "windows_mingw",
    constraint_values = [
        "@platforms//os:windows",
    ],
    flag_values = {
        "@bazel_tools//tools/cpp:compiler": "mingw-gcc",
    },
)

# Custom host make toolchain for rules_foreign_cc when cross compiling from Linux to Windows
native_tool_toolchain(
    name = "custom_make",
    path = "make",
)

toolchain(
    name = "custom_make_toolchain",
    exec_compatible_with = [
        "@platforms//os:linux",
        "@platforms//cpu:x86_64",
    ],
    target_compatible_with = [
        "@platforms//os:windows",
    ],
    toolchain = ":custom_make",
    toolchain_type = "@rules_foreign_cc//toolchains:make_toolchain",
)

# Custom host m4 toolchain for rules_foreign_cc when cross compiling from Linux to Windows
native_tool_toolchain(
    name = "custom_m4",
    path = "m4",
)

toolchain(
    name = "custom_m4_toolchain",
    exec_compatible_with = [
        "@platforms//os:linux",
        "@platforms//cpu:x86_64",
    ],
    target_compatible_with = [
        "@platforms//os:windows",
    ],
    toolchain = ":custom_m4",
    toolchain_type = "@rules_foreign_cc//toolchains:m4_toolchain",
)
