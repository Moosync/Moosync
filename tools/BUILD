load("@aspect_bazel_lib//lib:copy_to_directory.bzl", "copy_to_directory")
load("@rules_python//python:defs.bzl", "py_binary")
load("@tauri_crate//:defs.bzl", "TAURI_CRATE", "tauri_android_srcs")
load("//tools:settings.bzl", "target_os_setting")

package(default_visibility = ["//visibility:public"])

# Build setting for target OS - persists across platform transitions
target_os_setting(
    name = "target_os",
    build_setting_default = "linux",
)

# Config settings for selecting based on target OS
config_setting(
    name = "target_android",
    flag_values = {"//tools:target_os": "android"},
)

config_setting(
    name = "target_linux",
    flag_values = {"//tools:target_os": "linux"},
)

config_setting(
    name = "target_windows",
    flag_values = {"//tools:target_os": "windows"},
)

# Tauri Android sources from external crate - needed for plugin build scripts
# TAURI_CRATE is auto-discovered from @crates//:tauri alias
copy_to_directory(
    name = "tauri_android_sources",
    srcs = tauri_android_srcs(),
    include_external_repositories = [TAURI_CRATE],
)

py_binary(
    name = "replace_bin",
    srcs = ["replace.py"],
    main = "replace.py",
)

py_binary(
    name = "html_generator",
    srcs = ["gen_html.py"],
    main = "gen_html.py",
)
