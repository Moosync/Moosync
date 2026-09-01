load("@rules_cc//cc:cc_library.bzl", "cc_library")

exports_files(["rustfmt.toml"])

config_setting(
    name = "release",
    values = {"compilation_mode": "opt"},
    visibility = ["//visibility:public"],
)

config_setting(
    name = "android_x86_64_config",
    constraint_values = [
        "@platforms//os:android",
        "@platforms//cpu:x86_64",
    ],
    visibility = ["//visibility:public"],
)

config_setting(
    name = "android_arm64_config",
    constraint_values = [
        "@platforms//os:android",
        "@platforms//cpu:aarch64",
    ],
    visibility = ["//visibility:public"],
)

config_setting(
    name = "linux_x86_64_config",
    constraint_values = [
        "@platforms//os:linux",
        "@platforms//cpu:x86_64",
    ],
    visibility = ["//visibility:public"],
)

config_setting(
    name = "windows_x86_64_config",
    constraint_values = [
        "@platforms//os:windows",
        "@platforms//cpu:x86_64",
    ],
    visibility = ["//visibility:public"],
)

config_setting(
    name = "macos_x86_64_config",
    constraint_values = [
        "@platforms//os:macos",
        "@platforms//cpu:x86_64",
    ],
    visibility = ["//visibility:public"],
)

platform(
    name = "x86_64",
    constraint_values = [
        "@platforms//cpu:x86_64",
        "@platforms//os:android",
    ],
    flags = [
        "--android_platforms=//:x86_64",
    ],
    visibility = ["//visibility:public"],
)

platform(
    name = "arm64-v8a",
    constraint_values = [
        "@platforms//cpu:aarch64",
        "@platforms//os:android",
    ],
    flags = [
        "--android_platforms=//:arm64-v8a",
    ],
    visibility = ["//visibility:public"],
)

alias(
    name = "android_x86_64",
    actual = ":x86_64",
    visibility = ["//visibility:public"],
)

alias(
    name = "android_arm64",
    actual = ":arm64-v8a",
    visibility = ["//visibility:public"],
)

platform(
    name = "windows_x86_64_gnu",
    constraint_values = [
        "@platforms//cpu:x86_64",
        "@platforms//os:windows",
        "@rules_rust//rust/platform:gnu",
    ],
    visibility = ["//visibility:public"],
)

platform(
    name = "linux_x86_64",
    constraint_values = [
        "@platforms//cpu:x86_64",
        "@platforms//os:linux",
    ],
    visibility = ["//visibility:public"],
)

platform(
    name = "macos_x86_64",
    constraint_values = [
        "@platforms//cpu:x86_64",
        "@platforms//os:macos",
    ],
    visibility = ["//visibility:public"],
)

cc_library(
    name = "libwindows_0_42_2",
    srcs = ["@crates__windows_x86_64_gnu-0.42.2//:lib/libwindows.a"],
    visibility = ["//visibility:public"],
)

cc_library(
    name = "libwindows_0_52_0",
    srcs = ["@crates__windows_x86_64_gnu-0.52.6//:lib/libwindows.0.52.0.a"],
    visibility = ["//visibility:public"],
)

cc_library(
    name = "libwindows_0_53_0",
    srcs = ["@crates__windows_x86_64_gnu-0.53.1//:lib/libwindows.0.53.0.a"],
    visibility = ["//visibility:public"],
)

cc_library(
    name = "libwinapi_all",
    srcs = [
        "@crates__winapi-x86_64-pc-windows-gnu-0.4.0//:lib/libwinapi_advapi32.a",
        "@crates__winapi-x86_64-pc-windows-gnu-0.4.0//:lib/libwinapi_cfgmgr32.a",
        "@crates__winapi-x86_64-pc-windows-gnu-0.4.0//:lib/libwinapi_credui.a",
        "@crates__winapi-x86_64-pc-windows-gnu-0.4.0//:lib/libwinapi_gdi32.a",
        "@crates__winapi-x86_64-pc-windows-gnu-0.4.0//:lib/libwinapi_kernel32.a",
        "@crates__winapi-x86_64-pc-windows-gnu-0.4.0//:lib/libwinapi_msimg32.a",
        "@crates__winapi-x86_64-pc-windows-gnu-0.4.0//:lib/libwinapi_ntdll.a",
        "@crates__winapi-x86_64-pc-windows-gnu-0.4.0//:lib/libwinapi_ole32.a",
        "@crates__winapi-x86_64-pc-windows-gnu-0.4.0//:lib/libwinapi_opengl32.a",
        "@crates__winapi-x86_64-pc-windows-gnu-0.4.0//:lib/libwinapi_secur32.a",
        "@crates__winapi-x86_64-pc-windows-gnu-0.4.0//:lib/libwinapi_shell32.a",
        "@crates__winapi-x86_64-pc-windows-gnu-0.4.0//:lib/libwinapi_user32.a",
        "@crates__winapi-x86_64-pc-windows-gnu-0.4.0//:lib/libwinapi_winspool.a",
        "@crates__winapi-x86_64-pc-windows-gnu-0.4.0//:lib/libwinapi_ws2_32.a",
    ],
    visibility = ["//visibility:public"],
)

toolchain(
    name = "shell_android_x86_64_toolchain",
    exec_compatible_with = [
        "@platforms//os:linux",
        "@platforms//cpu:x86_64",
    ],
    target_compatible_with = [
        "@platforms//os:android",
        "@platforms//cpu:x86_64",
    ],
    toolchain = "@local_config_shell//:linux_sh",
    toolchain_type = "@rules_shell//shell:toolchain_type",
)

toolchain(
    name = "shell_android_arm64_toolchain",
    exec_compatible_with = [
        "@platforms//os:linux",
        "@platforms//cpu:x86_64",
    ],
    target_compatible_with = [
        "@platforms//os:android",
        "@platforms//cpu:aarch64",
    ],
    toolchain = "@local_config_shell//:linux_sh",
    toolchain_type = "@rules_shell//shell:toolchain_type",
)

filegroup(
    name = "ndk_sysroot",
    srcs = ["@androidndk//toolchains/llvm/prebuilt/linux-x86_64/sysroot:all_files"],
    visibility = ["//visibility:public"],
)

filegroup(
    name = "mingw_sysroot",
    srcs = ["@mingw_compiler_files//:all_files"],
    visibility = ["//visibility:public"],
)

test_suite(
    name = "core_tests",
    tests = [
        "//core/database:database_test",
        "//core/extensions:extensions_test",
        "//core/file_scanner:file_scanner_test",
        "//core/lyrics:lyrics_test",
        "//core/mpris:mpris_test",
        "//core/player:player_test",
        "//core/preferences:preferences_test",
        "//core/state_manager:state_manager_test",
        "//core/themes:themes_test",
        "//core/types:types_test",
        "//ui/pref_macro:pref_macro_test",
        "//ui/slint:slint_app_lib_test",
    ],
)
