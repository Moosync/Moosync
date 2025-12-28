load("@rules_pkg//pkg:mappings.bzl", "pkg_files")
load("@rules_pkg//pkg:tar.bzl", "pkg_tar")
load("//tools:defs.bzl", "dirgroup")

package(default_visibility = [
    "//package:__subpackages__",
    "//tauri:__subpackages__",
])

dirgroup(
    name = "icons",
    srcs = glob([
        "icons/**",
    ]),
    visibility = ["//visibility:public"],
)

pkg_files(
    name = "moosync_icon_mapping",
    srcs = [
        "icons/128x128.png",
        "icons/32x32.png",
        "icons/icon.png",
    ],
    renames = {
        "icons/32x32.png": "/usr/share/icons/hicolor/32x32/apps/moosync.png",
        "icons/128x128.png": "/usr/share/icons/hicolor/128x128/apps/moosync.png",
        "icons/icon.png": "/usr/share/pixmaps/moosync.png",
    },
)

pkg_tar(
    name = "moosync_icons_tar",
    srcs = [":moosync_icon_mapping"],
    mode = "0644",
)

filegroup(
    name = "default_icon",
    srcs = ["icons/icon.png"],
)
